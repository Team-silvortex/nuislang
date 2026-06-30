use nuis_semantics::model::{AstBinaryOp, AstTypeRef, AstUnaryOp};

use super::super::lower_type_ref;

pub(super) fn overloaded_binary_trait(op: AstBinaryOp) -> Option<(&'static str, &'static str)> {
    match op {
        AstBinaryOp::Add => Some(("Addable", "add")),
        AstBinaryOp::Sub => Some(("Subtractable", "sub")),
        AstBinaryOp::Mul => Some(("Multipliable", "mul")),
        AstBinaryOp::Div => Some(("Dividable", "div")),
        AstBinaryOp::Rem => Some(("Remainderable", "rem")),
        AstBinaryOp::Eq | AstBinaryOp::Ne => Some(("Equatable", "eq")),
        AstBinaryOp::Lt => Some(("Orderable", "lt")),
        AstBinaryOp::Le => Some(("Orderable", "le")),
        AstBinaryOp::Gt => Some(("Orderable", "gt")),
        AstBinaryOp::Ge => Some(("Orderable", "ge")),
        _ => None,
    }
}

pub(super) fn overloaded_unary_trait(op: AstUnaryOp) -> Option<(&'static str, &'static str)> {
    match op {
        AstUnaryOp::Not => Some(("Notable", "not")),
        AstUnaryOp::Neg => Some(("Negatable", "neg")),
        AstUnaryOp::Deref => None,
    }
}

pub(super) fn builtin_binary_supported_ast(
    op: AstBinaryOp,
    lhs_ty: &AstTypeRef,
    rhs_ty: &AstTypeRef,
) -> bool {
    let same = lower_type_ref(lhs_ty).render() == lower_type_ref(rhs_ty).render();
    if !same {
        return false;
    }
    match op {
        AstBinaryOp::And | AstBinaryOp::Or => is_plain_scalar(lhs_ty, "bool"),
        AstBinaryOp::Add
        | AstBinaryOp::Sub
        | AstBinaryOp::Mul
        | AstBinaryOp::Div
        | AstBinaryOp::Rem => is_plain_numeric_scalar(lhs_ty),
        AstBinaryOp::Eq | AstBinaryOp::Ne => {
            is_plain_integer_scalar(lhs_ty)
                || is_plain_float_scalar(lhs_ty)
                || is_plain_scalar(lhs_ty, "bool")
        }
        AstBinaryOp::Lt | AstBinaryOp::Le | AstBinaryOp::Gt | AstBinaryOp::Ge => {
            is_plain_numeric_scalar(lhs_ty)
        }
    }
}

pub(super) fn builtin_unary_supported_ast(op: AstUnaryOp, operand_ty: &AstTypeRef) -> bool {
    match op {
        AstUnaryOp::Not => is_plain_scalar(operand_ty, "bool") || is_ref_type(operand_ty),
        AstUnaryOp::Neg => {
            is_plain_scalar(operand_ty, "i64")
                || is_plain_scalar(operand_ty, "f32")
                || is_plain_scalar(operand_ty, "f64")
        }
        AstUnaryOp::Deref => {
            operand_ty.name == "Node" && operand_ty.is_ref && !operand_ty.is_optional
        }
    }
}

fn is_plain_scalar(ty: &AstTypeRef, name: &str) -> bool {
    ty.name == name && !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty()
}

fn is_plain_integer_scalar(ty: &AstTypeRef) -> bool {
    is_plain_scalar(ty, "i32") || is_plain_scalar(ty, "i64")
}

fn is_plain_float_scalar(ty: &AstTypeRef) -> bool {
    is_plain_scalar(ty, "f32") || is_plain_scalar(ty, "f64")
}

fn is_plain_numeric_scalar(ty: &AstTypeRef) -> bool {
    is_plain_integer_scalar(ty) || is_plain_float_scalar(ty)
}

fn is_ref_type(ty: &AstTypeRef) -> bool {
    ty.is_ref && !ty.is_optional
}
