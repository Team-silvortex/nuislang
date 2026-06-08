use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstStmt, AstTypeRef,
};

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
        } => render_ast_destructure_let(type_ref.as_ref(), fields, value),
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
pub(super) fn render_ast_destructure_let(type_ref: Option<&AstTypeRef>, fields: &[AstDestructureField], value: &AstExpr) -> String {
    let fields = fields
        .iter()
        .map(render_ast_destructure_field)
        .collect::<Vec<_>>()
        .join(", ");
    match type_ref {
        Some(type_ref) => format!("let {} {{ {} }} = {}", super::render_ast_type(type_ref), fields, super::render_ast_expr(value)),
        None => format!("let {{ {} }} = {}", fields, super::render_ast_expr(value)),
    }
}

fn render_ast_destructure_field(field: &AstDestructureField) -> String {
    match &field.binding {
        AstDestructureBinding::Bind(binding) if field.field == *binding => field.field.clone(),
        AstDestructureBinding::Bind(binding) => format!("{}: {}", field.field, binding),
        AstDestructureBinding::Ignore => format!("{}: _", field.field),
        AstDestructureBinding::Nested { type_ref, fields } => {
            let nested = fields
                .iter()
                .map(render_ast_destructure_field)
                .collect::<Vec<_>>()
                .join(", ");
            match type_ref {
                Some(type_ref) => format!(
                    "{}: {} {{ {} }}",
                    field.field,
                    super::render_ast_type(type_ref),
                    nested
                ),
                None => format!("{}: {{ {} }}", field.field, nested),
            }
        }
    }
}
