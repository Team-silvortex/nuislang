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
        let variants = collect_trait_name_variants(&bound.name, visible_trait_names);
        if let Some(preferred) = preferred_trait_name_variant(&bound.name, &variants) {
            return Err(format!(
                "{context} references unknown generic bound trait `{}`; did you mean `{}`?",
                bound.name, preferred
            ));
        }
        if variants.len() == 1 {
            return Err(format!(
                "{context} references unknown generic bound trait `{}`; did you mean `{}`?",
                bound.name, variants[0]
            ));
        }
        if !variants.is_empty() {
            return Err(format!(
                "{context} references unknown generic bound trait `{}`; matching visible traits: {}",
                bound.name,
                variants.join(", ")
            ));
        }
        return Err(format!(
            "{context} references unknown generic bound trait `{}`",
            bound.name
        ));
    }
    Ok(bound.name.clone())
}

fn collect_trait_name_variants(
    trait_name: &str,
    visible_trait_names: &BTreeSet<String>,
) -> Vec<String> {
    let short_name = trait_name.rsplit('.').next().unwrap_or(trait_name);
    let mut variants = visible_trait_names
        .iter()
        .filter(|candidate| candidate.as_str() != trait_name)
        .filter(|candidate| {
            candidate
                .rsplit('.')
                .next()
                .is_some_and(|name| name == short_name)
        })
        .cloned()
        .collect::<Vec<_>>();
    variants.sort_by_key(|candidate| (!candidate.contains('.'), candidate.clone()));
    variants
}

fn preferred_trait_name_variant(trait_name: &str, variants: &[String]) -> Option<String> {
    if trait_name.contains('.') {
        let qualified = variants
            .iter()
            .filter(|candidate| candidate.contains('.'))
            .cloned()
            .collect::<Vec<_>>();
        if qualified.len() == 1 {
            return qualified.into_iter().next();
        }
    }
    None
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
    let short_name = required_bound.rsplit('.').next().unwrap_or(required_bound);
    let matching_variants = impl_lookup
        .keys()
        .filter(|(_, for_type)| for_type == &rendered)
        .map(|(trait_name, _)| trait_name)
        .filter(|trait_name| {
            trait_name.as_str() != required_bound
                && trait_name
                    .rsplit('.')
                    .next()
                    .is_some_and(|name| name == short_name)
        })
        .cloned()
        .collect::<Vec<_>>();
    if matching_variants.len() == 1 {
        return Ok(());
    }
    if matching_variants.len() > 1 {
        return Err(format!(
            "type `{}` ambiguously satisfies bound `{}` for {}; matching visible trait variants: {}",
            rendered,
            required_bound,
            context,
            matching_variants.join(", ")
        ));
    }
    Err(format!(
        "type `{}` does not satisfy bound `{}` for {}",
        rendered, required_bound, context
    ))
}

pub(super) fn validate_generic_parameter_use_site_bound(
    generic_name: &str,
    ty: &AstTypeRef,
    required_bound: &str,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
) -> Result<(), String> {
    validate_generic_bound_satisfaction(
        ty,
        required_bound,
        visible_type_aliases,
        impl_lookup,
        &BTreeMap::new(),
        &format!("generic parameter `{generic_name}`"),
    )
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
