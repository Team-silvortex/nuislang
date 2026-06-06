use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstGenericParam, AstImplDef, AstModule, AstTypeAlias, AstTypeRef};

use super::{is_public_visibility, lower_type_ref, resolve_ast_type_ref_aliases};

pub(super) fn collect_visible_trait_names(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
) -> BTreeSet<String> {
    let mut trait_names = module
        .traits
        .iter()
        .map(|definition| definition.name.clone())
        .collect::<BTreeSet<_>>();
    for helper in local_cpu_helpers {
        for definition in helper
            .traits
            .iter()
            .filter(|definition| is_public_visibility(definition.visibility))
        {
            trait_names.insert(definition.name.clone());
            trait_names.insert(format!("{}.{}", helper.unit, definition.name));
        }
    }
    trait_names
}

pub(super) fn build_generic_bound_env(
    generic_params: &[AstGenericParam],
    visible_trait_names: &BTreeSet<String>,
    context: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut env = BTreeMap::new();
    for param in generic_params {
        if let Some(bound) = &param.bound {
            let bound_name = validate_generic_bound_type(bound, visible_trait_names, context)?;
            env.insert(param.name.clone(), bound_name);
        }
    }
    Ok(env)
}

pub(super) fn validate_generic_bound_type(
    bound: &AstTypeRef,
    visible_trait_names: &BTreeSet<String>,
    context: &str,
) -> Result<String, String> {
    if bound.is_ref || bound.is_optional || !bound.generic_args.is_empty() {
        return Err(format!(
            "{context} uses unsupported generic bound `{}`; generic bounds currently require a bare trait name",
            lower_type_ref(bound).render()
        ));
    }
    if !visible_trait_names.contains(&bound.name) {
        return Err(format!(
            "{context} references unknown generic bound trait `{}`",
            bound.name
        ));
    }
    Ok(bound.name.clone())
}

pub(super) fn validate_generic_bound_satisfaction(
    ty: &AstTypeRef,
    required_bound: &str,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    generic_bounds: &BTreeMap<String, String>,
    context: &str,
) -> Result<(), String> {
    let resolved = resolve_ast_type_ref_aliases(ty, visible_type_aliases)?;
    if resolved.generic_args.is_empty() {
        if let Some(actual_bound) = generic_bounds.get(&resolved.name) {
            if actual_bound == required_bound {
                return Ok(());
            }
        }
    }
    let rendered = lower_type_ref(&resolved).render();
    if impl_lookup.contains_key(&(required_bound.to_owned(), rendered.clone())) {
        return Ok(());
    }
    Err(format!(
        "type `{}` does not satisfy bound `{}` for {}",
        rendered, required_bound, context
    ))
}

pub(super) fn alias_param_context(
    parent_context: &str,
    alias_name: &str,
    param_name: &str,
) -> String {
    format!(
        "{} via type alias `{}` generic parameter `{}`",
        parent_context, alias_name, param_name
    )
}

pub(super) fn alias_target_context(parent_context: &str, alias_name: &str) -> String {
    format!("{parent_context} via type alias `{alias_name}` target")
}
