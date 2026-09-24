use super::*;

#[test]
fn terminal_branch_writes_project_old_parameter_and_local_snapshots() {
    for local in [false, true] {
        let (prefix, name) = if local {
            ("let current = state;", "current")
        } else {
            ("", "state")
        };
        let mut module = module(&format!("fn helper(state: State, flag: bool) -> i64 {{
            {prefix} let old = {name};
            if flag {{
                let {name} = State {{ a: Pair {{ x: 30, y: 0 }}, b: Pair {{ x: 0, y: 7 }}, unused: 0 }};
                return old.a.x + {name}.b.y;
            }}
            return old.b.y;
        }}
        fn entry(state: State, flag: bool) -> i64 {{ return helper(state, flag); }}"));
        assert!(project(&mut module, &["helper"]));
        assert_eq!(
            function(&module, "helper")
                .params
                .iter()
                .map(|p| p.ty.name.as_str())
                .collect::<Vec<_>>(),
            ["i64", "i64", "bool"]
        );
        assert_eq!(function(&module, "entry").params[0].ty.name, "State");
        assert!(!project(&mut module, &["helper"]));
    }
}

#[test]
fn terminal_snapshot_versions_keep_nested_returns_and_fallthrough_values_distinct() {
    let mut module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
        struct State { left: i64, right: i64, unused: i64 }
        @noinline fn helper(state: State, flag: bool, other: bool) -> i64 {
            let old = state;
            let state = State { left: state.right, right: state.left, unused: 0 };
            if flag {
                let branch_old = state;
                let state = State { left: state.left + 1, right: state.right + 2, unused: 0 };
                if other {
                    let state = State { left: state.left + 1, right: 0, unused: 0 };
                    return old.left + branch_old.right + state.left;
                } else {
                    let state = State { left: state.right, right: state.left, unused: 0 };
                    return old.right + branch_old.left + state.right;
                }
            }
            return old.left + state.right;
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
            assert!(project(&mut module, &["helper"]));
            assert_eq!(function(&module, "helper").params.len(), 4);
            assert!(!project(&mut module, &["helper"]));
        }
        let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&module).unwrap();
        yir_lower_llvm::emit_module(&yir).unwrap();
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .unwrap();
        let prints = trace
            .events
            .iter()
            .filter(|event| event.contains("cpu.print"))
            .collect::<Vec<_>>();
        assert_eq!(prints.len(), 4);
        for (line, expected) in prints.iter().zip([59, 100, 24, 24]) {
            assert!(line.ends_with(&format!(": {expected}")), "{prints:?}");
        }
    }
}

#[test]
fn terminal_snapshot_proof_rejects_live_joins_and_loop_backedges() {
    let rebind =
        "let state = State { a: Pair { x: 30, y: 0 }, b: Pair { x: 0, y: 7 }, unused: 0 };";
    for body in [
        format!("if flag {{ {rebind} }} return state.a.x;"),
        format!("if flag {{ {rebind} return state.a.x; }} else {{ {rebind} }} return state.b.y;"),
        format!(
            "if flag {{ {rebind} if other {{ {rebind} }} return state.a.x; }} return state.b.y;"
        ),
        format!("while flag {{ {rebind} return state.a.x; }} return state.b.y;"),
        format!(
            "while flag {{ if other {{ {rebind} return state.a.x; }} break; }} return state.b.y;"
        ),
    ] {
        let mut module = module(&format!(
            "fn helper(state: State, flag: bool, other: bool) -> i64 {{ {body} }}"
        ));
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(&mut module.functions[0], &layouts);
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn terminal_snapshots_preserve_whole_uses_and_computed_caller_transactions() {
    for whole in [false, true] {
        let (returned, selected, caller) = if whole {
            ("State", "old", "helper(state, flag)")
        } else {
            ("Pair", "old.a", "helper(relay(state), flag)")
        };
        let mut module = module(&format!("fn relay(state: State) -> State {{ return state; }}
            fn helper(state: State, flag: bool) -> {returned} {{
                let old = state;
                if flag {{
                    let state = State {{ a: Pair {{ x: 30, y: 0 }}, b: Pair {{ x: 0, y: 7 }}, unused: 0 }};
                    return {selected};
                }}
                return {selected};
            }}
            fn entry(state: State, flag: bool) -> {returned} {{ return {caller}; }}"));
        let before = module.clone();
        assert!(!project(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn terminal_snapshots_keep_checked_constructor_work_on_selected_paths() {
    for (divisor, selected) in [(0, false), (2, true), (0, true)] {
        let mut module = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Main {{
            struct State {{ left: i64, right: i64, unused: i64 }}
            @noinline fn helper(state: State, divisor: i64, flag: bool) -> i64 {{
                let old = state;
                if flag {{
                    let state = State {{ left: 14 / divisor, right: 3, unused: 0 }};
                    return old.left;
                }}
                return old.right;
            }}
            fn main() -> i64 {{
                let state = State {{ left: 12, right: 33, unused: 99 }};
                print(helper(state, {divisor}, {selected})); return 0;
            }}
        }}"
        ))
        .unwrap();
        for projected in [false, true] {
            if projected {
                assert!(project(&mut module, &["helper"]));
            }
            let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&module).unwrap();
            yir_lower_llvm::emit_module(&yir).unwrap();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            );
            if selected && divisor == 0 {
                assert!(trace.is_err(), "discarded constructor must still trap");
            } else {
                let trace = trace.unwrap();
                let prints = trace
                    .events
                    .iter()
                    .filter(|event| event.contains("cpu.print"))
                    .collect::<Vec<_>>();
                assert_eq!(prints.len(), 1);
                assert!(prints[0].ends_with(if selected { ": 12" } else { ": 33" }));
            }
        }
    }
}

#[test]
fn terminal_snapshot_scopes_require_returns_not_break_continue_or_zero_trip_bodies() {
    for (body, returns) in [
        (vec![NirStmt::Return(None)], true),
        (
            vec![NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body: vec![NirStmt::Return(None)],
                else_body: vec![],
            }],
            false,
        ),
        (
            vec![NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body: vec![NirStmt::Return(None)],
                else_body: vec![NirStmt::Return(None)],
            }],
            true,
        ),
        (
            vec![NirStmt::While {
                condition: NirExpr::Bool(true),
                body: vec![NirStmt::Return(None)],
            }],
            false,
        ),
        (vec![NirStmt::Break, NirStmt::Return(None)], false),
        (
            vec![
                NirStmt::If {
                    condition: NirExpr::Bool(true),
                    then_body: vec![NirStmt::Continue],
                    else_body: vec![],
                },
                NirStmt::Return(None),
            ],
            false,
        ),
    ] {
        assert_eq!(scopes::collect(&body)[0].returns, returns, "{body:?}");
    }
    let body = vec![NirStmt::While {
        condition: NirExpr::Bool(true),
        body: vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Return(None)],
            else_body: vec![],
        }],
    }];
    let scopes = scopes::collect(&body);
    assert!(scopes.iter().skip(1).all(|scope| scope.in_loop));
}

#[test]
fn terminal_snapshot_candidates_still_require_exact_declared_value_types() {
    for case in 0..3 {
        let mut module = module(
            "fn helper(state: State, flag: bool) -> i64 {
            if flag {
                let state = State { a: Pair { x: 30, y: 0 }, b: Pair { x: 0, y: 7 }, unused: 0 };
                return state.a.x;
            }
            return state.b.y;
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
        normalize(&mut module.functions[0], &layouts);
        assert_eq!(module, before);
    }
}
