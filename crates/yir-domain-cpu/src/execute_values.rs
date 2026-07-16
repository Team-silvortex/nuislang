use yir_core::{ExecutionState, Node, Resource, StructValue, Value};

use crate::runtime_helpers::resolve_project_profile_ref;

pub(crate) fn execute_cpu_value_node(
    node: &Node,
    resource: &Resource,
    state: &mut ExecutionState,
) -> Result<Option<Value>, String> {
    let value = match node.op.instruction.as_str() {
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
        "async_value" => Ok(state.expect_value(&node.op.args[0])?.clone()),
        _ => return Ok(None),
    }?;
    Ok(Some(value))
}
