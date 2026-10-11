use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_update_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_return_nested_update_tests.rs"]
mod nested_tests;

#[cfg(test)]
#[path = "effectful_selections_return_ordered_update_tests.rs"]
mod ordered_tests;

#[cfg(test)]
#[path = "effectful_selections_return_retention_update_tests.rs"]
mod retention_tests;

#[cfg(test)]
#[path = "effectful_selections_return_trailing_update_tests.rs"]
mod trailing_tests;

#[cfg(test)]
#[path = "effectful_selections_return_terminal_child_update_tests.rs"]
mod terminal_child_tests;

#[path = "effectful_selections_return_retention_targets.rs"]
mod retention_targets;

// Only this owner can delegate an already proven, unsealed prefix-owned bool.
pub(super) struct Target<'a>(&'a str);

impl Target<'_> {
    pub(super) fn name(&self) -> &str {
        self.0
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    staged: &regions::Staging,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
    authorized: Option<&Target<'_>>,
) -> Option<nested::Proof> {
    if depth >= 8 {
        return None;
    }
    // Prefer eligible terminal exports. Private trailing bindings are not exports.
    // Retention-only tails can use earlier writes,
    // but neither scan manufactures authority or replaces full source proof.
    let eligible = eligible(staged, authorized);
    let mut remaining = 64;
    let yes_target = target(yes, &eligible, &mut remaining, depth + 1)?;
    let no_target = target(no, &eligible, &mut remaining, depth + 1)?;
    let fallback = if yes_target.is_none() && no_target.is_none() {
        let mut remaining = 64;
        let yes = retention_targets::discover(yes, &eligible, &mut remaining, depth + 1)?;
        let no = retention_targets::discover(no, &eligible, &mut remaining, depth + 1)?;
        retention_targets::merge(yes, no)?
    } else {
        None
    };
    // A delegated owner can retain its target through work-only child stages.
    // Root candidates have no delegation and still require actual write evidence.
    let delegated = authorized.map(|target| target.name().to_owned());
    let name = yes_target
        .as_ref()
        .or(no_target.as_ref())
        .or(fallback.as_ref())
        .or(delegated.as_ref())?;
    if yes_target.as_ref().is_some_and(|target| target != name)
        || no_target.as_ref().is_some_and(|target| target != name)
        || (!staged.defined.contains(name)
            && !authorized.is_some_and(|target| target.name() == name))
        || staged.constants.contains(name)
        || staged.local.get(name)? != &scalar_type("bool")
    {
        return None;
    }
    branch(
        condition,
        yes,
        no,
        name,
        &staged.local,
        signatures,
        budget,
        depth,
    )
}

fn eligible(staged: &regions::Staging, authorized: Option<&Target<'_>>) -> BTreeSet<String> {
    staged
        .local
        .iter()
        .filter(|(name, ty)| {
            **ty == scalar_type("bool")
                && !staged.constants.contains(*name)
                && (staged.defined.contains(*name)
                    || authorized.is_some_and(|target| target.name() == *name))
        })
        .map(|(name, _)| name.clone())
        .collect()
}

fn terminal_private_child(
    yes: &[NirStmt],
    no: &[NirStmt],
    scope: &Scope,
    staged: &regions::Staging,
    authorized: &Target<'_>,
    depth: usize,
) -> Option<bool> {
    let eligible = eligible(staged, Some(authorized));
    let mut remaining = 64;
    let yes = retention_targets::discover(yes, &eligible, &mut remaining, depth + 1)?;
    let no = retention_targets::discover(no, &eligible, &mut remaining, depth + 1)?;
    let candidate = retention_targets::merge(yes, no)?;
    // Classification is write evidence, not another permission or source version.
    // Only a prefix-defined child absent from the entering scope stays private.
    Some(candidate.is_some_and(|child| {
        child != authorized.name() && staged.defined.contains(&child) && !scope.contains_key(&child)
    }))
}

