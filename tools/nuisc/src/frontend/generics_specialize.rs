use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, AstFunction, AstMatchArm, AstMatchPattern, AstParam, AstStmt, AstTypeRef, NirTypeRef,
};

use super::super::types::ast_type_from_nir;

pub(crate) fn specialize_function_template(
    template: &AstFunction,
    specialized_name: &str,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstFunction, String> {
    Ok(AstFunction {
        name: specialized_name.to_owned(),
        visibility: template.visibility,
        attributes: template.attributes.clone(),
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: template.is_async,
        generic_params: vec![],
        where_bounds: vec![],
        params: template
            .params
            .iter()
            .map(|param| {
                Ok(AstParam {
                    name: param.name.clone(),
                    ty: specialize_ast_type_ref(&param.ty, substitutions)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        return_type: template
            .return_type
            .as_ref()
            .map(|ty| specialize_ast_type_ref(ty, substitutions))
            .transpose()?,
        body: specialize_stmt_types(&template.body, substitutions)?,
    })
}

pub(crate) fn specialize_stmt_types(
    body: &[AstStmt],
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<Vec<AstStmt>, String> {
    body.iter()
        .map(|stmt| {
            Ok(match stmt {
                AstStmt::Let {
                    name,
                    ty,
                    value,
                    mutable,
                } => AstStmt::Let {
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty
                        .as_ref()
                        .map(|ty| specialize_ast_type_ref(ty, substitutions))
                        .transpose()?,
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
                    name: name.clone(),
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::DestructureLet {
                    type_ref,
                    fields,
                    value,
                } => AstStmt::DestructureLet {
                    type_ref: type_ref
                        .as_ref()
                        .map(|type_ref| specialize_ast_type_ref(type_ref, substitutions))
                        .transpose()?,
                    fields: fields.clone(),
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::Const { name, ty, value } => AstStmt::Const {
                    name: name.clone(),
                    ty: ty
                        .as_ref()
                        .map(|ty| specialize_ast_type_ref(ty, substitutions))
                        .transpose()?,
                    value: specialize_expr_types(value, substitutions)?,
                },
                AstStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => AstStmt::If {
                    condition: specialize_expr_types(condition, substitutions)?,
                    then_body: specialize_stmt_types(then_body, substitutions)?,
                    else_body: specialize_stmt_types(else_body, substitutions)?,
                },
                AstStmt::Match { value, arms } => AstStmt::Match {
                    value: specialize_expr_types(value, substitutions)?,
                    arms: arms
                        .iter()
                        .map(|arm| {
                            Ok(AstMatchArm {
                                pattern: specialize_match_pattern(&arm.pattern, substitutions)?,
                                guard: arm
                                    .guard
                                    .as_ref()
                                    .map(|guard| specialize_expr_types(guard, substitutions))
                                    .transpose()?,
                                body: specialize_stmt_types(&arm.body, substitutions)?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                },
                AstStmt::While { condition, body } => AstStmt::While {
                    condition: specialize_expr_types(condition, substitutions)?,
                    body: specialize_stmt_types(body, substitutions)?,
                },
                AstStmt::Print(value) => {
                    AstStmt::Print(specialize_expr_types(value, substitutions)?)
                }
                AstStmt::Await(value) => {
                    AstStmt::Await(specialize_expr_types(value, substitutions)?)
                }
                AstStmt::Expr(value) => AstStmt::Expr(specialize_expr_types(value, substitutions)?),
                AstStmt::Return(value) => AstStmt::Return(
                    value
                        .as_ref()
                        .map(|value| specialize_expr_types(value, substitutions))
                        .transpose()?,
                ),
                AstStmt::Break => AstStmt::Break,
                AstStmt::Continue => AstStmt::Continue,
            })
        })
        .collect()
}

fn specialize_expr_types(
    expr: &AstExpr,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(specialize_expr_types(condition, substitutions)?),
            then_body: specialize_stmt_types(then_body, substitutions)?,
            else_body: specialize_stmt_types(else_body, substitutions)?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(specialize_expr_types(value, substitutions)?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: specialize_match_pattern(&arm.pattern, substitutions)?,
                        guard: arm
                            .guard
                            .as_ref()
                            .map(|guard| specialize_expr_types(guard, substitutions))
                            .transpose()?,
                        body: specialize_stmt_types(&arm.body, substitutions)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Await(value) => {
            AstExpr::Await(Box::new(specialize_expr_types(value, substitutions)?))
        }
        AstExpr::Instantiate { domain, unit } => AstExpr::Instantiate {
            domain: domain.clone(),
            unit: unit.clone(),
        },
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => AstExpr::Call {
            callee: callee.clone(),
            generic_args: generic_args
                .iter()
                .map(|arg| specialize_ast_type_ref(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
            args: args
                .iter()
                .map(|arg| specialize_expr_types(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::Invoke { callee, args } => AstExpr::Invoke {
            callee: Box::new(specialize_expr_types(callee, substitutions)?),
            args: args
                .iter()
                .map(|arg| specialize_expr_types(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(specialize_expr_types(receiver, substitutions)?),
            method: method.clone(),
            generic_args: generic_args.clone(),
            args: args
                .iter()
                .map(|arg| specialize_expr_types(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| specialize_ast_type_ref(arg, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
            fields: fields
                .iter()
                .map(|(field, value)| {
                    Ok((field.clone(), specialize_expr_types(value, substitutions)?))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(specialize_expr_types(base, substitutions)?),
            field: field.clone(),
        },
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(specialize_expr_types(operand, substitutions)?),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(specialize_expr_types(lhs, substitutions)?),
            rhs: Box::new(specialize_expr_types(rhs, substitutions)?),
        },
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => AstExpr::Lambda {
            params: params
                .iter()
                .map(|param| {
                    Ok(AstParam {
                        name: param.name.clone(),
                        ty: specialize_ast_type_ref(&param.ty, substitutions)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            return_type: return_type
                .as_ref()
                .map(|ty| specialize_ast_type_ref(ty, substitutions))
                .transpose()?,
            body: specialize_stmt_types(body, substitutions)?,
        },
        other => other.clone(),
    })
}

pub(crate) fn specialize_ast_type_ref(
    ty: &AstTypeRef,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstTypeRef, String> {
    if ty.generic_args.is_empty() {
        if let Some(substitution) = substitutions.get(&ty.name) {
            return Ok(ast_type_from_nir(substitution));
        }
    }
    Ok(AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| specialize_ast_type_ref(arg, substitutions))
            .collect::<Result<Vec<_>, _>>()?,
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    })
}

fn specialize_match_pattern(
    pattern: &AstMatchPattern,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> Result<AstMatchPattern, String> {
    Ok(match pattern {
        AstMatchPattern::Or(patterns) => AstMatchPattern::Or(
            patterns
                .iter()
                .map(|pattern| specialize_match_pattern(pattern, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        AstMatchPattern::PayloadStruct { type_ref, payload } => AstMatchPattern::PayloadStruct {
            type_ref: specialize_ast_type_ref(type_ref, substitutions)?,
            payload: Box::new(specialize_match_pattern(payload, substitutions)?),
        },
        AstMatchPattern::StructFields { type_ref, fields } => AstMatchPattern::StructFields {
            type_ref: type_ref
                .as_ref()
                .map(|type_ref| specialize_ast_type_ref(type_ref, substitutions))
                .transpose()?,
            fields: fields
                .iter()
                .map(|(field, pattern)| {
                    Ok((
                        field.clone(),
                        specialize_match_pattern(pattern, substitutions)?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        other => other.clone(),
    })
}
