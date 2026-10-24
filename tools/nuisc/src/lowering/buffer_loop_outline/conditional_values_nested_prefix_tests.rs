use super::*;

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/nested_prefix_join.ns"
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

fn outlined(module: &mut NirModule) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let catalog = scalar_helpers::collect_capture_values(module, &layouts);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    outline_declarations(module, &catalog, &layouts, &mut names)
}

// Exercise only this admission rule: failed parents must not borrow successful
// child transformations from the general declaration walk.
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
    .nested_declaration(&mut before[1], &mut scope, suffix);
    if !accepted {
        assert_eq!(names, old_names);
        assert_eq!(scope, old_scope);
        assert!(helpers.is_empty());
    }
    module.functions.extend(helpers);
    accepted
}

#[test]
fn nested_prefix_join_proves_completed_dependencies_before_allocating_any_helper() {
    let original = module();
    let layouts = control_values::TypedLayouts::collect(&original);
    let catalog = scalar_helpers::collect_capture_values(&original, &layouts);
    let f = packet(&original);
    let scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    assert!(scalar_joins::prefixed::declaration(
        &f.body[1],
        &scope,
        &f.body[2..],
        &catalog,
        &layouts
    )
    .is_none());
    for dependency in [
        "decide",
        "nested_decide",
        "prefix_seed",
        "prefix_alt",
        "prefix_step",
        "yes_head",
        "yes_tail",
        "no_head",
        "no_tail",
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
            let before = module.clone();
            assert!(!attempt(&mut module, Some(dependency)), "{dependency}");
            assert_eq!(module, before);
            if effectful {
                assert!(
                    !attempt(&mut module, None),
                    "transitive effect {dependency}"
                );
                assert_eq!(module, before);
            }
        }
    }
}

