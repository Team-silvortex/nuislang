use super::*;
use nuis_semantics::model::{NirParam, NirVisibility};

type Scope = BTreeMap<String, NirTypeRef>;

// Keep iteration effects inside a private helper; the existing scoped-call contract
// supplies the induction value and preserves borrowed-buffer lifetime edges.
pub(super) fn outline_buffer_loops(module: &mut NirModule) -> Result<(), String> {
    let mut names = module
        .functions
        .iter()
        .map(|function| function.name.clone())
        .chain(module.externs.iter().map(|function| function.name.clone()))
        .collect::<BTreeSet<_>>();
    let mut helpers = Vec::new();
    for function in &mut module.functions {
        let mut scope = function
            .params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect();
        outline_body(&mut function.body, &mut scope, &mut names, &mut helpers);
    }
    if !helpers.is_empty() {
        module.functions.extend(helpers);
        crate::nir_verify::verify_nir_module(module)?;
    }
    Ok(())
}

fn outline_body(
    body: &mut [NirStmt],
    scope: &mut Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
) {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let inferred = ty.clone().or_else(|| infer_local(value, scope));
                scope.remove(name);
                if let Some(ty) = inferred {
                    scope.insert(name.clone(), ty);
                }
            }
            NirStmt::Const { name, ty, .. } => {
                scope.insert(name.clone(), ty.clone());
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                outline_body(then_body, &mut scope.clone(), names, helpers);
                outline_body(else_body, &mut scope.clone(), names, helpers);
            }
            NirStmt::While { condition, body } => {
                if let Some(params) = buffer_loop_params(condition, body, scope) {
                    let mut suffix = helpers.len();
                    let name = loop {
                        let name = format!("__nuis_buffer_iteration_{suffix}");
                        if names.insert(name.clone()) {
                            break name;
                        }
                        suffix += 1;
                    };
                    let step = body.pop().expect("validated counted step");
                    let mut helper_body = std::mem::take(body);
                    helper_body.push(NirStmt::Return(Some(NirExpr::Int(0))));
                    let args = params
                        .iter()
                        .map(|param| NirExpr::Var(param.name.clone()))
                        .collect();
                    *body = vec![
                        NirStmt::Expr(NirExpr::Call {
                            callee: name.clone(),
                            args,
                        }),
                        step,
                    ];
                    helpers.push(helper(name, params, helper_body));
                }
            }
            _ => {}
        }
    }
}

fn buffer_loop_params(
    condition: &NirExpr,
    body: &[NirStmt],
    scope: &Scope,
) -> Option<Vec<NirParam>> {
    let (step, effects) = body.split_last()?;
    let prepared = prepare_counted_while(
        condition,
        std::slice::from_ref(step),
        &BTreeSet::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )?;
    // Strict comparisons with unit steps cannot wrap before reaching the bound.
    if !matches!(prepared.step, NirExpr::Int(1))
        || !matches!(
            (prepared.compare, prepared.step_kind),
            (PreparedLoopCompare::Lt, PreparedLoopStepKind::Add)
                | (PreparedLoopCompare::Gt, PreparedLoopStepKind::Sub)
        )
        || scope.get(&prepared.binding_name)? != &scalar_type("i64")
    {
        return None;
    }
    let mut header_inputs = BTreeSet::new();
    if scalar_expr(&prepared.limit, scope, &mut header_inputs, false)? != scalar_type("i64")
        || header_inputs.contains(&prepared.binding_name)
    {
        return None;
    }
    let mut locals = scope.clone();
    let mut inputs = BTreeSet::new();
    let mut has_store = false;
    for stmt in effects {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                // Rebinding captured state would require explicit loop carries.
                if locals.contains_key(name) {
                    return None;
                }
                let inferred = scalar_expr(value, &locals, &mut inputs, true)?;
                if ty.as_ref().is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                locals.insert(name.clone(), inferred);
            }
            NirStmt::Expr(NirExpr::StoreAt {
                buffer,
                index,
                value,
            }) => {
                buffer_input(buffer, &locals, &mut inputs)?;
                if scalar_expr(index, &locals, &mut inputs, true)? != scalar_type("i64")
                    || scalar_expr(value, &locals, &mut inputs, true)? != scalar_type("i64")
                {
                    return None;
                }
                has_store = true;
            }
            _ => return None,
        }
    }
    if !has_store {
        return None;
    }
    Some(
        inputs
            .into_iter()
            .filter_map(|name| {
                scope.get(&name).map(|ty| NirParam {
                    name: name.clone(),
                    ty: ty.clone(),
                })
            })
            .collect(),
    )
}

fn scalar_expr(
    expr: &NirExpr,
    scope: &Scope,
    inputs: &mut BTreeSet<String>,
    reads: bool,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::Var(name) => {
            let ty = scope.get(name)?;
            if ty != &scalar_type("i64") && ty != &scalar_type("bool") {
                return None;
            }
            inputs.insert(name.clone());
            Some(ty.clone())
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let lhs = scalar_expr(lhs, scope, inputs, reads)?;
            let rhs = scalar_expr(rhs, scope, inputs, reads)?;
            if lhs != rhs {
                return None;
            }
            match op {
                NirBinaryOp::Add
                | NirBinaryOp::Sub
                | NirBinaryOp::Mul
                | NirBinaryOp::Div
                | NirBinaryOp::Rem
                    if lhs == scalar_type("i64") =>
                {
                    Some(lhs)
                }
                NirBinaryOp::Lt | NirBinaryOp::Le | NirBinaryOp::Gt | NirBinaryOp::Ge
                    if lhs == scalar_type("i64") =>
                {
                    Some(scalar_type("bool"))
                }
                NirBinaryOp::Eq | NirBinaryOp::Ne => Some(scalar_type("bool")),
                NirBinaryOp::And | NirBinaryOp::Or | NirBinaryOp::Xor
                    if lhs == scalar_type("bool") =>
                {
                    Some(lhs)
                }
                _ => None,
            }
        }
        NirExpr::BufferLen(buffer) if reads => {
            buffer_input(buffer, scope, inputs)?;
            Some(scalar_type("i64"))
        }
        NirExpr::LoadAt { buffer, index } if reads => {
            buffer_input(buffer, scope, inputs)?;
            (scalar_expr(index, scope, inputs, reads)? == scalar_type("i64"))
                .then(|| scalar_type("i64"))
        }
        _ => None,
    }
}

fn buffer_input(expr: &NirExpr, scope: &Scope, inputs: &mut BTreeSet<String>) -> Option<()> {
    let NirExpr::Var(name) = expr else {
        return None;
    };
    let ty = scope.get(name)?;
    if !ty.is_ref || ty.is_optional || ty.name != "Buffer" || !ty.generic_args.is_empty() {
        return None;
    }
    inputs.insert(name.clone());
    Some(())
}

fn infer_local(expr: &NirExpr, scope: &Scope) -> Option<NirTypeRef> {
    if matches!(expr, NirExpr::AllocBuffer { .. }) {
        let mut ty = scalar_type("Buffer");
        ty.is_ref = true;
        Some(ty)
    } else {
        scalar_expr(expr, scope, &mut BTreeSet::new(), true)
    }
}

fn scalar_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: vec![],
        is_ref: false,
        is_optional: false,
    }
}

fn helper(name: String, params: Vec<NirParam>, body: Vec<NirStmt>) -> NirFunction {
    NirFunction {
        visibility: NirVisibility::Private,
        name,
        annotations: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: false,
        generic_params: vec![],
        where_bounds: vec![],
        params,
        return_type: Some(scalar_type("i64")),
        body,
    }
}
