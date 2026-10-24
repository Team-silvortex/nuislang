use super::*;

#[test]
fn ready_rhs_never_erases_effectful_left_even_when_result_is_constant() {
    for (operator, rhs, expected) in [("&&", false, 2), ("||", true, 1)] {
        let source = format!("mod cpu Main {{
            @noinline fn predicate() -> bool {{ print(7); return true; }}
            fn main() -> i64 {{ if predicate() {operator} {rhs} {{ return 1; }} else {{ return 2; }} }}
        }}");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let main = module
            .functions
            .iter_mut()
            .find(|f| f.name == "main")
            .unwrap();
        let NirStmt::If { condition, .. } = &mut main.body[0] else {
            panic!()
        };
        let mut helpers = Vec::new();
        *condition = logical_conditions::predicate(
            condition.clone(),
            &Scope::new(),
            &mut BTreeSet::new(),
            &mut helpers,
            &mut BTreeSet::new(),
        );
        assert!(helpers.is_empty());
        let manifest =
            crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
                .unwrap();
        let yir = crate::lowering::lower_nir_to_yir(&module, &manifest, None).unwrap();
        let result = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .node
            .clone();
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .unwrap();
        assert_eq!(trace.values[&result], yir_core::Value::Int(expected));
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|e| e.contains("cpu.print") && e.ends_with(" 7"))
                .count(),
            1
        );
    }
}

#[test]
fn ready_logical_rhs_keeps_effectful_left_and_guards_nested_calls() {
    let scope = Scope::from([("flag".into(), scalar_type("bool"))]);
    for op in [NirBinaryOp::And, NirBinaryOp::Or] {
        let call = NirExpr::Call {
            callee: "predicate".into(),
            args: vec![],
        };
        let expr = NirExpr::Binary {
            op,
            lhs: Box::new(call.clone()),
            rhs: Box::new(NirExpr::Var("flag".into())),
        };
        let mut names = BTreeSet::new();
        let mut helpers = Vec::new();
        let mut guarded = BTreeSet::new();
        let lowered = logical_conditions::predicate(
            expr.clone(),
            &scope,
            &mut names,
            &mut helpers,
            &mut guarded,
        );
        assert_eq!(lowered, expr);
        assert!(helpers.is_empty());
        let nested = NirExpr::Binary {
            op,
            lhs: Box::new(NirExpr::Binary {
                op,
                lhs: Box::new(NirExpr::Var("flag".into())),
                rhs: Box::new(call),
            }),
            rhs: Box::new(NirExpr::Bool(true)),
        };
        logical_conditions::predicate(nested, &scope, &mut names, &mut helpers, &mut guarded);
        assert_eq!(helpers.len(), 1);
        assert_eq!(guarded.len(), 1);
        assert!(matches!(helpers[0].body[0], NirStmt::If { .. }));
    }
}

#[test]
fn ready_update_selects_execute_in_order_on_both_paths_and_reversed_yir() {
    for gate in [false, true] {
        let source = format!("mod cpu Main {{ fn main() -> i64 {{ let gate = {gate}; let a = 2; let b = 7; return a * 10 + b; }} }}");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        let scope = Scope::from([
            ("a".into(), scalar_type("i64")),
            ("b".into(), scalar_type("i64")),
        ]);
        let set = |name: &str, value| NirStmt::Let {
            name: name.into(),
            ty: None,
            value,
        };
        let then_body = [
            set("a", NirExpr::Int(1)),
            set("b", NirExpr::Var("a".into())),
        ];
        let else_body = [
            set("a", NirExpr::Var("b".into())),
            set("b", NirExpr::Int(3)),
        ];
        let body = &mut module.functions[0].body;
        let returned = body.pop().unwrap();
        body.extend(updates(&then_body, &scope, "gate", true).unwrap());
        body.extend(updates(&else_body, &scope, "gate", false).unwrap());
        body.push(returned);
        let manifest =
            crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
                .unwrap();
        let yir = crate::lowering::lower_nir_to_yir(&module, &manifest, None).unwrap();
        for reversed in [false, true] {
            let mut yir = yir.clone();
            if reversed {
                yir.nodes.reverse();
                for function in &mut yir.functions {
                    function.body_nodes.reverse();
                }
            }
            let result = yir
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .node
                .clone();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            )
            .unwrap();
            assert_eq!(
                trace.values[&result],
                yir_core::Value::Int(if gate { 11 } else { 73 })
            );
        }
    }
}

