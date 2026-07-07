mod aliases;
mod annotations;
mod binary_lowering;
mod call_helpers;
mod call_lowering;
mod call_routing;
mod call_routing_byte_splits;
mod call_routing_bytes;
mod call_routing_slice_helpers;
mod call_routing_slices;
mod const_assembly;
mod data_builtins;
mod data_profile_builtins;
mod direct_calls;
mod doc_index;
mod expr_lowering;
mod function_context;
mod function_lowering;
mod generic_rewrite;
mod generics;
mod higher_order;
mod kernel_builtins;
mod lambda_expansion;
mod lambda_validation;
mod lexer;
mod match_hoist;
mod match_lowering;
mod metadata;
mod module_assembly;
mod name_suggestions;
mod network_builtins;
mod nova_builtins;
mod parser;
mod receiver_expected;
mod return_inference;
mod shader_builtins;
mod signature_building;
mod specialization_pipeline;
mod stmt_lowering;
mod stmt_lowering_control;
mod stmt_lowering_destructure;
mod stmt_lowering_sequence;
mod stmt_lowering_try;
mod stmt_lowering_try_helpers;
mod stmt_lowering_try_nested;
mod task_builtins;
#[cfg(test)]
mod tests_benchmark_functions;
#[cfg(test)]
mod tests_comments;
#[cfg(test)]
mod tests_consts_aliases;
#[cfg(test)]
mod tests_control_flow;
#[cfg(test)]
mod tests_destructure_let;
#[cfg(test)]
mod tests_doc_index;
#[cfg(test)]
mod tests_enums;
#[cfg(test)]
mod tests_frontend_core;
#[cfg(test)]
mod tests_frontend_semantics;
mod text_handle_rewrite;

// Generic specialization and higher-order behavior.
#[cfg(test)]
mod tests_generic_constraints;
#[cfg(test)]
mod tests_generic_structs;
#[cfg(test)]
mod tests_generics;
#[cfg(test)]
mod tests_higher_order;
#[cfg(test)]
mod tests_lambda_higher_order;

// Generic method-bound validation across surface shapes.
#[cfg(test)]
mod tests_generic_destructure_let;
#[cfg(test)]
mod tests_generic_method_bounds;
#[cfg(test)]
mod tests_generic_method_bounds_control_flow;
#[cfg(test)]
mod tests_generic_method_bounds_if_bindings;
#[cfg(test)]
mod tests_generic_method_bounds_lambda_bindings;
#[cfg(test)]
mod tests_generic_method_bounds_nested_match;

// Pattern/control-flow surface coverage.
#[cfg(test)]
mod tests_match_patterns;
#[cfg(test)]
mod tests_match_payload_bindings;
#[cfg(test)]
mod tests_match_struct_bindings;
#[cfg(test)]
mod tests_match_struct_patterns;
#[cfg(test)]
mod tests_packet_test_meta;
#[cfg(test)]
mod tests_parse_annotations;
#[cfg(test)]
mod tests_return_inference;
#[cfg(test)]
mod tests_test_functions;
#[cfg(test)]
mod tests_try;
#[cfg(test)]
mod tests_types_async_window;
mod types;
mod unary_lowering;
mod validation;
mod validation_assignments;
mod validation_binding_env;
mod validation_generic_constraints;
mod validation_helpers;
mod validation_method_bounds;
mod validation_trait_bounds;

use std::collections::BTreeMap;

