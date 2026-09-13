use super::*;

pub(super) struct ScalarHelper {
    params: Vec<NirTypeRef>,
    result: NirTypeRef,
    dependencies: BTreeSet<String>,
    pub(super) may_loop: bool,
}

pub(super) type ScalarHelpers = BTreeMap<String, ScalarHelper>;

pub(super) fn collect(module: &NirModule) -> ScalarHelpers {
    collect_profile(module, &control_values::FlatLayouts::new(), false)
}

pub(super) fn collect_with_layouts(
    module: &NirModule,
    layouts: &control_values::FlatLayouts,
) -> ScalarHelpers {
    collect_profile(module, layouts, true)
}

fn collect_profile(
    module: &NirModule,
    layouts: &control_values::FlatLayouts,
    allow_loops: bool,
) -> ScalarHelpers {
    let mut candidates = module
        .functions
        .iter()
        .filter(|function| {
            !function.is_async
                && function.generic_params.is_empty()
                && function.where_bounds.is_empty()
                && function
                    .params
                    .iter()
                    .all(|param| control_values::supported_type(&param.ty, layouts))
                && function
                    .return_type
                    .as_ref()
                    .is_some_and(|ty| control_values::supported_type(ty, layouts))
        })
        .map(|function| {
            (
                function.name.clone(),
                ScalarHelper {
                    params: function
                        .params
                        .iter()
                        .map(|param| param.ty.clone())
                        .collect(),
                    result: function.return_type.clone().expect("scalar return"),
                    dependencies: BTreeSet::new(),
                    may_loop: control_loops::contains_loop(&function.body),
                },
            )
        })
        .collect::<ScalarHelpers>();
    let valid = module
        .functions
        .iter()
        .filter(|function| candidates.contains_key(&function.name))
        .filter(|function| validate_body(function, &candidates, layouts, allow_loops).is_some())
        .map(|function| {
            let mut dependencies = BTreeSet::new();
            collect_calls(&function.body, &mut dependencies);
            (function.name.clone(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    candidates.retain(|name, helper| {
        if let Some(dependencies) = valid.get(name) {
            helper.dependencies.clone_from(dependencies);
            true
        } else {
            false
        }
    });

    // Admit leaves before callers. Missing callees and cycles never become ready;
    // discovery does not recurse down an arbitrarily deep source call graph.
    let mut remaining = BTreeMap::new();
    let mut callers = BTreeMap::<String, Vec<String>>::new();
    let mut ready = BTreeSet::new();
    for (name, helper) in &candidates {
        remaining.insert(name.clone(), helper.dependencies.len());
        if helper.dependencies.is_empty() {
            ready.insert(name.clone());
        }
        for callee in &helper.dependencies {
            callers
                .entry(callee.clone())
                .or_default()
                .push(name.clone());
        }
    }
    let mut admitted = ScalarHelpers::new();
    while let Some(name) = ready.pop_first() {
        let mut helper = candidates.remove(&name).expect("ready helper");
        helper.may_loop |= helper
            .dependencies
            .iter()
            .any(|name| admitted[name].may_loop);
        admitted.insert(name.clone(), helper);
        for caller in callers.get(&name).into_iter().flatten() {
            let count = remaining.get_mut(caller).expect("known caller");
            *count -= 1;
            if *count == 0 {
                ready.insert(caller.clone());
            }
        }
    }
    admitted
}

fn validate_body(
    function: &NirFunction,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
    allow_loops: bool,
) -> Option<()> {
    let mut locals = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    validate_block(
        &function.body,
        &mut locals,
        &mut BTreeSet::new(),
        function.return_type.as_ref()?,
        catalog,
        layouts,
        allow_loops,
    )?
    .then_some(())
}

// Branch scopes never export bindings; every path through the helper must return.
fn validate_block(
    body: &[NirStmt],
    locals: &mut Scope,
    loop_bindings: &mut BTreeSet<String>,
    result: &NirTypeRef,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
    allow_loops: bool,
) -> Option<bool> {
    let mut returned = false;
    for stmt in body {
        if returned {
            return None;
        }
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                let declared = match stmt {
                    NirStmt::Let { ty, .. } => ty.as_ref(),
                    NirStmt::Const { ty, .. } => Some(ty),
                    _ => unreachable!(),
                };
                if locals.contains_key(name) {
                    return None;
                }
                let inferred = control_values::value_type(value, locals, catalog, layouts)?;
                if declared.is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                locals.insert(name.clone(), inferred);
                if matches!(stmt, NirStmt::Let { .. }) {
                    loop_bindings.insert(name.clone());
                }
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if control_values::value_type(condition, locals, catalog, layouts)?
                    != scalar_type("bool")
                {
                    return None;
                }
                let validate_arm = |body: &[NirStmt]| {
                    validate_block(
                        body,
                        &mut locals.clone(),
                        &mut loop_bindings.clone(),
                        result,
                        catalog,
                        layouts,
                        allow_loops,
                    )
                };
                let then_returns = validate_arm(then_body)?;
                let else_returns = validate_arm(else_body)?;
                returned = then_returns && else_returns;
            }
            NirStmt::While { condition, body } if allow_loops => {
                control_loops::validate(condition, body, locals, loop_bindings)?;
            }
            NirStmt::Return(Some(value)) => {
                if &control_values::value_type(value, locals, catalog, layouts)? != result {
                    return None;
                }
                returned = true;
            }
            _ => return None,
        }
    }
    Some(returned)
}

pub(super) fn call_type(
    callee: &str,
    args: &[NirExpr],
    scope: &Scope,
    inputs: &mut BTreeSet<String>,
    reads: bool,
    catalog: &ScalarHelpers,
) -> Option<NirTypeRef> {
    let helper = catalog.get(callee)?;
    if args.len() != helper.params.len() {
        return None;
    }
    for (arg, expected) in args.iter().zip(&helper.params) {
        if &scalar_expr(arg, scope, inputs, reads, catalog)? != expected {
            return None;
        }
    }
    Some(helper.result.clone())
}

pub(super) fn value_call_type(
    callee: &str,
    args: &[NirExpr],
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
) -> Option<NirTypeRef> {
    let helper = catalog.get(callee)?;
    if args.len() != helper.params.len() {
        return None;
    }
    for (arg, expected) in args.iter().zip(&helper.params) {
        if &control_values::value_type(arg, scope, catalog, layouts)? != expected {
            return None;
        }
    }
    Some(helper.result.clone())
}

pub(super) fn retain_reachable(
    body: &[NirStmt],
    catalog: &ScalarHelpers,
    retained: &mut BTreeSet<String>,
) {
    let mut pending = BTreeSet::new();
    collect_calls(body, &mut pending);
    while let Some(name) = pending.pop_first() {
        if let Some(helper) = catalog.get(&name) {
            if retained.insert(name) {
                pending.extend(helper.dependencies.iter().cloned());
            }
        }
    }
}

fn collect_calls(body: &[NirStmt], calls: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => collect_expr_calls(value, calls),
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_expr_calls(condition, calls);
                collect_calls(then_body, calls);
                collect_calls(else_body, calls);
            }
            NirStmt::While { condition, body } => {
                collect_expr_calls(condition, calls);
                collect_calls(body, calls);
            }
            _ => {}
        }
    }
}

