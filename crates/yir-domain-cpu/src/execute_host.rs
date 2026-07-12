use std::env;

use yir_core::{ExecutionState, Node, Resource, StructValue, Value};

use crate::runtime_helpers::{
    execute_extern_i32, execute_extern_i64, normalize_channel, unwrap_present_frame_payload,
};

pub(crate) fn execute_cpu_host_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Option<Value>, String> {
    let value: Value = match node.op.instruction.as_str() {
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
            Ok::<Value, String>(Value::Tuple(values))
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
            let frame = unwrap_present_frame_payload(state.expect_value(&node.op.args[0])?.clone());
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
        _ => return Ok(None),
    }?;
    Ok(Some(value))
}
