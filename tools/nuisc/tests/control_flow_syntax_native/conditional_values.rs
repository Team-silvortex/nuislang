use super::compile_and_run;

#[test]
fn mixed_nested_guarded_values_run_with_the_ordinary_owned_abi() {
    let source = "mod cpu Main {
        struct Leaf { flag: bool, value: i64 }
        struct Packet { payload: Leaf }
        fn relay(value: Packet) -> Packet { return value; }
        fn checked(value: Packet, divisor: i64) -> Packet {
            return Packet { payload: Leaf { flag: value.payload.flag, value: value.payload.value / divisor } };
        }
        @noinline fn event(skip: bool, divisor: i64) -> i64 {
            let saved = Packet { payload: Leaf { flag: skip, value: 12 } };
            let before = saved;
            let saved: Packet = if skip { relay(saved) } else { checked(saved, divisor) };
            print(saved.payload.value);
            return before.payload.value + saved.payload.value;
        }
        fn main() -> i64 { return event(true, 0) + event(false, 2); }
    }";
    assert_eq!(
        compile_and_run("mixed_nested_guarded_values", source).code(),
        Some(42)
    );
}

#[test]
fn guarded_local_values_run_in_effectful_native_functions_and_buffer_loops() {
    let source = r#"mod cpu Main {
        struct State { value: i64, divisor: i64 }
        @noinline fn event(state: State, kind: i64) -> i64 {
            if kind < 0 { return state.value; }
            let selected: i64 = if kind == 0 { 7 } else { state.value % state.divisor };
            print(selected); return selected;
        }
        @noinline fn fill(buffer: ref Buffer, divisor: i64) -> i64 {
            let index: i64 = 0;
            while index < 3 {
                let value: i64 = if divisor == 0 { index } else { index / divisor };
                buffer[index] = value;
                let index: i64 = index + 1;
            }
            return buffer[0] + buffer[1] + buffer[2];
        }
        fn main() -> i64 {
            let ignored = event(State { value: 9, divisor: 0 }, -1);
            let skipped = event(State { value: 9, divisor: 0 }, 0);
            let reached = event(State { value: 9, divisor: 2 }, 1);
            let buffer: ref Buffer = alloc_buffer(3, 0);
            let first = fill(buffer, 0);
            let second = fill(buffer, 2);
            free(buffer);
            return ignored + skipped + reached + first + second;
        }
    }"#;
    assert_eq!(
        compile_and_run("guarded_local_values", source).code(),
        Some(21)
    );
}

#[test]
fn native_local_value_failures_are_selected_not_speculated_or_discarded() {
    for op in ["/", "%"] {
        for operands in ["9, 0", "-9223372036854775807 - 1, -1"] {
            for enabled in [false, true] {
                let source = format!(
                    r#"mod cpu Main {{
                    fn checked(a: i64, b: i64) -> i64 {{ return a {op} b; }}
                    fn ignore(value: i64) -> i64 {{ return 3; }}
                    @noinline fn event(enabled: bool, a: i64, b: i64) -> i64 {{
                        let unused: i64 = if enabled {{ ignore(checked(a, b)) }} else {{ 0 }};
                        print(99); return 0;
                    }}
                    fn main() -> i64 {{ return event({enabled}, {operands}); }}
                }}"#
                );
                let status = compile_and_run("native_local_value_failure", &source);
                if !enabled {
                    assert!(status.success(), "{op} {operands}: {status}");
                } else {
                    assert!(!status.success(), "{op} {operands}: {status}");
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn nested_native_local_selections_preserve_captures_and_nominal_values() {
    let source = r#"mod cpu Main {
        struct Pair { left: i64, right: i64 }
        @noinline fn choose(enabled: bool, divisor: i64, __nuis_value_condition_0: i64) -> i64 {
            let value: i64 = 12;
            let value: i64 = if enabled {
                if divisor == 0 { value } else { value / divisor }
            } else { __nuis_value_condition_0 };
            let flag: bool = if enabled { value % 5 == 2 } else { true };
            let pair: Pair = if flag {
                Pair { right: value % 5, left: value }
            } else { Pair { left: 0, right: 0 } };
            print(pair.left); return pair.left + pair.right;
        }
        fn main() -> i64 {
            let first = choose(true, 0, 4);
            let second = choose(false, 0, 4);
            let third = choose(true, 6, 4);
            return first + second + third;
        }
    }"#;
    assert_eq!(
        compile_and_run("nested_native_local_values", source).code(),
        Some(26)
    );
}
