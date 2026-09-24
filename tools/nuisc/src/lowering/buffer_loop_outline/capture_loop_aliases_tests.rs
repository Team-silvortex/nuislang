use super::*;

#[test]
fn invariant_loop_alias_chains_keep_nested_scopes_and_exact_field_demand() {
    let mut module = module(
        "fn helper(state: State, flag: bool, limit: i64) -> i64 {
            let i = 0; let total = 0;
            while i < limit {
                let saved = state; const second: State = saved;
                if flag { let part = second.a; let total = total + part.x; }
                else { let part = second.b; let total = total + part.y; }
                let j = 0;
                while j < 2 { let inner = state.a; let total = total + inner.x; let j = j + 1; }
                let i = i + 1;
            }
            return total;
        }
        fn entry(state: State, flag: bool, limit: i64) -> i64 { return helper(state, flag, limit); }",
    );
    assert!(project(&mut module, &["helper"]));
    assert_eq!(
        function(&module, "helper")
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["i64", "i64", "bool", "i64"]
    );
    let mut reads = BTreeSet::new();
    walk::visit(&function(&module, "helper").body, |expr| {
        if let NirExpr::Var(name) = expr {
            reads.insert(name.clone());
        }
        true
    });
    assert!(!reads.contains("state"));
    assert_eq!(function(&module, "entry").params[0].ty.name, "State");
    assert!(!project(&mut module, &["helper"]));
}

