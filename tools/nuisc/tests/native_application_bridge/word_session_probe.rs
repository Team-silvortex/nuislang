use super::*;
use std::os::unix::process::ExitStatusExt;
use yir_lower_llvm::native_session::{emit_registered_with_literal_prints, LiteralPrintPolicy};

#[derive(Clone)]
pub(super) struct Row {
    pub input: Vec<i64>,
    pub state: Vec<i64>,
    pub trace: Vec<i64>,
}

#[allow(dead_code)]
pub(super) struct Failure {
    pub accepted: Row,
    pub input: Vec<i64>,
    pub before_failure: Vec<i64>,
}

pub(super) struct Sequence {
    // The first row opens the session; subsequent rows are accepted events.
    pub rows: Vec<Row>,
}

pub(super) struct SequenceFailure {
    pub accepted: Sequence,
    pub input: Vec<i64>,
    pub before_failure: Vec<i64>,
}

fn callback(
    llvm: &mut String,
    bridge: &NativeSessionBridge,
    input: &[i64],
    state_width: usize,
    role: usize,
    id: usize,
) {
    if role != 0 {
        for slot in 0..state_width {
            llvm.push_str(&format!("  %lastp{id}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %last{id}_{slot} = load volatile i64, ptr %lastp{id}_{slot}\n  %saved{id}_{slot} = getelementptr i64, ptr %input, i64 {slot}\n  store volatile i64 %last{id}_{slot}, ptr %saved{id}_{slot}\n"));
        }
    }
    for (slot, word) in input.iter().enumerate() {
        let index = slot + if role == 0 { 0 } else { state_width };
        llvm.push_str(&format!("  %arg{id}_{slot} = getelementptr i64, ptr %input, i64 {index}\n  store volatile i64 {word}, ptr %arg{id}_{slot}\n"));
    }
    llvm.push_str("  store volatile i64 91, ptr %output\n  store volatile i64 92, ptr %canary\n");
    let count = input.len() + if role == 0 { 0 } else { state_width };
    llvm.push_str(&format!("  %status{id} = call i32 @{}(ptr %input, i64 {count}, ptr %out, i64 {state_width})\n  %wide{id} = zext i32 %status{id} to i64\n  call void @word_print(i64 %wide{id})\n", bridge.callbacks[role].symbol));
    for slot in 0..state_width + 2 {
        llvm.push_str(&format!("  %p{id}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  %v{id}_{slot} = load volatile i64, ptr %p{id}_{slot}\n  call void @word_print(i64 %v{id}_{slot})\n"));
    }
}

fn accepted(row: &Row) -> Vec<i64> {
    let mut result = row.trace.clone();
    result.extend([0, 91]);
    result.extend(&row.state);
    result.push(92);
    result
}

