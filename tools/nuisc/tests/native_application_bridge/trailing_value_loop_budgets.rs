use super::*;

const SOURCE: &str = include_str!("literal_loops_budget.ns");
const OUTER: &str = "__nuis_scalar_iteration_0";
const INNER: &str = "__nuis_scalar_iteration_1";

fn source(child_effects: &str, suffix: &str) -> String {
    SOURCE
        .replace("      let index = index + 1;", "")
        .replace("        let child = child + 1;", "")
        .replace(
            "let total = total + identity(child);",
            &format!("{child_effects} let child = child + 1;"),
        )
        .replace(
            "      }\n    }",
            &format!("      }}\n      {suffix}\n      let index = index + 1;\n    }}"),
        )
}

#[test]
fn trailing_value_loops_reserve_full_work_without_advancing_break_indices() {
    let text = source(
        "let total = total + identity(child + 7); break;",
        "let total = total + child;",
    );
    let trace = [
        "start", "selected", OUTER, INNER, "identity", OUTER, INNER, "identity",
    ];
    execute(
        &text,
        8,
        8,
        &[
            open(&[2, 3], Some(14), &trace),
            open(&[2, 3], Some(14), &trace),
        ],
    );
    let mut exhausted = open(&[2, 3], None, &trace[..6]);
    exhausted.remaining_loop = 2;
    execute(&text, 7, 8, &[exhausted]);
    execute(&text, 8, 7, &[open(&[2, 3], None, &trace[..7])]);
    // A flag-only break must still use the existing canonical struct transport.
    let counter = source("break;", "").replace("return total;", "return index;");
    execute(
        &counter,
        8,
        6,
        &[open(
            &[2, 3],
            Some(2),
            &["start", "selected", OUTER, INNER, OUTER, INNER],
        )],
    );
    let following = source("let total = total + identity(child + 7); break;", "break;")
        .replace("return total;", "let tail = 0; while tail < rounds { let total = total + identity(tail); let tail = tail + 1; } return total + index;");
    let mut followed = trace[..5].to_vec();
    for _ in 0..3 {
        followed.extend(["__nuis_scalar_iteration_2", "identity"]);
    }
    // Neither the child break nor the parent break refunds its full reservation.
    execute(&following, 7, 11, &[open(&[3, 1], Some(10), &followed)]);
    let mut exhausted = open(&[3, 1], None, &trace[..5]);
    exhausted.remaining_loop = 2;
    execute(&following, 6, 11, &[exhausted]);
}

#[test]
fn trailing_value_continue_debits_one_step_and_only_entered_helpers() {
    let text = source(
        "let total = total + identity(child); let child = child + 1; continue;",
        "",
    );
    let mut trace = vec!["start", "selected"];
    for _ in 0..2 {
        trace.push(OUTER);
        for _ in 0..3 {
            trace.extend([INNER, "identity"]);
        }
    }
    execute(&text, 8, 16, &[open(&[2, 3], Some(6), &trace)]);
    let mut exhausted = open(&[2, 3], None, &trace[..10]);
    exhausted.remaining_loop = 2;
    execute(&text, 7, 16, &[exhausted]);
    execute(&text, 8, 15, &[open(&[2, 3], None, &trace[..15])]);
    execute(&text, 0, 2, &[open(&[0, 65537], Some(0), &trace[..2])]);
}
