use super::*;

const SOURCE: &str = "mod cpu Main {
    struct State { value: i64 }
    @noinline fn checked(value: i64, divisor: i64) -> i64 { return value / divisor; }
    @noinline fn predicate(divisor: i64) -> bool { return checked(8, divisor) > 1; }
    fn selected(divisor: i64) -> i64 { if predicate(divisor) { return 41; } return 42; }
    fn start(divisor: i64) -> State { return State { value: selected(divisor) }; }
    fn step(state: State) -> State { return state; }
    fn stop(state: State) -> State { return state; }
    fn main() -> i64 { return 0; }
}";
const CALLS: &[&str] = &["start", "selected", "predicate", "checked"];

#[test]
fn atomic_returns_keep_one_predicate_evaluation_and_exact_remaining_call_budget() {
    execute(
        SOURCE,
        0,
        4,
        &[
            open(&[2], Some(41), CALLS),
            open(&[8], Some(42), CALLS),
            open(&[2], Some(41), CALLS),
            open(&[0], None, CALLS),
        ],
    );
    execute(SOURCE, 0, 3, &[open(&[2], None, &CALLS[..3])]);
    // Equal arms are not permission to discard a fallible predicate.
    let equal = SOURCE.replace("return 42;", "return 41;");
    execute(
        &equal,
        0,
        4,
        &[open(&[8], Some(41), CALLS), open(&[0], None, CALLS)],
    );
}

#[test]
fn scalar_elision_keeps_nontrivial_fallback_arguments_inside_the_guard() {
    let source = SOURCE.replace(
        "if predicate(divisor) { return 41; } return 42;",
        "if divisor == 0 { return 41; } return checked(8, checked(2, divisor));",
    );
    let skipped = ["start", "selected", "__nuis_scalar_branch_0"];
    let taken = [
        "start",
        "selected",
        "__nuis_scalar_branch_0",
        "checked",
        "checked",
    ];
    execute(
        &source,
        0,
        5,
        &[
            open(&[0], Some(41), &skipped),
            open(&[2], Some(8), &taken),
            open(&[4], None, &taken),
        ],
    );
    // Argument evaluation enters the inner call before the outer call's debit.
    execute(&source, 0, 4, &[open(&[2], None, &taken[..4])]);
}

#[test]
fn ready_bool_and_flat_values_select_exact_snapshots_without_private_entries() {
    let source = SOURCE.replace(
        "fn selected(divisor: i64) -> i64 { if predicate(divisor) { return 41; } return 42; }",
        "struct Packet { value: i64, marker: i64 }
        @noinline fn choose_flag(flag: bool, yes: bool, no: bool) -> bool {
            if flag { return yes; } return no;
        }
        @noinline fn choose_packet(flag: bool, yes: Packet, no: Packet) -> Packet {
            if flag { return yes; } else { return no; }
        }
        @noinline fn selected(divisor: i64) -> i64 {
            let yes = Packet { value: 11, marker: 33 };
            let no = Packet { value: 44, marker: 77 };
            let saved = yes;
            let flag = choose_flag(divisor == 0, true, false);
            let packet = choose_packet(flag, yes, no);
            return packet.value + packet.marker + saved.marker;
        }",
    );
    let calls = ["start", "selected", "choose_flag", "choose_packet"];
    execute(
        &source,
        0,
        4,
        &[
            open(&[0], Some(77), &calls),
            open(&[1], Some(154), &calls),
            open(&[-1], Some(154), &calls),
            open(&[0], Some(77), &calls),
        ],
    );
    execute(&source, 0, 3, &[open(&[0], None, &calls[..3])]);
}
