use std::env;

use yir_core::{ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, Value};

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
            "null" => {
                if !node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects `cpu.null <name> <resource>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "borrow" | "move_ptr" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <ptr>`",
                        node.name, node.op.instruction
                    ));
                }

                Ok(InstructionSemantics::pure(node.op.args.clone()))
            }
            "add" | "sub" | "mul" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.{} <name> <resource> <lhs> <rhs>`",
                        node.name, node.op.instruction
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
                    format!("node `{}` has invalid width `{}`", node.name, node.op.args[0])
                })?;
                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!("node `{}` has invalid height `{}`", node.name, node.op.args[1])
                })?;

                Ok(InstructionSemantics::effect(Vec::new()))
            }
            "input_i64" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `cpu.input_i64 <name> <resource> <channel> <default>`",
                        node.name
                    ));
                }

                node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid default integer literal `{}`",
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
            "const" => Ok(Value::Int(node.op.args[0].parse::<i64>().map_err(|_| {
                format!(
                    "node `{}` has invalid integer literal `{}`",
                    node.name, node.op.args[0]
                )
            })?)),
            "null" => Ok(Value::Pointer(None)),
            "borrow" | "move_ptr" => Ok(Value::Pointer(state.expect_pointer(&node.op.args[0])?)),
            "add" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? + state.expect_int(&node.op.args[1])?,
            )),
            "sub" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? - state.expect_int(&node.op.args[1])?,
            )),
            "mul" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?,
            )),
            "madd" => Ok(Value::Int(
                state.expect_int(&node.op.args[0])? * state.expect_int(&node.op.args[1])?
                    + state.expect_int(&node.op.args[2])?,
            )),
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
                        next.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned())
                    ),
                );
                Ok(Value::Pointer(Some(address)))
            }
            "alloc_buffer" => {
                let len = state.expect_int(&node.op.args[0])?;
                if len < 0 {
                    return Err(format!("node `{}` allocates negative buffer length", node.name));
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned())
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned())
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned()),
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned()),
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned()),
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned()),
                        next.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned())
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned()),
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
                        pointer.map(|ptr| format!("&{ptr}")).unwrap_or_else(|| "null".to_owned())
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
                    format!("node `{}` has invalid width `{}`", node.name, node.op.args[0])
                })?;
                let height = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!("node `{}` has invalid height `{}`", node.name, node.op.args[1])
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
            "input_i64" => {
                let channel = &node.op.args[0];
                let default_value = node.op.args[1].parse::<i64>().map_err(|_| {
                    format!(
                        "node `{}` has invalid default integer literal `{}`",
                        node.name, node.op.args[1]
                    )
                })?;
                let env_name = format!("NUIS_UI_{}", normalize_channel(channel));
                let sampled = env::var(&env_name)
                    .ok()
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(default_value);
                state.push_resource_event(
                    resource,
                    format!(
                        "effect cpu.input_i64 @{} [{}] channel={} value={} source={}",
                        node.resource,
                        resource.kind.raw,
                        channel,
                        sampled,
                        if sampled == default_value { "default" } else { "env" }
                    ),
                );
                Ok(Value::Int(sampled))
            }
            "present_frame" => {
                let frame = state.expect_value(&node.op.args[0])?.clone();
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
                state.push_resource_event(resource, format!(
                    "effect cpu.print @{} [{}]: {}",
                    node.resource, resource.kind.raw, value
                ));
                Ok(Value::Unit)
            }
            other => Err(format!("unknown cpu instruction `{other}`")),
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
