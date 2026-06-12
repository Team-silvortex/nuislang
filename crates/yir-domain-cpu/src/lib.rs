use std::env;

use yir_core::{
    DataHandleTable, DataMarker, ExecutionState, InstructionSemantics, Node, RegisteredMod,
    RenderPipeline, Resource, StructValue, SurfaceTarget, TaskLifecycleState, Value, Viewport,
};

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
                if node.op.args.len() < 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.struct <name> <resource> <type_name> <field=value>...`",
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
            "spawn_task" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.spawn_task <name> <resource> <callee> <result>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(vec![node.op.args[1].clone()]))
            }
            "join" | "cancel" | "join_result" | "task_completed" | "task_timed_out"
            | "task_cancelled" | "task_value" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <task>`",
                        node.name, node.op.instruction
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
            "cast_i32_to_i64" | "cast_i64_to_i32" | "cast_i32_to_f32" | "cast_i32_to_f64"
            | "cast_f32_to_f64" | "cast_f64_to_f32" => {
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
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `cpu.target_config <name> <resource> <arch> <abi> <vector_bits>`",
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
            "extern_call_i64" => {
                if node.op.args.len() < 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.extern_call_i64 <name> <resource> <abi> <symbol> [args...]`",
                        node.name
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
                if node.op.args.len() < 7 || (node.op.args.len() - 5) % 2 != 0 {
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
                for carry_kind in node.op.args[6..].iter().step_by(2) {
                    if carry_kind == "add_current"
                        || carry_kind == "add_prev_current"
                        || carry_kind == "mul_current"
                        || carry_kind == "mul_prev_current"
                    {
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("add_prev_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("mul_prev_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
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
                    if let Some(index) = carry_kind.strip_prefix("mul_carry") {
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
                Ok(InstructionSemantics::effect(
                    node.op
                        .args
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index < 3 || (*index >= 5 && index % 2 == 1))
                        .map(|(_, arg)| arg.clone())
                        .collect(),
                ))
            }
            "loop_while_i64_async_chain" | "loop_while_scalar_async_chain" => {
                if node.op.args.len() < 6 || (node.op.args.len() - 4) % 2 != 0 {
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
                for carry_kind in node.op.args[5..].iter().step_by(2) {
                    if carry_kind == "add_current"
                        || carry_kind == "add_prev_current"
                        || carry_kind == "mul_current"
                        || carry_kind == "mul_prev_current"
                    {
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("add_prev_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
                        continue;
                    }
                    if let Some(index) = carry_kind.strip_prefix("mul_prev_carry") {
                        index.parse::<usize>().map_err(|_| {
                            format!(
                                "node `{}` has invalid carry kind `{}`",
                                node.name, carry_kind
                            )
                        })?;
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
                    if let Some(index) = carry_kind.strip_prefix("mul_carry") {
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
                Ok(InstructionSemantics::effect(
                    node.op
                        .args
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index < 2 || (*index >= 4 && index % 2 == 0))
                        .map(|(_, arg)| arg.clone())
                        .collect(),
                ))
            }
            "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain" => {
                if node.op.args.len() < 9 || (node.op.args.len() - 4) % 5 != 0 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_async_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> (<carry_initial> <condition_kind> <condition_rhs> <then_kind> <else_kind>)+`",
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
                for chunk in node.op.args[4..].chunks(5) {
                    let cond_kind = &chunk[1];
                    match cond_kind.as_str() {
                        "always" | "current_eq" | "current_ne" | "current_lt" | "current_le"
                        | "current_gt" | "current_ge" | "prev_current_eq" | "prev_current_ne"
                        | "prev_current_lt" | "prev_current_le" | "prev_current_gt"
                        | "prev_current_ge" => {}
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_eq") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_ne") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_lt") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_gt") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_le") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_ge") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_eq") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_ne") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_lt") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_gt") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_le") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_ge") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ => {
                            return Err(format!(
                                "node `{}` has invalid conditional carry kind `{}`",
                                node.name, cond_kind
                            ));
                        }
                    }
                    for carry_kind in [&chunk[3], &chunk[4]] {
                        if carry_kind == "keep"
                            || carry_kind == "add_current"
                            || carry_kind == "add_prev_current"
                            || carry_kind == "mul_current"
                            || carry_kind == "mul_prev_current"
                        {
                            continue;
                        }
                        if let Some(index) = carry_kind.strip_prefix("add_prev_carry") {
                            index.parse::<usize>().map_err(|_| {
                                format!(
                                    "node `{}` has invalid carry kind `{}`",
                                    node.name, carry_kind
                                )
                            })?;
                            continue;
                        }
                        if let Some(index) = carry_kind.strip_prefix("mul_prev_carry") {
                            index.parse::<usize>().map_err(|_| {
                                format!(
                                    "node `{}` has invalid carry kind `{}`",
                                    node.name, carry_kind
                                )
                            })?;
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
                        if let Some(index) = carry_kind.strip_prefix("mul_carry") {
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
                }
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                for chunk in node.op.args[4..].chunks(5) {
                    inputs.push(chunk[0].clone());
                    if chunk[1] != "always" {
                        inputs.push(chunk[2].clone());
                    }
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain" => {
                if node.op.args.len() < 10 || (node.op.args.len() - 5) % 5 != 0 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_scalar_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
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
                for chunk in node.op.args[5..].chunks(5) {
                    let cond_kind = &chunk[1];
                    match cond_kind.as_str() {
                        "always" | "current_eq" | "current_ne" | "current_lt" | "current_le"
                        | "current_gt" | "current_ge" | "prev_current_eq" | "prev_current_ne"
                        | "prev_current_lt" | "prev_current_le" | "prev_current_gt"
                        | "prev_current_ge" => {}
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_eq") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_ne") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_lt") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_gt") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_le") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("prev_carry") && cond_kind.ends_with("_ge") => {
                            cond_kind[10..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_eq") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_ne") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_lt") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_gt") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_le") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_ge") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ => {
                            return Err(format!(
                                "node `{}` has invalid conditional carry kind `{}`",
                                node.name, cond_kind
                            ));
                        }
                    }
                    for carry_kind in [&chunk[3], &chunk[4]] {
                        if carry_kind == "keep"
                            || carry_kind == "add_current"
                            || carry_kind == "add_prev_current"
                            || carry_kind == "mul_current"
                            || carry_kind == "mul_prev_current"
                        {
                            continue;
                        }
                        if let Some(index) = carry_kind.strip_prefix("add_prev_carry") {
                            index.parse::<usize>().map_err(|_| {
                                format!(
                                    "node `{}` has invalid carry kind `{}`",
                                    node.name, carry_kind
                                )
                            })?;
                            continue;
                        }
                        if let Some(index) = carry_kind.strip_prefix("mul_prev_carry") {
                            index.parse::<usize>().map_err(|_| {
                                format!(
                                    "node `{}` has invalid carry kind `{}`",
                                    node.name, carry_kind
                                )
                            })?;
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
                        if let Some(index) = carry_kind.strip_prefix("mul_carry") {
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
                }
                let mut inputs = node.op.args[..3].to_vec();
                for chunk in node.op.args[5..].chunks(5) {
                    inputs.push(chunk[0].clone());
                    if chunk[1] != "always" {
                        inputs.push(chunk[2].clone());
                    }
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_flow_chain" => {
                if node.op.args.len() < 8 || (node.op.args.len() - 8) % 2 != 0 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64_flow_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)*`",
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
            "loop_while_i64_async_flow_chain" => {
                if node.op.args.len() < 7 || (node.op.args.len() - 7) % 2 != 0 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64_async_flow_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)*`",
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
            "loop_while_i64_async_flow_cond_chain" => {
                let validate_flow_control_kind =
                    |kind: &str, node_name: &str| -> Result<(), String> {
                        match kind {
                            "current_eq" | "current_ne" | "current_lt" | "current_le"
                            | "current_gt" | "current_ge" => Ok(()),
                            other
                                if other.starts_with("carry")
                                    && ["_eq", "_ne", "_lt", "_le", "_gt", "_ge"]
                                        .iter()
                                        .any(|suffix| other.ends_with(suffix)) =>
                            {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid flow control kind `{}`",
                                        node_name, other
                                    )
                                })?;
                                Ok(())
                            }
                            other => Err(format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node_name, other
                            )),
                        }
                    };
                fn parse_flow_control_expr<F>(
                    args: &[String],
                    start: usize,
                    node_name: &str,
                    validate_flow_control_kind: &F,
                ) -> Result<(Vec<String>, usize), String>
                where
                    F: Fn(&str, &str) -> Result<(), String>,
                {
                    let Some(token) = args.get(start).map(String::as_str) else {
                        return Err(format!(
                            "node `{}` is missing flow control metadata",
                            node_name
                        ));
                    };
                    if token == "and" || token == "or" {
                        let (mut lhs_inputs, after_lhs) = parse_flow_control_expr(
                            args,
                            start + 1,
                            node_name,
                            validate_flow_control_kind,
                        )?;
                        let (rhs_inputs, after_rhs) = parse_flow_control_expr(
                            args,
                            after_lhs,
                            node_name,
                            validate_flow_control_kind,
                        )?;
                        lhs_inputs.extend(rhs_inputs);
                        Ok((lhs_inputs, after_rhs))
                    } else {
                        validate_flow_control_kind(token, node_name)?;
                        let Some(rhs) = args.get(start + 1) else {
                            return Err(format!(
                                "node `{}` is missing flow control rhs",
                                node_name
                            ));
                        };
                        Ok((vec![rhs.clone()], start + 2))
                    }
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ))
                    }
                }
                let (control_rhs_inputs, control_action_index) = parse_flow_control_expr(
                    &node.op.args,
                    4,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                if control_action_index >= node.op.args.len() {
                    return Err(format!(
                        "node `{}` is missing flow control action",
                        node.name
                    ));
                }
                let carry_start_index = control_action_index + 1;
                if node.op.args.len() < carry_start_index + 5
                    || (node.op.args.len() - carry_start_index) % 5 != 0
                {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64_async_flow_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_expr> <control_action> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                        node.name
                    ));
                }
                match node.op.args[control_action_index].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ))
                    }
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let cond_kind = &chunk[1];
                    match cond_kind.as_str() {
                        "always" | "current_eq" | "current_ne" | "current_lt" | "current_le"
                        | "current_gt" | "current_ge" | "prev_current_eq" | "prev_current_ne"
                        | "prev_current_lt" | "prev_current_le" | "prev_current_gt"
                        | "prev_current_ge" => {}
                        _ if cond_kind.starts_with("prev_carry")
                            || cond_kind.starts_with("carry") => {}
                        _ => {
                            return Err(format!(
                                "node `{}` has invalid conditional carry kind `{}`",
                                node.name, cond_kind
                            ))
                        }
                    }
                }
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                for rhs in &control_rhs_inputs {
                    inputs.push(rhs.clone());
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    inputs.push(chunk[0].clone());
                    if chunk[1] != "always" {
                        inputs.push(chunk[2].clone());
                    }
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_post_flow_chain" => {
                if node.op.args.len() < 10 || (node.op.args.len() - 8) % 2 != 0 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64_post_flow_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)+`",
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
            "loop_while_i64_async_post_flow_chain" => {
                if node.op.args.len() < 9 || (node.op.args.len() - 7) % 2 != 0 {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64_async_post_flow_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_kind> <control_rhs> <control_action> (<carry_initial> <carry_kind>)+`",
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
            "loop_while_i64_async_post_flow_cond_chain" => {
                let validate_flow_control_kind =
                    |kind: &str, node_name: &str| -> Result<(), String> {
                        match kind {
                            "current_eq" | "current_ne" | "current_lt" | "current_le"
                            | "current_gt" | "current_ge" => Ok(()),
                            other
                                if other.starts_with("carry")
                                    && ["_eq", "_ne", "_lt", "_le", "_gt", "_ge"]
                                        .iter()
                                        .any(|suffix| other.ends_with(suffix)) =>
                            {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid flow control kind `{}`",
                                        node_name, other
                                    )
                                })?;
                                Ok(())
                            }
                            other => Err(format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node_name, other
                            )),
                        }
                    };
                fn parse_flow_control_expr<F>(
                    args: &[String],
                    start: usize,
                    node_name: &str,
                    validate_flow_control_kind: &F,
                ) -> Result<(Vec<String>, usize), String>
                where
                    F: Fn(&str, &str) -> Result<(), String>,
                {
                    let Some(token) = args.get(start).map(String::as_str) else {
                        return Err(format!(
                            "node `{}` is missing flow control metadata",
                            node_name
                        ));
                    };
                    if token == "and" || token == "or" {
                        let (mut lhs_inputs, after_lhs) = parse_flow_control_expr(
                            args,
                            start + 1,
                            node_name,
                            validate_flow_control_kind,
                        )?;
                        let (rhs_inputs, after_rhs) = parse_flow_control_expr(
                            args,
                            after_lhs,
                            node_name,
                            validate_flow_control_kind,
                        )?;
                        lhs_inputs.extend(rhs_inputs);
                        Ok((lhs_inputs, after_rhs))
                    } else {
                        validate_flow_control_kind(token, node_name)?;
                        let Some(rhs) = args.get(start + 1) else {
                            return Err(format!(
                                "node `{}` is missing flow control rhs",
                                node_name
                            ));
                        };
                        Ok((vec![rhs.clone()], start + 2))
                    }
                }
                match node.op.args[3].as_str() {
                    "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid loop compare kind `{}`",
                            node.name, other
                        ))
                    }
                }
                let (control_rhs_inputs, control_action_index) = parse_flow_control_expr(
                    &node.op.args,
                    4,
                    &node.name,
                    &validate_flow_control_kind,
                )?;
                if control_action_index >= node.op.args.len() {
                    return Err(format!(
                        "node `{}` is missing flow control action",
                        node.name
                    ));
                }
                let carry_start_index = control_action_index + 1;
                if node.op.args.len() < carry_start_index + 5
                    || (node.op.args.len() - carry_start_index) % 5 != 0
                {
                    return Err(format!(
                        "node `{}` expects `cpu.loop_while_i64_async_post_flow_cond_chain <name> <resource> <initial> <limit> <step_callee> <cmp> <control_expr> <control_action> (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                        node.name
                    ));
                }
                match node.op.args[control_action_index].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ))
                    }
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let cond_kind = &chunk[1];
                    match cond_kind.as_str() {
                        "always" | "current_eq" | "current_ne" | "current_lt" | "current_le"
                        | "current_gt" | "current_ge" | "prev_current_eq" | "prev_current_ne"
                        | "prev_current_lt" | "prev_current_le" | "prev_current_gt"
                        | "prev_current_ge" => {}
                        _ if cond_kind.starts_with("prev_carry")
                            || cond_kind.starts_with("carry") => {}
                        _ => {
                            return Err(format!(
                                "node `{}` has invalid conditional carry kind `{}`",
                                node.name, cond_kind
                            ))
                        }
                    }
                }
                let mut inputs = vec![node.op.args[0].clone(), node.op.args[1].clone()];
                for rhs in &control_rhs_inputs {
                    inputs.push(rhs.clone());
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    inputs.push(chunk[0].clone());
                    if chunk[1] != "always" {
                        inputs.push(chunk[2].clone());
                    }
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_post_flow_cond_chain" => {
                let validate_flow_control_kind =
                    |kind: &str, node_name: &str| -> Result<(), String> {
                        match kind {
                            "current_eq" | "current_ne" | "current_lt" | "current_le"
                            | "current_gt" | "current_ge" => Ok(()),
                            other
                                if other.starts_with("carry")
                                    && ["_eq", "_ne", "_lt", "_le", "_gt", "_ge"]
                                        .iter()
                                        .any(|suffix| other.ends_with(suffix)) =>
                            {
                                let suffix_len = 3;
                                other[5..other.len() - suffix_len]
                                    .parse::<usize>()
                                    .map_err(|_| {
                                        format!(
                                            "node `{}` has invalid flow control kind `{}`",
                                            node_name, other
                                        )
                                    })?;
                                Ok(())
                            }
                            other => Err(format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node_name, other
                            )),
                        }
                    };
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
                let (control_rhs_inputs, control_action_index, carry_start_index) = match node
                    .op
                    .args
                    .get(5)
                    .map(String::as_str)
                {
                    Some("and") | Some("or") => {
                        if node.op.args.len() < 16 || (node.op.args.len() - 11) % 5 != 0 {
                            return Err(format!(
                                "node `{}` expects `cpu.loop_while_i64_post_flow_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<control_kind> <control_rhs> <control_action> | <and|or> <control_kind> <control_rhs> <control_kind> <control_rhs> <control_action>) (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                                node.name
                            ));
                        }
                        validate_flow_control_kind(&node.op.args[6], &node.name)?;
                        validate_flow_control_kind(&node.op.args[8], &node.name)?;
                        (
                            vec![node.op.args[7].clone(), node.op.args[9].clone()],
                            10,
                            11,
                        )
                    }
                    Some(kind) => {
                        if node.op.args.len() < 13 || (node.op.args.len() - 8) % 5 != 0 {
                            return Err(format!(
                                "node `{}` expects `cpu.loop_while_i64_post_flow_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<control_kind> <control_rhs> <control_action> | <and|or> <control_kind> <control_rhs> <control_kind> <control_rhs> <control_action>) (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                                node.name
                            ));
                        }
                        validate_flow_control_kind(kind, &node.name)?;
                        (vec![node.op.args[6].clone()], 7, 8)
                    }
                    None => {
                        return Err(format!(
                            "node `{}` is missing flow control arguments",
                            node.name
                        ));
                    }
                };
                match node.op.args[control_action_index].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ));
                    }
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let cond_kind = chunk[1].as_str();
                    if cond_kind != "always" {
                        match cond_kind {
                            "current_eq" | "current_ne" | "current_lt" | "current_le"
                            | "current_gt" | "current_ge" => {}
                            other if other.starts_with("carry") && other.ends_with("_eq") => {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, other
                                    )
                                })?;
                            }
                            other if other.starts_with("carry") && other.ends_with("_ne") => {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, other
                                    )
                                })?;
                            }
                            other if other.starts_with("carry") && other.ends_with("_lt") => {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, other
                                    )
                                })?;
                            }
                            other if other.starts_with("carry") && other.ends_with("_le") => {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, other
                                    )
                                })?;
                            }
                            other if other.starts_with("carry") && other.ends_with("_gt") => {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, other
                                    )
                                })?;
                            }
                            other if other.starts_with("carry") && other.ends_with("_ge") => {
                                other[5..other.len() - 3].parse::<usize>().map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, other
                                    )
                                })?;
                            }
                            other => {
                                return Err(format!(
                                    "node `{}` has invalid conditional carry kind `{}`",
                                    node.name, other
                                ));
                            }
                        }
                    }
                    for carry_kind in [&chunk[3], &chunk[4]] {
                        if carry_kind == "keep" || carry_kind == "add_current" {
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
                }
                let mut inputs = node.op.args[..3].to_vec();
                for control_rhs_input in &control_rhs_inputs {
                    inputs.push(control_rhs_input.clone());
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    inputs.push(chunk[0].clone());
                    if chunk[1] != "always" {
                        inputs.push(chunk[2].clone());
                    }
                }
                Ok(InstructionSemantics::effect(inputs))
            }
            "loop_while_i64_flow_cond_chain" => {
                let validate_flow_control_kind =
                    |kind: &str, node_name: &str| -> Result<(), String> {
                        match kind {
                            "current_eq" | "current_ne" | "current_lt" | "current_le"
                            | "current_gt" | "current_ge" => Ok(()),
                            other
                                if other.starts_with("carry")
                                    && ["_eq", "_ne", "_lt", "_le", "_gt", "_ge"]
                                        .iter()
                                        .any(|suffix| other.ends_with(suffix)) =>
                            {
                                let suffix_len = 3;
                                other[5..other.len() - suffix_len]
                                    .parse::<usize>()
                                    .map_err(|_| {
                                        format!(
                                            "node `{}` has invalid flow control kind `{}`",
                                            node_name, other
                                        )
                                    })?;
                                Ok(())
                            }
                            other => Err(format!(
                                "node `{}` has invalid flow control kind `{}`",
                                node_name, other
                            )),
                        }
                    };
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
                let (control_rhs_inputs, control_action_index, carry_start_index) = match node
                    .op
                    .args
                    .get(5)
                    .map(String::as_str)
                {
                    Some("and") | Some("or") => {
                        if node.op.args.len() < 16 || (node.op.args.len() - 11) % 5 != 0 {
                            return Err(format!(
                                    "node `{}` expects `cpu.loop_while_i64_flow_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<control_kind> <control_rhs> <control_action> | <and|or> <control_kind> <control_rhs> <control_kind> <control_rhs> <control_action>) (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                                    node.name
                                ));
                        }
                        validate_flow_control_kind(&node.op.args[6], &node.name)?;
                        validate_flow_control_kind(&node.op.args[8], &node.name)?;
                        (
                            vec![node.op.args[7].clone(), node.op.args[9].clone()],
                            10,
                            11,
                        )
                    }
                    Some(kind) => {
                        if node.op.args.len() < 13 || (node.op.args.len() - 8) % 5 != 0 {
                            return Err(format!(
                                    "node `{}` expects `cpu.loop_while_i64_flow_cond_chain <name> <resource> <initial> <limit> <step> <cmp> <step_kind> (<control_kind> <control_rhs> <control_action> | <and|or> <control_kind> <control_rhs> <control_kind> <control_rhs> <control_action>) (<carry_initial> <cond_kind> <cond_rhs> <then_kind> <else_kind>)+`",
                                    node.name
                                ));
                        }
                        validate_flow_control_kind(kind, &node.name)?;
                        (vec![node.op.args[6].clone()], 7, 8)
                    }
                    None => {
                        return Err(format!(
                            "node `{}` is missing flow control arguments",
                            node.name
                        ));
                    }
                };
                match node.op.args[control_action_index].as_str() {
                    "break" | "continue" => {}
                    other => {
                        return Err(format!(
                            "node `{}` has invalid flow control action `{}`",
                            node.name, other
                        ));
                    }
                }
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    let cond_kind = &chunk[1];
                    match cond_kind.as_str() {
                        "always" | "current_eq" | "current_ne" | "current_lt" | "current_le"
                        | "current_gt" | "current_ge" => {}
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_eq") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_ne") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_lt") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_le") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ if cond_kind.starts_with("carry") && cond_kind.ends_with("_gt") => {
                            cond_kind[5..cond_kind.len() - 3]
                                .parse::<usize>()
                                .map_err(|_| {
                                    format!(
                                        "node `{}` has invalid conditional carry kind `{}`",
                                        node.name, cond_kind
                                    )
                                })?;
                        }
                        _ => {
                            return Err(format!(
                                "node `{}` has invalid conditional carry kind `{}`",
                                node.name, cond_kind
                            ));
                        }
                    }
                    for carry_kind in [&chunk[3], &chunk[4]] {
                        if carry_kind == "keep" || carry_kind == "add_current" {
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
                }
                let mut inputs = node.op.args[..3].to_vec();
                inputs.extend(control_rhs_inputs);
                for chunk in node.op.args[carry_start_index..].chunks(5) {
                    inputs.push(chunk[0].clone());
                    if chunk[1] != "always" {
                        inputs.push(chunk[2].clone());
                    }
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
            "branch_print_return" => {
                if node.op.args.len() != 5 {
                    return Err(format!(
                        "node `{}` expects `cpu.branch_print_return <name> <resource> <condition> <then_print> <then_return> <else_print> <else_return>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(node.op.args.clone()))
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
            "loop_while_i64_async_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[5])?.to_string();
                let control_action = node.op.args.get(6).map_or("<missing>", String::as_str);
                let carries = node.op.args[7..]
                    .chunks(5)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        let rhs = if chunk[1] == "always" {
                            "<always>".to_owned()
                        } else {
                            state.expect_value(&chunk[2])?.to_string()
                        };
                        Ok(format!(
                            "{}:{} ? {} : {} @ {}",
                            initial, chunk[1], chunk[3], chunk[4], rhs
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64_async_flow_cond_chain @{} [{}]: start {}, loop while current {} {}, step await {}(current), if {} {} then {}, carries {}",
                        node.resource, resource.kind.raw, initial, cmp, limit, step_callee, control_kind, control_rhs, control_action, carries.join(", ")
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
                let cond = state.expect_int(&node.op.args[0])?;
                let then_value = state.expect_int(&node.op.args[1])?;
                let else_value = state.expect_int(&node.op.args[2])?;
                Ok(Value::Int(if cond != 0 { then_value } else { else_value }))
            }
            "cast_i32_to_i64" => Ok(Value::Int(state.expect_i32(&node.op.args[0])? as i64)),
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
                Ok(Value::Tuple(vec![
                    Value::Symbol(arch),
                    Value::Symbol(abi),
                    Value::Int(vector_bits),
                ]))
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
                let carries = node.op.args[4..]
                    .chunks(5)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        let rhs = if chunk[1] == "always" {
                            "<always>".to_owned()
                        } else {
                            state.expect_value(&chunk[2])?.to_string()
                        };
                        Ok(format!(
                            "{}:{} ? {} : {} @ {}",
                            initial, chunk[1], chunk[3], chunk[4], rhs
                        ))
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
                let carries = node.op.args[5..]
                    .chunks(5)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        let rhs = if chunk[1] == "always" {
                            "<always>".to_owned()
                        } else {
                            state.expect_value(&chunk[2])?.to_string()
                        };
                        Ok(format!(
                            "{}:{} ? {} : {} @ {}",
                            initial, chunk[1], chunk[3], chunk[4], rhs
                        ))
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
            "loop_while_i64_flow_chain" => {
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
                        "effect cpu.loop_while_i64_flow_chain @{} [{}]: start {}, loop while current {} {}, step {} {}, if {} {} then {}, carries {}",
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
            "loop_while_i64_async_flow_chain" => {
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
                        "effect cpu.loop_while_i64_async_flow_chain @{} [{}]: start {}, loop while current {} {}, step await {}(current), if {} {} then {}, carries {}",
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
            "loop_while_i64_post_flow_chain" => {
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
                        "effect cpu.loop_while_i64_post_flow_chain @{} [{}]: start {}, loop while current {} {}, step {} {}, update carries {}, then if {} {} {},",
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
            "loop_while_i64_async_post_flow_chain" => {
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
                        "effect cpu.loop_while_i64_async_post_flow_chain @{} [{}]: start {}, loop while current {} {}, step await {}(current), update carries {}, then if {} {} {},",
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
            "loop_while_i64_async_post_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step_callee = node.op.args.get(2).map_or("<missing>", String::as_str);
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[5])?.to_string();
                let control_action = node.op.args.get(6).map_or("<missing>", String::as_str);
                let carries = node.op.args[7..]
                    .chunks(5)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        let rhs = if chunk[1] == "always" {
                            "<always>".to_owned()
                        } else {
                            state.expect_value(&chunk[2])?.to_string()
                        };
                        Ok(format!(
                            "{}:{} ? {} : {} @ {}",
                            initial, chunk[1], chunk[3], chunk[4], rhs
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64_async_post_flow_cond_chain @{} [{}]: start {}, loop while current {} {}, step await {}(current), update carries {}, then if {} {} {},",
                        node.resource, resource.kind.raw, initial, cmp, limit, step_callee, carries.join(", "), control_kind, control_rhs, control_action
                    ),
                );
                Ok(Value::Unit)
            }
            "loop_while_i64_post_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(5).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[6])?.to_string();
                let control_action = node.op.args.get(7).map_or("<missing>", String::as_str);
                let carries = node.op.args[8..]
                    .chunks(5)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        let rhs = if chunk[1] == "always" {
                            "<always>".to_owned()
                        } else {
                            state.expect_value(&chunk[2])?.to_string()
                        };
                        Ok(format!(
                            "{}:{} ? {} : {} @ {}",
                            initial, chunk[1], chunk[3], chunk[4], rhs
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64_post_flow_cond_chain @{} [{}]: start {}, loop while current {} {}, step {} {}, update carries {}, then if {} {} {},",
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
            "loop_while_i64_flow_cond_chain" => {
                let initial = state.expect_value(&node.op.args[0])?.clone();
                let limit = state.expect_value(&node.op.args[1])?.clone();
                let step = state.expect_value(&node.op.args[2])?.clone();
                let cmp = node.op.args.get(3).map_or("<missing>", String::as_str);
                let step_kind = node.op.args.get(4).map_or("<missing>", String::as_str);
                let control_kind = node.op.args.get(5).map_or("<missing>", String::as_str);
                let control_rhs = state.expect_value(&node.op.args[6])?.to_string();
                let control_action = node.op.args.get(7).map_or("<missing>", String::as_str);
                let carries = node.op.args[8..]
                    .chunks(5)
                    .map(|chunk| {
                        let initial = state.expect_value(&chunk[0])?.clone();
                        let rhs = if chunk[1] == "always" {
                            "<always>".to_owned()
                        } else {
                            state.expect_value(&chunk[2])?.to_string()
                        };
                        Ok(format!(
                            "{}:{} ? {} : {} @ {}",
                            initial, chunk[1], chunk[3], chunk[4], rhs
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.loop_while_i64_flow_cond_chain @{} [{}]: start {}, loop while current {} {}, step {} {}, if {} {} then {}, carries {}",
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

fn unwrap_present_frame_payload(value: Value) -> Value {
    match value {
        Value::DataWindow(window) => (*window.base).clone(),
        other => other,
    }
}

fn task_lifecycle_state(task: &yir_core::TaskHandle) -> TaskLifecycleState {
    match task.state {
        TaskLifecycleState::Cancelled => TaskLifecycleState::Cancelled,
        TaskLifecycleState::TimedOut => TaskLifecycleState::TimedOut,
        TaskLifecycleState::Completed => TaskLifecycleState::Completed,
        TaskLifecycleState::Pending => {
            if matches!(task.limit, Some(limit) if limit <= 0) {
                TaskLifecycleState::TimedOut
            } else {
                TaskLifecycleState::Completed
            }
        }
    }
}

fn require_cpu_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("cpu") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses cpu mod on non-cpu resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn cpu_struct_field_is_literal(value: &str) -> bool {
    matches!(value, "true" | "false")
        || value.parse::<i64>().is_ok()
        || value.parse::<f64>().is_ok()
}

fn normalize_channel(channel: &str) -> String {
    channel
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn resolve_project_profile_ref(node: &Node) -> Result<Value, String> {
    let domain = node.op.args[0].as_str();
    let _unit = node.op.args[1].as_str();
    let slot = node.op.args[2].as_str();
    match (domain, slot) {
        ("kernel", "bind_core") => Ok(Value::Int(0)),
        ("kernel", "queue_depth") => Ok(Value::Int(8)),
        ("kernel", "batch_lanes") => Ok(Value::Int(16)),
        ("data", "bind_core") => Ok(Value::Int(0)),
        ("data", "window_offset") => Ok(Value::Int(0)),
        ("data", "uplink_len") | ("data", "downlink_len") => Ok(Value::Int(1)),
        ("data", "handle_table") => Ok(Value::DataHandleTable(DataHandleTable {
            entries: Vec::new(),
        })),
        ("data", marker) if marker.starts_with("marker:") => Ok(Value::DataMarker(DataMarker {
            tag: marker.trim_start_matches("marker:").to_owned(),
        })),
        ("shader", "target") => Ok(Value::Target(SurfaceTarget {
            format: "rgba8".to_owned(),
            width: 64,
            height: 64,
        })),
        ("shader", "viewport") => Ok(Value::Viewport(Viewport {
            width: 64,
            height: 64,
        })),
        ("shader", "pipeline") => Ok(Value::Pipeline(RenderPipeline {
            shading_model: "flat".to_owned(),
            topology: "triangle".to_owned(),
        })),
        ("shader", "vertex_count")
        | ("shader", "instance_count")
        | ("shader", "packet_color_slot")
        | ("shader", "packet_speed_slot")
        | ("shader", "packet_radius_slot")
        | ("shader", "packet_tag")
        | ("shader", "material_mode")
        | ("shader", "pass_kind")
        | ("shader", "packet_field_count") => Ok(Value::Int(1)),
        _ => Ok(Value::Symbol(format!("{domain}.{}", slot))),
    }
}

fn execute_extern_i64(abi: &str, symbol: &str, args: &[i64]) -> Result<i64, String> {
    if abi != "nurs" && abi != "c" {
        return Err(format!("unsupported extern ABI `{abi}`"));
    }
    match symbol {
        "host_color_bias" | "HostRenderCurves__color_bias" | "HostMath__color_bias" => {
            let [value] = args else {
                return Err("host_color_bias expects 1 arg".to_owned());
            };
            Ok((value + 12).clamp(0, 255))
        }
        "host_speed_curve" | "HostRenderCurves__speed_curve" | "HostMath__speed_curve" => {
            let [value] = args else {
                return Err("host_speed_curve expects 1 arg".to_owned());
            };
            Ok(value * 2 + 3)
        }
        "host_radius_curve" | "HostRenderCurves__radius_curve" => {
            let [value] = args else {
                return Err("host_radius_curve expects 1 arg".to_owned());
            };
            Ok((value * 3) / 2 + 8)
        }
        "host_mix_tick" | "HostRenderCurves__mix_tick" => {
            let [base, tick] = args else {
                return Err("host_mix_tick expects 2 args".to_owned());
            };
            Ok(base + tick)
        }
        _ => Err("unknown extern symbol".to_owned()),
    }
}
