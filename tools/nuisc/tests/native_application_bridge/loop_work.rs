use super::*;
use std::path::Path;
use yir_lower_llvm::native_session::emit_registered_with_loop_work_limit;

const SOURCE: &str = include_str!("loop_work.ns");
const SENTINEL: i64 = -700;

struct Case {
    callback: usize,
    inputs: [i64; 5],
    prior: i64,
    value: Option<i64>,
    trace: Vec<i64>,
    iterations: i64,
    remaining: u64,
}

fn open(
    inputs: [i64; 5],
    value: Option<i64>,
    trace: &[i64],
    iterations: i64,
    remaining: u64,
) -> Case {
    Case {
        callback: 0,
        inputs,
        prior: 0,
        value,
        trace: trace.to_vec(),
        iterations,
        remaining,
    }
}

// Trace tags: 41 = remaining budget after a reservation; 55 = evaluated argument.
// The budget and all real traps are unchanged. Only test evidence is global.
fn instrument(llvm: &str) -> String {
    let mut result = String::new();
    for (index, line) in llvm.lines().enumerate() {
        if line == "  call void @llvm.trap()" {
            result.push_str("  call void @probe_trap(ptr %nuis_loop_work)\n");
        }
        result.push_str(line);
        result.push('\n');
        if line.starts_with("  store i64 %") && line.contains("ptr %nuis_loop_work,") {
            result.push_str("  call void @probe_debit(ptr %nuis_loop_work)\n");
        }
        if line.starts_with("loop_while_") && line.contains("_body") && line.ends_with(':') {
            result.push_str(&format!("  %probe_before_{index} = load volatile i64, ptr @probe_iterations\n  %probe_after_{index} = add i64 %probe_before_{index}, 1\n  store volatile i64 %probe_after_{index}, ptr @probe_iterations\n"));
        }
        if line.starts_with("define i64 @nuis_fn_argument(") {
            result.push_str("  call void @nuis_debug_print_i64(i64 55)\n  call void @nuis_debug_print_i64(i64 %arg0)\n");
        }
    }
    result
}

