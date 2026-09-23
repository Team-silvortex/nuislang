use super::*;

const SOURCE: &str = "mod cpu Main {
    struct State { value: i64 }
    @noinline fn identity(value: i64) -> i64 { return value; }
    fn selected(limit: i64, value: i64) -> i64 {
        let index = 0;
        while index < limit { return identity(value); let index = index + 1; }
        return identity(5);
    }
    fn start(limit: i64, value: i64) -> State {
        let first = selected(limit, value);
        return State { value: first + selected(1, value) };
    }
    fn step(state: State) -> State { return state; }
    fn stop(state: State) -> State { return state; }
    fn main() -> i64 { return 0; }
}";

const CALL: &[&str] = &[
    "selected",
    "__nuis_scalar_iteration_0",
    "identity",
    "__nuis_scalar_branch_0",
];

#[test]
fn counted_returns_keep_shared_reservations_entries_and_atomic_callback_output() {
    let mut trace = vec!["start"];
    trace.extend(CALL);
    trace.extend(CALL);
    execute(
        SOURCE,
        9,
        9,
        &[
            open(&[8, 3], Some(6), &trace),
            open(&[8, 3], Some(6), &trace),
        ],
    );
    // Returning on the first iteration does not refund any of the eight trips.
    execute(SOURCE, 8, 9, &[open(&[8, 3], None, &trace[..6])]);
    // Even the unselected private guard is an actual call, but not its suffix.
    execute(SOURCE, 9, 8, &[open(&[8, 3], None, &trace[..8])]);
    let mut zero = vec!["start", "selected", "__nuis_scalar_branch_0", "identity"];
    zero.extend(CALL);
    execute(SOURCE, 1, 8, &[open(&[0, 3], Some(8), &zero)]);
    let mut preflight = open(&[65537, 3], None, &["start", "selected"]);
    preflight.remaining_loop = 9;
    execute(SOURCE, 9, 9, &[preflight]);
}

#[test]
fn counted_return_payload_traps_leave_callback_output_untouched() {
    for (expression, value) in [
        ("identity(value) / value", 0),
        ("identity(value) / -1", i64::MIN),
    ] {
        let source = SOURCE.replace("return identity(value);", &format!("return {expression};"));
        let mut failed = open(
            &[8, value],
            None,
            &["start", "selected", "__nuis_scalar_iteration_0", "identity"],
        );
        failed.remaining_loop = 1;
        execute(&source, 9, 11, &[failed]);
    }
}