#[test]
fn loop_aliases_keep_mutating_origins_and_rebound_local_snapshots_whole() {
    for body in [
        "let saved = state; let state = State { a: state.b, b: state.a, unused: 0 }; let total = total + saved.a.x;",
        "let saved = state; if flag { let state = State { a: state.b, b: state.a, unused: 0 }; } let total = total + saved.a.x;",
        "let saved = state; let j = 0; while j < 2 { let state = State { a: state.b, b: state.a, unused: 0 }; let j = j + 1; } let total = total + saved.a.x;",
        "let saved = state; let old = saved; let saved = State { a: saved.b, b: saved.a, unused: 0 }; let total = total + old.a.x + saved.b.y;",
    ] {
        let mut module = module(&format!(
            "fn helper(state: State, flag: bool) -> i64 {{
                let i = 0; let total = 0; while i < 2 {{ {body} let i = i + 1; }} return total;
            }} fn entry(state: State, flag: bool) -> i64 {{ return helper(state, flag); }}"
        ));
        let before = module.clone();
        assert!(!project(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn loop_alias_projection_preserves_whole_uses_and_computed_caller_transactions() {
    for (value, call) in [
        ("consume(saved)", "helper(state)"),
        ("saved.a.x", "helper(relay(state))"),
    ] {
        let mut module = module(&format!(
            "fn consume(state: State) -> i64 {{ return state.a.x; }}
            fn relay(state: State) -> State {{ return state; }}
            fn helper(state: State) -> i64 {{
                let i = 0; let total = 0;
                while i < 2 {{ let saved = state; let total = total + {value}; let i = i + 1; }}
                return total;
            }} fn entry(state: State) -> i64 {{ return {call}; }}"
        ));
        let before = module.clone();
        assert!(!project(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn loop_alias_origins_require_unwritten_parameters_not_computed_locals() {
    let module = module("fn helper(state: State) -> i64 { return state.a.x; }");
    let layouts = control_values::TypedLayouts::collect(&module);
    for (root, written) in [("state", false), ("state", true), ("computed", false)] {
        let parameters = BTreeSet::from(["state".to_owned()]);
        let mut writes = BTreeMap::from([("saved".to_owned(), 1)]);
        if written {
            writes.insert(root.to_owned(), 1);
        }
        let context = Context {
            parameters: &parameters,
            writes: &writes,
            layouts: &layouts,
        };
        let mut scope = BTreeMap::from([(
            "seed".to_owned(),
            Origin {
                path: vec![root.to_owned()],
                ty: scalar_type("State"),
            },
        )]);
        scope.insert(root.to_owned(), scope["seed"].clone());
        let keep = context.keep_binding(
            "saved",
            Some(&scalar_type("State")),
            &mut NirExpr::Var("seed".into()),
            &mut scope,
        );
        assert_eq!(keep, root != "state" || written);
        assert_eq!(scope.contains_key("saved"), !keep);
    }
}

#[test]
fn loop_aliases_do_not_hide_reference_or_annotation_mismatches() {
    for reference in [false, true] {
        let mut module = module(
            "fn helper(state: State) -> i64 {
                let i = 0; let total = 0;
                while i < 2 { let saved: State = state; let total = total + saved.a.x; let i = i + 1; }
                return total;
            }",
        );
        let layouts = control_values::TypedLayouts::collect(&module);
        let helper = &mut module.functions[0];
        if reference {
            helper.params[0].ty.is_ref = true;
        } else {
            let NirStmt::While { body, .. } = &mut helper.body[2] else {
                panic!()
            };
            let NirStmt::Let { ty, .. } = &mut body[0] else {
                panic!()
            };
            ty.as_mut().unwrap().name = "Pair".into();
        }
        let before = helper.clone();
        normalize(helper, &layouts);
        assert_eq!(*helper, before);
    }
}

#[test]
fn projected_loop_aliases_preserve_zero_trips_checked_work_and_reference_results() {
    for (left, limit, divisor, expected) in [
        (12, 0, 0, Some(0)),
        (12, 2, 3, Some(8)),
        (12, 3, -3, Some(-12)),
        (12, 2, 0, None),
        (i64::MIN, 1, -1, None),
    ] {
        let left_source = if left == i64::MIN {
            "(-9223372036854775807 - 1)".to_owned()
        } else {
            left.to_string()
        };
        let mut module = crate::frontend::parse_nuis_module(&format!(
            "mod cpu Main {{
            struct State {{ left: i64, right: i64, unused: i64 }}
            @noinline fn relay(value: i64) -> i64 {{ return value; }}
            @noinline fn helper(state: State, limit: i64, divisor: i64) -> i64 {{
                let i = 0; let total = 0;
                while i < limit {{
                    let i = i + 1;
                    let saved = state; let second = saved;
                    let total = total + relay(second.left) / divisor;
                }}
                return total;
            }}
            fn main() -> i64 {{
                let state = State {{ left: {left_source}, right: 33, unused: 99 }};
                print(helper(state, {limit}, {divisor})); return 0;
            }} }}"
        ))
        .unwrap();
        for projected in [false, true] {
            if projected {
                assert!(project(&mut module, &["helper"]));
                assert_eq!(function(&module, "helper").params.len(), 3);
                assert!(function(&module, "helper")
                    .params
                    .iter()
                    .all(|p| p.ty.name == "i64"));
            }
            let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&module).unwrap();
            yir_lower_llvm::emit_module(&yir).unwrap();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            );
            if let Some(expected) = expected {
                let trace = trace.unwrap();
                let prints = trace
                    .events
                    .iter()
                    .filter(|e| e.contains("cpu.print"))
                    .collect::<Vec<_>>();
                assert_eq!(prints.len(), 1);
                assert!(prints[0].ends_with(&format!(": {expected}")), "{prints:?}");
            } else {
                assert!(trace.is_err(), "selected arithmetic must still fail");
            }
        }
    }
}

#[test]
fn loop_aliases_keep_preceding_record_versions_after_top_level_rebinding() {
    let mut module = module(
        "fn helper(state: State) -> i64 {
            let old = state;
            let state = State { a: Pair { x: 30, y: 0 }, b: Pair { x: 0, y: 7 }, unused: 0 };
            let i = 0; let total = 0;
            while i < 2 { let saved = old; let total = total + saved.a.x + state.b.y; let i = i + 1; }
            return total;
        } fn entry(state: State) -> i64 { return helper(state); }",
    );
    assert!(project(&mut module, &["helper"]));
    assert_eq!(function(&module, "helper").params[0].ty.name, "i64");
    assert!(
        matches!(&function(&module, "helper").body[0], NirStmt::Let { name, .. }
        if name.starts_with("__nuis_capture_snapshot_"))
    );
    assert!(!project(&mut module, &["helper"]));
}
