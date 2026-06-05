use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstGenericParam, AstModule, AstTypeRef};

use super::{is_public_visibility, lower_type_ref};

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
