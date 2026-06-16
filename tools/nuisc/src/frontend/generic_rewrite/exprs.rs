use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstImplDef, AstMatchArm, AstStructDef, AstTypeAlias,
    AstTypeRef, AstUnaryOp,
};

use super::super::generics::{
    infer_alias_aware_ast_expr_type, infer_generic_substitutions, specialize_ast_type_ref,
    specialize_function_template, unify_generic_type_pattern,
};
use super::super::types::{ast_type_from_nir, infer_ast_expr_type_for_pattern};
use super::super::{
    lower_type_ref, lower_type_ref_with_aliases, resolve_ast_type_ref_aliases,
    substitute_ast_type_alias_target, FunctionSignature,
};
use crate::frontend::generic_rewrite::rewrite_generic_calls_in_function;
use crate::frontend::higher_order::rewrite_higher_order_calls_in_function;
use super::GenericImplMethodTemplate;

pub(super) fn rewrite_generic_calls_in_expr(
    expr: &AstExpr,
    context: &str,
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            AstExpr::If {
                condition: Box::new(rewrite_generic_calls_in_expr(
                    condition,
                    context,
                    Some(&super::super::ast_named_type("bool")),
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?),
                then_body: super::blocks::rewrite_generic_calls_in_block(
                    then_body,
                    &format!("{context} if-then"),
                    expected,
                    &mut then_env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?,
                else_body: super::blocks::rewrite_generic_calls_in_block(
                    else_body,
                    &format!("{context} if-else"),
                    expected,
                    &mut else_env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?,
            }
        }
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_generic_calls_in_expr(
                value,
                context,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?),
            arms: arms
                .iter()
                .map(|arm| {
                    let mut arm_env = env.clone();
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: match &arm.guard {
                            Some(guard) => Some(rewrite_generic_calls_in_expr(
                                guard,
                                context,
                                Some(&super::super::ast_named_type("bool")),
                                &arm_env,
                                visible_type_aliases,
                                generic_templates,
                                generic_impl_method_templates,
                                higher_order_templates,
                                function_table,
                                signatures,
                                impl_lookup,
                                struct_table,
                                function_return_types,
                                specialization_cache,
                                specialized_functions,
                                specialized_signatures,
                            )?),
                            None => None,
                        },
                        body: super::blocks::rewrite_generic_calls_in_block(
                            &arm.body,
                            &format!("{context} match-arm"),
                            expected,
                            &mut arm_env,
                            visible_type_aliases,
                            generic_templates,
                            generic_impl_method_templates,
                            higher_order_templates,
                            function_table,
                            signatures,
                            impl_lookup,
                            struct_table,
                            function_return_types,
                            specialization_cache,
                            specialized_functions,
                            specialized_signatures,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_generic_calls_in_expr(
            value,
            context,
            expected,
            env,
            visible_type_aliases,
            generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?)),
        AstExpr::Unary { op, operand } => {
            let rewritten_operand = rewrite_generic_calls_in_expr(
                operand,
                context,
                expected,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            if let Some((trait_name, method_name)) = overloaded_unary_trait(*op) {
                if let Some(operand_ty) = infer_alias_aware_ast_expr_type(
                    &rewritten_operand,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok())
                {
                    if !builtin_unary_supported_ast(*op, &operand_ty) {
                        let call_args = vec![rewritten_operand.clone()];
                        if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                            Some(trait_name),
                            method_name,
                            &call_args,
                            expected,
                            env,
                            visible_type_aliases,
                            generic_templates,
                            generic_impl_method_templates,
                            higher_order_templates,
                            function_table,
                            signatures,
                            impl_lookup,
                            struct_table,
                            function_return_types,
                            specialization_cache,
                            specialized_functions,
                            specialized_signatures,
                        )? {
                            return Ok(AstExpr::Call {
                                callee: specialized_name,
                                generic_args: Vec::new(),
                                args: call_args,
                            });
                        }
                    }
                }
            }
            AstExpr::Unary {
                op: *op,
                operand: Box::new(rewritten_operand),
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            let rewritten_generic_args = generic_args
                .iter()
                .map(|arg| resolve_ast_type_ref_aliases(arg, visible_type_aliases))
                .collect::<Result<Vec<_>, _>>()?;
            let rewritten_args = args
                .iter()
                .enumerate()
                .map(|(index, arg)| {
                    let arg_expected = call_arg_expected_type(
                        callee,
                        &rewritten_generic_args,
                        index,
                        expected,
                        generic_templates,
                        signatures,
                        visible_type_aliases,
                        struct_table,
                    );
                    rewrite_generic_calls_in_expr(
                        arg,
                        context,
                        arg_expected.as_ref(),
                        env,
                        visible_type_aliases,
                        generic_templates,
                        generic_impl_method_templates,
                        higher_order_templates,
                        function_table,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some((trait_name, method_name)) = callee.rsplit_once('.') {
                if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                    Some(trait_name),
                    method_name,
                    &rewritten_args,
                    expected,
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )? {
                    return Ok(AstExpr::Call {
                        callee: specialized_name,
                        generic_args: Vec::new(),
                        args: rewritten_args,
                    });
                }
            }
            if let Some(template) = generic_templates.get(callee) {
                let specialized_name = ensure_generic_specialization(
                    template,
                    &rewritten_generic_args,
                    &rewritten_args,
                    expected,
                    &format!("{context} call `{callee}`"),
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?;
                AstExpr::Call {
                    callee: specialized_name,
                    generic_args: Vec::new(),
                    args: rewritten_args,
                }
            } else {
                let rewritten_callee = resolved_struct_constructor_alias(
                    callee,
                    &rewritten_generic_args,
                    expected,
                    &rewritten_args,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )?
                .unwrap_or_else(|| (callee.clone(), rewritten_generic_args.clone()));
                AstExpr::Call {
                    callee: rewritten_callee.0,
                    generic_args: rewritten_callee.1,
                    args: rewritten_args,
                }
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            let explicit_receiver_expected = super::super::receiver_expected::explicit_receiver_expected_type(
                receiver,
                generic_args,
                visible_type_aliases,
            );
            let rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_generic_calls_in_expr(
                        arg,
                        context,
                        None,
                        env,
                        visible_type_aliases,
                        generic_templates,
                        generic_impl_method_templates,
                        higher_order_templates,
                        function_table,
                        signatures,
                        impl_lookup,
                        struct_table,
                        function_return_types,
                        specialization_cache,
                        specialized_functions,
                        specialized_signatures,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let receiver_expected = method_call_receiver_expected_type(
                receiver,
                method,
                generic_args,
                &rewritten_args,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            );
            let rewritten_receiver = rewrite_generic_calls_in_expr(
                receiver,
                context,
                receiver_expected.as_ref(),
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let rewritten_receiver = super::super::receiver_expected::specialize_receiver_constructor_from_expected(
                &rewritten_receiver,
                explicit_receiver_expected.as_ref(),
                visible_type_aliases,
                struct_table,
            );
            let mut call_args = vec![rewritten_receiver.clone()];
            call_args.extend(rewritten_args.clone());
            if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                None,
                method,
                &call_args,
                expected,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )? {
                AstExpr::Call {
                    callee: specialized_name,
                    generic_args: Vec::new(),
                    args: call_args,
                }
            } else {
                if let Some(explicit_receiver_expected) = receiver_expected.as_ref() {
                    if let Some(specialized_name) =
                        ensure_generic_impl_method_specialization_from_receiver_expected(
                            method,
                            explicit_receiver_expected,
                            &call_args,
                            expected,
                            env,
                            visible_type_aliases,
                            generic_templates,
                            generic_impl_method_templates,
                            higher_order_templates,
                            function_table,
                            signatures,
                            impl_lookup,
                            struct_table,
                            function_return_types,
                            specialization_cache,
                            specialized_functions,
                            specialized_signatures,
                        )?
                    {
                        return Ok(AstExpr::Call {
                            callee: specialized_name,
                            generic_args: Vec::new(),
                            args: call_args,
                        });
                    }
                }
                AstExpr::MethodCall {
                    receiver: Box::new(rewritten_receiver),
                    method: method.clone(),
                    generic_args: generic_args.clone(),
                    args: rewritten_args,
                }
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let rewritten_head = resolved_struct_literal_alias(
                type_name,
                type_args,
                expected,
                fields,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            )?
            .unwrap_or_else(|| (type_name.clone(), type_args.clone()));
            let concrete_literal_ty = concrete_struct_literal_type(
                &rewritten_head.0,
                &rewritten_head.1,
                expected,
                visible_type_aliases,
            );
            let final_name = concrete_literal_ty
                .as_ref()
                .map(|ty| ty.name.clone())
                .unwrap_or_else(|| rewritten_head.0.clone());
            let final_args = concrete_literal_ty
                .as_ref()
                .map(|ty| ty.generic_args.clone())
                .unwrap_or_else(|| rewritten_head.1.clone());
            AstExpr::StructLiteral {
                type_name: final_name.clone(),
                type_args: final_args.clone(),
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        let literal_ty = concrete_literal_ty
                        .clone()
                        .unwrap_or_else(|| AstTypeRef {
                            name: final_name.clone(),
                            generic_args: final_args.clone(),
                            is_optional: false,
                            is_ref: false,
                        });
                        let field_expected =
                            struct_field_expected_type(&literal_ty, name, struct_table);
                        Ok((
                            name.clone(),
                            rewrite_generic_calls_in_expr(
                                value,
                                context,
                                field_expected.as_ref(),
                                env,
                                visible_type_aliases,
                                generic_templates,
                                generic_impl_method_templates,
                                higher_order_templates,
                                function_table,
                                signatures,
                                impl_lookup,
                                struct_table,
                                function_return_types,
                                specialization_cache,
                                specialized_functions,
                                specialized_signatures,
                            )?,
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }
        }
        AstExpr::FieldAccess { base, field } => {
            let base_expected = field_access_base_expected_type(
                expected,
                field,
                visible_type_aliases,
                struct_table,
            );
            AstExpr::FieldAccess {
                base: Box::new(rewrite_generic_calls_in_expr(
                    base,
                    context,
                    base_expected.as_ref(),
                    env,
                    visible_type_aliases,
                    generic_templates,
                    generic_impl_method_templates,
                    higher_order_templates,
                    function_table,
                    signatures,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                    specialization_cache,
                    specialized_functions,
                    specialized_signatures,
                )?),
                field: field.clone(),
            }
        }
        AstExpr::Binary { op, lhs, rhs } => {
            let rewritten_lhs = rewrite_generic_calls_in_expr(
                lhs,
                context,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            let rewritten_rhs = rewrite_generic_calls_in_expr(
                rhs,
                context,
                None,
                env,
                visible_type_aliases,
                generic_templates,
                generic_impl_method_templates,
                higher_order_templates,
                function_table,
                signatures,
                impl_lookup,
                struct_table,
                function_return_types,
                specialization_cache,
                specialized_functions,
                specialized_signatures,
            )?;
            if let Some((trait_name, method_name)) = overloaded_binary_trait(*op) {
                let lhs_ty = infer_alias_aware_ast_expr_type(
                    &rewritten_lhs,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok());
                let rhs_ty = infer_alias_aware_ast_expr_type(
                    &rewritten_rhs,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
                .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok());
                if let (Some(lhs_ty), Some(rhs_ty)) = (lhs_ty, rhs_ty) {
                    if !builtin_binary_supported_ast(*op, &lhs_ty, &rhs_ty)
                        && lower_type_ref(&lhs_ty).render() == lower_type_ref(&rhs_ty).render()
                    {
                        let call_args = vec![rewritten_lhs.clone(), rewritten_rhs.clone()];
                        if let Some(specialized_name) = ensure_generic_impl_method_specialization(
                            Some(trait_name),
                            method_name,
                            &call_args,
                            expected,
                            env,
                            visible_type_aliases,
                            generic_templates,
                            generic_impl_method_templates,
                            higher_order_templates,
                            function_table,
                            signatures,
                            impl_lookup,
                            struct_table,
                            function_return_types,
                            specialization_cache,
                            specialized_functions,
                            specialized_signatures,
                        )? {
                            let call = AstExpr::Call {
                                callee: specialized_name,
                                generic_args: Vec::new(),
                                args: call_args,
                            };
                            return Ok(match op {
                                AstBinaryOp::Ne => AstExpr::Binary {
                                    op: AstBinaryOp::Eq,
                                    lhs: Box::new(call),
                                    rhs: Box::new(AstExpr::Bool(false)),
                                },
                                _ => call,
                            });
                        }
                    }
                }
            }
            AstExpr::Binary {
                op: *op,
                lhs: Box::new(rewritten_lhs),
                rhs: Box::new(rewritten_rhs),
            }
        }
        other => other.clone(),
    })
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

fn overloaded_unary_trait(op: AstUnaryOp) -> Option<(&'static str, &'static str)> {
    match op {
        AstUnaryOp::Not => Some(("Notable", "not")),
        AstUnaryOp::Neg => Some(("Negatable", "neg")),
        AstUnaryOp::Deref => None,
    }
}

fn builtin_binary_supported_ast(op: AstBinaryOp, lhs_ty: &AstTypeRef, rhs_ty: &AstTypeRef) -> bool {
    let same = lower_type_ref(lhs_ty).render() == lower_type_ref(rhs_ty).render();
    if !same {
        return false;
    }
    match op {
        AstBinaryOp::And | AstBinaryOp::Or => is_plain_scalar(lhs_ty, "bool"),
        AstBinaryOp::Add
        | AstBinaryOp::Sub
        | AstBinaryOp::Mul
        | AstBinaryOp::Div
        | AstBinaryOp::Rem => is_plain_numeric_scalar(lhs_ty),
        AstBinaryOp::Eq | AstBinaryOp::Ne => {
            is_plain_integer_scalar(lhs_ty) || is_plain_float_scalar(lhs_ty) || is_plain_scalar(lhs_ty, "bool")
        }
        AstBinaryOp::Lt | AstBinaryOp::Le | AstBinaryOp::Gt | AstBinaryOp::Ge => {
            is_plain_numeric_scalar(lhs_ty)
        }
    }
}

fn builtin_unary_supported_ast(op: AstUnaryOp, operand_ty: &AstTypeRef) -> bool {
    match op {
        AstUnaryOp::Not => is_plain_scalar(operand_ty, "bool") || is_ref_type(operand_ty),
        AstUnaryOp::Neg => {
            is_plain_scalar(operand_ty, "i64")
                || is_plain_scalar(operand_ty, "f32")
                || is_plain_scalar(operand_ty, "f64")
        }
        AstUnaryOp::Deref => operand_ty.name == "Node" && operand_ty.is_ref && !operand_ty.is_optional,
    }
}

fn is_plain_scalar(ty: &AstTypeRef, name: &str) -> bool {
    ty.name == name && !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty()
}

fn is_plain_integer_scalar(ty: &AstTypeRef) -> bool {
    is_plain_scalar(ty, "i32") || is_plain_scalar(ty, "i64")
}

fn is_plain_float_scalar(ty: &AstTypeRef) -> bool {
    is_plain_scalar(ty, "f32") || is_plain_scalar(ty, "f64")
}

fn is_plain_numeric_scalar(ty: &AstTypeRef) -> bool {
    is_plain_integer_scalar(ty) || is_plain_float_scalar(ty)
}

fn is_ref_type(ty: &AstTypeRef) -> bool {
    ty.is_ref && !ty.is_optional
}

fn concrete_struct_literal_type(
    rewritten_name: &str,
    rewritten_args: &[AstTypeRef],
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    if expected.name == rewritten_name {
        return Some(expected.clone());
    }
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    if resolved_expected.name == rewritten_name {
        return Some(resolved_expected);
    }
    if !rewritten_args.is_empty() {
        return Some(AstTypeRef {
            name: rewritten_name.to_owned(),
            generic_args: rewritten_args.to_vec(),
            is_optional: false,
            is_ref: false,
        });
    }
    None
}

fn resolved_struct_constructor_alias(
    callee: &str,
    generic_args: &[AstTypeRef],
    expected: Option<&AstTypeRef>,
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    if generic_args.is_empty() {
        if let Some(from_expected) = infer_alias_struct_target_from_expected(
            callee,
            expected,
            visible_type_aliases,
            struct_table,
        )? {
            return Ok(Some(from_expected));
        }
    }
    let type_ref = AstTypeRef {
        name: callee.to_owned(),
        generic_args: generic_args.to_vec(),
        is_optional: false,
        is_ref: false,
    };
    let resolved = match resolve_ast_type_ref_aliases(&type_ref, visible_type_aliases) {
        Ok(resolved) => resolved,
        Err(error) => {
            if generic_args.is_empty() {
                if let Some(inferred) = infer_alias_struct_constructor_type_from_args(
                    callee,
                    args,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )? {
                    return Ok(Some(inferred));
                }
            }
            if visible_type_aliases.contains_key(callee) {
                return Err(error);
            }
            return Ok(None);
        }
    };
    Ok(struct_table
        .contains_key(&resolved.name)
        .then_some((resolved.name, resolved.generic_args)))
}

fn resolved_struct_literal_alias(
    type_name: &str,
    type_args: &[AstTypeRef],
    expected: Option<&AstTypeRef>,
    fields: &[(String, AstExpr)],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    if type_args.is_empty() {
        if let Some(from_expected) = infer_alias_struct_target_from_expected(
            type_name,
            expected,
            visible_type_aliases,
            struct_table,
        )? {
            return Ok(Some(from_expected));
        }
    }
    let type_ref = AstTypeRef {
        name: type_name.to_owned(),
        generic_args: type_args.to_vec(),
        is_optional: false,
        is_ref: false,
    };
    let resolved = match resolve_ast_type_ref_aliases(&type_ref, visible_type_aliases) {
        Ok(resolved) => resolved,
        Err(error) => {
            if type_args.is_empty() {
                if let Some(inferred) = infer_alias_struct_literal_type_from_fields(
                    type_name,
                    fields,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )? {
                    return Ok(Some(inferred));
                }
            }
            if visible_type_aliases.contains_key(type_name) {
                return Err(error);
            }
            return Ok(None);
        }
    };
    Ok(struct_table
        .contains_key(&resolved.name)
        .then_some((resolved.name, resolved.generic_args)))
}

fn method_call_receiver_expected_type(
    receiver: &AstExpr,
    method: &str,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    if let Some(explicit) =
        super::super::receiver_expected::explicit_receiver_expected_type(
            receiver,
            generic_args,
            visible_type_aliases,
        )
    {
        return Some(explicit);
    }

    let arg_types = args
        .iter()
        .map(|arg| {
            infer_alias_aware_ast_expr_type(
                arg,
                env,
                visible_type_aliases,
                impl_lookup,
                struct_table,
                function_return_types,
            )
            .and_then(|ty| resolve_ast_type_ref_aliases(&ty, visible_type_aliases).ok())
        })
        .collect::<Option<Vec<_>>>()?;

    let mut candidates =
        impl_lookup
            .values()
            .filter_map(|definition| {
                let method_def = definition
                    .methods
                    .iter()
                    .find(|item| item.name == *method)?;
                if method_def.params.len() != arg_types.len() + 1 {
                    return None;
                }
                let params_match = method_def.params.iter().skip(1).zip(arg_types.iter()).all(
                    |(param, arg_ty)| {
                        resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)
                            .ok()
                            .is_some_and(|resolved| {
                                lower_type_ref(&resolved).render()
                                    == lower_type_ref(arg_ty).render()
                            })
                    },
                );
                params_match.then(|| definition.for_type.clone())
            })
            .collect::<Vec<_>>();
    if candidates.len() == 1 {
        let candidate = candidates.pop()?;
        if !generic_args.is_empty() && candidate.generic_args.len() == generic_args.len() {
            return Some(AstTypeRef {
                name: candidate.name,
                generic_args: generic_args.to_vec(),
                is_optional: candidate.is_optional,
                is_ref: candidate.is_ref,
            });
        }
        return Some(candidate);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_constructor_type_from_args(
    alias_name: &str,
    args: &[AstExpr],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    if alias_definition.generic_params.is_empty() {
        return Ok(None);
    }
    let resolved_target_pattern =
        resolve_ast_type_ref_aliases(&alias_definition.target, visible_type_aliases)?;
    let Some(target_definition) = struct_table.get(&resolved_target_pattern.name) else {
        return Err(format!(
            "payload-style alias constructor `{alias_name}(...)` is not yet supported for generic alias target `{}`; current frontend only supports aliases that resolve to struct targets",
            lower_type_ref(&alias_definition.target).render()
        ));
    };
    if target_definition.fields.len() != 1 || args.len() != 1 {
        return Ok(None);
    }
    let field_pattern = super::super::validation_binding_env::instantiate_ast_struct_field_type(
        &resolved_target_pattern,
        target_definition,
        &target_definition.fields[0].ty,
    );
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_alias_struct_target_from_usage_seeded(
        alias_name,
        alias_definition,
        &[field_pattern],
        &[args[0].clone()],
        visible_type_aliases,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &generic_names,
        BTreeMap::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_literal_type_from_fields(
    alias_name: &str,
    fields: &[(String, AstExpr)],
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    if alias_definition.generic_params.is_empty() {
        return Ok(None);
    }
    let resolved_target_pattern =
        resolve_ast_type_ref_aliases(&alias_definition.target, visible_type_aliases)?;
    let Some(target_definition) = struct_table.get(&resolved_target_pattern.name) else {
        return Err(format!(
            "struct literal alias `{alias_name}` is not yet supported for generic alias target `{}`; current frontend only supports aliases that resolve to struct targets",
            lower_type_ref(&alias_definition.target).render()
        ));
    };
    let mut field_patterns = Vec::new();
    for (name, _value) in fields {
        let Some(field) = target_definition
            .fields
            .iter()
            .find(|field| field.name == *name)
        else {
            return Ok(None);
        };
        let pattern = super::super::validation_binding_env::instantiate_ast_struct_field_type(
            &resolved_target_pattern,
            target_definition,
            &field.ty,
        );
        field_patterns.push(pattern);
    }
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_alias_struct_target_from_usage_seeded(
        alias_name,
        alias_definition,
        &field_patterns,
        &fields.iter().map(|(_, value)| value.clone()).collect::<Vec<_>>(),
        visible_type_aliases,
        env,
        impl_lookup,
        struct_table,
        function_return_types,
        &generic_names,
        BTreeMap::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn infer_alias_struct_target_from_usage_seeded(
    alias_name: &str,
    alias_definition: &AstTypeAlias,
    patterns: &[AstTypeRef],
    concrete_exprs: &[AstExpr],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_names: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, AstTypeRef>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let mut pending = patterns
        .iter()
        .zip(concrete_exprs.iter())
        .map(|(pattern, expr)| (pattern, expr))
        .collect::<Vec<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut next_pending = Vec::new();
        for (pattern, concrete_expr) in pending {
            let expected_pattern =
                specialize_ast_type_pattern_with_known_substitutions(pattern, &substitutions);
            let concrete = infer_ast_expr_type_for_pattern(
                concrete_expr,
                &expected_pattern,
                generic_names,
                env,
                impl_lookup,
                struct_table,
                function_return_types,
            )
            .or_else(|| {
                infer_alias_aware_ast_expr_type(
                    concrete_expr,
                    env,
                    visible_type_aliases,
                    impl_lookup,
                    struct_table,
                    function_return_types,
                )
            });
            let Some(concrete) = concrete else {
                next_pending.push((pattern, concrete_expr));
                continue;
            };
            if let Err(error) = unify_generic_type_pattern(
                pattern,
                &concrete,
                generic_names,
                &mut substitutions,
                alias_name,
            ) {
                if error.contains("resolved to conflicting types") {
                    return Err(format!(
                        "generic alias constructor `{alias_name}` inferred conflicting types while matching target `{}`",
                        lower_type_ref(&alias_definition.target).render()
                    ));
                }
                return Err(format!(
                    "generic alias constructor `{alias_name}` could not match target field shape `{}` with concrete type `{}`",
                    lower_type_ref(pattern).render(),
                    lower_type_ref(&concrete).render()
                ));
            }
            progress = true;
        }
        if !progress {
            return Err(format!(
                "generic alias constructor `{alias_name}` could not be inferred from current field/payload usage for target `{}`; add explicit type arguments or a stronger expected type",
                lower_type_ref(&alias_definition.target).render()
            ));
        }
        pending = next_pending;
    }
    let mut generic_args = Vec::new();
    for param in &alias_definition.generic_params {
        let Some(argument) = substitutions.get(&param.name).cloned() else {
            return Err(format!(
                "generic alias constructor `{alias_name}` could not infer generic parameter `{}` for target `{}`; add explicit type arguments or a stronger expected type",
                param.name,
                lower_type_ref(&alias_definition.target).render()
            ));
        };
        generic_args.push(argument);
    }
    let substituted = substitute_ast_type_alias_target(
        &alias_definition.target,
        &alias_definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .zip(generic_args)
            .collect::<BTreeMap<_, _>>(),
    )?;
    let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases)?;
    Ok(struct_table
        .contains_key(&resolved.name)
        .then_some((resolved.name, resolved.generic_args)))
}

fn specialize_ast_type_pattern_with_known_substitutions(
    pattern: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if pattern.generic_args.is_empty()
        && !pattern.is_optional
        && !pattern.is_ref
        && substitutions.contains_key(&pattern.name)
    {
        return substitutions
            .get(&pattern.name)
            .cloned()
            .unwrap_or_else(|| pattern.clone());
    }
    AstTypeRef {
        name: pattern.name.clone(),
        generic_args: pattern
            .generic_args
            .iter()
            .map(|arg| specialize_ast_type_pattern_with_known_substitutions(arg, substitutions))
            .collect(),
        is_optional: pattern.is_optional,
        is_ref: pattern.is_ref,
    }
}

fn infer_alias_struct_target_from_expected(
    alias_name: &str,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Result<Option<(String, Vec<AstTypeRef>)>, String> {
    let Some(expected) = expected else {
        return Ok(None);
    };
    let Some(alias_definition) = visible_type_aliases.get(alias_name) else {
        return Ok(None);
    };
    if alias_definition.generic_params.is_empty() {
        return Ok(None);
    }
    let resolved_target_pattern =
        resolve_ast_type_ref_aliases(&alias_definition.target, visible_type_aliases)?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases)?;
    let generic_names = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    if unify_generic_type_pattern(
        &resolved_target_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        alias_name,
    )
    .is_err()
    {
        return Ok(None);
    }
    let generic_args = alias_definition
        .generic_params
        .iter()
        .map(|param| substitutions.get(&param.name).cloned())
        .collect::<Option<Vec<_>>>();
    let Some(generic_args) = generic_args else {
        return Ok(None);
    };
    if generic_args
        .iter()
        .any(|arg| contains_ast_placeholder_generic_name(arg, &generic_names))
    {
        return Ok(None);
    }
    let generic_args = generic_args
        .into_iter()
        .collect::<Vec<_>>();
    let substituted = substitute_ast_type_alias_target(
        &alias_definition.target,
        &alias_definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .zip(generic_args.clone())
            .collect::<BTreeMap<_, _>>(),
    )?;
    let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases)?;
    if !struct_table.contains_key(&resolved.name) {
        return Ok(None);
    }
    Ok(Some((resolved.name, resolved.generic_args)))
}

fn contains_ast_placeholder_generic_name(
    ty: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_ast_placeholder_generic_name(arg, placeholder_names))
}

pub(super) fn ensure_generic_specialization(
    template: &AstFunction,
    explicit_generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    context: &str,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<String, String> {
    let substitutions = infer_generic_substitutions(
        template,
        explicit_generic_args,
        args,
        expected,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
        Some(context),
    )?;
    let specialized_name = format!(
        "{}__{}",
        template.name,
        template
            .generic_params
            .iter()
            .map(|param| substitutions[&param.name]
                .render()
                .replace(|ch: char| !ch.is_ascii_alphanumeric(), "_"))
            .collect::<Vec<_>>()
            .join("__")
    );
    if specialization_cache.insert(specialized_name.clone()) {
        let specialized =
            specialize_function_template(template, &specialized_name, &substitutions)?;
        let mut higher_order_specialization_cache = BTreeSet::new();
        let mut higher_order_specialized_templates = Vec::new();
        let higher_order_rewritten = rewrite_higher_order_calls_in_function(
            &specialized,
            higher_order_templates,
            function_table,
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
            visible_type_aliases,
            &mut higher_order_specialization_cache,
            &mut higher_order_specialized_templates,
        )?;
        let mut extended_generic_templates = generic_templates.clone();
        for template in higher_order_specialized_templates {
            if !template.generic_params.is_empty() {
                extended_generic_templates.insert(template.name.clone(), template);
            }
        }
        let rewritten = rewrite_generic_calls_in_function(
            &higher_order_rewritten,
            &BTreeMap::new(),
            visible_type_aliases,
            &extended_generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?;
        specialized_signatures.push((
            specialized_name.clone(),
            FunctionSignature {
                abi: "nuis".to_owned(),
                interface: None,
                symbol_name: specialized_name.clone(),
                params: rewritten
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: rewritten
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                    .transpose()?,
                is_extern: false,
                is_async: rewritten.is_async,
            },
        ));
        specialized_functions.push(rewritten);
    }
    Ok(specialized_name)
}

#[allow(clippy::too_many_arguments)]
fn ensure_generic_impl_method_specialization(
    trait_name: Option<&str>,
    method_name: &str,
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Option<String>, String> {
    let mut candidates = Vec::new();
    for template in generic_impl_method_templates.iter().filter(|template| {
        template.method_name == method_name
            && trait_name.is_none_or(|trait_name| {
                template.trait_name == trait_name
                    || template
                        .trait_name
                        .rsplit('.')
                        .next()
                        .is_some_and(|short| short == trait_name)
            })
            && template.function.params.len() == args.len()
    }) {
        if infer_generic_substitutions(
            &template.function,
            &[],
            args,
            expected,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
            None,
        )
        .is_ok()
        {
            candidates.push(template);
        }
    }
    if candidates.len() > 1 {
        return Err(format!(
            "generic impl method resolution for `{}` is ambiguous; matching impl method templates: {}",
            trait_name
                .map(|trait_name| format!("{trait_name}.{method_name}"))
                .unwrap_or_else(|| method_name.to_owned()),
            candidates
                .iter()
                .map(|template| template.function.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let Some(template) = candidates.into_iter().next() else {
        return Ok(None);
    };
    Ok(Some(ensure_generic_specialization(
        &template.function,
        &[],
        args,
        expected,
        method_name,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    )?))
}

#[allow(clippy::too_many_arguments)]
fn ensure_generic_impl_method_specialization_from_receiver_expected(
    method_name: &str,
    receiver_expected: &AstTypeRef,
    actual_args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Option<String>, String> {
    let inference_receiver = AstExpr::StructLiteral {
        type_name: receiver_expected.name.clone(),
        type_args: receiver_expected.generic_args.clone(),
        fields: Vec::new(),
    };
    let mut inference_args = vec![inference_receiver];
    inference_args.extend(actual_args.iter().skip(1).cloned());

    let mut candidates = Vec::new();
    for template in generic_impl_method_templates
        .iter()
        .filter(|template| template.method_name == method_name && template.function.params.len() == actual_args.len())
    {
        if infer_generic_substitutions(
            &template.function,
            &[],
            &inference_args,
            expected,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
            None,
        )
        .is_ok()
        {
            candidates.push(template);
        }
    }
    if candidates.len() > 1 {
        return Err(format!(
            "generic impl method resolution for `{method_name}` is ambiguous under explicit receiver generic anchoring; matching impl method templates: {}",
            candidates
                .iter()
                .map(|template| template.function.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let Some(template) = candidates.into_iter().next() else {
        return Ok(None);
    };
    Ok(Some(ensure_generic_specialization(
        &template.function,
        &[],
        &inference_args,
        expected,
        method_name,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    )?))
}

pub(super) fn call_arg_expected_type(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    expected: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    if let Some(from_explicit_generics) = generic_template_arg_expected_type_from_explicit_args(
        callee,
        generic_args,
        index,
        generic_templates,
        visible_type_aliases,
    ) {
        return Some(from_explicit_generics);
    }
    if let Some(from_template_expected) = generic_template_arg_expected_type_from_return(
        callee,
        index,
        expected,
        generic_templates,
        visible_type_aliases,
    ) {
        return Some(from_template_expected);
    }
    if let Some(from_signature_expected) = generic_signature_arg_expected_type_from_return(
        callee,
        index,
        expected,
        signatures,
        visible_type_aliases,
        struct_table,
    ) {
        return Some(from_signature_expected);
    }
    if let Some(from_task_builtin) = task_builtin_arg_expected_type(callee, index, expected) {
        return Some(from_task_builtin);
    }
    if let Some(from_signature) = signatures
        .get(callee)
        .and_then(|signature| signature.params.get(index))
        .map(ast_type_from_nir)
    {
        return Some(from_signature);
    }
    constructor_value_expected_type(
        callee,
        generic_args,
        index,
        expected,
        visible_type_aliases,
        struct_table,
    )
}

fn generic_template_arg_expected_type_from_explicit_args(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    generic_templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    if generic_args.is_empty() {
        return None;
    }
    let template = generic_templates.get(callee)?;
    if generic_args.len() != template.generic_params.len() {
        return None;
    }
    let param = template.params.get(index)?;
    let substitutions = template
        .generic_params
        .iter()
        .zip(generic_args.iter())
        .map(|(generic, arg)| {
            Some((
                generic.name.clone(),
                lower_type_ref(&resolve_ast_type_ref_aliases(arg, visible_type_aliases).ok()?),
            ))
        })
        .collect::<Option<BTreeMap<_, _>>>()?;
    specialize_ast_type_ref(&param.ty, &substitutions).ok()
}

fn generic_template_arg_expected_type_from_return(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let template = generic_templates.get(callee)?;
    let return_pattern = template.return_type.as_ref()?;
    let param = template.params.get(index)?;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    if unify_generic_type_pattern(
        return_pattern,
        expected,
        &generic_names,
        &mut substitutions,
        &template.name,
    )
    .is_err()
    {
        let resolved_return_pattern =
            resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases).ok()?;
        let resolved_expected =
            resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
        substitutions.clear();
        unify_generic_type_pattern(
            &resolved_return_pattern,
            &resolved_expected,
            &generic_names,
            &mut substitutions,
            &template.name,
        )
        .ok()?;
    }
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    let specialized = specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok()?;
    (!contains_ast_placeholder_generic_name(&specialized, &generic_names)).then_some(specialized)
}

fn generic_signature_arg_expected_type_from_return(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    let signature = signatures.get(callee)?;
    let return_ty = ast_type_from_nir(signature.return_type.as_ref()?);
    let param_ty = ast_type_from_nir(signature.params.get(index)?);
    let mut generic_names = BTreeSet::new();
    collect_signature_generic_placeholders(
        &return_ty,
        visible_type_aliases,
        struct_table,
        &mut generic_names,
    );
    collect_signature_generic_placeholders(
        &param_ty,
        visible_type_aliases,
        struct_table,
        &mut generic_names,
    );
    if generic_names.is_empty() {
        return None;
    }
    let resolved_return_pattern =
        resolve_ast_type_ref_aliases(&return_ty, visible_type_aliases).ok()?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    unify_generic_type_pattern(
        &resolved_return_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        callee,
    )
    .ok()?;
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    let specialized = specialize_ast_type_ref(&param_ty, &lowered_substitutions).ok()?;
    (!contains_ast_placeholder_generic_name(&specialized, &generic_names)).then_some(specialized)
}

fn task_builtin_arg_expected_type(
    callee: &str,
    index: usize,
    expected: Option<&AstTypeRef>,
) -> Option<AstTypeRef> {
    let expected = expected?;
    match callee {
        "spawn" if index == 0 && expected.name == "Task" && expected.generic_args.len() == 1 => {
            expected.generic_args.first().cloned()
        }
        "join" if index == 0 && expected.name != "Task" => Some(AstTypeRef {
            name: "Task".to_owned(),
            generic_args: vec![expected.clone()],
            is_optional: false,
            is_ref: false,
        }),
        _ => None,
    }
}

fn collect_signature_generic_placeholders(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
    out: &mut BTreeSet<String>,
) {
    if ty.generic_args.is_empty()
        && ty
            .name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        && !visible_type_aliases.contains_key(&ty.name)
        && !struct_table.contains_key(&ty.name)
        && ty.name != "String"
    {
        out.insert(ty.name.clone());
    }
    for arg in &ty.generic_args {
        collect_signature_generic_placeholders(arg, visible_type_aliases, struct_table, out);
    }
}

fn constructor_value_expected_type(
    callee: &str,
    generic_args: &[AstTypeRef],
    index: usize,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    if index != 0 {
        return None;
    }
    let concrete_head = if let Some(definition) = struct_table.get(callee) {
        if definition.fields.len() != 1 {
            return None;
        }
        if generic_args.is_empty() {
            expected
                .filter(|expected| expected.name == callee)
                .cloned()
                .or_else(|| {
                    let (parent_name, _) = callee.rsplit_once('.')?;
                    let resolved_expected =
                        resolve_ast_type_ref_aliases(expected?, visible_type_aliases).ok()?;
                    (resolved_expected.name == parent_name).then_some(AstTypeRef {
                        name: callee.to_owned(),
                        generic_args: resolved_expected.generic_args,
                        is_optional: false,
                        is_ref: false,
                    })
                })
        } else {
            Some(AstTypeRef {
                name: callee.to_owned(),
                generic_args: generic_args.to_vec(),
                is_optional: false,
                is_ref: false,
            })
        }
    } else if generic_args.is_empty() {
        infer_alias_struct_target_from_expected(
            callee,
            expected,
            visible_type_aliases,
            struct_table,
        )
        .ok()
        .flatten()
        .map(|(name, generic_args)| AstTypeRef {
            name,
            generic_args,
            is_optional: false,
            is_ref: false,
        })
    } else {
        let alias_definition = visible_type_aliases.get(callee)?;
        let substituted = substitute_ast_type_alias_target(
            &alias_definition.target,
            &alias_definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .zip(generic_args.iter().cloned())
                .collect::<BTreeMap<_, _>>(),
        )
        .ok()?;
        let resolved = resolve_ast_type_ref_aliases(&substituted, visible_type_aliases).ok()?;
        Some(resolved)
    }?;
    let definition = struct_table.get(&concrete_head.name)?;
    if definition.fields.len() != 1 {
        return None;
    }
    Some(
        super::super::validation_binding_env::instantiate_ast_struct_field_type(
            &concrete_head,
            definition,
            &definition.fields[0].ty,
        ),
    )
}

fn struct_field_expected_type(
    literal_ty: &AstTypeRef,
    field_name: &str,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let definition = struct_table.get(&literal_ty.name)?;
    let field = definition
        .fields
        .iter()
        .find(|field| field.name == field_name)?;
    Some(
        super::super::validation_binding_env::instantiate_ast_struct_field_type(
            literal_ty, definition, &field.ty,
        ),
    )
}

fn field_access_base_expected_type(
    field_expected: Option<&AstTypeRef>,
    field_name: &str,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    struct_table: &BTreeMap<String, AstStructDef>,
) -> Option<AstTypeRef> {
    let resolved_expected =
        resolve_ast_type_ref_aliases(field_expected?, visible_type_aliases).ok()?;
    let mut candidates = Vec::new();
    for definition in struct_table.values() {
        let Some(field) = definition.fields.iter().find(|field| field.name == field_name) else {
            continue;
        };
        let resolved_field_ty = resolve_ast_type_ref_aliases(&field.ty, visible_type_aliases).ok()?;
        let generic_names = definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
        if unify_generic_type_pattern(
            &resolved_field_ty,
            &resolved_expected,
            &generic_names,
            &mut substitutions,
            &definition.name,
        )
        .is_err()
        {
            continue;
        }
        let Some(generic_args) = definition
            .generic_params
            .iter()
            .map(|param| substitutions.get(&param.name).cloned())
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        if generic_args
            .iter()
            .any(|arg| contains_ast_placeholder_generic_name(arg, &generic_names))
        {
            continue;
        }
        candidates.push(AstTypeRef {
            name: definition.name.clone(),
            generic_args,
            is_optional: false,
            is_ref: false,
        });
    }
    (candidates.len() == 1).then(|| candidates.pop()).flatten()
}
