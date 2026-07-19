#![allow(
    clippy::if_same_then_else,
    clippy::needless_borrow,
    clippy::ptr_arg,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::useless_format,
    clippy::obfuscated_if_else
)]
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{CpuLlvmLoweringClass, Resource, YirModule};
use yir_verify::verify_module;
mod async_resource_lowering;
mod bitwise_lowering;
mod call_lowering;
mod call_return;
mod cast_lowering;
mod effect_flow_loop_lowering;
mod emit_utils;
mod extern_abi;
mod extern_call_lowering;
mod facts;
mod function_lowering;
mod guard_host_call;
mod guard_return_lowering;
mod loop_async_post_flow_payload;
mod loop_async_post_flow_source;
mod loop_carry_payload;
mod loop_carry_read_source;
mod loop_carry_scaled_source;
mod loop_carry_source;
mod loop_chain_result;
mod loop_effect_action;
mod loop_expr;
mod loop_flow_control_lowering;
mod loop_scalar;
mod memory_lowering;
mod owned_tree_call_args;
mod param_lowering;
mod preclassified_lowering;
mod print_lowering;
mod return_lowering;
mod scalar_equality_lowering;
mod scalar_lowering;
mod scalar_order_lowering;
mod select_lowering;
mod simple_loop_lowering;
mod static_lowering;
mod task_owned_payload;
mod topology;
mod types;
mod value_ref;
mod variant_select;

