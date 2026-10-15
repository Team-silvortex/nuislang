use super::*;
use std::os::unix::process::ExitStatusExt;
use yir_lower_llvm::native_session::{emit_registered_with_literal_prints, LiteralPrintPolicy};
use yir_runtime_host::ApplicationSessionPhase;

#[path = "return_exit_prefix_cases.rs"]
mod cases;
use cases::{Case, SOURCE};

fn values(words: Vec<i64>) -> Vec<Value> {
    words
        .into_iter()
        .enumerate()
        .map(|(i, n)| {
            if (1..=3).contains(&i) {
                Value::Bool(n != 0)
            } else {
                Value::Int(n)
            }
        })
        .collect()
}

fn trace(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn return_exit_prefix_reference_current_fields_lazy_work_and_last_publication() {
    for mirrored in [false, true] {
        let source = if mirrored {
            SOURCE.replace("select(pick,", "select(pick != true,")
        } else {
            SOURCE.to_owned()
        };
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            for case in cases::cases() {
                let (mut session, opened) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(case.words()),
                )
                .unwrap();
                assert_eq!(trace(&opened.events), case.trace(mirrored, false));
                assert_eq!(state_words(session.state()), vec![case.result() as u64]);
                let next = case.next();
                let event = session.event(values(next.words())).unwrap();
                assert_eq!(trace(&event.events), next.trace(mirrored, false));
                assert_eq!(state_words(session.state()), vec![next.result() as u64]);
                session.close(vec![]).unwrap();
                session.completion_status().unwrap();
            }
            for (case, slot, _) in cases::failures() {
                let (mut session, _) = ApplicationSession::open_registered(
                    &module,
                    &registry,
                    "counter",
                    values(case.next().words()),
                )
                .unwrap();
                let last = session.state().clone();
                let mut bad = case.words();
                bad[slot] = 0;
                assert!(session.event(values(bad)).unwrap_err().contains("zero"));
                assert_eq!(session.state(), &last);
                assert_eq!(session.phase(), ApplicationSessionPhase::Faulted);
                assert!(session.event(values(case.words())).is_err());
                session.close(vec![]).unwrap();
                assert!(session.completion_status().is_err());
            }
        }
    }
    // Current owner changes false to true; current child changes true to false.
    let a = Case {
        seed: 7,
        pick: true,
        take: true,
        inner: false,
    };
    assert_eq!(a.result(), 1);
    assert_eq!(a.next().result(), 0);
    assert_eq!(Case { inner: true, ..a }.result(), 0);
}

fn callback(
    llvm: &mut String,
    bridge: &NativeSessionBridge,
    case: Case,
    role: usize,
    poison: Option<usize>,
    id: usize,
) {
    if role != 0 {
        llvm.push_str(&format!("  %last{id} = load volatile i64, ptr %out\n  store volatile i64 %last{id}, ptr %input\n"));
    }
    let mut words = case.words();
    if let Some(slot) = poison {
        words[slot] = 0;
    }
    if role != 2 {
        for (slot, word) in words.iter().enumerate() {
            let index = slot + usize::from(role != 0);
            llvm.push_str(&format!("  %arg{id}_{slot} = getelementptr i64, ptr %input, i64 {index}\n  store volatile i64 {word}, ptr %arg{id}_{slot}\n"));
        }
    }
    llvm.push_str("  store volatile i64 91, ptr %output\n  store volatile i64 92, ptr %canary\n");
    let count = if role == 2 {
        1
    } else {
        words.len() + usize::from(role != 0)
    };
    llvm.push_str(&format!("  %status{id} = call i32 @{}(ptr %input, i64 {count}, ptr %out, i64 1)\n  %wide{id} = zext i32 %status{id} to i64\n  call void @exit_print(i64 %wide{id})\n", bridge.callbacks[role].symbol));
    for slot in 0..3 {
        llvm.push_str(&format!("  %p{id}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  %v{id}_{slot} = load volatile i64, ptr %p{id}_{slot}\n  call void @exit_print(i64 %v{id}_{slot})\n"));
    }
}

