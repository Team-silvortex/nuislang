use super::*;

fn source(helpers: &str, body: &str) -> String {
    format!(
        "mod cpu Main {{
        struct State {{ value: i64 }}
        @noinline fn checked(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
        {helpers}
        @noinline fn selected(divisor: i64) -> i64 {{ {body} }}
        fn start(divisor: i64) -> State {{ return State {{ value: selected(divisor) }}; }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

#[test]
fn terminal_forwarding_folds_single_use_boolean_and_record_returns() {
    let boolean = source(
        "@noinline fn truth(divisor: i64) -> bool {
            if divisor == 0 { return true; } return checked(8, divisor) > 1;
        }",
        "let value = truth(divisor); if value { return 41; } return 42;",
    );
    let calls = [
        "start",
        "selected",
        "truth",
        "__nuis_scalar_branch_0",
        "checked",
    ];
    execute(
        &boolean,
        0,
        5,
        &[
            open(&[0], Some(41), &calls[..4]),
            open(&[2], Some(41), &calls),
            open(&[16], Some(42), &calls),
            open(&[-2], Some(42), &calls),
        ],
    );
    execute(&boolean, 0, 4, &[open(&[2], None, &calls[..4])]);

    let record = source(
        "struct Packet { value: i64, marker: i64 }
        @noinline fn left(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn right(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn packet(divisor: i64, saved: Packet) -> Packet {
            if divisor == 0 { return saved; }
            return Packet { marker: right(4, divisor), value: left(8, divisor) };
        }",
        "let saved = Packet { value: 8, marker: 4 };
        let value = packet(divisor, saved);
        return value.value + value.marker + saved.value;",
    );
    // Construction follows source order, independently of declared field order.
    let calls = [
        "start",
        "selected",
        "packet",
        "__nuis_scalar_branch_0",
        "right",
        "left",
    ];
    execute(
        &record,
        0,
        6,
        &[
            open(&[0], Some(20), &calls[..4]),
            open(&[2], Some(14), &calls),
            open(&[-2], Some(2), &calls),
            open(&[0], Some(20), &calls[..4]),
        ],
    );
    execute(&record, 0, 5, &[open(&[2], None, &calls[..5])]);
}

#[test]
fn statement_suffixes_keep_unused_calls_argument_order_and_failure_sentinels() {
    let source = source(
        "@noinline fn right(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn left(value: i64, divisor: i64) -> i64 { return value / divisor; }",
        "if divisor == 0 { return 41; }
        let unused = right(4, divisor);
        let denominator = checked(2, divisor);
        return left(8, denominator);",
    );
    let calls = [
        "start",
        "selected",
        "__nuis_scalar_branch_0",
        "right",
        "checked",
        "left",
    ];
    execute(
        &source,
        0,
        6,
        &[
            open(&[0], Some(41), &calls[..3]),
            open(&[1], Some(4), &calls),
            open(&[2], Some(8), &calls),
            open(&[-2], Some(-8), &calls),
            open(&[4], None, &calls),
        ],
    );
    execute(&source, 0, 5, &[open(&[2], None, &calls[..5])]);
    let unused_trap = source.replace("right(4, divisor)", "right(4, divisor - 2)");
    execute(&unused_trap, 0, 6, &[open(&[2], None, &calls[..4])]);
}

#[test]
fn statement_suffixes_keep_boolean_and_record_snapshots() {
    let boolean = source(
        "@noinline fn truth(divisor: i64) -> bool {
            if divisor == 0 { return true; }
            let next = checked(8, divisor); return next > 1;
        }",
        "let value = truth(divisor); if value { return 41; } return 42;",
    );
    let calls = [
        "start",
        "selected",
        "truth",
        "__nuis_scalar_branch_0",
        "checked",
    ];
    execute(
        &boolean,
        0,
        5,
        &[
            open(&[0], Some(41), &calls[..4]),
            open(&[2], Some(41), &calls),
            open(&[-2], Some(42), &calls),
        ],
    );
    execute(&boolean, 0, 4, &[open(&[2], None, &calls[..4])]);
    let record = source(
        "struct Packet { value: i64, marker: i64 }
        @noinline fn packet(divisor: i64, saved: Packet) -> Packet {
            if divisor == 0 { return saved; }
            let next = checked(saved.value, divisor);
            let result = Packet { value: next, marker: saved.marker };
            return result;
        }",
        "let saved = Packet { value: 8, marker: 4 };
        let result = packet(divisor, saved);
        return result.value + result.marker + saved.value;",
    );
    let calls = [
        "start",
        "selected",
        "packet",
        "__nuis_scalar_branch_0",
        "checked",
    ];
    execute(
        &record,
        0,
        5,
        &[
            open(&[0], Some(20), &calls[..4]),
            open(&[2], Some(16), &calls),
            open(&[-2], Some(8), &calls),
        ],
    );
    execute(&record, 0, 4, &[open(&[2], None, &calls[..4])]);
}

#[test]
fn statement_suffix_name_collisions_keep_separate_lexical_bindings() {
    let source = source(
        "struct Packet { value: i64, marker: i64 }
        @noinline fn right(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn left(value: i64, divisor: i64) -> i64 { return value / divisor; }",
        "if divisor == 0 { return 41; } else {
            let result = Packet { value: right(8, divisor), marker: 2 };
            let unused = result.value + result.marker;
        }
        let result = left(16, divisor); return result;",
    );
    let calls = [
        "start",
        "selected",
        "__nuis_scalar_branch_0",
        "right",
        "left",
    ];
    execute(
        &source,
        0,
        5,
        &[
            open(&[0], Some(41), &calls[..3]),
            open(&[2], Some(8), &calls),
            open(&[-2], Some(-8), &calls),
        ],
    );
    execute(&source, 0, 4, &[open(&[2], None, &calls[..4])]);
}

#[test]
fn hygienic_continuations_preserve_loop_updates_and_work_reservations() {
    let local = source(
        "",
        "if divisor == 0 { return 41; } else {
            let index = 0; let result = 0;
            while index < 2 { let index = index + 1; let result = result + index; }
            let unused = checked(12, result);
        }
        let result = checked(8, divisor); return result;",
    );
    let calls = [
        "start",
        "selected",
        "__nuis_scalar_branch_0",
        "checked",
        "checked",
    ];
    execute(
        &local,
        2,
        5,
        &[
            open(&[0], Some(41), &calls[..3]),
            open(&[2], Some(4), &calls),
            open(&[-2], Some(-4), &calls),
        ],
    );
    execute(&local, 2, 4, &[open(&[2], None, &calls[..4])]);
    let mut exhausted = open(&[2], None, &calls[..3]);
    exhausted.remaining_loop = 1;
    execute(&local, 1, 5, &[exhausted]);

    let outer = source(
        "",
        "let index = 0; let result = 0;
        if divisor == 0 { return 41; } else {
            while index < 1 { let index = index + 1; let result = result + index; }
        }
        while index < 3 { let index = index + 1; let result = result + index; }
        return checked(result, divisor);",
    );
    execute(
        &outer,
        3,
        4,
        &[
            open(&[0], Some(41), &calls[..3]),
            open(&[2], Some(3), &calls[..4]),
            open(&[-2], Some(-3), &calls[..4]),
        ],
    );
    let mut exhausted = open(&[2], None, &calls[..3]);
    exhausted.remaining_loop = 1;
    execute(&outer, 2, 4, &[exhausted]);
}

#[test]
fn hygienic_continuations_keep_bool_and_record_backedges() {
    for (helpers, seed, update, observation) in [
        (
            "@noinline fn probe(flag: bool) -> i64 { if flag { return 2; } return 0; }",
            "false",
            "index > 1",
            "result",
        ),
        (
            "struct Packet { result: i64, marker: i64 }
         @noinline fn probe(value: i64) -> i64 { if value == 3 { return 2; } return 0; }",
            "Packet { result: 0, marker: 2 }",
            "Packet { result: result.result + index, marker: result.marker }",
            "result.result + result.marker - 2",
        ),
    ] {
        let source = source(
            helpers,
            &format!(
                "if divisor == 0 {{ return 41; }} else {{
                let index = 0; let result = {seed};
                while index < 2 {{ let index = index + 1; let result = {update}; }}
                let unused = checked(8, probe({observation}));
            }}
            let result = checked(8, divisor); return result;"
            ),
        );
        let calls = [
            "start",
            "selected",
            "__nuis_scalar_branch_0",
            "iteration",
            "iteration",
            "probe",
            "checked",
            "checked",
        ];
        execute(
            &source,
            2,
            8,
            &[
                open(&[0], Some(41), &calls[..3]),
                open(&[2], Some(4), &calls),
                open(&[-2], Some(-4), &calls),
            ],
        );
        execute(&source, 2, 7, &[open(&[2], None, &calls[..7])]);
        let mut exhausted = open(&[2], None, &calls[..3]);
        exhausted.remaining_loop = 1;
        execute(&source, 1, 8, &[exhausted]);
        let zero = source
            .replace("index < 2", "index < 0")
            .replace("if flag {", "if flag == false {")
            .replace("if value == 3", "if value == 0");
        let zero_calls = [&calls[..3], &calls[5..]].concat();
        execute(
            &zero,
            0,
            6,
            &[
                open(&[0], Some(41), &calls[..3]),
                open(&[2], Some(4), &zero_calls),
            ],
        );
        execute(&zero, 0, 5, &[open(&[2], None, &zero_calls[..5])]);
    }
}

#[test]
fn statement_suffix_merges_once_into_the_inner_shared_body() {
    let source = source(
        "@noinline fn right(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn after(value: i64, divisor: i64) -> i64 { return value / divisor; }",
        "if divisor == 0 { return 41; } else {
            if divisor > 1 { let unused = right(4, divisor); }
            let prefix = after(16, divisor);
        }
        let suffix = checked(8, divisor); return suffix;",
    );
    let early = ["start", "selected", "__nuis_scalar_branch_2"];
    let once = [
        "start",
        "selected",
        "__nuis_scalar_branch_2",
        "__nuis_scalar_branch_0",
        "__nuis_scalar_branch_1",
        "__nuis_scalar_continue_0",
        "after",
        "checked",
    ];
    let with_prefix = [
        "start",
        "selected",
        "__nuis_scalar_branch_2",
        "__nuis_scalar_branch_0",
        "right",
        "__nuis_scalar_continue_0",
        "after",
        "checked",
        "__nuis_scalar_branch_1",
    ];
    execute(
        &source,
        0,
        9,
        &[
            open(&[0], Some(41), &early),
            open(&[1], Some(8), &once),
            open(&[2], Some(4), &with_prefix),
            open(&[-2], Some(-4), &once),
        ],
    );
    execute(&source, 0, 7, &[open(&[2], None, &with_prefix[..7])]);
}

