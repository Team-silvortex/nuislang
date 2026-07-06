use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstGenericParam, AstImplDef, AstMatchArm, AstTypeRef,
};

use super::lambda_expansion_block::{expand_lambda_block, ExpandLambdaBlockInput};
use super::lambda_expansion_synth::{
    expected_callable_type_for_call_arg, expected_callable_type_for_method_arg,
    inline_lambda_return_type_from_callable, synthesize_lambda_function,
    synthesize_lambda_function_with_known_return_type, ExpectedCallArgInput,
    ExpectedMethodArgInput, KnownReturnLambdaSynthesisInput, LambdaSynthesisInput,
};
use super::lambda_expansion_types::{build_lambda_binding_value, build_lambda_call, LambdaBinding};

#[allow(clippy::too_many_arguments)]
pub(super) fn rewrite_lambda_expr(
    expr: &AstExpr,
    expected_expr_type: Option<&AstTypeRef>,
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    visible_locals: &BTreeSet<String>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_impls: &[AstImplDef],
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Var(name)
            if lambda_aliases.contains_key(name) && !module_const_names.contains(name) =>
        {
            let binding = lambda_aliases
                .get(name)
                .cloned()
                .expect("checked lambda alias presence");
            build_lambda_binding_value(&binding)
        }
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_lambda_expr(
                condition,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            then_body: expand_lambda_block(ExpandLambdaBlockInput {
                body: then_body,
                current_return_type: expected_expr_type,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                visible_structs: &BTreeMap::new(),
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            })?,
            else_body: expand_lambda_block(ExpandLambdaBlockInput {
                body: else_body,
                current_return_type: expected_expr_type,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                visible_structs: &BTreeMap::new(),
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            })?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_lambda_expr(
                value,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: match &arm.guard {
                            Some(guard) => Some(rewrite_lambda_expr(
                                guard,
                                None,
                                inherited_generic_params,
                                lambda_aliases,
                                visible_locals,
                                visible_local_types,
                                module_impls,
                                module_const_names,
                                module_function_table,
                                owning_function_name,
                                counter,
                                synthesized,
                            )?),
                            None => None,
                        },
                        body: expand_lambda_block(ExpandLambdaBlockInput {
                            body: &arm.body,
                            current_return_type: expected_expr_type,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            visible_structs: &BTreeMap::new(),
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        })?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => {
            let binding = synthesize_lambda_function(LambdaSynthesisInput {
                params,
                return_type,
                body,
                inherited_generic_params,
                lambda_aliases,
                outer_locals: visible_locals,
                outer_local_types: visible_local_types,
                module_impls,
                visible_structs: &BTreeMap::new(),
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            })?;
            build_lambda_binding_value(&binding)
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_lambda_expr(
            value,
            None,
            inherited_generic_params,
            lambda_aliases,
            visible_locals,
            visible_local_types,
            module_impls,
            module_const_names,
            module_function_table,
            owning_function_name,
            counter,
            synthesized,
        )?)),
        AstExpr::Try(value) => AstExpr::Try(Box::new(rewrite_lambda_expr(
            value,
            None,
            inherited_generic_params,
            lambda_aliases,
            visible_locals,
            visible_local_types,
            module_impls,
            module_const_names,
            module_function_table,
            owning_function_name,
            counter,
            synthesized,
        )?)),
        AstExpr::Invoke { callee, args } => {
            let rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        None,
                        inherited_generic_params,
                        lambda_aliases,
                        visible_locals,
                        visible_local_types,
                        module_impls,
                        module_const_names,
                        module_function_table,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            match callee.as_ref() {
                AstExpr::Lambda {
                    params,
                    return_type,
                    body,
                } => {
                    let binding = synthesize_lambda_function(LambdaSynthesisInput {
                        params,
                        return_type,
                        body,
                        inherited_generic_params,
                        lambda_aliases,
                        outer_locals: visible_locals,
                        outer_local_types: visible_local_types,
                        module_impls,
                        visible_structs: &BTreeMap::new(),
                        module_const_names,
                        module_function_table,
                        owning_function_name,
                        counter,
                        synthesized,
                    })?;
                    build_lambda_call(&binding, rewritten_args)
                }
                AstExpr::Var(name) => {
                    if let Some(binding) = lambda_aliases.get(name) {
                        build_lambda_call(binding, rewritten_args)
                    } else {
                        AstExpr::Call {
                            callee: name.clone(),
                            generic_args: Vec::new(),
                            args: rewritten_args,
                        }
                    }
                }
                _ => {
                    return Err(
                        "only immediate lambda invocation and named function or lambda binding invocation are supported in the current MVP"
                            .to_owned(),
                    )
                }
            }
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            let rewritten_args = args
                .iter()
                .enumerate()
                .map(|(index, arg)| {
                    if let AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    } = arg
                    {
                        let inferred_return_type = inline_lambda_return_type_from_callable(
                            params,
                            return_type,
                            expected_callable_type_for_call_arg(ExpectedCallArgInput {
                                callee,
                                index,
                                generic_args,
                                args,
                                expected_result_type: expected_expr_type,
                                visible_local_types,
                                module_function_table,
                                module_impls,
                            })
                            .as_ref(),
                        )?;
                        let binding = synthesize_lambda_function_with_known_return_type(
                            KnownReturnLambdaSynthesisInput {
                                params,
                                lambda_return_type: inferred_return_type.ok_or_else(|| {
                                    "inline lambda currently requires an explicit return type"
                                        .to_owned()
                                })?,
                                body,
                                inherited_generic_params,
                                lambda_aliases,
                                outer_locals: visible_locals,
                                outer_local_types: visible_local_types,
                                module_impls,
                                visible_structs: &BTreeMap::new(),
                                module_const_names,
                                module_function_table,
                                owning_function_name,
                                counter,
                                synthesized,
                            },
                        )?;
                        Ok(build_lambda_binding_value(&binding))
                    } else {
                        rewrite_lambda_expr(
                            arg,
                            None,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            if let Some(binding) = lambda_aliases.get(callee) {
                build_lambda_call(binding, rewritten_args)
            } else {
                AstExpr::Call {
                    callee: callee.clone(),
                    generic_args: generic_args.clone(),
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
            let rewritten_receiver = Box::new(rewrite_lambda_expr(
                receiver,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?);
            let rewritten_args = args
                .iter()
                .enumerate()
                .map(|(index, arg)| {
                    if let AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    } = arg
                    {
                        let inferred_return_type = inline_lambda_return_type_from_callable(
                            params,
                            return_type,
                            expected_callable_type_for_method_arg(ExpectedMethodArgInput {
                                receiver,
                                method,
                                index,
                                args,
                                expected_result_type: expected_expr_type,
                                visible_local_types,
                                module_function_table,
                                module_impls,
                            })
                            .as_ref(),
                        )?;
                        let binding = synthesize_lambda_function_with_known_return_type(
                            KnownReturnLambdaSynthesisInput {
                                params,
                                lambda_return_type: inferred_return_type.ok_or_else(|| {
                                    "inline lambda currently requires an explicit return type"
                                        .to_owned()
                                })?,
                                body,
                                inherited_generic_params,
                                lambda_aliases,
                                outer_locals: visible_locals,
                                outer_local_types: visible_local_types,
                                module_impls,
                                visible_structs: &BTreeMap::new(),
                                module_const_names,
                                module_function_table,
                                owning_function_name,
                                counter,
                                synthesized,
                            },
                        )?;
                        Ok(build_lambda_binding_value(&binding))
                    } else {
                        rewrite_lambda_expr(
                            arg,
                            None,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            AstExpr::MethodCall {
                receiver: rewritten_receiver,
                method: method.clone(),
                generic_args: generic_args.clone(),
                args: rewritten_args,
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rewrite_lambda_expr(
                            value,
                            None,
                            inherited_generic_params,
                            lambda_aliases,
                            visible_locals,
                            visible_local_types,
                            module_impls,
                            module_const_names,
                            module_function_table,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_lambda_expr(
                base,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            field: field.clone(),
        },
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(rewrite_lambda_expr(
                operand,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_lambda_expr(
                lhs,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
            rhs: Box::new(rewrite_lambda_expr(
                rhs,
                None,
                inherited_generic_params,
                lambda_aliases,
                visible_locals,
                visible_local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            )?),
        },
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => expr.clone(),
    })
}
