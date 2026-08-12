use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    nir_expr_effect_class, AstMatchArm, AstMatchPattern, AstTypeAlias, NirBinaryOp, NirExpr,
    NirExprEffectClass, NirScalarKind, NirStmt, NirStructDef, NirTypeRef,
};

use super::stmt_lowering::{lower_stmt_block_with_async, StmtBlockLoweringInput};
use super::{
    bool_type, infer_nir_expr_type, lower_expr_with_async, resolve_ast_type_ref_aliases,
    ExprWithAsyncInput, FunctionSignature, ModuleConstValue,
};

#[path = "match_lowering_input.rs"]
mod match_lowering_input;
#[path = "match_pattern_lowering.rs"]
mod match_pattern_lowering;
pub(super) use match_lowering_input::MatchStmtLoweringInput;
use match_pattern_lowering::{
    lower_match_pattern_condition_and_bindings, lower_pattern_type_for_scrutinee,
};

pub(super) fn lower_match_stmt_with_async(
    input: MatchStmtLoweringInput<'_>,
) -> Result<NirStmt, String> {
    let MatchStmtLoweringInput {
        value,
        arms,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
        allow_non_exhaustive_enum_fallthrough,
    } = input;
    if arms.is_empty() {
        return Err("`match` requires at least one arm".to_owned());
    }
    macro_rules! lower_block {
        ($body:expr, $bindings:expr) => {
            lower_stmt_block_with_async(StmtBlockLoweringInput {
                stmts: $body,
                current_domain,
                current_function_is_async,
                bindings: $bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            })
        };
    }
    let lowered_value = lower_expr_with_async(ExprWithAsyncInput {
        expr: value,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected: None,
        allow_async_calls: false,
    })?;
    let Some(value_ty) = infer_nir_expr_type(&lowered_value, bindings, signatures, struct_table)
    else {
        return Err("could not infer scrutinee type for `match`".to_owned());
    };
    let (match_value, hoisted_scrutinee) = match nir_expr_effect_class(&lowered_value) {
        NirExprEffectClass::Pure | NirExprEffectClass::LocalReadOnly => {
            (lowered_value.clone(), None)
        }
        _ => {
            let temp_name = "__nuis_match_scrutinee".to_owned();
            (
                NirExpr::Var(temp_name.clone()),
                Some(NirStmt::Let {
                    name: temp_name,
                    ty: Some(value_ty.clone()),
                    value: lowered_value.clone(),
                }),
            )
        }
    };
    let wildcard_index = arms
        .iter()
        .position(|arm| matches!(arm.pattern, AstMatchPattern::Wildcard) && arm.guard.is_none());

    let (arms_to_lower, mut else_body) = if let Some(wildcard_index) = wildcard_index {
        if wildcard_index != arms.len() - 1 {
            return Err(
                "minimal `match` currently requires an unguarded `_` to be the final arm"
                    .to_owned(),
            );
        }
        let mut wildcard_bindings = bindings.clone();
        let else_body = lower_block!(&arms[wildcard_index].body, &mut wildcard_bindings)?;
        (&arms[..wildcard_index], else_body)
    } else if is_exhaustive_enum_match(arms, &value_ty, type_aliases, struct_table)? {
        let (last_arm, arms_to_lower) = arms
            .split_last()
            .ok_or_else(|| "internal error: exhaustive match has no arms".to_owned())?;
        let (_, pattern_bindings) = lower_match_pattern_condition_and_bindings(
            &last_arm.pattern,
            &match_value,
            &value_ty,
            type_aliases,
            struct_table,
        )?;
        let mut last_bindings = bindings.clone();
        let mut else_body = Vec::new();
        for (name, ty, value) in pattern_bindings {
            last_bindings.insert(name.clone(), ty.clone());
            else_body.push(NirStmt::Let {
                name,
                ty: Some(ty),
                value,
            });
        }
        else_body.extend(lower_block!(&last_arm.body, &mut last_bindings)?);
        (arms_to_lower, else_body)
    } else if allow_non_exhaustive_enum_fallthrough
        && is_non_exhaustive_enum_match(arms, &value_ty, type_aliases, struct_table)?
    {
        (arms, Vec::new())
    } else {
        return Err(
            "minimal `match` currently requires a final unguarded `_` arm unless the match is explicitly exhaustive over all variants"
                .to_owned(),
        );
    };

    for arm in arms_to_lower.iter().rev() {
        let (mut condition, pattern_bindings) = lower_match_pattern_condition_and_bindings(
            &arm.pattern,
            &match_value,
            &value_ty,
            type_aliases,
            struct_table,
        )?;
        if let Some(guard) = &arm.guard {
            let mut guard_bindings = bindings.clone();
            for (name, ty, _) in &pattern_bindings {
                guard_bindings.insert(name.clone(), ty.clone());
            }
            let lowered_guard = lower_expr_with_async(ExprWithAsyncInput {
                expr: guard,
                current_domain,
                current_function_is_async,
                bindings: &guard_bindings,
                module_consts,
                signatures,
                struct_table,
                expected: Some(&bool_type()),
                allow_async_calls: false,
            })?;
            let lowered_guard = substitute_pattern_binding_vars(&lowered_guard, &pattern_bindings);
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
        let mut then_body = Vec::new();
        for (name, ty, value) in pattern_bindings {
            then_bindings.insert(name.clone(), ty.clone());
            then_body.push(NirStmt::Let {
                name,
                ty: Some(ty),
                value,
            });
        }
        then_body.extend(lower_block!(&arm.body, &mut then_bindings)?);
        else_body = vec![NirStmt::If {
            condition,
            then_body,
            else_body,
        }];
    }

    let lowered_match = else_body
        .into_iter()
        .next()
        .ok_or_else(|| "internal error: lowered empty `match` body".to_owned())?;
    if let Some(hoisted_scrutinee) = hoisted_scrutinee {
        Ok(NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![hoisted_scrutinee, lowered_match],
            else_body: Vec::new(),
        })
    } else {
        Ok(lowered_match)
    }
}

