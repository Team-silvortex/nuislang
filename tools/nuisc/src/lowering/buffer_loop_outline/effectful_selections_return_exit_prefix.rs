use super::*;

pub(super) fn prove(
    exit: &Exit<'_>,
    fields: &updates::PrivateFields,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    // Authority comes from the original continuing arms, never exit-local work.
    // This staging is emitted only inside the selected early-return plan.
    let mut staged = stage_authorized(&[], scope, signatures, budget, depth + 2, None)?;
    fields.tail(
        exit.prefix,
        scope,
        &mut staged,
        signatures,
        budget,
        depth + 2,
    )?;
    nested::consume_node(budget)?;
    let (value, reads, calls) = predicates::prove(exit.value, &staged.local, signatures, budget)?;
    staged
        .inputs
        .extend(reads.difference(&staged.defined).cloned());
    staged.calls.extend(calls);
    Some(ReturnedValue {
        value: if staged.statements.is_empty() {
            SelectedValue::Predicate(value)
        } else {
            SelectedValue::StagedPredicate(staged.statements, value)
        },
        inputs: staged.inputs,
        calls: staged.calls,
    })
}
