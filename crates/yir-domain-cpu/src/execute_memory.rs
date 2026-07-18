use yir_core::{ExecutionState, Node, Resource, Value};

pub(crate) fn execute_cpu_memory_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Option<Value>, String> {
    let value = match node.op.instruction.as_str() {
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
        "copy_buffer_owned" => {
            let pointer = state.expect_pointer(&node.op.args[0])?;
            let elements = state.read_heap_buffer(pointer)?.elements.clone();
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.copy_buffer_owned @{} [{}] ptr={} len={}",
                    node.resource,
                    resource.kind.raw,
                    pointer
                        .map(|ptr| format!("&{ptr}"))
                        .unwrap_or_else(|| "null".to_owned()),
                    elements.len()
                ),
            );
            Ok(Value::OwnedBytes(elements))
        }
        "owned_bytes_len" => {
            let Value::OwnedBytes(bytes) = state.expect_value(&node.op.args[0])? else {
                return Err(format!("node `{}` expects owned bytes", node.name));
            };
            let byte_len = bytes
                .len()
                .checked_mul(std::mem::size_of::<i64>())
                .and_then(|len| i64::try_from(len).ok())
                .ok_or_else(|| format!("node `{}` owned byte length overflows i64", node.name))?;
            Ok(Value::Int(byte_len))
        }
        "drop_owned_bytes" => {
            if !matches!(state.expect_value(&node.op.args[0])?, Value::OwnedBytes(_)) {
                return Err(format!("node `{}` expects owned bytes", node.name));
            }
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.drop_owned_bytes @{} [{}]",
                    node.resource, resource.kind.raw
                ),
            );
            Ok(Value::Unit)
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

        _ => return Ok(None),
    };
    value.map(Some)
}
