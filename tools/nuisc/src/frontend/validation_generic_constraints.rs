use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstEnumDef, AstEnumVariantKind, AstFunction, AstImplDef, AstModule, AstStructDef, AstTypeAlias,
    AstTypeRef,
};

use super::build_function_return_type_table;
use super::function_context::render_function_validation_context;
use super::validation_binding_env::collect_visible_structs;
use super::validation_method_bounds::collect_visible_trait_methods;
use super::validation_trait_bounds::{build_generic_bound_env, collect_visible_trait_names};

#[path = "validation_generic_constraints_coherence.rs"]
mod validation_generic_constraints_coherence;
#[path = "validation_generic_constraints_expr.rs"]
mod validation_generic_constraints_expr;
#[path = "validation_generic_constraints_stmt.rs"]
mod validation_generic_constraints_stmt;
#[path = "validation_generic_constraints_types.rs"]
mod validation_generic_constraints_types;

use self::validation_generic_constraints_coherence::{
    render_impl_target_type, validate_trait_impl_coherence,
};
use self::validation_generic_constraints_stmt::validate_stmt_generic_constraints;
use self::validation_generic_constraints_types::validate_ast_type_ref_generic_constraints;

fn collect_visible_enums(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
) -> BTreeMap<String, AstEnumDef> {
    let mut enums = module
        .enums
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    for helper in local_cpu_helpers {
        for definition in helper
            .enums
            .iter()
            .filter(|definition| super::is_public_visibility(definition.visibility))
        {
            enums.insert(definition.name.clone(), definition.clone());
        }
    }
    enums
}

