use super::*;
#[path = "../../src/lowering/buffer_loop_outline/conditional_returns_effect_typed_return_fixtures.rs"]
mod fixtures;
use fixtures::{source, MODES, TYPES};

pub(super) fn words(ty: &str) -> Vec<u64> {
    match ty {
        "i32" => [0, -1, i32::MIN, i32::MAX]
            .map(|n| n as i64 as u64)
            .to_vec(),
        "f32" => vec![
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            0x3fa0_0000,
            0x7fc0_1234,
            0xff80_0001,
        ],
        "f64" => vec![
            0,
            0x8000_0000_0000_0000,
            1,
            0x8000_0000_0000_0001,
            0x3ff4_0000_0000_0000,
            0x7ff8_0000_0000_1234,
            0xfff0_0000_0000_0001,
        ],
        _ => unreachable!(),
    }
}

pub(super) fn inputs(word: u64, fallback: u64) -> Vec<[u64; 8]> {
    vec![
        [0, 1, 1, 0, word, fallback, 0, 0],
        [1, 1, 1, 0, word, fallback, 2, 0],
        [1, 0, 1, 0, word, fallback, 0, (-2_i64) as u64],
        [1, 1, 0, 0, word, fallback, 2, 0],
        [1, 0, 0, 0, word, fallback, 0, 2],
        [1, 1, 1, 1, word, fallback, 0, 0],
        [1, 0, 0, 1, word, fallback, 0, 0],
        [0, 0, 0, 1, word, fallback, 0, 0],
    ]
}

pub(super) fn expected(ty: &str, mode: &str, input: [u64; 8]) -> (Vec<u64>, Vec<i64>, [i64; 3]) {
    let [outer, gate, nested, early, value, fallback, _, _] = input;
    let early = if mode.ends_with("-no-exit") { 0 } else { early };
    let mut prints = vec![99];
    let mut result = fallback;
    if outer == 0 {
        prints.push(77);
    } else if early != 0 {
        prints.push(72);
    } else {
        prints.extend([if gate != 0 { 70 } else { 71 }, 80]);
        if mode.starts_with("complete") || (mode.starts_with("partial") && nested != 0) {
            let selected = if gate != 0 { value } else { fallback };
            result = selected
                ^ match ty {
                    "f32" => 1 << 31,
                    "f64" => 1 << 63,
                    _ => 0,
                };
        } else {
            prints.push(77);
        }
    }
    let calls = [
        i64::from(outer != 0),
        i64::from(outer != 0 && early == 0),
        i64::from(outer != 0 && early == 0),
    ];
    let mut state = input.to_vec();
    state.push(result);
    (state, prints, calls)
}

pub(super) fn values(ty: &str, input: [u64; 8]) -> Vec<Value> {
    input
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            0..=3 => Value::Bool(word != 0),
            4..=5 => match ty {
                "i32" => Value::I32(word as i32),
                "f32" => Value::F32(f32::from_bits(word as u32)),
                _ => Value::F64(f64::from_bits(word)),
            },
            _ => Value::Int(word as i64),
        })
        .collect()
}

pub(super) fn trace(events: &[String], prints: &[i64], calls: [i64; 3]) {
    let actual = events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .collect::<Vec<_>>();
    assert_eq!(actual.len(), prints.len(), "{events:?}");
    for (event, printed) in actual.iter().zip(prints) {
        assert!(
            event.ends_with(&format!(": {printed}"))
                || event.ends_with(&format!("then print {printed}")),
            "{event}"
        );
    }
    for (name, expected) in ["choose", "observe", "finish"].into_iter().zip(calls) {
        assert_eq!(
            events
                .iter()
                .filter(|e| e.contains(&format!("] {name}(")))
                .count(),
            expected as usize,
            "{name}: {events:?}"
        );
    }
}

pub(super) fn reordered(module: &YirModule, reversed: bool) -> YirModule {
    let mut module = module.clone();
    if reversed {
        module.nodes.reverse();
        module.edges.reverse();
        module.functions.reverse();
        for f in &mut module.functions {
            f.body_nodes.reverse();
        }
    }
    module
}

