use super::*;

#[cfg(test)]
#[path = "effectful_selections_tests.rs"]
mod tests;

#[path = "effectful_selections_nested.rs"]
mod nested;

#[path = "effectful_selections_regions.rs"]
mod regions;

#[path = "effectful_selections_ordered.rs"]
mod ordered;

#[path = "effectful_selections_predicates.rs"]
mod predicates;

type Signatures = BTreeMap<String, (Vec<NirTypeRef>, NirTypeRef)>;

// Signature discovery is not body/effect admission. Generated guards retain
// original calls; verification and backend capability checks still own admission.
pub(super) fn outline(module: &mut NirModule, names: &mut BTreeSet<String>) -> BTreeSet<String> {
    let signatures = module
        .functions
        .iter()
        .filter_map(|f| {
            let result = f.return_type.as_ref()?;
            (!f.is_async
                && f.generic_params.is_empty()
                && f.where_bounds.is_empty()
                && scalar(result)
                && f.params.iter().all(|p| scalar(&p.ty)))
            .then(|| {
                (
                    f.name.clone(),
                    (
                        f.params.iter().map(|p| p.ty.clone()).collect(),
                        result.clone(),
                    ),
                )
            })
        })
        .collect::<Signatures>();
    let pure = collect_pure_helper_functions(module);
    let mut helpers = Vec::new();
    for function in &mut module.functions {
        if function.is_async
            || !function.generic_params.is_empty()
            || !function.where_bounds.is_empty()
        {
            continue;
        }
        let mut scope = function
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect::<Scope>();
        let mut bindings = scope.keys().cloned().collect();
        let mut constants = BTreeSet::new();
        branches::collect_bindings(&function.body, &mut bindings);
        let mut output = Vec::new();
        for stmt in std::mem::take(&mut function.body) {
            if let NirStmt::If {
                condition,
                then_body,
                else_body,
            } = &stmt
            {
                let plan = if regions::writes_logical_constants(then_body, else_body, &constants) {
                    None
                } else {
                    nested::prepare(condition, then_body, else_body, &scope, &signatures, &pure)
                        .or_else(|| {
                            prepare(condition, then_body, else_body, &scope, &signatures, &pure)
                        })
                };
                if let Some(plan) = plan {
                    let constant = plan
                        .destination
                        .as_ref()
                        .is_some_and(|(_, constant)| *constant);
                    let (generated, name, ty) = install(plan, names, &mut bindings, &mut helpers);
                    if constant {
                        constants.insert(name.clone());
                    }
                    scope.insert(name, ty);
                    output.extend(generated);
                    continue;
                }
            }
            match &stmt {
                NirStmt::Let { name, ty, value } => {
                    let inferred = ty
                        .clone()
                        .or_else(|| inspect(value, &scope, &signatures).map(|p| p.0));
                    scope.remove(name);
                    if let Some(ty) = inferred {
                        scope.insert(name.clone(), ty);
                    }
                }
                NirStmt::Const { name, ty, .. } => {
                    constants.insert(name.clone());
                    scope.insert(name.clone(), ty.clone());
                }
                _ => {}
            }
            output.push(stmt);
        }
        function.body = output;
    }
    let generated = helpers.iter().map(|f| f.name.clone()).collect();
    module.functions.extend(helpers);
    generated
}

fn scalar(ty: &NirTypeRef) -> bool {
    ty == &scalar_type(&ty.name)
        && matches!(ty.name.as_str(), "bool" | "i32" | "i64" | "f32" | "f64")
}

struct Plan {
    condition: predicates::Predicate,
    yes: SelectedValue,
    no: SelectedValue,
    destination: Option<(String, bool)>,
    ty: NirTypeRef,
    params: Vec<NirParam>,
}

enum SelectedValue {
    Expression(NirExpr),
    Nested(Box<Plan>),
    Region(Vec<regions::Statement>, NirExpr),
    Ordered(
        Vec<(Vec<regions::Statement>, Box<Plan>)>,
        Vec<regions::Statement>,
        NirExpr,
    ),
}

