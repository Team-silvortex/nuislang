use super::*;
use nuis_semantics::model::{NirParam, NirVisibility};

type Scope = BTreeMap<String, NirTypeRef>;

#[path = "buffer_loop_outline/branches.rs"]
mod branches;
#[path = "buffer_loop_outline/scalar_carries.rs"]
mod scalar_carries;
#[path = "buffer_loop_outline/scalar_control.rs"]
mod scalar_control;
#[path = "buffer_loop_outline/scalar_helpers.rs"]
mod scalar_helpers;
use scalar_helpers::ScalarHelpers;

struct BufferLoopPlan {
    params: Vec<NirParam>,
    carries: Vec<String>,
}

#[derive(Default)]
pub(super) struct BufferLoopOutlines {
    pub functions: BTreeSet<String>,
    pub guarded_functions: BTreeSet<String>,
}

// Keep iteration effects inside a private helper; the existing scoped-call contract
// supplies the induction value and preserves borrowed-buffer lifetime edges.
pub(super) fn outline_buffer_loops(module: &mut NirModule) -> Result<BufferLoopOutlines, String> {
    let catalog = scalar_helpers::collect(module);
    let mut names = module
        .functions
        .iter()
        .map(|function| function.name.clone())
        .chain(module.externs.iter().map(|function| function.name.clone()))
        .chain(
            module
                .structs
                .iter()
                .map(|definition| definition.name.clone()),
        )
        .chain(
            module
                .enums
                .iter()
                .map(|definition| definition.name.clone()),
        )
        .collect::<BTreeSet<_>>();
    let mut helpers = Vec::new();
    let mut outlined = BufferLoopOutlines::default();
    for function in &mut module.functions {
        let mut scope = function
            .params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect();
        outline_body(
            &mut function.body,
            &mut scope,
            &mut names,
            &mut helpers,
            &mut outlined.guarded_functions,
            &catalog,
            &mut outlined.functions,
            &mut module.structs,
        );
    }
    scalar_control::outline(
        module,
        &outlined.functions,
        &mut names,
        &mut helpers,
        &mut outlined.guarded_functions,
        &catalog,
    );
    if !helpers.is_empty() {
        outlined
            .functions
            .extend(helpers.iter().map(|function| function.name.clone()));
        module.functions.extend(helpers);
        crate::nir_verify::verify_nir_module(module)?;
    }
    Ok(outlined)
}

