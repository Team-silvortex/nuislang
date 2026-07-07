use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef, AstUnaryOp,
};

use super::{impl_matches_receiver_type, parent_enum_ast_type};
use crate::frontend::{
    infer_ast_expr_type, lower_type_ref, name_suggestions::suggest_similar_name,
    resolve_ast_type_ref_aliases, substitute_ast_type_alias_target,
};

pub(super) fn unary_operator_trait_requirement(
    op: AstUnaryOp,
) -> Option<(&'static str, &'static str, &'static str)> {
    match op {
        AstUnaryOp::Not => Some(("!", "not", "Notable")),
        AstUnaryOp::Neg => Some(("-", "neg", "Negatable")),
        AstUnaryOp::Deref => None,
    }
}

pub(super) fn binary_operator_trait_requirement(
    op: AstBinaryOp,
) -> Option<(&'static str, &'static str, &'static str)> {
    match op {
        AstBinaryOp::Add => Some(("+", "add", "Addable")),
        AstBinaryOp::Sub => Some(("-", "sub", "Subtractable")),
        AstBinaryOp::Mul => Some(("*", "mul", "Multipliable")),
        AstBinaryOp::Div => Some(("/", "div", "Dividable")),
        AstBinaryOp::Rem => Some(("%", "rem", "Remainderable")),
        AstBinaryOp::Eq => Some(("==", "eq", "Equatable")),
        AstBinaryOp::Ne => Some(("!=", "eq", "Equatable")),
        AstBinaryOp::Lt => Some(("<", "lt", "Orderable")),
        AstBinaryOp::Le => Some(("<=", "le", "Orderable")),
        AstBinaryOp::Gt => Some((">", "gt", "Orderable")),
        AstBinaryOp::Ge => Some((">=", "ge", "Orderable")),
        _ => None,
    }
}

fn resolve_generic_receiver_bound_context(
    receiver_ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    context: &str,
) -> Result<Option<(String, String, Vec<String>)>, String> {
    let Some((generic_name, receiver_context)) = resolve_generic_receiver_context(
        receiver_ty,
        visible_type_aliases,
        generic_param_names,
        &mut BTreeSet::new(),
    )?
    else {
        return Ok(None);
    };
    Ok(Some((
        generic_name.clone(),
        format!("{context}{receiver_context}"),
        generic_bounds
            .get(&generic_name)
            .cloned()
            .unwrap_or_default(),
    )))
}

fn bound_matches_required_trait(
    bound: &str,
    required_trait: &str,
    required_method: &str,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    if bound == required_trait
        && trait_methods
            .get(bound)
            .is_some_and(|methods| methods.contains(required_method))
    {
        return true;
    }
    let variants = collect_trait_name_variants(required_trait, trait_methods);
    variants.iter().any(|candidate| candidate == bound)
        && trait_methods
            .get(bound)
            .is_some_and(|methods| methods.contains(required_method))
}

fn render_declared_bounds(bounds: &[String]) -> String {
    bounds.join(" + ")
}

pub(super) fn validate_generic_receiver_method_bound(
    receiver_ty: &AstTypeRef,
    method: &str,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
    generic_param_names: &BTreeSet<String>,
    generic_bounds: &BTreeMap<String, Vec<String>>,
    context: &str,
) -> Result<(), String> {
    let Some((generic_name, context, bounds)) = resolve_generic_receiver_bound_context(
        receiver_ty,
        visible_type_aliases,
        generic_param_names,
        generic_bounds,
        context,
    )?
    else {
        return Ok(());
    };

    let candidates = trait_methods
        .iter()
        .filter(|(_, methods)| methods.contains(method))
        .map(|(trait_name, _)| trait_name.clone())
        .collect::<Vec<_>>();

    if !bounds.is_empty() {
        if bounds.iter().any(|bound| {
            trait_methods
                .get(bound)
                .is_some_and(|methods| methods.contains(method))
        }) {
            return Ok(());
        }
        let declared = render_declared_bounds(&bounds);
        if let Some((bound, suggested_method)) = bounds.iter().find_map(|bound| {
            trait_methods
                .get(bound)
                .and_then(|methods| suggest_trait_method_name(method, methods))
                .map(|suggested| (bound, suggested))
        }) {
            if bounds.len() == 1 {
                return Err(format!(
                    "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{bound}` does not define that method; did you mean `{}`?",
                    suggested_method
                ));
            }
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but declared bounds `{declared}` do not define that method; trait `{bound}` suggests `{}`",
                suggested_method
            ));
        }
        if candidates.is_empty() {
            if bounds.len() == 1 {
                return Err(format!(
                    "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{declared}` does not define that method"
                ));
            }
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but declared bounds `{declared}` do not define that method"
            ));
        }
        if candidates.len() == 1 {
            if bounds.len() == 1 {
                return Err(format!(
                    "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{declared}` does not define that method; consider bound `{}`",
                    candidates[0]
                ));
            }
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but declared bounds `{declared}` do not define that method; consider bound `{}`",
                candidates[0]
            ));
        }
        if bounds.len() == 1 {
            return Err(format!(
                "{context} calls method `{method}` on generic parameter `{generic_name}` but bound `{declared}` does not define that method; candidate bounds: {}",
                candidates.join(", ")
            ));
        }
        return Err(format!(
            "{context} calls method `{method}` on generic parameter `{generic_name}` but declared bounds `{declared}` do not define that method; candidate bounds: {}",
            candidates.join(", ")
        ));
    }

    if candidates.len() == 1 {
        return Err(format!(
            "{context} calls method `{method}` on generic parameter `{generic_name}` without required bound `{}`",
            candidates[0]
        ));
    }
    if candidates.len() > 1 {
        return Err(format!(
            "{context} calls method `{method}` on generic parameter `{generic_name}` without a trait bound; candidate bounds: {}",
            candidates.join(", ")
        ));
    }
    Ok(())
}

