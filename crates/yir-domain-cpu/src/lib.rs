use std::env;
use yir_core::{
    ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, StructValue,
    TaskLifecycleState, Value,
};

mod carry_payload;
mod loop_metadata;
mod runtime_helpers;

use carry_payload::*;
use loop_metadata::*;
use runtime_helpers::*;
pub struct CpuMod;

impl RegisteredMod for CpuMod {
    fn module_name(&self) -> &'static str {
        "cpu"
    }
    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        require_cpu_resource(node, resource)?;
        match node.op.instruction.as_str() {
            "text" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.text <name> <resource> <string>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid integer literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "project_profile_ref" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.project_profile_ref <name> <resource> <domain> <unit> <slot>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_bool" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const_bool <name> <resource> <true|false>`",
                        node.name
                    ));
                }
                match node.op.args[0].as_str() {
                    "true" | "false" => Ok(InstructionSemantics::pure(Vec::new())),
                    _ => Err(format!(
                        "node `{}` has invalid bool literal `{}`",
                        node.name, node.op.args[0]
                    )),
                }
            }
            "const_i32" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const_i32 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<i32>().map_err(|_| {
                    format!(
                        "node `{}` has invalid i32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_i64" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const_i64 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid i64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_f32" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const_f32 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<f32>().map_err(|_| {
                    format!(
                        "node `{}` has invalid f32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "const_f64" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.const_f64 <name> <resource> <value>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<f64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid f64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "struct" => {
                if node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects `cpu.struct <name> <resource> <type_name> [field=value]...`",
                        node.name
                    ));
                }
                for entry in &node.op.args[1..] {
                    let Some((field, value_name)) = entry.split_once('=') else {
                        return Err(format!(
                            "node `{}` has invalid struct field binding `{}`",
                            node.name, entry
                        ));
                    };
                    if field.trim().is_empty() || value_name.trim().is_empty() {
                        return Err(format!(
                            "node `{}` has empty struct field binding `{}`",
                            node.name, entry
                        ));
                    }
                }
                Ok(InstructionSemantics::pure(
                    node.op.args[1..]
                        .iter()
                        .filter_map(|entry| {
                            entry
                                .split_once('=')
                                .map(|(_, value)| value.trim())
                                .filter(|value| !cpu_struct_field_is_literal(value))
                                .map(str::to_owned)
                        })
                        .collect(),
                ))
            }
            "field" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.field <name> <resource> <struct> <field_name>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            "variant_is" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.variant_is <name> <resource> <value> <variant_name>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            "variant_field" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.variant_field <name> <resource> <value> <variant_name> <field_name>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            "null" => {
                if !node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects `cpu.null <name> <resource>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "borrow" | "borrow_end" | "move_ptr" | "neg" | "not" | "await" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <input>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "param_bool" | "param_i32" | "param_i64" | "param_f32" | "param_f64" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <index>`",
                        node.name, node.op.instruction
                    ));
                }
                node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid parameter index `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "call_bool" | "call_i32" | "call_i64" | "call_f32" | "call_f64" => {
                if node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <callee> [arg...]`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(
                    node.op.args.iter().skip(1).cloned().collect(),
                ))
            }
            "return_bool" | "return_i32" | "return_i64" | "return_f32" | "return_f64" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <value>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "async_call" => {
                if node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects `cpu.async_call <name> <resource> <callee> [arg...]`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(
                    node.op.args.iter().skip(1).cloned().collect(),
                ))
            }
            "spawn_task" | "spawn_thread" | "thread_spawn" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <callee> <result>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::effect(vec![node.op.args[1].clone()]))
            }
            "join" | "cancel" | "join_result" | "thread_join" | "thread_join_result"
            | "task_completed" | "task_timed_out" | "task_cancelled" | "task_value"
            | "mutex_lock" | "mutex_unlock" | "mutex_value" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <input>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "mutex_new" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.mutex_new <name> <resource> <value>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "timeout" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.timeout <name> <resource> <task> <limit>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "add" | "sub" | "mul" | "div" | "rem" | "eq" | "ne" | "lt" | "gt" | "le" | "ge"
            | "and" | "or" | "xor" | "shl" | "shr" | "add_i32" | "sub_i32" | "mul_i32"
            | "div_i32" | "add_f32" | "sub_f32" | "mul_f32" | "div_f32" | "add_f64" | "sub_f64"
            | "mul_f64" | "div_f64" | "eq_i32" | "lt_i32" | "gt_i32" | "eq_f32" | "lt_f32"
            | "gt_f32" | "eq_f64" | "lt_f64" | "gt_f64" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <lhs> <rhs>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "cast_bool_to_i64" | "cast_i32_to_i64" | "cast_i64_to_bool" | "cast_i64_to_i32"
            | "cast_i32_to_f32" | "cast_i32_to_f64" | "cast_f32_to_f64" | "cast_f64_to_f32" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <input>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "select" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.select <name> <resource> <cond> <then> <else>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "alloc_node" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.alloc_node <name> <resource> <value> <next_ptr>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "alloc_buffer" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.alloc_buffer <name> <resource> <len> <fill>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "load_value" | "load_next" | "is_null" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <ptr>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "buffer_len" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.buffer_len <name> <resource> <buffer_ptr>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "load_at" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.load_at <name> <resource> <buffer_ptr> <index>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "store_value" | "store_next" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <ptr> <value>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "store_at" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.store_at <name> <resource> <buffer_ptr> <index> <value>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "free" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.free <name> <resource> <ptr>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "madd" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.madd <name> <resource> <lhs> <rhs> <acc>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "target_config" => {
                if node.op.args.len() != 3 && node.op.args.len() != 5 {
                    return Err(format!(
                        "node `{}` expects `cpu.target_config <name> <resource> <arch> <abi> <vector_bits> [isa_family isa_features]`",
                        node.name
                    ));
                }

                node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vector width `{}`",
                        node.name, node.op.args[2]
                    )
                })?;
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "instantiate_unit" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.instantiate_unit <name> <resource> <domain> <unit>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "bind_core" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.bind_core <name> <resource> <core_index>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid core index `{}`",
                        node.name, node.op.args[0]
                    )
                })?;

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "window" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.window <name> <resource> <width> <height> <title>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid width `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid height `{}`",
                        node.name, node.op.args[1]
                    )
                })?;

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "extern_call_i64" | "extern_call_i32" => {
                if node.op.args.len() < 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <abi> <symbol> [args...]`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args[2..].to_vec()))
            }
            "input_i64" => {
                if node.op.args.len() != 2 && node.op.args.len() != 5 {
                    return Err(format!(
                        "node `{}` expects `cpu.input_i64 <name> <resource> <channel> <default> [<min> <max> <step>]`",
                        node.name
                    ));
                }

                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid default integer literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                if node.op.args.len() == 5 {
                    let min_value = node.op.args[2].parse::<i64>().map_err(|_| {
                        format!(
                            "node `{}` has invalid min integer literal `{}`",
                            node.name, node.op.args[2]
                        )
                    })?;
                    let max_value = node.op.args[3].parse::<i64>().map_err(|_| {
                        format!(
                            "node `{}` has invalid max integer literal `{}`",
                            node.name, node.op.args[3]
                        )
                    })?;
                    let step_value = node.op.args[4].parse::<i64>().map_err(|_| {
                        format!(
                            "node `{}` has invalid step integer literal `{}`",
                            node.name, node.op.args[4]
                        )
                    })?;
                    if min_value > max_value {
                        return Err(format!(
                            "node `{}` requires min <= max, got {} > {}",
                            node.name, min_value, max_value
                        ));
                    }
                    if step_value <= 0 {
                        return Err(format!(
                            "node `{}` requires positive step, got `{}`",
                            node.name, step_value
                        ));
                    }
                }

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "tick_i64" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.tick_i64 <name> <resource> <start> <step>`",
                        node.name
                    ));
                }

                node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid tick start literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid tick step literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "present_frame" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.present_frame <name> <resource> <frame>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "print" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.print <name> <resource> <input>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "guard_print" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.guard_print <name> <resource> <condition> <print>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "loop_while_i64" => {
                if node.op.args.len() != 5 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64 <name> <resource> <initial> <limit> <step> <cmp> <step_kind>`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[4].as_str() {
                    "add" | "sub" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop step kind `{}`",
                            node.name, other
                        ));
                    }
                }
                Ok(InstructionSemantics::effect(node.op.args[..3].to_vec()))
            }
            "loop_while_i64_chain" | "loop_while_scalar_chain" => {
                if node.op.args.len() < 7 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[4].as_str() {
                    "add" | "sub" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop step kind `{}`",
                            node.name, other
                        ));
                    }
                }
                let mut effect_args = node.op.args[..3].to_vec();
                let mut cursor = 5usize;
                let mut parsed_any_carry = false;
                while cursor < node.op.args.len() {
                    let Some(carry_initial) = node.op.args.get(cursor) else {
                        break;
                    };
                    let Some(carry_kind) = node.op.args.get(cursor + 1) else {
                        return Err(format!(
                            "node `{}` expects `cpu.loop_while_scalar_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <carry_kind>)+`",
                            node.name
                        ));
                    };
                    let Some(payload_len) = carry_source_payload_len(carry_kind) else {
                        return Err(format!(
                            "node `{}` has invalid carry kind `{}`",
                            node.name, carry_kind
                        ));
                    };
                    let payload_end = cursor + 2 + payload_len;
                    if payload_end > node.op.args.len() {
                        return Err(format!(
                            "node `{}` is missing carry payload for `{}`",
                            node.name, carry_kind
                        ));
                    }
                    effect_args.push(carry_initial.clone());
                    effect_args.extend(node.op.args[cursor + 2..payload_end].iter().cloned());
                    cursor = payload_end;
                    parsed_any_carry = true;
                }
                if !parsed_any_carry || cursor != node.op.args.len() {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(effect_args))
            }
            "loop_while_i64_async_chain" | "loop_while_scalar_async_chain" => {
                if node.op.args.len() < 6 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                let mut effect_args = node.op.args[..2].to_vec();
                let mut cursor = 4usize;
                let mut parsed_any_carry = false;
                while cursor < node.op.args.len() {
                    let Some(carry_initial) = node.op.args.get(cursor) else {
                        break;
                    };
                    let Some(carry_kind) = node.op.args.get(cursor + 1) else {
                        return Err(format!(
                            "node `{}` expects `cpu.loop_while_scalar_async_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <carry_kind>)+`",
                            node.name
                        ));
                    };
                    let Some(payload_len) = carry_source_payload_len(carry_kind) else {
                        return Err(format!(
                            "node `{}` has invalid carry kind `{}`",
                            node.name, carry_kind
                        ));
                    };
                    let payload_end = cursor + 2 + payload_len;
                    if payload_end > node.op.args.len() {
                        return Err(format!(
                            "node `{}` is missing carry payload for `{}`",
                            node.name, carry_kind
                        ));
                    }
                    effect_args.push(carry_initial.clone());
                    effect_args.extend(node.op.args[cursor + 2..payload_end].iter().cloned());
                    cursor = payload_end;
                    parsed_any_carry = true;
                }
                if !parsed_any_carry || cursor != node.op.args.len() {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(effect_args))
            }
            "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain" => {
                if node.op.args.len() < 6 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <condition_kind> <condition_rhs> <then_kind> <else_kind>)+`",
                        node.name
                    ));
                }
                validate_loop_compare_kind(&node.op.args[3], &node.name)?;
                let carries = parse_conditional_carries(&node.op.args, 4, &node.name, true)?;
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                for carry in &carries {
                    inputs.push(carry.initial.clone());
                    collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain" => {
                if node.op.args.len() < 7 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                        node.name
                    ));
                }
                validate_loop_compare_kind(&node.op.args[3], &node.name)?;
                validate_loop_step_kind(&node.op.args[4], &node.name)?;
                let carries = parse_conditional_carries(&node.op.args, 5, &node.name, true)?;
                let mut inputs = node.op.args[..3].to_vec();
                for carry in &carries {
                    inputs.push(carry.initial.clone());
                    collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain" => {
                if node.op.args.len() < 8 || !(node.op.args.len() - 8).is_multiple_of(2) {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_flow_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)*`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[4].as_str() {
                    "add" | "sub" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop step kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[5].as_str() {
                    "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
                    | "current_ge" => {}
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[7].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ));
                    }
                }
                for carry_kind in node.op.args[9..].iter().step_by(2) {
                    if carry_kind == "add_current" {
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("add_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
                        continue;
                    }
                    return Err(format!(
                        "node `{}` has invalid carry kind `{}`",
                        node.name, carry_kind
                    ));
                }
                let mut inputs = node.op.args[..3].to_vec();
                inputs.push(node.op.args[6].clone());
                for chunk in node.op.args[8..].chunks(2) {
                    inputs.push(chunk[0].clone());
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain" => {
                if node.op.args.len() < 7 || !(node.op.args.len() - 7).is_multiple_of(2) {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_flow_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)*`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[4].as_str() {
                    "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
                    | "current_ge" => {}
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[6].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ));
                    }
                }
                for carry_kind in node.op.args[8..].iter().step_by(2) {
                    if carry_kind == "add_current" {
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("add_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
                        continue;
                    }
                    return Err(format!(
                        "node `{}` has invalid carry kind `{}`",
                        node.name, carry_kind
                    ));
                }
                let mut inputs = vec![
                    node.op.args[0].clone(),
                    node.op.args[1].clone(),
                    node.op.args[5].clone(),
                ];
                for chunk in node.op.args[7..].chunks(2) {
                    inputs.push(chunk[0].clone());
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain" => {
                validate_loop_compare_kind(&node.op.args[3], &node.name)?;
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    4,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                if carry_start_index < node.op.args.len()
                    && !(node.op.args.len() - carry_start_index).is_multiple_of(5)
                {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_flow_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_flow_expr> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)*`",
                        node.name
                    ));
                }
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
                for carry in &carries {
                    inputs.push(carry.initial.clone());
                    collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain" => {
                if node.op.args.len() < 10 || !(node.op.args.len() - 8).is_multiple_of(2) {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_post_flow_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[4].as_str() {
                    "add" | "sub" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop step kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[5].as_str() {
                    "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
                    | "current_ge" => {}
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[7].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ));
                    }
                }
                for carry_kind in node.op.args[9..].iter().step_by(2) {
                    if carry_kind == "add_current" {
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("add_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
                        continue;
                    }
                    return Err(format!(
                        "node `{}` has invalid carry kind `{}`",
                        node.name, carry_kind
                    ));
                }
                let mut inputs = node.op.args[..3].to_vec();
                inputs.push(node.op.args[6].clone());
                for chunk in node.op.args[8..].chunks(2) {
                    inputs.push(chunk[0].clone());
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain" => {
                if node.op.args.len() < 9 || !(node.op.args.len() - 7).is_multiple_of(2) {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_post_flow_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)+`",
                        node.name
                    ));
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[4].as_str() {
                    "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
                    | "current_ge" => {}
                    other if other.starts_with("carry") && other.ends_with("_eq") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ne") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_lt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_le") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_gt") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other if other.starts_with("carry") && other.ends_with("_ge") => {
                        other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node.name, other
                            )
                        })?;
                    }
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control kind `{}`",
                            node.name, other
                        ));
                    }
                }
                match node.op.args[6].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ));
                    }
                }
                for carry_kind in node.op.args[8..].iter().step_by(2) {
                    if carry_kind == "add_current" {
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("add_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
                        continue;
                    }
                    return Err(format!(
                        "node `{}` has invalid carry kind `{}`",
                        node.name, carry_kind
                    ));
                }
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                inputs.push(node.op.args[5].clone());
                for chunk in node.op.args[7..].chunks(2) {
                    inputs.push(chunk[0].clone());
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_async_post_flow_cond_chain"
            | "loop_while_scalar_async_post_flow_cond_chain" => {
                validate_loop_compare_kind(&node.op.args[3], &node.name)?;
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    4,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
                for carry in &carries {
                    inputs.push(carry.initial.clone());
                    collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain" => {
                validate_loop_compare_kind(&node.op.args[3], &node.name)?;
                validate_loop_step_kind(&node.op.args[4], &node.name)?;
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    5,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
                let mut inputs = node.op.args[..3].to_vec();
                collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
                for carry in &carries {
                    inputs.push(carry.initial.clone());
                    collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain" => {
                validate_loop_compare_kind(&node.op.args[3], &node.name)?;
                validate_loop_step_kind(&node.op.args[4], &node.name)?;
                let (control_expr, carry_start_index) = parse_loop_flow_expr(
                    &node.op.args,
                    5,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                let carries =
                    parse_conditional_carries(&node.op.args, carry_start_index, &node.name, true)?;
                let mut inputs = node.op.args[..3].to_vec();
                collect_loop_flow_rhs_inputs(&control_expr, &mut inputs);
                for carry in &carries {
                    inputs.push(carry.initial.clone());
                    collect_loop_condition_rhs_inputs(&carry.condition, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.then_source, &mut inputs);
                    collect_carry_branch_source_inputs(&carry.else_source, &mut inputs);
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "guard_return" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.guard_return <name> <resource> <condition> <return>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "guard_print_return" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.guard_print_return <name> <resource> <condition> <print> <return>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "guard_host_call_return" => {
                if node.op.args.len() < 4 {
                    return Err(
                        "cpu.guard_host_call_return expects condition return call-chain".to_owned(),
                    );
                }
                let mut inputs = vec![node.op.args[0].clone()];
                match node.op.args.get(1).map(String::as_str) {
                    Some("value") => inputs.push(node.op.args[3].clone()),
                    Some("write_flush_exit_code") => {}
                    _ if node.op.args[2].parse::<usize>().is_ok() => {
                        inputs.push(node.op.args[1].clone())
                    }
                    _ => inputs.push(node.op.args[3].clone()),
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "branch_print_return" => {
                if node.op.args.len() != 5 {
                    return Err(format!(
                        "node `{}` expects `cpu.branch_print_return <name> <resource> <condition> <then_print> <then_return> <else_print> <else_return>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(node.op.args.clone()))
            }
            "branch_host_call_return" => {
                if node.op.args.len() < 9 {
                    return Err(
                        "cpu.branch_host_call_return expects condition then-chain else-chain"
                            .to_owned(),
                    );
                }
                Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
            }
            other => Err(format!("unknown cpu instruction `{other}`")),
        }
    }
    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        match node.op.instruction.as_str() {
            "text" => Ok(Value::Symbol(node.op.args[0].clone())),
            "const" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid integer literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "project_profile_ref" => resolve_project_profile_ref(node),
            "const_bool" => Ok(Value::Bool(match node.op.args[0].as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(format!(
                        "node `{}` has invalid bool literal `{}`",
                        node.name, node.op.args[0]
                    ))
                }
            })),
            "const_i32" => Ok(Value::I32(node.op.args[0].parse::<i32>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid i32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_i64" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid i64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_f32" => Ok(Value::F32(node.op.args[0].parse::<f32>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid f32 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "const_f64" => Ok(Value::F64(node.op.args[0].parse::<f64>().map_err(
                |_| {
                    format!(
                        "node `{}` has invalid f64 literal `{}`",
                        node.name, node.op.args[0]
                    )
                },
            )?)),
            "struct" => {
                let type_name = node.op.args[0].clone();
                let mut fields = Vec::with_capacity(node.op.args.len().saturating_sub(1));
                for entry in &node.op.args[1..] {
                    let Some((field, value_name)) = entry.split_once('=') else {
                        return Err(format!(
                            "node `{}` has invalid struct field binding `{}`",
                            node.name, entry
                        ));
                    };
                    let value = state.expect_value(value_name.trim())?.clone();
                    fields.push((field.trim().to_owned(), value));
                }
                Ok(Value::Struct(StructValue { type_name, fields }))
            }
            "field" => {
                let struct_value = state.expect_struct(&node.op.args[0])?;
                let field_name = &node.op.args[1];
                struct_value
                    .fields
                    .iter()
                    .find(|(name, _)| name == field_name)
                    .map(|(_, value)| value.clone())
                    .ok_or_else(|| {
                        format!(
                            "node `{}` reads missing field `{}` from `{}`",
                            node.name, field_name, node.op.args[0]
                        )
                    })
            }
            "variant_is" => {
                let value = state.expect_value(&node.op.args[0])?;
                Ok(Value::Bool(match value {
                    Value::Struct(struct_value) => struct_value.type_name == node.op.args[1],
                    Value::VariantUnion(union) => union.active_variant == node.op.args[1],
                    other => {
                        return Err(format!(
                            "node `{}` expects variant-shaped value from `{}`, got {}",
                            node.name, node.op.args[0], other
                        ))
                    }
                }))
            }
            "variant_field" => {
                let value = state.expect_value(&node.op.args[0])?;
                let variant_name = &node.op.args[1];
                let field_name = &node.op.args[2];
                let struct_value = match value {
                    Value::Struct(struct_value) if &struct_value.type_name == variant_name => {
                        struct_value
                    }
                    Value::Struct(struct_value) => {
                        return Err(format!(
                            "node `{}` expects variant `{}` from `{}`, got `{}`",
                            node.name, variant_name, node.op.args[0], struct_value.type_name
                        ))
                    }
                    Value::VariantUnion(union) => {
                        union.variants.get(variant_name).ok_or_else(|| {
                            format!(
                                "node `{}` reads missing variant `{}` from union `{}`",
                                node.name, variant_name, union.parent_type_name
                            )
                        })?
                    }
                    other => {
                        return Err(format!(
                            "node `{}` expects variant-shaped value from `{}`, got {}",
                            node.name, node.op.args[0], other
                        ))
                    }
                };
                struct_value
                    .fields
                    .iter()
                    .find(|(name, _)| name == field_name)
                    .map(|(_, value)| value.clone())
                    .ok_or_else(|| {
                        format!(
                            "node `{}` reads missing field `{}` from variant `{}`",
                            node.name, field_name, variant_name
                        )
                    })
            }
            "null" => Ok(Value::Pointer(None)),
            "borrow" | "move_ptr" => Ok(Value::Pointer(state.expect_pointer(&node.op.args[0])?)),
            "param_bool" => Ok(Value::Bool(false)),
            "param_i32" => Ok(Value::I32(0)),
            "param_i64" => Ok(Value::Int(0)),
            "param_f32" => Ok(Value::F32(0.0)),
            "param_f64" => Ok(Value::F64(0.0)),
            "call_bool" | "call_i32" | "call_i64" | "call_f32" | "call_f64" => {
                let callee = &node.op.args[0];
                let args = node.op.args[1..]
                    .iter()
                    .map(|arg| state.expect_value(arg).map(|value| value.to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.call_i64 @{} [{}] {}({})",
                        node.resource,
                        resource.kind.raw,
                        callee,
                        args.join(", ")
                    ),
                );
                match node.op.instruction.as_str() {
                    "call_bool" => Ok(Value::Bool(false)),
                    "call_i32" => Ok(Value::I32(0)),
                    "call_f32" => Ok(Value::F32(0.0)),
                    "call_f64" => Ok(Value::F64(0.0)),
                    _ => Ok(Value::Int(0)),
                }
            }
            "return_bool" => {
                let value = state.expect_bool(&node.op.args[0])?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.return_bool @{} [{}] {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::Bool(value))
            }
            "return_i32" => {
                let value = state.expect_i32(&node.op.args[0])?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.return_i32 @{} [{}] {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::I32(value))
            }
            "return_i64" => {
                let value = state.expect_int(&node.op.args[0])?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.return_i64 @{} [{}] {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::Int(value))
            }
            "return_f32" => {
                let value = state.expect_f32(&node.op.args[0])?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.return_f32 @{} [{}] {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::F32(value))
            }
            "return_f64" => {
                let value = state.expect_f64(&node.op.args[0])?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.return_f64 @{} [{}] {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::F64(value))
            }
            "async_call" => {
                let callee = &node.op.args[0];
                let args = node.op.args[1..]
                    .iter()
                    .map(|arg| state.expect_value(arg).map(|value| value.to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.async_call @{} [{}] {}({})",
                        node.resource,
                        resource.kind.raw,
                        callee,
                        args.join(", ")
                    ),
                );
                Ok(Value::Unit)
            }
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
            "spawn_task" => {
                let callee = &node.op.args[0];
                let result = state.expect_value(&node.op.args[1])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.spawn_task @{} [{}] {} => {}",
                        node.resource, resource.kind.raw, callee, node.name
                    ),
                );
                Ok(Value::Task(yir_core::TaskHandle {
                    label: format!("{callee}@{}", node.name),
                    result: Box::new(result),
                    limit: None,
                    state: TaskLifecycleState::Pending,
                }))
            }
            "spawn_thread" | "thread_spawn" => {
                let callee = &node.op.args[0];
                let result = state.expect_value(&node.op.args[1])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.{} @{} [{}] {} => {}",
                        node.op.instruction, node.resource, resource.kind.raw, callee, node.name
                    ),
                );
                Ok(Value::Thread(yir_core::ThreadHandle {
                    label: format!("{callee}@{}", node.name),
                    result: Box::new(result),
                    state: TaskLifecycleState::Pending,
                }))
            }
            "join" => {
                let task = state.expect_task(&node.op.args[0])?;
                let label = task.label.clone();
                let result = (*task.result).clone();
                let lifecycle = task_lifecycle_state(task);
                if lifecycle == TaskLifecycleState::Cancelled {
                    return Err(format!("task `{label}` was cancelled before join"));
                }
                if lifecycle == TaskLifecycleState::TimedOut {
                    return Err(format!("task `{label}` timed out before join"));
                }
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.join @{} [{}]: {}",
                        node.resource, resource.kind.raw, label
                    ),
                );
                Ok(result)
            }
            "thread_join" => {
                let thread = state.expect_thread(&node.op.args[0])?;
                let label = thread.label.clone();
                let result = (*thread.result).clone();
                let lifecycle = task_lifecycle_state_for_thread(thread);
                if lifecycle == TaskLifecycleState::Cancelled {
                    return Err(format!("thread `{label}` was cancelled before join"));
                }
                if lifecycle == TaskLifecycleState::TimedOut {
                    return Err(format!("thread `{label}` timed out before join"));
                }
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.thread_join @{} [{}]: {}",
                        node.resource, resource.kind.raw, label
                    ),
                );
                Ok(result)
            }
            "cancel" => {
                let task = state.expect_task(&node.op.args[0])?;
                let label = task.label.clone();
                let result = (*task.result).clone();
                let limit = task.limit;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.cancel @{} [{}]: {}",
                        node.resource, resource.kind.raw, label
                    ),
                );
                Ok(Value::Task(yir_core::TaskHandle {
                    label,
                    result: Box::new(result),
                    limit,
                    state: TaskLifecycleState::Cancelled,
                }))
            }
            "join_result" => {
                let task = state.expect_task(&node.op.args[0])?;
                let label = task.label.clone();
                let lifecycle = task_lifecycle_state(task);
                let result = if lifecycle == TaskLifecycleState::Completed {
                    Some(task.result.clone())
                } else {
                    None
                };
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.join_result @{} [{}]: {} => {}",
                        node.resource, resource.kind.raw, label, lifecycle
                    ),
                );
                Ok(Value::TaskResult(yir_core::TaskResultHandle {
                    label,
                    state: lifecycle,
                    result,
                }))
            }
            "thread_join_result" => {
                let thread = state.expect_thread(&node.op.args[0])?;
                let label = thread.label.clone();
                let lifecycle = task_lifecycle_state_for_thread(thread);
                let result = if lifecycle == TaskLifecycleState::Completed {
                    Some(thread.result.clone())
                } else {
                    None
                };
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.thread_join_result @{} [{}]: {} => {}",
                        node.resource, resource.kind.raw, label, lifecycle
                    ),
                );
                Ok(Value::TaskResult(yir_core::TaskResultHandle {
                    label,
                    state: lifecycle,
                    result,
                }))
            }
            "task_completed" => {
                let result = state.expect_task_result(&node.op.args[0])?;
                Ok(Value::Bool(result.state == TaskLifecycleState::Completed))
            }
            "task_timed_out" => {
                let result = state.expect_task_result(&node.op.args[0])?;
                Ok(Value::Bool(result.state == TaskLifecycleState::TimedOut))
            }
            "task_cancelled" => {
                let result = state.expect_task_result(&node.op.args[0])?;
                Ok(Value::Bool(result.state == TaskLifecycleState::Cancelled))
            }
            "task_value" => {
                let result = state.expect_task_result(&node.op.args[0])?;
                result.result.as_deref().cloned().ok_or_else(|| {
                    format!(
                        "task result `{}` has no value in state `{}`",
                        result.label, result.state
                    )
                })
            }
            "timeout" => {
                let task = state.expect_task(&node.op.args[0])?;
                let label = task.label.clone();
                let result = (*task.result).clone();
                let limit = state.expect_int(&node.op.args[1])?;
                let lifecycle = task_lifecycle_state(task);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.timeout @{} [{}]: {} <= {}",
                        node.resource, resource.kind.raw, label, limit
                    ),
                );
                Ok(Value::Task(yir_core::TaskHandle {
                    label,
                    result: Box::new(result),
                    limit: Some(limit),
                    state: lifecycle,
                }))
            }
            "mutex_new" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.mutex_new @{} [{}]: {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::Mutex(yir_core::MutexHandle {
                    label: node.name.clone(),
                    value: Box::new(value),
                }))
            }
            "mutex_lock" => {
                let mutex = state.expect_mutex(&node.op.args[0])?;
                let label = mutex.label.clone();
                let value = mutex.value.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.mutex_lock @{} [{}]: {}",
                        node.resource, resource.kind.raw, label
                    ),
                );
                Ok(Value::MutexGuard(yir_core::MutexGuardHandle {
                    label,
                    value,
                }))
            }
            "mutex_unlock" => {
                let guard = state.expect_mutex_guard(&node.op.args[0])?;
                let label = guard.label.clone();
                let value = guard.value.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.mutex_unlock @{} [{}]: {}",
                        node.resource, resource.kind.raw, label
                    ),
                );
                Ok(Value::Mutex(yir_core::MutexHandle { label, value }))
            }
            "mutex_value" => {
                let guard = state.expect_mutex_guard(&node.op.args[0])?;
                let label = guard.label.clone();
                let value = (*guard.value).clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.mutex_value @{} [{}]: {}",
                        node.resource, resource.kind.raw, label
                    ),
                );
                Ok(value)
            }
            "await" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.await @{} [{}]: {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(value)
            }
            "borrow_end" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.borrow_end @{} [{}] ptr={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Unit)
            }
            "neg" => Ok(Value::Int(-state.expect_int(&node.op.args[0])?)),
            "not" => Ok(Value::Int(!state.expect_int(&node.op.args[0])?)),
            "add" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int(lhs + rhs))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::F32(lhs + rhs))
                } else {
                    Ok(Value::F64(
                        state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
                    ))
                }
            }
            "add_i32" => Ok(Value::I32(
                state.expect_i32(&node.op.args[0])? + state.expect_i32(&node.op.args[1])?,
            )),
            "add_f32" => Ok(Value::F32(
                state.expect_f32(&node.op.args[0])? + state.expect_f32(&node.op.args[1])?,
            )),
            "add_f64" => Ok(Value::F64(
                state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
            )),
            "sub" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int(lhs - rhs))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::F32(lhs - rhs))
                } else {
                    Ok(Value::F64(
                        state.expect_f64(&node.op.args[0])? - state.expect_f64(&node.op.args[1])?,
                    ))
                }
            }
            "sub_i32" => Ok(Value::I32(
                state.expect_i32(&node.op.args[0])? - state.expect_i32(&node.op.args[1])?,
            )),
            "sub_f32" => Ok(Value::F32(
                state.expect_f32(&node.op.args[0])? - state.expect_f32(&node.op.args[1])?,
            )),
            "sub_f64" => Ok(Value::F64(
                state.expect_f64(&node.op.args[0])? - state.expect_f64(&node.op.args[1])?,
            )),
            "mul" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int(lhs * rhs))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::F32(lhs * rhs))
                } else {
                    Ok(Value::F64(
                        state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
                    ))
                }
            }
            "mul_i32" => Ok(Value::I32(
                state.expect_i32(&node.op.args[0])? * state.expect_i32(&node.op.args[1])?,
            )),
            "mul_f32" => Ok(Value::F32(
                state.expect_f32(&node.op.args[0])? * state.expect_f32(&node.op.args[1])?,
            )),
            "mul_f64" => Ok(Value::F64(
                state.expect_f64(&node.op.args[0])? * state.expect_f64(&node.op.args[1])?,
            )),
            "div" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    if rhs == 0 {
                        return Err(format!("node `{}` divides by zero", node.name));
                    }
                    Ok(Value::Int(lhs / rhs))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    if rhs == 0.0 {
                        return Err(format!("node `{}` divides by zero", node.name));
                    }
                    Ok(Value::F32(lhs / rhs))
                } else {
                    let lhs = state.expect_f64(&node.op.args[0])?;
                    let rhs = state.expect_f64(&node.op.args[1])?;
                    if rhs == 0.0 {
                        return Err(format!("node `{}` divides by zero", node.name));
                    }
                    Ok(Value::F64(lhs / rhs))
                }
            }
            "div_i32" => {
                let lhs = state.expect_i32(&node.op.args[0])?;
                let rhs = state.expect_i32(&node.op.args[1])?;
                if rhs == 0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::I32(lhs / rhs))
            }
            "div_f32" => {
                let lhs = state.expect_f32(&node.op.args[0])?;
                let rhs = state.expect_f32(&node.op.args[1])?;
                if rhs == 0.0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::F32(lhs / rhs))
            }
            "div_f64" => {
                let lhs = state.expect_f64(&node.op.args[0])?;
                let rhs = state.expect_f64(&node.op.args[1])?;
                if rhs == 0.0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::F64(lhs / rhs))
            }
            "rem" => {
                let lhs = state.expect_int(&node.op.args[0])?;
                let rhs = state.expect_int(&node.op.args[1])?;
                if rhs == 0 {
                    return Err(format!("node `{}` computes remainder by zero", node.name));
                }
                Ok(Value::Int(lhs % rhs))
            }
            "eq" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs == rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_i32(&node.op.args[0]),
                    state.expect_i32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs == rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs == rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f64(&node.op.args[0]),
                    state.expect_f64(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs == rhs) as i64))
                } else {
                    Ok(Value::Int(
                        (state.expect_bool(&node.op.args[0])?
                            == state.expect_bool(&node.op.args[1])?) as i64,
                    ))
                }
            }
            "eq_i32" => Ok(Value::Bool(
                state.expect_i32(&node.op.args[0])? == state.expect_i32(&node.op.args[1])?,
            )),
            "eq_f32" => Ok(Value::Bool(
                state.expect_f32(&node.op.args[0])? == state.expect_f32(&node.op.args[1])?,
            )),
            "eq_f64" => Ok(Value::Bool(
                state.expect_f64(&node.op.args[0])? == state.expect_f64(&node.op.args[1])?,
            )),
            "ne" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs != rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_i32(&node.op.args[0]),
                    state.expect_i32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs != rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs != rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f64(&node.op.args[0]),
                    state.expect_f64(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs != rhs) as i64))
                } else {
                    Ok(Value::Int(
                        (state.expect_bool(&node.op.args[0])?
                            != state.expect_bool(&node.op.args[1])?) as i64,
                    ))
                }
            }
            "lt" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs < rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs < rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f64(&node.op.args[0]),
                    state.expect_f64(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs < rhs) as i64))
                } else {
                    Ok(Value::Int(
                        (state.expect_i32(&node.op.args[0])?
                            < state.expect_i32(&node.op.args[1])?) as i64,
                    ))
                }
            }
            "lt_i32" => Ok(Value::Bool(
                state.expect_i32(&node.op.args[0])? < state.expect_i32(&node.op.args[1])?,
            )),
            "lt_f32" => Ok(Value::Bool(
                state.expect_f32(&node.op.args[0])? < state.expect_f32(&node.op.args[1])?,
            )),
            "lt_f64" => Ok(Value::Bool(
                state.expect_f64(&node.op.args[0])? < state.expect_f64(&node.op.args[1])?,
            )),
            "gt" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs > rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs > rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f64(&node.op.args[0]),
                    state.expect_f64(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs > rhs) as i64))
                } else {
                    Ok(Value::Int(
                        (state.expect_i32(&node.op.args[0])?
                            > state.expect_i32(&node.op.args[1])?) as i64,
                    ))
                }
            }
            "gt_i32" => Ok(Value::Bool(
                state.expect_i32(&node.op.args[0])? > state.expect_i32(&node.op.args[1])?,
            )),
            "gt_f32" => Ok(Value::Bool(
                state.expect_f32(&node.op.args[0])? > state.expect_f32(&node.op.args[1])?,
            )),
            "gt_f64" => Ok(Value::Bool(
                state.expect_f64(&node.op.args[0])? > state.expect_f64(&node.op.args[1])?,
            )),
            "le" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs <= rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs <= rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f64(&node.op.args[0]),
                    state.expect_f64(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs <= rhs) as i64))
                } else {
                    Ok(Value::Int(
                        (state.expect_i32(&node.op.args[0])?
                            <= state.expect_i32(&node.op.args[1])?) as i64,
                    ))
                }
            }
            "ge" => {
                if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_int(&node.op.args[0]),
                    state.expect_int(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs >= rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f32(&node.op.args[0]),
                    state.expect_f32(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs >= rhs) as i64))
                } else if let (Ok(lhs), Ok(rhs)) = (
                    state.expect_f64(&node.op.args[0]),
                    state.expect_f64(&node.op.args[1]),
                ) {
                    Ok(Value::Int((lhs >= rhs) as i64))
                } else {
                    Ok(Value::Int(
                        (state.expect_i32(&node.op.args[0])?
                            >= state.expect_i32(&node.op.args[1])?) as i64,
                    ))
                }
            }
            "and" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? & state.expect_int(&node.op.args[1])?,
            )),
            "or" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? | state.expect_int(&node.op.args[1])?,
            )),
            "xor" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? ^ state.expect_int(&node.op.args[1])?,
            )),
            "shl" => {
                let lhs = state.expect_int(&node.op.args[0])?;
                let rhs = state.expect_int(&node.op.args[1])?;
                if rhs < 0 {
                    return Err(format!("node `{}` shifts by negative amount", node.name));
                }
                Ok(Value::Int(lhs.wrapping_shl(rhs as u32)))
            }
            "shr" => {
                let lhs = state.expect_int(&node.op.args[0])?;
                let rhs = state.expect_int(&node.op.args[1])?;
                if rhs < 0 {
                    return Err(format!("node `{}` shifts by negative amount", node.name));
                }
                Ok(Value::Int(lhs >> rhs))
            }
            "madd" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?
                    + state.expect_int(&node.op.args[2])?,
            )),
            "select" => {
                let cond = match state.expect_value(&node.op.args[0])? {
                    Value::Bool(value) => *value,
                    Value::Int(value) => *value != 0,
                    other => {
                        return Err(format!(
                            "node `{}` expects bool or i64 select condition, got {}",
                            node.name, other
                        ))
                    }
                };
                let then_value = state.expect_value(&node.op.args[1])?;
                let else_value = state.expect_value(&node.op.args[2])?;
                if let Some(union) = select_variant_union(cond, then_value, else_value) {
                    return Ok(Value::VariantUnion(union));
                }
                Ok(if cond {
                    then_value.clone()
                } else {
                    else_value.clone()
                })
            }
            "cast_bool_to_i64" => Ok(Value::Int(if state.expect_bool(&node.op.args[0])? {
                1
            } else {
                0
            })),
            "cast_i32_to_i64" => Ok(Value::Int(state.expect_i32(&node.op.args[0])? as i64)),
            "cast_i64_to_bool" => Ok(Value::Bool(state.expect_int(&node.op.args[0])? != 0)),
            "cast_i64_to_i32" => Ok(Value::I32(state.expect_int(&node.op.args[0])? as i32)),
            "cast_i32_to_f32" => Ok(Value::F32(state.expect_i32(&node.op.args[0])? as f32)),
            "cast_i32_to_f64" => Ok(Value::F64(state.expect_i32(&node.op.args[0])? as f64)),
            "cast_f32_to_f64" => Ok(Value::F64(state.expect_f32(&node.op.args[0])? as f64)),
            "cast_f64_to_f32" => Ok(Value::F32(state.expect_f64(&node.op.args[0])? as f32)),
            "alloc_node" => {
                let value = state.expect_int(&node.op.args[0])?;
                let next = state.expect_pointer(&node.op.args[1])?;
                let address = state.alloc_heap_node(value, next);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.alloc_node @{} [{}] -> &{} value={} next={}",
                        node.resource,
                        resource.kind.raw,
                        address,
                        value,
                        next.map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Pointer(Some(address)))
            }
            "alloc_buffer" => {
                let len = state.expect_int(&node.op.args[0])?;
                if len < 0 {
                    return Err(format!(
                        "node `{}` allocates negative buffer length",
                        node.name
                    ));
                }
                let fill = state.expect_int(&node.op.args[1])?;
                let address = state.alloc_heap_buffer(len as usize, fill);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.alloc_buffer @{} [{}] -> &{} len={} fill={}",
                        node.resource, resource.kind.raw, address, len, fill
                    ),
                );
                Ok(Value::Pointer(Some(address)))
            }
            "load_value" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let node_value = state.read_heap_node(pointer)?.value;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.load_value @{} [{}] ptr={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Int(node_value))
            }
            "load_next" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let next = state.read_heap_node(pointer)?.next;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.load_next @{} [{}] ptr={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Pointer(next))
            }
            "buffer_len" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let len = state.heap_buffer_len(pointer)? as i64;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.buffer_len @{} [{}] ptr={} len={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned()),
                        len
                    ),
                );
                Ok(Value::Int(len))
            }
            "load_at" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let index = state.expect_int(&node.op.args[1])?;
                if index < 0 {
                    return Err(format!("node `{}` reads negative buffer index", node.name));
                }
                let value = state.read_heap_buffer_at(pointer, index as usize)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.load_at @{} [{}] ptr={} index={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned()),
                        index
                    ),
                );
                Ok(Value::Int(value))
            }
            "is_null" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                Ok(Value::Int(if pointer.is_none() { 1 } else { 0 }))
            }
            "store_value" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let value = state.expect_int(&node.op.args[1])?;
                state.write_heap_value(pointer, value)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.store_value @{} [{}] ptr={} value={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned()),
                        value
                    ),
                );
                Ok(Value::Unit)
            }
            "store_next" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let next = state.expect_pointer(&node.op.args[1])?;
                state.write_heap_next(pointer, next)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.store_next @{} [{}] ptr={} next={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned()),
                        next.map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Unit)
            }
            "store_at" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                let index = state.expect_int(&node.op.args[1])?;
                if index < 0 {
                    return Err(format!("node `{}` writes negative buffer index", node.name));
                }
                let value = state.expect_int(&node.op.args[2])?;
                state.write_heap_buffer_at(pointer, index as usize, value)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.store_at @{} [{}] ptr={} index={} value={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned()),
                        index,
                        value
                    ),
                );
                Ok(Value::Unit)
            }
            "free" => {
                let pointer = state.expect_pointer(&node.op.args[0])?;
                state.free_heap_node(pointer)?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.free @{} [{}] ptr={}",
                        node.resource,
                        resource.kind.raw,
                        pointer
                            .map(|ptr| format!("&{ptr}"))
                            .unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Unit)
            }
            "target_config" => {
                let arch = node.op.args[0].clone();
                let abi = node.op.args[1].clone();
                let vector_bits = node.op.args[2].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid vector width `{}`",
                        node.name, node.op.args[2]
                    )
                })?;
                let mut values = vec![
                    Value::Symbol(arch),
                    Value::Symbol(abi),
                    Value::Int(vector_bits),
                ];
                if node.op.args.len() >= 5 {
                    values.push(Value::Symbol(node.op.args[3].clone()));
                    values.push(Value::Symbol(node.op.args[4].clone()));
                }
                Ok(Value::Tuple(values))
            }
            "instantiate_unit" => {
                let domain = node.op.args[0].clone();
                let unit = node.op.args[1].clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.instantiate_unit @{} [{}] {}::{}",
                        node.resource, resource.kind.raw, domain, unit
                    ),
                );
                Ok(Value::Struct(StructValue {
                    type_name: "UnitInstance".to_owned(),
                    fields: vec![
                        ("domain".to_owned(), Value::Symbol(domain)),
                        ("unit".to_owned(), Value::Symbol(unit)),
                    ],
                }))
            }
            "bind_core" => {
                let core_index = node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid core index `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.bind_core @{} [{}] core={}",
                        node.resource, resource.kind.raw, core_index
                    ),
                );
                Ok(Value::Unit)
            }
            "window" => {
                let width = node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid width `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                let height = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid height `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                let title = &node.op.args[2];
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.window @{} [{}] {}x{} title={}",
                        node.resource, resource.kind.raw, width, height, title
                    ),
                );
                Ok(Value::Tuple(vec![Value::Int(width), Value::Int(height)]))
            }
            "extern_call_i64" => {
                let abi = &node.op.args[0];
                let symbol = &node.op.args[1];
                let args = node.op.args[2..]
                    .iter()
                    .map(|arg| state.expect_int(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let value = execute_extern_i64(abi, symbol, &args).map_err(|message| {
                    format!(
                        "node `{}` extern call `{symbol}` failed: {message}",
                        node.name
                    )
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.extern_call_i64 @{} [{}] {}::{}({}) -> {}",
                        node.resource,
                        resource.kind.raw,
                        abi,
                        symbol,
                        args.iter()
                            .map(|value| value.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        value
                    ),
                );
                Ok(Value::Int(value))
            }
            "extern_call_i32" => {
                let abi = &node.op.args[0];
                let symbol = &node.op.args[1];
                let args = node.op.args[2..]
                    .iter()
                    .map(|arg| state.expect_int(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let value = execute_extern_i32(abi, symbol, &args).map_err(|message| {
                    format!(
                        "node `{}` extern call `{symbol}` failed: {message}",
                        node.name
                    )
                })?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.extern_call_i32 @{} [{}] {}::{}({}) -> {}",
                        node.resource,
                        resource.kind.raw,
                        abi,
                        symbol,
                        args.iter()
                            .map(|value| value.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        value
                    ),
                );
                Ok(Value::I32(value))
            }
            "input_i64" => {
                let channel = &node.op.args[0];
                let default_value = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid default integer literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                let (min_value, max_value, step_value) = if node.op.args.len() == 5 {
                    let min_value = node.op.args[2].parse::<i64>().map_err(|_| {
                        format!(
                            "node `{}` has invalid min integer literal `{}`",
                            node.name, node.op.args[2]
                        )
                    })?;
                    let max_value = node.op.args[3].parse::<i64>().map_err(|_| {
                        format!(
                            "node `{}` has invalid max integer literal `{}`",
                            node.name, node.op.args[3]
                        )
                    })?;
                    let step_value = node.op.args[4].parse::<i64>().map_err(|_| {
                        format!(
                            "node `{}` has invalid step integer literal `{}`",
                            node.name, node.op.args[4]
                        )
                    })?;
                    (min_value, max_value, step_value)
                } else {
                    (0, 255, 1)
                };
                let env_name = format!("NUIS_UI_{}", normalize_channel(channel));
                let raw_sampled = env::var(&env_name)
                    .ok()
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(default_value);
                let clamped = raw_sampled.clamp(min_value, max_value);
                let snapped = min_value + ((clamped - min_value) / step_value) * step_value;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.input_i64 @{} [{}] channel={} value={} source={} range=[{},{}] step={}",
                        node.resource,
                        resource.kind.raw,
                        channel,
                        snapped,
                        if raw_sampled == default_value {
                            "default"
                        } else {
                            "env"
                        },
                        min_value,
                        max_value,
                        step_value
                    ),
                );
                Ok(Value::Int(snapped))
            }
            "tick_i64" => {
                let start = node.op.args[0].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid tick start literal `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                let step = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid tick step literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                let tick_index = env::var("NUIS_TICK")
                    .ok()
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(0);
                let value = start + tick_index * step;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.tick_i64 @{} [{}] tick={} start={} step={} value={}",
                        node.resource, resource.kind.raw, tick_index, start, step, value
                    ),
                );
                Ok(Value::Int(value))
            }
            "present_frame" => {
                let frame =
                    unwrap_present_frame_payload(state.expect_value(&node.op.args[0])?.clone());
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.present_frame @{} [{}]: {}",
                        node.resource, resource.kind.raw, frame
                    ),
                );
                Ok(Value::Unit)
            }
            "print" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.print @{} [{}]: {}",
                        node.resource, resource.kind.raw, value
                    ),
                );
                Ok(Value::Unit)
            }
            "guard_print" => {
                let condition = state.expect_value(&node.op.args[0])?.clone();
                let printed = state.expect_value(&node.op.args[1])?.clone();
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.guard_print @{} [{}]: if {} then print {}",
                        node.resource, resource.kind.raw, condition, printed
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
}

#[cfg(test)]
mod tests;
