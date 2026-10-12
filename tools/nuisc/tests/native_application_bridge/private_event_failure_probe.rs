use super::*;
use std::os::unix::process::ExitStatusExt;
use yir_lower_llvm::native_session::{emit_registered_with_literal_prints, LiteralPrintPolicy};

const ENV: &str = "NUIS_PRIVATE_EVENT_CASE";
const BEFORE_FAILURE: i64 = -70001;
const AFTER_FAILURE: i64 = -70002;
const AFTER_CLOSE: i64 = -70003;

fn table(llvm: &mut String, name: &str, rows: &[Vec<u64>]) {
    let row_count = rows.len();
    let count = rows[0].len();
    assert!(rows.iter().all(|row| row.len() == count));
    let rows = rows
        .iter()
        .map(|row| {
            format!(
                "[{count} x i64] [{}]",
                row.iter()
                    .map(|word| format!("i64 {}", *word as i64))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    llvm.push_str(&format!(
        "\n@{name} = internal constant [{} x [{count} x i64]] [{rows}], align 8\n",
        row_count
    ));
}

fn driver(bridge: &NativeSessionBridge, selected: &[Case]) -> String {
    let outc = selected[0].state_words().len();
    let eventc = selected[0].values.len();
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    // Only observations are flushed. Source callees, checked divisions, private
    // helpers, joins, callback output and real llvm.trap are not substituted.
    llvm = llvm.replace(
        "call void @nuis_debug_print_i64(",
        "call void @private_event_print(",
    );
    if !llvm.contains("declare i32 @fflush(") {
        llvm.push_str("\ndeclare i32 @fflush(ptr)\n");
    }
    llvm.push_str("\ndefine void @private_event_print(i64 %word) {\n  call void @nuis_debug_print_i64(i64 %word)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n");
    let opens = selected.iter().map(Case::open_words).collect::<Vec<_>>();
    let previous = selected
        .iter()
        .map(|case| {
            let previous = case.previous();
            previous.event_words(&previous.values)
        })
        .collect::<Vec<_>>();
    let bad = selected
        .iter()
        .enumerate()
        .map(|(index, case)| case.event_words(&case.bad_values(index)))
        .collect::<Vec<_>>();
    table(&mut llvm, "private_open", &opens);
    table(&mut llvm, "private_previous", &previous);
    table(&mut llvm, "private_bad", &bad);
    llvm.push_str(&format!("\n@private_case_env = private constant [{} x i8] c\"{ENV}\\00\"\ndeclare ptr @getenv(ptr)\ndeclare i32 @atoi(ptr)\n", ENV.len() + 1));
    llvm.push_str(&format!("\ndefine i64 @nuis_yir_entry() {{\n  %env = call ptr @getenv(ptr @private_case_env)\n  %missing = icmp eq ptr %env, null\n  br i1 %missing, label %reject, label %parse\nparse:\n  %index32 = call i32 @atoi(ptr %env)\n  %index = zext i32 %index32 to i64\n  %valid = icmp ult i64 %index, {}\n  br i1 %valid, label %run, label %reject\nreject:\n  call void @llvm.trap()\n  unreachable\nrun:\n  %input = alloca [{} x i64], align 8\n  %output = alloca [{} x i64], align 8\n  %out = getelementptr i64, ptr %output, i64 1\n", selected.len(), outc + eventc, outc + 2));
    for (role, name) in ["private_open", "private_previous", "private_bad"]
        .into_iter()
        .enumerate()
    {
        let rowc = if role == 0 { eventc + 1 } else { eventc };
        let inputc = if role == 0 { rowc } else { outc + rowc };
        if role != 0 {
            // Feed the next callback the actual preceding callback output,
            // never an oracle-manufactured session state.
            for slot in 0..outc {
                llvm.push_str(&format!("  %prev{role}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %saved{role}_{slot} = load volatile i64, ptr %prev{role}_{slot}\n  %in{role}_{slot} = getelementptr i64, ptr %input, i64 {slot}\n  store volatile i64 %saved{role}_{slot}, ptr %in{role}_{slot}\n"));
            }
        }
        for slot in 0..rowc {
            let to = if role == 0 { slot } else { outc + slot };
            llvm.push_str(&format!("  %source{role}_{slot} = getelementptr [{} x [{rowc} x i64]], ptr @{name}, i64 0, i64 %index, i64 {slot}\n  %arg{role}_{slot} = load volatile i64, ptr %source{role}_{slot}\n  %dest{role}_{slot} = getelementptr i64, ptr %input, i64 {to}\n  store volatile i64 %arg{role}_{slot}, ptr %dest{role}_{slot}\n", selected.len()));
        }
        for (slot, word) in [(0, 91), (outc + 1, 92)] {
            llvm.push_str(&format!("  %guard{role}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  store volatile i64 {word}, ptr %guard{role}_{slot}\n"));
        }
        if role == 2 {
            llvm.push_str(&format!(
                "  call void @private_event_print(i64 {BEFORE_FAILURE})\n"
            ));
        }
        let symbol = &bridge.callbacks[usize::from(role != 0)].symbol;
        llvm.push_str(&format!("  %status{role} = call i32 @{symbol}(ptr %input, i64 {inputc}, ptr %out, i64 {outc})\n"));
        if role == 2 {
            break;
        }
        llvm.push_str(&format!("  %wide{role} = zext i32 %status{role} to i64\n  call void @private_event_print(i64 %wide{role})\n"));
        for slot in 0..outc + 2 {
            llvm.push_str(&format!("  %p{role}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  %word{role}_{slot} = load volatile i64, ptr %p{role}_{slot}\n  call void @private_event_print(i64 %word{role}_{slot})\n"));
        }
    }
    llvm.push_str(&format!("  call void @private_event_print(i64 {AFTER_FAILURE})\n  %closed = call i32 @{}(ptr %input, i64 {outc}, ptr %out, i64 {outc})\n  call void @private_event_print(i64 {AFTER_CLOSE})\n  ret i64 0\n}}\n", bridge.callbacks[2].symbol));
    llvm
}

fn expected(case: &Case, index: usize) -> Vec<i64> {
    let previous = case.previous();
    let mut result = Vec::new();
    for accepted in [case, &previous] {
        result.extend(accepted.trace.iter().copied().map(i64::from));
        result.extend([0, 91]);
        result.extend(accepted.state_words().into_iter().map(|word| word as i64));
        result.push(92);
    }
    result.push(BEFORE_FAILURE);
    let stage = case.family.stages()[index];
    let position = case.trace.iter().position(|v| *v == stage).unwrap();
    result.extend(case.trace[..position].iter().copied().map(i64::from));
    result
}

pub(super) fn check(family: Family) {
    let source = family.source();
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let selected = family.selected();
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
        let llvm = driver(&bridge, &selected);
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
        for (index, case) in selected.iter().enumerate() {
            let mut command = Command::new(&artifact.binary_path);
            command.env(ENV, index.to_string());
            let run = dynamic_loop_guard::run_command_bounded(command, &project.0);
            assert!(
                matches!(run.status.signal(), Some(4 | 5)),
                "{:?}\n{}",
                run.status,
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|v| v.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                actual,
                expected(case, index),
                "{family:?} stage={index} reversed={reversed}"
            );
            assert!(!actual.contains(&AFTER_FAILURE) && !actual.contains(&AFTER_CLOSE));
        }
    }
}