use async_resource_lowering::lower_cpu_async_resource_node;
use bitwise_lowering::lower_cpu_bitwise_node;
use call_lowering::{lower_cpu_branch_owned_call_node, lower_cpu_call_node};
use call_return::{
    cpu_call_scalar_kind_for_instruction, cpu_param_binding, cpu_scalar_kind_llvm_type,
    emit_typed_return_from_last_value,
};
use cast_lowering::lower_cpu_cast_node;
use effect_flow_loop_lowering::lower_cpu_effect_flow_loop_node;
use emit_utils::{fresh_block, fresh_global, fresh_reg, llvm_c_string_bytes, lower_buffer_fill};
use extern_abi::render_dynamic_extern_decls;
use extern_call_lowering::lower_cpu_extern_call_node;
use facts::KnownFacts;
use function_lowering::emit_cpu_function;
use guard_return_lowering::{lower_cpu_guard_return_node, GuardReturnLoweringOutcome};
use loop_async_post_flow_payload::async_post_flow_carry_source_payload_len;
use loop_async_post_flow_source::resolve_source_for_async_post_flow;
use loop_carry_payload::{loop_async_chain_carry_payload_len, loop_carry_payload_len};
use loop_carry_read_source::try_resolve_loop_carry_read_source;
use loop_carry_scaled_source::try_resolve_loop_carry_scaled_source;
use loop_carry_source::resolve_loop_carry_term;
use loop_chain_result::{insert_i64_loop_chain_result, insert_scalar_loop_chain_result};
use loop_expr::{
    canonical_loop_block_prefix, canonical_loop_instruction, collect_resolved_loop_flow_leaves,
    parse_loop_flow_expr_for_llvm, resolve_loop_flow_expr_for_llvm, ResolvedLoopControlExpr,
};
use loop_flow_control_lowering::emit_loop_flow_control_expr;
use loop_scalar::{
    coerce_to_loop_scalar, emit_loop_compare, emit_loop_numeric_op, infer_loop_scalar_kind,
    loop_scalar_llvm_type, loop_scalar_value_ref,
};
use memory_lowering::lower_cpu_memory_node;
use param_lowering::lower_cpu_param_node;
use preclassified_lowering::{
    lower_cpu_aggregate_node, lower_cpu_literal_node, lower_cpu_pointer_node,
    lower_network_observer_node,
};
use print_lowering::lower_cpu_print_node;
use return_lowering::{lower_cpu_return_node, ReturnLoweringOutcome};
use scalar_equality_lowering::lower_cpu_scalar_equality_node;
use scalar_lowering::lower_cpu_scalar_node;
use scalar_order_lowering::lower_cpu_scalar_order_node;
use select_lowering::lower_cpu_select_node;
use simple_loop_lowering::lower_cpu_simple_loop_node;
use static_lowering::lower_cpu_static_node;
use topology::topological_order;
pub(crate) use types::{
    CpuCallScalarKind, CpuHelperSignature, CpuLoopScalarKind, EmittedCpuFunction,
    LlvmLoweringState, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef,
    NetworkResultLlvmValueRef, StructLlvmValueRef, TaskLlvmValueRef, TaskResultLlvmValueRef,
    TaskThunkArgument, ThreadLlvmValueRef, VariantUnionLlvmValueRef,
};
use value_ref::coerce_to_i64;
pub fn emit_module(module: &YirModule) -> Result<String, String> {
    verify_module(module)?;

    let resources = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource))
        .collect::<BTreeMap<String, &Resource>>();
    let order = topological_order(module)?;
    let helper_lanes = module
        .node_lanes
        .values()
        .filter(|lane| lane.starts_with("fn:"))
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut global_counter = 0usize;
    let mut globals = Vec::new();
    let mut helper_defs = Vec::new();

    let mut helper_signatures = BTreeMap::<String, CpuHelperSignature>::new();
    for lane in &helper_lanes {
        let function_name = lane.trim_start_matches("fn:").to_owned();
        let mut params = module
            .nodes
            .iter()
            .filter(|node| module.node_lanes.get(&node.name) == Some(lane))
            .filter_map(|node| {
                let kind = cpu_call_scalar_kind_for_instruction(&node.op.instruction)?;
                node.op
                    .instruction
                    .starts_with("param_")
                    .then_some((node, kind))
            })
            .map(|(node, kind)| {
                let index = node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "invalid {} index `{}`",
                        node.op.full_name(),
                        node.op.args[0]
                    )
                })?;
                Ok((index, node.name.clone(), kind))
            })
            .collect::<Result<Vec<_>, String>>()?;
        params.sort_by_key(|(index, _, _)| *index);
        let ret = module
            .nodes
            .iter()
            .filter(|node| module.node_lanes.get(&node.name) == Some(lane))
            .find_map(|node| {
                if node.op.instruction == "return_owned_struct" {
                    return Some(CpuCallScalarKind::I64);
                }
                let kind = cpu_call_scalar_kind_for_instruction(&node.op.instruction)?;
                node.op.instruction.starts_with("return_").then_some(kind)
            })
            .ok_or_else(|| {
                format!("helper lane `{function_name}` does not contain a typed cpu.return_*")
            })?;
        helper_signatures.insert(
            function_name,
            CpuHelperSignature {
                params: params.iter().map(|(_, _, kind)| *kind).collect(),
                ret,
            },
        );
    }

    for lane in &helper_lanes {
        let lane_nodes = order
            .iter()
            .filter(|name| module.node_lanes.get(*name) == Some(lane))
            .cloned()
            .collect::<Vec<_>>();
        let mut params = module
            .nodes
            .iter()
            .filter(|node| module.node_lanes.get(&node.name) == Some(lane))
            .filter_map(|node| {
                let kind = cpu_call_scalar_kind_for_instruction(&node.op.instruction)?;
                node.op
                    .instruction
                    .starts_with("param_")
                    .then_some((node, kind))
            })
            .map(|(node, kind)| {
                let index = node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "invalid {} index `{}`",
                        node.op.full_name(),
                        node.op.args[0]
                    )
                })?;
                Ok((index, node.name.clone(), kind))
            })
            .collect::<Result<Vec<_>, String>>()?;
        params.sort_by_key(|(index, _, _)| *index);
        let param_bindings = params
            .iter()
            .map(|(index, name, kind)| (name.clone(), cpu_param_binding(*kind, *index)))
            .collect::<BTreeMap<_, _>>();
        let param_buffer_lengths = params
            .iter()
            .filter(|(_, _, kind)| *kind == CpuCallScalarKind::BorrowedBuffer)
            .map(|(index, name, _)| (name.clone(), format!("%arg{index}_len")))
            .collect::<BTreeMap<_, _>>();
        let function_name = lane.trim_start_matches("fn:");
        let emitted = emit_cpu_function(
            module,
            &resources,
            &lane_nodes,
            &param_bindings,
            &param_buffer_lengths,
            &helper_signatures,
            helper_signatures
                .get(function_name)
                .expect("helper signature should exist")
                .ret,
            &mut global_counter,
        )?;
        globals.extend(emitted.globals);
        let args_sig = params
            .iter()
            .map(|(index, _, kind)| {
                if *kind == CpuCallScalarKind::BorrowedBuffer {
                    format!("ptr %arg{index}, i64 %arg{index}_len")
                } else {
                    format!("{} %arg{index}", cpu_scalar_kind_llvm_type(*kind))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let ret_sig = cpu_scalar_kind_llvm_type(
            helper_signatures
                .get(function_name)
                .expect("helper signature should exist")
                .ret,
        );
        helper_defs.push(format!(
            "define {ret_sig} @nuis_fn_{function_name}({args_sig}) {{\n{}\n}}\n",
            emitted.body
        ));
        if let Some(invoker) = render_scalar_task_invoker(
            function_name,
            helper_signatures
                .get(function_name)
                .expect("helper signature should exist"),
        ) {
            helper_defs.push(invoker);
        }
    }

    let entry_nodes = order
        .iter()
        .filter(|name| {
            module
                .node_lanes
                .get(*name)
                .is_none_or(|lane| !lane.starts_with("fn:"))
        })
        .cloned()
        .collect::<Vec<_>>();
    let entry_emitted = emit_cpu_function(
        module,
        &resources,
        &entry_nodes,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &helper_signatures,
        CpuCallScalarKind::I64,
        &mut global_counter,
    )?;
    globals.extend(entry_emitted.globals);
    let dynamic_extern_decls = render_dynamic_extern_decls(module);
    let dynamic_extern_block = (!dynamic_extern_decls.is_empty())
        .then(|| format!("{}\n\n", dynamic_extern_decls.join("\n")))
        .unwrap_or_default();
    Ok(format!(
        "; yir version: {}\n\
{}\n\
%cpu.node = type {{ i64, ptr }}\n\
declare ptr @malloc(i64)\ndeclare void @free(ptr)\ndeclare i32 @puts(ptr)\ndeclare i64 @nuis_host_text_lift(ptr)\ndeclare ptr @nuis_host_text_ptr(i64)\n\
declare void @nuis_debug_print_bool(i32)\ndeclare void @nuis_debug_print_i32(i32)\ndeclare void @nuis_debug_print_i64(i64)\ndeclare void @nuis_debug_print_f32(float)\ndeclare void @nuis_debug_print_f64(double)\n\n\
declare i64 @nuis_scheduler_task_spawn_i64_v1(i64)\ndeclare i64 @nuis_scheduler_task_spawn_invoker_i64_v1(ptr, ptr)\ndeclare void @nuis_scheduler_task_timeout_v1(i64, i64)\ndeclare void @nuis_scheduler_task_ready_after_v1(i64, i64)\ndeclare void @nuis_scheduler_task_cancel_v1(i64)\ndeclare i64 @nuis_scheduler_task_join_state_v1(i64)\ndeclare void @nuis_scheduler_task_require_completed_v1(i64)\ndeclare i64 @nuis_scheduler_task_value_i64_v1(i64)\n\
declare i64 @nuis_scheduler_task_spawn_owned_v1(ptr)\ndeclare i64 @nuis_scheduler_task_take_owned_v1(i64, ptr)\ndeclare void @nuis_scheduler_owned_payload_drop_v1(ptr)\ndeclare void @nuis_scheduler_payload_free_v1(ptr)\n\
declare ptr @nuis_scheduler_owned_blob_copy_v1(ptr, i64, i64)\ndeclare ptr @nuis_scheduler_owned_blob_copy_text_v1(i64, i64)\ndeclare i64 @nuis_scheduler_owned_blob_text_lift_v1(ptr)\ndeclare i64 @nuis_scheduler_owned_blob_len_v1(ptr)\ndeclare void @nuis_scheduler_owned_blob_drop_v1(ptr)\n\
declare ptr @nuis_scheduler_owned_aggregate_alloc_v1(i64)\ndeclare i64 @nuis_scheduler_owned_aggregate_set_scalar_v1(ptr, i64, i64)\ndeclare i64 @nuis_scheduler_owned_aggregate_set_blob_v1(ptr, i64, ptr)\n\
declare ptr @nuis_scheduler_owned_aggregate_finish_v1(ptr)\ndeclare void @nuis_scheduler_owned_aggregate_require_v1(ptr)\n\
declare i64 @nuis_scheduler_owned_aggregate_get_v1(ptr, i64)\ndeclare ptr @nuis_scheduler_owned_aggregate_take_blob_v1(ptr, i64)\ndeclare void @nuis_scheduler_owned_aggregate_drop_v1(ptr)\n\
declare i64 @nuis_scheduler_task_spawn_owned_invoker_v1(ptr, ptr, i64, i64, i64, ptr)\n\
declare i64 @host_color_bias(i64)\ndeclare i64 @host_speed_curve(i64)\ndeclare i64 @host_radius_curve(i64)\ndeclare i64 @host_mix_tick(i64, i64)\ndeclare i64 @host_text_handle(i64)\ndeclare i64 @host_text_len(i64)\ndeclare i64 @host_text_line_count(i64)\ndeclare i64 @host_text_word_count(i64)\ndeclare i64 @host_text_concat(i64, i64)\n\
declare i64 @host_argv_count()\ndeclare i64 @host_argv_at(i64)\ndeclare i64 @host_env_has(i64)\ndeclare i64 @host_env_get(i64)\ndeclare i64 @host_file_open(i64, i64)\ndeclare i64 @host_file_read(i64, i64, i64)\ndeclare i64 @host_file_write(i64, i64)\ndeclare i64 @host_file_close(i64)\n\
declare i64 @host_stdout_write(i64)\ndeclare i64 @host_stdout_flush()\ndeclare i64 @host_stderr_write(i64)\ndeclare i64 @host_stderr_flush()\ndeclare i64 @host_serialize_i64_into(i64, i64, i64)\ndeclare i64 @host_serialize_text_into(i64, i64, i64)\ndeclare i64 @host_deserialize_text_from(i64, i64, i64)\ndeclare i64 @host_monotonic_time_ns()\n\
declare i64 @HostRenderCurves__color_bias(i64)\ndeclare i64 @HostRenderCurves__speed_curve(i64)\ndeclare i64 @HostRenderCurves__radius_curve(i64)\ndeclare i64 @HostRenderCurves__mix_tick(i64, i64)\ndeclare i64 @HostMath__speed_curve(i64)\n\
{}\n\
{}\n\
define i64 @nuis_yir_entry() {{\n{}\n}}\n",
        module.version,
        globals.join("\n"),
        dynamic_extern_block,
        helper_defs.join("\n"),
        entry_emitted.body
    ))
}

fn render_scalar_task_invoker(
    function_name: &str,
    signature: &CpuHelperSignature,
) -> Option<String> {
    if !is_normalized_task_scalar(signature.ret)
        || signature
            .params
            .iter()
            .any(|kind| !is_normalized_task_scalar(*kind))
    {
        return None;
    }
    let mut body = Vec::new();
    let mut call_args = Vec::new();
    for (index, kind) in signature.params.iter().copied().enumerate() {
        let pointer = if index == 0 {
            "%context".to_owned()
        } else {
            let pointer = format!("%task_arg{index}_ptr");
            body.push(format!(
                "  {pointer} = getelementptr i8, ptr %context, i64 {}",
                index * 8
            ));
            pointer
        };
        let packed = if kind == CpuCallScalarKind::I64 {
            format!("%task_arg{index}")
        } else {
            format!("%task_arg{index}_packed")
        };
        body.push(format!("  {packed} = load i64, ptr {pointer}, align 8"));
        let argument = match kind {
            CpuCallScalarKind::Bool => {
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = trunc i64 {packed} to i1"));
                argument
            }
            CpuCallScalarKind::I32 => {
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = trunc i64 {packed} to i32"));
                argument
            }
            CpuCallScalarKind::I64 => packed,
            CpuCallScalarKind::F32 => {
                let bits = format!("%task_arg{index}_bits");
                body.push(format!("  {bits} = trunc i64 {packed} to i32"));
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = bitcast i32 {bits} to float"));
                argument
            }
            CpuCallScalarKind::F64 => {
                let argument = format!("%task_arg{index}");
                body.push(format!("  {argument} = bitcast i64 {packed} to double"));
                argument
            }
            CpuCallScalarKind::BorrowedBuffer => {
                unreachable!("borrowed buffers do not have task invokers")
            }
            CpuCallScalarKind::OwnedBytes => {
                unreachable!("direct owned Bytes params do not have scalar task invokers")
            }
        };
        call_args.push(format!("{} {argument}", cpu_scalar_kind_llvm_type(kind)));
    }
    let return_ty = cpu_scalar_kind_llvm_type(signature.ret);
    body.push(format!(
        "  %task_result = call {return_ty} @nuis_fn_{function_name}({})",
        call_args.join(", ")
    ));
    match signature.ret {
        CpuCallScalarKind::Bool => {
            body.push("  %task_result_packed = zext i1 %task_result to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::I32 => {
            body.push("  %task_result_packed = sext i32 %task_result to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::I64 => body.push("  ret i64 %task_result".to_owned()),
        CpuCallScalarKind::F32 => {
            body.push("  %task_result_bits = bitcast float %task_result to i32".to_owned());
            body.push("  %task_result_packed = zext i32 %task_result_bits to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::F64 => {
            body.push("  %task_result_packed = bitcast double %task_result to i64".to_owned());
            body.push("  ret i64 %task_result_packed".to_owned());
        }
        CpuCallScalarKind::BorrowedBuffer => {
            unreachable!("borrowed buffers cannot return from task invokers")
        }
        CpuCallScalarKind::OwnedBytes => {
            unreachable!("owned Bytes cannot return from scalar task invokers")
        }
    }
    Some(format!(
        "define i64 @nuis_task_invoker_{function_name}(ptr %context) {{\n{}\n}}\n",
        body.join("\n")
    ))
}

fn is_normalized_task_scalar(kind: CpuCallScalarKind) -> bool {
    matches!(
        kind,
        CpuCallScalarKind::Bool
            | CpuCallScalarKind::I32
            | CpuCallScalarKind::I64
            | CpuCallScalarKind::F32
            | CpuCallScalarKind::F64
    )
}

#[cfg(test)]
mod tests;
