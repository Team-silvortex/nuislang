use super::*;

// This is write evidence only. Fresh child declarations cannot join `eligible`;
// the ordinary owner/staging proof still validates every original statement.
pub(super) fn discover(
    body: &[NirStmt],
    eligible: &BTreeSet<String>,
    remaining: &mut usize,
    depth: usize,
) -> Option<Option<String>> {
    *remaining = remaining.checked_sub(body.len())?;
    let mut target = None;
    for statement in body {
        let next = match statement {
            NirStmt::Let { name, .. } if eligible.contains(name) => Some(name.clone()),
            NirStmt::Let { .. } | NirStmt::Const { .. } => None,
            NirStmt::If {
                then_body,
                else_body,
                ..
            } if depth < 8 => {
                let yes = discover(then_body, eligible, remaining, depth + 1)?;
                let no = discover(else_body, eligible, remaining, depth + 1)?;
                merge(yes, no)?
            }
            _ => return None,
        };
        target = merge(target, next)?;
    }
    Some(target)
}

pub(super) fn merge(yes: Option<String>, no: Option<String>) -> Option<Option<String>> {
    if yes
        .as_ref()
        .zip(no.as_ref())
        .is_some_and(|(yes, no)| yes != no)
    {
        return None;
    }
    Some(yes.or(no))
}
