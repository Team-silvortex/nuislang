#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
use std::os::unix::process::ExitStatusExt;
use yir_lower_llvm::native_session::{emit_registered_with_literal_prints, LiteralPrintPolicy};

#[path = "private_event_budget_cases.rs"]
mod cases;
use cases::{Case, Family};

const ENTRY: i64 = -81001;
const REJECT: i64 = -81002;
const BEFORE: i64 = -81003;
const AFTER: i64 = -81004;
const CLOSED: i64 = -81005;

fn driver(bridge: &NativeSessionBridge, cases: &[Case; 3]) -> (String, Vec<String>) {
    let mut llvm = String::new();
    let mut names = Vec::new();
    let mut rejected = false;
    for line in bridge.llvm_ir.lines() {
        // Add observations only; preserve admission comparisons, debit stores,
        // private bodies, joins, output stores and the original fatal trap.
        if line.starts_with("define ") && line.contains(" @nuis_fn_") {
            names.push(
                line.split(" @nuis_fn_")
                    .nth(1)
                    .unwrap()
                    .split('(')
                    .next()
                    .unwrap()
                    .into(),
            );
        }
        if line.starts_with("native_helper_entry_rejected") {
            rejected = true;
        }
        if rejected && line == "  call void @llvm.trap()" {
            llvm.push_str("  call void @budget_rejected(ptr %nuis_helper_entries)\n");
            rejected = false;
        }
        llvm.push_str(&line.replace(
            "call void @nuis_debug_print_i64(",
            "call void @budget_print(",
        ));
        llvm.push('\n');
        if line.starts_with("  store i64 %") && line.contains("ptr %nuis_helper_entries,") {
            llvm.push_str(&format!(
                "  call void @budget_entry(i64 {}, ptr %nuis_helper_entries)\n",
                names.len()
            ));
        }
    }
    llvm = llvm.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let outc = cases[0].state_words().len();
    llvm.push_str(&format!("\n@budget_output = internal global [{} x i64] zeroinitializer, align 8\ndeclare i32 @fflush(ptr)\ndefine void @budget_print(i64 %word) {{\n  call void @nuis_debug_print_i64(i64 %word)\n  %flush = call i32 @fflush(ptr null)\n  ret void\n}}\ndefine void @budget_entry(i64 %id, ptr %entries) {{\n  call void @budget_print(i64 {ENTRY})\n  call void @budget_print(i64 %id)\n  %remaining = load i64, ptr %entries\n  call void @budget_print(i64 %remaining)\n  ret void\n}}\ndefine void @budget_rejected(ptr %entries) {{\n  call void @budget_print(i64 {REJECT})\n  %remaining = load i64, ptr %entries\n  call void @budget_print(i64 %remaining)\n", outc + 2));
    for slot in 0..outc + 2 {
        llvm.push_str(&format!("  %p{slot} = getelementptr i64, ptr @budget_output, i64 {slot}\n  %v{slot} = load volatile i64, ptr %p{slot}\n  call void @budget_print(i64 %v{slot})\n"));
    }
    llvm.push_str("  ret void\n}\ndefine i64 @nuis_yir_entry() {\n");
    llvm.push_str(&format!("  %args = alloca [{} x i64], align 8\n  %out = getelementptr i64, ptr @budget_output, i64 1\n", outc + cases[0].values.len()));
    assert_eq!(
        llvm.matches("call void @budget_entry(").count(),
        names.len()
    );
    assert_eq!(llvm.matches("%nuis_helper_entries = alloca i64").count(), 3);
    assert!(!llvm.contains("thread_local") && !llvm.contains("@nuis_helper_entries"));
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    for (role, case) in [&cases[0], &cases[1], &cases[2], &cases[2]]
        .into_iter()
        .enumerate()
    {
        let args = if role == 0 {
            case.open_words()
        } else {
            case.event_words(&case.values)
        };
        if role != 0 {
            for slot in 0..outc {
                llvm.push_str(&format!("  %old{role}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %value{role}_{slot} = load volatile i64, ptr %old{role}_{slot}\n  %in{role}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 %value{role}_{slot}, ptr %in{role}_{slot}\n"));
            }
        }
        for (slot, word) in args.iter().enumerate() {
            let to = slot + if role == 0 { 0 } else { outc };
            llvm.push_str(&format!("  %arg{role}_{slot} = getelementptr i64, ptr %args, i64 {to}\n  store volatile i64 {}, ptr %arg{role}_{slot}\n", *word as i64));
        }
        if role == 0 {
            llvm.push_str(&format!("  store volatile i64 91, ptr @budget_output\n  %guard = getelementptr i64, ptr @budget_output, i64 {}\n  store volatile i64 92, ptr %guard\n", outc + 1));
        }
        if role == 2 {
            llvm.push_str(&format!("  call void @budget_print(i64 {BEFORE})\n"));
        }
        let inputc = args.len() + if role == 0 { 0 } else { outc };
        llvm.push_str(&format!("  %status{role} = call i32 @{}(ptr %args, i64 {inputc}, ptr %out, i64 {outc})\n  %wide{role} = zext i32 %status{role} to i64\n  call void @budget_print(i64 %wide{role})\n", bridge.callbacks[usize::from(role != 0)].symbol));
        for slot in 0..outc + 2 {
            llvm.push_str(&format!("  %p{role}_{slot} = getelementptr i64, ptr @budget_output, i64 {slot}\n  %v{role}_{slot} = load volatile i64, ptr %p{role}_{slot}\n  call void @budget_print(i64 %v{role}_{slot})\n"));
        }
        if role == 2 {
            llvm.push_str(&format!("  call void @budget_print(i64 {AFTER})\n"));
        }
    }
    for slot in 0..outc {
        llvm.push_str(&format!("  %close_out{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %close_word{slot} = load volatile i64, ptr %close_out{slot}\n  %close_in{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 %close_word{slot}, ptr %close_in{slot}\n"));
    }
    llvm.push_str(&format!("  %closed = call i32 @{}(ptr %args, i64 {outc}, ptr %out, i64 {outc})\n  %close_status = zext i32 %closed to i64\n  call void @budget_print(i64 %close_status)\n", bridge.callbacks[2].symbol));
    for slot in 0..outc + 2 {
        llvm.push_str(&format!("  %close_p{slot} = getelementptr i64, ptr @budget_output, i64 {slot}\n  %close_v{slot} = load volatile i64, ptr %close_p{slot}\n  call void @budget_print(i64 %close_v{slot})\n"));
    }
    llvm.push_str(&format!(
        "  call void @budget_print(i64 {CLOSED})\n  ret i64 0\n}}\n"
    ));
    (llvm, names)
}

