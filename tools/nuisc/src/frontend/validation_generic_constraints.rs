use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstImplMethod, AstMatchArm, AstModule, AstStmt, AstStructDef,
    AstTraitDef, AstTraitMethodSig, AstTypeAlias, AstTypeRef,
};

use super::validation_binding_env::{
    bind_destructure_fields_for_type, bind_match_pattern_for_type, collect_visible_structs,
    simple_match_value_type,
};
use super::validation_method_bounds::{
    collect_visible_trait_methods, validate_expr_generic_method_bounds,
};
use super::validation_trait_bounds::{
    alias_param_context, alias_target_context, build_generic_bound_env,
    collect_visible_trait_names, validate_generic_bound_satisfaction, validate_generic_bound_type,
};
use super::{
    build_function_return_type_table, infer_ast_expr_type, lower_type_ref,
    lower_type_ref_with_aliases,
    substitute_ast_type_alias_target,
};

fn render_validation_function_context(function_name: &str) -> String {
    if let Some(owner) = lambda_owner_name(function_name) {
        format!("function `{owner}` body lambda")
    } else if let Some(template) = higher_order_template_name(function_name) {
        format!("function `{template}` body higher-order specialization")
    } else {
        format!("function `{function_name}`")
    }
}

fn lambda_owner_name(function_name: &str) -> Option<&str> {
    let remainder = function_name.strip_prefix("__lambda_")?;
    let unspecialized = remainder
        .split_once("__")
        .map(|(base, _)| base)
        .unwrap_or(remainder);
    let (owner, counter) = unspecialized.rsplit_once('_')?;
    counter
        .chars()
        .all(|ch| ch.is_ascii_digit())
        .then_some(owner)
}

fn higher_order_template_name(function_name: &str) -> Option<&str> {
    let remainder = function_name.strip_prefix("__hof_")?;
    remainder
        .split_once("___lambda_")
        .map(|(template, _)| template)
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
            matches!(definition.visibility, nuis_semantics::model::AstVisibility::Public)
        }) {
            trait_defs.insert(definition.name.clone(), definition.clone());
            trait_defs.insert(
                format!("{}.{}", helper.unit, definition.name),
                definition.clone(),
            );
        }
    }

    let mut seen_impls = BTreeSet::new();
    for definition in &module.impls {
        if !visible_trait_names.contains(&definition.trait_name) {
            return Err(format!(
                "impl references unknown trait `{}`",
                definition.trait_name
            ));
        }
        let rendered_for_type = render_impl_target_type(&definition.for_type, visible_type_aliases)?;
        if !seen_impls.insert((definition.trait_name.clone(), rendered_for_type.clone())) {
            return Err(format!(
                "duplicate impl for trait `{}` and type `{}`",
                definition.trait_name, rendered_for_type
            ));
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
                return Err(format!(
                    "impl `{}` for `{}` is missing required trait method `{}`",
                    definition.trait_name, rendered_for_type, trait_method.name
                ));
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
            &visible_trait_names,
            &format!("type alias `{}`", alias.name),
        )?;
        validate_ast_type_ref_generic_constraints(
            &alias.target,
            visible_type_aliases,
            impl_lookup,
            &visible_trait_names,
            &generic_bounds,
            &format!("type alias `{}` target", alias.name),
        )?;
    }

    for definition in &module.structs {
        let generic_bounds = build_generic_bound_env(
            &definition.generic_params,
            &visible_trait_names,
            &format!("struct `{}`", definition.name),
        )?;
        for field in &definition.fields {
            validate_ast_type_ref_generic_constraints(
                &field.ty,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
                &generic_bounds,
                &format!("struct `{}` field `{}`", definition.name, field.name),
            )?;
        }
    }

    for function in &module.externs {
        for param in &function.params {
            validate_ast_type_ref_generic_constraints(
                &param.ty,
                visible_type_aliases,
                impl_lookup,
                &visible_trait_names,
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
        validate_ast_type_ref_generic_constraints(
            &definition.for_type,
            visible_type_aliases,
            impl_lookup,
            &visible_trait_names,
            &BTreeMap::new(),
            &format!("impl `{}` target type", definition.trait_name),
        )?;
        for method in &definition.methods {
            for param in &method.params {
                validate_ast_type_ref_generic_constraints(
                    &param.ty,
                    visible_type_aliases,
                    impl_lookup,
                    &visible_trait_names,
                    &BTreeMap::new(),
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
                    &BTreeMap::new(),
                    &format!(
                        "impl `{}` method `{}` return type",
                        definition.trait_name, method.name
                    ),
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
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Result<(), String> {
    let function_context = render_validation_function_context(&function.name);
    let generic_bounds = build_generic_bound_env(
        &function.generic_params,
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
            function_return_types,
            &generic_param_names,
            &generic_bounds,
            &mut local_type_env,
            &format!("{function_context} body"),
        )?;
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
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, String>,
    local_type_env: &mut BTreeMap<String, AstTypeRef>,
    context: &str,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let { name, ty, .. } | AstStmt::Const { name, ty, .. } => {
            let value = match stmt {
                AstStmt::Let { value, .. } | AstStmt::Const { value, .. } => value,
                _ => unreachable!(),
            };
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
                validate_ast_type_ref_generic_constraints(
                    ty,
                    visible_type_aliases,
                    impl_lookup,
                    visible_trait_names,
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
        AstStmt::DestructureLet { type_ref, .. } => {
            let value = match stmt {
                AstStmt::DestructureLet { value, .. } => value,
                _ => unreachable!(),
            };
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
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut else_env,
                    context,
                )?;
            }
        }
        AstStmt::Match { value, arms } => {
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
                    function_return_types,
                    generic_param_names,
                    generic_bounds,
                    &mut loop_env,
                    context,
                )?;
            }
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
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
    generic_bounds: &BTreeMap<String, String>,
    context: &str,
) -> Result<(), String> {
    let mut visiting = BTreeSet::new();
    validate_ast_type_ref_generic_constraints_inner(
        ty,
        visible_type_aliases,
        impl_lookup,
        visible_trait_names,
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
    generic_bounds: &BTreeMap<String, String>,
    context: &str,
    visiting: &mut BTreeSet<String>,
) -> Result<(), String> {
    for arg in &ty.generic_args {
        validate_ast_type_ref_generic_constraints_inner(
            arg,
            visible_type_aliases,
            impl_lookup,
            visible_trait_names,
            generic_bounds,
            context,
            visiting,
        )?;
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

    for (param, arg) in alias_definition.generic_params.iter().zip(&ty.generic_args) {
        if let Some(bound) = &param.bound {
            let bound_name = validate_generic_bound_type(
                bound,
                visible_trait_names,
                &alias_param_context(context, &alias_definition.name, &param.name),
            )?;
            validate_generic_bound_satisfaction(
                arg,
                &bound_name,
                visible_type_aliases,
                impl_lookup,
                generic_bounds,
                &alias_param_context(context, &alias_definition.name, &param.name),
            )?;
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
        generic_bounds,
        &expanded_context,
        visiting,
    )?;
    visiting.remove(&visit_key);
    Ok(())
}
