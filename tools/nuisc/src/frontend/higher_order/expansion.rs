use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstMatchArm, AstModule, AstParam, AstStmt, AstStructDef,
    AstTypeAlias, AstTypeRef,
};

use super::super::validation_binding_env::instantiate_ast_struct_field_type;

use super::super::generics::{specialize_ast_type_ref, unify_generic_type_pattern};
use super::super::{
    build_impl_method_function, impl_method_symbol_name, lower_type_ref,
    lower_type_ref_with_aliases, resolve_ast_type_ref_aliases,
};
use super::callables::{
    function_type_matches_callable, is_callable_type_with_aliases, sanitize_symbol_fragment,
};
use super::templates::{rewrite_higher_order_template_expr, specialize_higher_order_template};

const LAMBDA_BIND_PREFIX: &str = "__lambda_bind.";

fn render_local_access_path(expr: &AstExpr) -> Option<String> {
    match expr {
        AstExpr::Var(name) => Some(name.clone()),
        AstExpr::FieldAccess { base, field } => {
            Some(format!("{}.{}", render_local_access_path(base)?, field))
        }
        _ => None,
    }
}

fn extend_local_field_bindings_from_expr(
    binding_path: &str,
    expr: &AstExpr,
    local_types: &mut BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) {
    let AstExpr::StructLiteral { fields, .. } = expr else {
        return;
    };
    for (field_name, value) in fields {
        let Some(field_ty) =
            infer_local_binding_type(value, local_types, function_table, module_impls)
        else {
            continue;
        };
        let field_path = format!("{binding_path}.{}", field_name);
        local_types.insert(field_path.clone(), field_ty);
        extend_local_field_bindings_from_expr(
            &field_path,
            value,
            local_types,
            function_table,
            module_impls,
        );
    }
}

fn extend_local_field_bindings_from_type(
    binding_path: &str,
    ty: &AstTypeRef,
    visible_structs: &BTreeMap<String, AstStructDef>,
    local_types: &mut BTreeMap<String, AstTypeRef>,
) {
    let Some(definition) = visible_structs.get(&ty.name) else {
        return;
    };
    for field in &definition.fields {
        let field_ty = instantiate_ast_struct_field_type(ty, definition, &field.ty);
        let field_path = format!("{}.{}", binding_path, field.name);
        local_types.insert(field_path.clone(), field_ty.clone());
        extend_local_field_bindings_from_type(&field_path, &field_ty, visible_structs, local_types);
    }
}

fn specialize_type_with_substitutions(
    ty: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if ty.generic_args.is_empty() {
        if let Some(substitution) = substitutions.get(&ty.name) {
            return substitution.clone();
        }
    }
    AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| specialize_type_with_substitutions(arg, substitutions))
            .collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}

