use super::*;
use crate::data_mod_describe::describe_data_node;

pub struct DataMod;

impl RegisteredMod for DataMod {
    fn module_name(&self) -> &'static str {
        "data"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        describe_data_node(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if node.op.instruction != "move" {
            require_data_resource(node, resource)?;
        }
        match node.op.instruction.as_str() {
            "move" => {
                let input = &node.op.args[0];
                let target = &node.op.args[1];
                let value = state.expect_value(input)?.clone();
                if !is_move_value_legal(&value) {
                    return Err(format!(
                        "data.move only accepts Value payloads, got {}",
                        value
                    ));
                }
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.move @{} [{}] -> {}: {}",
                        node.resource, resource.kind.raw, target, value
                    ),
                );
                Ok(value)
            }
            "copy_window" | "immutable_window" => {
                let base = state.expect_value(&node.op.args[0])?.clone();
                if !is_window_base_legal(&base) {
                    return Err(format!(
                        "data.{} cannot wrap non-window-compatible payload {}",
                        node.op.instruction, base
                    ));
                }
                let offset = resolve_window_usize_arg(state, node, 1, "offset")?;
                let len = resolve_window_usize_arg(state, node, 2, "len")?;
                validate_data_window_range(state, &base, offset, len)?;
                let window = Value::DataWindow(DataWindow {
                    base: Box::new(base),
                    offset,
                    len,
                    immutable: node.op.instruction == "immutable_window",
                });
                Ok(window)
            }
            "read_window" => {
                let window = state.expect_value(&node.op.args[0])?.clone();
                let index = resolve_window_usize_arg(state, node, 1, "index")?;
                read_data_window_value(state, window, index)
            }
            "write_window" => {
                let window = state.expect_value(&node.op.args[0])?.clone();
                let index = resolve_window_usize_arg(state, node, 1, "index")?;
                let value = state.expect_value(&node.op.args[2])?.clone();
                write_data_window_value(state, window, index, value)
            }
            "freeze_window" => {
                let window = state.expect_value(&node.op.args[0])?.clone();
                match window {
                    Value::DataWindow(window) => Ok(Value::DataWindow(DataWindow {
                        immutable: true,
                        ..window
                    })),
                    other => Err(format!(
                        "data.freeze_window expects window input, got {}",
                        other
                    )),
                }
            }
            "marker" => Ok(Value::DataMarker(DataMarker {
                tag: node.op.args[0].clone(),
            })),
            "output_pipe" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                if !is_pipe_payload_legal(&value) {
                    return Err(format!(
                        "data.output_pipe cannot wrap illegal pipe payload {}",
                        value
                    ));
                }
                let pipe = Value::DataPipe(DataPipe {
                    direction: DataPipeDirection::Output,
                    payload: Box::new(value),
                });
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.output_pipe @{} [{}]: {}",
                        node.resource, resource.kind.raw, pipe
                    ),
                );
                Ok(pipe)
            }
            "input_pipe" => {
                let pipe = state.expect_value(&node.op.args[0])?.clone();
                match pipe {
                    Value::DataPipe(DataPipe {
                        direction: DataPipeDirection::Output,
                        payload,
                    }) => {
                        let value = (*payload).clone();
                        state.push_resource_event(
                            resource,
                            format!(
                                "effect data.input_pipe @{} [{}]: {}",
                                node.resource, resource.kind.raw, value
                            ),
                        );
                        Ok(value)
                    }
                    other => Err(format!(
                        "data.input_pipe expects output pipe, got {}",
                        other
                    )),
                }
            }
            "observe" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let flow = parse_data_flow_state(&node.op.args[1])?;
                Ok(Value::DataResult(DataResultHandle {
                    state: flow,
                    value: Box::new(value),
                }))
            }
            "is_ready" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(result.state, DataFlowState::Ready)))
            }
            "is_moved" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(result.state, DataFlowState::Moved)))
            }
            "is_windowed" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(result.state, DataFlowState::Windowed)))
            }
            "value" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok((*result.value).clone())
            }
            "handle_table" => {
                let mut entries = Vec::with_capacity(node.op.args.len());
                for entry in &node.op.args {
                    let Some((slot, resource_name)) = entry.split_once('=') else {
                        return Err(format!(
                            "node `{}` has invalid handle-table entry `{}`",
                            node.name, entry
                        ));
                    };
                    let slot = slot.trim();
                    let resource_name = resource_name.trim();
                    if slot.is_empty() || resource_name.is_empty() {
                        return Err(format!(
                            "node `{}` has empty handle-table slot/resource in `{}`",
                            node.name, entry
                        ));
                    }
                    entries.push((slot.to_owned(), resource_name.to_owned()));
                }
                Ok(Value::DataHandleTable(DataHandleTable { entries }))
            }
            "provider_request_ingress" => {
                let request_handle = state.expect_value(&node.op.args[0])?.clone();
                for arg in &node.op.args[1..] {
                    state.expect_value(arg)?;
                }
                let capsule = if node.op.args.len() == 8 {
                    format!(
                        ", capsule {}, input roles {}, output roles {}",
                        node.op.args[5], node.op.args[6], node.op.args[7]
                    )
                } else {
                    String::new()
                };
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.provider_request_ingress @{} [{}]: request {}, descriptor table {}, descriptor count {}, provider {}, capability {}{}",
                        node.resource,
                        resource.kind.raw,
                        node.op.args[0],
                        node.op.args[1],
                        node.op.args[2],
                        node.op.args[3],
                        node.op.args[4],
                        capsule,
                    ),
                );
                Ok(request_handle)
            }
            "bind_core" => {
                let core_index = node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid fabric core index `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                let binding = Value::DataCoreBinding(DataCoreBinding { core_index });
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.bind_core @{} [{}]: {}",
                        node.resource, resource.kind.raw, binding
                    ),
                );
                Ok(binding)
            }
            other => Err(format!("unknown data instruction `{other}`")),
        }
    }
}

pub(super) fn require_data_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("data") || resource.kind.is_family("fabric") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses data mod on non-data resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

pub(super) fn parse_data_flow_state(raw: &str) -> Result<DataFlowState, String> {
    match raw {
        "ready" => Ok(DataFlowState::Ready),
        "moved" => Ok(DataFlowState::Moved),
        "windowed" => Ok(DataFlowState::Windowed),
        other => Err(format!("unknown data flow state `{other}`")),
    }
}

fn is_move_value_legal(value: &Value) -> bool {
    match value {
        Value::DataWindow(_)
        | Value::DataPipe(_)
        | Value::DataResult(_)
        | Value::DataMarker(_)
        | Value::DataHandleTable(_) => false,
        Value::Tuple(items) => items.iter().all(is_move_value_legal),
        Value::Struct(value) => value
            .fields
            .iter()
            .all(|(_, value)| is_move_value_legal(value)),
        Value::VariantUnion(value) => value.variants.values().all(|variant| {
            variant
                .fields
                .iter()
                .all(|(_, value)| is_move_value_legal(value))
        }),
        _ => true,
    }
}

fn is_window_base_legal(value: &Value) -> bool {
    match value {
        Value::DataWindow(_)
        | Value::DataHandleTable(_)
        | Value::DataMarker(_)
        | Value::DataPipe(_)
        | Value::DataResult(_) => false,
        Value::Tuple(items) => items.iter().all(is_move_value_legal),
        Value::Struct(value) => value
            .fields
            .iter()
            .all(|(_, value)| is_move_value_legal(value)),
        Value::VariantUnion(value) => value.variants.values().all(|variant| {
            variant
                .fields
                .iter()
                .all(|(_, value)| is_move_value_legal(value))
        }),
        _ => true,
    }
}

fn is_pipe_payload_legal(value: &Value) -> bool {
    match value {
        Value::DataWindow(window) if !window.immutable => false,
        Value::DataPipe(_) | Value::DataResult(_) => false,
        Value::Tuple(items) => items.iter().all(is_move_value_legal),
        Value::Struct(value) => value
            .fields
            .iter()
            .all(|(_, value)| is_move_value_legal(value)),
        _ => true,
    }
}

fn validate_data_window_range(
    state: &ExecutionState,
    base: &Value,
    offset: usize,
    len: usize,
) -> Result<(), String> {
    match base {
        Value::Pointer(pointer) => {
            let Some(address) = pointer else {
                return Err("data window cannot wrap null buffer pointer".to_owned());
            };
            if state.heap.contains_key(address) {
                return Err(format!(
                    "data window cannot wrap node pointer `&{address}` as buffer backing"
                ));
            }
            let buffer = state.buffers.get(address).ok_or_else(|| {
                format!("data window cannot wrap dangling buffer pointer `&{address}`")
            })?;
            let end = offset
                .checked_add(len)
                .ok_or_else(|| "data window range overflows usize".to_owned())?;
            if end > buffer.elements.len() {
                return Err(format!(
                    "data window range offset={} len={} exceeds buffer length {}",
                    offset,
                    len,
                    buffer.elements.len()
                ));
            }
            Ok(())
        }
        _ if offset == 0 && len == 1 => Ok(()),
        _ => Err(format!(
            "inline data window currently supports only offset=0 len=1, got offset={} len={}",
            offset, len
        )),
    }
}

fn write_data_window_value(
    state: &mut ExecutionState,
    window: Value,
    index: usize,
    value: Value,
) -> Result<Value, String> {
    let Value::DataWindow(window) = window else {
        return Err("data.write_window expects window input".to_owned());
    };
    if window.immutable {
        return Err("data.write_window cannot modify immutable window".to_owned());
    }
    let slot = window
        .offset
        .checked_add(index)
        .ok_or_else(|| "data.write_window index overflows usize".to_owned())?;
    if index >= window.len {
        return Err(format!(
            "data.write_window index {} out of bounds for window len={}",
            index, window.len
        ));
    }
    if let Value::Pointer(pointer) = window.base.as_ref() {
        let scalar = match value {
            Value::Int(value) => value,
            Value::I32(value) => value as i64,
            other => {
                return Err(format!(
                    "data.write_window expects i64-like payload for buffer-backed window, got {}",
                    other
                ))
            }
        };
        state.write_heap_buffer_at(*pointer, slot, scalar)?;
        return Ok(Value::DataWindow(window));
    }
    if window.offset != 0 || window.len != 1 || index != 0 {
        return Err(format!(
            "inline data.write_window currently supports only single-slot mutable windows, got offset={} len={} index={index}",
            window.offset, window.len
        ));
    }
    Ok(Value::DataWindow(DataWindow {
        base: Box::new(value),
        ..window
    }))
}

fn read_data_window_value(
    state: &ExecutionState,
    window: Value,
    index: usize,
) -> Result<Value, String> {
    let Value::DataWindow(window) = window else {
        return Err("data.read_window expects window input".to_owned());
    };
    if index >= window.len {
        return Err(format!(
            "data.read_window index {} out of bounds for window len={}",
            index, window.len
        ));
    }
    let slot = window
        .offset
        .checked_add(index)
        .ok_or_else(|| "data.read_window index overflows usize".to_owned())?;
    if let Value::Pointer(pointer) = window.base.as_ref() {
        return Ok(Value::Int(state.read_heap_buffer_at(*pointer, slot)?));
    }
    if window.offset != 0 || window.len != 1 || index != 0 {
        return Err(format!(
            "inline data.read_window currently supports only single-slot windows, got offset={} len={} index={index}",
            window.offset, window.len
        ));
    }
    Ok((*window.base).clone())
}

fn resolve_window_usize_arg(
    state: &ExecutionState,
    node: &Node,
    index: usize,
    label: &str,
) -> Result<usize, String> {
    let raw = &node.op.args[index];
    if let Ok(value) = raw.parse::<usize>() {
        return Ok(value);
    }
    let value = state.expect_int(raw)?;
    usize::try_from(value).map_err(|_| {
        format!(
            "node `{}` has invalid window {} `{}`",
            node.name, label, raw
        )
    })
}

pub struct LegacyFabricMod;

impl RegisteredMod for LegacyFabricMod {
    fn module_name(&self) -> &'static str {
        "fabric"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        DataMod.describe(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        DataMod.execute(node, resource, state)
    }
}
