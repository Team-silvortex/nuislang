use super::*;

#[cfg(test)]
#[path = "scalar_joins_discarded_records_tests.rs"]
mod tests;

// Preflight every synthetic record in the original tree before any zero is built.
pub(super) fn bounded(
    yes: &[NirStmt],
    no: &[NirStmt],
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    let mut remaining = 4096usize;
    let mut blocks = vec![yes, no];
    while let Some(body) = blocks.pop() {
        for stmt in body {
            if let NirStmt::If {
                then_body,
                else_body,
                ..
            } = stmt
            {
                let retained = match (then_body.is_empty(), else_body.is_empty()) {
                    (false, true) => Some(then_body),
                    (true, false) => Some(else_body),
                    _ => None,
                };
                if let Some(retained) = retained {
                    let Some((_, ty, _)) = result(retained) else {
                        return false;
                    };
                    if !layouts.scalar(&ty.name) && !neutral(ty, layouts, &mut remaining) {
                        return false;
                    }
                }
                blocks.extend([then_body.as_slice(), else_body.as_slice()]);
            }
        }
    }
    true
}

fn neutral(
    ty: &NirTypeRef,
    layouts: &impl control_values::ValueLayouts,
    remaining: &mut usize,
) -> bool {
    let mut pending = vec![(ty.clone(), 0usize)];
    while let Some((ty, depth)) = pending.pop() {
        if depth >= 64 || !control_values::supported_type(&ty, layouts) {
            return false;
        }
        // An i32 zero is a cast plus its i64 operand, not a single leaf node.
        let cost = if ty.name == "i32" { 2 } else { 1 };
        let Some(next) = remaining.checked_sub(cost) else {
            return false;
        };
        if ty.name == "i32" && depth + 1 >= 64 {
            return false;
        }
        *remaining = next;
        if let Some(fields) = layouts.fields(&ty.name) {
            if pending.len() + fields.len() > *remaining {
                return false;
            }
            pending.extend(fields.map(|(_, field)| (field, depth + 1)));
        }
    }
    true
}
