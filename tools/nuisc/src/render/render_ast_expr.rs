use super::*;

pub(super) fn render_ast_expr(value: &AstExpr) -> String {
    match value {
        AstExpr::Bool(value) => value.to_string(),
        AstExpr::Text(text) => format!("\"{}\"", escape_debug(text)),
        AstExpr::Int(value) => value.to_string(),
        AstExpr::Float(value) => value.clone(),
        AstExpr::Var(name) => name.clone(),
        AstExpr::Try(value) => format!("{}?", render_ast_expr(value)),
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => format!(
            "if {} {{ {} }} else {{ {} }}",
            render_ast_expr(condition),
            then_body
                .iter()
                .map(render_ast_stmt_inline)
                .collect::<Vec<_>>()
                .join(" "),
            else_body
                .iter()
                .map(render_ast_stmt_inline)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        AstExpr::Match { value, arms } => format!(
            "match {} {{ {} }}",
            render_ast_expr(value),
            arms.iter()
                .map(|arm| {
                    let pattern = render_ast_match_pattern(&arm.pattern);
                    let guard = arm
                        .guard
                        .as_ref()
                        .map(|guard| format!(" if {}", render_ast_expr(guard)))
                        .unwrap_or_default();
                    let body = arm
                        .body
                        .iter()
                        .map(render_ast_stmt_inline)
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{pattern}{guard} => {{ {body} }}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => {
            let params = params
                .iter()
                .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_suffix = return_type
                .as_ref()
                .map(|ty| format!(" -> {}", render_ast_type(ty)))
                .unwrap_or_default();
            let body = body
                .iter()
                .map(render_ast_stmt_inline)
                .collect::<Vec<_>>()
                .join("; ");
            format!("|{params}|{return_suffix} {{ {body} }}")
        }
        AstExpr::Await(value) => format!("await {}", render_ast_expr(value)),
        AstExpr::Instantiate { domain, unit } => format!("instantiate {} {}", domain, unit),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => format!(
            "{}{}({})",
            callee,
            render_ast_generic_args(generic_args),
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::Invoke { callee, args } => format!(
            "({})({})",
            render_ast_expr(callee),
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => format!(
            "{}.{}{}({})",
            render_ast_expr(receiver),
            method,
            render_ast_generic_args(generic_args),
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => format!(
            "{}{} {{ {} }}",
            type_name,
            render_ast_generic_args(type_args),
            fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", render_ast_expr(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::FieldAccess { base, field } => format!("{}.{}", render_ast_expr(base), field),
        AstExpr::Unary { op, operand } => {
            format!("({}{})", render_ast_unary_op(*op), render_ast_expr(operand))
        }
        AstExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_ast_expr(lhs),
            render_ast_binary_op(*op),
            render_ast_expr(rhs)
        ),
    }
}
