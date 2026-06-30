use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstImplDef, AstModule, AstStructDef, AstTraitDef, AstTypeAlias, AstTypeRef,
};

use super::validation_binding_env::simple_match_value_type;
use super::{infer_ast_expr_type, resolve_ast_type_ref_aliases};

pub(super) fn inferred_match_value_type(
    value: &AstExpr,
    local_type_env: &BTreeMap<String, AstTypeRef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
) -> Option<AstTypeRef> {
    simple_match_value_type(value, local_type_env).or_else(|| {
        infer_ast_expr_type(
            value,
            local_type_env,
            impl_lookup,
            visible_structs,
            function_return_types,
        )
    })
}

pub(super) fn normalize_method_bound_context(context: &str) -> String {
    if context.contains("higher-order specialization body") {
        return context.to_owned();
    }
    context.replace(" match-arm", "")
}

pub(super) fn parent_enum_ast_type(receiver_ty: &AstTypeRef) -> Option<AstTypeRef> {
    let (parent, _variant) = receiver_ty.name.rsplit_once('.')?;
    Some(AstTypeRef {
        name: parent.to_owned(),
        generic_args: receiver_ty.generic_args.clone(),
        is_optional: receiver_ty.is_optional,
        is_ref: receiver_ty.is_ref,
    })
}

pub(super) fn impl_target_matches_receiver(
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
            .all(|(lhs, rhs)| impl_target_matches_receiver(lhs, pattern_generics, rhs));
    }
    false
}

pub(super) fn impl_matches_receiver_type(
    definition: &AstImplDef,
    receiver_ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<bool, String> {
    let pattern = resolve_ast_type_ref_aliases(&definition.for_type, visible_type_aliases)?;
    let generics = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    if impl_target_matches_receiver(&pattern, &generics, receiver_ty) {
        return Ok(true);
    }
    if let Some(parent) = parent_enum_ast_type(receiver_ty) {
        return Ok(impl_target_matches_receiver(&pattern, &generics, &parent));
    }
    Ok(false)
}

pub(super) fn collect_visible_trait_methods(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut methods = BTreeMap::new();
    for definition in &module.traits {
        insert_trait_methods(&mut methods, definition.name.clone(), definition);
    }
    for helper in local_cpu_helpers {
        for definition in helper
            .traits
            .iter()
            .filter(|definition| super::is_public_visibility(definition.visibility))
        {
            insert_trait_methods(&mut methods, definition.name.clone(), definition);
            insert_trait_methods(
                &mut methods,
                format!("{}.{}", helper.unit, definition.name),
                definition,
            );
        }
    }
    methods
}

#[path = "validation_method_bounds_bounds.rs"]
mod validation_method_bounds_bounds;
#[path = "validation_method_bounds_expr.rs"]
mod validation_method_bounds_expr;
#[path = "validation_method_bounds_stmt.rs"]
mod validation_method_bounds_stmt;

pub(in crate::frontend) use validation_method_bounds_expr::validate_expr_generic_method_bounds;

fn insert_trait_methods(
    methods: &mut BTreeMap<String, BTreeSet<String>>,
    name: String,
    definition: &AstTraitDef,
) {
    methods.insert(
        name,
        definition
            .methods
            .iter()
            .map(|method| method.name.clone())
            .collect(),
    );
}
