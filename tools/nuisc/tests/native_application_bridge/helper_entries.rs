use super::*;
use std::path::Path;
use yir_lower_llvm::native_session::emit_registered_with_work_limits;

const SOURCE: &str = include_str!("helper_entries.ns");
const SENTINEL: i64 = -700;
const PAIR: &[&str] = &[
    "start",
    "selected",
    "__nuis_scalar_branch_0",
    "__nuis_scalar_branch_1",
    "__nuis_scalar_continue_0",
    "pair",
    "argument",
    "leaf",
    "argument",
    "leaf",
];

struct Case {
    callback: usize,
    inputs: Vec<i64>,
    status: Option<i32>,
    value: i64,
    entered: Vec<&'static str>,
    remaining_loop: u64,
}

fn open(inputs: &[i64], value: Option<i64>, entered: &[&'static str]) -> Case {
    Case {
        callback: 0,
        inputs: inputs.to_vec(),
        status: value.map(|_| 0),
        value: value.unwrap_or(SENTINEL),
        entered: entered.to_vec(),
        remaining_loop: 0,
    }
}

fn execute(source: &str, loop_limit: u64, entry_limit: u64, cases: &[Case]) {
    assert!(!cases.is_empty());
    assert!(cases[..cases.len() - 1]
        .iter()
        .all(|case| case.status.is_some()));
    let project = Project::with_source(source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.functions.reverse();
    compiled.yir.nodes.reverse();
    let bridge =
        emit_registered_with_work_limits(&compiled.yir, "counter", loop_limit, entry_limit)
            .unwrap();
    assert_eq!(bridge.helper_entry_limit, entry_limit);
    assert_eq!(bridge.loop_work_limit, loop_limit);
    assert_eq!(
        bridge
            .llvm_ir
            .matches("%nuis_helper_entries = alloca i64")
            .count(),
        3
    );
    assert!(!bridge.llvm_ir.contains("thread_local"));
    assert!(!bridge.llvm_ir.contains("@nuis_helper_entries"));
    let ordinary = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    assert!(!ordinary.contains("%nuis_helper_entries"));
    assert!(!ordinary.contains("native_helper_entry_"));

    // Global state is only test output. Real callback counters remain stack-owned.
    let mut functions = Vec::new();
    let mut llvm = String::new();
    for line in bridge.llvm_ir.lines() {
        if line.starts_with("define ") && line.contains(" @nuis_fn_") {
            let name = line
                .split(" @nuis_fn_")
                .nth(1)
                .unwrap()
                .split('(')
                .next()
                .unwrap();
            functions.push(name.to_owned());
            assert!(line.ends_with("ptr %nuis_loop_work, ptr %nuis_helper_entries) {"));
        }
        if line == "  call void @llvm.trap()" {
            llvm.push_str(
                "  call void @probe_trap(ptr %nuis_helper_entries, ptr %nuis_loop_work)\n",
            );
        }
        llvm.push_str(line);
        llvm.push('\n');
        if line.starts_with("  store i64 %") && line.contains("ptr %nuis_helper_entries,") {
            llvm.push_str(&format!(
                "  call void @probe_entry(i64 {}, ptr %nuis_helper_entries)\n",
                functions.len()
            ));
        }
    }
    assert_eq!(
        llvm.matches("call void @probe_entry(").count(),
        functions.len()
    );
    llvm = llvm.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(PROBES);
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [16 x i64], align 8\n");
    let mut expected = Vec::new();
    let registry = yir_verify::default_registry();
    for (index, case) in cases.iter().enumerate() {
        if case.callback == 0 && case.status == Some(0) {
            let arguments = case
                .inputs
                .iter()
                .zip(&bridge.callbacks[0].arguments)
                .map(|(&value, kind)| match kind {
                    ScalarKind::Bool => Value::Bool(value != 0),
                    ScalarKind::I64 => Value::Int(value),
                    _ => panic!("unexpected fixture input"),
                })
                .collect();
            let (reference, _) =
                ApplicationSession::open_registered(&compiled.yir, &registry, "counter", arguments)
                    .unwrap();
            assert_eq!(state_words(reference.state()), vec![case.value as u64]);
        }
        llvm.push_str(&format!(
            "  store volatile i64 {SENTINEL}, ptr @probe_out\n"
        ));
        for (slot, value) in case.inputs.iter().enumerate() {
            llvm.push_str(&format!("  %arg_{index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {value}, ptr %arg_{index}_{slot}\n"));
        }
        llvm.push_str(&format!("  %status_{index} = call i32 @{}(ptr %args, i64 {}, ptr @probe_out, i64 1)\n  %wide_{index} = zext i32 %status_{index} to i64\n  call void @nuis_debug_print_i64(i64 %wide_{index})\n  %out_{index} = load volatile i64, ptr @probe_out\n  call void @nuis_debug_print_i64(i64 %out_{index})\n  %flush_{index} = call i32 @fflush(ptr null)\n", bridge.callbacks[case.callback].symbol, case.inputs.len()));
        for (entry, name) in case.entered.iter().enumerate() {
            let id = functions
                .iter()
                .position(|f| {
                    f == name || (*name == "iteration" && f.starts_with("__nuis_scalar_iteration_"))
                })
                .unwrap_or_else(|| panic!("missing {name}: {functions:?}"))
                + 1;
            expected.extend([id as i64, (entry_limit - entry as u64 - 1) as i64]);
        }
        if let Some(status) = case.status {
            expected.extend([i64::from(status), case.value]);
        } else {
            expected.extend([
                73,
                (entry_limit - case.entered.len() as u64) as i64,
                case.remaining_loop as i64,
                SENTINEL,
            ]);
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
    let selected_ir = llvm
        .split("define i64 @nuis_fn_selected(")
        .nth(1)
        .unwrap()
        .split("\n}")
        .next()
        .unwrap();
    let trap = cases.last().unwrap().status.is_none();
    assert_eq!(
        run.status.success(),
        !trap,
        "stdout: {}\nstderr: {}\nselected: {selected_ir}",
        String::from_utf8_lossy(&run.stdout),
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
    assert_eq!(
        actual, expected,
        "helper ids: {functions:?}\nselected: {selected_ir}"
    );
}

const PROBES: &str = "
@probe_out = internal global i64 -700, align 8
declare i32 @fflush(ptr)
define void @probe_entry(i64 %id, ptr %entries) {
  call void @nuis_debug_print_i64(i64 %id)
  %remaining = load i64, ptr %entries
  call void @nuis_debug_print_i64(i64 %remaining)
  ret void
}
define void @probe_trap(ptr %entries, ptr %work) {
  call void @nuis_debug_print_i64(i64 73)
  %remaining = load i64, ptr %entries
  call void @nuis_debug_print_i64(i64 %remaining)
  %loops = load i64, ptr %work
  call void @nuis_debug_print_i64(i64 %loops)
  %out = load volatile i64, ptr @probe_out
  call void @nuis_debug_print_i64(i64 %out)
  %flushed = call i32 @fflush(ptr null)
  ret void
}
";

#[test]
fn helper_entries_share_exact_budget_and_reset_at_lifecycle_roots() {
    let mut cases = vec![open(&[10, 0], Some(12), PAIR)];
    for (callback, name, prior) in [(1, "step", 12), (1, "step", 24), (2, "stop", 36)] {
        let mut case = open(&[prior, 10, 0], Some(prior + 12), PAIR);
        case.callback = callback;
        case.entered[0] = name;
        cases.push(case);
    }
    cases.push(open(&[10, 0], Some(12), PAIR));
    execute(SOURCE, 0, 10, &cases);
}

#[test]
fn helper_entries_reject_before_body_but_after_argument_evaluation() {
    execute(SOURCE, 0, 9, &[open(&[10, 0], None, &PAIR[..9])]);
    execute(SOURCE, 0, 0, &[open(&[10, 0], None, &[])]);
}

#[test]
fn helper_entries_skip_unselected_paths_and_validate_transport_first() {
    execute(
        SOURCE,
        0,
        4,
        &[
            open(&[10, 1], Some(10), &PAIR[..4]),
            open(&[10, 0], None, &PAIR[..4]),
        ],
    );
    let mut wrong_shape = open(&[10], Some(SENTINEL), &[]);
    wrong_shape.status = Some(1);
    let mut wrong_scalar = open(&[10, 2], Some(SENTINEL), &[]);
    wrong_scalar.status = Some(2);
    execute(SOURCE, 0, 0, &[wrong_shape, wrong_scalar]);
}

#[test]
fn helper_entries_use_unsigned_limits_without_wrapping() {
    execute(SOURCE, 0, u64::MAX, &[open(&[10, 0], Some(12), PAIR)]);
}

#[test]
fn helper_entries_count_loop_free_roots_and_unused_results() {
    let root = SOURCE.replacen(
        "return State { value: selected(value, skip) };",
        "return State { value: value };",
        1,
    );
    execute(&root, 0, 1, &[open(&[10, 0], Some(10), &["start"])]);
    execute(&root, 0, 0, &[open(&[10, 0], None, &[])]);
    let unused = SOURCE.replace(
        "return pair(value);",
        "let unused = pair(value); return value;",
    );
    execute(&unused, 0, 10, &[open(&[10, 0], Some(10), PAIR)]);
    execute(&unused, 0, 9, &[open(&[10, 0], None, &PAIR[..9])]);
}

#[test]
fn helper_entries_include_scoped_iterations_without_resetting_loop_reservations() {
    let source = include_str!("loop_work.ns");
    let calls = [
        "start",
        "selected",
        "__nuis_scalar_branch_0",
        "__nuis_scalar_branch_1",
        "__nuis_scalar_continue_0",
        "nested",
        "iteration",
        "argument",
        "relay",
        "leaf",
        "iteration",
        "argument",
        "relay",
        "leaf",
    ];
    execute(source, 8, 14, &[open(&[2, 3, 1, 0, 0], Some(6), &calls)]);
    let mut rejected = open(&[2, 3, 1, 0, 0], None, &calls[..13]);
    rejected.remaining_loop = 3;
    execute(source, 8, 13, &[rejected]);
    let mut rejected = open(&[2, 3, 1, 0, 0], None, &calls[..10]);
    rejected.remaining_loop = 3;
    execute(source, 8, 10, &[rejected]);
    // Per-loop admission still traps after the containing function entry, before debit.
    let mut invalid = open(&[65537, 3, 1, 0, 0], None, &calls[..6]);
    invalid.remaining_loop = 8;
    execute(source, 8, 14, &[invalid]);
}
