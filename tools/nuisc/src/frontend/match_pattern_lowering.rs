use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstMatchPattern, AstTypeAlias, AstTypeRef, NirBinaryOp, NirExpr, NirStructDef, NirTypeRef,
};

use super::super::{
    compatible_types, instantiate_struct_field_type, lower_type_ref, lower_type_ref_with_aliases,
    resolve_ast_type_ref_aliases,
};

pub(super) type MatchPatternConditionAndBindings = (NirExpr, Vec<(String, NirTypeRef, NirExpr)>);

pub(super) fn lower_match_pattern_condition_and_bindings(
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

pub(super) fn lower_pattern_type_for_scrutinee(
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
