use std::collections::BTreeMap;

use nuis_semantics::model::{
    nir_expr_effect_class, AstExpr, AstMatchArm, AstMatchPattern, AstTypeAlias, AstTypeRef,
    NirBinaryOp, NirExpr, NirExprEffectClass, NirStmt, NirStructDef, NirTypeRef,
};

use super::stmt_lowering::lower_stmt_block_with_async;
use super::{
    bool_type, infer_nir_expr_type, instantiate_struct_field_type, lower_expr_with_async,
    lower_type_ref, lower_type_ref_with_aliases, resolve_ast_type_ref_aliases, FunctionSignature,
    ModuleConstValue,
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
        let (mut condition, pattern_bindings) = lower_match_pattern_condition_and_bindings(
            &arm.pattern,
            &lowered_value,
            &value_ty,
            type_aliases,
            struct_table,
        )?;
        if let Some(guard) = &arm.guard {
            let mut guard_bindings = bindings.clone();
            for (name, ty, _) in &pattern_bindings {
                guard_bindings.insert(name.clone(), ty.clone());
            }
            let lowered_guard = lower_expr_with_async(
                guard,
                current_domain,
                current_function_is_async,
                &mut guard_bindings,
                module_consts,
                signatures,
                struct_table,
                Some(&bool_type()),
                false,
            )?;
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
        then_body.extend(lower_stmt_block_with_async(
            &arm.body,
            current_domain,
            current_function_is_async,
            &mut then_bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?);
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

fn lower_match_pattern_condition_and_bindings(
    pattern: &AstMatchPattern,
    lowered_value: &NirExpr,
    value_ty: &NirTypeRef,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(NirExpr, Vec<(String, NirTypeRef, NirExpr)>), String> {
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
            let lowered_pattern_ty = lower_type_ref_with_aliases(&resolved_type_ref, type_aliases)?;
            if *value_ty != lowered_pattern_ty {
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
            let field_expr = NirExpr::FieldAccess {
                base: Box::new(lowered_value.clone()),
                field: field.name.clone(),
            };
            match payload.as_ref() {
                AstMatchPattern::Wildcard => Ok((NirExpr::Bool(true), Vec::new())),
                other => lower_match_pattern_condition_and_bindings(
                    other,
                    &field_expr,
                    &field_ty,
                    type_aliases,
                    struct_table,
                ),
            }
        }
        (AstMatchPattern::StructFields { type_ref, fields }, _) => {
            let lowered_pattern_ty = if let Some(type_ref) = type_ref {
                let resolved_type_ref = resolve_ast_type_ref_aliases(type_ref, type_aliases)?;
                let lowered_pattern_ty =
                    lower_type_ref_with_aliases(&resolved_type_ref, type_aliases)?;
                if *value_ty != lowered_pattern_ty {
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