fn is_exhaustive_enum_match(
    arms: &[AstMatchArm],
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<bool, String> {
    if value_ty.scalar_kind() == Some(NirScalarKind::Bool) {
        return Ok(match_bool_pattern_coverage(arms, /*require_exhaustive=*/true)?);
    }
    if arms.is_empty() || arms.iter().any(|arm| arm.guard.is_some()) {
        return Ok(false);
    }

    let mut parent_name: Option<String> = None;
    let mut variants = BTreeSet::new();
    for arm in arms {
        let Some((parent, variant)) =
            exhaustive_enum_variant_name(&arm.pattern, value_ty, type_aliases)?
        else {
            return Ok(false);
        };
        if let Some(existing) = parent_name.as_ref() {
            if existing != &parent {
                return Ok(false);
            }
        } else {
            parent_name = Some(parent);
        }
        variants.insert(variant);
    }

    let Some(parent_name) = parent_name else {
        return Ok(false);
    };

    let all_variants = exhaustive_enum_variants(&parent_name, struct_table);
    Ok(variants == all_variants)
}

fn is_non_exhaustive_enum_match(
    arms: &[AstMatchArm],
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<bool, String> {
    if value_ty.scalar_kind() == Some(NirScalarKind::Bool) {
        return Ok(match_bool_pattern_coverage(arms, /*require_exhaustive=*/false)?);
    }
    if arms.is_empty() || arms.iter().any(|arm| arm.guard.is_some()) {
        return Ok(false);
    }

    let mut parent_name: Option<String> = None;
    let mut variants = BTreeSet::new();
    for arm in arms {
        let Some((parent, variant)) =
            exhaustive_enum_variant_name(&arm.pattern, value_ty, type_aliases)?
        else {
            return Ok(false);
        };
        if let Some(existing) = parent_name.as_ref() {
            if existing != &parent {
                return Ok(false);
            }
        } else {
            parent_name = Some(parent);
        }
        variants.insert(variant);
    }

    let Some(parent_name) = parent_name else {
        return Ok(false);
    };
    let all_variants = exhaustive_enum_variants(&parent_name, struct_table);
    if all_variants.is_empty() {
        return Ok(false);
    }

    Ok(variants.len() < all_variants.len())
}

fn exhaustive_enum_variant_name(
    pattern: &AstMatchPattern,
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<Option<(String, String)>, String> {
    let type_ref = match pattern {
        AstMatchPattern::PayloadStruct { type_ref, .. } => type_ref,
        AstMatchPattern::StructFields {
            type_ref: Some(type_ref),
            ..
        } => type_ref,
        _ => return Ok(None),
    };
    let resolved_type_ref = resolve_ast_type_ref_aliases(type_ref, type_aliases)?;
    let lowered_pattern_ty =
        lower_pattern_type_for_scrutinee(&resolved_type_ref, value_ty, type_aliases)?;
    let Some((parent, variant)) = lowered_pattern_ty.name.rsplit_once('.') else {
        return Ok(None);
    };
    Ok(Some((parent.to_owned(), variant.to_owned())))
}

fn exhaustive_enum_variants(
    parent: &str,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> BTreeSet<String> {
    struct_table
        .keys()
        .filter_map(|name| {
            let (enum_name, variant_name) = name.split_once('.')?;
            (enum_name == parent).then(|| variant_name.to_owned())
        })
        .collect()
}

fn match_bool_pattern_coverage(
    arms: &[AstMatchArm],
    require_exhaustive: bool,
) -> Result<bool, String> {
    let mut has_true = false;
    let mut has_false = false;
    for arm in arms {
        if arm.guard.is_some() {
            return Ok(false);
        }
        match arm.pattern {
            AstMatchPattern::Bool(true) => has_true = true,
            AstMatchPattern::Bool(false) => has_false = true,
            _ => return Ok(false),
        }
    }

    if has_true && has_false {
        return Ok(true);
    }
    Ok(!require_exhaustive && (has_true || has_false))
}

fn substitute_pattern_binding_vars(
    expr: &NirExpr,
    pattern_bindings: &[(String, NirTypeRef, NirExpr)],
) -> NirExpr {
    match expr {
        NirExpr::Var(name) => pattern_bindings
            .iter()
            .find(|(binding_name, _, _)| binding_name == name)
            .map(|(_, _, value)| value.clone())
            .unwrap_or_else(|| expr.clone()),
        NirExpr::Await(value) => NirExpr::Await(Box::new(substitute_pattern_binding_vars(
            value,
            pattern_bindings,
        ))),
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(substitute_pattern_binding_vars(base, pattern_bindings)),
            field: field.clone(),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(substitute_pattern_binding_vars(lhs, pattern_bindings)),
            rhs: Box::new(substitute_pattern_binding_vars(rhs, pattern_bindings)),
        },
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args
                .iter()
                .map(|arg| substitute_pattern_binding_vars(arg, pattern_bindings))
                .collect(),
        },
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => NirExpr::MethodCall {
            receiver: Box::new(substitute_pattern_binding_vars(receiver, pattern_bindings)),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| substitute_pattern_binding_vars(arg, pattern_bindings))
                .collect(),
        },
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| {
                    (
                        field.clone(),
                        substitute_pattern_binding_vars(value, pattern_bindings),
                    )
                })
                .collect(),
        },
        _ => expr.clone(),
    }
}
