use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_update_tests.rs"]
mod tests;

#[allow(clippy::too_many_arguments)]
pub(super) fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    staged: &regions::Staging,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    if depth >= 8 {
        return None;
    }
    let target = |body: &[NirStmt]| match body {
        [] => Some(None),
        [NirStmt::Let { name, .. }] => Some(Some(name.clone())),
        _ => None,
    };
    let yes_target = target(yes)?;
    let no_target = target(no)?;
    let name = yes_target.as_ref().or(no_target.as_ref())?;
    if yes_target.as_ref().is_some_and(|target| target != name)
        || no_target.as_ref().is_some_and(|target| target != name)
        || !staged.defined.contains(name)
        || staged.constants.contains(name)
        || staged.local.get(name)? != &scalar_type("bool")
    {
        return None;
    }
    nested::consume_node(budget)?;
    let (condition, reads, effects) =
        predicates::prove(condition, &staged.local, signatures, budget)?;
    let mut arm = |body: &[NirStmt]| {
        if let [statement] = body {
            nested::leaf(statement, &staged.local, signatures, budget)
        } else {
            // Retention is a transport outcome, not invented original source work.
            Some(nested::Proof {
                value: SelectedValue::Expression(NirExpr::Var(name.clone())),
                name: name.clone(),
                ty: scalar_type("bool"),
                inputs: BTreeSet::from([name.clone()]),
                calls: BTreeSet::new(),
            })
        }
    };
    let yes = arm(yes)?;
    let no = arm(no)?;
    nested::join(condition, reads, effects, yes, no, &staged.local)
}
