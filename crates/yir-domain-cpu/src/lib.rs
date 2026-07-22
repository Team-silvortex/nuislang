use yir_core::{
    BranchEffectAction, BranchEffectActionCapability, ExecutionState, InstructionSemantics, Node,
    RegisteredMod, Resource, Value,
};

mod branch_effect;
mod carry_payload;
mod describe;
mod describe_async;
mod describe_basic;
mod describe_host;
mod describe_loops_control;
mod describe_post_control;
mod describe_scalar_memory;
mod execute_host;
mod execute_memory;
mod execute_scalar;
mod execute_tasks;
mod execute_values;
mod loop_metadata;
mod runtime_helpers;

use branch_effect::{execute_cpu_branch_effect_action, CPU_BRANCH_EFFECT_ACTIONS};
use carry_payload::*;
use describe::describe_cpu_node;
use execute_host::execute_cpu_host_node;
use execute_memory::execute_cpu_memory_node;
use execute_scalar::execute_cpu_scalar_node;
use execute_tasks::execute_cpu_task_node;
use execute_values::execute_cpu_value_node;
use loop_metadata::*;
pub use loop_metadata::{
    parse_conditional_carries, LoopCondExpr, ParsedCarryBranchSource, ParsedConditionalCarry,
};
use runtime_helpers::*;
pub struct CpuMod;