fn arm(body: &[NirStmt]) -> Option<(&NirExpr, Option<(&str, Option<&NirTypeRef>, bool)>)> {
    match body {
        [NirStmt::Let { name, ty, value }] => Some((value, Some((name, ty.as_ref(), false)))),
        [NirStmt::Const { name, ty, value }] => Some((value, Some((name, Some(ty), true)))),
        [NirStmt::Return(Some(value))] => Some((value, None)),
        _ => None,
    }
}

fn prepare(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    pure: &BTreeSet<String>,
) -> Option<Plan> {
    if yes.is_empty() || no.is_empty() {
        let live = if yes.is_empty() { no } else { yes };
        let [NirStmt::Let { name, .. }] = live else {
            return None;
        };
        let old_ty = scope.get(name)?;
        if !scalar(old_ty) {
            return None;
        }
        // Retain the pre-branch binding, not the inactive guard's neutral zero.
        // Fresh branch-local names and one-sided returns/constants stay excluded.
        let retained = [NirStmt::Let {
            name: name.clone(),
            ty: Some(old_ty.clone()),
            value: NirExpr::Var(name.clone()),
        }];
        let (yes, no) = if yes.is_empty() {
            (retained.as_slice(), no)
        } else {
            (yes, retained.as_slice())
        };
        return prepare(condition, yes, no, scope, signatures, pure);
    }
    let (yes, yes_destination) = arm(yes)?;
    let (no, no_destination) = arm(no)?;
    let (yes_ty, yes_inputs, yes_calls) = inspect(yes, scope, signatures)?;
    let (no_ty, no_inputs, no_calls) = inspect(no, scope, signatures)?;
    // Preserve pure-selection ownership and reject shapes beyond this proof.
    if yes_ty != no_ty
        || !yes_calls
            .iter()
            .chain(&no_calls)
            .any(|name| !pure.contains(name))
    {
        return None;
    }
    let destination = match (yes_destination, no_destination) {
        (Some((a, a_ty, a_const)), Some((b, b_ty, b_const)))
            if a == b
                && a_const == b_const
                && a_ty.is_none_or(|ty| ty == &yes_ty)
                && b_ty.is_none_or(|ty| ty == &yes_ty) =>
        {
            Some((a.to_owned(), a_const))
        }
        (None, None) => None,
        _ => return None,
    };
    if inspect(condition, scope, signatures)?.0 != scalar_type("bool") {
        return None;
    }
    let inputs = yes_inputs
        .union(&no_inputs)
        .cloned()
        .collect::<BTreeSet<_>>();
    if inputs.len() > 31 {
        return None;
    }
    let params = inputs
        .into_iter()
        .map(|name| {
            let ty = scope.get(&name)?.clone();
            scalar(&ty).then_some(NirParam { name, ty })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Plan {
        condition: predicates::Predicate::Scalar(condition.clone()),
        yes: SelectedValue::Expression(yes.clone()),
        no: SelectedValue::Expression(no.clone()),
        destination,
        ty: yes_ty,
        params,
    })
}

fn install(
    plan: Plan,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
) -> (Vec<NirStmt>, String, NirTypeRef) {
    let gate = branches::fresh_name("__nuis_effect_call_gate", bindings);
    let condition = predicates::install(plan.condition, names, bindings, helpers);
    let mut output = vec![NirStmt::Let {
        name: gate.clone(),
        ty: Some(scalar_type("bool")),
        value: condition,
    }];
    let mut values = Vec::new();
    for (selected, value) in [(true, plan.yes), (false, plan.no)] {
        let name = branches::fresh_name("__nuis_effect_call_arm", names);
        let mut params = vec![NirParam {
            name: gate.clone(),
            ty: scalar_type("bool"),
        }];
        params.extend(plan.params.clone());
        let args = params
            .iter()
            .map(|p| NirExpr::Var(p.name.clone()))
            .collect();
        let mut body = vec![NirStmt::If {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Ne,
                lhs: Box::new(NirExpr::Var(gate.clone())),
                rhs: Box::new(NirExpr::Bool(selected)),
            },
            then_body: vec![NirStmt::Return(Some(control_values::zero_value(
                &plan.ty,
                &control_values::TypedLayouts::default(),
            )))],
            else_body: vec![],
        }];
        // Descendant predicates are emitted only after this ancestor guard.
        let value = match value {
            SelectedValue::Expression(value) => value,
            SelectedValue::Region(statements, value) => {
                body.extend(regions::install(statements, names, bindings, helpers));
                value
            }
            SelectedValue::Ordered(children, suffix, result) => {
                for (stage, child) in children {
                    body.extend(regions::install(stage, names, bindings, helpers));
                    let (statements, _, _) = install(*child, names, bindings, helpers);
                    body.extend(statements);
                }
                body.extend(regions::install(suffix, names, bindings, helpers));
                result
            }
            SelectedValue::Nested(child) => {
                let (statements, result, _) = install(*child, names, bindings, helpers);
                body.extend(statements);
                NirExpr::Var(result)
            }
        };
        body.push(NirStmt::Return(Some(value)));
        let mut function = helper(name.clone(), params, body);
        function.return_type = Some(plan.ty.clone());
        helpers.push(function);
        let temporary = branches::fresh_name("__nuis_effect_call_value", bindings);
        output.push(NirStmt::Let {
            name: temporary.clone(),
            ty: Some(plan.ty.clone()),
            value: NirExpr::Call { callee: name, args },
        });
        values.push(temporary);
    }
    let returns = plan.destination.is_none();
    let (destination, constant) = plan.destination.unwrap_or_else(|| {
        (
            branches::fresh_name("__nuis_effect_call_result", bindings),
            false,
        )
    });
    let bind = |value| {
        if constant {
            NirStmt::Const {
                name: destination.clone(),
                ty: plan.ty.clone(),
                value,
            }
        } else {
            NirStmt::Let {
                name: destination.clone(),
                ty: Some(plan.ty.clone()),
                value,
            }
        }
    };
    output.push(NirStmt::If {
        condition: NirExpr::Var(gate),
        then_body: vec![bind(NirExpr::Var(values[0].clone()))],
        else_body: vec![bind(NirExpr::Var(values[1].clone()))],
    });
    if returns {
        output.push(NirStmt::Return(Some(NirExpr::Var(destination.clone()))));
    }
    (output, destination, plan.ty)
}

