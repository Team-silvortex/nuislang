use nuis_semantics::model::{AstExpr, AstStmt, AstTypeRef};

pub(super) fn render_ast_stmt_inline(stmt: &AstStmt) -> String {
    match stmt {
        AstStmt::Let { name, ty, value } => {
            let suffix = render_ast_type_suffix(ty.as_ref());
            format!("let {}{} = {}", name, suffix, super::render_ast_expr(value))
        }
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => render_ast_destructure_let(type_ref, fields, value),
        AstStmt::Const { name, ty, value } => {
            let suffix = render_ast_type_suffix(ty.as_ref());
            format!(
                "const {}{} = {}",
                name,
                suffix,
                super::render_ast_expr(value)
            )
        }
        AstStmt::Print(value) => format!("print {}", super::render_ast_expr(value)),
        AstStmt::Await(value) => format!("await {}", super::render_ast_expr(value)),
        AstStmt::Expr(expr) => super::render_ast_expr(expr),
        AstStmt::If { .. } => "if ...".to_owned(),
        AstStmt::Match { .. } => "match ...".to_owned(),
        AstStmt::While { .. } => "while ...".to_owned(),
        AstStmt::Break => "break".to_owned(),
        AstStmt::Continue => "continue".to_owned(),
        AstStmt::Return(value) => match value {
            Some(value) => format!("return {}", super::render_ast_expr(value)),
            None => "return".to_owned(),
        },
    }
}

pub(super) fn render_ast_type_suffix(ty: Option<&AstTypeRef>) -> String {
    ty.map(|ty| format!(": {}", super::render_ast_type(ty)))
        .unwrap_or_default()
}

#[rustfmt::skip]
pub(super) fn render_ast_destructure_let(type_ref: &AstTypeRef, fields: &[String], value: &AstExpr) -> String {
    format!("let {} {{ {} }} = {}", super::render_ast_type(type_ref), fields.join(", "), super::render_ast_expr(value))
}
