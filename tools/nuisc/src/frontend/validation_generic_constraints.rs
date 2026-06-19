use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstEnumDef, AstEnumVariantKind, AstExpr, AstFunction, AstImplDef, AstImplMethod, AstMatchArm,
    AstModule, AstStmt, AstStructDef, AstTraitDef, AstTraitMethodSig, AstTypeAlias, AstTypeRef,
};

use super::function_context::render_function_validation_context;
use super::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type, collect_visible_structs,
    simple_match_value_type,
};
use super::validation_method_bounds::{
    collect_visible_trait_methods, validate_expr_generic_method_bounds,
};
use super::validation_trait_bounds::{
    alias_param_context, alias_target_context, build_generic_bound_env,
    collect_visible_trait_names, validate_generic_bound_satisfaction,
};
use super::{
    build_function_return_type_table, infer_ast_expr_type, lower_type_ref,
    lower_type_ref_with_aliases, substitute_ast_type_alias_target,
};

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

fn substitute_self_type(ty: &AstTypeRef, self_type: &AstTypeRef) -> AstTypeRef {
    let substituted = if ty.name == "Self" && ty.generic_args.is_empty() {
        self_type.clone()
    } else {
        AstTypeRef {
            name: ty.name.clone(),
            generic_args: ty
                .generic_args
                .iter()
                .map(|arg| substitute_self_type(arg, self_type))
                .collect(),
            is_optional: ty.is_optional,
            is_ref: ty.is_ref,
        }
    };
    AstTypeRef {
        is_optional: ty.is_optional || substituted.is_optional,
        is_ref: ty.is_ref || substituted.is_ref,
        ..substituted
    }
}

fn render_impl_target_type(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<String, String> {
    Ok(lower_type_ref_with_aliases(ty, visible_type_aliases)?.render())
}

fn impl_targets_overlap(
    lhs: &AstTypeRef,
    lhs_generic_params: &BTreeSet<String>,
    rhs: &AstTypeRef,
    rhs_generic_params: &BTreeSet<String>,
) -> bool {
    if lhs.is_optional != rhs.is_optional || lhs.is_ref != rhs.is_ref {
        return false;
    }
    if lhs_generic_params.contains(&lhs.name) && lhs.generic_args.is_empty() {
        return true;
    }
    if rhs_generic_params.contains(&rhs.name) && rhs.generic_args.is_empty() {
        return true;
    }
    if lhs.name != rhs.name || lhs.generic_args.len() != rhs.generic_args.len() {
        return false;
    }
    lhs.generic_args
        .iter()
        .zip(&rhs.generic_args)
        .all(|(lhs_arg, rhs_arg)| {
            impl_targets_overlap(lhs_arg, lhs_generic_params, rhs_arg, rhs_generic_params)
        })
}

fn validate_impl_method_signature_matches_trait(
    trait_name: &str,
    for_type: &AstTypeRef,
    trait_method: &AstTraitMethodSig,
    impl_method: &AstImplMethod,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<(), String> {
    if trait_method.params.len() != impl_method.params.len() {
        return Err(format!(
            "method `{}` in impl `{}` for `{}` does not match trait signature",
            impl_method.name,
            trait_name,
            render_impl_target_type(for_type, visible_type_aliases)?,
        ));
    }
    for (trait_param, impl_param) in trait_method.params.iter().zip(&impl_method.params) {
        let expected = lower_type_ref_with_aliases(
            &substitute_self_type(&trait_param.ty, for_type),
            visible_type_aliases,
        )?
        .render();
        let actual = lower_type_ref_with_aliases(&impl_param.ty, visible_type_aliases)?.render();
        if expected != actual {
            return Err(format!(
                "method `{}` in impl `{}` for `{}` does not match trait signature",
                impl_method.name,
                trait_name,
                render_impl_target_type(for_type, visible_type_aliases)?,
            ));
        }
    }
    let expected_return = trait_method
        .return_type
        .as_ref()
        .map(|ty| {
            lower_type_ref_with_aliases(&substitute_self_type(ty, for_type), visible_type_aliases)
                .map(|ty| ty.render())
        })
        .transpose()?;
    let actual_return = impl_method
        .return_type
        .as_ref()
        .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases).map(|ty| ty.render()))
        .transpose()?;
    if expected_return != actual_return {
        return Err(format!(
            "method `{}` in impl `{}` for `{}` does not match trait signature",
            impl_method.name,
            trait_name,
            render_impl_target_type(for_type, visible_type_aliases)?,
        ));
    }
    Ok(())
}

