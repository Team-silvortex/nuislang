use super::*;

const MAX_NODES: usize = 65_536;
const MAX_DEPTH: usize = 64;

pub(super) struct Budget(usize);

impl Budget {
    pub(super) fn new() -> Self {
        Self(MAX_NODES)
    }
}

enum Node<'a> {
    Statement(&'a NirStmt),
    Expression(&'a NirExpr),
    Type(&'a NirTypeRef),
}

// Preflight both trees before any recursive equality checks. Unsupported
// resource/async syntax fails closed, never becoming effect-free by omission.
pub(super) fn preflight(
    body: &[NirStmt],
    budget: &mut Budget,
    mut names: Option<&mut BTreeSet<String>>,
) -> Option<()> {
    if body.len() > budget.0 {
        return None;
    }
    let mut pending = body
        .iter()
        .map(|s| (Node::Statement(s), 0))
        .collect::<Vec<_>>();
    while let Some((node, depth)) = pending.pop() {
        if depth >= MAX_DEPTH {
            return None;
        }
        budget.0 = budget.0.checked_sub(1)?;
        let mut push = |node| {
            if pending.len() >= budget.0 {
                return None;
            }
            pending.push((node, depth + 1));
            Some(())
        };
        match node {
            Node::Statement(stmt) => match stmt {
                NirStmt::Let { name, ty, value } => {
                    if let Some(names) = &mut names {
                        names.insert(name.clone());
                    }
                    if let Some(ty) = ty {
                        push(Node::Type(ty))?;
                    }
                    push(Node::Expression(value))?;
                }
                NirStmt::Const { name, ty, value } => {
                    if let Some(names) = &mut names {
                        names.insert(name.clone());
                    }
                    push(Node::Type(ty))?;
                    push(Node::Expression(value))?;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    push(Node::Expression(condition))?;
                    for stmt in then_body.iter().chain(else_body) {
                        push(Node::Statement(stmt))?;
                    }
                }
                NirStmt::While { condition, body } => {
                    push(Node::Expression(condition))?;
                    for stmt in body {
                        push(Node::Statement(stmt))?;
                    }
                }
                NirStmt::Return(Some(value)) | NirStmt::Print(value) => {
                    push(Node::Expression(value))?;
                }
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
                _ => return None,
            },
            Node::Expression(expr) => match expr {
                NirExpr::Var(name) => {
                    if let Some(names) = &mut names {
                        names.insert(name.clone());
                    }
                }
                NirExpr::Binary { lhs, rhs, .. } => {
                    push(Node::Expression(lhs))?;
                    push(Node::Expression(rhs))?;
                }
                NirExpr::Call { args, .. } => {
                    for value in args {
                        push(Node::Expression(value))?;
                    }
                }
                NirExpr::StructLiteral {
                    type_args, fields, ..
                } => {
                    for ty in type_args {
                        push(Node::Type(ty))?;
                    }
                    for (_, value) in fields {
                        push(Node::Expression(value))?;
                    }
                }
                NirExpr::FieldAccess { base, .. }
                | NirExpr::CastI64ToI32(base)
                | NirExpr::CastI32ToI64(base)
                | NirExpr::PackF64Word(base)
                | NirExpr::UnpackF64Word(base)
                | NirExpr::PackF32Word(base)
                | NirExpr::UnpackF32Word(base) => push(Node::Expression(base))?,
                NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::F32(_) | NirExpr::F64(_) => {}
                _ => return None,
            },
            Node::Type(ty) => {
                for arg in &ty.generic_args {
                    push(Node::Type(arg))?;
                }
            }
        }
        // Limit the work queue as well as the visited-node count.
        if pending.len() > budget.0 {
            return None;
        }
    }
    Some(())
}
