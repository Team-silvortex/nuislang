use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/prefixed_scalar_join.ns"
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

fn outlined(module: &mut NirModule) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let pure = scalar_helpers::collect_capture_values(module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    conditional_values::outline_declarations(module, &pure, &layouts, &mut names)
}

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn value(stmt: &mut NirStmt) -> &mut NirExpr {
    match stmt {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
        _ => panic!(),
    }
}

#[test]
fn prefixed_scalar_join_requires_completed_dependencies_before_any_mutation() {
    let original = module();
    let layouts = control_values::TypedLayouts::collect(&original);
    let pure = scalar_helpers::collect_capture_values(&original, &layouts);
    let f = packet(&original);
    let scope = scope(f);
    assert!(super::super::declaration(&f.body[1], &scope, &f.body[2..]).is_none());
    assert!(
        super::super::guarded_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts)
            .is_none()
    );
    assert!(
        super::super::computed_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts)
            .is_none()
    );
    assert!(
        super::super::logical_declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts)
            .is_none()
    );
    assert!(super::super::logical_value_declaration(
        &f.body[1],
        &scope,
        &f.body[2..],
        &pure,
        &layouts
    )
    .is_none());
    assert!(declaration(&f.body[1], &scope, &f.body[2..], &pure, &layouts).is_some());
    for dependency in [
        "decide",
        "prefix_seed",
        "prefix_step",
        "yes_head",
        "yes_tail",
        "no_head",
        "no_tail",
    ] {
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
fn prefixed_scalar_join_keeps_ordered_locals_inside_each_arm_without_exporting_them() {
    for reverse in [false, true] {
        let mut module = module();
        let original = packet(&module).body[1].clone();
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
        let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
        for param in &helper.params {
            assert!(!["seed", "prepared", "head", "unused"].contains(&param.name.as_str()));
        }
        let [NirStmt::If {
            then_body,
            else_body,
            ..
        }] = helper.body.as_slice()
        else {
            panic!()
        };
        let NirStmt::If {
            then_body: yes,
            else_body: no,
            ..
        } = &original
        else {
            panic!()
        };
        for (arm, original) in [(then_body, yes), (else_body, no)] {
            assert_eq!(&arm[..4], &original[..4]);
            assert!(
                matches!(arm.last(), Some(NirStmt::Return(Some(NirExpr::Binary { lhs, .. }))) if matches!(lhs.as_ref(), NirExpr::Call { callee, .. } if generated.contains(callee)))
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
fn prefixed_scalar_join_keeps_rebindings_forward_reads_effects_and_types_closed() {
    for mutation in 0..13 {
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
        match mutation {
            0 | 1 | 2 => {
                let index = usize::from(mutation == 1);
                let (NirStmt::Let { name, .. } | NirStmt::Const { name, .. }) =
                    &mut then_body[index]
                else {
                    panic!()
                };
                *name = match mutation {
                    0 => "yes",
                    1 => "seed",
                    2 => "selected",
                    _ => unreachable!(),
                }
                .into();
            }
            3 => *value(&mut then_body[0]) = NirExpr::Var("prepared".into()),
            4 => *value(&mut then_body[0]) = NirExpr::Var("seed".into()),
            5 => {
                let NirStmt::Const { ty, .. } = &mut then_body[1] else {
                    panic!()
                };
                *ty = scalar_type("i32");
            }
            6 => then_body.insert(0, NirStmt::Print(NirExpr::Int(88))),
            7 => {
                let NirStmt::Let { ty, .. } = then_body.last_mut().unwrap() else {
                    panic!()
                };
                *ty = None;
            }
            8 => else_body.clear(),
            9 => {
                let NirStmt::Let { name, .. } = else_body.last_mut().unwrap() else {
                    panic!()
                };
                *name = "different".into();
            }
            10 => f.params.push(NirParam {
                name: "selected".into(),
                ty: scalar_type("bool"),
            }),
            11 => {
                let NirStmt::Const { ty, .. } = &mut then_body[1] else {
                    panic!()
                };
                ty.is_ref = true;
            }
            12 => {
                *value(&mut then_body[2]) = NirExpr::Call {
                    callee: "decide".into(),
                    args: vec![
                        binary(NirBinaryOp::And, NirExpr::Bool(true), NirExpr::Bool(false)),
                        NirExpr::Int(1),
                    ],
                }
            }
            _ => unreachable!(),
        }
        assert_rejected(&mut module, mutation);
    }
}

fn assert_rejected(module: &mut NirModule, mutation: usize) {
    let before = module.clone();
    let layouts = control_values::TypedLayouts::collect(module);
    let pure = scalar_helpers::collect_capture_values(module, &layouts);
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let old_names = names.clone();
    assert!(
        conditional_values::outline_declarations(module, &pure, &layouts, &mut names).is_empty(),
        "mutation {mutation}"
    );
    assert_eq!(*module, before);
    assert_eq!(names, old_names);
}

fn chain(edges: usize) -> NirExpr {
    (0..edges).fold(NirExpr::Bool(true), |lhs, _| {
        binary(NirBinaryOp::And, lhs, NirExpr::Bool(true))
    })
}

#[test]
fn prefixed_scalar_join_shares_all_source_roots_and_bounds_each_arm() {
    for mutation in 0..5 {
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
            0 | 1 => {
                let last = then_body.pop().unwrap();
                let locals = if mutation == 0 { 31 } else { 32 };
                while then_body.len() < locals {
                    then_body.push(NirStmt::Let {
                        name: format!("fill_{}", then_body.len()),
                        ty: None,
                        value: NirExpr::Int(1),
                    });
                }
                then_body.push(last);
            }
            2 => {
                let mut tree = NirExpr::Int(1);
                for _ in 0..10 {
                    tree = binary(NirBinaryOp::Add, tree.clone(), tree);
                }
                assert!(conditional_values::prefix::computed_logical_root(&tree));
                *value(&mut then_body[0]) = tree.clone();
                *value(&mut else_body[0]) = tree;
            }
            3 | 4 => {
                *condition = chain(if mutation == 3 { 8 } else { 9 });
                *value(&mut then_body[2]) = chain(10);
                *value(&mut else_body[2]) = chain(10);
            }
            _ => unreachable!(),
        }
        if matches!(mutation, 0 | 3) {
            assert_eq!(outlined(&mut module).len(), 3);
        } else {
            assert_rejected(&mut module, mutation);
        }
    }
    let mut module = module();
    let f = module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap();
    let NirStmt::If { then_body, .. } = &mut f.body[1] else {
        panic!()
    };
    for _ in 0..64 {
        let expr = value(&mut then_body[0]);
        *expr = binary(NirBinaryOp::Add, expr.clone(), NirExpr::Int(0));
    }
    assert_rejected(&mut module, 5);
}

#[test]
fn prefixed_scalar_join_supports_all_scalar_results_and_keeps_unused_record_checks() {
    for kind in ["i64", "i32", "bool", "f32", "f64"] {
        let source = format!("mod cpu Main {{ fn choose(gate: bool, a: {kind}, b: {kind}) -> {kind} {{ print(99); let picked: {kind} = if gate {{ const seed: {kind} = a; let stage = seed; stage }} else {{ b }}; return picked; }} }}");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert_eq!(outlined(&mut module).len(), 1, "{kind}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
    for (gate, divisor, expected) in [(false, 0, Some(0)), (true, 1, Some(7)), (true, 0, None)] {
        let source = format!("mod cpu Main {{
            struct Record {{ good: i64, unused: i64 }}
            fn make(divisor: i64) -> Record {{ return Record {{ good: 7, unused: 1 / divisor }}; }}
            fn choose(gate: bool, divisor: i64) -> i64 {{ print(99); let picked: i64 = if gate {{ const saved: Record = make(divisor); let local = saved.good; local }} else {{ 0 }}; return picked; }}
            fn main() -> i64 {{ return choose({gate}, {divisor}); }}
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
