use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstImplDef, AstModule, AstParam, AstStructDef, AstTypeAlias, AstTypeRef,
};

use super::super::generics::specialize_function_template;
use super::super::types::ast_type_from_nir;
use super::super::{
    build_impl_method_function, impl_method_symbol_name, lower_type_ref,
    lower_type_ref_with_aliases, resolve_ast_type_ref_aliases,
};
use super::callables::{
    function_type_matches_callable, is_callable_type_with_aliases, sanitize_symbol_fragment,
};
use super::expansion_rewrite::rewrite_higher_order_calls_in_function;
use super::expansion_rewrite_expr::rewrite_higher_order_calls_in_expr;
use super::templates::{rewrite_higher_order_template_expr, specialize_higher_order_template};

const LAMBDA_BIND_PREFIX: &str = "__lambda_bind.";

use super::expansion_callable_inference::{
    infer_callable_generic_substitutions, type_ref_contains_unresolved_placeholder,
    type_ref_looks_unresolved_placeholder,
};
use super::expansion_expected::{
    annotate_expr_head_with_expected_type, explicit_higher_order_generic_substitutions,
    higher_order_param_expected_type,
};
use super::expansion_inference::{
    infer_higher_order_substitutions, specialize_type_with_substitutions,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BoundCallable {
    pub(crate) symbol: String,
    pub(crate) capture_args: Vec<AstExpr>,
    pub(crate) capture_params: Vec<AstParam>,
}

pub(super) fn parse_bound_callable_expr(
    expr: &AstExpr,
    template_callable_bindings: Option<&BTreeMap<String, BoundCallable>>,
) -> Option<BoundCallable> {
    match expr {
        AstExpr::Var(name) => Some(
            template_callable_bindings
                .and_then(|bindings| bindings.get(name))
                .cloned()
                .unwrap_or_else(|| BoundCallable {
                    symbol: name.clone(),
                    capture_args: Vec::new(),
                    capture_params: Vec::new(),
                }),
        ),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty() && callee.starts_with(LAMBDA_BIND_PREFIX) => {
            Some(BoundCallable {
                symbol: callee[LAMBDA_BIND_PREFIX.len()..].to_owned(),
                capture_args: args.to_vec(),
                capture_params: Vec::new(),
            })
        }
        _ => None,
    }
}

pub(crate) fn expand_higher_order_functions(
    module: &AstModule,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<AstModule, String> {
    let visible_structs = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut templates = module
        .functions
        .iter()
        .filter(|function| {
            function.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            })
        })
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut method_template_lookup = BTreeMap::<(String, String), String>::new();
    for definition in &module.impls {
        let lowered_for_type =
            lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?;
        for method in &definition.methods {
            if !method.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            }) {
                continue;
            }
            let symbol_name =
                impl_method_symbol_name(&definition.trait_name, &lowered_for_type, &method.name);
            templates.insert(
                symbol_name.clone(),
                build_impl_method_function(definition, method, &symbol_name),
            );
            method_template_lookup.insert(
                (lowered_for_type.render(), method.name.clone()),
                symbol_name,
            );
        }
    }
    if templates.is_empty() {
        return Ok(module.clone());
    }

    let mut function_table = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    function_table.extend(templates.clone());

    let mut expanded = module.clone();
    expanded.functions.clear();
    let mut specialized_cache = BTreeSet::new();
    let mut specialized_functions = Vec::new();

    for function in &module.functions {
        if templates.contains_key(&function.name) {
            continue;
        }
        expanded
            .functions
            .push(rewrite_higher_order_calls_in_function(
                function,
                &templates,
                &function_table,
                &module.impls,
                &visible_structs,
                &method_template_lookup,
                visible_type_aliases,
                &mut specialized_cache,
                &mut specialized_functions,
            )?);
    }
    expanded.functions.extend(specialized_functions);
    Ok(expanded)
}