fn driver(
    bridge: &NativeSessionBridge,
    rows: &[Sequence],
    failures: &[SequenceFailure],
    observer: &str,
) -> String {
    let state_width = rows[0].rows[0].state.len();
    let input_width = rows
        .iter()
        .flat_map(|sequence| sequence.rows.iter().map(|row| &row.input))
        .chain(failures.iter().flat_map(|f| {
            f.accepted
                .rows
                .iter()
                .map(|row| &row.input)
                .chain(std::iter::once(&f.input))
        }))
        .map(Vec::len)
        .max()
        .unwrap()
        + state_width;
    assert!(rows
        .iter()
        .all(|s| !s.rows.is_empty() && s.rows.iter().all(|r| r.state.len() == state_width)));
    assert!(failures.iter().all(|f| !f.accepted.rows.is_empty()
        && f.accepted.rows.iter().all(|r| r.state.len() == state_width)));
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let start = aggregate_values::definition(&llvm, "answer_work");
    let at = start + llvm[start..].find("{\n").unwrap() + 2;
    // Observe real callee arguments without replacing any source computation.
    llvm.insert_str(at, observer);
    llvm = llvm.replace("call void @nuis_debug_print_i64(", "call void @word_print(");
    if !llvm.contains("declare i32 @fflush(") {
        llvm.push_str("\ndeclare i32 @fflush(ptr)\n");
    }
    llvm.push_str("\ndefine void @word_print(i64 %word) {\n  call void @nuis_debug_print_i64(i64 %word)\n  %flushed = call i32 @fflush(ptr null)\n  ret void\n}\n@word_env = private constant [17 x i8] c\"NUIS_EXIT_PREFIX\\00\"\ndeclare ptr @getenv(ptr)\ndeclare i32 @atoi(ptr)\n");
    llvm.push_str(&format!("\ndefine i64 @nuis_yir_entry() {{\n  %input = alloca [{input_width} x i64], align 8\n  %output = alloca [{} x i64], align 8\n  %out = getelementptr i64, ptr %output, i64 1\n  %canary = getelementptr i64, ptr %output, i64 {}\n  %env = call ptr @getenv(ptr @word_env)\n  %mode = call i32 @atoi(ptr %env)\n  switch i32 %mode, label %reject [\n    i32 0, label %healthy\n", state_width + 2, state_width + 1));
    for i in 0..failures.len() {
        llvm.push_str(&format!("    i32 {}, label %bad{i}\n", i + 1));
    }
    llvm.push_str("  ]\nreject:\n  call void @llvm.trap()\n  unreachable\nhealthy:\n");
    let mut id = 0;
    for sequence in rows {
        for (index, row) in sequence.rows.iter().enumerate() {
            callback(
                &mut llvm,
                bridge,
                &row.input,
                state_width,
                usize::from(index != 0),
                id,
            );
            id += 1;
        }
        callback(&mut llvm, bridge, &[], state_width, 2, id);
        id += 1;
    }
    llvm.push_str("  ret i64 0\n");
    for (i, failure) in failures.iter().enumerate() {
        llvm.push_str(&format!("bad{i}:\n"));
        for (index, row) in failure.accepted.rows.iter().enumerate() {
            callback(
                &mut llvm,
                bridge,
                &row.input,
                state_width,
                usize::from(index != 0),
                id,
            );
            id += 1;
        }
        llvm.push_str("  call void @word_print(i64 -70001)\n");
        callback(&mut llvm, bridge, &failure.input, state_width, 1, id);
        id += 1;
        llvm.push_str("  call void @word_print(i64 -70002)\n  ret i64 0\n");
    }
    llvm.push_str("}\n");
    llvm
}

#[allow(dead_code)]
pub(super) fn check(source: &str, rows: &[(Row, Row)], failures: &[Failure], observer: &str) {
    let sequences = rows
        .iter()
        .map(|(a, b)| Sequence {
            rows: vec![a.clone(), b.clone()],
        })
        .collect::<Vec<_>>();
    let failures = failures
        .iter()
        .map(|f| SequenceFailure {
            accepted: Sequence {
                rows: vec![f.accepted.clone()],
            },
            input: f.input.clone(),
            before_failure: f.before_failure.clone(),
        })
        .collect::<Vec<_>>();
    check_sequences(source, &sequences, &failures, observer);
}

pub(super) fn check_sequences(
    source: &str,
    rows: &[Sequence],
    failures: &[SequenceFailure],
    observer: &str,
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
        let llvm = driver(&bridge, rows, failures, observer);
        assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
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
        for mode in 0..=failures.len() {
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
                for sequence in rows {
                    for row in &sequence.rows {
                        expected.extend(accepted(row));
                    }
                    expected.extend([0, 91]);
                    expected.extend(&sequence.rows.last().unwrap().state);
                    expected.push(92);
                }
            } else {
                assert!(
                    matches!(run.status.signal(), Some(4 | 5)),
                    "{:?}",
                    run.status
                );
                let failure = &failures[mode - 1];
                for row in &failure.accepted.rows {
                    expected.extend(accepted(row));
                }
                expected.push(-70001);
                expected.extend(&failure.before_failure);
            }
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|n| n.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "reversed={reversed} mode={mode}");
        }
    }
}
