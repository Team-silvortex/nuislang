use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstImplDef, AstImplMethod, AstModule, AstTraitDef, AstTraitMethodSig, AstTypeAlias, AstTypeRef,
};

use super::super::lower_type_ref_with_aliases;

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

pub(super) fn render_impl_target_type(
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

pub(super) fn validate_trait_impl_coherence(
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
