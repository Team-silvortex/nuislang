use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstEnumDef, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef};

use super::super::validation_trait_bounds::{
    alias_param_context, alias_target_context, build_generic_bound_env,
    validate_generic_bound_satisfaction,
};
use super::super::{lower_type_ref, substitute_ast_type_alias_target};

pub(super) fn validate_ast_type_ref_generic_constraints(
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