fn infer_impl_method_substitutions(
    definition: &AstImplDef,
    method_index: usize,
    receiver_ty: &AstTypeRef,
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> BTreeMap<String, AstTypeRef> {
    let generic_names = collect_generic_type_names(&definition.for_type);
    let mut substitutions = BTreeMap::new();
    unify_generic_receiver_pattern(
        &definition.for_type,
        receiver_ty,
        &generic_names,
        &mut substitutions,
    );
    let Some(method_def) = definition.methods.get(method_index) else {
        return substitutions;
    };
    for (param, arg) in method_def.params.iter().skip(1).zip(args.iter()) {
        let Some(arg_ty) = infer_local_binding_type(arg, local_types, function_table, module_impls)
        else {
            continue;
        };
        unify_generic_receiver_pattern(&param.ty, &arg_ty, &generic_names, &mut substitutions);
    }
    let method_return_ty = method_def
        .return_type
        .as_ref()
        .unwrap_or(&definition.for_type);
    if let Some(expected_result_type) = expected_result_type {
        unify_generic_receiver_pattern(
            method_return_ty,
            expected_result_type,
            &generic_names,
            &mut substitutions,
        );
    }
    substitutions
}

fn unify_generic_receiver_pattern(
    pattern: &AstTypeRef,
    actual: &AstTypeRef,
    generic_names: &BTreeSet<String>,
    substitutions: &mut BTreeMap<String, AstTypeRef>,
) {
    if pattern.is_optional != actual.is_optional || pattern.is_ref != actual.is_ref {
        return;
    }
    if pattern.generic_args.is_empty() && generic_names.contains(&pattern.name) {
        if !substitutions.contains_key(&pattern.name) {
            substitutions.insert(pattern.name.clone(), actual.clone());
        }
        return;
    }
    if pattern.name != actual.name || pattern.generic_args.len() != actual.generic_args.len() {
        return;
    }
    for (pattern_arg, actual_arg) in pattern.generic_args.iter().zip(actual.generic_args.iter()) {
        unify_generic_receiver_pattern(pattern_arg, actual_arg, generic_names, substitutions);
    }
}

fn infer_local_binding_type(
    value: &AstExpr,
    local_types: &BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match value {
        AstExpr::Bool(_) => Some(AstTypeRef {
            name: "bool".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Text(_) => Some(AstTypeRef {
            name: "String".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Int(_) => Some(AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Float(_) => Some(AstTypeRef {
            name: "f64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Var(name) => local_types.get(name).cloned(),
        AstExpr::FieldAccess { .. } => {
            render_local_access_path(value).and_then(|path| local_types.get(&path).cloned())
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            ..
        } => Some(AstTypeRef {
            name: type_name.clone(),
            generic_args: type_args.clone(),
            is_optional: false,
            is_ref: false,
        }),
        AstExpr::Call { callee, .. } => function_table
            .get(callee)
            .and_then(|function| function.return_type.clone()),
        AstExpr::If {
            then_body,
            else_body,
            ..
        } => {
            let then_ty =
                infer_block_result_type(then_body, local_types, function_table, module_impls)?;
            let else_ty =
                infer_block_result_type(else_body, local_types, function_table, module_impls)?;
            if lower_type_ref(&then_ty).render() == lower_type_ref(&else_ty).render() {
                Some(then_ty)
            } else {
                None
            }
        }
        AstExpr::Match { arms, .. } => {
            let mut arm_types = arms.iter().filter_map(|arm| {
                infer_block_result_type(&arm.body, local_types, function_table, module_impls)
            });
            let first = arm_types.next()?;
            if arm_types.all(|ty| lower_type_ref(&first).render() == lower_type_ref(&ty).render()) {
                Some(first)
            } else {
                None
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args: _,
            args,
        } => {
            let receiver_ty =
                infer_local_binding_type(receiver, local_types, function_table, module_impls)?;
            for definition in module_impls {
                let Some(method_index) = definition
                    .methods
                    .iter()
                    .position(|item| item.name == *method)
                else {
                    continue;
                };
                let substitutions = infer_impl_method_substitutions(
                    definition,
                    method_index,
                    &receiver_ty,
                    args,
                    None,
                    local_types,
                    function_table,
                    module_impls,
                );
                let specialized_for_type =
                    specialize_type_with_substitutions(&definition.for_type, &substitutions);
                if lower_type_ref(&specialized_for_type).render()
                    != lower_type_ref(&receiver_ty).render()
                {
                    continue;
                }
                let method_def = &definition.methods[method_index];
                let method_return_ty = method_def
                    .return_type
                    .as_ref()
                    .unwrap_or(&definition.for_type);
                return Some(specialize_type_with_substitutions(
                    method_return_ty,
                    &substitutions,
                ));
            }
            None
        }
        _ => None,
    }
}

fn infer_block_result_type(
    body: &[AstStmt],
    local_types: &BTreeMap<String, AstTypeRef>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    match body.last() {
        Some(AstStmt::Return(Some(expr))) | Some(AstStmt::Expr(expr)) => {
            infer_local_binding_type(expr, local_types, function_table, module_impls)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundCallable {
    pub(crate) symbol: String,
    pub(crate) capture_args: Vec<AstExpr>,
}

fn parse_bound_callable_expr(
    expr: &AstExpr,
    template_callable_bindings: Option<&BTreeMap<String, BoundCallable>>,
) -> Option<BoundCallable> {
    match expr {
        AstExpr::Var(name) => Some(
            template_callable_bindings
                .and_then(|bindings| bindings.get(name))
                .cloned()
                .unwrap_or_else(|| BoundCallable {
                    symbol: name.clone(),
                    capture_args: Vec::new(),
                }),
        ),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty() && callee.starts_with(LAMBDA_BIND_PREFIX) => {
            Some(BoundCallable {
                symbol: callee[LAMBDA_BIND_PREFIX.len()..].to_owned(),
                capture_args: args.to_vec(),
            })
        }
        _ => None,
    }
}

pub(crate) fn expand_higher_order_functions(
    module: &AstModule,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<AstModule, String> {
    let visible_structs = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut templates = module
        .functions
        .iter()
        .filter(|function| {
            function.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            })
        })
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut method_template_lookup = BTreeMap::<(String, String), String>::new();
    for definition in &module.impls {
        let lowered_for_type =
            lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?;
        for method in &definition.methods {
            if !method.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            }) {
                continue;
            }
            let symbol_name =
                impl_method_symbol_name(&definition.trait_name, &lowered_for_type, &method.name);
            templates.insert(
                symbol_name.clone(),
                build_impl_method_function(definition, method, &symbol_name),
            );
            method_template_lookup.insert(
                (lowered_for_type.render(), method.name.clone()),
                symbol_name,
            );
        }
    }
    if templates.is_empty() {
        return Ok(module.clone());
    }

    let mut function_table = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    function_table.extend(templates.clone());

    let mut expanded = module.clone();
    expanded.functions.clear();
    let mut specialized_cache = BTreeSet::new();
    let mut specialized_functions = Vec::new();

    for function in &module.functions {
        if templates.contains_key(&function.name) {
            continue;
        }
        expanded
            .functions
            .push(rewrite_higher_order_calls_in_function(
                function,
                &templates,
                &function_table,
                &module.impls,
                &visible_structs,
                &method_template_lookup,
                visible_type_aliases,
                &mut specialized_cache,
                &mut specialized_functions,
            )?);
    }
    expanded.functions.extend(specialized_functions);
    Ok(expanded)
}

pub(crate) fn rewrite_higher_order_calls_in_function(
    function: &AstFunction,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    method_template_lookup: &BTreeMap<(String, String), String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstFunction, String> {
    let mut local_types = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for param in &function.params {
        extend_local_field_bindings_from_type(
            &param.name,
            &param.ty,
            visible_structs,
            &mut local_types,
        );
    }
    let body = rewrite_higher_order_calls_in_block(
        &function.body,
        function.return_type.as_ref(),
        &local_types,
        templates,
        function_table,
        module_impls,
        visible_structs,
        method_template_lookup,
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok(rewritten)
}

pub(crate) fn rewrite_higher_order_calls_in_block(
    body: &[AstStmt],
    current_return_type: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    method_template_lookup: &BTreeMap<(String, String), String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    let mut env = local_types.clone();
    let mut rewritten = Vec::with_capacity(body.len());
    for stmt in body {
        let rewritten_stmt = rewrite_higher_order_calls_in_stmt(
            stmt,
            current_return_type,
            &env,
            templates,
            function_table,
            module_impls,
            visible_structs,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?;
        match &rewritten_stmt {
            AstStmt::Let {
                name, ty, value, ..
            }
            | AstStmt::Const { name, ty, value } => {
                if let Some(ty) = ty.clone() {
                    env.insert(name.clone(), ty);
                    if let Some(bound_ty) = env.get(name).cloned() {
                        extend_local_field_bindings_from_type(
                            name,
                            &bound_ty,
                            visible_structs,
                            &mut env,
                        );
                    }
                } else if let Some(inferred_ty) =
                    infer_local_binding_type(value, &env, function_table, module_impls)
                {
                    env.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    value,
                    &mut env,
                    function_table,
                    module_impls,
                );
            }
            AstStmt::AssignLocal { name, value } => {
                if let Some(inferred_ty) =
                    infer_local_binding_type(value, &env, function_table, module_impls)
                {
                    env.insert(name.clone(), inferred_ty);
                }
                extend_local_field_bindings_from_expr(
                    name,
                    value,
                    &mut env,
                    function_table,
                    module_impls,
                );
            }
            _ => {}
        }
        rewritten.push(rewritten_stmt);
    }
    Ok(rewritten)
}

pub(crate) fn rewrite_higher_order_calls_in_stmt(
    stmt: &AstStmt,
    current_return_type: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    method_template_lookup: &BTreeMap<(String, String), String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstStmt, String> {
    Ok(match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => AstStmt::Let {
            mutable: *mutable,
            name: name.clone(),
            ty: ty.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
                ty.as_ref(),
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::AssignLocal { name, value } => AstStmt::AssignLocal {
            name: name.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => AstStmt::DestructureLet {
            type_ref: type_ref.clone(),
            fields: fields.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
                type_ref.as_ref(),
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Const { name, ty, value } => AstStmt::Const {
            name: name.clone(),
            ty: ty.clone(),
            value: rewrite_higher_order_calls_in_expr(
                value,
                ty.as_ref(),
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Print(value) => AstStmt::Print(rewrite_higher_order_calls_in_expr(
            value,
            None,
            local_types,
            templates,
            function_table,
            module_impls,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Await(value) => AstStmt::Await(rewrite_higher_order_calls_in_expr(
            value,
            None,
            local_types,
            templates,
            function_table,
            module_impls,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => AstStmt::If {
            condition: rewrite_higher_order_calls_in_expr(
                condition,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Match { value, arms } => AstStmt::Match {
            value: rewrite_higher_order_calls_in_expr(
                value,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: arm
                            .guard
                            .as_ref()
                            .map(|guard| {
                                rewrite_higher_order_calls_in_expr(
                                    guard,
                                    None,
                                    local_types,
                                    templates,
                                    function_table,
                                    module_impls,
                                    method_template_lookup,
                                    visible_type_aliases,
                                    specialized_cache,
                                    specialized_functions,
                                )
                            })
                            .transpose()?,
                        body: rewrite_higher_order_calls_in_block(
                            &arm.body,
                            current_return_type,
                            local_types,
                            templates,
                            function_table,
                            module_impls,
                            visible_structs,
                            method_template_lookup,
                            visible_type_aliases,
                            specialized_cache,
                            specialized_functions,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstStmt::While { condition, body } => AstStmt::While {
            condition: rewrite_higher_order_calls_in_expr(
                condition,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            body: rewrite_higher_order_calls_in_block(
                body,
                current_return_type,
                local_types,
                templates,
                function_table,
                module_impls,
                visible_structs,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstStmt::Expr(expr) => AstStmt::Expr(rewrite_higher_order_calls_in_expr(
            expr,
            None,
            local_types,
            templates,
            function_table,
            module_impls,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?),
        AstStmt::Return(Some(value)) => AstStmt::Return(Some(rewrite_higher_order_calls_in_expr(
            value,
            current_return_type,
            local_types,
            templates,
            function_table,
            module_impls,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?)),
        AstStmt::Return(None) => AstStmt::Return(None),
        AstStmt::Break => AstStmt::Break,
        AstStmt::Continue => AstStmt::Continue,
    })
}

pub(crate) fn rewrite_higher_order_calls_in_expr(
    expr: &AstExpr,
    expected: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    method_template_lookup: &BTreeMap<(String, String), String>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_higher_order_calls_in_expr(
                condition,
                Some(&super::super::ast_named_type("bool")),
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            then_body: rewrite_higher_order_calls_in_block(
                then_body,
                expected,
                local_types,
                templates,
                function_table,
                module_impls,
                &BTreeMap::new(),
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
            else_body: rewrite_higher_order_calls_in_block(
                else_body,
                expected,
                local_types,
                templates,
                function_table,
                module_impls,
                &BTreeMap::new(),
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_higher_order_calls_in_expr(
                value,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            arms: arms
                .iter()
                .map(|arm| {
                    Ok(AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: match &arm.guard {
                            Some(guard) => Some(rewrite_higher_order_calls_in_expr(
                                guard,
                                Some(&super::super::ast_named_type("bool")),
                                local_types,
                                templates,
                                function_table,
                                module_impls,
                                method_template_lookup,
                                visible_type_aliases,
                                specialized_cache,
                                specialized_functions,
                            )?),
                            None => None,
                        },
                        body: rewrite_higher_order_calls_in_block(
                            &arm.body,
                            expected,
                            local_types,
                            templates,
                            function_table,
                            module_impls,
                            &BTreeMap::new(),
                            method_template_lookup,
                            visible_type_aliases,
                            specialized_cache,
                            specialized_functions,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if templates.contains_key(callee) => specialize_higher_order_call(
            callee,
            args,
            generic_args,
            None,
            expected,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?,
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_higher_order_calls_in_expr(
            value,
            expected,
            local_types,
            templates,
            function_table,
            module_impls,
            method_template_lookup,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?)),
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(rewrite_higher_order_calls_in_expr(
                operand,
                expected,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
        },
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => AstExpr::Call {
            callee: callee.clone(),
            generic_args: generic_args.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
                        None,
                        local_types,
                        templates,
                        function_table,
                        module_impls,
                        method_template_lookup,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::Invoke { callee, args } => AstExpr::Invoke {
            callee: Box::new(rewrite_higher_order_calls_in_expr(
                callee,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_higher_order_calls_in_expr(
                        arg,
                        None,
                        local_types,
                        templates,
                        function_table,
                        module_impls,
                        method_template_lookup,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            if let Some(receiver_ty) =
                infer_local_binding_type(receiver, local_types, function_table, module_impls)
            {
                if let Some(template_name) = method_template_lookup
                    .get(&(lower_type_ref(&receiver_ty).render(), method.clone()))
                {
                    let mut full_args = Vec::with_capacity(args.len() + 1);
                    full_args.push(receiver.as_ref().clone());
                    full_args.extend(args.iter().cloned());
                    return specialize_higher_order_call(
                        template_name,
                        &full_args,
                        &[],
                        None,
                        expected,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    );
                }
                if let Some(template_name) = find_generic_method_template_name(
                    &receiver_ty,
                    method,
                    templates,
                    visible_type_aliases,
                )? {
                    let mut full_args = Vec::with_capacity(args.len() + 1);
                    full_args.push(receiver.as_ref().clone());
                    full_args.extend(args.iter().cloned());
                    return specialize_higher_order_call(
                        &template_name,
                        &full_args,
                        &[],
                        None,
                        expected,
                        templates,
                        function_table,
                        visible_type_aliases,
                        specialized_cache,
                        specialized_functions,
                    );
                }
            }
            AstExpr::MethodCall {
                receiver: Box::new(rewrite_higher_order_calls_in_expr(
                    receiver,
                    None,
                    local_types,
                    templates,
                    function_table,
                    module_impls,
                    method_template_lookup,
                    visible_type_aliases,
                    specialized_cache,
                    specialized_functions,
                )?),
                method: method.clone(),
                generic_args: generic_args.clone(),
                args: args
                    .iter()
                    .map(|arg| {
                        rewrite_higher_order_calls_in_expr(
                            arg,
                            None,
                            local_types,
                            templates,
                            function_table,
                            module_impls,
                            method_template_lookup,
                            visible_type_aliases,
                            specialized_cache,
                            specialized_functions,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
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
                        rewrite_higher_order_calls_in_expr(
                            value,
                            None,
                            local_types,
                            templates,
                            function_table,
                            module_impls,
                            method_template_lookup,
                            visible_type_aliases,
                            specialized_cache,
                            specialized_functions,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_higher_order_calls_in_expr(
                base,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_higher_order_calls_in_expr(
                lhs,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
            rhs: Box::new(rewrite_higher_order_calls_in_expr(
                rhs,
                None,
                local_types,
                templates,
                function_table,
                module_impls,
                method_template_lookup,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?),
        },
        _ => expr.clone(),
    })
}

pub(crate) fn specialize_higher_order_call(
    callee: &str,
    args: &[AstExpr],
    explicit_generic_args: &[AstTypeRef],
    template_callable_bindings: Option<&BTreeMap<String, BoundCallable>>,
    expected: Option<&AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    let template = templates
        .get(callee)
        .ok_or_else(|| format!("unknown higher-order template `{callee}`"))?;
    if template.params.len() != args.len() {
        return Err(format!(
            "function `{}` expects {} args, found {}",
            callee,
            template.params.len(),
            args.len()
        ));
    }

    let explicit_substitutions = explicit_higher_order_generic_substitutions(
        template,
        explicit_generic_args,
        visible_type_aliases,
    )?;
    let mut callable_bindings = BTreeMap::<String, BoundCallable>::new();
    let mut ordinary_args = Vec::new();
    let mut callable_fragments = Vec::new();
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();

    for (param, arg) in template.params.iter().zip(args) {
        let callable_param = is_callable_type_with_aliases(&param.ty, visible_type_aliases)?;
        let ordinary_expected = higher_order_param_expected_type(
            template,
            param,
            &explicit_substitutions,
            expected,
            visible_type_aliases,
        );
        if callable_param {
            let rewritten_arg = rewrite_higher_order_argument_expr(
                arg,
                template_callable_bindings,
                ordinary_expected.as_ref(),
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?;
            let Some(bound_callable) =
                parse_bound_callable_expr(&rewritten_arg, template_callable_bindings)
            else {
                return Err(format!(
                    "higher-order parameter `{}` currently expects a lambda or named function symbol",
                    param.name
                ));
            };
            let callable_name = bound_callable.symbol.clone();
            if !function_table.contains_key(&callable_name) {
                return Err(format!(
                    "higher-order parameter `{}` references unknown callable `{}`",
                    param.name, callable_name
                ));
            }
            let callable = function_table.get(&callable_name).ok_or_else(|| {
                format!(
                    "higher-order parameter `{}` references unknown callable `{}`",
                    param.name, callable_name
                )
            })?;
            if !function_type_matches_callable(
                callable,
                &param.ty,
                &generic_names,
                visible_type_aliases,
            )? {
                return Err(format!(
                    "callable `{}` does not match higher-order parameter `{}` of type `{}`",
                    callable_name, param.name, param.ty.name
                ));
            }
            let resolved_callable_ty =
                resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
            let callable_arity = super::callables::callable_type_arity(&resolved_callable_ty)
                .ok_or_else(|| {
                    format!("higher-order parameter `{}` is not callable", param.name)
                })?;
            let capture_param_defs = callable
                .params
                .iter()
                .skip(callable_arity)
                .collect::<Vec<_>>();
            if capture_param_defs.len() != bound_callable.capture_args.len() {
                return Err(format!(
                    "callable `{}` capture shape does not match higher-order binding `{}`",
                    callable_name, param.name
                ));
            }
            let helper_capture_args = capture_param_defs
                .iter()
                .enumerate()
                .map(|(index, capture_param)| {
                    AstExpr::Var(format!(
                        "__capture_{}_{}_{}",
                        sanitize_symbol_fragment(&param.name),
                        sanitize_symbol_fragment(&capture_param.name),
                        index
                    ))
                })
                .collect::<Vec<_>>();
            for capture_arg in &bound_callable.capture_args {
                ordinary_args.push(annotate_expr_head_with_expected_type(
                    capture_arg.clone(),
                    None,
                ));
            }
            callable_bindings.insert(
                param.name.clone(),
                BoundCallable {
                    symbol: callable_name.clone(),
                    capture_args: helper_capture_args,
                },
            );
            callable_fragments.push(sanitize_symbol_fragment(&callable_name));
        } else {
            let rewritten_arg = rewrite_higher_order_argument_expr(
                arg,
                template_callable_bindings,
                ordinary_expected.as_ref(),
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?;
            ordinary_args.push(annotate_expr_head_with_expected_type(
                rewritten_arg,
                ordinary_expected.as_ref(),
            ));
        }
    }

    let specialized_name = format!(
        "__hof_{}_{}",
        sanitize_symbol_fragment(callee),
        callable_fragments.join("__")
    );
    if specialized_cache.insert(specialized_name.clone()) {
        let specialized = specialize_higher_order_template(
            template,
            &specialized_name,
            &callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?;
        specialized_functions.push(specialized.clone());
    }

    Ok(AstExpr::Call {
        callee: specialized_name,
        generic_args: explicit_generic_args.to_vec(),
        args: ordinary_args,
    })
}

fn rewrite_higher_order_argument_expr(
    expr: &AstExpr,
    template_callable_bindings: Option<&BTreeMap<String, BoundCallable>>,
    expected: Option<&AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    if let Some(bindings) = template_callable_bindings {
        return rewrite_higher_order_template_expr(
            expr,
            bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        );
    }
    rewrite_higher_order_calls_in_expr(
        expr,
        expected,
        &BTreeMap::new(),
        templates,
        function_table,
        &[],
        &BTreeMap::new(),
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    )
}

fn find_generic_method_template_name(
    receiver_ty: &AstTypeRef,
    method_name: &str,
    templates: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<Option<String>, String> {
    let resolved_receiver = resolve_ast_type_ref_aliases(receiver_ty, visible_type_aliases)?;
    let mut matches = Vec::new();
    for (template_name, template) in templates {
        if !template_name.starts_with("impl.") || template.name != *template_name {
            continue;
        }
        if template.params.is_empty() || template.name.rsplit('.').next() != Some(method_name) {
            continue;
        }
        let receiver_pattern =
            resolve_ast_type_ref_aliases(&template.params[0].ty, visible_type_aliases)?;
        let generic_names = collect_generic_type_names(&receiver_pattern);
        if generic_names.is_empty() {
            continue;
        }
        let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
        if unify_generic_type_pattern(
            &receiver_pattern,
            &resolved_receiver,
            &generic_names,
            &mut substitutions,
            template_name,
        )
        .is_ok()
        {
            matches.push(template_name.clone());
        }
    }
    if matches.len() > 1 {
        return Err(format!(
            "generic higher-order impl method resolution for `{}` is ambiguous; matching templates: {}",
            method_name,
            matches.join(", ")
        ));
    }
    Ok(matches.into_iter().next())
}

fn collect_generic_type_names(ty: &AstTypeRef) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_generic_type_names_into(ty, &mut names);
    names
}

fn collect_generic_type_names_into(ty: &AstTypeRef, names: &mut BTreeSet<String>) {
    if ty.generic_args.is_empty() {
        names.insert(ty.name.clone());
    }
    for arg in &ty.generic_args {
        collect_generic_type_names_into(arg, names);
    }
}

fn explicit_higher_order_generic_substitutions(
    template: &AstFunction,
    explicit_generic_args: &[AstTypeRef],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<BTreeMap<String, AstTypeRef>, String> {
    if explicit_generic_args.is_empty() {
        return Ok(BTreeMap::new());
    }
    if explicit_generic_args.len() != template.generic_params.len() {
        return Err(format!(
            "generic function `{}` expects {} explicit generic argument(s), found {}",
            template.name,
            template.generic_params.len(),
            explicit_generic_args.len()
        ));
    }
    template
        .generic_params
        .iter()
        .zip(explicit_generic_args.iter())
        .map(|(param, arg)| {
            Ok((
                param.name.clone(),
                resolve_ast_type_ref_aliases(arg, visible_type_aliases)?,
            ))
        })
        .collect()
}

fn higher_order_param_expected_type(
    template: &AstFunction,
    param: &AstParam,
    explicit_substitutions: &BTreeMap<String, AstTypeRef>,
    expected: Option<&AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Option<AstTypeRef> {
    if !explicit_substitutions.is_empty() {
        let lowered_substitutions = explicit_substitutions
            .iter()
            .map(|(name, ty)| (name.clone(), lower_type_ref(ty)))
            .collect::<BTreeMap<_, _>>();
        return specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok();
    }
    let expected = expected?;
    let return_pattern = template.return_type.as_ref()?;
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if generic_names.is_empty() {
        return None;
    }
    let resolved_return_pattern =
        resolve_ast_type_ref_aliases(return_pattern, visible_type_aliases).ok()?;
    let resolved_expected = resolve_ast_type_ref_aliases(expected, visible_type_aliases).ok()?;
    let mut substitutions = BTreeMap::<String, AstTypeRef>::new();
    unify_generic_type_pattern(
        &resolved_return_pattern,
        &resolved_expected,
        &generic_names,
        &mut substitutions,
        &template.name,
    )
    .ok()?;
    let lowered_substitutions = substitutions
        .into_iter()
        .map(|(name, ty)| (name, lower_type_ref(&ty)))
        .collect::<BTreeMap<_, _>>();
    specialize_ast_type_ref(&param.ty, &lowered_substitutions).ok()
}

fn annotate_expr_head_with_expected_type(expr: AstExpr, expected: Option<&AstTypeRef>) -> AstExpr {
    let Some(expected) = expected else {
        return expr;
    };
    match expr {
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty()
            && callee == expected.name
            && !expected.generic_args.is_empty() =>
        {
            AstExpr::Call {
                callee,
                generic_args: expected.generic_args.clone(),
                args,
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if type_args.is_empty()
            && type_name == expected.name
            && !expected.generic_args.is_empty() =>
        {
            AstExpr::StructLiteral {
                type_name,
                type_args: expected.generic_args.clone(),
                fields,
            }
        }
        other => other,
    }
}
