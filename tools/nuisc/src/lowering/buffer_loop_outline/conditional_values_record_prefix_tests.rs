use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/record_prefix_join.ns"
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
    match stmt {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
        _ => panic!(),
    }
}
fn attempt(module: &mut NirModule, denied: Option<&str>) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let mut catalog = scalar_helpers::collect_capture_values(module, &layouts);
    if let Some(name) = denied {
        catalog.remove(name);
    }
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let old_names = names.clone();
    let mut helpers = Vec::new();
    let checked = BTreeSet::new();
    let f = module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap();
    let mut scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    let old_scope = scope.clone();
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
    .record_declaration(&mut before[1], &mut scope, suffix);
    if !accepted {
        assert_eq!(scope, old_scope);
        assert_eq!(names, old_names);
        assert!(helpers.is_empty());
    }
    module.functions.extend(helpers);
    accepted
}
fn reject(module: &mut NirModule) {
    let old = module.clone();
    assert!(!attempt(module, None));
    assert_eq!(module, &old);
}

#[test]
fn record_prefix_join_requires_completed_field_dependencies_without_widening_scalar_proof() {
    let original = module();
    let layouts = control_values::TypedLayouts::collect(&original);
    let catalog = scalar_helpers::collect_capture_values(&original, &layouts);
    let f = packet(&original);
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
    .is_some());
    for dependency in [
        "record_word",
        "record_unused",
        "record_seed",
        "nested_decide",
        "prefix_seed",
        "prefix_alt",
    ] {
        for effectful in [false, true] {
            let mut module = original.clone();
            if effectful {
                module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == dependency)
                    .unwrap()
                    .body
                    .insert(0, NirStmt::Print(NirExpr::Int(99)));
            }
            let old = module.clone();
            assert!(!attempt(&mut module, Some(dependency)), "{dependency}");
            assert_eq!(module, old);
            if effectful {
                assert!(!attempt(&mut module, None));
                assert_eq!(module, old);
            }
        }
    }
    let scalar = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/nested_prefix_join.ns"
    ))
    .unwrap();
    let mut scalar = scalar;
    reject(&mut scalar);
}

#[test]
fn record_prefix_join_retains_field_order_layout_and_private_child_scopes() {
    for reversed in [false, true] {
        let mut module = module();
        if reversed {
            module.functions.reverse();
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let generated = outline_declarations(&mut module, &catalog, &layouts, &mut names);
        // The two final field-word comparisons add one guarded bool helper per arm.
        assert_eq!(generated.len(), 7);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let joins = module
            .functions
            .iter()
            .filter(|f| {
                generated.contains(&f.name)
                    && f.return_type.as_ref() == Some(&scalar_type("Choice"))
            })
            .collect::<Vec<_>>();
        assert_eq!(joins.len(), 2);
        for join in joins {
            assert!(join.params.iter().all(|p| p.name != "temp"));
            let [NirStmt::If {
                then_body,
                else_body,
                ..
            }] = join.body.as_slice()
            else {
                panic!()
            };
            for (body, order) in [
                (then_body, vec!["value", "checksum", "tag", "unused"]),
                (else_body, vec!["unused", "tag", "checksum", "value"]),
            ] {
                let Some(NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))) =
                    body.last()
                else {
                    panic!()
                };
                assert_eq!(
                    fields
                        .iter()
                        .map(|(name, _)| name.as_str())
                        .collect::<Vec<_>>(),
                    order
                );
            }
        }
        let before = module.clone();
        assert!(outline_declarations(&mut module, &catalog, &layouts, &mut names).is_empty());
        assert_eq!(module, before);
        let carries = control_values::layouts(&module);
        assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
    }
}

