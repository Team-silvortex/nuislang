use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_mixed_sequence_tree_tests.rs"]
mod tests;

pub(super) fn continued(yes: &[NirStmt], no: &[NirStmt]) -> Option<bool> {
    let mut pending = vec![yes, no];
    let mut remaining = 64usize;
    let mut found = false;
    while let Some(body) = pending.pop() {
        remaining = remaining.checked_sub(body.len())?;
        for (index, statement) in body.iter().enumerate() {
            if let NirStmt::If {
                then_body,
                else_body,
                ..
            } = statement
            {
                found |= index + 1 < body.len();
                pending.extend([then_body.as_slice(), else_body.as_slice()]);
            }
        }
    }
    Some(found)
}

fn shape(body: &[NirStmt], remaining: &mut usize, depth: usize) -> Option<()> {
    *remaining = remaining.checked_sub(body.len())?;
    if body.len() > 16 {
        return None;
    }
    for statement in body {
        match statement {
            NirStmt::Let { .. } | NirStmt::Const { .. } => {}
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if depth < 8 => {
                shape(then_body, remaining, depth + 1)?;
                shape(else_body, remaining, depth + 1)?;
            }
            _ => return None,
        }
    }
    Some(())
}

pub(super) fn preflight(
    yes: &[NirStmt],
    no: &[NirStmt],
    suffix: &[NirStmt],
    depth: usize,
) -> Option<()> {
    if depth >= 8 {
        return None;
    }
    // One counter spans the original forest, including the first selection.
    let mut remaining = 63;
    shape(yes, &mut remaining, depth + 1)?;
    shape(no, &mut remaining, depth + 1)?;
    shape(suffix, &mut remaining, depth)
}

pub(super) fn isolated(body: &[NirStmt], scope: &Scope, future: &BTreeSet<String>) -> Option<()> {
    let mut pending = vec![body];
    let mut remaining = 64usize;
    while let Some(body) = pending.pop() {
        remaining = remaining.checked_sub(body.len())?;
        if !fresh_names(body, scope).is_disjoint(future) {
            return None;
        }
        for statement in body {
            if let NirStmt::If {
                then_body,
                else_body,
                ..
            } = statement
            {
                pending.extend([then_body.as_slice(), else_body.as_slice()]);
            }
        }
    }
    Some(())
}