use self::annotations::{
    extern_function_symbol_name, function_host_symbol_name, validate_const_item,
    validate_export_annotations, validate_extern_host_symbols, validate_function_annotations,
    validate_host_symbol_bridge_annotations, validate_struct_annotations,
};
use self::call_helpers::{
    ensure_mutex_guard_like, ensure_mutex_like, ensure_ref_like, ensure_spawn_input_safe,
    ensure_task_like, ensure_thread_like, lower_result_observer_call_with_consts,
    lower_result_wrapper_call_with_consts, ResultObserverCallInput, ResultWrapperCallInput,
};
use self::call_lowering::{lower_call_expr_with_async, CallLoweringInput};
use self::call_routing::{lower_routed_call_or_core_builtin, RoutedCallLoweringInput};
use self::const_assembly::assemble_module_consts;
use self::direct_calls::{lower_direct_call_builtin_or_named_call, DirectCallBuiltinInput};
use self::expr_lowering::{
    lower_expr, lower_expr_with_async, lower_nested_expr_with_async,
    lower_nested_expr_with_async_and_consts, ExprWithAsyncInput, NestedExprWithConstsInput,
};
use self::function_lowering::find_impl_method_signature;
use self::function_lowering::{
    build_default_impl_method, build_default_impl_method_function, build_impl_method_function,
    impl_method_lookup_key, impl_method_lookup_keys, impl_method_symbol_name,
    impl_method_symbol_names, lower_function,
};
use self::generic_rewrite::{
    rewrite_generic_calls_in_function, GenericFunctionRewriteInput, GenericImplMethodTemplate,
};
use self::higher_order::expand_higher_order_functions;
use self::lambda_expansion::expand_module_lambdas;
use self::match_hoist::expand_effectful_match_scrutinees;
use self::metadata::{helper_visible_struct_annotations, lower_ast_attributes, ModuleConstValue};
use self::module_assembly::{
    build_impl_lookup, build_module_struct_table, build_visible_enum_defs,
    build_visible_struct_defs, lower_extern_items, lower_type_alias_items,
};
use self::return_inference::infer_missing_function_return_type;
use self::signature_building::{build_initial_function_signatures, FunctionSignature};
use self::specialization_pipeline::{build_lowered_functions_and_impls, LoweredFunctionsInput};
use self::text_handle_rewrite::rewrite_text_handle_helpers;
use self::validation::validate_declared_nir_types;
use self::validation_assignments::validate_ast_assignments;
use self::validation_generic_constraints::validate_ast_generic_constraints;
use self::validation_helpers::{
    async_boundary_violation_detail, async_parameter_violation_detail, render_type_name,
    select_expected_semantic_token_type, validate_benchmark_function_signature,
    validate_test_function_signature, validate_type_ref,
};
use aliases::*;
use nuis_semantics::model::{
    AstExpr, AstModule, AstStmt, AstTypeAlias, AstTypeRef, AstVisibility, NirExpr, NirFunction,
    NirModule, NirStmt, NirStructDef, NirTypeRef, NirUse, NirVisibility,
};
use types::*;

pub use self::doc_index::{extract_ast_doc_index, AstDocIndex, AstDocIndexItem};

pub fn frontend_name() -> &'static str {
    "nuisc-parser-minimal"
}

fn lower_visibility(visibility: AstVisibility) -> NirVisibility {
    match visibility {
        AstVisibility::Private => NirVisibility::Private,
        AstVisibility::Public => NirVisibility::Public,
    }
}

fn is_public_visibility(visibility: AstVisibility) -> bool {
    matches!(visibility, AstVisibility::Public)
}

fn render_field_access_path(expr: &AstExpr) -> Option<String> {
    match expr {
        AstExpr::Var(name) => Some(name.clone()),
        AstExpr::FieldAccess { base, field } => {
            Some(format!("{}.{}", render_field_access_path(base)?, field))
        }
        _ => None,
    }
}

pub fn parse_nuis_ast(input: &str) -> Result<AstModule, String> {
    let tokens = lexer::tokenize(input)?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_module()
}

pub fn lower_ast_to_nir(module: &AstModule) -> Result<NirModule, String> {
    lower_project_ast_to_nir(module, &[])
}

