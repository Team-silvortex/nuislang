use nuis_semantics::model::NirExpr;

use super::{ast_type_from_nir, AstStmt};

pub(super) fn rewrite_try_payload_placeholder(
    stmt: AstStmt,
    payload_name: &str,
) -> Result<AstStmt, String> {
    match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } => Ok(AstStmt::Let {
            mutable,
            name,
            ty,
            value: rewrite_try_payload_placeholder_expr(value, payload_name),
        }),
        AstStmt::Const { name, ty, value } => Ok(AstStmt::Const {
            name,
            ty,
            value: rewrite_try_payload_placeholder_expr(value, payload_name),
        }),
        AstStmt::Print(value) => Ok(AstStmt::Print(rewrite_try_payload_placeholder_expr(
            value,
            payload_name,
        ))),
        AstStmt::Expr(value) => Ok(AstStmt::Expr(rewrite_try_payload_placeholder_expr(
            value,
            payload_name,
        ))),
        AstStmt::Return(Some(value)) => Ok(AstStmt::Return(Some(
            rewrite_try_payload_placeholder_expr(value, payload_name),
        ))),
        other => Err(format!(
            "internal error: unsupported `?` payload rewrite target `{other:?}`"
        )),
    }
}

fn rewrite_try_payload_placeholder_expr(
    expr: super::AstExpr,
    payload_name: &str,
) -> super::AstExpr {
    match expr {
        super::AstExpr::Var(var) if var == "__nuis_try_payload" => {
            super::AstExpr::Var(payload_name.to_owned())
        }
        super::AstExpr::Await(value) => super::AstExpr::Await(Box::new(
            rewrite_try_payload_placeholder_expr(*value, payload_name),
        )),
        super::AstExpr::Try(value) => super::AstExpr::Try(Box::new(
            rewrite_try_payload_placeholder_expr(*value, payload_name),
        )),
        super::AstExpr::Call {
            callee,
            generic_args,
            args,
        } => super::AstExpr::Call {
            callee,
            generic_args,
            args: args
                .into_iter()
                .map(|arg| rewrite_try_payload_placeholder_expr(arg, payload_name))
                .collect(),
        },
        super::AstExpr::Invoke { callee, args } => super::AstExpr::Invoke {
            callee: Box::new(rewrite_try_payload_placeholder_expr(*callee, payload_name)),
            args: args
                .into_iter()
                .map(|arg| rewrite_try_payload_placeholder_expr(arg, payload_name))
                .collect(),
        },
        super::AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => super::AstExpr::MethodCall {
            receiver: Box::new(rewrite_try_payload_placeholder_expr(
                *receiver,
                payload_name,
            )),
            method,
            generic_args,
            args: args
                .into_iter()
                .map(|arg| rewrite_try_payload_placeholder_expr(arg, payload_name))
                .collect(),
        },
        super::AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => super::AstExpr::StructLiteral {
            type_name,
            type_args,
            fields: fields
                .into_iter()
                .map(|(field, value)| {
                    (
                        field,
                        rewrite_try_payload_placeholder_expr(value, payload_name),
                    )
                })
                .collect(),
        },
        super::AstExpr::FieldAccess { base, field } => super::AstExpr::FieldAccess {
            base: Box::new(rewrite_try_payload_placeholder_expr(*base, payload_name)),
            field,
        },
        super::AstExpr::Unary { op, operand } => super::AstExpr::Unary {
            op,
            operand: Box::new(rewrite_try_payload_placeholder_expr(*operand, payload_name)),
        },
        super::AstExpr::Binary { op, lhs, rhs } => super::AstExpr::Binary {
            op,
            lhs: Box::new(rewrite_try_payload_placeholder_expr(*lhs, payload_name)),
            rhs: Box::new(rewrite_try_payload_placeholder_expr(*rhs, payload_name)),
        },
        other => other,
    }
}

pub(super) fn ast_expr_from_nir(expr: NirExpr) -> super::AstExpr {
    match expr {
        NirExpr::Bool(value) => super::AstExpr::Bool(value),
        NirExpr::Text(text) => super::AstExpr::Text(text),
        NirExpr::Int(value) => super::AstExpr::Int(value),
        NirExpr::F32(value) | NirExpr::F64(value) => super::AstExpr::Float(value),
        NirExpr::Var(name) => super::AstExpr::Var(name),
        NirExpr::Await(value) => super::AstExpr::Await(Box::new(ast_expr_from_nir(*value))),
        NirExpr::Call { callee, args } => super::AstExpr::Call {
            callee,
            generic_args: Vec::new(),
            args: args.into_iter().map(ast_expr_from_nir).collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => super::AstExpr::StructLiteral {
            type_name,
            type_args: type_args.iter().map(ast_type_from_nir).collect(),
            fields: fields
                .into_iter()
                .map(|(field, value)| (field, ast_expr_from_nir(value)))
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => super::AstExpr::FieldAccess {
            base: Box::new(ast_expr_from_nir(*base)),
            field,
        },
        NirExpr::Binary { op, lhs, rhs } => super::AstExpr::Binary {
            op: match op {
                nuis_semantics::model::NirBinaryOp::And => nuis_semantics::model::AstBinaryOp::And,
                nuis_semantics::model::NirBinaryOp::Or => nuis_semantics::model::AstBinaryOp::Or,
                nuis_semantics::model::NirBinaryOp::Add => nuis_semantics::model::AstBinaryOp::Add,
                nuis_semantics::model::NirBinaryOp::Sub => nuis_semantics::model::AstBinaryOp::Sub,
                nuis_semantics::model::NirBinaryOp::Mul => nuis_semantics::model::AstBinaryOp::Mul,
                nuis_semantics::model::NirBinaryOp::Div => nuis_semantics::model::AstBinaryOp::Div,
                nuis_semantics::model::NirBinaryOp::Rem => nuis_semantics::model::AstBinaryOp::Rem,
                nuis_semantics::model::NirBinaryOp::Eq => nuis_semantics::model::AstBinaryOp::Eq,
                nuis_semantics::model::NirBinaryOp::Ne => nuis_semantics::model::AstBinaryOp::Ne,
                nuis_semantics::model::NirBinaryOp::Lt => nuis_semantics::model::AstBinaryOp::Lt,
                nuis_semantics::model::NirBinaryOp::Le => nuis_semantics::model::AstBinaryOp::Le,
                nuis_semantics::model::NirBinaryOp::Gt => nuis_semantics::model::AstBinaryOp::Gt,
                nuis_semantics::model::NirBinaryOp::Ge => nuis_semantics::model::AstBinaryOp::Ge,
            },
            lhs: Box::new(ast_expr_from_nir(*lhs)),
            rhs: Box::new(ast_expr_from_nir(*rhs)),
        },
        other => panic!("internal error: unsupported NIR-to-AST try expansion expr {other:?}"),
    }
}