#[test]
fn ready_predicates_do_not_grant_speculation_to_calls_checks_or_projections() {
    let scope = Scope::from([
        ("flag".into(), scalar_type("bool")),
        ("n".into(), scalar_type("i64")),
        ("record".into(), scalar_type("Pair")),
    ]);
    let equal = |lhs, rhs| NirExpr::Binary {
        op: NirBinaryOp::Eq,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    };
    let comparison = equal(NirExpr::Var("n".into()), NirExpr::Int(2));
    assert!(total_predicate(&comparison, &scope));
    assert!(total_predicate(
        &NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Var("flag".into())),
            rhs: Box::new(comparison),
        },
        &scope
    ));
    for value in [
        NirExpr::Call {
            callee: "pure_but_fallible".into(),
            args: vec![],
        },
        NirExpr::Binary {
            op: NirBinaryOp::Div,
            lhs: Box::new(NirExpr::Int(1)),
            rhs: Box::new(NirExpr::Var("n".into())),
        },
        NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("record".into())),
            field: "n".into(),
        },
        NirExpr::Var("unknown".into()),
        NirExpr::Var("record".into()),
    ] {
        assert!(!total_predicate(&equal(value, NirExpr::Int(0)), &scope));
    }
    assert!(!total_predicate(&NirExpr::Var("n".into()), &scope));
}

#[test]
fn ready_updates_preserve_sequential_reads_and_reject_partial_or_nominal_work() {
    let scope = Scope::from([
        ("a".into(), scalar_type("i64")),
        ("b".into(), scalar_type("i64")),
        ("record".into(), scalar_type("Pair")),
    ]);
    let assign = |name: &str, value| NirStmt::Let {
        name: name.into(),
        ty: None,
        value,
    };
    let arm = vec![
        assign("a", NirExpr::Int(1)),
        assign("b", NirExpr::Var("a".into())),
    ];
    for selected in [true, false] {
        let result = updates(&arm, &scope, "gate", selected).unwrap();
        assert_eq!(result.len(), 2);
        for (i, stmt) in result.iter().enumerate() {
            let NirStmt::If {
                condition,
                then_body,
                else_body,
            } = stmt
            else {
                panic!()
            };
            assert_eq!(*condition, NirExpr::Var("gate".into()));
            let (changed, kept) = if selected {
                (then_body, else_body)
            } else {
                (else_body, then_body)
            };
            let NirStmt::Let { name, value, .. } = &changed[0] else {
                panic!()
            };
            assert_eq!(name, if i == 0 { "a" } else { "b" });
            assert_eq!(
                *value,
                if i == 0 {
                    NirExpr::Int(1)
                } else {
                    NirExpr::Var("a".into())
                }
            );
            assert!(matches!(&kept[0], NirStmt::Let { value: NirExpr::Var(v), .. } if v == name));
        }
    }
    for bad in [
        assign("new_local", NirExpr::Int(0)),
        assign("record", NirExpr::Var("record".into())),
        assign("a", NirExpr::Bool(false)),
        assign(
            "a",
            NirExpr::Call {
                callee: "effect".into(),
                args: vec![],
            },
        ),
        NirStmt::Print(NirExpr::Int(9)),
        NirStmt::Return(Some(NirExpr::Int(0))),
    ] {
        let mut mixed = arm.clone();
        mixed.push(bad);
        assert!(updates(&mixed, &scope, "gate", true).is_none());
    }
}
