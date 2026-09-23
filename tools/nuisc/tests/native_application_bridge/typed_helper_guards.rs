use super::*;

const SOURCE: &str = "mod cpu Main {
    struct Payload { flag: bool, tag: i32, value: i64, gain: f32, scale: f64 }
    struct Packet { payload: Payload }
    struct State { payload: Payload }
    @noinline fn packet(skip: bool, tag: i32, a: i64, b: i64, gain: f32, scale: f64) -> Packet {
        if skip { return Packet { payload: Payload { scale: scale, gain: gain, value: a, tag: tag, flag: skip } }; }
        return Packet { payload: Payload { scale: scale, gain: gain, value: a / b, tag: tag, flag: skip } };
    }
    @noinline fn relay(packet: Packet) -> Packet { return packet; }
    fn start(skip: bool, tag: i32, a: i64, b: i64, gain: f32, scale: f64) -> State {
        let value = relay(packet(skip, tag, a, b, gain, scale));
        return State { payload: value.payload };
    }
    fn step(state: State) -> State { return state; }
    fn stop(state: State) -> State { return state; }
    fn main() -> i64 { return 0; }
}";

#[test]
fn typed_helper_guards_preserve_lazy_traps_shared_entries_and_output_sentinels() {
    check_guards(SOURCE, 3, false);
}

#[test]
fn typed_local_rebinding_guards_preserve_lazy_traps_shared_entries_and_output_sentinels() {
    let source = SOURCE.replace(
        "if skip { return Packet { payload: Payload { scale: scale, gain: gain, value: a, tag: tag, flag: skip } }; }\n        return Packet { payload: Payload { scale: scale, gain: gain, value: a / b, tag: tag, flag: skip } };",
        "let result = Packet { payload: Payload { scale: scale, gain: gain, value: a, tag: tag, flag: skip } };
        if !skip { let result = Packet { payload: Payload { scale: result.payload.scale, gain: result.payload.gain, value: result.payload.value / b, tag: result.payload.tag, flag: result.payload.flag } }; }
        return result;",
    );
    assert_ne!(source, SOURCE);
    check_guards(&source, 5, false);
    let unused = source.replace(
        "return result;",
        "return Packet { payload: Payload { scale: scale, gain: gain, value: a, tag: tag, flag: skip } };",
    );
    check_guards(&unused, 5, true);
}

