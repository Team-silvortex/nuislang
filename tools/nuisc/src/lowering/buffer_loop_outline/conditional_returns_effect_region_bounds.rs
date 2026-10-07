use super::*;

pub(super) fn region_end(body: &[NirStmt]) -> Option<usize> {
    let mut statements = 32usize;
    let mut end = 0;
    for (index, stmt) in body.iter().enumerate() {
        let mut pending = vec![stmt];
        while let Some(stmt) = pending.pop() {
            statements = statements.checked_sub(1)?;
            match stmt {
                NirStmt::Print(_) => end = index + 1,
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    if then_body.len() + else_body.len() > statements {
                        return None;
                    }
                    pending.extend(then_body.iter().chain(else_body));
                }
                _ => {}
            }
        }
    }
    Some(end)
}

pub(in super::super::super) fn preflight(body: &[NirStmt], end: usize) -> bool {
    let mut statements = 32;
    let mut expressions = Vec::new();
    append_roots(&body[..end], true, &mut statements, &mut expressions)
        && append_roots(&body[end..], false, &mut statements, &mut expressions)
        && conditional_values::prefix::computed_expression_roots(expressions)
}

// Source and expanded ledgers use the same root authority and statement charge.
pub(in super::super::super) fn append_roots<'a>(
    body: &'a [NirStmt],
    prefix: bool,
    statements: &mut usize,
    expressions: &mut Vec<(&'a NirExpr, usize, bool)>,
) -> bool {
    let mut pending = vec![body];
    while let Some(body) = pending.pop() {
        let Some(remaining) = statements.checked_sub(body.len()) else {
            return false;
        };
        *statements = remaining;
        for stmt in body {
            match stmt {
                NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => {
                    expressions.push((value, 0, true))
                }
                NirStmt::Print(value) if prefix => expressions.push((value, 0, false)),
                NirStmt::Return(Some(value)) => expressions.push((value, 0, true)),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push((condition, 0, true));
                    pending.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                _ => return false,
            }
        }
    }
    true
}
