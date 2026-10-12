use super::super::super::super::super::{
    append_authorized as append_prefix, stage_authorized as stage_prefix,
};
use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_private_fields_tests.rs"]
mod tests;

// This certificate is lexical write authority, not extra outward results.
pub(in super::super::super::super) struct Children(Vec<String>);

fn writes(
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    depth: usize,
) -> Option<BTreeSet<String>> {
    fn collect(
        body: &[NirStmt],
        scope: &Scope,
        remaining: &mut usize,
        depth: usize,
        result: &mut BTreeSet<String>,
    ) -> Option<()> {
        *remaining = remaining.checked_sub(body.len())?;
        for statement in body {
            match statement {
                NirStmt::Let { name, .. } if scope.contains_key(name) => {
                    result.insert(name.clone());
                    if result.len() > 3 {
                        return None;
                    }
                }
                NirStmt::Let { .. } | NirStmt::Const { .. } => {}
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } if depth < 8 => {
                    collect(then_body, scope, remaining, depth + 1, result)?;
                    collect(else_body, scope, remaining, depth + 1, result)?;
                }
                _ => return None,
            }
        }
        Some(())
    }
    let mut result = BTreeSet::new();
    let mut remaining = 64;
    collect(yes, scope, &mut remaining, depth + 1, &mut result)?;
    collect(no, scope, &mut remaining, depth + 1, &mut result)?;
    Some(result)
}

pub(in super::super::super::super) fn classify(
    yes: &[NirStmt],
    no: &[NirStmt],
    entering: &Scope,
    staged: &regions::Staging,
    owner: &Target<'_>,
    depth: usize,
) -> Option<Option<Children>> {
    let mut candidates = writes(yes, no, &staged.local, depth)?;
    if candidates.len() < 3 {
        return Some(None);
    }
    if !transport::numeric(staged.local.get(owner.name())?)
        || staged.constants.contains(owner.name())
        || !candidates.remove(owner.name())
        || candidates.len() != 2
    {
        return None;
    }
    for child in &candidates {
        if entering.contains_key(child)
            || !staged.defined.contains(child)
            || staged.constants.contains(child)
            || !transport::numeric(staged.local.get(child)?)
        {
            return None;
        }
    }
    Some(Some(Children(candidates.into_iter().collect())))
}

#[allow(clippy::too_many_arguments)]
pub(in super::super::super::super) fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    suffix: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    children: &Children,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    if children.0.len() != 2 || suffix.len() > 16 {
        return None;
    }
    tree::preflight(yes, no, suffix, depth)?;
    let mut staged = stage_prefix(&[], scope, signatures, budget, depth, Some(owner))?;
    selection(
        condition,
        yes,
        no,
        suffix,
        &mut staged,
        owner,
        children,
        signatures,
        budget,
        depth,
    )?;
    append_body(
        suffix,
        scope,
        &mut staged,
        owner,
        children,
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
fn selection(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    future: &[NirStmt],
    staged: &mut regions::Staging,
    owner: &Target<'_>,
    children: &Children,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<()> {
    if depth >= 8
        || staged.constants.contains(owner.name())
        || children
            .0
            .iter()
            .any(|name| staged.constants.contains(name))
    {
        return None;
    }
    let scope = &staged.local;
    let future = fresh_names(future, scope);
    tree::isolated(yes, scope, &future)?;
    tree::isolated(no, scope, &future)?;
    nested::consume_node(budget)?;
    let (condition, inputs, calls) = predicates::prove(condition, scope, signatures, budget)?;
    let yes = body(yes, scope, owner, children, signatures, budget, depth + 1)?;
    let no = body(no, scope, owner, children, signatures, budget, depth + 1)?;
    let (plan, reads, effects) =
        transport::prepare_fields(condition, yes, no, scope, owner.name(), &children.0)?;
    staged.inputs.extend(
        inputs
            .union(&reads)
            .filter(|n| !staged.defined.contains(*n))
            .cloned(),
    );
    staged.calls.extend(calls.union(&effects).cloned());
    staged
        .statements
        .push(regions::Statement::PrivateFields(Box::new(plan)));
    staged.defined.insert(owner.name().into());
    staged.defined.extend(children.0.iter().cloned());
    Some(())
}

#[allow(clippy::too_many_arguments)]
fn body(
    statements: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    children: &Children,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<regions::Staging> {
    let mut staged = stage_prefix(&[], scope, signatures, budget, depth, Some(owner))?;
    append_body(
        statements,
        scope,
        &mut staged,
        owner,
        children,
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
    children: &Children,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<()> {
    // Walk the original forest once; never copy a continuation into both arms.
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
                children,
                signatures,
                budget,
                depth,
            )?,
            NirStmt::Let { name, .. } if children.0.contains(name) => {
                // Only certified private bindings advance here, never entering neighbours.
                let next = regions::stage(
                    std::slice::from_ref(statement),
                    &staged.local,
                    &staged.constants,
                    signatures,
                    budget,
                )?;
                staged
                    .inputs
                    .extend(next.inputs.difference(&staged.defined).cloned());
                staged.calls.extend(next.calls);
                staged.defined.extend(next.defined);
                staged.local = next.local;
                staged.constants = next.constants;
                staged.statements.extend(next.statements);
            }
            _ => append_prefix(
                statement,
                scope,
                staged,
                signatures,
                budget,
                depth,
                Some(owner),
            )?,
        }
    }
    Some(())
}
