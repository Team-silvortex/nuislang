use super::*;

#[path = "capture_loop_join_tests.rs"]
mod loop_tests;

#[test]
fn fallthrough_record_joins_project_fields_without_changing_public_inputs() {
    for local in [false, true] {
        let (prefix, name) = if local {
            ("let current = state;", "current")
        } else {
            ("", "state")
        };
        let mut module = module(&format!(
            "fn helper(state: State, flag: bool) -> i64 {{
                {prefix} let old = {name};
                if flag {{
                    let {name} = State {{ a: Pair {{ x: {name}.a.x + 1, y: 0 }},
                        b: Pair {{ x: 0, y: 7 }}, unused: 0 }};
                }}
                return old.a.x + {name}.b.y;
            }}
            fn entry(state: State, flag: bool) -> i64 {{ return helper(state, flag); }}"
        ));
        assert!(run(&mut module, &["helper"]));
        assert_eq!(
            function(&module, "helper")
                .params
                .iter()
                .map(|p| p.ty.name.as_str())
                .collect::<Vec<_>>(),
            ["i64", "i64", "bool"]
        );
        assert_eq!(function(&module, "entry").params[0].ty.name, "State");
        assert!(!run(&mut module, &["helper"]));
    }
}

#[test]
fn fallthrough_record_joins_keep_nested_reaching_values_and_old_copies() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct State { left: i64, right: i64, unused: i64 }
        @noinline fn relay(value: i64) -> i64 { return value; }
        @noinline fn helper(state: State, flag: bool, other: bool) -> i64 {
            const old: State = state;
            if flag {
                let before = state;
                let state = State { left: state.right, right: state.left, unused: 0 };
                if other {
                    let state = State { left: state.right + 1, right: before.right + 2, unused: 0 };
                }
            } else {
                if other { let state = State { left: 70, right: 80, unused: 0 }; }
            }
            let saved = state;
            if other { let state = State { left: state.right, right: state.left, unused: 0 }; }
            return relay(old.left * 10000 + saved.left * 100 + state.right);
        }
        fn main() -> i64 {
            let state = State { left: 12, right: 33, unused: 99 };
            print(helper(state, true, true)); print(helper(state, true, false));
            print(helper(state, false, true)); print(helper(state, false, false)); return 0;
        }
    }",
    )
    .unwrap();
    for projected in [false, true] {
        if projected {
            assert!(run(&mut module, &["helper"]));
            assert_eq!(function(&module, "helper").params.len(), 4);
            assert!(!run(&mut module, &["helper"]));
        }
        assert_eq!(execute(&module).unwrap(), [121313, 123312, 127070, 121233]);
    }
}

#[test]
fn fallthrough_record_joins_retain_loop_reads_without_rewriting_backedges() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct State { left: i64, right: i64, unused: i64 }
        @noinline fn helper(state: State, flag: bool, limit: i64) -> i64 {
            let old = state;
            if flag { let state = State { left: state.right, right: state.left, unused: 0 }; }
            let i = 0; let total = 0;
            while i < limit { let total = total + state.right; let i = i + 1; }
            return old.left + total;
        }
        fn main() -> i64 {
            let state = State { left: 12, right: 33, unused: 99 };
            print(helper(state, true, 2)); print(helper(state, false, 2));
            print(helper(state, false, 0)); return 0;
        }
    }",
    )
    .unwrap();
    for projected in [false, true] {
        if projected {
            assert!(run(&mut module, &["helper"]));
        }
        assert_eq!(execute(&module).unwrap(), [36, 78, 12]);
    }
}

#[test]
fn fallthrough_record_joins_preserve_selected_unused_constructor_work_and_calls() {
    for (divisor, selected) in [(0, false), (2, true), (0, true)] {
        let mut module = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Main {{
            struct State {{ left: i64, right: i64, unused: i64 }}
            @noinline fn quotient(divisor: i64) -> i64 {{ return 14 / divisor; }}
            @noinline fn helper(state: State, divisor: i64, flag: bool) -> i64 {{
                let old = state;
                if flag {{ let state = State {{ left: state.left, right: 3, unused: quotient(divisor) }}; }}
                return old.left + state.right;
            }}
            fn main() -> i64 {{
                let state = State {{ left: 12, right: 33, unused: 99 }};
                print(helper(state, {divisor}, {selected})); return 0;
            }} }}"
        )).unwrap();
        for projected in [false, true] {
            if projected {
                assert!(run(&mut module, &["helper"]));
                let mut calls = 0;
                walk::visit(&function(&module, "helper").body, |expr| {
                    if matches!(expr, NirExpr::Call { callee, .. } if callee == "quotient") {
                        calls += 1;
                    }
                    true
                });
                assert_eq!(calls, 1);
            }
            let result = execute(&module);
            if selected && divisor == 0 {
                assert!(result.is_err());
            } else {
                assert_eq!(result.unwrap(), [if selected { 15 } else { 45 }]);
            }
        }
    }
}

