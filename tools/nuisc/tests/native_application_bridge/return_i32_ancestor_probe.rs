use super::*;
use std::os::unix::process::ExitStatusExt;
use yir_lower_llvm::native_session::{emit_registered_with_literal_prints, LiteralPrintPolicy};

const ENV: &str = "NUIS_ANCESTOR_CASE";
const BEFORE: i64 = -70001;
const AFTER: i64 = -70002;

fn table(llvm: &mut String, name: &str, rows: &[Vec<i64>]) {
    let width = rows[0].len();
    assert!(rows.iter().all(|row| row.len() == width));
    let values = rows
        .iter()
        .map(|row| {
            format!(
                "[{width} x i64] [{}]",
                row.iter()
                    .map(|v| format!("i64 {v}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    llvm.push_str(&format!(
        "\n@{name} = internal constant [{} x [{width} x i64]] [{values}], align 8\n",
        rows.len()
    ));
}

fn driver(
    bridge: &NativeSessionBridge,
    cases: &[Case],
    failure: bool,
    observations: &[(&str, &str)],
    open_words: fn(&Case) -> Vec<i64>,
) -> String {
    let state_width = cases[0].state_words().len();
    let open_width = open_words(&cases[0]).len();
    let event_width = cases[0].words().len();
    assert!(cases
        .iter()
        .all(|case| case.state_words().len() == state_width
            && open_words(case).len() == open_width
            && case.words().len() == event_width));
    let input_width = open_width.max(state_width + event_width);
    let output_width = state_width + 2;
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let start = aggregate_values::definition(&llvm, "answer_work");
    let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
    // Observe the actual compiled callee argument, not an oracle-manufactured
    // value. Source computations, private joins and real traps remain intact.
    llvm.insert_str(insertion, OWNER_PROBE);
    for &(callee, observer) in observations {
        let start = aggregate_values::definition(&llvm, callee);
        let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
        llvm.insert_str(insertion, observer);
    }
    llvm = llvm.replace(
        "call void @nuis_debug_print_i64(",
        "call void @ancestor_print(",
    );
    if !llvm.contains("declare i32 @fflush(") {
        llvm.push_str("\ndeclare i32 @fflush(ptr)\n");
    }
    llvm.push_str("\ndefine void @ancestor_print(i64 %word) {\n  call void @nuis_debug_print_i64(i64 %word)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n");
    table(
        &mut llvm,
        "ancestor_open",
        &cases.iter().map(open_words).collect::<Vec<_>>(),
    );
    table(
        &mut llvm,
        "ancestor_event",
        &cases
            .iter()
            .map(|case| case.previous().words())
            .collect::<Vec<_>>(),
    );
    if failure {
        table(
            &mut llvm,
            "ancestor_bad",
            &cases
                .iter()
                .enumerate()
                .map(|(index, case)| {
                    let mut words = case.words();
                    words[index + 2] = 0;
                    words
                })
                .collect::<Vec<_>>(),
        );
    }
    llvm.push_str(&format!("\n@ancestor_env = private constant [{} x i8] c\"{ENV}\\00\"\ndeclare ptr @getenv(ptr)\ndeclare i32 @atoi(ptr)\n", ENV.len() + 1));
    llvm.push_str(&format!("\ndefine i64 @nuis_yir_entry() {{\n  %env = call ptr @getenv(ptr @ancestor_env)\n  %missing = icmp eq ptr %env, null\n  br i1 %missing, label %reject, label %parse\nparse:\n  %index32 = call i32 @atoi(ptr %env)\n  %index = zext i32 %index32 to i64\n  %valid = icmp ult i64 %index, {}\n  br i1 %valid, label %run, label %reject\nreject:\n  call void @llvm.trap()\n  unreachable\nrun:\n  %input = alloca [{input_width} x i64], align 8\n  %output = alloca [{output_width} x i64], align 8\n  %out = getelementptr i64, ptr %output, i64 1\n", cases.len()));
    for role in 0..3 {
        if role != 0 {
            for slot in 0..state_width {
                llvm.push_str(&format!("  %prev{role}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %saved{role}_{slot} = load volatile i64, ptr %prev{role}_{slot}\n  %in{role}_{slot} = getelementptr i64, ptr %input, i64 {slot}\n  store volatile i64 %saved{role}_{slot}, ptr %in{role}_{slot}\n"));
            }
        }
        let width = if role == 0 {
            open_width
        } else if role == 1 || failure {
            event_width
        } else {
            0
        };
        let name = ["ancestor_open", "ancestor_event", "ancestor_bad"][role];
        for slot in 0..width {
            let to = if role == 0 { slot } else { state_width + slot };
            llvm.push_str(&format!("  %source{role}_{slot} = getelementptr [{} x [{width} x i64]], ptr @{name}, i64 0, i64 %index, i64 {slot}\n  %arg{role}_{slot} = load volatile i64, ptr %source{role}_{slot}\n  %dest{role}_{slot} = getelementptr i64, ptr %input, i64 {to}\n  store volatile i64 %arg{role}_{slot}, ptr %dest{role}_{slot}\n", cases.len()));
        }
        for (slot, word) in [(0, 91), (state_width + 1, 92)] {
            llvm.push_str(&format!("  %guard{role}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  store volatile i64 {word}, ptr %guard{role}_{slot}\n"));
        }
        if failure && role == 2 {
            llvm.push_str(&format!("  call void @ancestor_print(i64 {BEFORE})\n"));
        }
        let callback = if failure && role == 2 { 1 } else { role };
        let count = if role == 0 {
            width
        } else {
            state_width + width
        };
        llvm.push_str(&format!(
            "  %status{role} = call i32 @{}(ptr %input, i64 {count}, ptr %out, i64 {state_width})\n",
            bridge.callbacks[callback].symbol
        ));
        if failure && role == 2 {
            llvm.push_str(&format!("  call void @ancestor_print(i64 {AFTER})\n"));
        }
        llvm.push_str(&format!("  %wide{role} = zext i32 %status{role} to i64\n  call void @ancestor_print(i64 %wide{role})\n"));
        for slot in 0..output_width {
            llvm.push_str(&format!("  %p{role}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  %word{role}_{slot} = load volatile i64, ptr %p{role}_{slot}\n  call void @ancestor_print(i64 %word{role}_{slot})\n"));
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
    llvm
}

fn expected(case: &Case, failure: bool, index: usize) -> Vec<i64> {
    let previous = case.previous();
    let mut trace = Vec::new();
    for accepted in [case, &previous] {
        trace.extend(accepted.observed());
        trace.extend([0, 91]);
        trace.extend(accepted.state_words());
        trace.push(92);
    }
    if failure {
        trace.push(BEFORE);
        // Preserve observations from earlier callees but exclude the trapped
        // callee: its bad proof is evaluated before entry.
        trace.extend(case.before_failure(index));
    } else {
        trace.extend([0, 91]);
        trace.extend(previous.state_words());
        trace.push(92);
    }
    trace
}

#[allow(dead_code)] // Single-seed adapter is unused by two-seed test instantiations.
pub(super) fn check(
    source: &str,
    cases: &[Case],
    failure: bool,
    observation: Option<(&str, &str)>,
) {
    check_with_open_words(source, cases, failure, observation, |case| {
        std::iter::once(i64::from(case.seed))
            .chain(case.words())
            .collect()
    });
}

#[allow(dead_code)]
pub(super) fn check_with_open_words(
    source: &str,
    cases: &[Case],
    failure: bool,
    observation: Option<(&str, &str)>,
    open_words: fn(&Case) -> Vec<i64>,
) {
    let observations = observation.into_iter().collect::<Vec<_>>();
    check_with_observations(source, cases, failure, &observations, open_words);
}

pub(super) fn check_with_observations(
    source: &str,
    cases: &[Case],
    failure: bool,
    observations: &[(&str, &str)],
    open_words: fn(&Case) -> Vec<i64>,
) {
    let project = Project::with_source(source);
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
        let llvm = driver(&bridge, cases, failure, observations, open_words);
        let artifact = nuisc::aot::write_and_link_with_source(
            &project.0.join("main.ns"),
            &project.0.join(format!("out-{reversed}")),
            source,
            nuisc::aot::AotCompileProgram {
                ast: &compiled.ast,
                nir: &compiled.nir,
                yir: &module,
                llvm_ir: Some(&llvm),
            },
            &nuisc::aot::host_cpu_build_target(),
        )
        .unwrap();
        for (index, case) in cases.iter().enumerate() {
            let mut command = Command::new(&artifact.binary_path);
            command.env(ENV, index.to_string());
            let run = dynamic_loop_guard::run_command_bounded(command, &project.0);
            if failure {
                assert!(
                    matches!(run.status.signal(), Some(4 | 5)),
                    "{:?}\n{}",
                    run.status,
                    String::from_utf8_lossy(&run.stderr)
                );
            } else {
                assert!(
                    run.status.success(),
                    "{:?}\n{}",
                    run.status,
                    String::from_utf8_lossy(&run.stderr)
                );
            }
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|v| v.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            // Full trace equality distinguishes AFTER from matching field words.
            assert_eq!(
                actual,
                expected(case, failure, index),
                "case={index} reversed={reversed} failure={failure}"
            );
        }
    }
}