fn output_words(expected: &mut Vec<i64>, case: &Case, status: bool) {
    if status {
        expected.push(0);
    }
    expected.push(91);
    expected.extend(case.state_words().into_iter().map(|word| word as i64));
    expected.push(92);
}

fn expected(family: Family, limit: u64, selected: &[Case; 3], names: &[String]) -> Vec<i64> {
    let mut expected = Vec::new();
    for (role, case) in [&selected[0], &selected[1], &selected[2], &selected[2]]
        .into_iter()
        .enumerate()
    {
        if role == 2 {
            expected.push(BEFORE);
        }
        let high = role >= 2;
        let path = cases::path(family, high, role != 0);
        let mut trace = path.iter().filter_map(|entry| entry.1).collect::<Vec<_>>();
        if !high && !matches!(family, Family::Bool) {
            trace.push(79);
        }
        assert_eq!(
            trace,
            case.trace
                .iter()
                .copied()
                .map(i64::from)
                .collect::<Vec<_>>()
        );
        if high {
            assert_eq!(path.len() as u64, cases::limit(family));
        }
        for (entry, (name, printed)) in path.iter().enumerate() {
            if entry as u64 == limit {
                expected.extend([REJECT, 0]);
                output_words(&mut expected, &selected[1], false);
                return expected;
            }
            let id = names
                .iter()
                .position(|n| n == name)
                .unwrap_or_else(|| panic!("missing {name}"))
                + 1;
            expected.extend([ENTRY, id as i64, (limit - entry as u64 - 1) as i64]);
            expected.extend(printed);
        }
        if !high && !matches!(family, Family::Bool) {
            expected.push(79);
        }
        output_words(&mut expected, case, true);
        if role == 2 {
            expected.push(AFTER);
        }
    }
    let stop = names.iter().position(|n| n == "stop").unwrap() + 1;
    expected.extend([ENTRY, stop as i64, (limit - 1) as i64]);
    output_words(&mut expected, &selected[2], true);
    expected.push(CLOSED);
    expected
}

fn check(family: Family) {
    let source = family.source();
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let selected = cases::selected(family);
    assert_ne!(selected[0].state_words(), selected[1].state_words());
    assert_ne!(selected[1].state_words(), selected[2].state_words());
    assert_eq!(
        cases::low_trace(family),
        selected[0]
            .trace
            .iter()
            .copied()
            .map(i64::from)
            .collect::<Vec<_>>()
    );
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
        for limit in [cases::limit(family) - 1, cases::limit(family)] {
            let bridge =
                emit_registered_with_literal_prints(&module, "counter", &policy, 0, limit).unwrap();
            assert_eq!(bridge.helper_entry_limit, limit);
            let (llvm, names) = driver(&bridge, &selected);
            let artifact = nuisc::aot::write_and_link_with_source(
                &project.0.join("main.ns"),
                &project.0.join(format!("out-{reversed}-{limit}")),
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
            let trapped = limit < cases::limit(family);
            if trapped {
                assert!(
                    matches!(run.status.signal(), Some(4 | 5)),
                    "{:?}",
                    run.status
                );
            } else {
                assert!(run.status.success(), "{:?}", run.status);
            }
            let words = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|v| v.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                words,
                expected(family, limit, &selected, &names),
                "{family:?} limit={limit} reversed={reversed}; helpers={names:?}"
            );
            assert_eq!(words.contains(&AFTER), !trapped);
            assert_eq!(words.contains(&CLOSED), !trapped);
        }
    }
}

#[test]
fn private_event_budget_bool() {
    check(Family::Bool);
}
#[test]
fn private_event_budget_i32() {
    check(Family::I32);
}
#[test]
fn private_event_budget_i64() {
    check(Family::I64);
}
#[test]
fn private_event_budget_f32() {
    check(Family::F32);
}
#[test]
fn private_event_budget_f64() {
    check(Family::F64);
}
