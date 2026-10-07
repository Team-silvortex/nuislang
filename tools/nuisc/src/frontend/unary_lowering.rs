use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, AstUnaryOp, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef};

use super::{
    bool_type, compatible_types, f32_type, f64_type, find_impl_method_signature,
    infer_nir_expr_type, lower_nested_expr_with_async_and_consts, FunctionSignature,
    ModuleConstValue, NestedExprWithConstsInput,
};

pub(super) struct UnaryLoweringInput<'a> {
    pub(super) op: &'a AstUnaryOp,
    pub(super) operand: &'a AstExpr,
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
    pub(super) expected: Option<&'a NirTypeRef>,
}

pub(super) fn lower_unary_expr_with_async(
    input: UnaryLoweringInput<'_>,
) -> Result<NirExpr, String> {
    let UnaryLoweringInput {
        op,
        operand,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    } = input;
    let lowered_operand = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
        expr: operand,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    })?;
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
            } else if operand_ty == f32_type() {
                Ok(match lowered_operand {
                    NirExpr::F32(value) => NirExpr::F32(negated_float_literal(value)),
                    operand => NirExpr::UnpackF32Word(Box::new(sign_flipped_word(
                        NirExpr::PackF32Word(Box::new(operand)),
                        1_i64 << 31,
                    ))),
                })
            } else if operand_ty == f64_type() {
                Ok(match lowered_operand {
                    NirExpr::F64(value) => NirExpr::F64(negated_float_literal(value)),
                    operand => NirExpr::UnpackF64Word(Box::new(sign_flipped_word(
                        NirExpr::PackF64Word(Box::new(operand)),
                        i64::MIN,
                    ))),
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
                Err(format!(
                    "unary `*` currently expects `ref Node` operand, found `{}`",
                    operand_ty.render()
                ))
            }
        }
    }
}

fn sign_flipped_word(word: NirExpr, sign: i64) -> NirExpr {
    // Flip only the IEEE sign bit, without floating arithmetic or operand replay.
    NirExpr::Binary {
        op: NirBinaryOp::Xor,
        lhs: Box::new(word),
        rhs: Box::new(NirExpr::Int(sign)),
    }
}

fn negated_float_literal(value: String) -> String {
    // Toggle the sign after typing, preserving signed zero and decimal spelling.
    match value.strip_prefix('-') {
        Some(unsigned) => unsigned.to_owned(),
        None => format!("-{value}"),
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
    let Some(signature) =
        find_impl_method_signature(signatures, trait_name, operand_ty, method_name)
    else {
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
            (operand_ty.name == "i64" && !operand_ty.is_ref && !operand_ty.is_optional)
                || operand_ty == &f32_type()
                || operand_ty == &f64_type()
        }
        AstUnaryOp::Deref => {
            operand_ty.name == "Node" && operand_ty.is_ref && !operand_ty.is_optional
        }
    }
}

#[cfg(test)]
#[test]
fn unary_float_negation_exact_owned_builtin_profile() {
    for ty in [f32_type(), f64_type()] {
        assert!(builtin_unary_supported(AstUnaryOp::Neg, &ty));
        for mutation in ["reference", "optional", "generic"] {
            let mut invalid = ty.clone();
            match mutation {
                "reference" => invalid.is_ref = true,
                "optional" => invalid.is_optional = true,
                "generic" => invalid.generic_args.push(bool_type()),
                _ => unreachable!(),
            }
            assert!(!builtin_unary_supported(AstUnaryOp::Neg, &invalid));
        }
    }
}
