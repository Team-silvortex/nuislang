use super::*;
use typed_record_inputs::fixture;

#[test]
fn typed_record_inputs_reject_layout_drift_before_execution_or_publication() {
    let project = Project::with_source(&fixture::source(false));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for (old, new) in [("State{", "Other{"), ("f0:i64", "f0:bool")] {
        let mut module = compiled.yir.clone();
        let parameter = module
            .nodes
            .iter_mut()
            .find(|node| node.op.instruction == "param_value_struct")
            .unwrap();
        assert!(parameter.op.args[1].contains(old));
        parameter.op.args[1] = parameter.op.args[1].replace(old, new);
        assert!(emit_registered(&module, "counter").is_err());
        let registry = yir_verify::default_registry();
        let opened =
            ApplicationSession::open_registered(&module, &registry, "counter", vec![Value::Int(5)]);
        if new == "Other{" {
            assert!(
                opened.is_err(),
                "nominal declaration drift must fail verification"
            );
        } else {
            let (mut session, _) = opened.unwrap();
            let initial = state_words(session.state());
            let error = session.event(vec![]).unwrap_err();
            assert!(error.contains("does not match nominal layout"), "{error}");
            assert_eq!(state_words(session.state()), initial);
        }
    }
}

#[test]
fn typed_record_inputs_keep_lazy_traps_and_shared_entry_limits_atomic() {
    check_traps(fixture::source(false));
}

pub(super) fn check_traps(original: String) {
    check_limits(
        original,
        &[
            (-2, 64, 64, false),
            (4, 64, 64, false),
            (2, 64, 64, true),
            (4, 64, 1, true),
        ],
    );
}

pub(super) fn check_limits(original: String, cases: &[(i64, u64, u64, bool)]) {
    let source = original.replace("value.f0 + 1", "10 / (value.f0 - 2)");
    assert_ne!(source, original);
    let cases = cases
        .iter()
        .map(|&(seed, loops, entries, trap)| (seed, loops, entries, seed == 2, trap))
        .collect::<Vec<_>>();
    check_prepared(&source, &cases);
}

pub(super) fn check_prepared(source: &str, cases: &[(i64, u64, u64, bool, bool)]) {
    let project = Project::with_source(source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(compiled
        .yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "param_value_struct"));
    for &(seed, loop_budget, budget, reference_trap, trap) in cases {
        let registry = yir_verify::default_registry();
        let (mut reference, _) = ApplicationSession::open_registered(
            &compiled.yir,
            &registry,
            "counter",
            vec![Value::Int(seed)],
        )
        .unwrap();
        let initial = state_words(reference.state());
        if reference_trap {
            assert!(reference.event(vec![]).is_err());
            assert_eq!(state_words(reference.state()), initial);
        } else {
            reference.event(vec![]).unwrap();
        }
        let bridge = yir_lower_llvm::native_session::emit_registered_with_work_limits(
            &compiled.yir,
            "counter",
            loop_budget,
            budget,
        )
        .unwrap();
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
        let sentinels = vec!["i64 -700"; 64].join(", ");
        llvm.push_str(&format!("\n@probe_out = internal global [64 x i64] [{sentinels}], align 8\ndefine void @probe_words() {{\n"));
        for slot in 0..64 {
            llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr @probe_out, i64 {slot}\n  %v{slot} = load volatile i64, ptr %p{slot}\n  call void @nuis_debug_print_i64(i64 %v{slot})\n"));
        }
        for counter in ["probe_allocs", "probe_drops"] {
            llvm.push_str(&format!("  %{counter} = load i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter})\n"));
        }
        llvm.push_str("  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\ndefine void @probe_trap() {\n  call void @nuis_debug_print_i64(i64 73)\n  call void @probe_words()\n  ret void\n}\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [64 x i64], align 8\n");
        for (slot, word) in initial.iter().enumerate() {
            llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {}, ptr %p{slot}\n", *word as i64));
        }
        llvm.push_str(&format!("  %status = call i32 @{}(ptr %args, i64 64, ptr @probe_out, i64 64)\n  %wide = zext i32 %status to i64\n  call void @nuis_debug_print_i64(i64 %wide)\n  call void @probe_words()\n  ret i64 0\n}}\n", bridge.callbacks[1].symbol));
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
        assert_eq!(
            run.status.success(),
            !trap,
            "seed={seed}, budget={budget}: {}",
            String::from_utf8_lossy(&run.stderr)
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
        let mut expected = vec![if trap { 73 } else { 0 }];
        expected.extend(if trap {
            vec![-700; 64]
        } else {
            state_words(reference.state())
                .into_iter()
                .map(|word| word as i64)
                .collect()
        });
        expected.extend([0, 0]);
        let actual = String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .map(|line| line.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "seed={seed}, budget={budget}");
    }
}
