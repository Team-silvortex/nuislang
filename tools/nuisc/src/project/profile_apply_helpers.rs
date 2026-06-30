use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstModule, AstStmt};
use yir_core::{Node, Operation, Resource, ResourceKind, YirModule};

use super::sanitize_ident;

pub(in crate::project) fn collect_profile_int_bindings(body: &[AstStmt]) -> BTreeMap<String, i64> {
    let mut bindings = BTreeMap::new();
    for stmt in body {
        if let Some((name, value)) = extract_profile_int_binding(stmt) {
            bindings.insert(name.to_owned(), value);
        }
    }
    bindings
}

pub(in crate::project) fn extract_profile_call(stmt: &AstStmt) -> Option<(&str, &str, &[AstExpr])> {
    match stmt {
        AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
            if let AstExpr::Call {
                callee,
                generic_args: _,
                args,
            } = value
            {
                Some((name.as_str(), callee.as_str(), args.as_slice()))
            } else {
                None
            }
        }
        AstStmt::Expr(AstExpr::Call {
            callee,
            generic_args: _,
            args,
        }) => Some((callee.as_str(), callee.as_str(), args.as_slice())),
        _ => None,
    }
}

pub(in crate::project) fn extract_profile_int_binding(stmt: &AstStmt) -> Option<(&str, i64)> {
    match stmt {
        AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
            if let AstExpr::Int(value) = value {
                Some((name.as_str(), *value))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(in crate::project) fn expect_text_arg(
    args: &[AstExpr],
    index: usize,
    callee: &str,
) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Text(value)) => Ok(value.clone()),
        _ => Err(format!(
            "{callee}(...) expects string literal arg {}",
            index + 1
        )),
    }
}

pub(in crate::project) fn expect_profile_int_arg(
    args: &[AstExpr],
    index: usize,
    callee: &str,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<i64, String> {
    match args.get(index) {
        Some(AstExpr::Int(value)) => Ok(*value),
        Some(AstExpr::Var(name)) => int_bindings.get(name).copied().ok_or_else(|| {
            format!(
                "{callee}(...) expects integer literal or profile const arg {}, unknown `{}`",
                index + 1,
                name
            )
        }),
        _ => Err(format!(
            "{callee}(...) expects integer literal or profile const arg {}",
            index + 1
        )),
    }
}

pub(in crate::project) fn expect_profile_value_input_name(
    ast: &AstModule,
    args: &[AstExpr],
    index: usize,
    callee: &str,
) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Var(name)) => Ok(format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(name)
        )),
        _ => Err(format!(
            "{callee}(...) expects profile value reference arg {}",
            index + 1
        )),
    }
}

pub(in crate::project) fn ensure_project_resource(module: &mut YirModule, name: &str, kind: &str) {
    if let Some(resource) = module
        .resources
        .iter_mut()
        .find(|resource| resource.name == name)
    {
        resource.kind = ResourceKind::parse(kind);
        return;
    }
    module.resources.push(Resource {
        name: name.to_owned(),
        kind: ResourceKind::parse(kind),
    });
}

pub(in crate::project) fn push_profile_node(
    module: &mut YirModule,
    name: String,
    resource: &str,
    op: Operation,
) {
    if module.nodes.iter().any(|node| node.name == name) {
        return;
    }
    module.nodes.push(Node {
        name,
        resource: resource.to_owned(),
        op,
    });
}
