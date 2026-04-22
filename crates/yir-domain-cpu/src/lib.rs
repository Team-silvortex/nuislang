use std::env;

use yir_core::{
    ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, StructValue,
    TaskLifecycleState, Value,
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
                                .map(|(_, value)| value.trim().to_owned())
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
            "add" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? + state.expect_int(&node.op.args[1])?,
            )),
            "add_i32" => Ok(Value::I32(
                state.expect_i32(&node.op.args[0])? + state.expect_i32(&node.op.args[1])?,
            )),
            "add_f32" => Ok(Value::F32(
                state.expect_f32(&node.op.args[0])? + state.expect_f32(&node.op.args[1])?,
            )),
            "add_f64" => Ok(Value::F64(
                state.expect_f64(&node.op.args[0])? + state.expect_f64(&node.op.args[1])?,
            )),
            "sub" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? - state.expect_int(&node.op.args[1])?,
            )),
            "sub_i32" => Ok(Value::I32(
                state.expect_i32(&node.op.args[0])? - state.expect_i32(&node.op.args[1])?,
            )),
            "sub_f32" => Ok(Value::F32(
                state.expect_f32(&node.op.args[0])? - state.expect_f32(&node.op.args[1])?,
            )),
            "sub_f64" => Ok(Value::F64(
                state.expect_f64(&node.op.args[0])? - state.expect_f64(&node.op.args[1])?,
            )),
            "mul" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?,
            )),
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
                let lhs = state.expect_int(&node.op.args[0])?;
                let rhs = state.expect_int(&node.op.args[1])?;
                if rhs == 0 {
                    return Err(format!("node `{}` divides by zero", node.name));
                }
                Ok(Value::Int(lhs / rhs))
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
            "eq" => Ok(Value::Int(
                (state.expect_int(&node.op.args[0])? == state.expect_int(&node.op.args[1])?) as i64,
            )),
            "eq_i32" => Ok(Value::Bool(
                state.expect_i32(&node.op.args[0])? == state.expect_i32(&node.op.args[1])?,
            )),
            "eq_f32" => Ok(Value::Bool(
                state.expect_f32(&node.op.args[0])? == state.expect_f32(&node.op.args[1])?,
            )),
            "eq_f64" => Ok(Value::Bool(
                state.expect_f64(&node.op.args[0])? == state.expect_f64(&node.op.args[1])?,
            )),
            "ne" => Ok(Value::Int(
                (state.expect_int(&node.op.args[0])? != state.expect_int(&node.op.args[1])?) as i64,
            )),
            "lt" => Ok(Value::Int(
                (state.expect_int(&node.op.args[0])? < state.expect_int(&node.op.args[1])?) as i64,
            )),
            "lt_i32" => Ok(Value::Bool(
                state.expect_i32(&node.op.args[0])? < state.expect_i32(&node.op.args[1])?,
            )),
            "lt_f32" => Ok(Value::Bool(
                state.expect_f32(&node.op.args[0])? < state.expect_f32(&node.op.args[1])?,
            )),
            "lt_f64" => Ok(Value::Bool(
                state.expect_f64(&node.op.args[0])? < state.expect_f64(&node.op.args[1])?,
            )),
            "gt" => Ok(Value::Int(
                (state.expect_int(&node.op.args[0])? > state.expect_int(&node.op.args[1])?) as i64,
            )),
            "gt_i32" => Ok(Value::Bool(
                state.expect_i32(&node.op.args[0])? > state.expect_i32(&node.op.args[1])?,
            )),
            "gt_f32" => Ok(Value::Bool(
                state.expect_f32(&node.op.args[0])? > state.expect_f32(&node.op.args[1])?,
            )),
            "gt_f64" => Ok(Value::Bool(
                state.expect_f64(&node.op.args[0])? > state.expect_f64(&node.op.args[1])?,
            )),
            "le" => Ok(Value::Int(
                (state.expect_int(&node.op.args[0])? <= state.expect_int(&node.op.args[1])?) as i64,
            )),
            "ge" => Ok(Value::Int(
                (state.expect_int(&node.op.args[0])? >= state.expect_int(&node.op.args[1])?) as i64,
            )),
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
