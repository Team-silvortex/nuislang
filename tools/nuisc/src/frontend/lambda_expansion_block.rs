use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstFunction, AstGenericParam, AstImplDef,
    AstMatchArm, AstStmt, AstStructDef, AstTypeRef,
};

use super::lambda_expansion_expr::{rewrite_lambda_expr, LambdaExprRewriteInput};
use super::lambda_expansion_synth::{synthesize_lambda_function, LambdaSynthesisInput};
use super::lambda_expansion_types::{
    callable_binding_return_type, callable_type_from_function, callable_type_from_signature,
    callable_type_matches_signature, extend_local_field_bindings_from_expr,
    extend_local_field_bindings_from_type, infer_local_binding_type, LambdaBinding,
};

pub(super) struct ExpandLambdaBlockInput<'a> {
    pub(super) body: &'a [AstStmt],
    pub(super) current_return_type: Option<&'a AstTypeRef>,
    pub(super) inherited_generic_params: &'a [AstGenericParam],
    pub(super) lambda_aliases: &'a BTreeMap<String, LambdaBinding>,
    pub(super) visible_locals: &'a BTreeSet<String>,
    pub(super) visible_local_types: &'a BTreeMap<String, AstTypeRef>,
    pub(super) module_impls: &'a [AstImplDef],
    pub(super) visible_structs: &'a BTreeMap<String, AstStructDef>,
    pub(super) module_const_names: &'a BTreeSet<String>,
    pub(super) module_function_table: &'a BTreeMap<String, AstFunction>,
    pub(super) owning_function_name: &'a str,
    pub(super) counter: &'a mut usize,
    pub(super) synthesized: &'a mut Vec<AstFunction>,
}

