use std::collections::BTreeMap;

use nuis_semantics::model::{
    nir_expr_effect_class, AstExpr, AstMatchArm, AstMatchPattern, AstTypeAlias, AstTypeRef,
    NirBinaryOp, NirExpr, NirExprEffectClass, NirStmt, NirStructDef, NirTypeRef,
};

use super::stmt_lowering::lower_stmt_block_with_async;
use super::{
    bool_type, infer_nir_expr_type, lower_expr_with_async, lower_type_ref,
    lower_type_ref_with_aliases, resolve_ast_type_ref_aliases, FunctionSignature, ModuleConstValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_match_stmt_with_async(
    value: &AstExpr,
    arms: &[AstMatchArm],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirStmt, String> {
    if arms.is_empty() {
        return Err("`match` requires at least one arm".to_owned());
    }
    let lowered_value = lower_expr_with_async(
        value,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
        false,
    )?;
    match nir_expr_effect_class(&lowered_value) {
        NirExprEffectClass::Pure | NirExprEffectClass::LocalReadOnly => {}
        _ => {
            return Err(
                "minimal `match` currently requires a pure or local-read-only scrutinee".to_owned(),
            )
        }
    }
    let Some(value_ty) = infer_nir_expr_type(&lowered_value, bindings, signatures, struct_table)
    else {
        return Err("could not infer scrutinee type for `match`".to_owned());
    };
    let wildcard_index = arms
        .iter()
        .position(|arm| matches!(arm.pattern, AstMatchPattern::Wildcard) && arm.guard.is_none())
        .ok_or_else(|| "minimal `match` currently requires a final unguarded `_` arm".to_owned())?;
    if wildcard_index != arms.len() - 1 {
        return Err(
            "minimal `match` currently requires an unguarded `_` to be the final arm".to_owned(),
        );
    }

    let mut wildcard_bindings = bindings.clone();
    let mut else_body = lower_stmt_block_with_async(
        &arms[wildcard_index].body,
        current_domain,
        current_function_is_async,
        &mut wildcard_bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )?;

    for arm in arms[..wildcard_index].iter().rev() {
        let mut condition = lower_match_pattern_condition(
            &arm.pattern,
            &lowered_value,
            &value_ty,
            type_aliases,
            struct_table,
        )?;
        if let Some(guard) = &arm.guard {
            let lowered_guard = lower_expr_with_async(
                guard,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                Some(&bool_type()),
                false,
            )?;
            match nir_expr_effect_class(&lowered_guard) {
                NirExprEffectClass::Pure | NirExprEffectClass::LocalReadOnly => {}
                _ => {
                    return Err(
                        "minimal `match` currently requires a pure or local-read-only guard"
                            .to_owned(),
                    )
                }
            }
            condition = NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(condition),
                rhs: Box::new(lowered_guard),
            };
        }
        let mut then_bindings = bindings.clone();
        let then_body = lower_stmt_block_with_async(
            &arm.body,
            current_domain,
            current_function_is_async,
            &mut then_bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?;
        else_body = vec![NirStmt::If {
            condition,
            then_body,
            else_body,
        }];
    }

    else_body
        .into_iter()
        .next()
        .ok_or_else(|| "internal error: lowered empty `match` body".to_owned())
}

fn lower_match_pattern_condition(
    pattern: &AstMatchPattern,
    lowered_value: &NirExpr,
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    match (pattern, value_ty.scalar_kind()) {
        (AstMatchPattern::Wildcard, _) => {
            unreachable!("wildcard pattern should be handled before lowering")
        }
        (AstMatchPattern::Bool(true), Some(nuis_semantics::model::NirScalarKind::Bool)) => {
            Ok(lowered_value.clone())
        }
        (AstMatchPattern::Bool(false), Some(nuis_semantics::model::NirScalarKind::Bool)) => {
            Ok(NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(lowered_value.clone()),
                rhs: Box::new(NirExpr::Bool(false)),
            })
        }
        (AstMatchPattern::Int(value), Some(nuis_semantics::model::NirScalarKind::I64)) => {
            Ok(NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(lowered_value.clone()),
                rhs: Box::new(NirExpr::Int(*value)),
            })
        }
        (
            AstMatchPattern::IntRangeInclusive(start, end),
            Some(nuis_semantics::model::NirScalarKind::I64),
        ) => Ok(NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Ge,
                lhs: Box::new(lowered_value.clone()),
                rhs: Box::new(NirExpr::Int(*start)),
            }),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Le,
                lhs: Box::new(lowered_value.clone()),
                rhs: Box::new(NirExpr::Int(*end)),
            }),
        }),
        (AstMatchPattern::Or(patterns), _) => {
            let mut conditions = patterns
                .iter()
                .map(|pattern| {
                    lower_match_pattern_condition(
                        pattern,
                        lowered_value,
                        value_ty,
                        type_aliases,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let first = conditions
                .drain(..1)
                .next()
                .ok_or_else(|| "multi-pattern match arm cannot be empty".to_owned())?;
            Ok(conditions
                .into_iter()
                .fold(first, |lhs, rhs| NirExpr::Binary {
                    op: NirBinaryOp::Or,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }))
        }
        (AstMatchPattern::StructFields { type_ref, fields }, _) => {
            let resolved_type_ref = resolve_ast_type_ref_aliases(type_ref, type_aliases)?;
            let lowered_pattern_ty = lower_type_ref_with_aliases(&resolved_type_ref, type_aliases)?;
            if *value_ty != lowered_pattern_ty {
                return Err(format!(
                    "struct match pattern `{}` requires scrutinee of type `{}`, found `{}`",
                    lower_type_ref(type_ref).render(),
                    lowered_pattern_ty.render(),
                    value_ty.render()
                ));
            }
            let definition = struct_table.get(&lowered_pattern_ty.name).ok_or_else(|| {
                format!(
                    "struct match pattern references unknown struct `{}`",
                    lowered_pattern_ty.render()
                )
            })?;
            let mut conditions = Vec::new();
            for (field_name, field_pattern) in fields {
                if matches!(field_pattern, AstMatchPattern::Wildcard) {
                    return Err(format!(
                        "struct match pattern field `{field_name}: _` is not needed; omit the field instead"
                    ));
                }
                let field_def = definition.field(field_name).ok_or_else(|| {
                    format!(
                        "struct match pattern `{}` references unknown field `{}`",
                        lowered_pattern_ty.render(),
                        field_name
                    )
                })?;
                let field_expr = NirExpr::FieldAccess {
                    base: Box::new(lowered_value.clone()),
                    field: field_name.clone(),
                };
                conditions.push(lower_match_pattern_condition(
                    field_pattern,
                    &field_expr,
                    &field_def.ty,
                    type_aliases,
                    struct_table,
                )?);
            }
            let first = conditions
                .drain(..1)
                .next()
                .ok_or_else(|| "struct match pattern cannot be empty".to_owned())?;
            Ok(conditions
                .into_iter()
                .fold(first, |lhs, rhs| NirExpr::Binary {
                    op: NirBinaryOp::And,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }))
        }
        (AstMatchPattern::Bool(_), _) => {
            Err("`match` arm pattern `true`/`false` requires a `bool` scrutinee".to_owned())
        }
        (AstMatchPattern::Int(_) | AstMatchPattern::IntRangeInclusive(_, _), _) => {
            Err("minimal `match` integer patterns require an `i64` scrutinee".to_owned())
        }
    }
}
