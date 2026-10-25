use super::*;

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/logical_record_fields.ns"
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
    .logical_field_declaration(&mut before[1], &mut scope, suffix);
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

fn expr_edges(expr: &NirExpr) -> usize {
    match expr {
        NirExpr::Binary { op, lhs, rhs } => {
            usize::from(matches!(op, NirBinaryOp::And | NirBinaryOp::Or))
                + expr_edges(lhs)
                + expr_edges(rhs)
        }
        NirExpr::Call { args, .. } => args.iter().map(expr_edges).sum(),
        NirExpr::StructLiteral { fields, .. } => fields.iter().map(|(_, v)| expr_edges(v)).sum(),
        NirExpr::FieldAccess { base, .. } => expr_edges(base),
        _ => 0,
    }
}
fn edges(body: &[NirStmt]) -> usize {
    body.iter()
        .map(|stmt| match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Return(Some(value)) => expr_edges(value),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => expr_edges(condition) + edges(then_body) + edges(else_body),
            _ => 0,
        })
        .sum()
}

#[test]
fn logical_record_fields_have_independent_authority_and_no_remaining_field_gates() {
    for simplify in [false, true] {
        for reversed in [false, true] {
            let mut module = module(simplify);
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
            let count = edges(&f.body[1..2]);
            assert!(count >= 4);
            assert!(scalar_joins::nested_prefix::outer_record_declaration(
                &f.body[1],
                &scope,
                &f.body[2..],
                &catalog,
                &layouts
            )
            .is_none());
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let generated = module
                .functions
                .iter()
                .filter(|f| f.name.starts_with("__nuis_conditional_value"))
                .collect::<Vec<_>>();
            assert_eq!(
                generated
                    .iter()
                    .filter(|f| f.return_type.as_ref() == Some(&scalar_type("bool")))
                    .count(),
                count
            );
            assert_eq!(generated.len(), count + 3);
            assert!(generated.iter().all(|f| f
                .params
                .iter()
                .all(|p| !matches!(p.name.as_str(), "picked" | "unused" | "p"))));
            assert_eq!(
                module
                    .functions
                    .iter()
                    .map(|f| edges(&f.body))
                    .sum::<usize>(),
                0
            );
            let before = module.clone();
            assert!(!attempt(&mut module, None));
            assert_eq!(module, before);
            let carries = control_values::layouts(&module);
            assert!(
                scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet")
            );
        }
    }
}

#[test]
fn logical_record_fields_revoke_completed_dependencies_and_effects_atomically() {
    for name in [
        "discarded_decide",
        "discarded_first",
        "discarded_last",
        "outer_word",
        "outer_unused",
        "field_head",
        "field_tail",
        "field_spare",
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
    let mut other = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/outer_record_prefix.ns"
    ))
    .unwrap();
    reject(&mut other);
}

#[test]
fn logical_record_fields_reject_hidden_arguments_comparisons_projections_and_bad_layouts() {
    for mutation in 0..9 {
        let mut module = module(mutation == 4);
        if mutation == 6 {
            module
                .structs
                .iter_mut()
                .find(|s| s.name == "Record")
                .unwrap()
                .fields
                .iter_mut()
                .find(|f| f.name == "tag")
                .unwrap()
                .ty = scalar_type("i64");
        }
        if mutation == 7 {
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
        if mutation == 8 {
            f.params.push(NirParam {
                name: "picked".into(),
                ty: scalar_type("Record"),
            });
        }
        let (_, yes, _) = arms(&mut f.body[1]);
        let original = value(yes.last_mut().unwrap()).clone();
        let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
            panic!()
        };
        let index = fields.iter().position(|(n, _)| n == "tag").unwrap();
        let gate = fields[index].1.clone();
        match mutation {
            0 => {
                fields[index].1 = NirExpr::Call {
                    callee: "discarded_decide".into(),
                    args: vec![gate, NirExpr::Int(1)],
                }
            }
            1 => fields[index].1 = binary(NirBinaryOp::Eq, gate, NirExpr::Bool(true)),
            2 => {
                fields[index].1 = NirExpr::FieldAccess {
                    base: Box::new(original),
                    field: "tag".into(),
                }
            }
            3 => {
                fields.iter_mut().find(|(n, _)| n == "check").unwrap().1 = gate;
                fields[index].1 = NirExpr::Bool(true);
            }
            4 => {
                fields[index].1 = binary(
                    NirBinaryOp::And,
                    NirExpr::FieldAccess {
                        base: Box::new(NirExpr::Var("unused".into())),
                        field: "tag".into(),
                    },
                    NirExpr::Bool(true),
                )
            }
            5 => {
                fields.pop();
            }
            6 | 7 | 8 => {}
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!attempt(&mut module, None), "mutation {mutation}");
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
fn logical_record_fields_share_original_statement_edge_and_expression_budgets() {
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
                *condition = (0..if mutation == 2 { 26 } else { 27 })
                    .fold(NirExpr::Bool(true), |lhs, _| {
                        binary(NirBinaryOp::And, lhs, NirExpr::Bool(true))
                    })
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
}

#[test]
fn logical_record_fields_keep_nested_local_scopes_and_unprojected_field_failures() {
    for (outer, gate, divisor, expected) in [
        (true, false, 0, Some(7)),
        (false, true, 0, Some(9)),
        (true, true, 1, Some(7)),
        (false, false, 1, Some(9)),
        (true, true, 0, None),
        (false, false, 0, None),
    ] {
        let source = format!(
            r#"mod cpu Main {{
            struct Flag {{ tag:bool }} struct R {{ flag:Flag, value:i64 }}
            @noinline fn checked(divisor:i64)->bool {{return 1/divisor!=0;}}
            @noinline fn packet(outer:bool,gate:bool,divisor:i64)->R {{
                print(99);
                let picked:R=if outer {{
                    const earlier:i64=divisor;
                    let local:R=R {{flag:Flag {{tag:gate && checked(earlier)}},value:7}};
                    local
                }}else{{
                    const earlier:i64=divisor;
                    let local:R=R {{value:9,flag:Flag {{tag:gate || checked(earlier)}}}};
                    local
                }};
                return picked;
            }}
            fn main()->i64 {{return packet({outer},{gate},{divisor}).value;}}
        }}"#
        );
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert!(attempt(&mut module, None));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert_eq!(
            module
                .functions
                .iter()
                .map(|f| edges(&f.body))
                .sum::<usize>(),
            0
        );
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
            assert!(result.unwrap_err().contains("zero"));
        }
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}
