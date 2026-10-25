use super::*;

fn module(simplify: bool) -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/direct_record_return.ns"
    ))
    .unwrap();
    if simplify {
        crate::optimize::simplify_nir_module(&mut module);
    }
    module
}
pub(super) fn producer(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "returned_record")
        .unwrap()
}
pub(super) fn attempt(module: &mut NirModule, denied: Option<&str>) -> bool {
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
    let f = producer(module);
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    let mut bindings = scope.keys().cloned().collect();
    branches::collect_bindings(&f.body, &mut bindings);
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
    .direct_record_returns(&mut f.body, &scope, f.return_type.as_ref());
    if !accepted {
        assert_eq!(names, before_names);
        assert!(helpers.is_empty());
    }
    module.functions.extend(helpers);
    accepted
}
pub(super) fn reject(module: &mut NirModule) {
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
        NirExpr::StructLiteral { fields, .. } => fields.iter().map(|(_, v)| expr_edges(v)).sum(),
        NirExpr::Call { args, .. } => args.iter().map(expr_edges).sum(),
        NirExpr::FieldAccess { base, .. } => expr_edges(base),
        _ => 0,
    }
}
pub(super) fn edges(body: &[NirStmt]) -> usize {
    body.iter()
        .map(|s| match s {
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
fn returned(stmt: &mut NirStmt) -> &mut NirExpr {
    let NirStmt::Return(Some(v)) = stmt else {
        panic!()
    };
    v
}
fn first_return(f: &mut NirFunction) -> &mut NirExpr {
    let NirStmt::If { then_body, .. } = &mut f.body[1] else {
        panic!()
    };
    returned(then_body.last_mut().unwrap())
}

#[test]
fn direct_record_return_preserves_effect_boundaries_original_scopes_and_idempotence() {
    for simplify in [false, true] {
        for reversed in [false, true] {
            let mut module = module(simplify);
            if reversed {
                module.functions.reverse();
            }
            let before = producer(&mut module).clone();
            let count = edges(&before.body);
            assert!(count >= 4);
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let f = producer(&mut module);
            assert_eq!(f.params, before.params);
            assert_eq!(f.return_type, before.return_type);
            assert!(matches!(f.body[0], NirStmt::Print(NirExpr::Int(86))));
            let NirStmt::If { then_body, .. } = &f.body[1] else {
                panic!()
            };
            assert!(matches!(then_body[0], NirStmt::Print(NirExpr::Int(87))));
            assert!(matches!(f.body[2], NirStmt::Print(NirExpr::Int(88))));
            assert_eq!(edges(&f.body), 0);
            let generated = module
                .functions
                .iter()
                .filter(|f| f.name.starts_with("__nuis_conditional_value"))
                .collect::<Vec<_>>();
            assert_eq!(generated.len(), count);
            assert!(generated
                .iter()
                .all(|f| f.return_type.as_ref() == Some(&scalar_type("bool"))));
            let before = module.clone();
            assert!(!attempt(&mut module, None));
            assert_eq!(module, before);
        }
    }
}

#[test]
fn direct_record_return_revokes_prefix_and_field_dependencies_atomically() {
    for name in [
        "return_prefix",
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
}

#[test]
fn direct_record_return_rejects_bad_types_hidden_leaves_scope_leaks_and_unsupported_control() {
    for mutation in 0..13 {
        let mut module = module(false);
        if mutation == 5 {
            module
                .structs
                .iter_mut()
                .find(|s| s.name == "Leaf")
                .unwrap()
                .fields[0]
                .ty
                .is_ref = true;
        }
        let f = producer(&mut module);
        match mutation {
            0 => {
                let value = first_return(f);
                let NirExpr::StructLiteral { fields, .. } = value else {
                    panic!()
                };
                let tag = fields.iter_mut().find(|(n, _)| n == "tag").unwrap();
                tag.1 = NirExpr::Call {
                    callee: "discarded_decide".into(),
                    args: vec![tag.1.clone(), NirExpr::Int(1)],
                };
            }
            1 => {
                let NirExpr::StructLiteral { fields, .. } = first_return(f) else {
                    panic!()
                };
                let tag = fields.iter_mut().find(|(n, _)| n == "tag").unwrap();
                tag.1 = binary(NirBinaryOp::Eq, tag.1.clone(), NirExpr::Bool(true));
            }
            2 => {
                let NirExpr::StructLiteral { fields, .. } = first_return(f) else {
                    panic!()
                };
                fields.pop();
            }
            3 => {
                f.body.pop();
            }
            4 => f.return_type.as_mut().unwrap().is_ref = true,
            5 => {}
            6 => f.body.push(NirStmt::Print(NirExpr::Int(99))),
            7 => {
                let NirStmt::If { then_body, .. } = &mut f.body[1] else {
                    panic!()
                };
                let NirStmt::Const { name, .. } = &mut then_body[1] else {
                    panic!()
                };
                *name = "value".into();
            }
            8 => {
                let NirExpr::StructLiteral { fields, .. } = returned(f.body.last_mut().unwrap())
                else {
                    panic!()
                };
                fields.iter_mut().find(|(n, _)| n == "check").unwrap().1 =
                    NirExpr::Var("branch_only".into());
            }
            9 => f.body.insert(
                0,
                NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body: vec![],
                },
            ),
            10 => {
                let value = first_return(f);
                let NirExpr::StructLiteral { type_name, .. } = value else {
                    panic!()
                };
                *type_name = "Packet".into();
            }
            11 => {
                for stmt in &mut f.body {
                    if let NirStmt::If { then_body, .. } = stmt {
                        let NirExpr::StructLiteral { fields, .. } =
                            returned(then_body.last_mut().unwrap())
                        else {
                            panic!()
                        };
                        fields.iter_mut().find(|(n, _)| n == "tag").unwrap().1 =
                            NirExpr::Bool(true);
                        fields.iter_mut().find(|(n, _)| n == "spare").unwrap().1 =
                            NirExpr::Bool(false);
                    } else if let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) =
                        stmt
                    {
                        fields.iter_mut().find(|(n, _)| n == "tag").unwrap().1 =
                            NirExpr::Bool(false);
                        fields.iter_mut().find(|(n, _)| n == "spare").unwrap().1 =
                            NirExpr::Bool(false);
                    }
                }
            }
            12 => f.body.insert(
                0,
                NirStmt::Print(binary(
                    NirBinaryOp::And,
                    NirExpr::Bool(false),
                    NirExpr::Call {
                        callee: "field_tail".into(),
                        args: vec![NirExpr::Int(0)],
                    },
                )),
            ),
            _ => unreachable!(),
        }
        let before = module.clone();
        assert!(!attempt(&mut module, None), "mutation {mutation}");
        assert_eq!(module, before);
    }
    let mut module = module(false);
    producer(&mut module).is_async = true;
    let before = producer(&mut module).clone();
    let layouts = control_values::TypedLayouts::collect(&module);
    let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    outline_declarations(&mut module, &catalog, &layouts, &mut names);
    assert_eq!(producer(&mut module), &before);
}
fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}
fn pad(body: &mut Vec<NirStmt>, extra: &mut usize, id: &mut usize) {
    while *extra > 0 && body.len() < 32 {
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
fn direct_record_return_shares_function_statement_expression_edge_and_depth_budgets() {
    for mutation in 0..3 {
        let mut module = module(false);
        let f = producer(&mut module);
        if mutation < 2 {
            let NirStmt::If { condition, .. } = &mut f.body[1] else {
                panic!()
            };
            *condition = (0..if mutation == 0 { 26 } else { 27 })
                .fold(NirExpr::Bool(true), |lhs, _| {
                    binary(NirBinaryOp::And, lhs, NirExpr::Bool(true))
                });
        } else {
            let mut expr = NirExpr::Int(1);
            for _ in 0..10 {
                expr = binary(NirBinaryOp::Add, expr.clone(), expr);
            }
            let NirExpr::StructLiteral { fields, .. } = first_return(f) else {
                panic!()
            };
            fields.iter_mut().find(|(n, _)| n == "check").unwrap().1 = expr.clone();
            let NirExpr::StructLiteral { fields, .. } = returned(f.body.last_mut().unwrap()) else {
                panic!()
            };
            fields.iter_mut().find(|(n, _)| n == "check").unwrap().1 = expr;
        }
        if mutation == 0 {
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            reject(&mut module);
        }
    }
    let source="mod cpu Main {struct R {tag:bool,value:i64} @noinline fn field_tail(divisor:i64)->bool{return 1/divisor!=0;} fn returned_record(outer:bool,gate:bool,divisor:i64,value:i64)->R {return R{tag:gate&&field_tail(divisor),value:value};}}";
    let base = crate::frontend::parse_nuis_module(source).unwrap();
    let leaf = producer(&mut base.clone()).body[0].clone();
    let branch = |yes: Vec<NirStmt>, no: Vec<NirStmt>| NirStmt::If {
        condition: NirExpr::Var("outer".into()),
        then_body: yes,
        else_body: no,
    };
    for extra in [121, 122] {
        let mut module = base.clone();
        let f = producer(&mut module);
        f.body = vec![branch(
            vec![branch(vec![leaf.clone()], vec![leaf.clone()])],
            vec![branch(vec![leaf.clone()], vec![leaf.clone()])],
        )];
        let mut remaining = extra;
        let mut id = 0;
        pad(&mut f.body, &mut remaining, &mut id);
        assert_eq!(remaining, 0);
        if extra == 121 {
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            reject(&mut module);
        }
    }
    for depth in [8, 9] {
        let mut module = base.clone();
        let mut tree = leaf.clone();
        for _ in 0..depth {
            tree = branch(vec![tree], vec![leaf.clone()]);
        }
        producer(&mut module).body = vec![tree];
        if depth == 8 {
            assert!(attempt(&mut module, None));
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            reject(&mut module);
        }
    }
}

#[test]
fn direct_record_return_executes_unprojected_logical_fields_after_prefix_effects() {
    for (op, gate, divisor, prefix, expected) in [
        ("&&", false, 0, 1, Some(7)),
        ("||", true, 0, 1, Some(7)),
        ("&&", true, 1, 1, Some(7)),
        ("||", false, 1, 1, Some(7)),
        ("&&", true, 0, 1, None),
        ("||", false, 0, 1, None),
        ("&&", false, 0, 0, None),
    ] {
        let source=format!("mod cpu Main {{struct R {{value:i64,unused:bool}} @noinline fn checked(divisor:i64)->bool{{return 1/divisor!=0;}} @noinline fn returned_record(value:i64,gate:bool,divisor:i64,prefix:i64)->R{{print(81);const prepared:i64=value+0/prefix;return R{{value:prepared,unused:gate {op} checked(divisor)}};}}fn main()->i64{{return returned_record(7,{gate},{divisor},{prefix}).value;}}}}");
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
