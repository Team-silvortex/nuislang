use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstUnaryOp, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef};

use super::{
    bool_type, compatible_types, impl_method_symbol_name, infer_nir_expr_type,
    lower_nested_expr_with_async_and_consts, FunctionSignature, ModuleConstValue,
};

pub(super) fn lower_unary_expr_with_async(
    op: &AstUnaryOp,
    operand: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    let lowered_operand = lower_nested_expr_with_async_and_consts(
        operand,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    )?;
    let operand_ty = infer_nir_expr_type(&lowered_operand, bindings, signatures, struct_table)
        .ok_or_else(|| "cannot infer unary operand type".to_owned())?;
    if let Some(overloaded) =
        lower_overloaded_unary_operator(*op, lowered_operand.clone(), &operand_ty, signatures)?
    {
        return Ok(overloaded);
    }
    match op {
        AstUnaryOp::Not => {
            if operand_ty.is_address_type() {
                Ok(NirExpr::IsNull(Box::new(lowered_operand)))
            } else if !operand_ty.is_bool_scalar() {
                Err(format!(
                    "unary `!` currently expects bool scalar or `ref` address operand, found `{}`",
                    operand_ty.render()
                ))
            } else {
                Ok(NirExpr::Binary {
                    op: NirBinaryOp::Eq,
                    lhs: Box::new(lowered_operand),
                    rhs: Box::new(NirExpr::Bool(false)),
                })
            }
        }
        AstUnaryOp::Neg => {
            if operand_ty.name == "i64" && !operand_ty.is_ref && !operand_ty.is_optional {
                Ok(NirExpr::Binary {
                    op: NirBinaryOp::Sub,
                    lhs: Box::new(NirExpr::Int(0)),
                    rhs: Box::new(lowered_operand),
                })
            } else if operand_ty.name == "f32" && !operand_ty.is_ref && !operand_ty.is_optional {
                Ok(NirExpr::Binary {
                    op: NirBinaryOp::Sub,
                    lhs: Box::new(NirExpr::F32("0.0".to_owned())),
                    rhs: Box::new(lowered_operand),
                })
            } else if operand_ty.name == "f64" && !operand_ty.is_ref && !operand_ty.is_optional {
                Ok(NirExpr::Binary {
                    op: NirBinaryOp::Sub,
                    lhs: Box::new(NirExpr::F64("0.0".to_owned())),
                    rhs: Box::new(lowered_operand),
                })
            } else {
                Err(format!(
                    "unary `-` currently expects numeric scalar operand, found `{}`",
                    operand_ty.render()
                ))
            }
        }
        AstUnaryOp::Deref => {
            if operand_ty.name == "Node" && operand_ty.is_ref && !operand_ty.is_optional {
                Ok(NirExpr::LoadValue(Box::new(lowered_operand)))
            } else {
                return Err(format!(
                    "unary `*` currently expects `ref Node` operand, found `{}`",
                    operand_ty.render()
                ));
            }
        }
    }
}

fn lower_overloaded_unary_operator(
    op: AstUnaryOp,
    lowered_operand: NirExpr,
    operand_ty: &NirTypeRef,
    signatures: &BTreeMap<String, FunctionSignature>,
) -> Result<Option<NirExpr>, String> {
    let Some((trait_name, method_name)) = overloaded_unary_trait(op) else {
        return Ok(None);
    };
    if builtin_unary_supported(op, operand_ty) {
        return Ok(None);
    }
    let symbol_name = impl_method_symbol_name(trait_name, operand_ty, method_name);
    let Some(signature) = signatures.get(&symbol_name) else {
        return Ok(None);
    };
    if signature.params.len() != 1 {
        return Err(format!(
            "trait method `{}.{}` for `{}` expects {} args, found 1",
            trait_name,
            method_name,
            operand_ty.render(),
            signature.params.len()
        ));
    }
    if let Some(return_type) = &signature.return_type {
        match op {
            AstUnaryOp::Not if !compatible_types(return_type, &bool_type()) => {
                return Err(format!(
                    "trait method `{}.{}` for `{}` must return `bool`, found `{}`",
                    trait_name,
                    method_name,
                    operand_ty.render(),
                    return_type.render()
                ));
            }
            AstUnaryOp::Neg if !compatible_types(return_type, operand_ty) => {
                return Err(format!(
                    "trait method `{}.{}` for `{}` must return `{}`, found `{}`",
                    trait_name,
                    method_name,
                    operand_ty.render(),
                    operand_ty.render(),
                    return_type.render()
                ));
            }
            _ => {}
        }
    }
    Ok(Some(NirExpr::Call {
        callee: signature.symbol_name.clone(),
        args: vec![lowered_operand],
    }))
}

fn overloaded_unary_trait(op: AstUnaryOp) -> Option<(&'static str, &'static str)> {
    match op {
        AstUnaryOp::Not => Some(("Notable", "not")),
        AstUnaryOp::Neg => Some(("Negatable", "neg")),
        AstUnaryOp::Deref => None,
    }
}

fn builtin_unary_supported(op: AstUnaryOp, operand_ty: &NirTypeRef) -> bool {
    match op {
        AstUnaryOp::Not => operand_ty.is_bool_scalar() || operand_ty.is_address_type(),
        AstUnaryOp::Neg => {
            (operand_ty.name == "i64" || operand_ty.name == "f32" || operand_ty.name == "f64")
                && !operand_ty.is_ref
                && !operand_ty.is_optional
        }
        AstUnaryOp::Deref => operand_ty.name == "Node" && operand_ty.is_ref && !operand_ty.is_optional,
    }
}
