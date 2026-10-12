use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_mixed_sequence_tests.rs"]
mod tests;

#[path = "effectful_selections_return_mixed_sequence_tree.rs"]
mod tree;

#[path = "effectful_selections_return_private_fields.rs"]
pub(super) mod fields;

pub(super) fn continued(yes: &[NirStmt], no: &[NirStmt]) -> Option<bool> {
    tree::continued(yes, no)
}

fn shape(body: &[NirStmt], scope: &Scope, future: &BTreeSet<String>) -> Option<()> {
    tree::isolated(body, scope, future)
}

#[allow(clippy::too_many_arguments)]
fn selection(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    future: &[NirStmt],
    staged: &mut regions::Staging,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<()> {
    if depth >= 8 || staged.constants.contains(owner.name()) || staged.constants.contains(&child.0)
    {
        return None;
    }
    let scope = &staged.local;
    let future = fresh_names(future, scope);
    shape(yes, scope, &future)?;
    shape(no, scope, &future)?;
    nested::consume_node(budget)?;
    let (condition, inputs, calls) = predicates::prove(condition, scope, signatures, budget)?;
    let yes = body(yes, scope, owner, child, signatures, budget, depth + 1)?;
    let no = body(no, scope, owner, child, signatures, budget, depth + 1)?;
    let (plan, reads, effects) =
        transport::prepare(condition, yes, no, scope, owner.name(), &child.0)?;
    staged.inputs.extend(
        inputs
            .union(&reads)
            .filter(|n| !staged.defined.contains(*n))
            .cloned(),
    );
    staged.calls.extend(calls.union(&effects).cloned());
    staged
        .statements
        .push(regions::Statement::PrivatePair(Box::new(plan)));
    // Joined versions include selected writes AND explicitly captured retention.
    // This is internal transport, never evidence authorizing the outward owner.
    staged
        .defined
        .extend([owner.name().into(), child.0.clone()]);
    Some(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    suffix: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    if (suffix.is_empty()
        && !transport::numeric(scope.get(&child.0)?)
        && !numeric_owner(scope.get(owner.name())?))
        || suffix.len() > 16
    {
        return None;
    }
    tree::preflight(yes, no, suffix, depth)?;
    let mut staged = super::super::super::super::stage_authorized(
        &[],
        scope,
        signatures,
        budget,
        depth,
        Some(owner),
    )?;
    selection(
        condition,
        yes,
        no,
        suffix,
        &mut staged,
        owner,
        child,
        signatures,
        budget,
        depth,
    )?;
    append_body(
        suffix,
        scope,
        &mut staged,
        owner,
        child,
        signatures,
        budget,
        depth,
    )?;
    if !staged.defined.contains(owner.name()) {
        staged.inputs.insert(owner.name().into());
    }
    Some(nested::Proof {
        value: SelectedValue::Region(staged.statements, NirExpr::Var(owner.name().into())),
        name: owner.name().into(),
        ty: scope.get(owner.name())?.clone(),
        inputs: staged.inputs,
        calls: staged.calls,
    })
}

#[allow(clippy::too_many_arguments)]
fn body(
    statements: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<regions::Staging> {
    let mut staged = super::super::super::super::stage_authorized(
        &[],
        scope,
        signatures,
        budget,
        depth,
        Some(owner),
    )?;
    append_body(
        statements,
        scope,
        &mut staged,
        owner,
        child,
        signatures,
        budget,
        depth,
    )?;
    Some(staged)
}

#[allow(clippy::too_many_arguments)]
fn append_body(
    statements: &[NirStmt],
    scope: &Scope,
    staged: &mut regions::Staging,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<()> {
    // Each original selection and continuation statement is proved once.
    // No arm receives a copy of the remaining sequence.
    for (index, statement) in statements.iter().enumerate() {
        match statement {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => selection(
                condition,
                then_body,
                else_body,
                &statements[index + 1..],
                staged,
                owner,
                child,
                signatures,
                budget,
                depth,
            )?,
            _ => super::super::append(
                statement, scope, staged, owner, child, signatures, budget, depth,
            )?,
        }
    }
    Some(())
}
