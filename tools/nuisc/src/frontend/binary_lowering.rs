use std::collections::BTreeMap;

use nuis_semantics::model::{AstBinaryOp, AstExpr, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef};

use super::{
    bool_type, compatible_types, find_impl_method_signature, infer_nir_expr_type,
    lower_nested_expr_with_async_and_consts, FunctionSignature, ModuleConstValue,
    NestedExprWithConstsInput,
};

pub(super) struct BinaryLoweringInput<'a> {
    pub(super) op: &'a AstBinaryOp,
    pub(super) lhs: &'a AstExpr,
    pub(super) rhs: &'a AstExpr,
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
    pub(super) expected: Option<&'a NirTypeRef>,
}

pub(super) fn lower_binary_expr_with_async(
    input: BinaryLoweringInput<'_>,
) -> Result<NirExpr, String> {
    let BinaryLoweringInput {
        op,
        lhs,
        rhs,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    } = input;
    let operand_expected = match op {
        AstBinaryOp::Add
        | AstBinaryOp::Sub
        | AstBinaryOp::Mul
        | AstBinaryOp::Div
        | AstBinaryOp::Rem => expected,
        _ => None,
    };
    let mut lowered_lhs = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
        expr: lhs,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected: operand_expected,
    })?;
    let mut lowered_rhs = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
        expr: rhs,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected: operand_expected,
    })?;
    let mut lhs_ty = infer_nir_expr_type(&lowered_lhs, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer binary lhs type".to_owned())?;
    let mut rhs_ty = infer_nir_expr_type(&lowered_rhs, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer binary rhs type".to_owned())?;
    if !compatible_types(&lhs_ty, &rhs_ty) {
        if matches!(lhs, AstExpr::Float(_)) && rhs_ty.is_float_scalar() {
            lowered_lhs = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
                expr: lhs,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: Some(&rhs_ty),
            })?;
            lhs_ty = infer_nir_expr_type(&lowered_lhs, bindings, signatures, struct_table)
                .ok_or_else(|| "cannot infer binary lhs type".to_owned())?;
        }
        if matches!(rhs, AstExpr::Float(_)) && lhs_ty.is_float_scalar() {
            lowered_rhs = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
                expr: rhs,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: Some(&lhs_ty),
            })?;
            rhs_ty = infer_nir_expr_type(&lowered_rhs, bindings, signatures, struct_table)
                .ok_or_else(|| "cannot infer binary rhs type".to_owned())?;
        }
    }
    if let Some(overloaded) = lower_overloaded_binary_operator(
        *op,
        lowered_lhs.clone(),
        lowered_rhs.clone(),
        &lhs_ty,
        &rhs_ty,
        signatures,
    )? {
        return Ok(overloaded);
    }
    let result_ty = binary_result_type(*op, &lhs_ty, &rhs_ty)?;
    if matches!(
        op,
        AstBinaryOp::Add
            | AstBinaryOp::Sub
            | AstBinaryOp::Mul
            | AstBinaryOp::Div
            | AstBinaryOp::Rem
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
            AstBinaryOp::Rem => NirBinaryOp::Rem,
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

fn lower_overloaded_binary_operator(
    op: AstBinaryOp,
    lowered_lhs: NirExpr,
    lowered_rhs: NirExpr,
    lhs_ty: &NirTypeRef,
    rhs_ty: &NirTypeRef,
    signatures: &BTreeMap<String, FunctionSignature>,
) -> Result<Option<NirExpr>, String> {
    let Some((trait_name, method_name)) = overloaded_binary_trait(op) else {
        return Ok(None);
    };
    if builtin_binary_supported(op, lhs_ty, rhs_ty) {
        return Ok(None);
    }
    if !compatible_types(lhs_ty, rhs_ty) {
        return Ok(None);
    }
    let Some(signature) = find_impl_method_signature(signatures, trait_name, lhs_ty, method_name)
    else {
        return Ok(None);
    };
    if signature.params.len() != 2 {
        return Err(format!(
            "trait method `{}.{}` for `{}` expects {} args, found 2",
            trait_name,
            method_name,
            lhs_ty.render(),
            signature.params.len()
        ));
    }
    let call = NirExpr::Call {
        callee: signature.symbol_name.clone(),
        args: vec![lowered_lhs, lowered_rhs],
    };
    Ok(Some(match op {
        AstBinaryOp::Ne => NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(call),
            rhs: Box::new(NirExpr::Bool(false)),
        },
        _ => call,
    }))
}

fn overloaded_binary_trait(op: AstBinaryOp) -> Option<(&'static str, &'static str)> {
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

fn builtin_binary_supported(op: AstBinaryOp, lhs_ty: &NirTypeRef, rhs_ty: &NirTypeRef) -> bool {
    if !compatible_types(lhs_ty, rhs_ty) {
        return false;
    }
    match op {
        AstBinaryOp::And | AstBinaryOp::Or => lhs_ty.is_bool_scalar() && rhs_ty.is_bool_scalar(),
        AstBinaryOp::Add
        | AstBinaryOp::Sub
        | AstBinaryOp::Mul
        | AstBinaryOp::Div
        | AstBinaryOp::Rem => lhs_ty.is_numeric_scalar() && rhs_ty.is_numeric_scalar(),
        AstBinaryOp::Eq | AstBinaryOp::Ne => {
            (lhs_ty.is_integer_scalar() && rhs_ty.is_integer_scalar())
                || (lhs_ty.is_float_scalar() && rhs_ty.is_float_scalar())
                || (lhs_ty.is_bool_scalar() && rhs_ty.is_bool_scalar())
        }
        AstBinaryOp::Lt | AstBinaryOp::Le | AstBinaryOp::Gt | AstBinaryOp::Ge => {
            lhs_ty.is_numeric_scalar() && rhs_ty.is_numeric_scalar()
        }
    }
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
        AstBinaryOp::Add
        | AstBinaryOp::Sub
        | AstBinaryOp::Mul
        | AstBinaryOp::Div
        | AstBinaryOp::Rem => {
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
                || (lhs.is_float_scalar() && rhs.is_float_scalar())
                || (lhs.is_bool_scalar() && rhs.is_bool_scalar()))
            {
                return Err(format!(
                    "binary `{}` currently expects integer, float, or bool scalar operands, found `{}` and `{}`",
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
            if !lhs.is_numeric_scalar() || !rhs.is_numeric_scalar() {
                return Err(format!(
                    "binary `{}` currently expects numeric scalar operands, found `{}` and `{}`",
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
        AstBinaryOp::Rem => "%",
        AstBinaryOp::Eq => "==",
        AstBinaryOp::Ne => "!=",
        AstBinaryOp::Lt => "<",
        AstBinaryOp::Le => "<=",
        AstBinaryOp::Gt => ">",
        AstBinaryOp::Ge => ">=",
    }
}
