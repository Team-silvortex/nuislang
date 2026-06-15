use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstTypeAlias, AstTypeRef, AstVisibility, NirFunction,
    NirGenericParam, NirStructDef, NirTypeRef, NirWherePredicate,
};

use super::stmt_lowering::lower_stmt_block_with_async;
use super::{
    lower_ast_attributes, lower_param_with_aliases, lower_type_ref_with_aliases, lower_visibility,
    FunctionSignature, ModuleConstValue,
};

pub(super) fn impl_method_lookup_key(for_type: &NirTypeRef, method: &str) -> String {
    format!("{}.{}", for_type.render(), method)
}

pub(super) fn parent_enum_type(for_type: &NirTypeRef) -> Option<NirTypeRef> {
    let (parent, _variant) = for_type.name.rsplit_once('.')?;
    Some(NirTypeRef {
        name: parent.to_owned(),
        generic_args: for_type.generic_args.clone(),
        is_optional: for_type.is_optional,
        is_ref: for_type.is_ref,
    })
}

pub(super) fn impl_method_lookup_keys(for_type: &NirTypeRef, method: &str) -> Vec<String> {
    let mut keys = vec![impl_method_lookup_key(for_type, method)];
    if let Some(parent) = parent_enum_type(for_type) {
        keys.push(impl_method_lookup_key(&parent, method));
    }
    keys
}

pub(super) fn impl_method_symbol_name(
    trait_name: &str,
    for_type: &NirTypeRef,
    method: &str,
) -> String {
    let rendered = for_type
        .render()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '_',
        })
        .collect::<String>();
    format!("impl.{}.for.{}.{}", trait_name, rendered, method)
}

pub(super) fn impl_method_symbol_names(
    trait_name: &str,
    for_type: &NirTypeRef,
    method: &str,
) -> Vec<String> {
    let mut names = vec![impl_method_symbol_name(trait_name, for_type, method)];
    if let Some(parent) = parent_enum_type(for_type) {
        names.push(impl_method_symbol_name(trait_name, &parent, method));
    }
    names
}

pub(super) fn find_impl_method_signature<'a>(
    signatures: &'a BTreeMap<String, FunctionSignature>,
    trait_name: &str,
    for_type: &NirTypeRef,
    method: &str,
) -> Option<&'a FunctionSignature> {
    if let Some(signature) = impl_method_symbol_names(trait_name, for_type, method)
        .into_iter()
        .find_map(|symbol_name| signatures.get(&symbol_name))
    {
        return Some(signature);
    }

    let mut suffixes = vec![format!(".for.{}.{}", sanitize_impl_type_name(for_type), method)];
    if let Some(parent) = parent_enum_type(for_type) {
        suffixes.push(format!(
            ".for.{}.{}",
            sanitize_impl_type_name(&parent),
            method
        ));
    }
    let short_name = trait_name.rsplit('.').next().unwrap_or(trait_name);
    signatures.values().find(|signature| {
        signature.symbol_name.starts_with("impl.")
            && signature
                .symbol_name
                .strip_prefix("impl.")
                .and_then(|rest| rest.split_once(".for."))
                .is_some_and(|(candidate_trait, _)| {
                    (candidate_trait == trait_name
                        || candidate_trait
                            .rsplit('.')
                            .next()
                            .is_some_and(|candidate| candidate == short_name))
                        && suffixes
                            .iter()
                            .any(|suffix| signature.symbol_name.ends_with(suffix))
                })
    })
}

fn sanitize_impl_type_name(for_type: &NirTypeRef) -> String {
    for_type
        .render()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '_',
        })
        .collect::<String>()
}

pub(super) fn build_impl_method_function(
    definition: &AstImplDef,
    method: &nuis_semantics::model::AstImplMethod,
    symbol_name: &str,
) -> AstFunction {
    AstFunction {
        name: symbol_name.to_owned(),
        visibility: AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        is_async: false,
        generic_params: definition.generic_params.clone(),
        where_bounds: definition.where_bounds.clone(),
        params: method.params.clone(),
        return_type: method.return_type.clone().or_else(|| {
            Some(AstTypeRef {
                name: definition.for_type.name.clone(),
                generic_args: definition.for_type.generic_args.clone(),
                is_optional: definition.for_type.is_optional,
                is_ref: definition.for_type.is_ref,
            })
        }),
        body: method.body.clone(),
    }
}

pub(super) fn lower_function(
    function: &AstFunction,
    current_domain: &str,
    _current_unit: &str,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirFunction, String> {
    let mut bindings = BTreeMap::<String, NirTypeRef>::new();
    for param in &function.params {
        bindings.insert(
            param.name.clone(),
            lower_type_ref_with_aliases(&param.ty, type_aliases)?,
        );
    }

    Ok(NirFunction {
        name: function.name.clone(),
        annotations: lower_ast_attributes(&function.attributes),
        visibility: lower_visibility(function.visibility),
        test_name: function.test_name.clone(),
        test_ignored: function.test_ignored,
        test_should_fail: function.test_should_fail,
        test_reason: function.test_reason.clone(),
        test_timeout_ms: function.test_timeout_ms,
        test_clock_domain: function.test_clock_domain.clone(),
        test_clock_policy: function.test_clock_policy,
        is_async: function.is_async,
        generic_params: function
            .generic_params
            .iter()
            .map(|param| {
                Ok(NirGenericParam {
                    name: param.name.clone(),
                    bounds: param
                        .bounds
                        .iter()
                        .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        where_bounds: function
            .where_bounds
            .iter()
            .map(|predicate| {
                Ok(NirWherePredicate {
                    param_name: predicate.param_name.clone(),
                    bounds: predicate
                        .bounds
                        .iter()
                        .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        params: function
            .params
            .iter()
            .map(|param| lower_param_with_aliases(param, type_aliases))
            .collect::<Result<Vec<_>, _>>()?,
        return_type: function
            .return_type
            .as_ref()
            .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
            .transpose()?,
        body: lower_stmt_block_with_async(
            &function.body,
            current_domain,
            function.is_async,
            &mut bindings,
            module_consts,
            function.return_type.as_ref(),
            type_aliases,
            signatures,
            struct_table,
        )?,
    })
}