pub fn lower_project_ast_to_nir(
    module: &AstModule,
    local_modules: &[AstModule],
) -> Result<NirModule, String> {
    let expanded_module = expand_module_lambdas(module)?;
    let local_cpu_helpers = expanded_module
        .uses
        .iter()
        .filter(|item| item.domain == expanded_module.domain)
        .filter_map(|item| {
            local_modules
                .iter()
                .find(|candidate| candidate.domain == item.domain && candidate.unit == item.unit)
        })
        .collect::<Vec<_>>();
    let visible_type_aliases = build_visible_type_alias_map(&expanded_module, &local_cpu_helpers)?;
    let expanded_module = expand_higher_order_functions(&expanded_module, &visible_type_aliases)?;
    let expanded_module = expand_effectful_match_scrutinees(&expanded_module);
    let expanded_module = rewrite_text_handle_helpers(&expanded_module);
    let module = &expanded_module;
    validate_export_annotations(module)?;
    validate_extern_host_symbols(module)?;
    validate_host_symbol_bridge_annotations(module)?;
    validate_ast_assignments(module)?;
    for definition in &module.structs {
        validate_struct_annotations(definition)?;
    }
    for constant in &module.consts {
        validate_const_item(constant)?;
    }
    for function in &module.functions {
        validate_function_annotations(function)?;
    }

    let struct_defs = build_visible_struct_defs(module, &local_cpu_helpers, &visible_type_aliases)?;
    let enum_defs = build_visible_enum_defs(module, &local_cpu_helpers, &visible_type_aliases)?;
    let struct_table = struct_defs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();

    let initial_signatures =
        build_initial_function_signatures(module, &local_cpu_helpers, &visible_type_aliases)?;
    let mut signatures = initial_signatures.signatures;
    let generic_templates = initial_signatures.generic_templates;
    let concrete_module_functions = initial_signatures.concrete_module_functions;
    let module_struct_table = build_module_struct_table(module);
    let impl_lookup = build_impl_lookup(module, &visible_type_aliases)?;
    validate_ast_generic_constraints(
        module,
        &local_cpu_helpers,
        &visible_type_aliases,
        &impl_lookup,
    )?;
    let const_assembly = assemble_module_consts(
        module,
        &local_cpu_helpers,
        &visible_type_aliases,
        &signatures,
        &struct_table,
    )?;
    let lowered_consts = const_assembly.lowered_consts;
    let helper_const_maps = const_assembly.helper_const_maps;
    let module_const_values = const_assembly.module_const_values;
    let module_const_env = const_assembly.module_const_env;

    let lowered = build_lowered_functions_and_impls(LoweredFunctionsInput {
        module,
        local_cpu_helpers: &local_cpu_helpers,
        visible_type_aliases: &visible_type_aliases,
        module_const_values: &module_const_values,
        module_const_env: &module_const_env,
        helper_const_maps: &helper_const_maps,
        signatures: &mut signatures,
        struct_table: &struct_table,
        module_struct_table: &module_struct_table,
        impl_lookup: &impl_lookup,
        generic_templates: &generic_templates,
        concrete_module_functions: &concrete_module_functions,
    })?;
    let lowered_functions = lowered.functions;
    let lowered_traits = lowered.traits;
    let lowered_impls = lowered.impls;
    let (lowered_externs, lowered_extern_interfaces) =
        lower_extern_items(module, &visible_type_aliases)?;

    let nir = NirModule {
        annotations: lower_ast_attributes(&module.attributes),
        uses: module
            .uses
            .iter()
            .map(|item| NirUse {
                domain: item.domain.clone(),
                unit: item.unit.clone(),
            })
            .collect(),
        domain: module.domain.clone(),
        unit: module.unit.clone(),
        type_aliases: lower_type_alias_items(module, &visible_type_aliases)?,
        externs: lowered_externs,
        extern_interfaces: lowered_extern_interfaces,
        consts: lowered_consts,
        structs: struct_defs,
        enums: enum_defs,
        traits: lowered_traits,
        impls: lowered_impls,
        functions: lowered_functions,
    };
    validate_declared_nir_types(&nir)?;
    Ok(nir)
}

pub fn parse_nuis_module(input: &str) -> Result<NirModule, String> {
    let ast = parse_nuis_ast(input)?;
    lower_ast_to_nir(&ast)
}

pub fn collect_nir_tests(module: &NirModule) -> Vec<&NirFunction> {
    module
        .functions
        .iter()
        .filter(|function| function.test_name.is_some())
        .collect()
}

pub fn collect_nir_benchmarks(module: &NirModule) -> Vec<&NirFunction> {
    module
        .functions
        .iter()
        .filter(|function| function.benchmark_name.is_some())
        .collect()
}