impl RegisteredMod for CpuMod {
    fn module_name(&self) -> &'static str {
        "cpu"
    }
    fn branch_effect_action_capabilities(&self) -> &'static [BranchEffectActionCapability] {
        CPU_BRANCH_EFFECT_ACTIONS
    }
    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        describe_cpu_node(node, resource)
    }
    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if let Some(value) = execute_cpu_task_node(node, resource, state)? {
            return Ok(value);
        }
        if let Some(value) = execute_cpu_memory_node(node, resource, state)? {
            return Ok(value);
        }
        if let Some(value) = execute_cpu_scalar_node(node, state)? {
            return Ok(value);
        }
        if let Some(value) = execute_cpu_value_node(node, resource, state)? {
            return Ok(value);
        }
        if let Some(value) = execute_cpu_host_node(node, resource, state)? {
            return Ok(value);
        }

        match node.op.instruction.as_str() {
            "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    4,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?
                        .iter()
                        .map(|carry| {
                            format_conditional_carry(carry, &|value_name| {
                                state
                                    .expect_value(value_name)
                                    .map(|value| value.to_string())
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                let control_display = format_loop_flow_expr(&control_expr, &|value_name| {
                    state
                        .expect_value(value_name)
                        .map(|value| value.to_string())
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step await {}(current), {}, carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_callee,
                        control_display,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64 @{} [{}]: start {}, loop while current {} {}, step {} {}",
                        node.resource, resource.kind.raw, initial, cmp, limit, step_kind, step
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_chain" | "loop_while_scalar_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let carries = node.op.args[5..]
                    .chunks(2)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        Ok(format!("{}:{}", initial, chunk[1]))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step {} {}, carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_kind,
                        step,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_async_chain" | "loop_while_scalar_async_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let carries = node.op.args[4..]
                    .chunks(2)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        Ok(format!("{}:{}", initial, chunk[1]))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step await {}(current), carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_callee,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let carries = parse_conditional_carries(&node.op.args, 4, &node.name, true)?
                    .iter()
                    .map(|carry| {
                        format_conditional_carry(carry, &|value_name| {
                            state
                                .expect_value(value_name)
                                .map(|value| value.to_string())
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step await {}(current), carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_callee,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let carries = parse_conditional_carries(&node.op.args, 5, &node.name, true)?
                    .iter()
                    .map(|carry| {
                        format_conditional_carry(carry, &|value_name| {
                            state
                                .expect_value(value_name)
                                .map(|value| value.to_string())
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step {} {}, carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_kind,
                        step,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(5).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[6])?.to_string();
                let control_action = node.op.args.get(7).map_or("<missing>", String::as_str);
                let carries = node.op.args[8..]
                    .chunks(2)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        Ok(format!("{}:{}", initial, chunk[1]))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step {} {}, if {} {} then {}, carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_kind,
                        step,
                        control_kind,
                        control_rhs,
                        control_action,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[5])?.to_string();
                let control_action = node.op.args.get(6).map_or("<missing>", String::as_str);
                let carries = node.op.args[7..]
                    .chunks(2)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        Ok(format!("{}:{}", initial, chunk[1]))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step await {}(current), if {} {} then {}, carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_callee,
                        control_kind,
                        control_rhs,
                        control_action,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(5).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[6])?.to_string();
                let control_action = node.op.args.get(7).map_or("<missing>", String::as_str);
                let carries = node.op.args[8..]
                    .chunks(2)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        Ok(format!("{}:{}", initial, chunk[1]))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step {} {}, update carries {}, then if {} {} {},",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_kind,
                        step,
                        carries.join(", "),
                        control_kind,
                        control_rhs,
                        control_action,
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[5])?.to_string();
                let control_action = node.op.args.get(6).map_or("<missing>", String::as_str);
                let carries = node.op.args[7..]
                    .chunks(2)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        Ok(format!("{}:{}", initial, chunk[1]))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step await {}(current), update carries {}, then if {} {} {},",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_callee,
                        carries.join(", "),
                        control_kind,
                        control_rhs,
                        control_action,
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_async_post_flow_cond_chain"
            | "loop_while_scalar_async_post_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    4,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?
                        .iter()
                        .map(|carry| {
                            format_conditional_carry(carry, &|value_name| {
                                state
                                    .expect_value(value_name)
                                    .map(|value| value.to_string())
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                let control_display = format_loop_flow_expr(&control_expr, &|value_name| {
                    state
                        .expect_value(value_name)
                        .map(|value| value.to_string())
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step await {}(current), update carries {}, then {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_callee,
                        carries.join(", "),
                        control_display
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    5,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?
                        .iter()
                        .map(|carry| {
                            format_conditional_carry(carry, &|value_name| {
                                state
                                    .expect_value(value_name)
                                    .map(|value| value.to_string())
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                let control_display = format_loop_flow_expr(&control_expr, &|value_name| {
                    state
                        .expect_value(value_name)
                        .map(|value| value.to_string())
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step {} {}, update carries {}, then {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_kind,
                        step,
                        carries.join(", "),
                        control_display,
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    5,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?
                        .iter()
                        .map(|carry| {
                            format_conditional_carry(carry, &|value_name| {
                                state
                                    .expect_value(value_name)
                                    .map(|value| value.to_string())
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                let control_display = format_loop_flow_expr(&control_expr, &|value_name| {
                    state
                        .expect_value(value_name)
                        .map(|value| value.to_string())
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}]: start {}, loop while current {} {}, step {} {}, {}, carries {}",
                        node.op.instruction,
                        node.resource,
                        resource.kind.raw,
                        initial,
                        cmp,
                        limit,
                        step_kind,
                        step,
                        control_display,
                        carries.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
            "guard_return" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let returned = state.expect_value(&node.op.args[1])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.guard_return @{} [{}]: if {} then return {}",
                        node.resource, resource.kind.raw, condition, returned
                    ),
                );
                Ok(Value::Unit)
            }
            "guard_drop_owned_bytes_return" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let bytes = state.expect_value(&node.op.args[1])?.clone();
                let returned = state.expect_value(&node.op.args[2])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.guard_drop_owned_bytes_return @{} [{}]: if {} then drop {} and return {}",
                        node.resource, resource.kind.raw, condition, bytes, returned
                    ),
                );
                Ok(Value::Unit)
            }
            "branch_drop_owned_bytes_return" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let then_bytes = state.expect_value(&node.op.args[1])?.clone();
                let then_returned = state.expect_value(&node.op.args[2])?.clone();
                let else_bytes = state.expect_value(&node.op.args[3])?.clone();
                let else_returned = state.expect_value(&node.op.args[4])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.branch_drop_owned_bytes_return @{} [{}]: if {} then drop {} and return {} else drop {} and return {}",
                        node.resource,
                        resource.kind.raw,
                        condition,
                        then_bytes,
                        then_returned,
                        else_bytes,
                        else_returned
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_owned_bytes_copy_drop_break" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let buffer = state.expect_value(&node.op.args[1])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_owned_bytes_copy_drop_break @{} [{}]: while {} copy/drop {} then break",
                        node.resource, resource.kind.raw, condition, buffer
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_effect" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let action_args = node.op.args[8..]
                    .iter()
                    .map(|name| state.expect_value(name).cloned())
                    .collect::<Result<Vec<_>, _>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64_effect @{} [{}]: {} {} {}, step {} {}, action {}.{}({:?})",
                        node.resource,
                        resource.kind.raw,
                        initial,
                        node.op.args[3],
                        limit,
                        node.op.args[4],
                        step,
                        node.op.args[5],
                        node.op.args[6],
                        action_args
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_effect_flow" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let control_count = node.op.args[5]
                    .parse::<usize>()
                    .map_err(|_| format!("node `{}` has invalid control count", node.name))?;
                let control_end = 6 + control_count;
                let control = node.op.args[6..control_end].join(" ");
                let carry_count = node.op.args[control_end]
                    .parse::<usize>()
                    .map_err(|_| format!("node `{}` has invalid carry count", node.name))?;
                let mut action_offset = control_end + 1;
                let mut carries = Vec::with_capacity(carry_count);
                for _ in 0..carry_count {
                    carries.push(state.expect_value(&node.op.args[action_offset])?.clone());
                    let kind = &node.op.args[action_offset + 1];
                    action_offset += 2 + carry_source_payload_len(kind).ok_or_else(|| {
                        format!("node `{}` has invalid carry kind `{kind}`", node.name)
                    })?;
                }
                let buffer = state
                    .expect_value(&node.op.args[action_offset + 3])?
                    .clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64_effect_flow @{} [{}]: {} {} {}, then {} {}, control [{}], carries {:?}, action cpu.owned_bytes_copy_drop({})",
                        node.resource,
                        resource.kind.raw,
                        initial,
                        node.op.args[3],
                        limit,
                        node.op.args[4],
                        step,
                        control,
                        carries,
                        buffer
                    ),
                );
                Ok(Value::Unit)
            }
            "guard_print_return" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let printed = state.expect_value(&node.op.args[1])?.clone();
                let returned = state.expect_value(&node.op.args[2])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.guard_print_return @{} [{}]: if {} then print {} and return {}",
                        node.resource, resource.kind.raw, condition, printed, returned
                    ),
                );
                Ok(Value::Unit)
            }
            "guard_host_call_return" => {
                state.push_resource_event(resource, "effect cpu.guard_host_call_return".to_owned());
                Ok(Value::Unit)
            }
            "branch_host_call_return" => {
                state
                    .push_resource_event(resource, "effect cpu.branch_host_call_return".to_owned());
                Ok(Value::Unit)
            }
            "branch_print_return" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let then_printed = state.expect_value(&node.op.args[1])?.clone();
                let then_returned = state.expect_value(&node.op.args[2])?.clone();
                let else_printed = state.expect_value(&node.op.args[3])?.clone();
                let else_returned = state.expect_value(&node.op.args[4])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.branch_print_return @{} [{}]: if {} then print {} and return {} else print {} and return {}",
                        node.resource,
                        resource.kind.raw,
                        condition,
                        then_printed,
                        then_returned,
                        else_printed,
                        else_returned
                    ),
                );
                Ok(Value::Unit)
            }
            other => Err(format!("unknown cpu instruction `{other}`")),
        }
    }

    fn execute_branch_effect_action(
        &self,
        action: &BranchEffectAction<'_>,
        parent: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        execute_cpu_branch_effect_action(action, parent, resource, state)
    }
}

#[cfg(test)]
mod tests;
