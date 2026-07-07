use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    nir_expr_effect_class, AstMatchArm, AstMatchPattern, AstTypeAlias, AstTypeRef, NirBinaryOp,
    NirExpr, NirExprEffectClass, NirStmt, NirStructDef, NirTypeRef,
};

use super::stmt_lowering::{lower_stmt_block_with_async, StmtBlockLoweringInput};
use super::{
    bool_type, compatible_types, infer_nir_expr_type, instantiate_struct_field_type,
    lower_expr_with_async, lower_type_ref, lower_type_ref_with_aliases,
    resolve_ast_type_ref_aliases, ExprWithAsyncInput, FunctionSignature, ModuleConstValue,
};

#[path = "match_lowering_input.rs"]
mod match_lowering_input;
pub(super) use match_lowering_input::MatchStmtLoweringInput;

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
    } else if is_exhaustive_option_or_result_match(arms, &value_ty, type_aliases)? {
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
    } else {
        return Err(
            "minimal `match` currently requires a final unguarded `_` arm unless an `Option` or `Result` match is explicitly exhaustive"
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

fn is_exhaustive_option_or_result_match(
    arms: &[AstMatchArm],
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<bool, String> {
    if arms.is_empty() || arms.iter().any(|arm| arm.guard.is_some()) {
        return Ok(false);
    }

    let mut parent_name: Option<String> = None;
    let mut variants = BTreeSet::new();
    for arm in arms {
        let Some((parent, variant)) =
            exhaustive_option_or_result_variant_name(&arm.pattern, value_ty, type_aliases)?
        else {
            return Ok(false);
        };
        if !matches!(parent.as_str(), "Option" | "Result") {
            return Ok(false);
        }
        if let Some(existing) = parent_name.as_ref() {
            if existing != &parent {
                return Ok(false);
            }
        } else {
            parent_name = Some(parent);
        }
        variants.insert(variant);
    }

    match parent_name.as_deref() {
        Some("Option") => Ok(variants.contains("Some") && variants.contains("None")),
        Some("Result") => Ok(variants.contains("Ok") && variants.contains("Err")),
        _ => Ok(false),
    }
}

fn exhaustive_option_or_result_variant_name(
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
type MatchPatternConditionAndBindings = (NirExpr, Vec<(String, NirTypeRef, NirExpr)>);

fn lower_match_pattern_condition_and_bindings(
    pattern: &AstMatchPattern,
    lowered_value: &NirExpr,
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<MatchPatternConditionAndBindings, String> {
    match (pattern, value_ty.scalar_kind()) {
        (AstMatchPattern::Wildcard, _) => {
            unreachable!("wildcard pattern should be handled before lowering")
        }
        (AstMatchPattern::Bool(true), Some(nuis_semantics::model::NirScalarKind::Bool)) => {
            Ok((lowered_value.clone(), Vec::new()))
        }
        (AstMatchPattern::Bool(false), Some(nuis_semantics::model::NirScalarKind::Bool)) => Ok((
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(lowered_value.clone()),
                rhs: Box::new(NirExpr::Bool(false)),
            },
            Vec::new(),
        )),
        (AstMatchPattern::Int(value), Some(nuis_semantics::model::NirScalarKind::I64)) => Ok((
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(lowered_value.clone()),
                rhs: Box::new(NirExpr::Int(*value)),
            },
            Vec::new(),
        )),
        (
            AstMatchPattern::IntRangeInclusive(start, end),
            Some(nuis_semantics::model::NirScalarKind::I64),
        ) => Ok((
            NirExpr::Binary {
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
            },
            Vec::new(),
        )),
        (AstMatchPattern::Or(patterns), _) => {
            let mut conditions = patterns
                .iter()
                .map(|pattern| {
                    lower_match_pattern_condition_and_bindings(
                        pattern,
                        lowered_value,
                        value_ty,
                        type_aliases,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if conditions.iter().any(|(_, bindings)| !bindings.is_empty()) {
                return Err(
                    "minimal match bindings are currently not supported inside multi-pattern arms"
                        .to_owned(),
                );
            }
            let (first, _) = conditions
                .drain(..1)
                .next()
                .ok_or_else(|| "multi-pattern match arm cannot be empty".to_owned())?;
            Ok((
                conditions
                    .into_iter()
                    .map(|(condition, _)| condition)
                    .fold(first, |lhs, rhs| NirExpr::Binary {
                        op: NirBinaryOp::Or,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                Vec::new(),
            ))
        }
        (AstMatchPattern::PayloadStruct { type_ref, payload }, _) => {
            let resolved_type_ref = resolve_ast_type_ref_aliases(type_ref, type_aliases)?;
            let lowered_pattern_ty =
                lower_pattern_type_for_scrutinee(&resolved_type_ref, value_ty, type_aliases)?;
            if !compatible_types(value_ty, &lowered_pattern_ty)
                && !compatible_types(&lowered_pattern_ty, value_ty)
            {
                return Err(format!(
                    "payload-style struct match pattern `{}` requires scrutinee of type `{}`, found `{}`",
                    lower_type_ref(type_ref).render(),
                    lowered_pattern_ty.render(),
                    value_ty.render()
                ));
            }
            let definition = struct_table.get(&lowered_pattern_ty.name).ok_or_else(|| {
                format!(
                    "payload-style struct match pattern references unknown struct `{}`",
                    lowered_pattern_ty.render()
                )
            })?;
            if definition.fields.len() != 1 {
                return Err(format!(
                    "payload-style struct match pattern `{}` requires a struct with exactly one field",
                    lowered_pattern_ty.render()
                ));
            }
            let field = &definition.fields[0];
            let field_ty =
                instantiate_struct_field_type(&lowered_pattern_ty, definition, &field.ty);
            let variant_condition = NirExpr::VariantIs {
                base: Box::new(lowered_value.clone()),
                variant: lowered_pattern_ty.name.clone(),
            };
            let field_expr = NirExpr::VariantFieldAccess {
                base: Box::new(lowered_value.clone()),
                variant: lowered_pattern_ty.name.clone(),
                field: field.name.clone(),
            };
            let (payload_condition, bindings) = match payload.as_ref() {
                AstMatchPattern::Wildcard => (NirExpr::Bool(true), Vec::new()),
                other => lower_match_pattern_condition_and_bindings(
                    other,
                    &field_expr,
                    &field_ty,
                    type_aliases,
                    struct_table,
                )?,
            };
            Ok((
                match payload_condition {
                    NirExpr::Bool(true) => variant_condition,
                    other => NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs: Box::new(variant_condition),
                        rhs: Box::new(other),
                    },
                },
                bindings,
            ))
        }
        (AstMatchPattern::StructFields { type_ref, fields }, _) => {
            let lowered_pattern_ty = if let Some(type_ref) = type_ref {
                let resolved_type_ref = resolve_ast_type_ref_aliases(type_ref, type_aliases)?;
                let lowered_pattern_ty =
                    lower_pattern_type_for_scrutinee(&resolved_type_ref, value_ty, type_aliases)?;
                if !compatible_types(value_ty, &lowered_pattern_ty)
                    && !compatible_types(&lowered_pattern_ty, value_ty)
                {
                    return Err(format!(
                        "struct match pattern `{}` requires scrutinee of type `{}`, found `{}`",
                        lower_type_ref(type_ref).render(),
                        lowered_pattern_ty.render(),
                        value_ty.render()
                    ));
                }
                lowered_pattern_ty
            } else {
                value_ty.clone()
            };
            let definition = struct_table.get(&lowered_pattern_ty.name).ok_or_else(|| {
                format!(
                    "struct match pattern references unknown struct `{}`",
                    lowered_pattern_ty.render()
                )
            })?;
            let mut conditions = Vec::new();
            let mut bindings = Vec::new();
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
                let field_ty =
                    instantiate_struct_field_type(&lowered_pattern_ty, definition, &field_def.ty);
                let (field_condition, field_bindings) = lower_match_pattern_condition_and_bindings(
                    field_pattern,
                    &field_expr,
                    &field_ty,
                    type_aliases,
                    struct_table,
                )?;
                conditions.push(field_condition);
                bindings.extend(field_bindings);
            }
            if conditions.is_empty() {
                if definition.fields.is_empty() {
                    return Ok((NirExpr::Bool(true), Vec::new()));
                }
                return Err(format!(
                    "empty struct match pattern `{}` is only supported for zero-field structs",
                    lowered_pattern_ty.render()
                ));
            }
            let first = conditions
                .drain(..1)
                .next()
                .ok_or_else(|| "struct match pattern cannot be empty".to_owned())?;
            Ok((
                conditions
                    .into_iter()
                    .fold(first, |lhs, rhs| NirExpr::Binary {
                        op: NirBinaryOp::And,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                bindings,
            ))
        }
        (AstMatchPattern::Bool(_), _) => {
            Err("`match` arm pattern `true`/`false` requires a `bool` scrutinee".to_owned())
        }
        (AstMatchPattern::Int(_) | AstMatchPattern::IntRangeInclusive(_, _), _) => {
            Err("minimal `match` integer patterns require an `i64` scrutinee".to_owned())
        }
        (AstMatchPattern::Bind(name), _) => Ok((
            NirExpr::Bool(true),
            vec![(name.clone(), value_ty.clone(), lowered_value.clone())],
        )),
    }
}

fn lower_pattern_type_for_scrutinee(
    resolved_type_ref: &AstTypeRef,
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<NirTypeRef, String> {
    let lowered = lower_type_ref_with_aliases(resolved_type_ref, type_aliases)?;
    if lowered.name == value_ty.name
        && lowered.generic_args.is_empty()
        && !value_ty.generic_args.is_empty()
        && !lowered.is_optional
        && !lowered.is_ref
    {
        return Ok(value_ty.clone());
    }
    if let Some((parent, _variant)) = lowered.name.rsplit_once('.') {
        if value_ty.name == parent {
            if lowered.generic_args.is_empty() && !value_ty.generic_args.is_empty() {
                return Ok(NirTypeRef {
                    name: lowered.name,
                    generic_args: value_ty.generic_args.clone(),
                    is_optional: lowered.is_optional,
                    is_ref: lowered.is_ref,
                });
            }
            if value_ty.generic_args.len() == lowered.generic_args.len()
                && value_ty
                    .generic_args
                    .iter()
                    .zip(&lowered.generic_args)
                    .all(|(lhs, rhs)| lhs.render() == rhs.render())
            {
                return Ok(lowered);
            }
        }
    }
    Ok(lowered)
}
