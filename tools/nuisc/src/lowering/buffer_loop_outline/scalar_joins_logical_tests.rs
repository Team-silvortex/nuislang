use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/logical_scalar_join.ns"
    ))
    .unwrap()
}

fn packet(module: &NirModule) -> &NirFunction {
    module
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap()
}

fn scope(function: &NirFunction) -> Scope {
    function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect()
}

#[test]
fn logical_scalar_join_requires_completed_dependencies_before_any_mutation() {
    let original = module();
    let layouts = control_values::TypedLayouts::collect(&original);
    let pure = scalar_helpers::collect_capture_values(&original, &layouts);
    let f = packet(&original);
    let scope = scope(f);
    assert!(declaration(&f.body[1], &scope, &f.body[2..]).is_none());
    assert!(guarded_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_none());
    assert!(computed_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_none());
    assert!(logical_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_some());
    for dependency in ["decide", "right", "fallback", "quotient"] {
        for effectful in [false, true] {
            let mut module = original.clone();
            let mut denied = pure.clone();
            if effectful {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == dependency)
                    .unwrap()
                    .body
                    .insert(0, NirStmt::Print(NirExpr::Int(88)));
                denied = scalar_helpers::collect_capture_values(&module, &layouts);
            } else {
                denied.remove(dependency);
            }
            assert!(!denied.contains_key(dependency));
            let before = module.clone();
            let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
            let old_names = names.clone();
            assert!(
                conditional_values::outline_declarations(
                    &mut module,
                    &denied,
                    &layouts,
                    &mut names
                )
                .is_empty(),
                "{dependency} {effectful}"
            );
            assert_eq!(module, before);
            assert_eq!(names, old_names);
        }
    }
}

#[test]
fn logical_scalar_join_uses_existing_lazy_edges_once_and_is_order_independent() {
    for reverse in [false, true] {
        let mut module = module();
        if reverse {
            module.functions.reverse();
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let pure = scalar_helpers::collect_capture_values(&module, &layouts);
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated =
            conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names);
        assert_eq!(generated.len(), 3);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let NirStmt::Let {
            value:
                NirExpr::Call {
                    callee: value_join,
                    args,
                },
            ..
        } = &packet(&module).body[1]
        else {
            panic!()
        };
        assert!(generated.contains(value_join));
        assert!(args[1..].iter().all(|arg| matches!(arg, NirExpr::Var(_))));
        let NirExpr::Call {
            callee: outer,
            args,
        } = &args[0]
        else {
            panic!()
        };
        let NirExpr::Call {
            callee: inner,
            args,
        } = &args[0]
        else {
            panic!()
        };
        assert!(matches!(&args[0], NirExpr::Call { callee, .. } if callee == "decide"));
        for (name, selected, skipped, disjunction) in [
            (inner, "right", false, false),
            (outer, "fallback", true, true),
        ] {
            let helper = module.functions.iter().find(|f| &f.name == name).unwrap();
            let [NirStmt::If {
                condition: NirExpr::Var(_),
                then_body,
                else_body,
            }] = helper.body.as_slice()
            else {
                panic!()
            };
            let (called, constant) = if disjunction {
                (else_body, then_body)
            } else {
                (then_body, else_body)
            };
            assert!(
                matches!(called.as_slice(), [NirStmt::Return(Some(NirExpr::Call { callee, .. }))] if callee == selected)
            );
            assert_eq!(constant, &[NirStmt::Return(Some(NirExpr::Bool(skipped)))]);
        }
        let carries = control_values::layouts(&module);
        assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
        let before = module.clone();
        assert!(
            conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names)
                .is_empty()
        );
        assert_eq!(module, before);
    }
}

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

#[test]
fn logical_scalar_join_total_predicates_need_no_lazy_helper() {
    let mut module = module();
    let f = module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap();
    let NirStmt::If { condition, .. } = &mut f.body[1] else {
        panic!()
    };
    *condition = binary(
        NirBinaryOp::Or,
        binary(
            NirBinaryOp::And,
            NirExpr::Var("choice".into()),
            NirExpr::Bool(true),
        ),
        NirExpr::Bool(false),
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    assert_eq!(
        conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names).len(),
        1
    );
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn logical_scalar_join_preserves_edge_depth_work_and_scope_boundaries_atomically() {
    for mutation in 0..9 {
        let mut module = module();
        let f = module
            .functions
            .iter_mut()
            .find(|f| f.name == "packet")
            .unwrap();
        let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = &mut f.body[1]
        else {
            panic!()
        };
        match mutation {
            0 => {
                for _ in 0..31 {
                    *condition = binary(NirBinaryOp::And, condition.clone(), NirExpr::Bool(true));
                }
            }
            1 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..10 {
                    tree = binary(NirBinaryOp::Add, tree.clone(), tree);
                }
                for body in [then_body, else_body] {
                    let NirStmt::Let { value, .. } = &mut body[0] else {
                        panic!()
                    };
                    *value = tree.clone();
                }
            }
            2 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..64 {
                    tree = binary(NirBinaryOp::Add, tree, NirExpr::Int(0));
                }
                *condition = binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    binary(NirBinaryOp::Eq, tree, NirExpr::Int(1)),
                );
            }
            3 => {
                *condition = binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    NirExpr::Call {
                        callee: "decide".into(),
                        args: vec![
                            binary(NirBinaryOp::And, NirExpr::Bool(true), NirExpr::Bool(false)),
                            NirExpr::Int(1),
                        ],
                    },
                )
            }
            4 => {
                for body in [then_body, else_body] {
                    let NirStmt::Let { value, ty, .. } = &mut body[0] else {
                        panic!()
                    };
                    *ty = Some(scalar_type("bool"));
                    // Direct bool roots now have a separate value-arm proof;
                    // hidden logical comparison operands still fail closed.
                    *value = binary(
                        NirBinaryOp::Eq,
                        binary(NirBinaryOp::And, NirExpr::Bool(true), NirExpr::Bool(false)),
                        NirExpr::Bool(true),
                    );
                }
            }
            5 => then_body.insert(0, NirStmt::Print(NirExpr::Int(88))),
            6 => else_body.clear(),
            7 => f.params.push(NirParam {
                name: "selected".into(),
                ty: scalar_type("i64"),
            }),
            8 => *condition = binary(NirBinaryOp::And, NirExpr::Int(1), NirExpr::Bool(true)),
            _ => unreachable!(),
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let pure = scalar_helpers::collect_capture_values(&module, &layouts);
        let before = module.clone();
        let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
        let old_names = names.clone();
        assert!(
            conditional_values::outline_declarations(&mut module, &pure, &layouts, &mut names)
                .is_empty(),
            "mutation {mutation}"
        );
        assert_eq!(module, before);
        assert_eq!(names, old_names);
    }
    let mut module = module();
    let f = module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap();
    let NirStmt::If { condition, .. } = &mut f.body[1] else {
        panic!()
    };
    for _ in 0..30 {
        *condition = binary(NirBinaryOp::And, condition.clone(), NirExpr::Bool(true));
    }
    let layouts = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &layouts);
    let f = packet(&module);
    assert!(logical_declaration(&f.body[1], &scope(f), &f.body[2..], &pure, &layouts).is_some());
}
