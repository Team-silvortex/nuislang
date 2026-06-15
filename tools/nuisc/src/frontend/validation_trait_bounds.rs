use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstGenericParam, AstImplDef, AstModule, AstTypeAlias, AstTypeRef, AstWherePredicate,
};

use super::{is_public_visibility, lower_type_ref, resolve_ast_type_ref_aliases};

fn parent_enum_ast_type(ty: &AstTypeRef) -> Option<AstTypeRef> {
    let (parent, _variant) = ty.name.rsplit_once('.')?;
    Some(AstTypeRef {
        name: parent.to_owned(),
        generic_args: ty.generic_args.clone(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    })
}

fn impl_lookup_types(ty: &AstTypeRef) -> Vec<String> {
    let mut rendered = vec![lower_type_ref(ty).render()];
    if let Some(parent) = parent_enum_ast_type(ty) {
        rendered.push(lower_type_ref(&parent).render());
    }
    rendered
}

fn impl_target_matches_concrete(
    pattern: &AstTypeRef,
    pattern_generics: &BTreeSet<String>,
    concrete: &AstTypeRef,
) -> bool {
    if pattern.is_optional != concrete.is_optional || pattern.is_ref != concrete.is_ref {
        return false;
    }
    if pattern_generics.contains(&pattern.name) && pattern.generic_args.is_empty() {
        return true;
    }
    if pattern.name == concrete.name && pattern.generic_args.len() == concrete.generic_args.len() {
        return pattern
            .generic_args
            .iter()
            .zip(&concrete.generic_args)
            .all(|(lhs, rhs)| impl_target_matches_concrete(lhs, pattern_generics, rhs));
    }
    false
}

fn impl_definition_satisfies_type(
    definition: &AstImplDef,
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<bool, String> {
    let pattern = resolve_ast_type_ref_aliases(&definition.for_type, visible_type_aliases)?;
    let generics = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if impl_target_matches_concrete(&pattern, &generics, ty) {
        return Ok(true);
    }
    if let Some(parent) = parent_enum_ast_type(ty) {
        return Ok(impl_target_matches_concrete(&pattern, &generics, &parent));
    }
    Ok(false)
}

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
    where_predicates: &[AstWherePredicate],
    visible_trait_names: &BTreeSet<String>,
    context: &str,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut env = BTreeMap::new();
    for param in generic_params {
        if !param.bounds.is_empty() {
            let mut bounds = Vec::with_capacity(param.bounds.len());
            for bound in &param.bounds {
                bounds.push(validate_generic_bound_type(
                    bound,
                    visible_trait_names,
                    &generic_param_bound_context(context, &param.name),
                )?);
            }
            env.insert(param.name.clone(), bounds);
        }
    }
    let generic_names = generic_params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<BTreeSet<_>>();
    for predicate in where_predicates {
        if !generic_names.contains(predicate.param_name.as_str()) {
            return Err(format!(
                "{context} where clause references unknown generic parameter `{}`",
                predicate.param_name
            ));
        }
        let merged = env.entry(predicate.param_name.clone()).or_default();
        for bound in &predicate.bounds {
            let bound_name = validate_generic_bound_type(
                bound,
                visible_trait_names,
                &where_predicate_bound_context(context, &predicate.param_name),
            )?;
            if !merged.contains(&bound_name) {
                merged.push(bound_name);
            }
        }
    }
    Ok(env)
}

fn generic_param_bound_context(context: &str, param_name: &str) -> String {
    format!("{context} generic parameter `{param_name}`")
}

fn where_predicate_bound_context(context: &str, param_name: &str) -> String {
    format!("{context} where clause for generic parameter `{param_name}`")
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
    generic_bounds: &BTreeMap<String, Vec<String>>,
    context: &str,
) -> Result<(), String> {
    let resolved = resolve_ast_type_ref_aliases(ty, visible_type_aliases)?;
    if resolved.generic_args.is_empty() {
        if let Some(actual_bounds) = generic_bounds.get(&resolved.name) {
            if actual_bounds.iter().any(|bound| bound == required_bound) {
                return Ok(());
            }
        }
    }
    let rendered = lower_type_ref(&resolved).render();
    for candidate in impl_lookup_types(&resolved) {
        if impl_lookup.contains_key(&(required_bound.to_owned(), candidate.clone())) {
            return Ok(());
        }
    }
    if impl_lookup.values().any(|definition| {
        definition.trait_name == required_bound
            && impl_definition_satisfies_type(definition, &resolved, visible_type_aliases)
                .unwrap_or(false)
    }) {
        return Ok(());
    }
    let short_name = required_bound.rsplit('.').next().unwrap_or(required_bound);
    let matching_variants = impl_lookup
        .keys()
        .filter(|(_, for_type)| impl_lookup_types(&resolved).iter().any(|candidate| candidate == for_type))
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
