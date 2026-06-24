mod aliases;
mod annotations;
mod binary_lowering;
mod call_helpers;
mod call_lowering;
mod call_routing;
mod const_assembly;
mod data_builtins;
mod data_profile_builtins;
mod direct_calls;
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
use self::binary_lowering::lower_binary_expr_with_async;
use self::call_helpers::{
    ensure_mutex_guard_like, ensure_mutex_like, ensure_ref_like, ensure_spawn_input_safe,
    ensure_task_like, ensure_thread_like, lower_result_observer_call_with_consts,
    lower_result_wrapper_call_with_consts,
};
use self::call_lowering::lower_call_expr_with_async;
use self::call_routing::lower_routed_call_or_core_builtin;
use self::const_assembly::assemble_module_consts;
use self::direct_calls::lower_direct_call_builtin_or_named_call;
use self::expr_lowering::{
    lower_expr, lower_expr_with_async, lower_nested_expr_with_async,
    lower_nested_expr_with_async_and_consts,
};
use self::function_lowering::find_impl_method_signature;
use self::function_lowering::{
    build_default_impl_method, build_default_impl_method_function, build_impl_method_function,
    impl_method_lookup_key, impl_method_lookup_keys, impl_method_symbol_name,
    impl_method_symbol_names, lower_function,
};
use self::generic_rewrite::{rewrite_generic_calls_in_function, GenericImplMethodTemplate};
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
use self::specialization_pipeline::build_lowered_functions_and_impls;
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
    AstAttribute, AstAttributeArg, AstAttributeValue, AstEnumVariant, AstExpr, AstModule, AstStmt,
    AstStructField, AstTraitMethodSig, AstTypeAlias, AstTypeRef, AstVisibility, NirExpr,
    NirFunction, NirModule, NirStmt, NirStructDef, NirTypeRef, NirUse, NirVisibility,
};
use types::*;

