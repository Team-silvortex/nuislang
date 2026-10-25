use super::*;

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/outer_record_prefix.ns"
    ))
    .unwrap();
    if simplify {
        crate::optimize::simplify_nir_module(&mut module);
    }
    module
}
fn packet(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap()
}
fn arms(stmt: &mut NirStmt) -> (&mut NirExpr, &mut Vec<NirStmt>, &mut Vec<NirStmt>) {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        panic!()
    };
    (condition, then_body, else_body)
}
fn value(stmt: &mut NirStmt) -> &mut NirExpr {
    let NirStmt::Let { value, .. } = stmt else {
        panic!()
    };
    value
}
fn attempt(module: &mut NirModule, denied: Option<&str>) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let mut catalog = scalar_helpers::collect_capture_values(module, &layouts);
    if let Some(name) = denied {
        catalog.remove(name);
    }
    let mut names = module
        .functions
        .iter()
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    let before_names = names.clone();
    let mut helpers = Vec::new();
    let checked = BTreeSet::new();
    let f = packet(module);
    let mut scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    let before_scope = scope.clone();
    let mut bindings = scope.keys().cloned().collect();
    branches::collect_bindings(&f.body, &mut bindings);
    let (before, suffix) = f.body.split_at_mut(2);
    let accepted = Builder {
        catalog: &catalog,
        layouts: &layouts,
        names: &mut names,
        checked: &checked,
        helpers: &mut helpers,
        bindings,
        extract_selections: false,
        predicate_returns: false,
    }
    .outer_record_declaration(&mut before[1], &mut scope, suffix);
    if !accepted {
        assert_eq!(names, before_names);
        assert_eq!(scope, before_scope);
        assert!(helpers.is_empty());
    }
    module.functions.extend(helpers);
    accepted
}
fn reject(module: &mut NirModule) {
    let before = module.clone();
    assert!(!attempt(module, None));
    assert_eq!(module, &before);
}

#[test]
fn outer_record_prefix_has_independent_authority_and_preserves_layout_order_and_scopes() {
    for reversed in [false, true] {
        let mut module = module(true);
        if reversed {
            module.functions.reverse();
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let f = packet(&mut module);
        let scope = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        assert!(scalar_joins::nested_prefix::declaration(
            &f.body[1],
            &scope,
            &f.body[2..],
            &catalog,
            &layouts
        )
        .is_none());
        assert!(scalar_joins::nested_prefix::record_declaration(
            &f.body[1],
            &scope,
            &f.body[2..],
            &catalog,
            &layouts
        )
        .is_none());
        assert!(scalar_joins::nested_prefix::discarded_record_declaration(
            &f.body[1],
            &scope,
            &f.body[2..],
            &catalog,
            &layouts
        )
        .is_none());
        assert!(attempt(&mut module, None));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let NirStmt::Let {
            value: NirExpr::Call { callee, .. },
            ty: Some(ty),
            ..
        } = &packet(&mut module).body[1]
        else {
            panic!()
        };
        assert_eq!(ty, &scalar_type("Record"));
        let callee = callee.clone();
        let helpers = module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_conditional_value"))
            .collect::<Vec<_>>();
        assert_eq!(helpers.len(), 3);
        assert!(helpers.iter().all(|f| f
            .params
            .iter()
            .all(|p| !matches!(p.name.as_str(), "picked" | "unused" | "p"))));
        let root = module.functions.iter().find(|f| f.name == callee).unwrap();
        assert_eq!(root.return_type.as_ref(), Some(&scalar_type("Record")));
        let [NirStmt::If {
            then_body,
            else_body,
            ..
        }] = root.body.as_slice()
        else {
            panic!()
        };
        for (body, outer_order, leaf_order) in [
            (
                then_body,
                vec!["leaf", "tag", "check"],
                vec!["value", "unused"],
            ),
            (
                else_body,
                vec!["tag", "check", "leaf"],
                vec!["unused", "value"],
            ),
        ] {
            assert!(
                matches!(&body[0],NirStmt::Let {name,..} if name.starts_with("__nuis_discarded_value"))
            );
            let Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) = body.last()
            else {
                panic!()
            };
            assert_eq!(
                fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
                outer_order
            );
            let NirExpr::StructLiteral { fields, .. } =
                &fields.iter().find(|(n, _)| n == "leaf").unwrap().1
            else {
                panic!()
            };
            assert_eq!(
                fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
                leaf_order
            );
        }
        let before = module.clone();
        assert!(!attempt(&mut module, None));
        assert_eq!(module, before);
        let carries = control_values::layouts(&module);
        assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
    }
}