fn execute(source: &str, budget: u64, cases: &[Case]) {
    assert!(!cases.is_empty());
    assert!(cases[..cases.len() - 1]
        .iter()
        .all(|case| case.value.is_some()));
    let trap = cases.last().unwrap().value.is_none();
    let project = Project::with_source(source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.functions.reverse();
    compiled.yir.nodes.reverse();
    let bridge = emit_registered_with_loop_work_limit(&compiled.yir, "counter", budget).unwrap();
    assert_eq!(bridge.loop_work_limit, budget);
    assert_eq!(
        bridge
            .llvm_ir
            .matches("%nuis_loop_work = alloca i64")
            .count(),
        3
    );
    assert!(!bridge.llvm_ir.contains("thread_local"));
    assert!(!bridge.llvm_ir.contains("@nuis_loop_work"));
    let ordinary = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    assert!(!ordinary.contains("%nuis_loop_work"));
    assert!(!ordinary.contains("native_loop_work"));
    let mut llvm = instrument(&bridge.llvm_ir).replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(PROBES);
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [6 x i64], align 8\n");
    let mut expected = Vec::new();
    let registry = yir_verify::default_registry();
    for (index, case) in cases.iter().enumerate() {
        if let Some(value) = case.value {
            let [rounds, trips, stride, early, skip] = case.inputs;
            let (reference, _) = ApplicationSession::open_registered(
                &compiled.yir,
                &registry,
                "counter",
                vec![
                    Value::Int(rounds),
                    Value::Int(trips),
                    Value::Int(stride),
                    Value::Bool(early != 0),
                    Value::Bool(skip != 0),
                ],
            )
            .unwrap();
            assert_eq!(
                state_words(reference.state()),
                vec![(value - case.prior) as u64]
            );
        }
        llvm.push_str(&format!("  store volatile i64 0, ptr @probe_iterations\n  store volatile i64 {SENTINEL}, ptr @probe_out\n"));
        let mut args = Vec::new();
        if case.callback != 0 {
            args.push(case.prior);
        }
        args.extend(case.inputs);
        for (slot, value) in args.iter().enumerate() {
            llvm.push_str(&format!("  %arg_{index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {value}, ptr %arg_{index}_{slot}\n"));
        }
        llvm.push_str(&format!("  %status_{index} = call i32 @{}(ptr %args, i64 {}, ptr @probe_out, i64 1)\n  %wide_{index} = zext i32 %status_{index} to i64\n  call void @nuis_debug_print_i64(i64 %wide_{index})\n  %out_{index} = load volatile i64, ptr @probe_out\n  call void @nuis_debug_print_i64(i64 %out_{index})\n  %trips_{index} = load volatile i64, ptr @probe_iterations\n  call void @nuis_debug_print_i64(i64 %trips_{index})\n  %flush_{index} = call i32 @fflush(ptr null)\n", bridge.callbacks[case.callback].symbol, args.len()));
        expected.extend(&case.trace);
        if let Some(value) = case.value {
            expected.extend([0, value, case.iterations]);
        } else {
            expected.extend([73, case.remaining as i64, case.iterations, SENTINEL]);
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
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
    let run = dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0);
    assert_eq!(
        run.status.success(),
        !trap,
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    if trap {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(run.status.signal(), Some(4 | 5)),
                "{:?}",
                run.status
            );
        }
    }
    let actual = String::from_utf8(run.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

const PROBES: &str = "
@probe_iterations = internal global i64 0, align 8
@probe_out = internal global i64 -700, align 8
declare i32 @fflush(ptr)
define void @probe_debit(ptr %work) {
  call void @nuis_debug_print_i64(i64 41)
  %remaining = load i64, ptr %work
  call void @nuis_debug_print_i64(i64 %remaining)
  ret void
}
define void @probe_trap(ptr %work) {
  call void @nuis_debug_print_i64(i64 73)
  %remaining = load i64, ptr %work
  call void @nuis_debug_print_i64(i64 %remaining)
  %trips = load volatile i64, ptr @probe_iterations
  call void @nuis_debug_print_i64(i64 %trips)
  %out = load volatile i64, ptr @probe_out
  call void @nuis_debug_print_i64(i64 %out)
  %flushed = call i32 @fflush(ptr null)
  ret void
}
";

#[test]
fn whole_callback_loop_work_is_shared_and_resets_for_every_lifecycle_entry() {
    let inputs = [2, 3, 1, 0, 0];
    let trace = [41, 6, 55, 3, 41, 3, 55, 3, 41, 0];
    let mut cases = vec![open(inputs, Some(6), &trace, 8, 0)];
    for (callback, prior) in [(1, 6), (1, 12), (2, 18), (0, 0)] {
        cases.push(Case {
            callback,
            prior,
            ..open(inputs, Some(prior + 6), &trace, 8, 0)
        });
    }
    execute(SOURCE, 8, &cases);
}

#[test]
fn whole_callback_loop_work_traps_before_an_over_budget_child_or_outer_body() {
    execute(
        SOURCE,
        7,
        &[open(
            [2, 3, 1, 0, 0],
            None,
            &[41, 5, 55, 3, 41, 2, 55, 3],
            5,
            2,
        )],
    );
    execute(SOURCE, 1, &[open([2, 3, 1, 0, 0], None, &[], 0, 1)]);
}

#[test]
fn whole_callback_loop_work_checks_induction_before_debit_and_skips_dead_work() {
    execute(
        SOURCE,
        0,
        &[
            open([65537, 7, 0, 0, 1], Some(0), &[], 0, 0),
            open([0, 7, 0, 0, 0], Some(0), &[], 0, 0),
            open([65537, 7, 0, 0, 0], None, &[], 0, 0),
        ],
    );
    execute(
        SOURCE,
        2,
        &[
            open([2, 0, 0, 0, 0], Some(0), &[41, 0, 55, 0, 55, 0], 2, 0),
            open([1, 3, 0, 0, 0], None, &[41, 1, 55, 3], 1, 1),
        ],
    );
    execute(
        SOURCE,
        8,
        &[open([1, 3, 0, 0, 0], None, &[41, 7, 55, 3], 1, 7)],
    );
}

#[test]
fn whole_callback_loop_work_does_not_refund_early_break() {
    // Guarded-break helpers are admitted as ordinary calls, not by the narrower
    // iteration-helper catalog. Do not widen that independent source boundary.
    let source = SOURCE
        .replace(
            "return nested(rounds, trips, stride, early);",
            "let first = shortened(trips); return first + shortened(trips);",
        )
        .replace(
            "fn main()",
            "@noinline fn shortened(trips: i64) -> i64 {
            let index: i64 = 0; let value: i64 = 0;
            while index < trips {
                let value: i64 = value + 1;
                if index == 0 { break; }
                let index: i64 = index + 1;
            }
            return value;
        } fn main()",
        );
    execute(
        &source,
        6,
        &[open([0, 3, 1, 1, 0], Some(2), &[41, 3, 41, 0], 2, 0)],
    );
    execute(&source, 5, &[open([0, 3, 1, 1, 0], None, &[41, 2], 1, 2)]);
}

#[test]
fn whole_callback_loop_work_counts_constant_and_sequential_calls() {
    let source = SOURCE
        .replace(
            "return nested(rounds, trips, stride, early);",
            "let first = relay(trips, stride, early); return first + relay(trips, stride, early);",
        )
        .replace("while index < trips", "while index < 3")
        .replace("index + stride", "index + 1");
    execute(
        &source,
        6,
        &[open([0, 3, 1, 0, 0], Some(6), &[41, 3, 41, 0], 6, 0)],
    );
    execute(&source, 5, &[open([0, 3, 1, 0, 0], None, &[41, 2], 3, 2)]);
    execute(
        &source.replace("while index < 3", "while index < 0"),
        0,
        &[open([0, 3, 1, 0, 0], Some(0), &[], 0, 0)],
    );
}

#[test]
fn whole_callback_loop_work_uses_unsigned_budget_without_wrapping() {
    execute(
        SOURCE,
        u64::MAX,
        &[open(
            [2, 3, 1, 0, 0],
            Some(6),
            &[41, -3, 55, 3, 41, -6, 55, 3, 41, -9],
            8,
            u64::MAX - 8,
        )],
    );
}

#[test]
fn whole_callback_loop_work_reentrant_entry_cannot_reset_its_callers_budget() {
    let project = Project::with_source(SOURCE);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = yir_lower_llvm::native_session::emit_registered_with_work_limits(
        &compiled.yir,
        "counter",
        8,
        14,
    )
    .unwrap();
    let symbol = &bridge.callbacks[0].symbol;
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let definition = llvm.find("define i64 @nuis_fn_argument(").unwrap();
    let insertion = definition + llvm[definition..].find("{\n").unwrap() + 2;
    // Reenter the public ABI while the original callback's stack counter is live.
    // This is a transport isolation probe, not new recursive Nuis admission.
    llvm.insert_str(
        insertion,
        &format!(
            "
  %probe_reenter = icmp eq i64 %arg0, 3
  br i1 %probe_reenter, label %probe_nested, label %probe_original
probe_nested:
  %probe_before = load volatile i64, ptr %nuis_loop_work
  %probe_entries_before = load volatile i64, ptr %nuis_helper_entries
  %probe_args = alloca [5 x i64], align 8
  store [5 x i64] [i64 2, i64 1, i64 1, i64 0, i64 0], ptr %probe_args
  %probe_out = alloca i64, align 8
  %probe_status = call i32 @{symbol}(ptr %probe_args, i64 5, ptr %probe_out, i64 1)
  %probe_value = load i64, ptr %probe_out
  %probe_after = load volatile i64, ptr %nuis_loop_work
  %probe_entries_after = load volatile i64, ptr %nuis_helper_entries
  %probe_unchanged = icmp eq i64 %probe_before, %probe_after
  %probe_entries_unchanged = icmp eq i64 %probe_entries_before, %probe_entries_after
  %probe_budgets_unchanged = and i1 %probe_unchanged, %probe_entries_unchanged
  %probe_completed = icmp eq i32 %probe_status, 0
  %probe_correct = icmp eq i64 %probe_value, 2
  %probe_success = and i1 %probe_completed, %probe_correct
  %probe_safe = and i1 %probe_success, %probe_budgets_unchanged
  br i1 %probe_safe, label %probe_original, label %probe_failure
probe_failure:
  call void @llvm.trap()
  unreachable
probe_original:
"
        ),
    );
    llvm.push_str(&format!(
        "
define i64 @nuis_yir_entry() {{
  %args = alloca [5 x i64], align 8
  store [5 x i64] [i64 2, i64 3, i64 1, i64 0, i64 0], ptr %args
  %out = alloca i64, align 8
  %status = call i32 @{symbol}(ptr %args, i64 5, ptr %out, i64 1)
  %wide = zext i32 %status to i64
  call void @nuis_debug_print_i64(i64 %wide)
  %value = load i64, ptr %out
  call void @nuis_debug_print_i64(i64 %value)
  ret i64 0
}}
"
    ));
    let artifact = nuisc::aot::write_and_link_with_source(
        &project.0.join("main.ns"),
        &project.0.join("out"),
        SOURCE,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&llvm),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    let run = dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "0\n6\n");
}