fn driver(bridge: &NativeSessionBridge) -> String {
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let start = aggregate_values::definition(&llvm, "answer_work");
    let at = start + llvm[start..].find("{\n").unwrap() + 2;
    llvm.insert_str(at, "  call void @nuis_debug_print_i64(i64 %arg0)\n  %small_word = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %small_word)\n  %flag_word = zext i1 %arg2 to i64\n  call void @nuis_debug_print_i64(i64 %flag_word)\n");
    llvm = llvm.replace("call void @nuis_debug_print_i64(", "call void @exit_print(");
    if !llvm.contains("declare i32 @fflush(") {
        llvm.push_str("\ndeclare i32 @fflush(ptr)\n");
    }
    llvm.push_str("\ndefine void @exit_print(i64 %word) {\n  call void @nuis_debug_print_i64(i64 %word)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n@exit_env = private constant [17 x i8] c\"NUIS_EXIT_PREFIX\\00\"\ndeclare ptr @getenv(ptr)\ndeclare i32 @atoi(ptr)\n");
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %input = alloca [12 x i64], align 8\n  %output = alloca [3 x i64], align 8\n  %out = getelementptr i64, ptr %output, i64 1\n  %canary = getelementptr i64, ptr %output, i64 2\n  %env = call ptr @getenv(ptr @exit_env)\n  %mode = call i32 @atoi(ptr %env)\n  switch i32 %mode, label %reject [\n    i32 0, label %healthy\n");
    for i in 0..7 {
        llvm.push_str(&format!("    i32 {}, label %bad{i}\n", i + 1));
    }
    llvm.push_str("  ]\nreject:\n  call void @llvm.trap()\n  unreachable\nhealthy:\n");
    let mut id = 0;
    for case in cases::cases() {
        callback(&mut llvm, bridge, case, 0, None, id);
        id += 1;
        callback(&mut llvm, bridge, case.next(), 1, None, id);
        id += 1;
        callback(&mut llvm, bridge, case.next(), 2, None, id);
        id += 1;
    }
    llvm.push_str("  ret i64 0\n");
    for (i, (case, slot, _)) in cases::failures().into_iter().enumerate() {
        llvm.push_str(&format!("bad{i}:\n"));
        callback(&mut llvm, bridge, case.next(), 0, None, id);
        id += 1;
        llvm.push_str("  call void @exit_print(i64 -70001)\n");
        callback(&mut llvm, bridge, case, 1, Some(slot), id);
        id += 1;
        llvm.push_str("  call void @exit_print(i64 -70002)\n  ret i64 0\n");
    }
    llvm.push_str("}\n");
    llvm
}

fn accepted(case: Case, mirrored: bool) -> Vec<i64> {
    let mut expected = case.trace(mirrored, true);
    expected.extend([0, 91, case.result(), 92]);
    expected
}

#[test]
fn return_exit_prefix_native_observed_fields_effect_cuts_canaries_and_reentry() {
    for mirrored in [false, true] {
        let source = if mirrored {
            SOURCE.replace("select(pick,", "select(pick != true,")
        } else {
            SOURCE.to_owned()
        };
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let policy = LiteralPrintPolicy::new(
                module
                    .nodes
                    .iter()
                    .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
                    .map(|n| n.name.clone()),
            )
            .unwrap();
            let bridge =
                emit_registered_with_literal_prints(&module, "counter", &policy, 0, 100).unwrap();
            let llvm = driver(&bridge);
            assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
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
            for mode in 0..=7 {
                let mut command = Command::new(&artifact.binary_path);
                command.env("NUIS_EXIT_PREFIX", mode.to_string());
                let run = dynamic_loop_guard::run_command_bounded(command, &project.0);
                let mut expected = Vec::new();
                if mode == 0 {
                    assert!(
                        run.status.success(),
                        "{}",
                        String::from_utf8_lossy(&run.stderr)
                    );
                    for case in cases::cases() {
                        expected.extend(accepted(case, mirrored));
                        expected.extend(accepted(case.next(), mirrored));
                        expected.extend([0, 91, case.next().result(), 92]);
                    }
                } else {
                    assert!(
                        matches!(run.status.signal(), Some(4 | 5)),
                        "{:?}",
                        run.status
                    );
                    let (case, _, stage) = cases::failures()[mode - 1];
                    expected.extend(accepted(case.next(), mirrored));
                    expected.push(-70001);
                    let trace = case.trace(mirrored, true);
                    let cut = if stage == 28 {
                        trace.len() - 4
                    } else {
                        trace.iter().position(|&v| v == stage).unwrap()
                    };
                    expected.extend(&trace[..cut]);
                }
                let actual = String::from_utf8(run.stdout)
                    .unwrap()
                    .lines()
                    .map(|n| n.parse::<i64>().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(
                    actual, expected,
                    "mirrored={mirrored} reversed={reversed} mode={mode}"
                );
            }
        }
    }
}
