use super::*;
use nuis_semantics::model::{NirParam, NirVisibility};

type Scope = BTreeMap<String, NirTypeRef>;

#[path = "buffer_loop_outline/branches.rs"]
mod branches;
#[path = "buffer_loop_outline/control_flow.rs"]
mod control_flow;
#[path = "buffer_loop_outline/scalar_carries.rs"]
mod scalar_carries;
#[path = "buffer_loop_outline/scalar_control.rs"]
mod scalar_control;
#[path = "buffer_loop_outline/scalar_helpers.rs"]
mod scalar_helpers;
use scalar_helpers::ScalarHelpers;
#[path = "buffer_loop_outline/validation.rs"]
mod validation;
use validation::{buffer_loop_params, captured_params, validate_effects};

struct MutationScope {
    writable: BTreeSet<String>,
    protected: BTreeSet<String>,
}

struct BufferLoopPlan {
    params: Vec<NirParam>,
    carries: Vec<String>,
    induction: String,
    header_inputs: BTreeSet<String>,
    mutations: MutationScope,
    has_store: bool,
    normalized_effects: Option<Vec<NirStmt>>,
    break_flag: Option<String>,
}

#[derive(Default)]
pub(super) struct BufferLoopOutlines {
    pub functions: BTreeSet<String>,
    pub guarded_functions: BTreeSet<String>,
    pub break_controls: BTreeMap<String, String>,
}

// Keep iteration effects inside a private helper; the existing scoped-call contract
// supplies induction/carry values, validated break control and borrowed-buffer lifetimes.
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
    let checked_arithmetic = speculation::collect_checked_arithmetic(module);
    for function in &module.functions {
        if catalog.contains_key(&function.name) && checked_arithmetic.contains(&function.name) {
            outlined.functions.insert(function.name.clone());
            scalar_helpers::retain_reachable(&function.body, &catalog, &mut outlined.functions);
        }
    }
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
            &mut outlined.break_controls,
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
    break_controls: &mut BTreeMap<String, String>,
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
                    break_controls,
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
                    break_controls,
                );
            }
            NirStmt::While { condition, body } => {
                if let Some(plan) =
                    buffer_loop_params(condition, body, scope, catalog, &BTreeSet::new())
                        .filter(|plan| plan.has_store || plan.break_flag.is_some())
                {
                    scalar_helpers::retain_reachable(body, catalog, retained);
                    outline_loop(
                        body,
                        scope,
                        names,
                        helpers,
                        guarded,
                        catalog,
                        structs,
                        plan,
                        break_controls,
                    );
                }
            }
            _ => {}
        }
    }
}

fn outline_loop(
    body: &mut Vec<NirStmt>,
    scope: &Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    structs: &mut Vec<NirStructDef>,
    plan: BufferLoopPlan,
    break_controls: &mut BTreeMap<String, String>,
) {
    let mut suffix = helpers.len();
    let name = loop {
        let name = format!("__nuis_buffer_iteration_{suffix}");
        if names.insert(name.clone()) {
            break name;
        }
        suffix += 1;
    };
    let step = body.pop().expect("validated counted step");
    if let Some(flag) = &plan.break_flag {
        break_controls.insert(name.clone(), flag.clone());
    }
    let aggregate = (plan.carries.len() > 1 || plan.break_flag.is_some())
        .then(|| scalar_carries::state_type(&plan.carries, names, structs));
    let returned = scalar_carries::value(&plan.carries, aggregate.as_ref());
    let effects = std::mem::take(body);
    let mut helper_body = branches::outline_effects(
        plan.normalized_effects.unwrap_or(effects),
        &mut scope.clone(),
        names,
        helpers,
        guarded,
        catalog,
        &plan.mutations,
        structs,
        break_controls,
    );
    helper_body.push(NirStmt::Return(Some(returned)));
    let args = plan
        .params
        .iter()
        .map(|param| {
            if plan.break_flag.as_ref() == Some(&param.name) {
                NirExpr::Int(0)
            } else {
                NirExpr::Var(param.name.clone())
            }
        })
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
        if let Some(flag) = plan.break_flag {
            body.push(NirStmt::If {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(NirExpr::Var(flag)),
                    rhs: Box::new(NirExpr::Int(1)),
                },
                then_body: vec![NirStmt::Break],
                else_body: vec![],
            });
        }
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
