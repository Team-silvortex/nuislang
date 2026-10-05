use super::*;

#[cfg(test)]
#[path = "conditional_returns_suffix_tests.rs"]
mod tests;

pub(super) fn prepare(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Vec<NirStmt>> {
    // Validate original scopes before moving a suffix into continuing arms.
    // A branch-local value cannot become an input of its former parent suffix.
    if !preflight(body) {
        return None;
    }
    validate(body, scope.clone(), result, catalog, layouts)?;
    let mut statements = 32;
    let plan = plan(body.iter().collect(), &mut statements)?;
    let mut pending = vec![plan.as_slice()];
    let mut expressions = Vec::new();
    while let Some(body) = pending.pop() {
        for part in body {
            match part {
                Part::Statement(NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) => {
                    expressions.push((value, 0, true));
                }
                Part::Statement(NirStmt::Return(Some(value))) => {
                    expressions.push((value, 0, true));
                }
                Part::Branch(condition, yes, no) => {
                    expressions.push((*condition, 0, true));
                    pending.extend([yes.as_slice(), no.as_slice()]);
                }
                _ => unreachable!("preflighted source sequence"),
            }
        }
    }
    // Bound expanded code before cloning any expression. Normalization cannot
    // buy a larger statement/node budget through statically duplicated tails.
    conditional_values::prefix::computed_expression_roots(expressions).then(|| materialize(plan))
}

fn preflight(body: &[NirStmt]) -> bool {
    let mut pending = vec![body];
    let mut statements = 32;
    let mut expressions = Vec::new();
    let mut intermediate = false;
    while let Some(body) = pending.pop() {
        if body.len() > statements {
            return false;
        }
        statements -= body.len();
        for (index, stmt) in body.iter().enumerate() {
            match stmt {
                NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
                    expressions.push((value, 0, true));
                }
                NirStmt::Return(Some(value)) => expressions.push((value, 0, true)),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    intermediate |= index + 1 < body.len();
                    expressions.push((condition, 0, true));
                    pending.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                _ => return false,
            }
        }
    }
    intermediate && conditional_values::prefix::computed_expression_roots(expressions)
}

fn validate(
    body: &[NirStmt],
    mut scope: Scope,
    result: &NirTypeRef,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<bool> {
    let mut continues = true;
    for stmt in body {
        // Unreachable source suffixes are not new normalization authority.
        if !continues {
            return None;
        }
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                let declared = match stmt {
                    NirStmt::Let { ty, .. } => ty.as_ref(),
                    NirStmt::Const { ty, .. } => Some(ty),
                    _ => unreachable!(),
                };
                if scope.contains_key(name) {
                    return None;
                }
                let inferred = control_values::value_type(value, &scope, catalog, layouts)?;
                if declared.is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                scope.insert(name.clone(), inferred);
            }
            NirStmt::Return(Some(value)) => {
                if !expression(value, &scope)
                    || control_values::value_type(value, &scope, catalog, layouts).as_ref()
                        != Some(result)
                {
                    return None;
                }
                continues = false;
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if control_values::value_type(condition, &scope, catalog, layouts)
                    != Some(scalar_type("bool"))
                {
                    return None;
                }
                let yes = validate(then_body, scope.clone(), result, catalog, layouts)?;
                let no = validate(else_body, scope.clone(), result, catalog, layouts)?;
                continues = yes || no;
            }
            _ => return None,
        }
    }
    Some(continues)
}

enum Part<'a> {
    Statement(&'a NirStmt),
    Branch(&'a NirExpr, Vec<Part<'a>>, Vec<Part<'a>>),
}

fn plan<'a>(sequence: Vec<&'a NirStmt>, statements: &mut usize) -> Option<Vec<Part<'a>>> {
    let mut output = Vec::new();
    for (index, stmt) in sequence.iter().enumerate() {
        *statements = statements.checked_sub(1)?;
        match stmt {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                let tail = &sequence[index + 1..];
                let yes = plan(
                    then_body.iter().chain(tail.iter().copied()).collect(),
                    statements,
                )?;
                let no = plan(
                    else_body.iter().chain(tail.iter().copied()).collect(),
                    statements,
                )?;
                output.push(Part::Branch(condition, yes, no));
                return Some(output);
            }
            NirStmt::Return(Some(_)) => {
                output.push(Part::Statement(stmt));
                // Do not append any inherited suffix to an observed return.
                return Some(output);
            }
            _ => output.push(Part::Statement(stmt)),
        }
    }
    Some(output)
}

fn materialize(plan: Vec<Part<'_>>) -> Vec<NirStmt> {
    plan.into_iter()
        .map(|part| match part {
            Part::Statement(stmt) => stmt.clone(),
            Part::Branch(condition, yes, no) => NirStmt::If {
                condition: condition.clone(),
                then_body: materialize(yes),
                else_body: materialize(no),
            },
        })
        .collect()
}