#[test]
fn typed_sparse_capture_guards_preserve_selected_failures_bits_and_entry_budgets() {
    let padding = (0..58)
        .map(|i| format!("p{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..58)
        .map(|i| format!("p{i}: {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut source = SOURCE.replace(
        "    struct Packet",
        &format!(
            "    struct Wide {{ payload: Payload, divisor: i64, {padding} }}\n    struct Packet"
        ),
    );
    let start = source.find("        if skip").unwrap();
    let end = source[start..].find("\n    }").unwrap() + start;
    source.replace_range(start..end, &format!("let wide = Wide {{ payload: Payload {{ flag: skip, tag: tag, value: a, gain: gain, scale: scale }}, divisor: b, {values} }};
        let result: Payload = if skip {{ wide.payload }} else {{
            Payload {{ flag: wide.payload.flag, tag: wide.payload.tag, value: wide.payload.value / wide.divisor, gain: wide.payload.gain, scale: wide.payload.scale }}
        }};
        return Packet {{ payload: result }};"));
    // This fixture keeps both guarded arms: root, packet, selection, two arms, relay.
    check_guards(&source, 6, false);
}

fn check_guards(source: &str, required: u64, unused: bool) {
    let project = Project::with_source(source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let registry = yir_verify::default_registry();
    for (skip, a, b, budget, result) in [
        (1, 7, 0, required, Some(7)),
        (0, 7, 2, required, Some(3)),
        (1, i64::MIN, -1, required, Some(i64::MIN)),
        (0, 7, 0, required, None),
        (0, i64::MIN, -1, required, None),
        (0, 10, 2, required - 1, None),
        (1, 7, 0, 0, None),
        (2, 7, 0, 0, None),
    ] {
        let bridge = yir_lower_llvm::native_session::emit_registered_with_work_limits(
            &compiled.yir,
            "counter",
            0,
            budget,
        )
        .unwrap();
        let input = [skip, -17, a, b, 0x7fc0_1234, i64::MIN];
        let words = result.map(|value| {
            vec![
                skip,
                -17,
                if unused { a } else { value },
                0x7fc0_1234,
                i64::MIN,
            ]
        });
        if let Some(words) = &words {
            let args = bridge.callbacks[0]
                .arguments
                .iter()
                .zip(input)
                .map(|(kind, word)| kind.unpack(word as u64).unwrap())
                .collect();
            let (reference, _) =
                ApplicationSession::open_registered(&compiled.yir, &registry, "counter", args)
                    .unwrap();
            assert_eq!(
                state_words(reference.state()),
                words.iter().map(|v| *v as u64).collect::<Vec<_>>()
            );
        }
        let mut llvm = bridge
            .llvm_ir
            .replacen(
                "define i64 @nuis_yir_entry()",
                "define i64 @unused_native_entry()",
                1,
            )
            .replace(
                "  call void @llvm.trap()",
                "  call void @probe_trap()\n  call void @llvm.trap()",
            );
        llvm.push_str(multi_execution::ALLOCATION_PROBE);
        llvm.push_str("\n@probe_out = internal global [5 x i64] [i64 -700, i64 -700, i64 -700, i64 -700, i64 -700], align 8\ndefine void @probe_words() {\n");
        for slot in 0..5 {
            llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr @probe_out, i64 {slot}\n  %v{slot} = load volatile i64, ptr %p{slot}\n  call void @nuis_debug_print_i64(i64 %v{slot})\n"));
        }
        for counter in ["probe_allocs", "probe_drops"] {
            llvm.push_str(&format!("  %{counter} = load i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter})\n"));
        }
        llvm.push_str("  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\ndefine void @probe_trap() {\n  call void @nuis_debug_print_i64(i64 73)\n  call void @probe_words()\n  ret void\n}\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [6 x i64], align 8\n");
        for (slot, word) in input.into_iter().enumerate() {
            llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {word}, ptr %p{slot}\n"));
        }
        llvm.push_str(&format!("  %status = call i32 @{}(ptr %args, i64 6, ptr @probe_out, i64 5)\n  %wide = zext i32 %status to i64\n  call void @nuis_debug_print_i64(i64 %wide)\n  call void @probe_words()\n  ret i64 0\n}}\n", bridge.callbacks[0].symbol));
        let artifact = nuisc::aot::write_and_link_with_source(
            &project.0.join("main.ns"),
            &project.0.join("out"),
            source,
            nuisc::aot::AotCompileProgram {
                ast: &compiled.ast,
                nir: &compiled.nir,
                yir: &compiled.yir,
                llvm_ir: Some(&llvm),
            },
            &nuisc::aot::host_cpu_build_target(),
        )
        .unwrap();
        let run = dynamic_loop_guard::run_bounded(
            std::path::Path::new(&artifact.binary_path),
            &project.0,
        );
        let trap = result.is_none() && skip != 2;
        assert_eq!(
            run.status.success(),
            !trap,
            "skip={skip} a={a} b={b} budget={budget}: {}\nstdout: {}\nhelper graph:\n{}",
            String::from_utf8_lossy(&run.stderr),
            String::from_utf8_lossy(&run.stdout),
            bridge
                .llvm_ir
                .lines()
                .filter(|line| {
                    (line.starts_with("define ") || line.contains("call "))
                        && line.contains("@nuis_fn_")
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
        #[cfg(unix)]
        if trap {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(run.status.signal(), Some(4 | 5)),
                "{:?}",
                run.status
            );
        }
        let mut expected = vec![if trap {
            73
        } else if skip == 2 {
            2
        } else {
            0
        }];
        expected.extend(words.unwrap_or_else(|| vec![-700; 5]));
        expected.extend([0, 0]);
        let actual = String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .map(|line| line.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "skip={skip} a={a} b={b} budget={budget}");
    }
}
