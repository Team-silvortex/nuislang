use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_early_fields_tests.rs"]
mod tests;

pub(super) fn candidate(body: &[NirStmt]) -> bool {
    body.iter().any(|s| {
        matches!(s, NirStmt::If { then_body, else_body, .. }
        if then_body.iter().chain(else_body).any(|s| matches!(s, NirStmt::Return(_))))
    })
}

pub(super) fn preflight(body: &[NirStmt], remaining: &mut usize, depth: usize) -> Option<()> {
    if depth >= 8 || body.len() > 16 {
        return None;
    }
    *remaining = remaining.checked_sub(body.len())?;
    for (index, statement) in body.iter().enumerate() {
        match statement {
            NirStmt::Let { .. } | NirStmt::Const { .. } => {}
            NirStmt::Return(Some(_)) if index + 1 == body.len() => {}
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                preflight(then_body, remaining, depth + 1)?;
                preflight(else_body, remaining, depth + 1)?;
            }
            _ => return None,
        }
    }
    Some(())
}

pub(super) fn prove(
    body: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    let mut remaining = 64;
    preflight(body, &mut remaining, depth)?;
    let (result, prefix) = returned(body)?;
    let mut staged = stage(&[], scope, signatures, budget, depth)?;
    let mut owner = None;
    for (index, statement) in prefix.iter().enumerate() {
        if let NirStmt::If {
            condition,
            then_body,
            else_body,
        } = statement
        {
            if let [NirStmt::Return(Some(value))] = then_body.as_slice() {
                if !else_body.is_empty() {
                    return None;
                }
                let owner = owner.as_deref()?;
                nested::consume_node(budget)?;
                let (condition, reads, effects) =
                    predicates::prove(condition, &staged.local, signatures, budget)?;
                nested::consume_node(budget)?;
                let (early, early_reads, early_effects) =
                    predicates::prove(value, &staged.local, signatures, budget)?;
                let tail = updates::continue_owner(
                    &prefix[index + 1..],
                    &staged.local,
                    owner,
                    signatures,
                    budget,
                    depth + 1,
                )?;
                nested::consume_node(budget)?;
                let (result, result_reads, result_effects) =
                    predicates::prove(result, &tail.local, signatures, budget)?;
                let mut continued_reads = tail.inputs;
                continued_reads.extend(result_reads.difference(&tail.defined).cloned());
                let mut continued_effects = tail.calls;
                continued_effects.extend(result_effects);
                let all_reads = reads
                    .union(&early_reads)
                    .cloned()
                    .chain(continued_reads.iter().cloned())
                    .collect::<BTreeSet<_>>();
                staged
                    .inputs
                    .extend(all_reads.difference(&staged.defined).cloned());
                staged.calls.extend(effects);
                staged.calls.extend(early_effects.iter().cloned());
                staged.calls.extend(continued_effects.iter().cloned());
                let plan = super::super::plan(
                    condition,
                    ReturnedValue {
                        value: SelectedValue::Predicate(early),
                        inputs: early_reads,
                        calls: early_effects,
                    },
                    ReturnedValue {
                        value: SelectedValue::StagedPredicate(tail.statements, result),
                        inputs: continued_reads,
                        calls: continued_effects,
                    },
                    &staged.local,
                )?;
                // The continuation is installed once in the non-exited bool arm.
                // Numeric field records never carry function-level exit authority.
                return Some(ReturnedValue {
                    value: SelectedValue::StagedNested(staged.statements, Box::new(plan)),
                    inputs: staged.inputs,
                    calls: staged.calls,
                });
            }
            if owner.is_some() {
                return None;
            }
            owner = Some(updates::private_field_stage(
                condition,
                then_body,
                else_body,
                &prefix[index + 1..],
                scope,
                &mut staged,
                signatures,
                budget,
                depth,
            )?);
        } else {
            append_authorized(
                statement,
                scope,
                &mut staged,
                signatures,
                budget,
                depth,
                None,
            )?;
        }
    }
    None
}
