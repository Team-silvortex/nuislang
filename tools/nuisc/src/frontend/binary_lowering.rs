use std::collections::BTreeMap;

use nuis_semantics::model::{AstBinaryOp, AstExpr, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef};

use super::{
    bool_type, compatible_types, infer_nir_expr_type, lower_nested_expr_with_async_and_consts,
    FunctionSignature, ModuleConstValue,
};

pub(super) fn lower_binary_expr_with_async(
    op: &AstBinaryOp,
    lhs: &AstExpr,
    rhs: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let lowered_lhs = lower_nested_expr_with_async_and_consts(
        lhs,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
    )?;
    let lowered_rhs = lower_nested_expr_with_async_and_consts(
        rhs,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
    )?;
    let lhs_ty = infer_nir_expr_type(&lowered_lhs, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer binary lhs type".to_owned())?;
    let rhs_ty = infer_nir_expr_type(&lowered_rhs, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer binary rhs type".to_owned())?;
    let result_ty = binary_result_type(*op, &lhs_ty, &rhs_ty)?;
    if matches!(
        op,
        AstBinaryOp::Add
            | AstBinaryOp::Sub
            | AstBinaryOp::Mul
            | AstBinaryOp::Div
            | AstBinaryOp::And
            | AstBinaryOp::Or
    ) && (!compatible_types(&lhs_ty, &result_ty) || !compatible_types(&rhs_ty, &result_ty))
    {
        return Err(format!(
            "binary operands must agree on type, found `{}` and `{}`",
            lhs_ty.render(),
            rhs_ty.render()
        ));
    }
    Ok(NirExpr::Binary {
        op: match op {
            AstBinaryOp::And => NirBinaryOp::And,
            AstBinaryOp::Or => NirBinaryOp::Or,
            AstBinaryOp::Add => NirBinaryOp::Add,
            AstBinaryOp::Sub => NirBinaryOp::Sub,
            AstBinaryOp::Mul => NirBinaryOp::Mul,
            AstBinaryOp::Div => NirBinaryOp::Div,
            AstBinaryOp::Eq => NirBinaryOp::Eq,
            AstBinaryOp::Ne => NirBinaryOp::Ne,
            AstBinaryOp::Lt => NirBinaryOp::Lt,
            AstBinaryOp::Le => NirBinaryOp::Le,
            AstBinaryOp::Gt => NirBinaryOp::Gt,
            AstBinaryOp::Ge => NirBinaryOp::Ge,
        },
        lhs: Box::new(lowered_lhs),
        rhs: Box::new(lowered_rhs),
    })
}

fn binary_result_type(
    op: AstBinaryOp,
    lhs: &NirTypeRef,
    rhs: &NirTypeRef,
) -> Result<NirTypeRef, String> {
    match op {
        AstBinaryOp::And | AstBinaryOp::Or => {
            if !compatible_types(lhs, rhs) {
                return Err(format!(
                    "binary `{}` expects matching operand types, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            if !lhs.is_bool_scalar() || !rhs.is_bool_scalar() {
                return Err(format!(
                    "binary `{}` currently expects bool scalar operands, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            Ok(bool_type())
        }
        AstBinaryOp::Add | AstBinaryOp::Sub | AstBinaryOp::Mul | AstBinaryOp::Div => {
            if !compatible_types(lhs, rhs) {
                return Err(format!(
                    "binary `{}` expects matching operand types, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            if !lhs.is_numeric_scalar() || !rhs.is_numeric_scalar() {
                return Err(format!(
                    "binary `{}` currently expects numeric scalar operands, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            Ok(lhs.clone())
        }
        AstBinaryOp::Eq | AstBinaryOp::Ne => {
            if !compatible_types(lhs, rhs) {
                return Err(format!(
                    "binary `{}` expects matching operand types, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            if !((lhs.is_integer_scalar() && rhs.is_integer_scalar())
                || (lhs.is_bool_scalar() && rhs.is_bool_scalar()))
            {
                return Err(format!(
                    "binary `{}` currently expects integer or bool scalar operands, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            Ok(bool_type())
        }
        AstBinaryOp::Lt | AstBinaryOp::Le | AstBinaryOp::Gt | AstBinaryOp::Ge => {
            if !compatible_types(lhs, rhs) {
                return Err(format!(
                    "binary `{}` expects matching operand types, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            if !lhs.is_integer_scalar() || !rhs.is_integer_scalar() {
                return Err(format!(
                    "binary `{}` currently expects integer scalar operands, found `{}` and `{}`",
                    render_binary_op(op),
                    lhs.render(),
                    rhs.render()
                ));
            }
            Ok(bool_type())
        }
    }
}

fn render_binary_op(op: AstBinaryOp) -> &'static str {
    match op {
        AstBinaryOp::And => "&&",
        AstBinaryOp::Or => "||",
        AstBinaryOp::Add => "+",
        AstBinaryOp::Sub => "-",
        AstBinaryOp::Mul => "*",
        AstBinaryOp::Div => "/",
        AstBinaryOp::Eq => "==",
        AstBinaryOp::Ne => "!=",
        AstBinaryOp::Lt => "<",
        AstBinaryOp::Le => "<=",
        AstBinaryOp::Gt => ">",
        AstBinaryOp::Ge => ">=",
    }
}