#[test]
fn fallthrough_record_joins_project_copy_families_and_nominal_prefixes() {
    let mut module = module(
        "fn consume(pair: Pair) -> i64 { return pair.x; }
        fn helper(left: State, right: State, flag: bool) -> i64 {
            let current = left; let old = current;
            if flag { let current = right; }
            return consume(current.a) + old.a.y;
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
    assert!(!run(&mut module, &["helper"]));
}

#[test]
fn fallthrough_record_joins_leave_whole_uses_and_computed_callers_unchanged() {
    for (update, result, caller) in [
        (
            "if flag { let state = State { a: state.b, b: state.a, unused: 0 }; }",
            "consume(old)",
            "helper(state, flag)",
        ),
        (
            "if flag { let state = State { a: state.b, b: state.a, unused: 0 }; }",
            "state.a.x",
            "helper(relay(state), flag)",
        ),
    ] {
        let mut module = module(&format!(
            "fn consume(state: State) -> i64 {{ return state.a.x; }}
            fn relay(state: State) -> State {{ return state; }}
            fn helper(state: State, flag: bool) -> i64 {{ let old = state; {update} return {result}; }}
            fn entry(state: State, flag: bool) -> i64 {{ return {caller}; }}"
        ));
        let before = module.clone();
        assert!(!run(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn fallthrough_record_joins_require_exact_types_and_preserve_name_hygiene() {
    for case in 0..3 {
        let mut module = module(
            "fn helper(state: State, flag: bool) -> i64 {
                if flag { let state = State { a: state.b, b: state.a, unused: 0 }; }
                return state.a.x;
            }",
        );
        let layouts = control_values::TypedLayouts::collect(&module);
        let NirStmt::If { then_body, .. } = &mut module.functions[0].body[0] else {
            panic!()
        };
        let NirStmt::Let { ty, .. } = &mut then_body[0] else {
            panic!()
        };
        match case {
            0 => *ty = None,
            1 => ty.as_mut().unwrap().is_ref = true,
            _ => ty.as_mut().unwrap().name = "Pair".into(),
        }
        let before = module.clone();
        joins::normalize(&mut module.functions[0], &layouts);
        assert_eq!(module, before);
    }
    let mut module = module(
        "fn helper(state: State, flag: bool, __nuis_capture_join_input_0: i64) -> i64 {
            if flag { let state = State { a: state.b, b: state.a, unused: 0 }; }
            let __nuis_capture_join_input_1 = 7;
            return state.a.x + __nuis_capture_join_input_0 + __nuis_capture_join_input_1;
        }",
    );
    assert!(run(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    let mut names = BTreeSet::new();
    branches::collect_bindings(&helper.body, &mut names);
    assert!(!names.contains("__nuis_capture_join_input_0"));
    assert!(names.contains("__nuis_capture_join_input_1"));
    assert!(names.contains("__nuis_capture_join_input_2"));
}

#[test]
fn fallthrough_record_joins_preserve_scalar_kinds() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct State { enabled: bool, gain: f32, scale: f64, tag: i32, unused: i64 }
        fn helper(state: State, flag: bool) -> f64 {
            let old = state;
            if flag { let state = State { enabled: false, gain: old.gain, scale: 2.5, tag: old.tag, unused: 0 }; }
            if state.enabled { return old.scale; } return state.scale;
        }
        fn entry(state: State, flag: bool) -> f64 { return helper(state, flag); }
        fn main() -> i64 { return 0; }
    }"
    ).unwrap();
    assert!(run(&mut module, &["helper"]));
    assert_eq!(
        function(&module, "helper")
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["bool", "f32", "f64", "i32", "bool"]
    );
    let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&module).unwrap();
    yir_lower_llvm::emit_module(&yir).unwrap();
}

fn execute(module: &NirModule) -> Result<Vec<i64>, String> {
    let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
    yir_lower_llvm::emit_module(&yir).unwrap();
    yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .map(|trace| {
        trace
            .events
            .iter()
            .filter(|e| e.contains("cpu.print"))
            .map(|e| e.rsplit_once(": ").unwrap().1.parse().unwrap())
            .collect()
    })
    .map_err(|error| format!("{error:?}"))
}