fn validate_trait_impl_coherence(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    visible_trait_names: &BTreeSet<String>,
) -> Result<(), String> {
    let mut trait_defs = module
        .traits
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<String, AstTraitDef>>();
    for helper in local_cpu_helpers {
        for definition in helper.traits.iter().filter(|definition| {
            matches!(
                definition.visibility,
                nuis_semantics::model::AstVisibility::Public
            )
        }) {
            trait_defs.insert(definition.name.clone(), definition.clone());
            trait_defs.insert(
                format!("{}.{}", helper.unit, definition.name),
                definition.clone(),
            );
        }
    }

    let mut seen_impls = BTreeSet::new();
    let mut prior_impls = Vec::<(&AstImplDef, String)>::new();
    for definition in &module.impls {
        if !visible_trait_names.contains(&definition.trait_name) {
            return Err(format!(
                "impl references unknown trait `{}`",
                definition.trait_name
            ));
        }
        let rendered_for_type =
            render_impl_target_type(&definition.for_type, visible_type_aliases)?;
        if !seen_impls.insert((definition.trait_name.clone(), rendered_for_type.clone())) {
            return Err(format!(
                "duplicate impl for trait `{}` and type `{}`",
                definition.trait_name, rendered_for_type
            ));
        }
        let definition_generic_params = definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>();
        for (prior_definition, prior_rendered_for_type) in &prior_impls {
            if prior_definition.trait_name != definition.trait_name {
                continue;
            }
            let prior_generic_params = prior_definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            if impl_targets_overlap(
                &definition.for_type,
                &definition_generic_params,
                &prior_definition.for_type,
                &prior_generic_params,
            ) {
                return Err(format!(
                    "overlapping impls for trait `{}` between `{}` and `{}`",
                    definition.trait_name, prior_rendered_for_type, rendered_for_type
                ));
            }
        }
        let Some(trait_def) = trait_defs.get(&definition.trait_name) else {
            return Err(format!(
                "impl references unknown trait `{}`",
                definition.trait_name
            ));
        };
        let trait_methods = trait_def
            .methods
            .iter()
            .map(|method| (method.name.clone(), method))
            .collect::<BTreeMap<_, _>>();
        let mut impl_methods = BTreeMap::new();
        for method in &definition.methods {
            if impl_methods.insert(method.name.clone(), method).is_some() {
                return Err(format!(
                    "impl `{}` for `{}` declares duplicate method `{}`",
                    definition.trait_name, rendered_for_type, method.name
                ));
            }
        }
        for trait_method in &trait_def.methods {
            let Some(impl_method) = impl_methods.get(&trait_method.name) else {
                if trait_method.default_body.is_none() {
                    return Err(format!(
                        "impl `{}` for `{}` is missing required trait method `{}`",
                        definition.trait_name, rendered_for_type, trait_method.name
                    ));
                }
                continue;
            };
            validate_impl_method_signature_matches_trait(
                &definition.trait_name,
                &definition.for_type,
                trait_method,
                impl_method,
                visible_type_aliases,
            )?;
        }
        for impl_method in &definition.methods {
            if !trait_methods.contains_key(&impl_method.name) {
                return Err(format!(
                    "extra impl method `{}` is not declared by trait `{}`",
                    impl_method.name, definition.trait_name
                ));
            }
        }
        prior_impls.push((definition, rendered_for_type));
    }
    Ok(())
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

fn validate_expr_generic_constraints(
    expr: &AstExpr,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_trait_methods: &BTreeMap<String, BTreeSet<String>>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match expr {
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => {}
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr_generic_constraints(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut then_env = local_type_env.clone();
            for nested in then_body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut then_env,
                    &format!("{context} if-then"),
                )?;
            }
            let mut else_env = local_type_env.clone();
            for nested in else_body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut else_env,
                    &format!("{context} if-else"),
                )?;
            }
        }
        AstExpr::Match { value, arms } => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let match_value_ty = simple_match_value_type(value, local_type_env);
            for AstMatchArm {
                pattern,
                guard,
                body,
            } in arms
            {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        &pattern,
                        visible_type_aliases,
                        visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = guard {
                    validate_expr_generic_constraints(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_trait_names,
                        visible_trait_methods,
                        visible_structs,
                        visible_enums,
                        function_return_types,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                }
                for nested in body {
                    validate_stmt_generic_constraints(
                        &nested,
                        visible_type_aliases,
                        impl_lookup,
                        visible_trait_names,
                        visible_trait_methods,
                        visible_structs,
                        visible_enums,
                        function_return_types,
                        generic_param_names,
                        generic_bounds,
                        &mut arm_env,
                        &format!("{context} match-arm"),
                    )?;
                }
            }
        }
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => {
            let mut lambda_env = local_type_env.clone();
            for param in params {
                validate_ast_type_ref_generic_constraints(
                    &param.ty,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} lambda parameter `{}`", param.name),
                )?;
                lambda_env.insert(param.name.clone(), param.ty.clone());
            }
            if let Some(return_type) = return_type {
                validate_ast_type_ref_generic_constraints(
                    return_type,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} lambda return type"),
                )?;
            }
            for nested in body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut lambda_env,
                    &format!("{context} lambda body"),
                )?;
            }
        }
        AstExpr::Try(value) | AstExpr::Await(value) | AstExpr::FieldAccess { base: value, .. } => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            for (index, generic_arg) in generic_args.iter().enumerate() {
                validate_ast_type_ref_generic_constraints(
                    generic_arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} call `{callee}` generic argument #{}", index + 1),
                )?;
            }
            for arg in args {
                validate_expr_generic_constraints(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Invoke { callee, args } => {
            validate_expr_generic_constraints(
                callee,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            for arg in args {
                validate_expr_generic_constraints(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::MethodCall {
            receiver,
            generic_args,
            args,
            ..
        } => {
            validate_expr_generic_constraints(
                receiver,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            for (index, generic_arg) in generic_args.iter().enumerate() {
                validate_ast_type_ref_generic_constraints(
                    generic_arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} method call generic argument #{}", index + 1),
                )?;
            }
            for arg in args {
                validate_expr_generic_constraints(
                    arg,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let literal_ty = AstTypeRef {
                name: type_name
                    .rsplit_once('.')
                    .and_then(|(parent, _)| visible_enums.contains_key(parent).then_some(parent))
                    .unwrap_or(type_name)
                    .to_owned(),
                generic_args: type_args.clone(),
                is_optional: false,
                is_ref: false,
            };
            validate_ast_type_ref_generic_constraints(
                &literal_ty,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_structs,
                visible_enums,
                generic_bounds,
                &format!("{context} struct literal `{type_name}`"),
            )?;
            for (_, value) in fields {
                validate_expr_generic_constraints(
                    value,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    local_type_env,
                    context,
                )?;
            }
        }
        AstExpr::Unary { operand, .. } => {
            validate_expr_generic_constraints(
                operand,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_expr_generic_constraints(
                lhs,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_constraints(
                rhs,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
    }
    Ok(())
}

fn validate_stmt_generic_constraints(
    stmt: &AstStmt,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_trait_methods: &BTreeMap<String, BTreeSet<String>>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let { name, ty, .. } | AstStmt::Const { name, ty, .. } => {
            let value = match stmt {
                AstStmt::Let { value, .. } | AstStmt::Const { value, .. } => value,
                _ => unreachable!(),
            };
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(ty) = ty {
                // Keep explicit annotation validation as its own pass after
                // walking the value expression. In deep expected-type chains,
                // this is where constrained aliases intentionally get a chance
                // to report their own bound failure context, even if an inner
                // generic call was also inferable from the same expected type.
                validate_ast_type_ref_generic_constraints(
                    ty,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} local `{name}`"),
                )?;
            }
            if let Some(inferred_ty) = ty.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                )
            }) {
                let name = match stmt {
                    AstStmt::Let { name, .. } | AstStmt::Const { name, .. } => name.clone(),
                    _ => unreachable!(),
                };
                local_type_env.insert(name, inferred_ty);
            }
        }
        AstStmt::AssignLocal { name, value } => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(inferred_ty) = infer_ast_expr_type(
                value,
                local_type_env,
                impl_lookup,
                visible_structs,
                function_return_types,
            )
            .or_else(|| local_type_env.get(name).cloned())
            {
                local_type_env.insert(name.clone(), inferred_ty);
            }
        }
        AstStmt::DestructureLet { type_ref, .. } => {
            let value = match stmt {
                AstStmt::DestructureLet { value, .. } => value,
                _ => unreachable!(),
            };
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            if let Some(type_ref) = type_ref {
                validate_ast_type_ref_generic_constraints(
                    type_ref,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_structs,
                    visible_enums,
                    generic_bounds,
                    &format!("{context} destructure type"),
                )?;
            }
            let fields = match stmt {
                AstStmt::DestructureLet { fields, .. } => fields,
                _ => unreachable!(),
            };
            let root_type = type_ref.clone().or_else(|| {
                infer_ast_expr_type(
                    value,
                    local_type_env,
                    impl_lookup,
                    visible_structs,
                    function_return_types,
                )
            });
            if let Some(root_type) = root_type.as_ref() {
                bind_destructure_fields_for_type(
                    root_type,
                    fields,
                    visible_type_aliases,
                    visible_structs,
                    local_type_env,
                )?;
            }
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            validate_expr_generic_constraints(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut then_env = local_type_env.clone();
            for nested in then_body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut then_env,
                    context,
                )?;
            }
            let mut else_env = local_type_env.clone();
            for nested in else_body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut else_env,
                    context,
                )?;
            }
        }
        AstStmt::Match { value, arms } => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let match_value_ty = simple_match_value_type(value, local_type_env);
            for AstMatchArm {
                pattern,
                guard,
                body,
            } in arms
            {
                let mut arm_env = local_type_env.clone();
                if let Some(match_value_ty) = match_value_ty.as_ref() {
                    bind_match_pattern_for_type(
                        match_value_ty,
                        pattern,
                        visible_type_aliases,
                        visible_structs,
                        &mut arm_env,
                    )?;
                }
                if let Some(guard) = guard {
                    validate_expr_generic_constraints(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_trait_names,
                        visible_trait_methods,
                        visible_structs,
                        visible_enums,
                        function_return_types,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                    validate_expr_generic_method_bounds(
                        guard,
                        visible_type_aliases,
                        impl_lookup,
                        visible_structs,
                        function_return_types,
                        visible_trait_methods,
                        generic_param_names,
                        generic_bounds,
                        &arm_env,
                        context,
                    )?;
                }
                for nested in body {
                    validate_stmt_generic_constraints(
                        nested,
                        visible_type_aliases,
                        impl_lookup,
                        visible_trait_names,
                        visible_trait_methods,
                        visible_structs,
                        visible_enums,
                        function_return_types,
                        generic_param_names,
                        generic_bounds,
                        &mut arm_env,
                        context,
                    )?;
                }
            }
        }
        AstStmt::While { condition, body } => {
            validate_expr_generic_constraints(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                condition,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            let mut loop_env = local_type_env.clone();
            for nested in body {
                validate_stmt_generic_constraints(
                    nested,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
                    visible_trait_methods,
                    visible_structs,
                    visible_enums,
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut loop_env,
                    context,
                )?;
            }
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstStmt::Return(Some(value)) => {
            validate_expr_generic_constraints(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_trait_names,
                visible_trait_methods,
                visible_structs,
                visible_enums,
                function_return_types,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
            validate_expr_generic_method_bounds(
                value,
                visible_type_aliases,
                impl_lookup,
                visible_structs,
                function_return_types,
                visible_trait_methods,
                generic_param_names,
                generic_bounds,
                local_type_env,
                context,
            )?;
        }
        AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
    }
    Ok(())
}

fn validate_ast_type_ref_generic_constraints(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    context: &str,
) -> Result<(), String> {
    let mut visiting = BTreeSet::new();
    validate_ast_type_ref_generic_constraints_inner(
        ty,
        visible_type_aliases,
        impl_lookup,
        visible_trait_names,
        visible_structs,
        visible_enums,
        generic_bounds,
        context,
        &mut visiting,
    )
}

fn validate_ast_type_ref_generic_constraints_inner(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_trait_names: &BTreeSet<String>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_enums: &BTreeMap<String, AstEnumDef>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    context: &str,
    visiting: &mut BTreeSet<String>,
) -> Result<(), String> {
    for arg in &ty.generic_args {
        validate_ast_type_ref_generic_constraints_inner(
            arg,
            visible_type_aliases,
            impl_lookup,
            visible_trait_names,
            visible_structs,
            visible_enums,
            generic_bounds,
            context,
            visiting,
        )?;
    }

    if let Some(struct_definition) = visible_structs.get(&ty.name) {
        if struct_definition.generic_params.len() == ty.generic_args.len() {
            let struct_bounds = build_generic_bound_env(
                &struct_definition.generic_params,
                &struct_definition.where_bounds,
                visible_trait_names,
                &format!("struct `{}`", struct_definition.name),
            )?;
            for (param, arg) in struct_definition
                .generic_params
                .iter()
                .zip(&ty.generic_args)
            {
                if let Some(bounds) = struct_bounds.get(&param.name) {
                    for bound_name in bounds {
                        validate_generic_bound_satisfaction(
                            arg,
                            bound_name,
                            visible_type_aliases,
                            impl_lookup,
                            generic_bounds,
                            &format!(
                                "{context} via struct `{}` generic parameter `{}`",
                                struct_definition.name, param.name
                            ),
                        )?;
                    }
                }
            }
        }
    }

    if let Some(enum_definition) = visible_enums.get(&ty.name) {
        if enum_definition.generic_params.len() == ty.generic_args.len() {
            let enum_bounds = build_generic_bound_env(
                &enum_definition.generic_params,
                &enum_definition.where_bounds,
                visible_trait_names,
                &format!("enum `{}`", enum_definition.name),
            )?;
            for (param, arg) in enum_definition.generic_params.iter().zip(&ty.generic_args) {
                if let Some(bounds) = enum_bounds.get(&param.name) {
                    for bound_name in bounds {
                        validate_generic_bound_satisfaction(
                            arg,
                            bound_name,
                            visible_type_aliases,
                            impl_lookup,
                            generic_bounds,
                            &format!(
                                "{context} via enum `{}` generic parameter `{}`",
                                enum_definition.name, param.name
                            ),
                        )?;
                    }
                }
            }
        }
    }

    let Some(alias_definition) = visible_type_aliases.get(&ty.name) else {
        return Ok(());
    };
    if alias_definition.generic_params.len() != ty.generic_args.len() {
        return Ok(());
    }

    let visit_key = lower_type_ref(ty).render();
    if !visiting.insert(visit_key.clone()) {
        return Ok(());
    }

    let alias_bounds = build_generic_bound_env(
        &alias_definition.generic_params,
        &alias_definition.where_bounds,
        visible_trait_names,
        &format!("type alias `{}`", alias_definition.name),
    )?;
    // Alias parameter bounds are checked before expanding the alias target.
    // That makes constrained aliases the current diagnostic owner for deep
    // expected-type chains that successfully reconstruct all the way out to an
    // alias application like Alias<Text>.
    for (param, arg) in alias_definition.generic_params.iter().zip(&ty.generic_args) {
        if let Some(bounds) = alias_bounds.get(&param.name) {
            for bound_name in bounds {
                validate_generic_bound_satisfaction(
                    arg,
                    bound_name,
                    visible_type_aliases,
                    impl_lookup,
                    generic_bounds,
                    &alias_param_context(context, &alias_definition.name, &param.name),
                )?;
            }
        }
    }

    let substitutions = alias_definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .zip(ty.generic_args.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    let expanded = substitute_ast_type_alias_target(&alias_definition.target, &substitutions)?;
    let expanded_context = alias_target_context(context, &alias_definition.name);
    validate_ast_type_ref_generic_constraints_inner(
        &expanded,
        visible_type_aliases,
        impl_lookup,
        visible_trait_names,
        visible_structs,
        visible_enums,
        generic_bounds,
        &expanded_context,
        visiting,
    )?;
    visiting.remove(&visit_key);
    Ok(())
}