pub(super) struct GenericReceiverOperatorBoundInput<'a> {
    pub(super) receiver_ty: &'a AstTypeRef,
    pub(super) operator: &'a str,
    pub(super) method: &'a str,
    pub(super) required_bound: &'a str,
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) trait_methods: &'a BTreeMap<String, BTreeSet<String>>,
    pub(super) generic_param_names: &'a BTreeSet<String>,
    pub(super) generic_bounds: &'a BTreeMap<String, Vec<String>>,
    pub(super) context: &'a str,
}

pub(super) fn validate_generic_receiver_operator_bound(
    input: GenericReceiverOperatorBoundInput<'_>,
) -> Result<(), String> {
    let GenericReceiverOperatorBoundInput {
        receiver_ty,
        operator,
        method,
        required_bound,
        visible_type_aliases,
        trait_methods,
        generic_param_names,
        generic_bounds,
        context,
    } = input;
    let Some((generic_name, context, bounds)) = resolve_generic_receiver_bound_context(
        receiver_ty,
        visible_type_aliases,
        generic_param_names,
        generic_bounds,
        context,
    )?
    else {
        return Ok(());
    };

    if !bounds.is_empty() {
        if bounds
            .iter()
            .any(|bound| bound_matches_required_trait(bound, required_bound, method, trait_methods))
        {
            return Ok(());
        }
        let declared = render_declared_bounds(&bounds);
        if bounds.len() == 1 {
            return Err(format!(
                "{context} calls operator `{operator}` on generic parameter `{generic_name}` but bound `{declared}` does not satisfy required trait `{required_bound}`"
            ));
        }
        return Err(format!(
            "{context} calls operator `{operator}` on generic parameter `{generic_name}` but declared bounds `{declared}` do not satisfy required trait `{required_bound}`"
        ));
    }

    Err(format!(
        "{context} calls operator `{operator}` on generic parameter `{generic_name}` without required bound `{required_bound}`"
    ))
}

pub(super) struct ExplicitTraitCallBoundInput<'a> {
    pub(super) trait_name: &'a str,
    pub(super) method: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) visible_type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) impl_lookup: &'a BTreeMap<(String, String), AstImplDef>,
    pub(super) visible_structs: &'a BTreeMap<String, AstStructDef>,
    pub(super) function_return_types: &'a BTreeMap<String, Option<AstTypeRef>>,
    pub(super) trait_methods: &'a BTreeMap<String, BTreeSet<String>>,
    pub(super) generic_param_names: &'a BTreeSet<String>,
    pub(super) generic_bounds: &'a BTreeMap<String, Vec<String>>,
    pub(super) local_type_env: &'a BTreeMap<String, AstTypeRef>,
    pub(super) context: &'a str,
}

