use super::*;

#[test]
fn loop_record_inputs_preserve_zero_trips_and_per_trip_snapshots() {
    for local in [false, true] {
        let (prefix, name) = if local {
            ("let current = state;", "current")
        } else {
            ("", "state")
        };
        let mut module = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Main {{
            struct State {{ left: i64, right: i64, unused: i64 }}
            @noinline fn helper(state: State, limit: i64) -> i64 {{
                {prefix} let initial = {name}; let i = 0; let total = 0;
                while i < limit {{
                    let {name} = {name}; let before = {name}; let second: State = before;
                    let {name} = State {{ left: second.left + 1, right: before.right + before.left, unused: 0 }};
                    let total = total + second.left * 100 + {name}.left;
                    let i = i + 1;
                }}
                return initial.right + {name}.right + total;
            }}
            fn main() -> i64 {{
                let state = State {{ left: 12, right: 33, unused: 99 }};
                print(helper(state, 0)); print(helper(state, 1)); print(helper(state, 3)); return 0;
            }} }}"
        )).unwrap();
        for projected in [false, true] {
            if projected {
                assert!(run(&mut module, &["helper"]));
                assert_eq!(function(&module, "helper").params.len(), 3);
                assert!(!run(&mut module, &["helper"]));
            }
            // Existing loop admission requires a seeded mutable local carry.
            if local {
                assert_eq!(execute(&module).unwrap(), [66, 1291, 4047]);
            }
        }
    }
}

#[test]
fn loop_record_inputs_preserve_nested_writes_and_control_exits() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct State { left: i64, right: i64, unused: i64 }
        @noinline fn helper(input: State, mode: i64) -> i64 {
            let state = input; let i = 0; let total = 0;
            while i < 4 {
                let i = i + 1; let state = state; let before = state;
                let j = 0;
                while j < 2 {
                    let state = State { left: state.left + 1, right: state.right, unused: 0 };
                    let j = j + 1;
                }
                if mode == 1 { if i == 2 { continue; } }
                let total = total + before.left;
                if mode == 2 { if i == 2 { break; } }
                if mode == 3 { if i == 2 { return before.left + state.left + total; } }
            }
            return total + state.left;
        }
        fn main() -> i64 {
            let state = State { left: 12, right: 33, unused: 99 };
            print(helper(state, 0)); print(helper(state, 1));
            print(helper(state, 2)); print(helper(state, 3)); return 0;
        } }",
    )
    .unwrap();
    for projected in [false, true] {
        if projected {
            assert!(run(&mut module, &["helper"]));
            assert_eq!(function(&module, "helper").params.len(), 3);
        }
        assert_eq!(execute(&module).unwrap(), [80, 66, 42, 56]);
    }
}

#[test]
fn loop_record_inputs_keep_checked_unused_constructor_work_per_trip() {
    for (limit, divisor, expected) in [
        (0, 0, Some(33)),
        (1, 1, Some(13)),
        (2, 3, Some(14)),
        (1, 0, None),
        (2, 1, None),
    ] {
        let mut module = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Main {{
            struct State {{ left: i64, right: i64, unused: i64 }}
            @noinline fn checked(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
            @noinline fn helper(state: State, limit: i64, divisor: i64) -> i64 {{
                let current = state; let i = 0;
                while i < limit {{
                    let i = i + 1;
                    let current = current; let before = current;
                    let current = State {{ left: before.left + 1, right: before.left + 1, unused: checked(before.left, divisor - i + 1) }};
                }}
                return current.right;
            }}
            fn main() -> i64 {{
                let state = State {{ left: 12, right: 33, unused: 99 }};
                print(helper(state, {limit}, {divisor})); return 0;
            }} }}"
        )).unwrap();
        for projected in [false, true] {
            if projected {
                assert!(run(&mut module, &["helper"]));
                assert_eq!(function(&module, "helper").params.len(), 4);
            }
            let result = execute(&module);
            if let Some(expected) = expected {
                assert_eq!(result.unwrap(), [expected]);
            } else {
                assert!(result.is_err());
            }
        }
    }
}

#[test]
fn loop_record_inputs_follow_cyclic_copies_and_nominal_prefixes() {
    let mut module = module(
        "fn consume(pair: Pair) -> i64 { return pair.x; }
        fn helper(left: State, right: State, flag: bool) -> i64 {
            let current = left; let saved = right;
            while flag {
                let before = current; let current = saved; let saved = before;
                break;
            }
            return consume(current.a) + saved.a.y;
        }
        fn entry(left: State, right: State, flag: bool) -> i64 { return helper(left, right, flag); }"
    );
    assert!(run(&mut module, &["helper"]));
    assert_eq!(
        function(&module, "helper")
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["Pair", "Pair", "bool"]
    );
    assert_eq!(function(&module, "entry").params[0].ty.name, "State");
    assert!(!run(&mut module, &["helper"]));
}

#[test]
fn loop_record_inputs_reject_whole_escapes_and_computed_callers_transactionally() {
    for (result, caller) in [
        ("consume(before)", "helper(state, flag)"),
        ("before.a.x", "helper(relay(state), flag)"),
    ] {
        let mut module = module(&format!(
            "fn consume(state: State) -> i64 {{ return state.a.x; }}
            fn relay(state: State) -> State {{ return state; }}
            fn helper(state: State, flag: bool) -> i64 {{
                while flag {{
                    let before = state;
                    let state = State {{ a: state.b, b: state.a, unused: 0 }};
                    return {result};
                }} return state.a.x;
            }}
            fn entry(state: State, flag: bool) -> i64 {{ return {caller}; }}"
        ));
        let before = module.clone();
        assert!(!run(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn loop_record_reconstruction_retains_original_bodies_and_exact_type_vetoes() {
    for case in 0..4 {
        let unused = if case == 0 { "1 / before.unused" } else { "0" };
        let mut module = module(&format!(
            "fn helper(state: State, flag: bool) -> i64 {{
                let current = state;
                while flag {{
                    let before = current;
                    let current = State {{ a: before.b, b: before.a, unused: {unused} }};
                    break;
                }}
                return current.a.x;
            }}"
        ));
        let layouts = control_values::TypedLayouts::collect(&module);
        let helper = &mut module.functions[0];
        let NirStmt::While { body, .. } = &mut helper.body[1] else {
            panic!()
        };
        let NirStmt::Let { ty, .. } = &mut body[1] else {
            panic!()
        };
        match case {
            0 => {}
            1 => *ty = None,
            2 => ty.as_mut().unwrap().is_ref = true,
            _ => ty.as_mut().unwrap().name = "Pair".into(),
        }
        let before = helper.clone();
        joins::normalize(helper, &layouts);
        // All fields are observed (including the unused constructor RHS), or
        // the family is malformed. Neither case permits entry reconstruction.
        assert_eq!(*helper, before);
    }
    let mut module = module(
        "fn helper(state: State, flag: bool) -> i64 {
            let current = state;
            while flag {
                const before: State = current;
                let current = State { a: before.b, b: before.a, unused: 7 / before.a.x };
                break;
            }
            return current.a.x;
        }",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let helper = &mut module.functions[0];
    let before = helper.clone();
    joins::normalize(helper, &layouts);
    assert_eq!(helper.body.len(), before.body.len() + 1);
    assert_eq!(&helper.body[2..], &before.body[1..]);
    assert_eq!(helper.params, before.params);
}
