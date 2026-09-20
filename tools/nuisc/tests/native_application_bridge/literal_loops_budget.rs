use super::*;

const SOURCE: &str = include_str!("literal_loops_budget.ns");
const OUTER: &str = "__nuis_scalar_iteration_0";
const INNER: &str = "__nuis_scalar_iteration_1";

#[test]
fn literal_nested_loops_share_exact_loop_and_helper_entry_budgets() {
    let mut trace = vec!["start", "selected"];
    for _ in 0..2 {
        trace.push(OUTER);
        for _ in 0..3 {
            trace.extend([INNER, "identity"]);
        }
    }
    assert_eq!(trace.len(), 16);
    execute(
        SOURCE,
        8,
        16,
        &[
            open(&[2, 3], Some(12), &trace),
            open(&[2, 3], Some(12), &trace),
        ],
    );
    // Outer reserves two trips; each selected inner invocation reserves three.
    let mut exhausted = open(&[2, 3], None, &trace[..10]);
    exhausted.remaining_loop = 2;
    execute(SOURCE, 7, 16, &[exhausted]);
    execute(SOURCE, 8, 15, &[open(&[2, 3], None, &trace[..15])]);
    let mut before_inner = open(&[2, 3], None, &trace[..2]);
    before_inner.remaining_loop = 6;
    execute(SOURCE, 8, 2, &[before_inner]);
}

#[test]
fn literal_nested_zero_trip_and_counter_only_loops_do_not_invent_helper_entries() {
    execute(
        SOURCE,
        0,
        2,
        &[open(&[0, 65537], Some(0), &["start", "selected"])],
    );
    execute(
        SOURCE,
        2,
        4,
        &[open(&[2, 0], Some(0), &["start", "selected", OUTER, OUTER])],
    );
    let source = SOURCE.replace("let total = total + identity(child);", "");
    // The counter-only inner body remains the existing metadata loop: no private
    // iteration helper or identity call consumes the separate entry budget.
    execute(
        &source,
        8,
        4,
        &[open(&[2, 3], Some(0), &["start", "selected", OUTER, OUTER])],
    );
    let mut exhausted = open(&[2, 3], None, &["start", "selected", OUTER, OUTER]);
    exhausted.remaining_loop = 2;
    execute(&source, 7, 4, &[exhausted]);

    let persistent = SOURCE
        .replace(
            "    let total: i64 = 0;",
            "    let total: i64 = 0;\n    let child: i64 = 0;",
        )
        .replace("      let child: i64 = 0;\n", "");
    let mut trace = vec!["start", "selected", OUTER];
    for _ in 0..3 {
        trace.extend([INNER, "identity"]);
    }
    trace.push(OUTER);
    // The second child invocation starts at three, so its zero trips debit no
    // loop work or child entries. Its final index crosses the outer backedge.
    execute(&persistent, 5, 10, &[open(&[2, 3], Some(6), &trace)]);
}

#[test]
fn literal_child_single_bool_guard_uses_scoped_not_metadata_predicates() {
    let source = SOURCE
        .replace(
            "let index: i64 = 0;",
            "let index: i64 = 0; let flag = width > 0;",
        )
        .replace(
            "let total = total + identity(child);",
            "if flag { let total = total + identity(child); }",
        );
    let mut trace = vec!["start", "selected"];
    for _ in 0..2 {
        trace.push(OUTER);
        for _ in 0..3 {
            trace.extend([INNER, "__nuis_buffer_branch_0", "identity"]);
        }
    }
    assert_eq!(trace.len(), 22);
    execute(&source, 8, 22, &[open(&[2, 3], Some(12), &trace)]);
}