pub(super) fn validate_explicit_trait_call_bound(
    input: ExplicitTraitCallBoundInput<'_>,
) -> Result<(), String> {
    let ExplicitTraitCallBoundInput {
        trait_name,
        method,
        args,
        visible_type_aliases,
        impl_lookup,
        visible_structs,
        function_return_types,
        trait_methods,
        generic_param_names,
        generic_bounds,
        local_type_env,
        context,
    } = input;
    let Some(receiver) = args.first() else {
        return Ok(());
    };
    if !trait_methods
        .get(trait_name)
        .is_some_and(|methods| methods.contains(method))
    {
        if let Some(suggested_method) = trait_methods
            .get(trait_name)
            .and_then(|methods| suggest_trait_method_name(method, methods))
        {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}`, but trait `{trait_name}` does not define method `{method}`; did you mean `{}.{}`?",
                trait_name, suggested_method
            ));
        }
        let variants = collect_trait_name_variants(trait_name, trait_methods);
        if variants.len() == 1
            && trait_methods
                .get(&variants[0])
                .is_some_and(|methods| methods.contains(method))
        {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}`, but trait `{trait_name}` does not define method `{method}`; did you mean `{}.{method}`?",
                variants[0]
            ));
        }
        return Err(format!(
            "{context} calls trait method `{trait_name}.{method}`, but trait `{trait_name}` does not define method `{method}`"
        ));
    }
    let Some(receiver_ty) = infer_ast_expr_type(
        receiver,
        local_type_env,
        impl_lookup,
        visible_structs,
        function_return_types,
    ) else {
        return Ok(());
    };
    let receiver_rendered = lower_type_ref(&receiver_ty).render();
    let Some((generic_name, context, bounds)) = resolve_generic_receiver_bound_context(
        &receiver_ty,
        visible_type_aliases,
        generic_param_names,
        generic_bounds,
        context,
    )?
    else {
        if impl_lookup.contains_key(&(trait_name.to_owned(), receiver_rendered.clone())) {
            return Ok(());
        }
        if let Some(parent_receiver_ty) = parent_enum_ast_type(&receiver_ty) {
            let parent_rendered = lower_type_ref(&parent_receiver_ty).render();
            if impl_lookup.contains_key(&(trait_name.to_owned(), parent_rendered)) {
                return Ok(());
            }
        }
        if impl_lookup.values().any(|definition| {
            definition.trait_name == trait_name
                && impl_matches_receiver_type(definition, &receiver_ty, visible_type_aliases)
                    .unwrap_or(false)
        }) {
            return Ok(());
        }
        let available_impls =
            collect_receiver_trait_impl_candidates(&receiver_rendered, impl_lookup);
        if available_impls.is_empty() {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}` for `{receiver_rendered}`, but trait `{trait_name}` has no impl for `{receiver_rendered}`"
            ));
        }
        return Err(format!(
            "{context} calls trait method `{trait_name}.{method}` for `{receiver_rendered}`, but trait `{trait_name}` has no impl for `{receiver_rendered}`; available trait impls for `{receiver_rendered}`: {}",
            available_impls.join(", ")
        ));
    };

    if !bounds.is_empty() {
        if bounds.iter().any(|bound| bound == trait_name) {
            return Ok(());
        }
        let variants = collect_trait_name_variants(trait_name, trait_methods);
        if let Some(bound) = bounds
            .iter()
            .find(|bound| variants.iter().any(|candidate| candidate == *bound))
        {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` but bound `{bound}` uses a different visible name for the same trait; use `{trait_name}` consistently"
            ));
        }
        let declared = render_declared_bounds(&bounds);
        if bounds.len() == 1 {
            return Err(format!(
                "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` but bound `{declared}` does not satisfy required trait `{trait_name}`"
            ));
        }
        return Err(format!(
            "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` but declared bounds `{declared}` do not satisfy required trait `{trait_name}`"
        ));
    }

    Err(format!(
        "{context} calls trait method `{trait_name}.{method}` on generic parameter `{generic_name}` without required bound `{trait_name}`"
    ))
}

fn collect_trait_name_variants(
    trait_name: &str,
    trait_methods: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<String> {
    let short_name = trait_name.rsplit('.').next().unwrap_or(trait_name);
    trait_methods
        .keys()
        .filter(|candidate| candidate.as_str() != trait_name)
        .filter(|candidate| {
            candidate
                .rsplit('.')
                .next()
                .is_some_and(|name| name == short_name)
        })
        .cloned()
        .collect()
}

fn collect_receiver_trait_impl_candidates(
    receiver_rendered: &str,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
) -> Vec<String> {
    impl_lookup
        .keys()
        .filter(|(_, for_type)| for_type == receiver_rendered)
        .map(|(trait_name, _)| trait_name.clone())
        .collect()
}

fn suggest_trait_method_name(method: &str, methods: &BTreeSet<String>) -> Option<String> {
    suggest_similar_name(method, methods)
}

fn resolve_generic_receiver_context(
    ty: &AstTypeRef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_param_names: &BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> Result<Option<(String, String)>, String> {
    if let Some(alias_definition) = visible_type_aliases.get(&ty.name) {
        if alias_definition.generic_params.len() == ty.generic_args.len() {
            let visit_key = format!("{}::{}", ty.name, lower_type_signature(ty));
            if !visiting.insert(visit_key.clone()) {
                return Ok(None);
            }

            let substitutions = alias_definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .zip(ty.generic_args.iter().cloned())
                .collect::<BTreeMap<_, _>>();
            let expanded =
                substitute_ast_type_alias_target(&alias_definition.target, &substitutions)?;
            let nested = resolve_generic_receiver_context(
                &expanded,
                visible_type_aliases,
                generic_param_names,
                visiting,
            )?;
            visiting.remove(&visit_key);
            if let Some((name, context)) = nested {
                return Ok(Some((
                    name,
                    format!(
                        "{context} via type alias `{}` target",
                        alias_definition.name
                    ),
                )));
            }
        }
    }

    let resolved = resolve_ast_type_ref_aliases(ty, visible_type_aliases)?;
    if resolved.generic_args.is_empty()
        && !resolved.is_optional
        && !resolved.is_ref
        && generic_param_names.contains(&resolved.name)
    {
        return Ok(Some((resolved.name.clone(), String::new())));
    }
    Ok(None)
}

fn lower_type_signature(ty: &AstTypeRef) -> String {
    lower_type_ref(ty).render()
}