#[test]
fn record_prefix_join_rejects_mismatched_partial_resource_and_hidden_field_shapes() {
    for mutation in 0..9 {
        let mut module = module();
        if mutation == 0 {
            let mut other = module
                .structs
                .iter()
                .find(|s| s.name == "Choice")
                .unwrap()
                .clone();
            other.name = "Other".into();
            module.structs.push(other);
        }
        if matches!(mutation, 4 | 5) {
            let choice = module
                .structs
                .iter_mut()
                .find(|s| s.name == "Choice")
                .unwrap();
            if mutation == 4 {
                choice.fields[0].ty.is_ref = true;
            } else {
                choice.fields[0].ty = scalar_type("Choice");
            }
        }
        let f = module
            .functions
            .iter_mut()
            .find(|f| f.name == "packet")
            .unwrap();
        let (_, outer, _) = arms(&mut f.body[1]);
        let (_, yes, no) = arms(&mut outer[0]);
        match mutation {
            0 => {
                let NirStmt::Let {
                    ty,
                    value: NirExpr::StructLiteral { type_name, .. },
                    ..
                } = no.last_mut().unwrap()
                else {
                    panic!()
                };
                *ty = Some(scalar_type("Other"));
                *type_name = "Other".into();
            }
            1 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields.pop();
            }
            2 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields[1].0 = "value".into();
            }
            3 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields[0].1 = NirExpr::Bool(true);
            }
            4 | 5 => {}
            6 => no.clear(),
            7 => {
                let NirExpr::StructLiteral { fields, .. } = value(yes.last_mut().unwrap()) else {
                    panic!()
                };
                fields[2].1 = binary(NirBinaryOp::And, NirExpr::Bool(true), NirExpr::Bool(false));
            }
            8 => {
                let (_, outer, other) = arms(&mut f.body[1]);
                for body in [outer, other] {
                    let NirStmt::Let { ty, .. } = body.last_mut().unwrap() else {
                        panic!()
                    };
                    *ty = Some(scalar_type("Choice"));
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
fn chain(n: usize) -> NirExpr {
    (0..n).fold(NirExpr::Bool(true), |lhs, _| {
        binary(NirBinaryOp::And, lhs, NirExpr::Bool(true))
    })
}
fn pad(body: &mut Vec<NirStmt>, extra: &mut usize, id: &mut usize) {
    while *extra > 0 && body.len() < 32 {
        body.insert(
            body.len() - 1,
            NirStmt::Let {
                name: format!("budget_{id}"),
                ty: None,
                value: NirExpr::Int(1),
            },
        );
        *extra -= 1;
        *id += 1;
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
fn record_prefix_join_shares_field_expression_statement_and_logical_budgets() {
    for mutation in 0..5 {
        let mut module = module();
        let f = module
            .functions
            .iter_mut()
            .find(|f| f.name == "packet")
            .unwrap();
        let (condition, yes, no) = arms(&mut f.body[1]);
        match mutation {
            0 | 1 => {
                let mut extra = if mutation == 0 { 108 } else { 109 };
                let mut id = 0;
                pad(yes, &mut extra, &mut id);
                pad(no, &mut extra, &mut id);
                assert_eq!(extra, 0);
            }
            2 | 3 => *condition = chain(if mutation == 2 { 26 } else { 27 }),
            4 => {
                let mut expr = NirExpr::Int(1);
                for _ in 0..10 {
                    expr = binary(NirBinaryOp::Add, expr.clone(), expr);
                }
                for arm in [yes, no] {
                    let (_, left, _) = arms(&mut arm[0]);
                    let NirExpr::StructLiteral { fields, .. } = value(left.last_mut().unwrap())
                    else {
                        panic!()
                    };
                    fields
                        .iter_mut()
                        .find(|(name, _)| name == "checksum")
                        .unwrap()
                        .1 = expr.clone();
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
fn record_prefix_join_supports_nested_value_layouts_and_preserves_unused_field_failures() {
    let source = "mod cpu Main { struct Leaf { value: i64 } struct Wrap { leaf: Leaf } fn choose(gate: bool, inner: bool, a: i64, b: i64) -> i64 { print(99); let picked: i64 = if gate { let local: Wrap = if inner { Wrap { leaf: Leaf { value: a } } } else { Wrap { leaf: Leaf { value: b } } }; local.leaf.value } else { b }; return picked; } }";
    let mut module = crate::frontend::parse_nuis_module(source).unwrap();
    let layouts = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    assert_eq!(
        outline_declarations(&mut module, &catalog, &layouts, &mut names).len(),
        2
    );
    crate::nir_verify::verify_nir_module(&module).unwrap();
    for (outer, inner, divisor, expected) in [
        (false, true, 0, Some(9)),
        (true, true, 1, Some(7)),
        (true, false, 1, Some(8)),
        (true, true, 0, None),
        (true, false, 0, None),
    ] {
        let source=format!("mod cpu Main {{ struct Data {{ value:i64, unused:i64 }} @noinline fn entry(outer:bool,inner:bool,divisor:i64)->i64 {{ print(99); let picked:i64=if outer {{ let local:Data=if inner {{ Data {{ value:7, unused:1/divisor }} }} else {{ Data {{ unused:1/divisor, value:8 }} }}; local.value }} else {{9}}; return picked; }} fn main()->i64 {{return entry({outer},{inner},{divisor});}} }}");
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