pub(super) fn validate_ast_generic_constraints(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
) -> Result<(), String> {
    let visible_trait_names = collect_visible_trait_names(module, local_cpu_helpers);
    let visible_trait_methods = collect_visible_trait_methods(module, local_cpu_helpers);
    let visible_structs = collect_visible_structs(module, local_cpu_helpers);
    let visible_enums = collect_visible_enums(module, local_cpu_helpers);
    let concrete_module_functions = module
        .functions
        .iter()
        .filter(|function| function.generic_params.is_empty())
        .cloned()
        .collect::<Vec<_>>();
    let generic_templates = module
        .functions
        .iter()
        .filter(|function| !function.generic_params.is_empty())
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    let function_return_types = build_function_return_type_table(
        module,
        &concrete_module_functions,
        &generic_templates,
        local_cpu_helpers,
        visible_type_aliases,
    );

    for alias in &module.type_aliases {
        let generic_bounds = build_generic_bound_env(
            &alias.generic_params,
            &alias.where_bounds,
            &visible_trait_names,
            &format!("type alias `{}`", alias.name),
        )?;
        validate_ast_type_ref_generic_constraints(
            &alias.target,
            visible_type_aliases,
            impl_lookup,
            &visible_trait_names,
            &visible_structs,
            &visible_enums,
            &generic_bounds,
            &format!("type alias `{}` target", alias.name),
        )?;
    }

    for definition in &module.structs {
        let generic_bounds = build_generic_bound_env(
            &definition.generic_params,
            &definition.where_bounds,
            &visible_trait_names,
            &format!("struct `{}`", definition.name),
        )?;
        for field in &definition.fields {
            validate_ast_type_ref_generic_constraints(
                &field.ty,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
                &visible_structs,
                &visible_enums,
                &generic_bounds,
                &format!("struct `{}` field `{}`", definition.name, field.name),
            )?;
        }
    }

    for definition in &module.enums {
        let generic_bounds = build_generic_bound_env(
            &definition.generic_params,
            &definition.where_bounds,
            &visible_trait_names,
            &format!("enum `{}`", definition.name),
        )?;
        for variant in &definition.variants {
            match &variant.kind {
                AstEnumVariantKind::Unit => {}
                AstEnumVariantKind::Tuple(fields) => {
                    for (index, field_ty) in fields.iter().enumerate() {
                        validate_ast_type_ref_generic_constraints(
                            field_ty,
                            visible_type_aliases,
                            impl_lookup,
                            &visible_trait_names,
                            &visible_structs,
                            &visible_enums,
                            &generic_bounds,
                            &format!(
                                "enum `{}` variant `{}` tuple field #{}",
                                definition.name, variant.name, index
                            ),
                        )?;
                    }
                }
                AstEnumVariantKind::Struct(fields) => {
                    for field in fields {
                        validate_ast_type_ref_generic_constraints(
                            &field.ty,
                            visible_type_aliases,
                            impl_lookup,
                            &visible_trait_names,
                            &visible_structs,
                            &visible_enums,
                            &generic_bounds,
                            &format!(
                                "enum `{}` variant `{}` field `{}`",
                                definition.name, variant.name, field.name
                            ),
                        )?;
                    }
                }
            }
        }
    }

    for function in &module.externs {
        for param in &function.params {
            validate_ast_type_ref_generic_constraints(
                &param.ty,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
                &visible_structs,
                &visible_enums,
                &BTreeMap::new(),
                &format!(
                    "extern function `{}` parameter `{}`",
                    function.name, param.name
                ),
            )?;
        }
        validate_ast_type_ref_generic_constraints(
            &function.return_type,
            visible_type_aliases,
            impl_lookup,
            &visible_trait_names,
            &visible_structs,
            &visible_enums,
            &BTreeMap::new(),
            &format!("extern function `{}` return type", function.name),
        )?;
    }

    for interface in &module.extern_interfaces {
        for method in &interface.methods {
            for param in &method.params {
                validate_ast_type_ref_generic_constraints(
                    &param.ty,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &visible_structs,
                    &visible_enums,
                    &BTreeMap::new(),
                    &format!(
                        "extern interface `{}` method `{}` parameter `{}`",
                        interface.name, method.name, param.name
                    ),
                )?;
            }
            validate_ast_type_ref_generic_constraints(
                &method.return_type,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
                &visible_structs,
                &visible_enums,
                &BTreeMap::new(),
                &format!(
                    "extern interface `{}` method `{}` return type",
                    interface.name, method.name
                ),
            )?;
        }
    }

    for definition in &module.traits {
        for method in &definition.methods {
            for param in &method.params {
                validate_ast_type_ref_generic_constraints(
                    &param.ty,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &visible_structs,
                    &visible_enums,
                    &BTreeMap::new(),
                    &format!(
                        "trait `{}` method `{}` parameter `{}`",
                        definition.name, method.name, param.name
                    ),
                )?;
            }
            if let Some(return_type) = &method.return_type {
                validate_ast_type_ref_generic_constraints(
                    return_type,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &visible_structs,
                    &visible_enums,
                    &BTreeMap::new(),
                    &format!(
                        "trait `{}` method `{}` return type",
                        definition.name, method.name
                    ),
                )?;
            }
        }
    }

    for definition in &module.impls {
        let impl_context = format!(
            "impl `{}` for `{}`",
            definition.trait_name,
            render_impl_target_type(&definition.for_type, visible_type_aliases)?
        );
        let generic_bounds = build_generic_bound_env(
            &definition.generic_params,
            &definition.where_bounds,
            &visible_trait_names,
            &impl_context,
        )?;
        let generic_param_names = definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        validate_ast_type_ref_generic_constraints(
            &definition.for_type,
            visible_type_aliases,
            impl_lookup,
            &visible_trait_names,
            &visible_structs,
            &visible_enums,
            &generic_bounds,
            &format!("impl `{}` target type", definition.trait_name),
        )?;
        for method in &definition.methods {
            for param in &method.params {
                validate_ast_type_ref_generic_constraints(
                    &param.ty,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &visible_structs,
                    &visible_enums,
                    &generic_bounds,
                    &format!(
                        "impl `{}` method `{}` parameter `{}`",
                        definition.trait_name, method.name, param.name
                    ),
                )?;
            }
            if let Some(return_type) = &method.return_type {
                validate_ast_type_ref_generic_constraints(
                    return_type,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &visible_structs,
                    &visible_enums,
                    &generic_bounds,
                    &format!(
                        "impl `{}` method `{}` return type",
                        definition.trait_name, method.name
                    ),
                )?;
            }
            let synthetic_function = AstFunction {
                name: format!("impl.{}.{}", definition.trait_name, method.name),
                visibility: nuis_semantics::model::AstVisibility::Private,
                attributes: vec![],
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
                is_async: false,
                generic_params: definition.generic_params.clone(),
                where_bounds: definition.where_bounds.clone(),
                params: method.params.clone(),
                return_type: method.return_type.clone(),
                body: method.body.clone(),
            };
            validate_function_generic_constraints(
                &synthetic_function,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
                &visible_trait_methods,
                &visible_structs,
                &visible_enums,
                &function_return_types,
            )?;
            let mut local_type_env = method
                .params
                .iter()
                .map(|param| (param.name.clone(), param.ty.clone()))
                .collect::<BTreeMap<_, _>>();
            for stmt in &method.body {
                validate_stmt_generic_constraints(
                    stmt,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &visible_trait_methods,
                    &visible_structs,
                    &visible_enums,
                    &function_return_types,
                    &generic_param_names,
                    &generic_bounds,
                    &mut local_type_env,
                    &format!("{impl_context} method `{}` body", method.name),
                )?;
            }
        }
    }

    validate_trait_impl_coherence(
        module,
        local_cpu_helpers,
        visible_type_aliases,
        &visible_trait_names,
    )?;

    for constant in &module.consts {
        if let Some(ty) = &constant.ty {
            validate_ast_type_ref_generic_constraints(
                ty,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
                &visible_structs,
                &visible_enums,
                &BTreeMap::new(),
                &format!("const `{}` type", constant.name),
            )?;
        }
    }

    for function in &module.functions {
        validate_function_generic_constraints(
            function,
            visible_type_aliases,
            impl_lookup,
            &visible_trait_names,
            &visible_trait_methods,
            &visible_structs,
            &visible_enums,
            &function_return_types,
        )?;
    }

    Ok(())
}

fn validate_function_generic_constraints(
    function: &AstFunction,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_trait_methods: &BTreeMap<String, BTreeSet<String>>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<(), String> {
    let function_context = render_function_validation_context(&function.name);
    let generic_bounds = build_generic_bound_env(
        &function.generic_params,
        &function.where_bounds,
        visible_trait_names,
        &function_context,
    )?;
    let generic_param_names = function
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut local_type_env = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for param in &function.params {
        validate_ast_type_ref_generic_constraints(
            &param.ty,
            visible_type_aliases,
            impl_lookup,
            visible_trait_names,
            visible_structs,
            visible_enums,
            &generic_bounds,
            &format!("{function_context} parameter `{}`", param.name),
        )?;
    }
    if let Some(return_type) = &function.return_type {
        validate_ast_type_ref_generic_constraints(
            return_type,
            visible_type_aliases,
            impl_lookup,
            visible_trait_names,
            visible_structs,
            visible_enums,
            &generic_bounds,
            &format!("{function_context} return type"),
        )?;
    }
    for stmt in &function.body {
        validate_stmt_generic_constraints(
            stmt,
            visible_type_aliases,
            impl_lookup,
            visible_trait_names,
            visible_trait_methods,
            visible_structs,
            visible_enums,
            function_return_types,
            &generic_param_names,
            &generic_bounds,
            &mut local_type_env,
            &format!("{function_context} body"),
        )?;
    }
    Ok(())
}
