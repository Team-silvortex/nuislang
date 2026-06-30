use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::generics::{infer_generic_substitutions, specialize_function_template};
use super::super::{lower_type_ref_with_aliases, FunctionSignature};
use super::GenericImplMethodTemplate;
use crate::frontend::generic_rewrite::rewrite_generic_calls_in_function;
use crate::frontend::higher_order::rewrite_higher_order_calls_in_function;

pub(super) fn ensure_generic_specialization(
    template: &AstFunction,
    explicit_generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    context: &str,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<String, String> {
    let substitutions = infer_generic_substitutions(
        template,
        explicit_generic_args,
        args,
        expected,
        env,
        visible_type_aliases,
        impl_lookup,
        struct_table,
        function_return_types,
        Some(context),
    )?;
    let specialized_name = format!(
        "{}__{}",
        template.name,
        template
            .generic_params
            .iter()
            .map(|param| substitutions[&param.name]
                .render()
                .replace(|ch: char| !ch.is_ascii_alphanumeric(), "_"))
            .collect::<Vec<_>>()
            .join("__")
    );
    if specialization_cache.insert(specialized_name.clone()) {
        let specialized =
            specialize_function_template(template, &specialized_name, &substitutions)?;
        let mut higher_order_specialization_cache = BTreeSet::new();
        let mut higher_order_specialized_templates = Vec::new();
        let higher_order_rewritten = rewrite_higher_order_calls_in_function(
            &specialized,
            higher_order_templates,
            function_table,
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
            visible_type_aliases,
            &mut higher_order_specialization_cache,
            &mut higher_order_specialized_templates,
        )?;
        let mut extended_generic_templates = generic_templates.clone();
        for template in higher_order_specialized_templates {
            if !template.generic_params.is_empty() {
                extended_generic_templates.insert(template.name.clone(), template);
            }
        }
        let rewritten = rewrite_generic_calls_in_function(
            &higher_order_rewritten,
            &BTreeMap::new(),
            visible_type_aliases,
            &extended_generic_templates,
            generic_impl_method_templates,
            higher_order_templates,
            function_table,
            signatures,
            impl_lookup,
            struct_table,
            function_return_types,
            specialization_cache,
            specialized_functions,
            specialized_signatures,
        )?;
        specialized_signatures.push((
            specialized_name.clone(),
            FunctionSignature {
                abi: "nuis".to_owned(),
                interface: None,
                symbol_name: specialized_name.clone(),
                params: rewritten
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: rewritten
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                    .transpose()?,
                is_extern: false,
                is_async: rewritten.is_async,
            },
        ));
        specialized_functions.push(rewritten);
    }
    Ok(specialized_name)
}

pub(super) fn generic_arg_contains_definition_placeholder(
    ty: &AstTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| generic_arg_contains_definition_placeholder(arg, placeholder_names))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn ensure_generic_impl_method_specialization(
    trait_name: Option<&str>,
    method_name: &str,
    args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Option<String>, String> {
    let mut candidates = Vec::new();
    for template in generic_impl_method_templates.iter().filter(|template| {
        template.method_name == method_name
            && trait_name.is_none_or(|trait_name| {
                template.trait_name == trait_name
                    || template
                        .trait_name
                        .rsplit('.')
                        .next()
                        .is_some_and(|short| short == trait_name)
            })
            && template.function.params.len() == args.len()
    }) {
        if infer_generic_substitutions(
            &template.function,
            &[],
            args,
            expected,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
            None,
        )
        .is_ok()
        {
            candidates.push(template);
        }
    }
    if candidates.len() > 1 {
        return Err(format!(
            "generic impl method resolution for `{}` is ambiguous; matching impl method templates: {}",
            trait_name
                .map(|trait_name| format!("{trait_name}.{method_name}"))
                .unwrap_or_else(|| method_name.to_owned()),
            candidates
                .iter()
                .map(|template| template.function.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let Some(template) = candidates.into_iter().next() else {
        return Ok(None);
    };
    Ok(Some(ensure_generic_specialization(
        &template.function,
        &[],
        args,
        expected,
        method_name,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    )?))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn ensure_generic_impl_method_specialization_from_receiver_expected(
    method_name: &str,
    receiver_expected: &AstTypeRef,
    actual_args: &[AstExpr],
    expected: Option<&AstTypeRef>,
    env: &BTreeMap<String, AstTypeRef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    generic_templates: &BTreeMap<String, AstFunction>,
    generic_impl_method_templates: &[GenericImplMethodTemplate],
    higher_order_templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    signatures: &BTreeMap<String, FunctionSignature>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    struct_table: &BTreeMap<String, AstStructDef>,
    function_return_types: &BTreeMap<String, Option<AstTypeRef>>,
    specialization_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
    specialized_signatures: &mut Vec<(String, FunctionSignature)>,
) -> Result<Option<String>, String> {
    let inference_receiver = AstExpr::StructLiteral {
        type_name: receiver_expected.name.clone(),
        type_args: receiver_expected.generic_args.clone(),
        fields: Vec::new(),
    };
    let mut inference_args = vec![inference_receiver];
    inference_args.extend(actual_args.iter().skip(1).cloned());

    let mut candidates = Vec::new();
    for template in generic_impl_method_templates.iter().filter(|template| {
        template.method_name == method_name && template.function.params.len() == actual_args.len()
    }) {
        if infer_generic_substitutions(
            &template.function,
            &[],
            &inference_args,
            expected,
            env,
            visible_type_aliases,
            impl_lookup,
            struct_table,
            function_return_types,
            None,
        )
        .is_ok()
        {
            candidates.push(template);
        }
    }
    if candidates.len() > 1 {
        return Err(format!(
            "generic impl method resolution for `{method_name}` is ambiguous under explicit receiver generic anchoring; matching impl method templates: {}",
            candidates
                .iter()
                .map(|template| template.function.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let Some(template) = candidates.into_iter().next() else {
        return Ok(None);
    };
    Ok(Some(ensure_generic_specialization(
        &template.function,
        &[],
        &inference_args,
        expected,
        method_name,
        env,
        visible_type_aliases,
        generic_templates,
        generic_impl_method_templates,
        higher_order_templates,
        function_table,
        signatures,
        impl_lookup,
        struct_table,
        function_return_types,
        specialization_cache,
        specialized_functions,
        specialized_signatures,
    )?))
}