pub(super) fn expand_lambda_block(
    input: ExpandLambdaBlockInput<'_>,
) -> Result<Vec<AstStmt>, String> {
    let ExpandLambdaBlockInput {
        body,
        current_return_type,
        inherited_generic_params,
        lambda_aliases,
        visible_locals,
        visible_local_types,
        module_impls,
        visible_structs,
        module_const_names,
        module_function_table,
        owning_function_name,
        counter,
        synthesized,
    } = input;
    let mut aliases = lambda_aliases.clone();
    let mut locals = visible_locals.clone();
    let mut local_types = visible_local_types.clone();
    let mut rewritten = Vec::new();
    macro_rules! rewrite_block_expr {
        ($expr:expr, $expected:expr) => {
            rewrite_lambda_expr(LambdaExprRewriteInput {
                expr: $expr,
                expected_expr_type: $expected,
                inherited_generic_params,
                lambda_aliases: &aliases,
                visible_locals: &locals,
                visible_local_types: &local_types,
                module_impls,
                module_const_names,
                module_function_table,
                owning_function_name,
                counter,
                synthesized,
            })
        };
    }
    for stmt in body {
        match stmt {
            AstStmt::Let {
                name,
                ty,
                mutable: _,
                value:
                    AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    },
            } => {
                let effective_return_type =
                    callable_binding_return_type(name, params, ty.as_ref(), return_type)?;
                let binding = synthesize_lambda_function(LambdaSynthesisInput {
                    params,
                    return_type: &effective_return_type,
                    body,
                    inherited_generic_params,
                    lambda_aliases: &aliases,
                    outer_locals: &locals,
                    outer_local_types: &local_types,
                    module_impls,
                    visible_structs,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                })
                .map_err(|error| {
                    if error == "inline lambda currently requires an explicit return type" {
                        format!(
                            "lambda binding `{name}` currently requires an explicit return type"
                        )
                    } else {
                        error
                    }
                })?;
                aliases.insert(name.clone(), binding);
                locals.insert(name.clone());
                if let Some(return_type) = effective_return_type.as_ref() {
                    if let Some(binding_ty) = callable_type_from_signature(params, return_type) {
                        local_types.insert(name.clone(), binding_ty);
                    }
                }
            }
            AstStmt::Let {
                name,
                ty: Some(ty),
                value: AstExpr::Var(value_name),
                mutable: _,
            } if !module_const_names.contains(value_name)
                && module_function_table.contains_key(value_name) =>
            {
                let function = module_function_table
                    .get(value_name)
                    .expect("checked function table presence");
                let Some(return_type) = function.return_type.as_ref() else {
                    return Err(format!(
                        "callable binding `{name}` target `{value_name}` requires an explicit return type"
                    ));
                };
                if !callable_type_matches_signature(&function.params, return_type, ty) {
                    return Err(format!(
                        "callable binding `{name}` target `{value_name}` does not match declared callable type `{}`",
                        ty.name
                    ));
                }
                aliases.insert(
                    name.clone(),
                    LambdaBinding {
                        symbol: value_name.clone(),
                        captured_locals: Vec::new(),
                    },
                );
                locals.insert(name.clone());
                local_types.insert(name.clone(), ty.clone());
            }
            AstStmt::Let {
                name,
                ty: None,
                value: AstExpr::Var(value_name),
                mutable: _,
            } if !module_const_names.contains(value_name)
                && module_function_table.contains_key(value_name) =>
            {
                aliases.insert(
                    name.clone(),
                    LambdaBinding {
                        symbol: value_name.clone(),
                        captured_locals: Vec::new(),
                    },
                );
                locals.insert(name.clone());
                if let Some(binding_ty) = module_function_table
                    .get(value_name)
                    .and_then(callable_type_from_function)
                {
                    local_types.insert(name.clone(), binding_ty);
                }
            }
            AstStmt::Let {
                name,
                ty,
                value,
                mutable,
            } => {
                let rewritten_value = rewrite_block_expr!(value, ty.as_ref())?;
                rewritten.push(AstStmt::Let {
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value.clone(),
                });
                aliases.remove(name);
                locals.insert(name.clone());
                if let Some(ty) = ty.clone() {
                    local_types.insert(name.clone(), ty);
                    if let Some(bound_ty) = local_types.get(name).cloned() {
                        extend_local_field_bindings_from_type(
                            name,
                            &bound_ty,
                            visible_structs,
                            &mut local_types,
                        );
                    }
                } else if let Some(inferred_ty) = infer_local_binding_type(
                    &rewritten_value,
                    &local_types,
                    module_function_table,
                    module_impls,
                ) {
                    local_types.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    &rewritten_value,
                    &mut local_types,
                    module_function_table,
                    module_impls,
                );
            }
            AstStmt::AssignLocal { name, value } => {
                let rewritten_value = rewrite_block_expr!(value, local_types.get(name))?;
                rewritten.push(AstStmt::AssignLocal {
                    name: name.clone(),
                    value: rewritten_value.clone(),
                });
                if let Some(inferred_ty) = infer_local_binding_type(
                    &rewritten_value,
                    &local_types,
                    module_function_table,
                    module_impls,
                ) {
                    local_types.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    &rewritten_value,
                    &mut local_types,
                    module_function_table,
                    module_impls,
                );
            }
            AstStmt::DestructureLet {
                type_ref,
                fields,
                value,
            } => {
                let rewritten_value = rewrite_block_expr!(value, type_ref.as_ref())?;
                rewritten.push(AstStmt::DestructureLet {
                    type_ref: type_ref.clone(),
                    fields: fields.clone(),
                    value: rewritten_value,
                });
                let mut names = Vec::new();
                collect_destructure_binding_names(fields, &mut names);
                for name in names {
                    aliases.remove(&name);
                    locals.insert(name.clone());
                    local_types.remove(&name);
                }
            }
            AstStmt::Const { name, ty, value } => {
                let rewritten_value = rewrite_block_expr!(value, ty.as_ref())?;
                rewritten.push(AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value.clone(),
                });
                aliases.remove(name);
                locals.insert(name.clone());
                if let Some(ty) = ty.clone() {
                    local_types.insert(name.clone(), ty);
                    if let Some(bound_ty) = local_types.get(name).cloned() {
                        extend_local_field_bindings_from_type(
                            name,
                            &bound_ty,
                            visible_structs,
                            &mut local_types,
                        );
                    }
                } else if let Some(inferred_ty) = infer_local_binding_type(
                    &rewritten_value,
                    &local_types,
                    module_function_table,
                    module_impls,
                ) {
                    local_types.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    &rewritten_value,
                    &mut local_types,
                    module_function_table,
                    module_impls,
                );
            }
            AstStmt::Print(value) => {
                rewritten.push(AstStmt::Print(rewrite_block_expr!(value, None)?))
            }
            AstStmt::Await(value) => {
                rewritten.push(AstStmt::Await(rewrite_block_expr!(value, None)?))
            }
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => rewritten.push(AstStmt::If {
                condition: rewrite_block_expr!(condition, None)?,
                then_body: expand_lambda_block(ExpandLambdaBlockInput {
                    body: then_body,
                    current_return_type,
                    inherited_generic_params,
                    lambda_aliases: &aliases,
                    visible_locals: &locals,
                    visible_local_types: &local_types,
                    module_impls,
                    visible_structs,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                })?,
                else_body: expand_lambda_block(ExpandLambdaBlockInput {
                    body: else_body,
                    current_return_type,
                    inherited_generic_params,
                    lambda_aliases: &aliases,
                    visible_locals: &locals,
                    visible_local_types: &local_types,
                    module_impls,
                    visible_structs,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                })?,
            }),
            AstStmt::Match { value, arms } => rewritten.push(AstStmt::Match {
                value: rewrite_block_expr!(value, None)?,
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm
                                .guard
                                .clone()
                                .map(|guard| rewrite_block_expr!(&guard, None))
                                .transpose()?,
                            body: expand_lambda_block(ExpandLambdaBlockInput {
                                body: &arm.body,
                                current_return_type,
                                inherited_generic_params,
                                lambda_aliases: &aliases,
                                visible_locals: &locals,
                                visible_local_types: &local_types,
                                module_impls,
                                visible_structs,
                                module_const_names,
                                module_function_table,
                                owning_function_name,
                                counter,
                                synthesized,
                            })?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }),
            AstStmt::While { condition, body } => rewritten.push(AstStmt::While {
                condition: rewrite_block_expr!(condition, None)?,
                body: expand_lambda_block(ExpandLambdaBlockInput {
                    body,
                    current_return_type,
                    inherited_generic_params,
                    lambda_aliases: &aliases,
                    visible_locals: &locals,
                    visible_local_types: &local_types,
                    module_impls,
                    visible_structs,
                    module_const_names,
                    module_function_table,
                    owning_function_name,
                    counter,
                    synthesized,
                })?,
            }),
            AstStmt::Expr(expr) => rewritten.push(AstStmt::Expr(rewrite_block_expr!(expr, None)?)),
            AstStmt::Return(value) => rewritten.push(AstStmt::Return(match value {
                Some(value) => Some(rewrite_block_expr!(value, current_return_type)?),
                None => None,
            })),
            AstStmt::Break => rewritten.push(AstStmt::Break),
            AstStmt::Continue => rewritten.push(AstStmt::Continue),
        }
    }
    Ok(rewritten)
}

fn collect_destructure_binding_names(fields: &[AstDestructureField], names: &mut Vec<String>) {
    for field in fields {
        match &field.binding {
            AstDestructureBinding::Bind(name) => names.push(name.clone()),
            AstDestructureBinding::Ignore => {}
            AstDestructureBinding::Nested { fields, .. } => {
                collect_destructure_binding_names(fields, names)
            }
        }
    }
}
