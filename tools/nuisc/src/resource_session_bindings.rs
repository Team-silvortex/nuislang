use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

use nuis_semantics::model::{AstExpr, AstModule, AstStmt, NirModule};

#[path = "resource_session_bindings/types.rs"]
mod types;

#[cfg(test)]
#[path = "resource_session_bindings_tests.rs"]
mod tests;

pub(crate) struct Normalized<'a> {
    pub ast: Cow<'a, AstModule>,
    shadows: BTreeMap<String, BTreeMap<String, Shadow>>,
}

struct Shadow {
    source: String,
    previous: String,
}

impl Normalized<'_> {
    pub(crate) fn lower(&self, local_modules: &[AstModule]) -> Result<NirModule, String> {
        let nir = crate::frontend::lower_project_ast_to_nir(&self.ast, local_modules)?;
        // Use canonical semantic types before optimization can erase declarations.
        types::validate(&nir, &self.shadows)?;
        Ok(nir)
    }
}

// NIR represents both declarations and assignments as Let. Preserve lexical
// candidates first, then admit only scalar identities after shared type resolution.
pub(crate) fn normalize<'a>(module: &'a AstModule, roots: &BTreeSet<String>) -> Normalized<'a> {
    if roots.is_empty() {
        return Normalized {
            ast: Cow::Borrowed(module),
            shadows: BTreeMap::new(),
        };
    }
    let mut module = module.clone();
    let mut shadows = BTreeMap::new();
    for function in &mut module.functions {
        if !roots.contains(&function.name) {
            continue;
        }
        let mut reserved = function
            .params
            .iter()
            .map(|p| p.name.clone())
            .chain(module.consts.iter().map(|c| c.name.clone()))
            .collect::<BTreeSet<_>>();
        // Do not partially rewrite a region with an unknown lexical scope.
        if !collect_names(&function.body, &mut reserved) {
            continue;
        }
        let visible = function
            .params
            .iter()
            .map(|p| (p.name.clone(), p.name.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut function_shadows = BTreeMap::new();
        let mut serial = 0;
        let mut pending = vec![(&mut function.body, visible, false)];
        while let Some((body, mut visible, nested)) = pending.pop() {
            for stmt in body {
                match stmt {
                    AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
                        rewrite_expr(value, &visible);
                        let identity = if let Some(previous) = visible.get(name).filter(|_| nested)
                        {
                            let identity = fresh(&mut reserved, &mut serial);
                            function_shadows.insert(
                                identity.clone(),
                                Shadow {
                                    source: name.clone(),
                                    previous: previous.clone(),
                                },
                            );
                            identity
                        } else {
                            name.clone()
                        };
                        visible.insert(name.clone(), identity.clone());
                        *name = identity;
                    }
                    AstStmt::AssignLocal { name, value } => {
                        rewrite_expr(value, &visible);
                        if let Some(identity) = visible.get(name) {
                            *name = identity.clone();
                        }
                    }
                    AstStmt::If {
                        condition,
                        then_body,
                        else_body,
                    } => {
                        rewrite_expr(condition, &visible);
                        pending.push((else_body, visible.clone(), true));
                        pending.push((then_body, visible.clone(), true));
                    }
                    AstStmt::Print(value)
                    | AstStmt::Await(value)
                    | AstStmt::Expr(value)
                    | AstStmt::Return(Some(value)) => {
                        rewrite_expr(value, &visible);
                    }
                    AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
                    _ => unreachable!("resource lexical region was preflighted"),
                }
            }
        }
        if !function_shadows.is_empty() {
            shadows.insert(function.name.clone(), function_shadows);
        }
    }
    Normalized {
        ast: Cow::Owned(module),
        shadows,
    }
}

fn fresh(reserved: &mut BTreeSet<String>, serial: &mut usize) -> String {
    loop {
        let name = format!("__nuis_resource_local_{serial}");
        *serial += 1;
        if reserved.insert(name.clone()) {
            return name;
        }
    }
}

fn collect_names(body: &[AstStmt], reserved: &mut BTreeSet<String>) -> bool {
    let mut blocks = vec![body];
    let mut exprs = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                AstStmt::Let { name, value, .. }
                | AstStmt::Const { name, value, .. }
                | AstStmt::AssignLocal { name, value } => {
                    reserved.insert(name.clone());
                    exprs.push(value);
                }
                AstStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    exprs.push(condition);
                    blocks.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                AstStmt::Print(value)
                | AstStmt::Await(value)
                | AstStmt::Expr(value)
                | AstStmt::Return(Some(value)) => exprs.push(value),
                AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
                _ => return false,
            }
        }
    }
    while let Some(expr) = exprs.pop() {
        match expr {
            AstExpr::Var(name) => {
                reserved.insert(name.clone());
            }
            AstExpr::Call { callee, args, .. } => {
                reserved.insert(callee.clone());
                exprs.extend(args);
            }
            AstExpr::MethodCall { receiver, args, .. } => {
                exprs.push(receiver);
                exprs.extend(args);
            }
            AstExpr::StructLiteral { fields, .. } => exprs.extend(fields.iter().map(|(_, v)| v)),
            AstExpr::FieldAccess { base, .. }
            | AstExpr::Unary { operand: base, .. }
            | AstExpr::Await(base)
            | AstExpr::Try(base) => exprs.push(base),
            AstExpr::Binary { lhs, rhs, .. } => exprs.extend([lhs.as_ref(), rhs.as_ref()]),
            AstExpr::Bool(_)
            | AstExpr::Int(_)
            | AstExpr::Float(_)
            | AstExpr::Text(_)
            | AstExpr::Instantiate { .. } => {}
            _ => return false,
        }
    }
    true
}

fn rewrite_expr(expr: &mut AstExpr, visible: &BTreeMap<String, String>) {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        match expr {
            AstExpr::Var(name) => {
                if let Some(identity) = visible.get(name) {
                    *name = identity.clone();
                }
            }
            // Function symbols, types and field labels are not local variables.
            AstExpr::Call { args, .. } => pending.extend(args),
            AstExpr::MethodCall { receiver, args, .. } => {
                pending.push(receiver);
                pending.extend(args);
            }
            AstExpr::StructLiteral { fields, .. } => {
                pending.extend(fields.iter_mut().map(|(_, v)| v))
            }
            AstExpr::FieldAccess { base, .. }
            | AstExpr::Unary { operand: base, .. }
            | AstExpr::Await(base)
            | AstExpr::Try(base) => pending.push(base),
            AstExpr::Binary { lhs, rhs, .. } => pending.extend([lhs.as_mut(), rhs.as_mut()]),
            AstExpr::Bool(_)
            | AstExpr::Int(_)
            | AstExpr::Float(_)
            | AstExpr::Text(_)
            | AstExpr::Instantiate { .. } => {}
            _ => unreachable!("resource lexical expression was preflighted"),
        }
    }
}
