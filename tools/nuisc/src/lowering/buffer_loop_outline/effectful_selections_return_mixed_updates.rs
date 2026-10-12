use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_mixed_update_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_return_i32_pair_tests.rs"]
mod i32_tests;

#[cfg(test)]
#[path = "effectful_selections_return_i64_pair_tests.rs"]
mod i64_tests;

#[cfg(test)]
#[path = "effectful_selections_return_float_pair_tests.rs"]
mod float_tests;

#[path = "effectful_selections_return_mixed_continuations.rs"]
mod continuations;
pub(super) use continuations::prove as continue_flat;

// This token carries private write authority, never an additional outward result.
pub(super) struct Child(String);

pub(super) fn classify(
    yes: &[NirStmt],
    no: &[NirStmt],
    entering: &Scope,
    staged: &regions::Staging,
    owner: &Target<'_>,
    depth: usize,
) -> Option<Option<Child>> {
    let mut eligible = eligible(staged, Some(owner));
    eligible.extend(
        staged
            .local
            .iter()
            .filter(|(name, ty)| {
                transport::numeric(ty)
                    && staged.defined.contains(*name)
                    && !staged.constants.contains(*name)
            })
            .map(|(name, _)| name.clone()),
    );
    let mut writes = selection_writes(yes, no, &eligible, depth)?;
    if writes.len() < 2 {
        return Some(None);
    }
    if !writes.remove(owner.name()) || writes.len() != 1 {
        return None;
    }
    let child = writes.pop_first()?;
    if entering.contains_key(&child)
        || !staged.defined.contains(&child)
        || staged.constants.contains(&child)
        || !(staged.local.get(&child)? == &scalar_type("bool")
            || transport::numeric(staged.local.get(&child)?))
    {
        return None;
    }
    // Numeric owners admit private bools and the separately proved numeric pair.
    if numeric_owner(staged.local.get(owner.name())?)
        && staged.local.get(&child)? != &scalar_type("bool")
        && !transport::numeric_pair(staged.local.get(owner.name())?, staged.local.get(&child)?)
    {
        return None;
    }
    Some(Some(Child(child)))
}

fn selection_writes(
    yes: &[NirStmt],
    no: &[NirStmt],
    eligible: &BTreeSet<String>,
    depth: usize,
) -> Option<BTreeSet<String>> {
    fn collect(
        body: &[NirStmt],
        eligible: &BTreeSet<String>,
        remaining: &mut usize,
        depth: usize,
        writes: &mut BTreeSet<String>,
    ) -> Option<()> {
        *remaining = remaining.checked_sub(body.len())?;
        for statement in body {
            match statement {
                NirStmt::Let { name, .. } if eligible.contains(name) => {
                    writes.insert(name.clone());
                    if writes.len() > 2 {
                        return None;
                    }
                }
                NirStmt::Let { .. } | NirStmt::Const { .. } => {}
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } if depth < 8 => {
                    collect(then_body, eligible, remaining, depth + 1, writes)?;
                    collect(else_body, eligible, remaining, depth + 1, writes)?;
                }
                _ => return None,
            }
        }
        Some(())
    }
    let mut writes = BTreeSet::new();
    let mut remaining = 64;
    collect(yes, eligible, &mut remaining, depth + 1, &mut writes)?;
    collect(no, eligible, &mut remaining, depth + 1, &mut writes)?;
    Some(writes)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    if transport::numeric(scope.get(&child.0)?) || numeric_owner(scope.get(owner.name())?) {
        return continue_flat(
            condition,
            yes,
            no,
            &[],
            scope,
            owner,
            child,
            signatures,
            budget,
            depth,
        );
    }
    if depth >= 8 {
        return None;
    }
    nested::consume_node(budget)?;
    let (condition, inputs, calls) = predicates::prove(condition, scope, signatures, budget)?;
    let yes = arm(yes, scope, owner, child, signatures, budget, depth + 1)?;
    let no = arm(no, scope, owner, child, signatures, budget, depth + 1)?;
    nested::join(condition, inputs, calls, yes, no, scope)
}

