use super::*;

const SOURCE: &str = include_str!("literal_loops_budget.ns");
const OUTER: &str = "__nuis_scalar_iteration_0";
const INNER: &str = "__nuis_scalar_iteration_1";

fn breaking_source() -> String {
    SOURCE
        .replace(
            "let total = total + identity(child);",
            "let total = total + identity(child); break;",
        )
        .replace(
            "      }\n    }",
            "      }\n      let total = total + child;\n    }",
        )
}

#[test]
fn value_loop_exits_reserve_full_work_but_charge_only_entered_helpers() {
    let source = breaking_source();
    let trace = [
        "start", "selected", OUTER, INNER, "identity", OUTER, INNER, "identity",
    ];
    execute(
        &source,
        8,
        8,
        &[
            open(&[2, 3], Some(4), &trace),
            open(&[2, 3], Some(4), &trace),
        ],
    );
    // Breaking after one child trip does not refund its three reserved trips.
    let mut exhausted = open(&[2, 3], None, &trace[..6]);
    exhausted.remaining_loop = 2;
    execute(&source, 7, 8, &[exhausted]);
    execute(&source, 8, 7, &[open(&[2, 3], None, &trace[..7])]);
    let mut before_outer = open(&[2, 3], None, &trace[..2]);
    before_outer.remaining_loop = 6;
    execute(&source, 8, 2, &[before_outer]);
}

#[test]
fn value_loop_exits_parent_break_does_not_refund_unentered_iterations() {
    let source = breaking_source().replace(
        "      let total = total + child;",
        "      let total = total + child; break;",
    );
    let trace = ["start", "selected", OUTER, INNER, "identity"];
    execute(&source, 6, 5, &[open(&[3, 3], Some(2), &trace)]);
    let mut exhausted = open(&[3, 3], None, &trace[..3]);
    exhausted.remaining_loop = 2;
    execute(&source, 5, 5, &[exhausted]);
    execute(&source, 0, 2, &[open(&[0, 65537], Some(0), &trace[..2])]);
    let following = source.replace(
        "return total;",
        "
        let tail = 0;
        while tail < rounds {
            let tail = tail + 1;
            let total = total + identity(tail);
        }
        return total;",
    );
    let mut followed = trace.to_vec();
    for _ in 0..3 {
        followed.extend(["__nuis_scalar_iteration_2", "identity"]);
    }
    // A one-trip child leaves no unused child work. The outer loop still keeps
    // its two unentered reservations when the following loop asks for three.
    execute(&following, 7, 11, &[open(&[3, 1], Some(8), &followed)]);
    let mut exhausted = open(&[3, 1], None, &trace);
    exhausted.remaining_loop = 2;
    execute(&following, 6, 11, &[exhausted]);
}

#[test]
fn value_loop_exits_continue_keeps_exact_iteration_and_entry_debits() {
    let source = SOURCE.replace(
        "let total = total + identity(child);",
        "let total = total + identity(child); continue;",
    );
    let mut trace = vec!["start", "selected"];
    for _ in 0..2 {
        trace.push(OUTER);
        for _ in 0..3 {
            trace.extend([INNER, "identity"]);
        }
    }
    assert_eq!(trace.len(), 16);
    execute(&source, 8, 16, &[open(&[2, 3], Some(12), &trace)]);
    let mut exhausted = open(&[2, 3], None, &trace[..10]);
    exhausted.remaining_loop = 2;
    execute(&source, 7, 16, &[exhausted]);
    execute(&source, 8, 15, &[open(&[2, 3], None, &trace[..15])]);
    let counter_only = source
        .replace("let total = total + identity(child);", "")
        .replace("return total;", "return index;");
    let entered = trace
        .iter()
        .copied()
        .filter(|name| *name != "identity")
        .collect::<Vec<_>>();
    execute(&counter_only, 8, 10, &[open(&[2, 3], Some(2), &entered)]);
}