fn driver(module: &YirModule, ty: &str, mode: &str, cases: &[[u64; 8]]) -> (String, Vec<i64>) {
    let mut llvm = yir_lower_llvm::emit_module(module).unwrap().replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    for name in ["choose", "observe", "finish"] {
        // Instrument the compiled source callee, never replace its computation.
        let start = aggregate_values::definition(&llvm, name);
        let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
        llvm.insert_str(insertion, &format!("  %probe_calls = load volatile i64, ptr @{name}_probe\n  %probe_next = add i64 %probe_calls, 1\n  store volatile i64 %probe_next, ptr @{name}_probe\n"));
        llvm.push_str(&format!(
            "\n@{name}_probe = internal global i64 0, align 8\n"
        ));
    }
    // Flush only test observations so selected traps have reliable partial stdout.
    llvm = llvm.replace(
        "call void @nuis_debug_print_i64(",
        "call void @typed_handoff_print(",
    );
    if !llvm.contains("declare i32 @fflush(") {
        llvm.push_str("\ndeclare i32 @fflush(ptr)\n");
    }
    llvm.push_str("\ndefine void @typed_handoff_print(i64 %value) {\n  call void @nuis_debug_print_i64(i64 %value)\n  %probe_flush = call i32 @fflush(ptr null)\n  ret void\n}\n");
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n");
    let mut output = Vec::new();
    for (case, &input) in cases.iter().enumerate() {
        for name in ["choose", "observe", "finish"] {
            llvm.push_str(&format!("  store volatile i64 0, ptr @{name}_probe\n"));
        }
        let (state, prints, calls) = expected(ty, mode, input);
        let native_ty = match ty {
            "i32" => "i32",
            "f32" => "float",
            _ => "double",
        };
        let mut args = input[..4]
            .iter()
            .map(|word| format!("i1 {word}"))
            .collect::<Vec<_>>();
        for (slot, &word) in input[4..6].iter().enumerate() {
            match ty {
                "i32" => args.push(format!("i32 {}", word as i32)),
                "f32" => {
                    llvm.push_str(&format!(
                        "  %arg{case}_{slot} = bitcast i32 {} to float\n",
                        word as u32 as i32
                    ));
                    args.push(format!("float %arg{case}_{slot}"));
                }
                _ => {
                    llvm.push_str(&format!(
                        "  %arg{case}_{slot} = bitcast i64 {} to double\n",
                        word as i64
                    ));
                    args.push(format!("double %arg{case}_{slot}"));
                }
            }
        }
        args.extend(
            input[6..]
                .iter()
                .map(|word| format!("i64 {}", *word as i64)),
        );
        llvm.push_str(&format!(
            "  %result{case} = call {native_ty} @nuis_fn_event({})\n",
            args.join(", ")
        ));
        match ty {
            "i32" => llvm.push_str(&format!("  %resultword{case} = sext i32 %result{case} to i64\n")),
            "f32" => llvm.push_str(&format!("  %resultbits{case} = bitcast float %result{case} to i32\n  %resultword{case} = zext i32 %resultbits{case} to i64\n")),
            _ => llvm.push_str(&format!("  %resultword{case} = bitcast double %result{case} to i64\n")),
        }
        llvm.push_str(&format!(
            "  call void @nuis_debug_print_i64(i64 %resultword{case})\n"
        ));
        output.extend(&prints);
        output.push(state[8] as i64);
        for (name, count) in ["choose", "observe", "finish"].into_iter().zip(calls) {
            llvm.push_str(&format!("  %{name}{case} = load volatile i64, ptr @{name}_probe\n  call void @nuis_debug_print_i64(i64 %{name}{case})\n"));
            output.push(count);
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
    (llvm, output)
}

fn check(ty: &str) {
    let registry = yir_verify::default_registry();
    for mode in MODES {
        let source = source(ty, mode, mode == "partial");
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        assert!(
            compiled
                .yir
                .functions
                .iter()
                .any(|f| f.name.starts_with("__nuis_conditional_return")),
            "{ty} {mode}"
        );
        let words = words(ty);
        let cases = words
            .iter()
            .enumerate()
            .flat_map(|(i, &word)| inputs(word, words[(i + 1) % words.len()]))
            .filter(|input| !mode.ends_with("-no-exit") || input[0] == 0 || input[3] == 0)
            .collect::<Vec<_>>();
        for reversed in [false, true] {
            let module = reordered(&compiled.yir, reversed);
            for &input in &cases {
                let (state, prints, calls) = expected(ty, mode, input);
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(ty, input),
                )
                .unwrap();
                trace(&opened.events, &prints, calls);
                assert_eq!(state_words(session.state()), state, "{ty} {mode} {input:?}");
                let updated = session.event(vec![]).unwrap();
                trace(&updated.events, &prints, calls);
                assert_eq!(state_words(session.state()), state);
                let closed = session.close(vec![]).unwrap().unwrap();
                trace(&closed.events, &prints, calls);
                assert_eq!(state_words(session.state()), state);
                session.completion_status().unwrap();
            }
            // The scalar session bridge deliberately rejects effects. Ordinary
            // AOT proves this source return without weakening that boundary.
            assert!(emit_registered(&module, "counter").is_err());
            let (llvm, expected) = driver(&module, ty, mode, &cases);
            let artifact = nuisc::aot::write_and_link_with_source(
                &project.0.join("main.ns"),
                &project.0.join(format!("out-{reversed}")),
                &source,
                nuisc::aot::AotCompileProgram {
                    ast: &compiled.ast,
                    nir: &compiled.nir,
                    yir: &module,
                    llvm_ir: Some(&llvm),
                },
                &nuisc::aot::host_cpu_build_target(),
            )
            .unwrap();
            let run = dynamic_loop_guard::run_bounded(
                std::path::Path::new(&artifact.binary_path),
                &project.0,
            );
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|line| line.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{ty} {mode} reversed={reversed}");
        }
    }
}

#[test]
fn conditional_return_typed_handoff_i32_native_selected_exits_and_continuations() {
    check("i32");
}

#[test]
fn conditional_return_typed_handoff_f32_native_zero_nan_words_and_once_only_calls() {
    check("f32");
}

#[test]
fn conditional_return_typed_handoff_f64_native_zero_nan_words_and_once_only_calls() {
    check("f64");
}

#[test]
fn conditional_return_typed_handoff_selected_division_traps_before_tail_or_print() {
    let registry = yir_verify::default_registry();
    for ty in TYPES {
        let source = source(ty, "complete", false);
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        for gate in [false, true] {
            let input = [
                1,
                u64::from(gate),
                1,
                0,
                0,
                0,
                if gate { 0 } else { 2 },
                if gate { 2 } else { 0 },
            ];
            for reversed in [false, true] {
                let module = reordered(&compiled.yir, reversed);
                let err = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(ty, input),
                )
                .err()
                .unwrap();
                assert!(err.contains("zero"), "{err}");
                let (llvm, _) = driver(&module, ty, "complete", &[input]);
                let artifact = nuisc::aot::write_and_link_with_source(
                    &project.0.join("main.ns"),
                    &project.0.join(format!("trap-{gate}-{reversed}")),
                    &source,
                    nuisc::aot::AotCompileProgram {
                        ast: &compiled.ast,
                        nir: &compiled.nir,
                        yir: &module,
                        llvm_ir: Some(&llvm),
                    },
                    &nuisc::aot::host_cpu_build_target(),
                )
                .unwrap();
                let run = dynamic_loop_guard::run_bounded(
                    std::path::Path::new(&artifact.binary_path),
                    &project.0,
                );
                assert!(!run.status.success(), "selected division must trap");
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    assert!(
                        matches!(run.status.signal(), Some(4 | 5)),
                        "{:?}",
                        run.status
                    );
                }
                assert_eq!(String::from_utf8(run.stdout).unwrap(), "99\n");
            }
        }
    }
}