fn outline_body(
    body: &mut [NirStmt],
    scope: &mut Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    retained: &mut BTreeSet<String>,
    structs: &mut Vec<NirStructDef>,
) {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                let inferred = ty.clone().or_else(|| infer_local(value, scope, catalog));
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
                outline_body(
                    then_body,
                    &mut scope.clone(),
                    names,
                    helpers,
                    guarded,
                    catalog,
                    retained,
                    structs,
                );
                outline_body(
                    else_body,
                    &mut scope.clone(),
                    names,
                    helpers,
                    guarded,
                    catalog,
                    retained,
                    structs,
                );
            }
            NirStmt::While { condition, body } => {
                if let Some(plan) = buffer_loop_params(condition, body, scope, catalog) {
                    scalar_helpers::retain_reachable(body, catalog, retained);
                    let mut suffix = helpers.len();
                    let name = loop {
                        let name = format!("__nuis_buffer_iteration_{suffix}");
                        if names.insert(name.clone()) {
                            break name;
                        }
                        suffix += 1;
                    };
                    let step = body.pop().expect("validated counted step");
                    let aggregate = (plan.carries.len() > 1)
                        .then(|| scalar_carries::state_type(&plan.carries, names, structs));
                    let returned = if let Some(ty) = &aggregate {
                        NirExpr::StructLiteral {
                            type_name: ty.name.clone(),
                            type_args: vec![],
                            fields: plan
                                .carries
                                .iter()
                                .enumerate()
                                .map(|(index, name)| {
                                    (format!("carry{index}"), NirExpr::Var(name.clone()))
                                })
                                .collect(),
                        }
                    } else if !plan.carries.is_empty() {
                        let NirStmt::Let { value, .. } =
                            body.pop().expect("validated carry update")
                        else {
                            unreachable!("validated carry binding")
                        };
                        value
                    } else {
                        NirExpr::Int(0)
                    };
                    let mut helper_body = branches::outline_branches(
                        std::mem::take(body),
                        &mut scope.clone(),
                        names,
                        helpers,
                        guarded,
                        catalog,
                    );
                    helper_body.push(NirStmt::Return(Some(returned)));
                    let args = plan
                        .params
                        .iter()
                        .map(|param| NirExpr::Var(param.name.clone()))
                        .collect();
                    let call = NirExpr::Call {
                        callee: name.clone(),
                        args,
                    };
                    let mut function = helper(name, plan.params, helper_body);
                    if let Some(ty) = aggregate {
                        let mut used = scope.keys().cloned().collect();
                        branches::collect_bindings(&function.body, &mut used);
                        let temporary = branches::fresh_name("__nuis_loop_state", &mut used);
                        *body = scalar_carries::projected_call(temporary, &ty, &plan.carries, call);
                        body.push(step);
                        function.return_type = Some(ty);
                    } else {
                        let action = match plan.carries.into_iter().next() {
                            Some(name) => NirStmt::Let {
                                name,
                                ty: Some(scalar_type("i64")),
                                value: call,
                            },
                            None => NirStmt::Expr(call),
                        };
                        *body = vec![action, step];
                    }
                    helpers.push(function);
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
    catalog: &ScalarHelpers,
) -> Option<BufferLoopPlan> {
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
    if scalar_expr(
        &prepared.limit,
        scope,
        &mut header_inputs,
        false,
        &ScalarHelpers::new(),
    )? != scalar_type("i64")
        || header_inputs.contains(&prepared.binding_name)
    {
        return None;
    }
    let mut inputs = BTreeSet::new();
    let mut locals = scope.clone();
    let mut carry_start = effects.len();
    while let Some(NirStmt::Let { name, .. }) = effects.get(carry_start.wrapping_sub(1)) {
        if !scope.contains_key(name) {
            break;
        }
        carry_start -= 1;
    }
    let (effects, updates) = effects.split_at(carry_start);
    if !validate_effects(effects, &mut locals, &mut inputs, catalog)? {
        return None;
    }
    let mut carries = Vec::new();
    for update in updates {
        if let NirStmt::Let { name, ty, value } = update {
            if name == &prepared.binding_name
                || header_inputs.contains(name)
                || carries.contains(name)
                || scope.get(name)? != &scalar_type("i64")
                || ty.as_ref().is_some_and(|ty| ty != &scalar_type("i64"))
            {
                return None;
            }
            if scalar_expr(value, &locals, &mut inputs, true, catalog)? != scalar_type("i64") {
                return None;
            }
            // Keep all seeds, including replacing updates, for the zero-trip result.
            inputs.insert(name.clone());
            carries.push(name.clone());
        } else {
            return None;
        }
    }
    Some(BufferLoopPlan {
        params: captured_params(inputs, scope),
        carries,
    })
}

fn captured_params(inputs: BTreeSet<String>, scope: &Scope) -> Vec<NirParam> {
    inputs
        .into_iter()
        .filter_map(|name| {
            scope.get(&name).map(|ty| NirParam {
                name: name.clone(),
                ty: ty.clone(),
            })
        })
        .collect()
}

fn validate_effects(
    effects: &[NirStmt],
    locals: &mut Scope,
    inputs: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
) -> Option<bool> {
    let mut has_store = false;
    for stmt in effects {
        match stmt {
            NirStmt::Let { name, ty, value } => {
                // Rebinding captured state would require explicit loop carries.
                if locals.contains_key(name) {
                    return None;
                }
                let inferred = scalar_expr(value, locals, inputs, true, catalog)?;
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
                buffer_input(buffer, locals, inputs)?;
                if scalar_expr(index, locals, inputs, true, catalog)? != scalar_type("i64")
                    || scalar_expr(value, locals, inputs, true, catalog)? != scalar_type("i64")
                {
                    return None;
                }
                has_store = true;
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if scalar_expr(condition, locals, inputs, true, catalog)? != scalar_type("bool") {
                    return None;
                }
                has_store |= validate_effects(then_body, &mut locals.clone(), inputs, catalog)?;
                has_store |= validate_effects(else_body, &mut locals.clone(), inputs, catalog)?;
            }
            _ => return None,
        }
    }
    Some(has_store)
}

fn scalar_expr(
    expr: &NirExpr,
    scope: &Scope,
    inputs: &mut BTreeSet<String>,
    reads: bool,
    catalog: &ScalarHelpers,
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
            let lhs = scalar_expr(lhs, scope, inputs, reads, catalog)?;
            let rhs = scalar_expr(rhs, scope, inputs, reads, catalog)?;
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
            (scalar_expr(index, scope, inputs, reads, catalog)? == scalar_type("i64"))
                .then(|| scalar_type("i64"))
        }
        NirExpr::Call { callee, args } => {
            scalar_helpers::call_type(callee, args, scope, inputs, reads, catalog)
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

fn infer_local(expr: &NirExpr, scope: &Scope, catalog: &ScalarHelpers) -> Option<NirTypeRef> {
    if matches!(expr, NirExpr::AllocBuffer { .. }) {
        let mut ty = scalar_type("Buffer");
        ty.is_ref = true;
        Some(ty)
    } else {
        scalar_expr(expr, scope, &mut BTreeSet::new(), true, catalog)
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