fn target(
    body: &[NirStmt],
    eligible: &BTreeSet<String>,
    remaining: &mut usize,
    depth: usize,
) -> Option<Option<String>> {
    *remaining = remaining.checked_sub(body.len())?;
    if body.is_empty() {
        return Some(None);
    }
    let (last, prefix) = body.split_last()?;
    if prefix.len() > 16 {
        return None;
    }
    match last {
        NirStmt::Let { name, .. } if eligible.contains(name) => Some(Some(name.clone())),
        NirStmt::Let { .. } | NirStmt::Const { .. } => Some(None),
        NirStmt::If {
            then_body,
            else_body,
            ..
        } if depth < 8 => {
            let yes = target(then_body, eligible, remaining, depth + 1)?;
            let no = target(else_body, eligible, remaining, depth + 1)?;
            if yes
                .as_ref()
                .zip(no.as_ref())
                .is_some_and(|(yes, no)| yes != no)
            {
                return None;
            }
            Some(yes.or(no))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn branch(
    condition: &NirExpr,
    yes: &[NirStmt],
    no: &[NirStmt],
    name: &str,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    if depth >= 8 {
        return None;
    }
    nested::consume_node(budget)?;
    let (condition, reads, effects) = predicates::prove(condition, scope, signatures, budget)?;
    let yes = arm(yes, name, scope, signatures, budget, depth + 1)?;
    let no = arm(no, name, scope, signatures, budget, depth + 1)?;
    nested::join(condition, reads, effects, yes, no, scope)
}

fn arm(
    body: &[NirStmt],
    name: &str,
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<nested::Proof> {
    let Some((last, prefix)) = body.split_last() else {
        // Retention is a transport outcome, not invented original source work.
        return Some(nested::Proof {
            value: SelectedValue::Expression(NirExpr::Var(name.into())),
            name: name.into(),
            ty: scalar_type("bool"),
            inputs: BTreeSet::from([name.into()]),
            calls: BTreeSet::new(),
        });
    };
    let authorized = Target(name);
    let mut staged =
        super::stage_authorized(prefix, scope, signatures, budget, depth, Some(&authorized))?;
    let mut proof = match last {
        statement @ NirStmt::Let { name: target, .. } if target == name => {
            nested::leaf(statement, &staged.local, signatures, budget)?
        }
        NirStmt::If {
            then_body,
            else_body,
            ..
        } if terminal_private_child(then_body, else_body, scope, &staged, &authorized, depth)? => {
            super::append_authorized(
                last,
                scope,
                &mut staged,
                signatures,
                budget,
                depth,
                Some(&authorized),
            )?;
            nested::Proof {
                value: SelectedValue::Expression(NirExpr::Var(name.into())),
                name: name.into(),
                ty: scalar_type("bool"),
                inputs: BTreeSet::from([name.into()]),
                calls: BTreeSet::new(),
            }
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => branch(
            condition,
            then_body,
            else_body,
            name,
            &staged.local,
            signatures,
            budget,
            depth,
        )?,
        NirStmt::Let { .. } | NirStmt::Const { .. } => {
            // Keep the final private statement in the same staging environment:
            // seals, prior reads and once-only work cannot restart at the tail.
            super::append_authorized(
                last,
                scope,
                &mut staged,
                signatures,
                budget,
                depth,
                Some(&authorized),
            )?;
            nested::Proof {
                value: SelectedValue::Expression(NirExpr::Var(name.into())),
                name: name.into(),
                ty: scalar_type("bool"),
                inputs: BTreeSet::from([name.into()]),
                calls: BTreeSet::new(),
            }
        }
        _ => return None,
    };
    let mut inputs = staged.inputs;
    inputs.extend(proof.inputs.difference(&staged.defined).cloned());
    proof.inputs = inputs;
    proof.calls.extend(staged.calls);
    if !staged.statements.is_empty() {
        proof.value = match proof.value {
            SelectedValue::Nested(plan) => SelectedValue::StagedNested(staged.statements, plan),
            SelectedValue::Expression(value) => SelectedValue::Region(staged.statements, value),
            SelectedValue::Region(statements, value) => {
                let mut prefix = staged.statements;
                prefix.extend(statements);
                SelectedValue::Region(prefix, value)
            }
            _ => unreachable!(),
        };
    }
    Some(proof)
}