#[test]
fn outer_record_prefix_revokes_field_and_nested_dependencies_before_allocation() {
    for name in [
        "discarded_decide",
        "discarded_first",
        "discarded_last",
        "outer_word",
        "outer_unused",
    ] {
        let mut module = module(true);
        let before = module.clone();
        assert!(!attempt(&mut module, Some(name)));
        assert_eq!(module, before);
        module
            .functions
            .iter_mut()
            .find(|f| f.name == name)
            .unwrap()
            .body
            .insert(0, NirStmt::Print(NirExpr::Int(99)));
        reject(&mut module);
    }
    for source in [
        include_str!("../../../tests/native_application_bridge/record_prefix_join.ns"),
        include_str!("../../../tests/native_application_bridge/discarded_record_prefix.ns"),
    ] {
        let mut other = crate::frontend::parse_nuis_module(source).unwrap();
        crate::optimize::simplify_nir_module(&mut other);
        reject(&mut other);
    }
}

#[test]
fn outer_record_prefix_rejects_nominal_partial_unobserved_resource_and_effectful_results() {
    for mutation in 0..12 {
        let mut module = module(true);
        if mutation == 0 {
            let mut other = module
                .structs
                .iter()
                .find(|s| s.name == "Record")
                .unwrap()
                .clone();
            other.name = "Other".into();
            module.structs.push(other);
        }
        if mutation == 10 {
            module
                .structs
                .iter_mut()
                .find(|s| s.name == "Leaf")
                .unwrap()
                .fields[0]
                .ty
                .is_ref = true;
        }
        let f = packet(&mut module);
        if mutation == 3 {
            f.body.truncate(2);
            f.body.push(NirStmt::Return(Some(NirExpr::StructLiteral {
                type_name: "Packet".into(),
                type_args: vec![],
                fields: vec![
                    ("value".into(), NirExpr::Int(7)),
                    ("unused".into(), NirExpr::Int(0)),
                ],
            })));
        }
        if mutation == 4 {
            f.params.push(NirParam {
                name: "picked".into(),
                ty: scalar_type("Record"),
            });
        }
        let (_, yes, no) = arms(&mut f.body[1]);
        match mutation {
            0 => {
                let NirStmt::Let {
                    ty: Some(ty),
                    value: NirExpr::StructLiteral { type_name, .. },
                    ..
                } = no.last_mut().unwrap()
                else {
                    panic!()
                };
                *ty = scalar_type("Other");
                *type_name = "Other".into();
            }
            1 => no.clear(),
            2 => {
                let NirStmt::Let { ty, .. } = yes.last_mut().unwrap() else {
                    panic!()
                };
                *ty = None;
            }
            3 | 4 | 10 => {}
            5 => {
                let NirStmt::Let { ty: Some(ty), .. } = yes.last_mut().unwrap() else {
                    panic!()
                };
                ty.is_ref = true;
            }
            6 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields.pop();
            }
            7 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields[1].1 = binary(NirBinaryOp::And, NirExpr::Bool(true), NirExpr::Bool(false));
            }
            8 => yes.insert(0, NirStmt::Print(NirExpr::Int(99))),
            9 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields[2].1 = NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var("unused".into())),
                    field: "check".into(),
                };
            }
            11 => {
                for body in [yes, no] {
                    let NirStmt::Let { ty, value, .. } = body.last_mut().unwrap() else {
                        panic!()
                    };
                    *ty = Some(scalar_type("i64"));
                    *value = NirExpr::Int(1);
                }
            }
            _ => unreachable!(),
        }
        reject(&mut module);
    }
}
fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}
fn pad(body: &mut Vec<NirStmt>, extra: &mut usize, id: &mut usize) {
    while *extra > 0 && !body.is_empty() && body.len() < 32 {
        body.insert(
            body.len() - 1,
            NirStmt::Let {
                name: format!("padding_{id}"),
                ty: None,
                value: NirExpr::Int(1),
            },
        );
        *id += 1;
        *extra -= 1;
    }
    for stmt in body {
        if let NirStmt::If {
            then_body,
            else_body,
            ..
        } = stmt
        {
            pad(then_body, extra, id);
            pad(else_body, extra, id);
        }
    }
}

