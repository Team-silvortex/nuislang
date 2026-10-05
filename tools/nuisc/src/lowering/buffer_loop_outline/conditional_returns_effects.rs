use super::*;

#[path = "conditional_returns_print_aliases.rs"]
pub(super) mod aliases;
#[path = "conditional_returns_print_values.rs"]
mod print_values;
#[path = "conditional_returns_effect_regions.rs"]
pub(super) mod regions;
#[cfg(test)]
#[path = "conditional_returns_effects_tests.rs"]
mod tests;

pub(super) struct EffectPlan<'a> {
    pure: Plan,
    yes: Vec<print_values::PrintValue<'a>>,
    no: Vec<print_values::PrintValue<'a>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prepare<'a>(
    condition: &NirExpr,
    then_body: &'a [NirStmt],
    else_body: &'a [NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    checked: &BTreeSet<String>,
) -> Option<EffectPlan<'a>> {
    let split = |body: &'a [NirStmt]| {
        let count = body
            .iter()
            .take_while(|stmt| matches!(stmt, NirStmt::Print(_)))
            .count();
        body.split_at(count)
    };
    let (yes, yes_tail) = split(then_body);
    let (no, no_tail) = split(else_body);
    if yes.is_empty() && no.is_empty() {
        return None;
    }
    let mut prepared = Vec::new();
    for (body, prints, tail) in [(then_body, yes, yes_tail), (else_body, no, no_tail)] {
        // Account for the original complete arm before recursive pure typing.
        if !preflight(body, prints.len()) {
            return None;
        }
        let values = prints
            .iter()
            .map(|stmt| match stmt {
                NirStmt::Print(value) => value,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        if !suffix::reserve_prefix(tail, &values) {
            return None;
        }
        prepared.push(
            values
                .into_iter()
                .map(|value| print_values::prepare(value, scope, catalog, layouts))
                .collect::<Option<Vec<_>>>()?,
        );
    }
    // Effects never enter the pure helper catalog or confer source-exit rights.
    let pure = super::prepare(
        condition, yes_tail, no_tail, result, scope, catalog, layouts, checked,
    )?;
    let no = prepared.pop()?;
    let yes = prepared.pop()?;
    Some(EffectPlan { pure, yes, no })
}

pub(super) fn preflight(body: &[NirStmt], prints: usize) -> bool {
    let mut pending = vec![(body, true)];
    let mut statements = 32usize;
    let mut expressions = Vec::new();
    while let Some((body, outer)) = pending.pop() {
        if body.len() > statements {
            return false;
        }
        statements -= body.len();
        for (index, stmt) in body.iter().enumerate() {
            match stmt {
                NirStmt::Print(value) if outer && index < prints => {
                    expressions.push((value, 0, false))
                }
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Return(Some(value)) => expressions.push((value, 0, true)),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push((condition, 0, true));
                    pending.extend([(then_body.as_slice(), false), (else_body.as_slice(), false)]);
                }
                _ => return false,
            }
        }
    }
    conditional_values::prefix::computed_expression_roots(expressions)
}

pub(super) fn install(
    mut plan: EffectPlan<'_>,
    result: &NirTypeRef,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    definitions: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let gate = branches::fresh_name("__nuis_effect_return_gate", bindings);
    let mut output = vec![NirStmt::Let {
        name: gate.clone(),
        ty: Some(scalar_type("bool")),
        value: plan.pure.condition,
    }];
    for (prints, selected) in [(plan.yes, true), (plan.no, false)] {
        for print in prints {
            let condition = if selected {
                NirExpr::Var(gate.clone())
            } else {
                NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(NirExpr::Var(gate.clone())),
                    rhs: Box::new(NirExpr::Bool(false)),
                }
            };
            let value =
                print_values::install(print, &condition, names, bindings, helpers, &mut output);
            output.push(NirStmt::If {
                condition,
                then_body: vec![NirStmt::Print(value)],
                else_body: vec![],
            });
        }
    }
    // Parent effects use the completed original gate, then the existing pure
    // tail/readiness route uses the same value without replaying the entry.
    plan.pure.condition = NirExpr::Var(gate);
    output.extend(super::install(
        plan.pure,
        result,
        names,
        bindings,
        helpers,
        definitions,
    ));
    output
}