pub fn frontend_name() -> &'static str {
    "nuisc-parser-minimal"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDocIndex {
    pub module_path: String,
    pub items: Vec<AstDocIndexItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDocIndexItem {
    pub kind: String,
    pub path: String,
    pub docs: Vec<String>,
    pub signature: Option<String>,
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

pub fn extract_ast_doc_index(module: &AstModule) -> AstDocIndex {
    let module_path = format!("{}.{}", module.domain, module.unit);
    let mut items = Vec::new();

    push_doc_index_item(
        &mut items,
        "module",
        module_path.clone(),
        doc_lines(&module.attributes),
        Some(format!("mod {} {}", module.domain, module.unit)),
    );

    for constant in &module.consts {
        push_doc_index_item(
            &mut items,
            "const",
            format!("{module_path}::{}", constant.name),
            doc_lines(&constant.attributes),
            Some(render_const_signature(constant)),
        );
    }
    for alias in &module.type_aliases {
        push_doc_index_item(
            &mut items,
            "type",
            format!("{module_path}::{}", alias.name),
            doc_lines(&alias.attributes),
            Some(render_type_alias_signature(alias)),
        );
    }
    for definition in &module.structs {
        let struct_path = format!("{module_path}::{}", definition.name);
        push_doc_index_item(
            &mut items,
            "struct",
            struct_path.clone(),
            doc_lines(&definition.attributes),
            Some(render_struct_signature(definition)),
        );
        for field in &definition.fields {
            push_doc_index_item(
                &mut items,
                "struct_field",
                format!("{struct_path}::{}", field.name),
                doc_lines(&field.attributes),
                Some(render_struct_field_signature(field)),
            );
        }
    }
    for definition in &module.enums {
        let enum_path = format!("{module_path}::{}", definition.name);
        push_doc_index_item(
            &mut items,
            "enum",
            enum_path.clone(),
            doc_lines(&definition.attributes),
            Some(render_enum_signature(definition)),
        );
        for variant in &definition.variants {
            push_doc_index_item(
                &mut items,
                "enum_variant",
                format!("{enum_path}::{}", variant.name),
                doc_lines(&variant.attributes),
                Some(render_enum_variant_signature(variant)),
            );
        }
    }
    for definition in &module.traits {
        let trait_path = format!("{module_path}::{}", definition.name);
        push_doc_index_item(
            &mut items,
            "trait",
            trait_path.clone(),
            doc_lines(&definition.attributes),
            Some(format!("trait {}", definition.name)),
        );
        for method in &definition.methods {
            push_doc_index_item(
                &mut items,
                "trait_method",
                format!("{trait_path}::{}", method.name),
                doc_lines(&method.attributes),
                Some(render_trait_method_signature(method)),
            );
        }
    }
    for function in &module.functions {
        push_doc_index_item(
            &mut items,
            "function",
            format!("{module_path}::{}", function.name),
            doc_lines(&function.attributes),
            Some(render_function_signature(function)),
        );
    }

    AstDocIndex { module_path, items }
}

fn push_doc_index_item(
    items: &mut Vec<AstDocIndexItem>,
    kind: &str,
    path: String,
    docs: Vec<String>,
    signature: Option<String>,
) {
    if docs.is_empty() {
        return;
    }
    items.push(AstDocIndexItem {
        kind: kind.to_owned(),
        path,
        docs,
        signature,
    });
}

fn doc_lines(attributes: &[AstAttribute]) -> Vec<String> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "doc")
        .filter_map(|attribute| match attribute.args.first() {
            Some(AstAttributeArg {
                name: None,
                value: AstAttributeValue::String(value),
            }) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn render_const_signature(constant: &nuis_semantics::model::AstConstItem) -> String {
    let ty = constant
        .ty
        .as_ref()
        .map(render_ast_type_name)
        .unwrap_or_else(|| "_".to_owned());
    format!("const {}: {}", constant.name, ty)
}

fn render_type_alias_signature(alias: &AstTypeAlias) -> String {
    format!(
        "type {} = {}",
        alias.name,
        render_ast_type_name(&alias.target)
    )
}

fn render_struct_signature(definition: &nuis_semantics::model::AstStructDef) -> String {
    format!("struct {}", definition.name)
}

fn render_struct_field_signature(field: &AstStructField) -> String {
    format!("field {}: {}", field.name, render_ast_type_name(&field.ty))
}

fn render_enum_signature(definition: &nuis_semantics::model::AstEnumDef) -> String {
    format!("enum {}", definition.name)
}

fn render_enum_variant_signature(variant: &AstEnumVariant) -> String {
    match &variant.kind {
        nuis_semantics::model::AstEnumVariantKind::Unit => format!("variant {}", variant.name),
        nuis_semantics::model::AstEnumVariantKind::Tuple(fields) => format!(
            "variant {}({})",
            variant.name,
            fields
                .iter()
                .map(render_ast_type_name)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        nuis_semantics::model::AstEnumVariantKind::Struct(fields) => format!(
            "variant {} {{ {} }}",
            variant.name,
            fields
                .iter()
                .map(render_struct_field_signature)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn render_trait_method_signature(method: &AstTraitMethodSig) -> String {
    let params = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type_name(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type_name(ty)))
        .unwrap_or_default();
    format!("fn {}({}){}", method.name, params, return_suffix)
}

fn render_function_signature(function: &nuis_semantics::model::AstFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type_name(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = function
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type_name(ty)))
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    format!(
        "{}fn {}({}){}",
        async_prefix, function.name, params, return_suffix
    )
}

fn render_ast_type_name(ty: &AstTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push('&');
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_ast_type_name)
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('>');
    }
    if ty.is_optional {
        out.push('?');
    }
    out
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

    let (mut signatures, generic_templates, concrete_module_functions) =
        build_initial_function_signatures(module, &local_cpu_helpers, &visible_type_aliases)?;
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

    let (lowered_functions, lowered_traits, lowered_impls) = build_lowered_functions_and_impls(
        module,
        &local_cpu_helpers,
        &visible_type_aliases,
        &module_const_values,
        &module_const_env,
        &helper_const_maps,
        &mut signatures,
        &struct_table,
        &module_struct_table,
        &impl_lookup,
        &generic_templates,
        &concrete_module_functions,
    )?;
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

pub fn collect_nir_tests<'a>(module: &'a NirModule) -> Vec<&'a NirFunction> {
    module
        .functions
        .iter()
        .filter(|function| function.test_name.is_some())
        .collect()
}

pub fn collect_nir_benchmarks<'a>(module: &'a NirModule) -> Vec<&'a NirFunction> {
    module
        .functions
        .iter()
        .filter(|function| function.benchmark_name.is_some())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_nuis_module;
    use nuis_semantics::model::{
        NirDataFlowState, NirExpr, NirKernelFlowState, NirShaderFlowState, NirStmt,
    };

    #[test]
    fn rejects_spawn_of_sync_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() {
                let task: Task<i64> = spawn(ping());
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("spawn(...) expects async function call"));
    }

    #[test]
    fn rejects_join_of_non_task_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return join(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("expects `Task<...>`"));
    }

    #[test]
    fn rejects_spawn_of_borrowed_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping(head_ref: ref Node) -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let head: ref Node = alloc_node(1, null());
                let task: Task<i64> = spawn(ping(borrow(head)));
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("does not currently allow borrowed task inputs"));
    }

    #[test]
    fn rejects_spawn_of_ref_typed_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping(head: ref Node) -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let head: ref Node = alloc_node(1, null());
                let task: Task<i64> = spawn(ping(head));
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("does not currently allow `ref` task inputs"));
    }

    #[test]
    fn rejects_async_function_ref_parameter_boundary() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping(head: ref Node) -> i64 {
                return 7;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot cross async boundary"));
        assert!(error.contains("`Task<...>`"));
    }

    #[test]
    fn rejects_async_function_result_family_return_boundary() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> TaskResult<i64> {
                return join_result(timeout(spawn(pong()), 16));
              }

              async fn pong() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return `TaskResult<i64>` across async boundary"));
        assert!(error.contains("*Result<...>"));
    }

    #[test]
    fn accepts_host_buffer_handle_to_extern_i64_host_handle_bridge() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              extern "c" fn host_stdin_read(buffer_handle: i64, len: i64) -> i64;

              fn main() -> i64 {
                let backing: ref Buffer = alloc_buffer(8, 0);
                return host_stdin_read(host_buffer_handle(backing), 8);
              }
            }
            "#,
        )
        .unwrap();

        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn rejects_ref_node_to_extern_i64_host_handle_slot() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              extern "c" fn host_stdin_read(buffer_handle: i64, len: i64) -> i64;

              fn main() -> i64 {
                let head: ref Node = alloc_node(1, null());
                return host_stdin_read(head, 8);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(
            error.contains("function `host_stdin_read` argument 1 expects `i64`, found `ref Node`")
        );
        assert!(error.contains("`ref Buffer -> i64`"));
    }

    #[test]
    fn accepts_ref_buffer_parameter_to_extern_host_handle_bridge() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              extern "c" fn host_stdin_read(buffer: ref Buffer, len: i64) -> i64;

              fn main() -> i64 {
                let backing: ref Buffer = alloc_buffer(8, 0);
                return host_stdin_read(backing, 8);
              }
            }
            "#,
        )
        .unwrap();

        let main = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            main.body.last(),
            Some(NirStmt::Return(Some(NirExpr::CpuExternCall { args, .. })))
                if matches!(args.first(), Some(NirExpr::HostBufferHandle(_)))
        ));
    }

    #[test]
    fn rejects_task_completed_on_raw_task_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> bool {
                let task: Task<i64> = spawn(ping());
                return task_completed(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("task_completed(...) expects `TaskResult<...>`"));
        assert!(error.contains("found `Task<i64>`"));
    }

    #[test]
    fn rejects_task_value_on_join_payload_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = spawn(ping());
                let value: i64 = join(task);
                return task_value(value);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("task_value(...) expects `TaskResult<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn rejects_network_style_sync_summary_calling_async_helper_directly() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct NetHttpClientExchangeSummary {
                exchange_value: i64,
              }

              struct NetSessionSummary {
                summary: NetHttpClientExchangeSummary,
                session_value: i64,
              }

              async fn capture_net_http_client_exchange_summary() -> NetHttpClientExchangeSummary {
                return NetHttpClientExchangeSummary { exchange_value: 41 };
              }

              fn capture_net_session_summary() -> NetSessionSummary {
                return NetSessionSummary {
                  summary: capture_net_http_client_exchange_summary(),
                  session_value: 99,
                };
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("can only be called inside `async fn`"));
        assert!(error.contains("capture_net_http_client_exchange_summary"));
    }

    #[test]
    fn rejects_network_style_spawn_of_sync_summary_builder() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct NetSessionSummary {
                session_value: i64,
              }

              fn capture_net_session_summary() -> NetSessionSummary {
                return NetSessionSummary { session_value: 99 };
              }

              fn main() -> i64 {
                let task: Task<NetSessionSummary> = spawn(capture_net_session_summary());
                return join(task).session_value;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("spawn(...) expects async function call"));
        assert!(error.contains("found sync function `capture_net_session_summary`"));
    }

    #[test]
    fn lowers_explicit_data_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod data FabricPlane {
              fn main() -> i64 {
                let pipe_result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
                let moved: bool = data_moved(pipe_result);
                let intake: DataResult<i64> = data_result(data_input_pipe(data_output_pipe(9)));
                let ready: bool = data_ready(intake);
                let value: i64 = data_value(intake);
                return value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataResult { state, .. },
                ..
            }) if ty.render() == "DataResult<Pipe<i64>>"
                && matches!(state, NirDataFlowState::Moved)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataMoved(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataResult { state, .. },
                ..
            }) if ty.render() == "DataResult<i64>"
                && matches!(state, NirDataFlowState::Ready)
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn rejects_data_result_of_non_data_operation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let result: DataResult<i64> = data_result(7);
                return data_value(result);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("data_result(...) expects a direct data operation"));
    }

    #[test]
    fn lowers_explicit_shader_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass_result: ShaderResult<Pass> = shader_result(shader_begin_pass(
                  shader_target("rgba8", 16, 16),
                  shader_pipeline("flat", "triangle"),
                  shader_viewport(16, 16)
                ));
                let frame_result: ShaderResult<Frame> = shader_result(shader_profile_render(
                  "SurfaceShader",
                  shader_profile_packet("SurfaceShader", 1, 2, 3)
                ));
                let ready: bool = shader_frame_ready(frame_result);
                let frame: Frame = shader_value(frame_result);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderResult { state, .. },
                ..
            }) if ty.render() == "ShaderResult<Pass>"
                && matches!(state, NirShaderFlowState::PassReady)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderResult { state, .. },
                ..
            }) if ty.render() == "ShaderResult<Frame>"
                && matches!(state, NirShaderFlowState::FrameReady)
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderFrameReady(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderValue(_),
                ..
            }) if ty.render() == "Frame"
        ));
    }

    #[test]
    fn lowers_shader_texture_sampling_builtins() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let texture: Texture =
                  shader_texture2d("r8", 2, 2, "1,2,3,4");
                let sampler: Sampler = shader_sampler("nearest", "clamp");
                let sampled: i64 = shader_sample_nearest(texture, sampler, 1, 0);
                return sampled;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderTexture2d { width, height, .. },
                ..
            }) if ty.render() == "Texture" && *width == 2 && *height == 2
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderSampler { .. },
                ..
            }) if ty.render() == "Sampler"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderSample { mode, .. },
                ..
            }) if ty.render() == "i64"
                && matches!(mode, nuis_semantics::model::NirShaderSampleMode::Nearest)
        ));
    }

    #[test]
    fn lowers_shader_uv_sampling_builtins() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let texture: Texture =
                  shader_texture2d("r8", 2, 2, "1,2,3,4");
                let sampler: Sampler = shader_sampler("linear", "clamp");
                let uv: UV = shader_uv(512, 256);
                let sampled: i64 = shader_sample_uv_linear(texture, sampler, uv);
                return sampled;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderUv { u, v },
                ..
            }) if ty.render() == "UV" && *u == 512 && *v == 256
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderSampleUv { mode, .. },
                ..
            }) if ty.render() == "i64"
                && matches!(mode, nuis_semantics::model::NirShaderSampleUvMode::Linear)
        ));
    }

    #[test]
    fn lowers_shader_binding_set_builtins() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pipeline: Pipeline = shader_pipeline("blit", "triangle-strip");
                let texture: Texture =
                  shader_texture2d("r8", 2, 2, "1,2,3,4");
                let sampler: Sampler = shader_sampler("linear", "clamp");
                let texture_binding: Binding = shader_texture_binding(0, texture);
                let sampler_binding: Binding = shader_sampler_binding(1, sampler);
                let bindings: BindingSet =
                  shader_bind_set(pipeline, texture_binding, sampler_binding);
                print(bindings);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding { kind, slot, .. },
                ..
            }) if ty.render() == "Binding" && kind == "texture_binding" && *slot == 0
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding { kind, slot, .. },
                ..
            }) if ty.render() == "Binding" && kind == "sampler_binding" && *slot == 1
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBindSet { bindings, .. },
                ..
            }) if ty.render() == "BindingSet" && bindings.len() == 2
        ));
    }

    #[test]
    fn lowers_shader_buffer_binding_builtins() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let uniform_buffer: ref Buffer = alloc_buffer(64, 0);
                let storage_buffer: ref Buffer = alloc_buffer(128, 1);
                let uniform_binding: Binding = shader_uniform_binding(2, uniform_buffer);
                let storage_binding: Binding = shader_storage_binding(3, storage_buffer);
                print(uniform_binding);
                print(storage_binding);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding { kind, slot, .. },
                ..
            }) if ty.render() == "Binding" && kind == "uniform_binding" && *slot == 2
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding { kind, slot, .. },
                ..
            }) if ty.render() == "Binding" && kind == "storage_binding" && *slot == 3
        ));
    }

    #[test]
    fn lowers_shader_buffer_binding_layout_builtins() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let uniform_buffer: ref Buffer = alloc_buffer(64, 0);
                let storage_buffer: ref Buffer = alloc_buffer(128, 1);
                let uniform_binding: Binding =
                  shader_uniform_binding_layout(2, "std140", uniform_buffer);
                let storage_binding: Binding =
                  shader_storage_binding_layout(3, "std430", storage_buffer);
                print(uniform_binding);
                print(storage_binding);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding { kind, slot, layout: Some(layout), .. },
                ..
            }) if ty.render() == "Binding"
                && kind == "uniform_binding"
                && *slot == 2
                && layout == "std140"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding { kind, slot, layout: Some(layout), .. },
                ..
            }) if ty.render() == "Binding"
                && kind == "storage_binding"
                && *slot == 3
                && layout == "std430"
        ));
    }

    #[test]
    fn lowers_shader_packet_binding_builtins() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
                let uniform_binding: Binding = shader_packet_uniform_binding(4, packet);
                let storage_binding: Binding = shader_packet_storage_binding(5, packet);
                print(uniform_binding);
                print(storage_binding);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding {
                    kind,
                    slot,
                    layout: Some(layout),
                    profile_contract: Some(profile_contract),
                    ..
                },
                ..
            }) if ty.render() == "Binding"
                && kind == "uniform_binding"
                && *slot == 4
                && layout == "std140"
                && profile_contract == "shader.profile.packet.nova.v1"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderBinding {
                    kind,
                    slot,
                    layout: Some(layout),
                    profile_contract: Some(profile_contract),
                    ..
                },
                ..
            }) if ty.render() == "Binding"
                && kind == "storage_binding"
                && *slot == 5
                && layout == "std430"
                && profile_contract == "shader.profile.packet.nova.v1"
        ));
    }

    #[test]
    fn lowers_nova_panel_packet_without_shader_unit_literal() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value:
                    NirExpr::ShaderProfilePacket {
                        unit,
                        packet_type_name,
                        accent: Some(_),
                        toggle_state: Some(_),
                        focus_index: Some(_),
                        ..
                    },
                ..
            }) if ty.render() == "NovaPanelPacket"
                && unit == "__nova__"
                && packet_type_name.as_deref() == Some("NovaPanelPacket")
        ));
    }

    #[test]
    fn lowers_nova_control_packet_builders() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
                let progress: NovaProgressPacket = nova_progress_packet(4, 10);
                let toggle: NovaTogglePacket = nova_toggle_packet(1, 1);
                let button: NovaButtonPacket = nova_button_packet(1, 9, 2);
                let text_input: NovaTextInputPacket =
                  nova_text_input_packet(8, 1, 4, 1, 1);
                let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 0);
                let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 1);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 0, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 0);
                let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
                let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
                let selection: NovaSelectionPacket = nova_selection_packet(1, 6, 1, 4);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSliderPacket" && type_name == "NovaSliderPacket"
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaProgressPacket" && type_name == "NovaProgressPacket"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTogglePacket" && type_name == "NovaTogglePacket"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaButtonPacket" && type_name == "NovaButtonPacket"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextInputPacket" && type_name == "NovaTextInputPacket"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectPacket" && type_name == "NovaSelectPacket"
        ));
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaCheckboxPacket" && type_name == "NovaCheckboxPacket"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaRadioPacket" && type_name == "NovaRadioPacket"
        ));
        assert!(matches!(
            function.body.get(8),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextAreaPacket" && type_name == "NovaTextAreaPacket"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTabsPacket" && type_name == "NovaTabsPacket"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaListPacket" && type_name == "NovaListPacket"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTablePacket" && type_name == "NovaTablePacket"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTreePacket" && type_name == "NovaTreePacket"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaInspectorPacket" && type_name == "NovaInspectorPacket"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaOutlinePacket" && type_name == "NovaOutlinePacket"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaThemePacket" && type_name == "NovaThemePacket"
        ));
        assert!(matches!(
            function.body.get(16),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionPacket" && type_name == "NovaSelectionPacket"
        ));
    }

    #[test]
    fn lowers_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
                let text_input: NovaTextInputPacket =
                  nova_text_input_packet(8, 1, 4, 1, 1);
                let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
                let slider_disabled: i64 = nova_slider_disabled(slider);
                let dirty: i64 = nova_text_input_dirty(text_input);
                let committed: i64 = nova_select_committed(select);
                return committed;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "disabled"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dirty"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "committed"
        ));
    }

    #[test]
    fn lowers_extended_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 1);
                let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 0);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 1, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 1);
                let checkbox_state: NovaCheckboxState = nova_checkbox_state(checkbox);
                let radio_state: NovaRadioState = nova_radio_state(radio);
                let textarea_state: NovaTextAreaState = nova_textarea_state(textarea);
                let tabs_state: NovaTabsState = nova_tabs_state(tabs);
                let checked: i64 = nova_checkbox_state_checked(checkbox_state);
                let radio_disabled: i64 = nova_radio_state_disabled(radio_state);
                let dirty: i64 = nova_textarea_state_dirty(textarea_state);
                let compact: i64 = nova_tabs_state_compact(tabs_state);
                return checked + radio_disabled + dirty + compact;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaCheckboxState" && type_name == "NovaCheckboxState"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaRadioState" && type_name == "NovaRadioState"
        ));
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextAreaState" && type_name == "NovaTextAreaState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTabsState" && type_name == "NovaTabsState"
        ));
        assert!(matches!(
            function.body.get(8),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "checked"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "disabled"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dirty"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "compact"
        ));
    }

    #[test]
    fn lowers_complex_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
                let list_state: NovaListState = nova_list_state(list);
                let table_state: NovaTableState = nova_table_state(table);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
                let tree_state: NovaTreeState = nova_tree_state(tree);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
                let inspector_state: NovaInspectorState = nova_inspector_state(inspector);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
                let outline_state: NovaOutlineState = nova_outline_state(outline);
                let dense: i64 = nova_list_state_dense(list_state);
                let selected: i64 = nova_list_state_selected(list_state);
                let zebra: i64 = nova_table_state_zebra(table_state);
                let selected_row: i64 = nova_table_state_selected_row(table_state);
                let expanded: i64 = nova_tree_state_expanded(tree_state);
                let tree_selected: i64 = nova_tree_state_selected(tree_state);
                let pinned: i64 = nova_inspector_state_pinned(inspector_state);
                let inspected: i64 = nova_inspector_state_selected(inspector_state);
                let collapsed: i64 = nova_outline_state_collapsed(outline_state);
                let outlined: i64 = nova_outline_state_selected(outline_state);
                return dense + selected + zebra + selected_row + expanded + tree_selected + pinned + inspected + collapsed + outlined;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaListState" && type_name == "NovaListState"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTableState" && type_name == "NovaTableState"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTreeState" && type_name == "NovaTreeState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaInspectorState" && type_name == "NovaInspectorState"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaOutlineState" && type_name == "NovaOutlineState"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dense"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "zebra"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected_row"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "expanded"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(16),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "pinned"
        ));
        assert!(matches!(
            function.body.get(17),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(18),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "collapsed"
        ));
        assert!(matches!(
            function.body.get(19),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
    }

    #[test]
    fn lowers_shared_nova_selection_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let selection: NovaSelectionPacket = nova_selection_packet(2, 6, 1, 4);
                let list: NovaListPacket = nova_list_packet(2, 6, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 2, 1);
                let tree: NovaTreePacket = nova_tree_packet(2, 6, 1, 7);
                let inspector: NovaInspectorPacket = nova_inspector_packet(2, 4, 1, 7);
                let outline: NovaOutlinePacket = nova_outline_packet(2, 6, 1, 7);
                let state: NovaSelectionState = nova_selection_state(selection);
                let list_selection: NovaSelectionState = nova_list_selection(list);
                let table_selection: NovaSelectionState = nova_table_selection(table);
                let tree_selection: NovaSelectionState = nova_tree_selection(tree);
                let inspector_selection: NovaSelectionState = nova_inspector_selection(inspector);
                let outline_selection: NovaSelectionState = nova_outline_selection(outline);
                let selected: i64 = nova_selection_state_selected(state);
                let span: i64 = nova_selection_state_span(list_selection);
                let mode: i64 = nova_selection_state_mode(table_selection);
                let origin: i64 = nova_selection_state_origin(tree_selection);
                let inspector_origin: i64 = nova_selection_state_origin(inspector_selection);
                let outline_origin: i64 = nova_selection_state_origin(outline_selection);
                return selected + span + mode + origin + inspector_origin + outline_origin;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "span"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "mode"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "origin"
        ));
    }

    #[test]
    fn lowers_nova_theme_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
                let state: NovaThemeState = nova_theme_state(theme);
                let accent: i64 = nova_theme_state_accent(state);
                let surface: i64 = nova_theme_state_surface(state);
                let panel_mode: i64 = nova_theme_state_panel_mode(state);
                let contrast: i64 = nova_theme_state_contrast(state);
                return accent + surface + panel_mode + contrast;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaThemeState" && type_name == "NovaThemeState"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "accent"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "contrast"
        ));
    }

    #[test]
    fn lowers_nova_render_state_contracts() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let surface_state: NovaSurfaceState = nova_surface_state(surface);
                let viewport_state: NovaViewportState = nova_viewport_state(viewport);
                let layer_state: NovaLayerState = nova_layer_state(layer);
                let density: i64 = nova_surface_state_density(surface_state);
                let width: i64 = nova_viewport_state_width(viewport_state);
                let visibility: i64 = nova_layer_state_visibility(layer_state);
                return density + width + visibility;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSurfaceState" && type_name == "NovaSurfaceState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaViewportState" && type_name == "NovaViewportState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLayerState" && type_name == "NovaLayerState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_state_contracts() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
                let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
                let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
                let scene_state: NovaSceneState = nova_scene_state(scene);
                let camera_state: NovaCameraState = nova_camera_state(camera);
                let material_state: NovaMaterialState = nova_material_state(material);
                let lights: i64 = nova_scene_state_light_count(scene_state);
                let zoom: i64 = nova_camera_state_zoom(camera_state);
                let emissive: i64 = nova_material_state_emissive(material_state);
                return lights + zoom + emissive;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneState" && type_name == "NovaSceneState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCameraState" && type_name == "NovaCameraState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaMaterialState" && type_name == "NovaMaterialState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_light_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
                let state: NovaLightState = nova_light_state(light);
                let intensity: i64 = nova_light_state_intensity(state);
                return intensity;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLightState" && type_name == "NovaLightState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_mesh_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
                let state: NovaMeshState = nova_mesh_state(mesh);
                let vertices: i64 = nova_mesh_state_vertex_count(state);
                return vertices;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaMeshState" && type_name == "NovaMeshState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_transform_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
                let state: NovaTransformState = nova_transform_state(transform);
                let scale: i64 = nova_transform_state_scale(state);
                return scale;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaTransformState" && type_name == "NovaTransformState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_node_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
                let state: NovaNodeState = nova_node_state(node);
                let depth: i64 = nova_node_state_depth(state);
                return depth;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaNodeState" && type_name == "NovaNodeState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_link_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let link: NovaSceneLinkPacket = nova_scene_link_packet(1, 2, 3, 4, 5, 6);
                let state: NovaSceneLinkState = nova_scene_link_state(link);
                let mesh_slot: i64 = nova_scene_link_state_mesh(state);
                return mesh_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneLinkState" && type_name == "NovaSceneLinkState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_instance_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let instance: NovaInstancePacket = nova_instance_packet(1, 2, 3, 4, 5, 6);
                let state: NovaInstanceState = nova_instance_state(instance);
                let count: i64 = nova_instance_state_count(state);
                return count;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaInstanceState" && type_name == "NovaInstanceState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_graph_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let graph: NovaSceneGraphPacket = nova_scene_graph_packet(1, 6, 3, 2, 1);
                let state: NovaSceneGraphState = nova_scene_graph_state(graph);
                let roots: i64 = nova_scene_graph_state_root(state);
                return roots;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneGraphState" && type_name == "NovaSceneGraphState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_node_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let node: NovaSceneNodePacket = nova_scene_node_packet(1, 2, 3, 4, 1);
                let state: NovaSceneNodeState = nova_scene_node_state(node);
                let child: i64 = nova_scene_node_state_first_child(state);
                return child;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneNodeState" && type_name == "NovaSceneNodeState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_instance_group_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let group: NovaInstanceGroupPacket = nova_instance_group_packet(1, 4, 3, 2, 8);
                let state: NovaInstanceGroupState = nova_instance_group_state(group);
                let visible: i64 = nova_instance_group_state_visible(state);
                return visible;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaInstanceGroupState" && type_name == "NovaInstanceGroupState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_cluster_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(1, 6, 3, 8, 1);
                let state: NovaSceneClusterState = nova_scene_cluster_state(cluster);
                let budget: i64 = nova_scene_cluster_state_budget(state);
                return budget;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneClusterState" && type_name == "NovaSceneClusterState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_visibility_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
                let state: NovaVisibilityState = nova_visibility_state(visibility);
                let visible: i64 = nova_visibility_state_visible(state);
                return visible;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaVisibilityState" && type_name == "NovaVisibilityState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_cull_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
                let state: NovaCullState = nova_cull_state(cull);
                let kept: i64 = nova_cull_state_kept(state);
                return kept;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCullState" && type_name == "NovaCullState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_lod_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
                let state: NovaLodState = nova_lod_state(lod);
                let active: i64 = nova_lod_state_active(state);
                return active;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLodState" && type_name == "NovaLodState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_streaming_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
                let state: NovaStreamingState = nova_streaming_state(streaming);
                let resident: i64 = nova_streaming_state_resident(state);
                return resident;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaStreamingState" && type_name == "NovaStreamingState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_residency_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
                let state: NovaResidencyState = nova_residency_state(residency);
                let committed: i64 = nova_residency_state_committed(state);
                return committed;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaResidencyState" && type_name == "NovaResidencyState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_eviction_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
                let state: NovaEvictionState = nova_eviction_state(eviction);
                let evicted: i64 = nova_eviction_state_evicted(state);
                return evicted;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaEvictionState" && type_name == "NovaEvictionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_prefetch_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
                let state: NovaPrefetchState = nova_prefetch_state(prefetch);
                let requested: i64 = nova_prefetch_state_requested(state);
                return requested;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPrefetchState" && type_name == "NovaPrefetchState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_budget_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
                let state: NovaBudgetState = nova_budget_state(budget);
                let total: i64 = nova_budget_state_total(state);
                return total;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaBudgetState" && type_name == "NovaBudgetState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_pressure_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
                let state: NovaPressureState = nova_pressure_state(pressure);
                let level: i64 = nova_pressure_state_level(state);
                return level;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPressureState" && type_name == "NovaPressureState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_thermal_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
                let state: NovaThermalState = nova_thermal_state(thermal);
                let level: i64 = nova_thermal_state_level(state);
                return level;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaThermalState" && type_name == "NovaThermalState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_power_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
                let state: NovaPowerState = nova_power_state(power);
                let level: i64 = nova_power_state_level(state);
                return level;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPowerState" && type_name == "NovaPowerState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_latency_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
                let state: NovaLatencyState = nova_latency_state(latency);
                let frame: i64 = nova_latency_state_frame(state);
                return frame;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLatencyState" && type_name == "NovaLatencyState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_pacing_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
                let state: NovaFramePacingState = nova_frame_pacing_state(pacing);
                let cadence: i64 = nova_frame_pacing_state_cadence(state);
                return cadence;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFramePacingState" && type_name == "NovaFramePacingState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_jank_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
                let state: NovaJankState = nova_jank_state(jank);
                let spikes: i64 = nova_jank_state_spikes(state);
                return spikes;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaJankState" && type_name == "NovaJankState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_variance_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
                let state: NovaFrameVarianceState = nova_frame_variance_state(variance);
                let frame: i64 = nova_frame_variance_state_frame(state);
                return frame;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFrameVarianceState" && type_name == "NovaFrameVarianceState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_pass_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
                let state: NovaPassState = nova_pass_state(pass);
                let samples: i64 = nova_pass_state_sample_count(state);
                return samples;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPassState" && type_name == "NovaPassState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
                let state: NovaFrameState = nova_frame_state(frame);
                let exposure: i64 = nova_frame_state_exposure(state);
                return exposure;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFrameState" && type_name == "NovaFrameState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_target_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
                let state: NovaTargetState = nova_target_state(target);
                let msaa: i64 = nova_target_state_multisample(state);
                return msaa;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaTargetState" && type_name == "NovaTargetState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_graph_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
                let state: NovaFrameGraphState = nova_frame_graph_state(frame_graph);
                let passes: i64 = nova_frame_graph_state_passes(state);
                return passes;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFrameGraphState" && type_name == "NovaFrameGraphState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_attachment_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
                let state: NovaAttachmentState = nova_attachment_state(attachment);
                let format_kind: i64 = nova_attachment_state_format_kind(state);
                return format_kind;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaAttachmentState" && type_name == "NovaAttachmentState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_pass_chain_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
                let state: NovaPassChainState = nova_pass_chain_state(pass_chain);
                let stages: i64 = nova_pass_chain_state_stages(state);
                return stages;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPassChainState" && type_name == "NovaPassChainState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_barrier_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
                let state: NovaBarrierState = nova_barrier_state(barrier);
                let flush_mode: i64 = nova_barrier_state_flush_mode(state);
                return flush_mode;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaBarrierState" && type_name == "NovaBarrierState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_resource_set_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
                let state: NovaResourceSetState = nova_resource_set_state(resource_set);
                let residency: i64 = nova_resource_set_state_residency(state);
                return residency;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaResourceSetState" && type_name == "NovaResourceSetState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_schedule_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
                let state: NovaScheduleState = nova_schedule_state(schedule);
                let budget: i64 = nova_schedule_state_async_budget(state);
                return budget;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaScheduleState" && type_name == "NovaScheduleState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_submission_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
                let state: NovaSubmissionState = nova_submission_state(submission);
                let batches: i64 = nova_submission_state_batches(state);
                return batches;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSubmissionState" && type_name == "NovaSubmissionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_queue_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
                let state: NovaQueueState = nova_queue_state(queue);
                let budget: i64 = nova_queue_state_budget(state);
                return budget;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaQueueState" && type_name == "NovaQueueState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_semaphore_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
                let state: NovaSemaphoreState = nova_semaphore_state(semaphore);
                let scope: i64 = nova_semaphore_state_scope(state);
                return scope;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSemaphoreState" && type_name == "NovaSemaphoreState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_timeline_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
                let state: NovaTimelineState = nova_timeline_state(timeline);
                let epoch: i64 = nova_timeline_state_epoch(state);
                return epoch;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaTimelineState" && type_name == "NovaTimelineState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_fence_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
                let state: NovaFenceState = nova_fence_state(fence);
                let scope: i64 = nova_fence_state_scope(state);
                return scope;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFenceState" && type_name == "NovaFenceState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_signal_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 4);
                let state: NovaSignalState = nova_signal_state(signal);
                let phase: i64 = nova_signal_state_phase(state);
                return phase;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSignalState" && type_name == "NovaSignalState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_event_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let event: NovaEventPacket = nova_event_packet(1, 2, 3, 4);
                let state: NovaEventState = nova_event_state(event);
                let route: i64 = nova_event_state_route(state);
                return route;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaEventState" && type_name == "NovaEventState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_dispatch_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 4);
                let state: NovaDispatchState = nova_dispatch_state(dispatch);
                let queue_kind: i64 = nova_dispatch_state_queue_kind(state);
                return queue_kind;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaDispatchState" && type_name == "NovaDispatchState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_feedback_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 4);
                let state: NovaFeedbackState = nova_feedback_state(feedback);
                let status: i64 = nova_feedback_state_status(state);
                return status;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFeedbackState" && type_name == "NovaFeedbackState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_intent_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 4);
                let state: NovaIntentState = nova_intent_state(intent);
                let target_slot: i64 = nova_intent_state_target(state);
                return target_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaIntentState" && type_name == "NovaIntentState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_reaction_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 4);
                let state: NovaReactionState = nova_reaction_state(reaction);
                let result_slot: i64 = nova_reaction_state_result(state);
                return result_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaReactionState" && type_name == "NovaReactionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_outcome_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 4);
                let state: NovaOutcomeState = nova_outcome_state(outcome);
                let final_slot: i64 = nova_outcome_state_final(state);
                return final_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaOutcomeState" && type_name == "NovaOutcomeState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_resolution_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 4);
                let state: NovaResolutionState = nova_resolution_state(resolution);
                let commit_slot: i64 = nova_resolution_state_commit(state);
                return commit_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaResolutionState" && type_name == "NovaResolutionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_commit_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 4);
                let state: NovaCommitState = nova_commit_state(commit);
                let applied_slot: i64 = nova_commit_state_applied(state);
                return applied_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCommitState" && type_name == "NovaCommitState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_snapshot_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 4);
                let state: NovaSnapshotState = nova_snapshot_state(snapshot);
                let source_slot: i64 = nova_snapshot_state_source(state);
                return source_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSnapshotState" && type_name == "NovaSnapshotState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_checkpoint_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 4);
                let state: NovaCheckpointState = nova_checkpoint_state(checkpoint);
                let anchor_slot: i64 = nova_checkpoint_state_anchor(state);
                return anchor_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCheckpointState" && type_name == "NovaCheckpointState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_panel_from_parts_builder() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let header: NovaHeaderPacket = nova_header_packet(8);
                let slider_color: NovaSliderPacket = nova_slider_packet(1);
                let slider_speed: NovaSliderPacket = nova_slider_packet(2);
                let slider_radius: NovaSliderPacket = nova_slider_packet(3);
                let sliders: NovaSliderGroupPacket =
                  nova_slider_group_packet(slider_color, slider_speed, slider_radius);
                let toggle: NovaTogglePacket = nova_toggle_packet(1);
                let progress: NovaProgressPacket = nova_progress_packet(2);
                let meter: NovaMeterPacket = nova_meter_packet(3);
                let button: NovaButtonPacket = nova_button_packet(1, 8);
                let text_input: NovaTextInputPacket = nova_text_input_packet(4, 1);
                let select: NovaSelectPacket = nova_select_packet(0, 8);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 8);
                let radio: NovaRadioPacket = nova_radio_packet(1, 4, 8);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(0, 4, 8);
                let list: NovaListPacket = nova_list_packet(1, 5, 8);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 8);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 8);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 8);
                let theme: NovaThemePacket = nova_theme_packet(8, 3, 1, 2);
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
                let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
                let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
                let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
                let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
                let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
                let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
                let scene_link: NovaSceneLinkPacket = nova_scene_link_packet(2, 12, 9, 8, 1, 1);
                let instance: NovaInstancePacket = nova_instance_packet(2, 3, 2, 1, 8, 1);
                let scene_graph: NovaSceneGraphPacket = nova_scene_graph_packet(2, 6, 3, 3, 1);
                let scene_node: NovaSceneNodePacket = nova_scene_node_packet(2, 4, 5, 3, 1);
                let instance_group: NovaInstanceGroupPacket = nova_instance_group_packet(3, 4, 3, 1, 8);
                let scene_cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(2, 6, 3, 8, 1);
                let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
                let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
                let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
                let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
                let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
                let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
                let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
                let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
                let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
                let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
                let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
                let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
                let frame_pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
                let frame_variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
                let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
                let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
                let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
                let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
                let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
                let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
                let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
                let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
                let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
                let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
                let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
                let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
                let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
                let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
                let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
                let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 1);
                let event: NovaEventPacket = nova_event_packet(1, 2, 3, 1);
                let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 1);
                let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 1);
                let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 1);
                let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 1);
                let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 1);
                let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 1);
                let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 1);
                let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 1);
                let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 1);
                let focus: NovaFocusPacket = nova_focus_packet(2);
                let panel: NovaPanelPacket = nova_panel_from_parts(
                  header,
                  sliders,
                  toggle,
                  progress,
                  meter,
                  button,
                  text_input,
                  select,
                  checkbox,
                  radio,
                  textarea,
                  tabs,
                  list,
                  table,
                  tree,
                  inspector,
                  outline,
                  theme,
                  surface,
                  viewport,
                  layer,
                  scene,
                  camera,
                  material,
                  light,
                  mesh,
                  transform,
                  node,
                  scene_link,
                  instance,
                  scene_graph,
                      scene_node,
                      instance_group,
                      scene_cluster,
                      visibility,
                  cull,
                        lod,
                  streaming,
                  residency,
                  eviction,
                  prefetch,
                  budget,
                  pressure,
                  thermal,
                  power,
                  latency,
                  frame_pacing,
                  frame_variance,
                  jank,
                  pass,
                  frame,
                  target,
                  frame_graph,
                  attachment,
                  pass_chain,
                  barrier,
                  resource_set,
                  schedule,
                  submission,
                  queue,
                  semaphore,
                  timeline,
                  fence,
                  signal,
                  event,
                  dispatch,
                  feedback,
                  intent,
                  reaction,
                  outcome,
                  resolution,
                  commit,
                  snapshot,
                  checkpoint,
                  focus
                );
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPanelPacket" && type_name == "NovaPanelPacket",
            _ => false,
        }));
    }

    #[test]
    fn lowers_explicit_kernel_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let lanes: KernelResult<i64> = kernel_result(kernel_profile_batch_lanes("KernelUnit"));
                let ready: bool = kernel_config_ready(lanes);
                let value: i64 = kernel_value(lanes);
                return value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelResult { state, .. },
                ..
            }) if ty.render() == "KernelResult<i64>"
                && matches!(state, NirKernelFlowState::ConfigReady)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelConfigReady(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn lowers_explicit_kernel_result_helpers_from_tensor_reductions() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let total: KernelResult<i64> = kernel_result(kernel_reduce_sum(input));
                let peak: KernelResult<i64> = kernel_result(kernel_reduce_max(input));
                let avg: KernelResult<i64> = kernel_result(kernel_reduce_mean(input));
                return kernel_value(total) + kernel_value(peak) + kernel_value(avg);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelResult { value, state },
                ..
            } => {
                ty.render() == "KernelResult<i64>"
                    && matches!(state, NirKernelFlowState::ConfigReady)
                    && matches!(value.as_ref(), NirExpr::KernelReduceSum(_))
            }
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                value: NirExpr::KernelResult { value, .. },
                ..
            } => matches!(value.as_ref(), NirExpr::KernelReduceMax(_)),
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                value: NirExpr::KernelResult { value, .. },
                ..
            } => matches!(value.as_ref(), NirExpr::KernelReduceMean(_)),
            _ => false,
        }));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let weights = kernel_tensor(3, 2, "1,-2,3,0,2,1");
                let bias = kernel_tensor(1, 2, "-4,3");
                let projected = kernel_matmul(input, weights);
                let shifted = kernel_add_bias(projected, bias);
                let activated = kernel_relu(shifted);
                return kernel_reduce_sum(activated);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelTensor { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelMatmul { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelAddBias { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelRelu(_),
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelReduceSum(_))))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_inspect_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let layout = kernel_shape(input);
                let rows: i64 = kernel_rows(input);
                let cols: i64 = kernel_cols(input);
                let first_row = kernel_row(input);
                let first_col = kernel_col(input);
                return kernel_element_at(first_row, 0, 1) + rows + cols + kernel_element_at(first_col, 0, 0);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelShape(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelRows(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelCols(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelRow(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelCol(_),
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_map_zip_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let lifted = kernel_map(input, "add_scalar", 3);
                let scaled = kernel_map(lifted, "mul_scalar", 2);
                let activated = kernel_map(scaled, "relu");
                let mask = kernel_tensor(1, 3, "1,0,1");
                let mixed = kernel_zip(activated, mask, "mul");
                return kernel_reduce_sum(mixed);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelMap { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelZip { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelReduceSum(_))))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reshape_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let reshaped = kernel_reshape(input, 3, 2);
                return kernel_element_at(reshaped, 2, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReshape { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_broadcast_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let widened = kernel_broadcast(input, 2, 3);
                return kernel_element_at(widened, 1, 2);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelBroadcast { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reduction_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let maxed: i64 = kernel_reduce_max(input);
                return maxed + kernel_reduce_mean(input);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceMax(_),
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_selection_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let hi: i64 = kernel_argmax(input);
                return hi + kernel_argmin(input);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelArgmax(_),
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reduce_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let row_sums = kernel_reduce_sum_axis(input, "rows");
                return kernel_element_at(row_sums, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceSumAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reduce_axis_family_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let row_max = kernel_reduce_max_axis(input, "rows");
                let col_mean = kernel_reduce_mean_axis(input, "cols");
                return kernel_element_at(row_max, 0, 0) + kernel_element_at(col_mean, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceMaxAxis { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceMeanAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::Binary { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_select_axis_family_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let row_hi = kernel_argmax_axis(input, "rows");
                let col_lo = kernel_argmin_axis(input, "cols");
                return kernel_element_at(row_hi, 0, 1) + kernel_element_at(col_lo, 0, 2);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelArgmaxAxis { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelArgminAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::Binary { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_topk_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let top2_rows = kernel_topk_axis(input, "rows", 2);
                return kernel_element_at(top2_rows, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelTopkAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_map_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "-2,4,-6,1,-3,5");
                let activated = kernel_map_axis(input, "rows", "relu");
                let lifted = kernel_map_axis(activated, "cols", "add_scalar", 2);
                return kernel_element_at(lifted, 0, 0);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelMapAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_sort_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let sorted_rows = kernel_sort_axis(input, "rows");
                return kernel_element_at(sorted_rows, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelSortAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_order_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let sorted = kernel_sort(input);
                let top2 = kernel_topk(input, 2);
                return kernel_element_at(sorted, 0, 0) + kernel_element_at(top2, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelSort(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelTopk { .. },
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_timeout_on_task_handle() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuTimeout { .. },
                ..
            }) if ty.render() == "Task<i64>"
        ));
    }

    #[test]
    fn lowers_explicit_join_result_and_task_state_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                let result: TaskResult<i64> = join_result(task);
                if task_completed(result) {
                  return task_value(result);
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
                ..
            }) if ty.render() == "TaskResult<i64>"
        ));
    }

    #[test]
    fn lowers_thread_and_mutex_builtins_with_expected_surface_types() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let worker: Thread<i64> = thread_spawn(ping());
                let joined: TaskResult<i64> = thread_join_result(worker);
                let lock: Mutex<i64> = mutex_new(11);
                let guard: MutexGuard<i64> = mutex_lock(lock);
                let value: i64 = mutex_value(guard);
                return value + task_value(joined);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuThreadSpawn { .. },
                ..
            }) if ty.render() == "Thread<i64>"
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuThreadJoinResult(_),
                ..
            }) if ty.render() == "TaskResult<i64>"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuMutexNew(_),
                ..
            }) if ty.render() == "Mutex<i64>"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuMutexLock(_),
                ..
            }) if ty.render() == "MutexGuard<i64>"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuMutexValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn rejects_thread_spawn_with_sync_callee() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let worker: Thread<i64> = thread_spawn(ping());
                return thread_join(worker);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("thread_spawn(...) expects async function call"));
        assert!(error.contains("sync function `ping`"));
    }

    #[test]
    fn rejects_thread_join_of_non_thread_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return thread_join(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("thread_join(...) expects `Thread<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn rejects_thread_join_result_of_non_thread_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let joined: TaskResult<i64> = thread_join_result(7);
                return task_value(joined);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("thread_join_result(...) expects `Thread<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn rejects_mutex_lock_of_non_mutex_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let guard: MutexGuard<i64> = mutex_lock(7);
                return mutex_value(guard);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mutex_lock(...) expects `Mutex<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn rejects_mutex_unlock_of_non_guard_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let lock: Mutex<i64> = mutex_unlock(7);
                let guard: MutexGuard<i64> = mutex_lock(lock);
                return mutex_value(guard);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mutex_unlock(...) expects `MutexGuard<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn rejects_mutex_value_of_non_guard_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return mutex_value(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mutex_value(...) expects `MutexGuard<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn rejects_mutex_new_of_result_family_payload() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = spawn(ping());
                let joined: TaskResult<i64> = join_result(task);
                let lock: Mutex<TaskResult<i64>> = mutex_new(joined);
                let guard: MutexGuard<TaskResult<i64>> = mutex_lock(lock);
                let observed: TaskResult<i64> = mutex_value(guard);
                return task_value(observed);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(
            error.contains("mutex_new(...) expects a staged mutex payload value")
                || error.contains("`Mutex<...>` expects a staged value payload"),
            "{error}"
        );
        assert!(error.contains("TaskResult<i64>"), "{error}");
    }

    #[test]
    fn rejects_timeout_with_non_integer_limit() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), "slow");
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("expects integer limit"));
    }

    #[test]
    fn rejects_await_inside_sync_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() {
                await ping();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("`await`"));
        assert!(error.contains("async fn"));
    }

    #[test]
    fn rejects_async_function_returning_ref_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn head() -> ref Node {
                return null();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return"));
        assert!(error.contains("ref Node"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_returning_result_family() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main() -> DataResult<i64> {
                return data_result(data_input_pipe(data_output_pipe(7)));
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("DataResult<i64>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_taking_instance_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn render(shader: Instance<SurfaceShader>) {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `shader`"));
        assert!(error.contains("Instance<SurfaceShader>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn accepts_async_function_taking_shader_result_family_param() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(result: ShaderResult<Frame>) -> i64 {
                if shader_frame_ready(result) {
                  return 1;
                }
                return 0;
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "consume")
            .unwrap();
        assert_eq!(function.params[0].ty.render(), "ShaderResult<Frame>");
    }

    #[test]
    fn accepts_async_function_taking_kernel_result_family_param() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(result: KernelResult<i64>) -> i64 {
                if kernel_config_ready(result) {
                  return kernel_value(result);
                }
                return 0;
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "consume")
            .unwrap();
        assert_eq!(function.params[0].ty.render(), "KernelResult<i64>");
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_ref_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct RefPacket {
                head: ref Node
              }

              async fn consume(packet: RefPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("RefPacket"));
        assert!(error.contains("nested field `RefPacket.head`"));
        assert!(error.contains("ref Node"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_returning_struct_with_nested_ref_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct RefPacket {
                head: ref Node
              }

              async fn emit() -> RefPacket {
                return RefPacket { head: null() };
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return `RefPacket` across async boundary"));
        assert!(error.contains("nested field `RefPacket.head`"));
        assert!(error.contains("ref Node"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_optional_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct OptionalPacket {
                value: i64?
              }

              async fn consume(packet: OptionalPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("OptionalPacket"));
        assert!(error.contains("nested field `OptionalPacket.value`"));
        assert!(error.contains("i64?"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_instance_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct ShaderPacket {
                shader: Instance<SurfaceShader>
              }

              async fn consume(packet: ShaderPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("ShaderPacket"));
        assert!(error.contains("nested field `ShaderPacket.shader`"));
        assert!(error.contains("Instance<SurfaceShader>"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_result_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct ResultPacket {
                result: TaskResult<i64>
              }

              async fn consume(packet: ResultPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("ResultPacket"));
        assert!(error.contains("nested field `ResultPacket.result`"));
        assert!(error.contains("TaskResult<i64>"));
    }

    #[test]
    fn rejects_async_function_taking_window_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(window: Window<i64>) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `window`"));
        assert!(error.contains("Window<i64>"));
        assert!(error.contains("resource-bearing"));
    }

    #[test]
    fn accepts_sync_function_declaring_staged_thread_and_mutex_types() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn hold(worker: Thread<i64>, lock: Mutex<i64>, guard: MutexGuard<i64>) -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "hold")
            .unwrap();
        assert_eq!(function.params[0].ty.render(), "Thread<i64>");
        assert_eq!(function.params[1].ty.render(), "Mutex<i64>");
        assert_eq!(function.params[2].ty.render(), "MutexGuard<i64>");
    }

    #[test]
    fn rejects_async_function_taking_thread_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(worker: Thread<i64>) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `worker`"));
        assert!(error.contains("Thread<i64>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_taking_mutex_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(lock: Mutex<i64>) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `lock`"));
        assert!(error.contains("Mutex<i64>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_marker_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct MarkerPacket {
                ready: Marker<CpuToShader>
              }

              async fn consume(packet: MarkerPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("MarkerPacket"));
        assert!(error.contains("nested field `MarkerPacket.ready`"));
        assert!(error.contains("resource-bearing `Marker<CpuToShader>`"));
    }

    #[test]
    fn allows_async_function_taking_nested_scalar_struct_payload() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              struct ScalarPair {
                lhs: i64,
                rhs: i64
              }

              struct NestedPacket {
                pair: ScalarPair,
                bias: i64
              }

              async fn add(packet: NestedPacket) -> i64 {
                return packet.pair.lhs + packet.pair.rhs + packet.bias;
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn allows_async_function_taking_nested_text_struct_payload() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              struct MessagePacket {
                message: String
              }

              struct LabeledMessage {
                packet: MessagePacket,
                label: String
              }

              async fn show(input: LabeledMessage) -> i64 {
                return 5;
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn rejects_async_shader_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod shader SurfaceShader {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod shader SurfaceShader"));
        assert!(error.contains("async fn profile"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_data_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod data FabricPlane {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod data FabricPlane"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_kernel_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod kernel KernelUnit {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod kernel KernelUnit"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_main_with_parameters() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main(seed: i64) {
                print(seed);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("async entry"));
        assert!(error.contains("Main::main"));
        assert!(error.contains("cannot take parameters"));
    }

    #[test]
    fn rejects_async_call_without_await() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() -> i64 {
                return ping();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("must be used under `await`"));
    }
}