#[test]
fn shared_terminal_work_keeps_one_body_and_guarded_call_sequence() {
    let source = source(
        "",
        "if divisor > 0 { if divisor == 1 { return 41; } }
        return checked(8, divisor);",
    );
    let early = [
        "start",
        "selected",
        "__nuis_scalar_branch_1",
        "__nuis_scalar_branch_0",
        "__nuis_scalar_branch_2",
    ];
    let positive = [
        "start",
        "selected",
        "__nuis_scalar_branch_1",
        "__nuis_scalar_branch_0",
        "__nuis_scalar_continue_0",
        "checked",
        "__nuis_scalar_branch_2",
    ];
    let negative = [
        "start",
        "selected",
        "__nuis_scalar_branch_1",
        "__nuis_scalar_branch_2",
        "__nuis_scalar_continue_0",
        "checked",
    ];
    execute(
        &source,
        0,
        7,
        &[
            open(&[1], Some(41), &early),
            open(&[2], Some(4), &positive),
            open(&[-2], Some(-4), &negative),
            open(&[0], None, &negative),
        ],
    );
    // A neutral guard still costs an entry after the selected computation.
    execute(&source, 0, 6, &[open(&[2], None, &positive[..6])]);
}

#[test]
fn terminal_forwarding_preserves_nested_prefix_and_shared_suffix_order() {
    let source = source(
        "@noinline fn right(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn after(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn left(value: i64, divisor: i64) -> i64 { return value / divisor; }",
        "if divisor == 0 { return 41; } else {
            if divisor > 1 { let unused = right(4, divisor); }
            let suffix = after(16, divisor);
        }
        return left(8, divisor);",
    );
    let early = ["start", "selected", "__nuis_scalar_branch_2"];
    let once = [
        "start",
        "selected",
        "__nuis_scalar_branch_2",
        "__nuis_scalar_branch_0",
        "__nuis_scalar_branch_1",
        "__nuis_scalar_continue_0",
        "after",
        "left",
    ];
    let with_prefix = [
        "start",
        "selected",
        "__nuis_scalar_branch_2",
        "__nuis_scalar_branch_0",
        "right",
        "__nuis_scalar_continue_0",
        "after",
        "left",
        "__nuis_scalar_branch_1",
    ];
    execute(
        &source,
        0,
        9,
        &[
            open(&[0], Some(41), &early),
            open(&[1], Some(8), &once),
            open(&[2], Some(4), &with_prefix),
            open(&[-2], Some(-4), &once),
        ],
    );
    execute(&source, 0, 7, &[open(&[2], None, &with_prefix[..7])]);
}