#[allow(clippy::too_many_arguments)]
fn append(
    statement: &NirStmt,
    scope: &Scope,
    staged: &mut regions::Staging,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<()> {
    if matches!(statement, NirStmt::Let { name, .. } if name == &child.0)
        && transport::numeric(scope.get(&child.0)?)
    {
        // Only the classified prefix-private child gets this scalar write.
        // Do not manufacture a bool Target token or authorize another binding.
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
        return Some(());
    }
    let name = match statement {
        NirStmt::Let { name, .. } if name == &child.0 => child.0.as_str(),
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            let mut eligible = eligible(staged, Some(owner));
            eligible.insert(child.0.clone());
            let writes = selection_writes(then_body, else_body, &eligible, depth)?;
            // A prefix needs both versions for any later child reads. Do not
            // silently discard the child by using the terminal-only join there.
            if writes.len() > 1 {
                return None;
            }
            if writes.contains(&child.0) {
                child.0.as_str()
            } else {
                owner.name()
            }
        }
        _ => owner.name(),
    };
    super::super::append_authorized(
        statement,
        scope,
        staged,
        signatures,
        budget,
        depth,
        Some(&Target(name)),
    )
}

#[allow(clippy::too_many_arguments)]
fn arm(
    body: &[NirStmt],
    scope: &Scope,
    owner: &Target<'_>,
    child: &Child,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    let mut staged =
        super::super::stage_authorized(&[], scope, signatures, budget, depth, Some(owner))?;
    let mut value = SelectedValue::Expression(NirExpr::Var(owner.name().into()));
    let mut reads = BTreeSet::from([owner.name().into()]);
    let mut calls = BTreeSet::new();
    if let Some((last, prefix)) = body.split_last() {
        if prefix.len() > 16 {
            return None;
        }
        for (index, statement) in prefix.iter().enumerate() {
            if let NirStmt::If {
                condition,
                then_body,
                else_body,
            } = statement
            {
                let mut eligible = eligible(&staged, Some(owner));
                eligible.insert(child.0.clone());
                let writes = selection_writes(then_body, else_body, &eligible, depth)?;
                if writes.len() == 2 && writes.contains(owner.name()) && writes.contains(&child.0) {
                    let proof = continue_flat(
                        condition,
                        then_body,
                        else_body,
                        &body[index + 1..],
                        &staged.local,
                        owner,
                        child,
                        signatures,
                        budget,
                        depth,
                    )?;
                    return Some(super::finish(staged, proof));
                }
            }
            append(
                statement,
                scope,
                &mut staged,
                owner,
                child,
                signatures,
                budget,
                depth,
            )?;
        }
        let terminal = match last {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let mut eligible = eligible(&staged, Some(owner));
                eligible.insert(child.0.clone());
                let writes = selection_writes(then_body, else_body, &eligible, depth)?;
                if writes.contains(owner.name()) {
                    if writes.iter().any(|n| n != owner.name() && n != &child.0) {
                        return None;
                    }
                    Some(prove(
                        condition,
                        then_body,
                        else_body,
                        &staged.local,
                        owner,
                        child,
                        signatures,
                        budget,
                        depth,
                    )?)
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(proof) = terminal {
            value = proof.value;
            reads = proof.inputs;
            calls = proof.calls;
        } else {
            append(
                last,
                scope,
                &mut staged,
                owner,
                child,
                signatures,
                budget,
                depth,
            )?;
        }
    }
    let mut inputs = staged.inputs;
    inputs.extend(reads.difference(&staged.defined).cloned());
    calls.extend(staged.calls);
    if !staged.statements.is_empty() {
        value = match value {
            SelectedValue::Nested(plan) => SelectedValue::StagedNested(staged.statements, plan),
            SelectedValue::Expression(value) => SelectedValue::Region(staged.statements, value),
            _ => unreachable!(),
        };
    }
    Some(nested::Proof {
        value,
        name: owner.name().into(),
        ty: scope.get(owner.name())?.clone(),
        inputs,
        calls,
    })
}