fn collect_expr_calls(expr: &NirExpr, calls: &mut BTreeSet<String>) {
    match expr {
        NirExpr::Call { callee, args } => {
            calls.insert(callee.clone());
            for arg in args {
                collect_expr_calls(arg, calls);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_expr_calls(lhs, calls);
            collect_expr_calls(rhs, calls);
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_expr_calls(value, calls);
            }
        }
        NirExpr::FieldAccess { base, .. } => collect_expr_calls(base, calls),
        NirExpr::LoadAt { index, .. } => collect_expr_calls(index, calls),
        NirExpr::StoreAt { index, value, .. } => {
            collect_expr_calls(index, calls);
            collect_expr_calls(value, calls);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parse_nuis_module;

    #[test]
    fn scalar_helper_admission_checks_transitive_bodies_and_cycles() {
        let module = parse_nuis_module(
            r#"
          mod cpu Main {
            fn main() -> i64 { return 0; }
            fn leaf(v: i64) -> i64 { return v; }
            fn good(v: i64) -> i64 { return leaf(v) + leaf(v); }
            fn impure(v: i64) -> i64 { print(v); return v; }
            fn wrapper(v: i64) -> i64 { return impure(v); }
            fn recursive(v: i64) -> i64 { return recursive(v); }
            fn cycle_a(v: i64) -> i64 { return cycle_b(v); }
            fn cycle_b(v: i64) -> i64 { return cycle_a(v); }
            fn outer(v: i64) -> i64 { return cycle_a(v); }
            fn branch(v: i64) -> i64 { if v > 0 { return v; } return 0; }
          }
        "#,
        )
        .unwrap();
        assert_eq!(
            collect(&module)
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["branch", "good", "leaf", "main"]
        );
    }

    #[test]
    fn scalar_helper_admission_checks_signature_arity_and_return_types() {
        let module = parse_nuis_module(
            r#"
          mod cpu Main {
            fn main() -> i64 { return 0; }
            fn leaf(v: i64) -> i64 { return v; }
            fn caller(v: i64) -> i64 { return leaf(v); }
          }
        "#,
        )
        .unwrap();
        for variant in 0..6 {
            let mut invalid = module.clone();
            let leaf = invalid
                .functions
                .iter_mut()
                .find(|f| f.name == "leaf")
                .unwrap();
            match variant {
                0 => leaf.is_async = true,
                1 => leaf.params[0].ty.is_ref = true,
                2 => leaf.params[0].ty.is_optional = true,
                3 => leaf.params.push(NirParam {
                    name: "second".into(),
                    ty: scalar_type("i64"),
                }),
                4 => leaf.return_type = Some(scalar_type("bool")),
                5 => leaf.body = vec![NirStmt::Return(Some(NirExpr::Var("missing".into())))],
                _ => unreachable!(),
            }
            let catalog = collect(&invalid);
            assert!(!catalog.contains_key("caller"), "variant {variant}");
            if variant != 3 {
                assert!(!catalog.contains_key("leaf"), "variant {variant}");
            }
        }
    }

    #[test]
    fn scalar_helper_discovery_and_reachability_are_iterative() {
        let mut module =
            parse_nuis_module("mod cpu Main { fn main() -> i64 { return 0; } }").unwrap();
        for index in 0..4096 {
            let value = if index == 4095 {
                NirExpr::Var("v".into())
            } else {
                NirExpr::Call {
                    callee: format!("helper_{}", index + 1),
                    args: vec![NirExpr::Var("v".into())],
                }
            };
            module.functions.push(helper(
                format!("helper_{index}"),
                vec![NirParam {
                    name: "v".into(),
                    ty: scalar_type("i64"),
                }],
                vec![NirStmt::Return(Some(value))],
            ));
        }
        let catalog = collect(&module);
        assert_eq!(catalog.len(), 4097);
        let body = vec![NirStmt::Return(Some(NirExpr::Call {
            callee: "helper_0".into(),
            args: vec![NirExpr::Int(1)],
        }))];
        let mut retained = BTreeSet::new();
        retain_reachable(&body, &catalog, &mut retained);
        assert_eq!(retained.len(), 4096);
        assert!(!retained.contains("main"));
        module.functions.reverse();
        assert!(collect(&module).keys().eq(catalog.keys()));
    }

    #[test]
    fn scalar_helper_control_admission_checks_every_path_and_scope() {
        let module = parse_nuis_module(
            "mod cpu Main { fn main() -> i64 { return 0; } \
            fn leaf(v: i64) -> i64 { return v; } \
            fn caller(v: i64) -> i64 { return leaf(v); } }",
        )
        .unwrap();
        let result = NirStmt::Return(Some(NirExpr::Var("v".into())));
        let branch = |body| NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: body,
            else_body: vec![],
        };
        let binding = |name: &str| NirStmt::Let {
            name: name.into(),
            ty: Some(scalar_type("i64")),
            value: NirExpr::Int(1),
        };
        for body in [
            vec![branch(vec![result.clone()])],
            vec![
                branch(vec![NirStmt::Return(Some(NirExpr::Bool(false)))]),
                result.clone(),
            ],
            vec![
                branch(vec![binding("inner")]),
                NirStmt::Return(Some(NirExpr::Var("inner".into()))),
            ],
            vec![branch(vec![binding("v")]), result.clone()],
            vec![
                branch(vec![NirStmt::Print(NirExpr::Int(1))]),
                result.clone(),
            ],
            vec![
                branch(vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "leaf".into(),
                    args: vec![NirExpr::Var("v".into())],
                }))]),
                result.clone(),
            ],
            vec![
                branch(vec![NirStmt::Return(Some(NirExpr::Call {
                    callee: "missing".into(),
                    args: vec![],
                }))]),
                result.clone(),
            ],
            vec![
                branch(vec![NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body: vec![],
                }]),
                result.clone(),
            ],
        ] {
            let mut invalid = module.clone();
            invalid
                .functions
                .iter_mut()
                .find(|f| f.name == "leaf")
                .unwrap()
                .body = body;
            let catalog = collect(&invalid);
            assert!(!catalog.contains_key("leaf"));
            assert!(!catalog.contains_key("caller"));
        }
    }

    #[test]
    fn scalar_helper_control_outlining_shares_linear_suffixes() {
        let mut module = parse_nuis_module(
            "mod cpu Main { fn main() -> i64 { return 0; } fn candidate(v: i64) -> i64 { return v; } }",
        ).unwrap();
        let candidate = module
            .functions
            .iter_mut()
            .find(|f| f.name == "candidate")
            .unwrap();
        let mut body = Vec::new();
        for index in 0..64 {
            body.push(NirStmt::If {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Gt,
                    lhs: Box::new(NirExpr::Var("v".into())),
                    rhs: Box::new(NirExpr::Int(index)),
                },
                then_body: vec![NirStmt::Let {
                    name: "local".into(),
                    ty: Some(scalar_type("i64")),
                    value: NirExpr::Binary {
                        op: NirBinaryOp::Div,
                        lhs: Box::new(NirExpr::Int(1)),
                        rhs: Box::new(NirExpr::Var("v".into())),
                    },
                }],
                else_body: vec![],
            });
        }
        body.push(NirStmt::Return(Some(NirExpr::Var("v".into()))));
        candidate.body = body;
        let catalog = collect(&module);
        assert!(catalog.contains_key("candidate"));
        let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
        let mut helpers = Vec::new();
        let mut guarded = BTreeSet::new();
        scalar_control::outline(
            &mut module,
            &BTreeSet::from(["candidate".into()]),
            &mut names,
            &mut helpers,
            &mut guarded,
            &catalog,
            &control_values::FlatLayouts::new(),
        );
        assert_eq!(helpers.len(), 64 * 3);
        assert_eq!(guarded.len(), 64 * 2);
        assert!(helpers.iter().all(|f| f.params.len() <= 2));
        assert!(helpers.iter().map(|f| f.body.len()).sum::<usize>() < 64 * 12);
        module.functions.extend(helpers);
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}
