use super::driver::Driver;
use super::*;
use typed_effect_returns::{expected, inputs, reordered, trace, values, words};

fn check(ty: &str) {
    let registry = yir_verify::default_registry();
    let mut transitions = 0;
    for (index, mode) in MODES.into_iter().enumerate() {
        let text = source(ty, mode, index % 2 != 0);
        let project = Project::with_source(&text);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let mut cases = Vec::new();
        for word in words(ty) {
            let fallback = match ty {
                "i32" => i32::MIN as i64 as u64,
                "f32" => 0xffc0_4567,
                _ => 0xfff8_0000_0000_4567,
            };
            cases.extend(
                inputs(word, fallback)
                    .into_iter()
                    .filter(|input| !mode.ends_with("-no-exit") || input[0] == 0 || input[3] == 0),
            );
        }
        for reversed in [false, true] {
            let module = reordered(&compiled.yir, reversed);
            assert!(emit_registered(&module, "counter").is_err());
            let policy = LiteralPrintPolicy::new(grants(&module)).unwrap();
            let bridge =
                emit_registered_with_literal_prints(&module, "counter", &policy, 0, 100).unwrap();
            assert!(!bridge.llvm_ir.contains("deferred lowering"));
            let mut native = Driver::new(&bridge);
            for &input in &cases {
                let (state, prints, calls) = expected(ty, mode, input);
                let (mut reference, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(ty, input),
                )
                .unwrap();
                trace(&opened.events, &prints, calls);
                assert_eq!(state_words(reference.state()), state);
                let updated = reference.event(vec![]).unwrap();
                trace(&updated.events, &prints, calls);
                assert_eq!(state_words(reference.state()), state);
                let closed = reference.close(vec![]).unwrap().unwrap();
                trace(&closed.events, &prints, calls);
                assert_eq!(state_words(reference.state()), state);
                reference.completion_status().unwrap();
                native.valid(&bridge, input, &state, &prints, calls);
                transitions += 3;
            }
            native.invalid(&bridge, ty);
            let (llvm, oracle) = native.finish();
            let artifact = nuisc::aot::write_and_link_with_source(
                &project.0.join("main.ns"),
                &project.0.join(format!("literal-{reversed}")),
                &text,
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
                "{ty} {mode} {reversed}: {}",
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|line| line.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(actual, oracle, "{ty} {mode} {reversed}");
        }
    }
    assert_eq!(transitions, if ty == "i32" { 864 } else { 1512 });
}

#[test]
fn native_literal_print_policy_i32_registered_state_effects_and_preflight() {
    check("i32");
}

#[test]
fn native_literal_print_policy_f32_registered_raw_words_effects_and_preflight() {
    check("f32");
}

#[test]
fn native_literal_print_policy_f64_registered_raw_words_effects_and_preflight() {
    check("f64");
}

#[test]
fn native_literal_print_policy_traps_keep_prior_effects_and_never_publish_output() {
    for ty in TYPES {
        let text = source(ty, "complete", false);
        let project = Project::with_source(&text);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        for reversed in [false, true] {
            let module = reordered(&compiled.yir, reversed);
            let policy = LiteralPrintPolicy::new(grants(&module)).unwrap();
            for (name, gate, fuel, prefix) in [
                ("zero-entry", 1, 0, vec![]),
                ("left-div", 1, 100, vec![99]),
                ("right-div", 0, 100, vec![99]),
                ("helper-entry", 1, 2, vec![99]),
            ] {
                let bridge =
                    emit_registered_with_literal_prints(&module, "counter", &policy, 0, fuel)
                        .unwrap();
                let mut native = Driver::new(&bridge);
                native.trap(&bridge, [1, gate, 1, 0, 0, 0, 0, 0]);
                let (llvm, _) = native.finish();
                let artifact = nuisc::aot::write_and_link_with_source(
                    &project.0.join("main.ns"),
                    &project.0.join(format!("{name}-{reversed}")),
                    &text,
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
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    assert!(
                        matches!(run.status.signal(), Some(4 | 5)),
                        "{:?}",
                        run.status
                    );
                }
                assert!(!run.status.success());
                let actual = String::from_utf8(run.stdout)
                    .unwrap()
                    .lines()
                    .map(|line| line.parse::<i64>().unwrap())
                    .collect::<Vec<_>>();
                let mut expected = prefix;
                expected.extend(Driver::trap_canaries());
                assert_eq!(actual, expected, "{ty} {name} {reversed}");
            }
        }
    }
}