#[test]
fn outer_record_prefix_keeps_complete_source_budgets_and_neutral_preflight() {
    for mutation in 0..5 {
        let mut module = module(false);
        let (condition, yes, no) = arms(&mut packet(&mut module).body[1]);
        match mutation {
            0 | 1 => {
                let mut extra = if mutation == 0 { 118 } else { 119 };
                let mut id = 0;
                pad(yes, &mut extra, &mut id);
                pad(no, &mut extra, &mut id);
                assert_eq!(extra, 0);
            }
            2 | 3 => {
                *condition = (0..if mutation == 2 { 32 } else { 33 })
                    .fold(NirExpr::Bool(true), |lhs, _| {
                        binary(NirBinaryOp::And, lhs, NirExpr::Bool(true))
                    });
            }
            4 => {
                let mut expr = NirExpr::Int(1);
                for _ in 0..10 {
                    expr = binary(NirBinaryOp::Add, expr.clone(), expr);
                }
                for body in [yes, no] {
                    let NirExpr::StructLiteral { fields, .. } = value(body.last_mut().unwrap())
                    else {
                        panic!()
                    };
                    fields.iter_mut().find(|(n, _)| n == "check").unwrap().1 = expr.clone();
                }
            }
            _ => unreachable!(),
        }
        if matches!(mutation, 0 | 2) {
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            reject(&mut module);
        }
    }
    // The outer-result entry must not bypass the existing whole-declaration preflight.
    for (fields, accepted) in [(2044, true), (2045, false)] {
        let mut module = module(true);
        let wide = module
            .structs
            .iter_mut()
            .find(|s| s.name == "Leaf")
            .unwrap();
        let field = wide.fields[0].clone();
        wide.fields.extend((2..fields).map(|i| {
            let mut field = field.clone();
            field.name = format!("extra_{i}");
            field
        }));
        let layouts = control_values::TypedLayouts::collect(&module);
        assert!(control_values::supported_type(
            &scalar_type("Record"),
            &layouts
        ));
        let f = packet(&mut module);
        f.params.push(NirParam {
            name: "seed".into(),
            ty: scalar_type("Record"),
        });
        let (_, yes, no) = arms(&mut f.body[1]);
        for body in [yes, no] {
            *value(body.last_mut().unwrap()) = NirExpr::Var("seed".into());
            let (_, child_yes, child_no) = arms(&mut body[0]);
            let retained = if child_yes.is_empty() {
                child_no
            } else {
                child_yes
            };
            *value(retained.last_mut().unwrap()) = NirExpr::Var("seed".into());
        }
        if accepted {
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            reject(&mut module);
        }
    }
}

#[test]
fn outer_record_prefix_supports_flat_prefixes_and_all_plain_scalar_field_kinds() {
    for kind in ["i64", "i32", "bool", "f32", "f64"] {
        let source = format!(
            r#"
mod cpu Main {{
    struct R {{ value:{kind}, unused:i64 }}
    fn packet(gate:bool,a:{kind},b:{kind},divisor:i64)->R {{
        print(81);
        let picked:R=if gate {{
            const checked:i64=1/divisor;
            R {{ value:a, unused:checked }}
        }} else {{
            const checked:i64=2/divisor;
            R {{ unused:checked, value:b }}
        }};
        print(82);
        return picked;
    }}
}}
"#
        );
        for prefix in [true, false] {
            let source = if prefix {
                source.clone()
            } else {
                source
                    .replace("const checked:i64=1/divisor;", "")
                    .replace("const checked:i64=2/divisor;", "")
                    .replace("unused:checked", "unused:1/divisor")
            };
            let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
            assert!(attempt(&mut module, None), "{kind} prefix={prefix}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert_eq!(
                module
                    .functions
                    .iter()
                    .filter(|f| f.name.starts_with("__nuis_conditional_value"))
                    .count(),
                1
            );
        }
    }
}