fn inspect(
    expr: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
) -> Option<(NirTypeRef, BTreeSet<String>, BTreeSet<String>)> {
    let mut inputs = BTreeSet::new();
    let mut calls = BTreeSet::new();
    let mut budget = 256;
    let ty = expression(
        expr,
        scope,
        signatures,
        &mut inputs,
        &mut calls,
        &mut budget,
        0,
    )?;
    Some((ty, inputs, calls))
}

#[allow(clippy::too_many_arguments)]
fn expression(
    expr: &NirExpr,
    scope: &Scope,
    signatures: &Signatures,
    inputs: &mut BTreeSet<String>,
    calls: &mut BTreeSet<String>,
    budget: &mut usize,
    depth: usize,
) -> Option<NirTypeRef> {
    if depth >= 32 || *budget == 0 {
        return None;
    }
    *budget -= 1;
    let mut next = |expr| expression(expr, scope, signatures, inputs, calls, budget, depth + 1);
    match expr {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::F32(_) => Some(scalar_type("f32")),
        NirExpr::F64(_) => Some(scalar_type("f64")),
        NirExpr::Var(name) => {
            let ty = scope.get(name)?;
            if !scalar(ty) {
                return None;
            }
            inputs.insert(name.clone());
            Some(ty.clone())
        }
        NirExpr::Call { callee, args } => {
            let (params, result) = signatures.get(callee)?;
            if params.len() != args.len() {
                return None;
            }
            for (arg, param) in args.iter().zip(params) {
                if next(arg)? != *param {
                    return None;
                }
            }
            calls.insert(callee.clone());
            Some(result.clone())
        }
        NirExpr::Binary { op, lhs, rhs } if !matches!(op, NirBinaryOp::And | NirBinaryOp::Or) => {
            control_values::binary_type(*op, next(lhs)?, next(rhs)?)
        }
        NirExpr::CastI64ToI32(v) => (next(v)? == scalar_type("i64")).then(|| scalar_type("i32")),
        NirExpr::CastI32ToI64(v) => (next(v)? == scalar_type("i32")).then(|| scalar_type("i64")),
        _ => None,
    }
}
