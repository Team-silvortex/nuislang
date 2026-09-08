use super::*;

pub(super) struct ScalarHelper {
    params: Vec<NirTypeRef>,
    result: NirTypeRef,
    dependencies: BTreeSet<String>,
}

pub(super) type ScalarHelpers = BTreeMap<String, ScalarHelper>;

pub(super) fn collect(module: &NirModule) -> ScalarHelpers {
    let mut candidates = module
        .functions
        .iter()
        .filter(|function| {
            !function.is_async
                && function.generic_params.is_empty()
                && function.where_bounds.is_empty()
                && function.params.iter().all(|param| is_scalar(&param.ty))
                && function.return_type.as_ref().is_some_and(is_scalar)
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
                },
            )
        })
        .collect::<ScalarHelpers>();
    let valid = module
        .functions
        .iter()
        .filter(|function| candidates.contains_key(&function.name))
        .filter(|function| validate_body(function, &candidates).is_some())
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
        admitted.insert(
            name.clone(),
            candidates.remove(&name).expect("ready helper"),
        );
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

fn is_scalar(ty: &NirTypeRef) -> bool {
    ty == &scalar_type("i64") || ty == &scalar_type("bool")
}

fn validate_body(function: &NirFunction, catalog: &ScalarHelpers) -> Option<()> {
    let (NirStmt::Return(Some(result)), bindings) = function.body.split_last()? else {
        return None;
    };
    let mut locals = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    let mut inputs = BTreeSet::new();
    for stmt in bindings {
        let (name, declared, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value),
            _ => return None,
        };
        if locals.contains_key(name) {
            return None;
        }
        let inferred = scalar_expr(value, &locals, &mut inputs, false, catalog)?;
        if declared.is_some_and(|ty| ty != &inferred) {
            return None;
        }
        locals.insert(name.clone(), inferred);
    }
    let inferred = scalar_expr(result, &locals, &mut inputs, false, catalog)?;
    (function.return_type.as_ref() == Some(&inferred)).then_some(())
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
            ["good", "leaf", "main"]
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
}
