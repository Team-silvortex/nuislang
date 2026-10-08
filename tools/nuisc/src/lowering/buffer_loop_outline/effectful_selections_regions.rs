use super::*;

#[cfg(test)]
#[path = "effectful_selections_regions_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_staging_logical_tests.rs"]
mod logical_tests;

#[cfg(test)]
#[path = "effectful_selections_update_logical_tests.rs"]
mod update_tests;

pub(super) enum Statement {
    Scalar(NirStmt),
    Logical(NirStmt, predicates::Predicate),
}

pub(super) struct Staging {
    pub(super) local: Scope,
    pub(super) defined: BTreeSet<String>,
    pub(super) inputs: BTreeSet<String>,
    pub(super) calls: BTreeSet<String>,
    pub(super) constants: BTreeSet<String>,
    pub(super) statements: Vec<Statement>,
}

pub(super) fn prove(
    statements: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<nested::Proof> {
    if !(2..=16).contains(&statements.len()) {
        return None;
    }
    let NirStmt::Let { name: target, .. } = statements.last()? else {
        return None;
    };
    let result_type = scope.get(target)?;
    if !scalar(result_type) {
        return None;
    }
    for statement in statements {
        let name = match statement {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => name,
            _ => return None,
        };
        if scope.contains_key(name) && name != target {
            return None;
        }
    }
    let staged = stage(statements, scope, &BTreeSet::new(), signatures, budget)?;
    if staged.local.get(target)? != result_type {
        return None;
    }
    Some(nested::Proof {
        value: SelectedValue::Region(staged.statements, NirExpr::Var(target.clone())),
        name: target.clone(),
        ty: result_type.clone(),
        inputs: staged.inputs,
        calls: staged.calls,
    })
}

pub(super) fn stage(
    statements: &[NirStmt],
    scope: &Scope,
    constants: &BTreeSet<String>,
    signatures: &Signatures,
    budget: &mut nested::Budget,
) -> Option<Staging> {
    if !(1..=16).contains(&statements.len()) {
        return None;
    }
    let mut local = scope.clone();
    let mut defined = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut calls = BTreeSet::new();
    let mut constants = constants.clone();
    let mut planned = Vec::new();
    for statement in statements {
        nested::consume_node(budget)?;
        let (name, ty, value, constant) = match statement {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value, false),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value, true),
            _ => return None,
        };
        if constants.contains(name) {
            return None;
        }
        // Capture the RHS version before installing its LHS. Each caller proves
        // which outer writes may be published; private staged names never escape.
        let (inferred, reads, effects, statement) = if matches!(
            value,
            NirExpr::Binary {
                op: NirBinaryOp::And | NirBinaryOp::Or,
                ..
            }
        ) {
            // Inspect the complete RHS against the preceding binding version.
            // Const roots stay fresh; existing let roots must be owned bools.
            if (constant && local.contains_key(name))
                || local.get(name).is_some_and(|ty| ty != &scalar_type("bool"))
            {
                return None;
            }
            let (predicate, reads, effects) = predicates::prove(value, &local, signatures, budget)?;
            (
                scalar_type("bool"),
                reads,
                effects,
                Statement::Logical(statement.clone(), predicate),
            )
        } else {
            if constant {
                return None;
            }
            let (inferred, reads, effects) = nested::inspect(value, &local, signatures, budget)?;
            (
                inferred,
                reads,
                effects,
                Statement::Scalar(statement.clone()),
            )
        };
        if ty.is_some_and(|ty| ty != &inferred) || local.get(name).is_some_and(|ty| ty != &inferred)
        {
            return None;
        }
        inputs.extend(reads.difference(&defined).cloned());
        calls.extend(effects);
        local.insert(name.clone(), inferred);
        defined.insert(name.clone());
        if constant {
            constants.insert(name.clone());
        }
        planned.push(statement);
    }
    Some(Staging {
        local,
        defined,
        inputs,
        calls,
        constants,
        statements: planned,
    })
}

pub(super) fn writes_logical_constants(
    yes: &[NirStmt],
    no: &[NirStmt],
    constants: &BTreeSet<String>,
) -> bool {
    if constants.is_empty() {
        return false;
    }
    // Candidate preflight is bounded independently of expression recursion.
    // Oversized candidates cannot enter the 64-node scalar-region proof.
    let mut pending = Vec::new();
    let mut visited = 0;
    if yes.len() + no.len() > 64 {
        return true;
    }
    pending.extend(yes.iter().chain(no));
    while let Some(statement) = pending.pop() {
        visited += 1;
        match statement {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. }
                if constants.contains(name)
                    && matches!(
                        value,
                        NirExpr::Binary {
                            op: NirBinaryOp::And | NirBinaryOp::Or,
                            ..
                        }
                    ) =>
            {
                return true
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                if visited + pending.len() + then_body.len() + else_body.len() > 64 {
                    return true;
                }
                pending.extend(then_body.iter().chain(else_body));
            }
            _ => {}
        }
    }
    false
}

pub(super) fn install(
    statements: Vec<Statement>,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
) -> Vec<NirStmt> {
    statements
        .into_iter()
        .map(|statement| match statement {
            Statement::Scalar(statement) => statement,
            Statement::Logical(mut statement, predicate) => {
                let value = match &mut statement {
                    NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
                    _ => unreachable!(),
                };
                *value = predicates::install(predicate, names, bindings, helpers);
                statement
            }
        })
        .collect()
}
