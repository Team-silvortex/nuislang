use super::*;

#[cfg(test)]
#[path = "conditional_returns_alias_tests.rs"]
mod alias_tests;
#[cfg(test)]
#[path = "conditional_returns_comparison_tests.rs"]
mod comparison_tests;
#[cfg(test)]
#[path = "conditional_returns_continuation_tests.rs"]
mod continuation_tests;
#[cfg(test)]
#[path = "conditional_returns_inline_tests.rs"]
mod inline_tests;
#[cfg(test)]
#[path = "conditional_returns_fallthrough_tests.rs"]
mod tests;

pub(super) struct Arm {
    pub(super) value: prefix::Arm,
    pub(super) ready: NirExpr,
    pub(super) has_return: bool,
}

pub(super) fn prepare(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Arm> {
    let inputs = terminal::partial_inputs(body, result, scope, catalog, layouts)?;
    let aliases = predicates::Values::from_parent(scope);
    let mut leaves = (false, false);
    let ready = readiness(body, &aliases, &mut leaves)?;
    if !leaves.1 {
        return None;
    }
    let mut body = body.to_vec();
    seed_fallthroughs(&mut body, result);
    Some(Arm {
        value: prefix::Arm { body, inputs },
        ready,
        has_return: leaves.0,
    })
}

fn readiness(
    body: &[NirStmt],
    aliases: &predicates::Values,
    leaves: &mut (bool, bool),
) -> Option<NirExpr> {
    // Preflight and type validation already bound this tree and forbid rebinding.
    // Flatten only proven total values, never replay calls or checked work.
    let mut aliases = aliases.clone();
    for stmt in body {
        let (name, value) = match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => (name, value),
            _ => break,
        };
        aliases.bind(name, value);
    }
    Some(match body.last() {
        None | Some(NirStmt::Let { .. } | NirStmt::Const { .. }) => {
            leaves.1 = true;
            NirExpr::Bool(false)
        }
        Some(NirStmt::Return(Some(_))) => {
            leaves.0 = true;
            NirExpr::Bool(true)
        }
        Some(NirStmt::If {
            condition,
            then_body,
            else_body,
        }) => select(
            aliases.boolean(condition)?,
            readiness(then_body, &aliases, leaves)?,
            readiness(else_body, &aliases, leaves)?,
        ),
        _ => unreachable!("validated partial return tree"),
    })
}

pub(super) fn select(condition: NirExpr, yes: NirExpr, no: NirExpr) -> NirExpr {
    match (&yes, &no) {
        (NirExpr::Bool(true), NirExpr::Bool(false)) => condition,
        (NirExpr::Bool(false), NirExpr::Bool(true)) => {
            binary(NirBinaryOp::Eq, condition, NirExpr::Bool(false))
        }
        (NirExpr::Bool(true), NirExpr::Bool(true)) => NirExpr::Bool(true),
        (NirExpr::Bool(false), NirExpr::Bool(false)) => NirExpr::Bool(false),
        _ => binary(
            NirBinaryOp::Or,
            binary(NirBinaryOp::And, condition.clone(), yes),
            binary(
                NirBinaryOp::And,
                binary(NirBinaryOp::Eq, condition, NirExpr::Bool(false)),
                no,
            ),
        ),
    }
}

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn seed_fallthroughs(body: &mut Vec<NirStmt>, result: &NirTypeRef) {
    match body.last_mut() {
        Some(NirStmt::If {
            then_body,
            else_body,
            ..
        }) => {
            seed_fallthroughs(then_body, result);
            seed_fallthroughs(else_body, result);
        }
        Some(NirStmt::Return(Some(_))) => {}
        None | Some(NirStmt::Let { .. } | NirStmt::Const { .. }) => {
            // The seed is not an exit signal. Only the parent's independent ready
            // predicate can select a return; a real return may itself be false/zero.
            body.push(NirStmt::Return(Some(if result == &scalar_type("bool") {
                NirExpr::Bool(false)
            } else {
                NirExpr::Int(0)
            })));
        }
        _ => unreachable!("validated partial return leaf"),
    }
}