#[test]
fn nested_prefix_join_keeps_child_scopes_private_and_is_order_independent_and_idempotent() {
    for reversed in [false, true] {
        let mut module = module();
        if reversed {
            module.functions.reverse();
        }
        let generated = outlined(&mut module);
        assert_eq!(generated.len(), 5);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let NirStmt::Let {
            value: NirExpr::Call { callee, args },
            ..
        } = &packet(&module).body[1]
        else {
            panic!()
        };
        assert!(args[1..].iter().all(|a| matches!(a, NirExpr::Var(_))));
        let selected = module.functions.iter().find(|f| &f.name == callee).unwrap();
        assert!(selected
            .params
            .iter()
            .all(|p| !["seed", "temp", "head", "prepared", "unused"].contains(&p.name.as_str())));
        let [NirStmt::If {
            then_body,
            else_body,
            ..
        }] = selected.body.as_slice()
        else {
            panic!()
        };
        for body in [then_body, else_body] {
            assert!(
                matches!(&body[0], NirStmt::Let { name, ty: Some(ty), value: NirExpr::Call { callee, .. } } if name == "seed" && ty == &scalar_type("i64") && generated.contains(callee))
            );
            assert!(matches!(&body[3], NirStmt::Let { name, .. } if name == "unused"));
        }
        let carries = control_values::layouts(&module);
        assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
        let before = module.clone();
        assert!(outlined(&mut module).is_empty());
        assert_eq!(module, before);
    }
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

fn rejected(module: &mut NirModule) {
    let before = module.clone();
    assert!(!attempt(module, None));
    assert_eq!(module, &before);
}

#[test]
fn nested_prefix_join_rejects_scope_leaks_partial_joins_effects_and_hidden_logic_atomically() {
    for mutation in 0..11 {
        let mut module = module();
        let f = module
            .functions
            .iter_mut()
            .find(|f| f.name == "packet")
            .unwrap();
        let (_, yes, _) = arms(&mut f.body[1]);
        match mutation {
            0 => {
                let (_, left, _) = arms(&mut yes[0]);
                let NirStmt::Const { name, .. } = &mut left[0] else {
                    panic!()
                };
                *name = "selected".into();
            }
            1 => *value(yes.last_mut().unwrap()) = NirExpr::Var("temp".into()),
            2 => {
                let (_, _, right) = arms(&mut yes[0]);
                let NirStmt::Let { name, .. } = right.last_mut().unwrap() else {
                    panic!()
                };
                *name = "other".into();
            }
            3 => {
                let (_, left, _) = arms(&mut yes[0]);
                let NirStmt::Let { ty, .. } = left.last_mut().unwrap() else {
                    panic!()
                };
                *ty = None;
            }
            4 => {
                let (_, _, right) = arms(&mut yes[0]);
                right.clear();
            }
            5 => {
                let (_, left, _) = arms(&mut yes[0]);
                left.insert(0, NirStmt::Print(NirExpr::Int(99)));
            }
            6 => {
                let NirStmt::Const { name, .. } = &mut yes[2] else {
                    panic!()
                };
                *name = "prepared".into();
            }
            7 => {
                let (_, left, _) = arms(&mut yes[0]);
                *value(&mut left[0]) = NirExpr::Var("later".into());
            }
            8 => {
                let (condition, _, _) = arms(&mut yes[0]);
                *condition = NirExpr::Call {
                    callee: "nested_decide".into(),
                    args: vec![chain(1), NirExpr::Int(1)],
                };
            }
            9 => {
                let (_, left, _) = arms(&mut yes[0]);
                let NirStmt::Const { ty, .. } = &mut left[0] else {
                    panic!()
                };
                ty.is_ref = true;
            }
            10 => {
                let (_, left, right) = arms(&mut yes[0]);
                left.clear();
                right.clear();
            }
            _ => unreachable!(),
        }
        rejected(&mut module);
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
        let at = body.len() - 1;
        body.insert(
            at,
            NirStmt::Let {
                name: format!("budget_{id}"),
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

fn tree(levels: usize, depth: usize) -> NirStmt {
    let name = if depth == 0 {
        "seed".into()
    } else {
        format!("child_{depth}")
    };
    let mut yes = Vec::new();
    let expr = if levels > 1 {
        yes.push(tree(levels - 1, depth + 1));
        NirExpr::Var(format!("child_{}", depth + 1))
    } else {
        NirExpr::Int(1)
    };
    yes.push(NirStmt::Let {
        name: name.clone(),
        ty: Some(scalar_type("i64")),
        value: expr,
    });
    NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: yes,
        else_body: vec![NirStmt::Let {
            name,
            ty: Some(scalar_type("i64")),
            value: NirExpr::Int(0),
        }],
    }
}

#[test]
fn nested_prefix_join_shares_statement_expression_edge_and_nesting_budgets() {
    for mutation in 0..7 {
        let mut module = module();
        let f = module
            .functions
            .iter_mut()
            .find(|f| f.name == "packet")
            .unwrap();
        let (condition, yes, no) = arms(&mut f.body[1]);
        match mutation {
            0 | 1 => {
                let mut extra = if mutation == 0 { 110 } else { 111 };
                let mut id = 0;
                pad(yes, &mut extra, &mut id);
                pad(no, &mut extra, &mut id);
                assert_eq!(extra, 0);
            }
            2 | 3 => yes[0] = tree(if mutation == 2 { 8 } else { 9 }, 0),
            4 | 5 => {
                *condition = chain(if mutation == 4 { 8 } else { 9 });
                *value(&mut yes[2]) = chain(10);
                *value(&mut no[2]) = chain(10);
            }
            6 => {
                let mut expr = NirExpr::Int(1);
                for _ in 0..10 {
                    expr = binary(NirBinaryOp::Add, expr.clone(), expr);
                }
                assert!(prefix::computed_logical_root(&expr));
                let (_, left, _) = arms(&mut yes[0]);
                *value(&mut left[0]) = expr.clone();
                let (_, left, _) = arms(&mut no[0]);
                *value(&mut left[0]) = expr;
            }
            _ => unreachable!(),
        }
        if matches!(mutation, 0 | 2 | 4) {
            assert!(attempt(&mut module, None), "{mutation}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            rejected(&mut module);
        }
    }
}

fn run(source: &str, expected: Option<i64>) {
    let compiled = crate::pipeline::compile_source(source).unwrap();
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

#[test]
fn nested_prefix_join_preserves_unused_fallible_selections_and_all_scalar_types() {
    for kind in ["i64", "i32", "bool", "f32", "f64"] {
        let source = format!("mod cpu Main {{ fn choose(gate: bool, inner: bool, a: {kind}, b: {kind}) -> {kind} {{ print(99); let picked: {kind} = if gate {{ let local: {kind} = if inner {{ a }} else {{ b }}; local }} else {{ b }}; return picked; }} }}");
        let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
        assert_eq!(outlined(&mut module).len(), 2, "{kind}");
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
    for (outer, inner, divisor, expected) in [
        (false, true, 0, Some(9)),
        (true, false, 0, Some(7)),
        (true, true, 1, Some(7)),
        (true, true, 0, None),
    ] {
        for unused in [
            "if inner { 1 / divisor } else { 0 }",
            "if inner && (1 / divisor != 0) { 0 } else { 1 }",
        ] {
            run(&format!("mod cpu Main {{ @noinline fn entry(outer: bool, inner: bool, divisor: i64) -> i64 {{ print(99); let picked: i64 = if outer {{ let unused: i64 = {unused}; 7 }} else {{ 9 }}; return picked; }} fn main() -> i64 {{ return entry({outer}, {inner}, {divisor}); }} }}"), expected);
        }
    }
}

#[test]
fn nested_prefix_join_normalizes_logical_local_before_dependent_selection() {
    for (gate, divisor, expected) in [
        (false, 0, Some(19)),
        (true, 1, Some(11)),
        (true, 2, Some(19)),
        (true, 0, None),
    ] {
        run(&format!("mod cpu Main {{ @noinline fn entry(outer: bool, gate: bool, divisor: i64) -> i64 {{ print(99); let picked: i64 = if outer {{ let ready = gate && (1 / divisor != 0); let local: i64 = if ready {{ 11 }} else {{ 19 }}; local }} else {{ 0 }}; return picked; }} fn main() -> i64 {{ return entry(true, {gate}, {divisor}); }} }}"), expected);
    }
}
