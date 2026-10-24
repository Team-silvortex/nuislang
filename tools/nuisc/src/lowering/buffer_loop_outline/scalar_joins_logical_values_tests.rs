use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/logical_value_join.ns"
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

fn scope(f: &NirFunction) -> Scope {
    f.params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect()
}

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn outlined(module: &mut NirModule) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let pure = scalar_helpers::collect_capture_values(module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    conditional_values::outline_declarations(module, &pure, &layouts, &mut names)
}

#[test]
fn logical_value_join_requires_completed_pure_dependencies_before_mutation() {
    let original = module();
    let layouts = control_values::TypedLayouts::collect(&original);
    let pure = scalar_helpers::collect_capture_values(&original, &layouts);
    let f = packet(&original);
    let scope = scope(f);
    assert!(declaration(&f.body[1], &scope, &f.body[2..]).is_none());
    assert!(guarded_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_none());
    assert!(computed_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_none());
    assert!(logical_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_none());
    assert!(logical_value_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_some());
    for dependency in ["decide", "yes_head", "yes_tail", "no_head", "no_tail"] {
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
fn logical_value_join_helpers_remain_inside_selected_arms_and_are_idempotent() {
    for reverse in [false, true] {
        let mut module = module();
        if reverse {
            module.functions.reverse();
        }
        let generated = outlined(&mut module);
        assert_eq!(generated.len(), 3);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let NirStmt::Let {
            value: NirExpr::Call { callee, args },
            ..
        } = &packet(&module).body[1]
        else {
            panic!()
        };
        assert!(args[1..].iter().all(|arg| matches!(arg, NirExpr::Var(_))));
        assert!(
            matches!(&args[0], NirExpr::Binary { op: NirBinaryOp::And, lhs, .. } if matches!(lhs.as_ref(), NirExpr::Call { callee, .. } if callee == "decide"))
        );
        let selected = module.functions.iter().find(|f| &f.name == callee).unwrap();
        let [NirStmt::If {
            condition: NirExpr::Var(_),
            then_body,
            else_body,
        }] = selected.body.as_slice()
        else {
            panic!()
        };
        for (body, head, tail, disjunction) in [
            (then_body, "yes_head", "yes_tail", false),
            (else_body, "no_head", "no_tail", true),
        ] {
            let [NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. }))] = body.as_slice() else {
                panic!()
            };
            assert!(matches!(rhs.as_ref(), NirExpr::Bool(_)));
            let NirExpr::Call { callee, args } = lhs.as_ref() else {
                panic!()
            };
            assert!(matches!(&args[0], NirExpr::Call { callee, .. } if callee == head));
            let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
            let [NirStmt::If {
                then_body,
                else_body,
                ..
            }] = helper.body.as_slice()
            else {
                panic!()
            };
            let (called, skipped) = if disjunction {
                (else_body, then_body)
            } else {
                (then_body, else_body)
            };
            assert!(
                matches!(called.as_slice(), [NirStmt::Return(Some(NirExpr::Call { callee, .. }))] if callee == tail)
            );
            assert_eq!(
                skipped,
                &[NirStmt::Return(Some(NirExpr::Bool(disjunction)))]
            );
        }
        let carries = control_values::layouts(&module);
        assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
        let before = module.clone();
        assert!(outlined(&mut module).is_empty());
        assert_eq!(module, before);
    }
}