pub(super) fn specialize_higher_order_call(
    callee: &str,
    args: &[AstExpr],
    explicit_generic_args: &[AstTypeRef],
    template_callable_bindings: Option<&BTreeMap<String, BoundCallable>>,
    expected: Option<&AstTypeRef>,
    local_types: &BTreeMap<String, AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    let template = templates
        .get(callee)
        .ok_or_else(|| format!("unknown higher-order template `{callee}`"))?;
    if template.params.len() != args.len() {
        return Err(format!(
            "function `{}` expects {} args, found {}",
            callee,
            template.params.len(),
            args.len()
        ));
    }

    let explicit_substitutions = explicit_higher_order_generic_substitutions(
        template,
        explicit_generic_args,
        visible_type_aliases,
    )?;
    let mut callable_bindings = BTreeMap::<String, BoundCallable>::new();
    let mut ordinary_args = Vec::new();
    let mut callable_fragments = Vec::new();
    let generic_names = template
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();

    for (param, arg) in template.params.iter().zip(args) {
        let callable_param = is_callable_type_with_aliases(&param.ty, visible_type_aliases)?;
        let ordinary_expected = higher_order_param_expected_type(
            template,
            param,
            &explicit_substitutions,
            expected,
            visible_type_aliases,
        );
        if callable_param {
            let rewritten_arg = rewrite_higher_order_argument_expr(
                arg,
                template_callable_bindings,
                ordinary_expected.as_ref(),
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?;
            let Some(bound_callable) =
                parse_bound_callable_expr(&rewritten_arg, template_callable_bindings)
            else {
                return Err(format!(
                    "higher-order parameter `{}` currently expects a lambda or named function symbol",
                    param.name
                ));
            };
            let callable_name = bound_callable.symbol.clone();
            if !function_table.contains_key(&callable_name) {
                return Err(format!(
                    "higher-order parameter `{}` references unknown callable `{}`",
                    param.name, callable_name
                ));
            }
            let callable = function_table.get(&callable_name).ok_or_else(|| {
                format!(
                    "higher-order parameter `{}` references unknown callable `{}`",
                    param.name, callable_name
                )
            })?;
            if !function_type_matches_callable(
                callable,
                &param.ty,
                &generic_names,
                visible_type_aliases,
            )? {
                return Err(format!(
                    "callable `{}` does not match higher-order parameter `{}` of type `{}`",
                    callable_name, param.name, param.ty.name
                ));
            }
            let resolved_callable_ty =
                resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
            let callable_arity = super::callables::callable_type_arity(&resolved_callable_ty)
                .ok_or_else(|| {
                    format!("higher-order parameter `{}` is not callable", param.name)
                })?;
            let capture_param_defs = callable
                .params
                .iter()
                .skip(callable_arity)
                .collect::<Vec<_>>();
            if capture_param_defs.len() != bound_callable.capture_args.len() {
                return Err(format!(
                    "callable `{}` capture shape does not match higher-order binding `{}`",
                    callable_name, param.name
                ));
            }
            let helper_capture_args = capture_param_defs
                .iter()
                .enumerate()
                .map(|(index, capture_param)| {
                    AstExpr::Var(format!(
                        "__capture_{}_{}_{}",
                        sanitize_symbol_fragment(&param.name),
                        sanitize_symbol_fragment(&capture_param.name),
                        index
                    ))
                })
                .collect::<Vec<_>>();
            for capture_arg in &bound_callable.capture_args {
                ordinary_args.push(annotate_expr_head_with_expected_type(
                    capture_arg.clone(),
                    None,
                ));
            }
            callable_bindings.insert(
                param.name.clone(),
                BoundCallable {
                    symbol: callable_name.clone(),
                    capture_args: helper_capture_args,
                    capture_params: capture_param_defs
                        .iter()
                        .map(|param| (*param).clone())
                        .collect(),
                },
            );
            callable_fragments.push(sanitize_symbol_fragment(&callable_name));
        } else {
            let rewritten_arg = rewrite_higher_order_argument_expr(
                arg,
                template_callable_bindings,
                ordinary_expected.as_ref(),
                templates,
                function_table,
                visible_type_aliases,
                specialized_cache,
                specialized_functions,
            )?;
            ordinary_args.push(annotate_expr_head_with_expected_type(
                rewritten_arg,
                ordinary_expected.as_ref(),
            ));
        }
    }

    let inferred_substitutions = infer_higher_order_substitutions(
        template,
        &explicit_substitutions,
        args,
        expected,
        local_types,
        function_table,
        module_impls,
        visible_structs,
        visible_type_aliases,
    )?;
    let type_fragments = template
        .generic_params
        .iter()
        .filter_map(|param| {
            inferred_substitutions
                .get(&param.name)
                .map(|ty| sanitize_symbol_fragment(&ty.render()))
        })
        .collect::<Vec<_>>();
    if !inferred_substitutions.is_empty() {
        let inferred_ast_substitutions = inferred_substitutions
            .iter()
            .map(|(name, ty)| (name.clone(), ast_type_from_nir(ty)))
            .collect::<BTreeMap<_, _>>();
        let inferred_contains_placeholder = inferred_ast_substitutions
            .values()
            .any(type_ref_looks_unresolved_placeholder);
        for param in &template.params {
            let Some(binding) = callable_bindings.get_mut(&param.name) else {
                continue;
            };
            let Some(callable) = function_table.get(&binding.symbol) else {
                continue;
            };
            let resolved_param_ty = resolve_ast_type_ref_aliases(&param.ty, visible_type_aliases)?;
            let specialized_param_ty =
                specialize_type_with_substitutions(&resolved_param_ty, &inferred_ast_substitutions);
            let callable_specific_substitutions = infer_callable_generic_substitutions(
                &specialized_param_ty,
                callable,
                visible_type_aliases,
            )?;
            let mut callable_specialization = inferred_ast_substitutions.clone();
            callable_specialization.extend(callable_specific_substitutions);
            let callable_contains_placeholder = callable_specialization
                .values()
                .any(type_ref_looks_unresolved_placeholder);
            let callable_needs_specialization = !callable.generic_params.is_empty()
                || callable
                    .params
                    .iter()
                    .any(|param| type_ref_contains_unresolved_placeholder(&param.ty))
                || callable
                    .return_type
                    .as_ref()
                    .is_some_and(type_ref_contains_unresolved_placeholder);
            if callable_needs_specialization
                && !inferred_contains_placeholder
                && !callable_contains_placeholder
            {
                let specialized_callable_name =
                    format!("{}__{}", binding.symbol, type_fragments.join("__"));
                if specialized_cache.insert(specialized_callable_name.clone()) {
                    let lowered_callable_specialization = callable_specialization
                        .clone()
                        .into_iter()
                        .map(|(name, ty)| (name, lower_type_ref(&ty)))
                        .collect();
                    let specialized_callable = specialize_function_template(
                        callable,
                        &specialized_callable_name,
                        &lowered_callable_specialization,
                    )?;
                    specialized_functions.push(specialized_callable);
                }
                binding.symbol = specialized_callable_name;
                binding.capture_params = binding
                    .capture_params
                    .iter()
                    .map(|param| AstParam {
                        name: param.name.clone(),
                        ty: specialize_type_with_substitutions(&param.ty, &callable_specialization),
                    })
                    .collect();
            }
        }
    }

    let callable_fragment = callable_fragments.join("__");
    let specialized_name = if type_fragments.is_empty() {
        format!(
            "__hof_{}_{}",
            sanitize_symbol_fragment(callee),
            callable_fragment
        )
    } else {
        format!(
            "__hof_{}_{}__{}",
            sanitize_symbol_fragment(callee),
            callable_fragment,
            type_fragments.join("__")
        )
    };
    if specialized_cache.insert(specialized_name.clone()) {
        let specialized = specialize_higher_order_template(
            template,
            &specialized_name,
            &callable_bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        )?;
        let inferred_ast_substitutions = inferred_substitutions
            .iter()
            .map(|(name, ty)| (name.clone(), ast_type_from_nir(ty)))
            .collect::<BTreeMap<_, _>>();
        let inferred_contains_placeholder = inferred_ast_substitutions
            .values()
            .any(type_ref_looks_unresolved_placeholder);
        let specialized = if inferred_substitutions.is_empty() || inferred_contains_placeholder {
            specialized
        } else {
            specialize_function_template(&specialized, &specialized_name, &inferred_substitutions)?
        };
        specialized_functions.push(specialized.clone());
    }

    Ok(AstExpr::Call {
        callee: specialized_name,
        generic_args: Vec::new(),
        args: ordinary_args,
    })
}

fn rewrite_higher_order_argument_expr(
    expr: &AstExpr,
    template_callable_bindings: Option<&BTreeMap<String, BoundCallable>>,
    expected: Option<&AstTypeRef>,
    templates: &BTreeMap<String, AstFunction>,
    function_table: &BTreeMap<String, AstFunction>,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    specialized_cache: &mut BTreeSet<String>,
    specialized_functions: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    if let Some(bindings) = template_callable_bindings {
        return rewrite_higher_order_template_expr(
            expr,
            bindings,
            templates,
            function_table,
            visible_type_aliases,
            specialized_cache,
            specialized_functions,
        );
    }
    rewrite_higher_order_calls_in_expr(
        expr,
        expected,
        None,
        &BTreeMap::new(),
        templates,
        function_table,
        &[],
        &BTreeMap::new(),
        &BTreeMap::new(),
        visible_type_aliases,
        specialized_cache,
        specialized_functions,
    )
}