#[test]
fn logical_value_join_supports_either_arm_and_ready_computed_or_logical_conditions() {
    for condition_kind in 0..3 {
        for arm_kind in 0..3 {
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
            match condition_kind {
                0 => *condition = NirExpr::Var("choice".into()),
                1 => {
                    *condition = NirExpr::Call {
                        callee: "decide".into(),
                        args: vec![
                            NirExpr::Var("choice".into()),
                            NirExpr::Var("condition".into()),
                        ],
                    }
                }
                _ => {}
            }
            if arm_kind != 0 {
                let body = if arm_kind == 1 { then_body } else { else_body };
                let NirStmt::Let { value, .. } = &mut body[0] else {
                    panic!()
                };
                *value = NirExpr::Bool(false);
            }
            assert_eq!(
                outlined(&mut module).len(),
                if arm_kind == 0 { 3 } else { 2 }
            );
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
    let mut module = module();
    let f = module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap();
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &mut f.body[1]
    else {
        panic!()
    };
    for body in [then_body, else_body] {
        let NirStmt::Let { value, .. } = &mut body[0] else {
            panic!()
        };
        *value = binary(
            NirBinaryOp::And,
            NirExpr::Var("choice".into()),
            NirExpr::Bool(true),
        );
    }
    assert_eq!(outlined(&mut module).len(), 1);
}

fn chain(edges: usize) -> NirExpr {
    (0..edges).fold(NirExpr::Bool(true), |lhs, _| {
        binary(NirBinaryOp::And, lhs, NirExpr::Bool(true))
    })
}

#[test]
fn logical_value_join_inline_checked_rhs_keeps_selected_and_skipped_faults() {
    for (gate, head, divisor, expected) in [
        (true, false, 0, Some(0)),
        (false, true, 0, Some(1)),
        (true, true, 1, Some(1)),
        (true, true, 2, Some(0)),
        (false, false, 1, Some(1)),
        (false, false, 2, Some(0)),
        (true, true, 0, None),
        (false, false, 0, None),
    ] {
        let source = format!("mod cpu Main {{
            @noinline fn entry(gate: bool, head: bool, divisor: i64) -> i64 {{
                print(81);
                let value: bool = if gate {{ head && (1 / divisor != 0) }} else {{ head || (1 / divisor != 0) }};
                if value {{ return 1; }} return 0;
            }}
            fn main() -> i64 {{ return entry({gate}, {head}, {divisor}); }}
        }}");
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        );
        if let Some(expected) = expected {
            let trace = result.unwrap();
            let main = compiled
                .yir
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap();
            assert_eq!(
                trace.values[&main.result.as_ref().unwrap().node],
                yir_core::Value::Int(expected)
            );
        } else {
            let error = result.unwrap_err();
            assert!(error.contains("zero"), "{error}");
        }
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}

#[test]
fn logical_value_join_shares_all_root_budgets_and_keeps_hidden_logic_and_scope_closed() {
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
        let NirStmt::Let { value: yes, ty, .. } = &mut then_body[0] else {
            panic!()
        };
        let NirStmt::Let { value: no, .. } = &mut else_body[0] else {
            panic!()
        };
        match mutation {
            0 => {
                *condition = chain(11);
                *yes = chain(11);
                *no = chain(11);
                for root in [&*condition, &*yes, &*no] {
                    assert!(conditional_values::prefix::computed_logical_root(root));
                }
            }
            1 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..10 {
                    tree = binary(NirBinaryOp::Add, tree.clone(), tree);
                }
                *condition = binary(NirBinaryOp::Eq, tree.clone(), NirExpr::Int(1));
                *no = binary(NirBinaryOp::Eq, tree, NirExpr::Int(1));
                for root in [&*condition, &*yes, &*no] {
                    assert!(conditional_values::prefix::computed_logical_root(root));
                }
            }
            2 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..64 {
                    tree = binary(NirBinaryOp::Add, tree, NirExpr::Int(0));
                }
                *yes = binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    binary(NirBinaryOp::Eq, tree, NirExpr::Int(1)),
                );
            }
            3 => {
                *yes = binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(true),
                    NirExpr::Call {
                        callee: "decide".into(),
                        args: vec![chain(1), NirExpr::Int(1)],
                    },
                )
            }
            4 => *yes = binary(NirBinaryOp::Eq, yes.clone(), NirExpr::Bool(true)),
            5 => *ty = Some(scalar_type("i64")),
            6 => then_body.insert(0, NirStmt::Print(NirExpr::Int(88))),
            7 => else_body.clear(),
            8 => f.params.push(NirParam {
                name: "selected".into(),
                ty: scalar_type("bool"),
            }),
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(outlined(&mut module).is_empty(), "mutation {mutation}");
        assert_eq!(module, before);
    }
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
    *condition = chain(10);
    for body in [then_body, else_body] {
        let NirStmt::Let { value, .. } = &mut body[0] else {
            panic!()
        };
        *value = chain(11);
    }
    assert_eq!(outlined(&mut module).len(), 1);
}
