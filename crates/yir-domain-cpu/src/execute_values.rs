use std::collections::BTreeMap;

use yir_core::{
    parse_branch_owned_call_args, parse_owned_struct_layout, ExecutionState, Node,
    OwnedStructFieldLayout, OwnedStructLayout, OwnedStructScalarLayout, Resource, StructValue,
    Value, VariantUnionValue, OWNED_VARIANT_UNION_LAYOUT_PREFIX,
};

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
        "param_buffer_ref" | "param_node_ref" => Ok(Value::Pointer(None)),
        "param_owned_bytes" => Ok(Value::OwnedBytes(Vec::new())),
        "loop_owned_result" => Ok(Value::OwnedBytes(Vec::new())),
        "loop_owned_struct_result" => {
            default_owned_layout_value(parse_owned_struct_layout(&node.op.args[1])?)
        }
        "call_bool"
        | "call_i32"
        | "call_i64"
        | "call_f32"
        | "call_f64"
        | "call_owned_bytes"
        | "call_owned_external_buffer" => {
            let callee = &node.op.args[0];
            let argument_offset = if node.op.instruction == "call_owned_external_buffer" {
                5
            } else {
                1
            };
            let args = node.op.args[argument_offset..]
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
                "call_owned_bytes" => Ok(Value::OwnedBytes(Vec::new())),
                "call_owned_external_buffer" => Ok(Value::Pointer(None)),
                _ => Ok(Value::Int(0)),
            }
        }
        "branch_call_owned_bytes" => {
            let args = parse_branch_owned_call_args(&node.op.args).ok_or_else(|| {
                format!(
                    "node `{}` has invalid branch scalar argument segments",
                    node.name
                )
            })?;
            let selected_callee = if state.expect_bool(args.condition)? {
                (args.then_callee, args.then_scalar_args)
            } else {
                (args.else_callee, args.else_scalar_args)
            };
            let Value::OwnedBytes(bytes) = state.expect_value(args.owner)? else {
                return Err(format!("node `{}` expects an owned bytes input", node.name));
            };
            let bytes = bytes.clone();
            for arg in selected_callee.1 {
                state.expect_value(arg)?;
            }
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.branch_call_owned_bytes @{} [{}] {}({})",
                    node.resource,
                    resource.kind.raw,
                    selected_callee.0,
                    selected_callee.1.join(", ")
                ),
            );
            Ok(Value::OwnedBytes(bytes))
        }
        "call_owned_struct" => {
            let layout_source = node.op.args.get(1).ok_or_else(|| {
                format!("node `{}` is missing its owned struct layout", node.name)
            })?;
            let layout = parse_owned_struct_layout(layout_source).map_err(|error| {
                format!(
                    "node `{}` has invalid owned struct layout: {error}",
                    node.name
                )
            })?;
            default_owned_layout_value(layout)
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
        "return_owned_struct" => {
            let value = state.expect_value(&node.op.args[0])?.clone();
            let type_name = match &value {
                Value::Struct(value) => value.type_name.as_str(),
                Value::VariantUnion(value) => {
                    let layout_source = node.op.args.get(1).ok_or_else(|| {
                        format!(
                            "node `{}` must bind an owned variant union layout",
                            node.name
                        )
                    })?;
                    let layout = parse_owned_struct_layout(layout_source).map_err(|error| {
                        format!(
                            "node `{}` has invalid owned variant union layout: {error}",
                            node.name
                        )
                    })?;
                    if layout
                        .type_name
                        .strip_prefix(OWNED_VARIANT_UNION_LAYOUT_PREFIX)
                        != Some(value.parent_type_name.as_str())
                    {
                        return Err(format!(
                            "node `{}` owned variant union layout does not match `{}`",
                            node.name, value.parent_type_name
                        ));
                    }
                    value.parent_type_name.as_str()
                }
                other => {
                    return Err(format!(
                        "node `{}` expects an owned struct or variant union, got {other}",
                        node.name
                    ))
                }
            };
            state.push_resource_event(
                resource,
                format!(
                    "effect cpu.return_owned_struct @{} [{}] {}",
                    node.resource, resource.kind.raw, type_name
                ),
            );
            Ok(value)
        }
        "return_owned_bytes" => {
            let Value::OwnedBytes(bytes) = state.expect_value(&node.op.args[0])? else {
                return Err(format!("node `{}` expects owned bytes", node.name));
            };
            Ok(Value::OwnedBytes(bytes.clone()))
        }
        "return_owned_external_buffer" => {
            Ok(Value::Pointer(state.expect_pointer(&node.op.args[0])?))
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

fn default_owned_layout_value(layout: OwnedStructLayout) -> Result<Value, String> {
    let Some(parent_type_name) = layout
        .type_name
        .strip_prefix(OWNED_VARIANT_UNION_LAYOUT_PREFIX)
        .map(str::to_owned)
    else {
        let fields = layout
            .fields
            .into_iter()
            .map(|(name, field)| Ok((name, default_owned_field_value(field)?)))
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(Value::Struct(StructValue {
            type_name: layout.type_name,
            fields,
        }));
    };

    let mut has_tag = false;
    let mut active_variant = None;
    let mut variants = BTreeMap::new();
    for (name, field) in layout.fields {
        if name == "tag" {
            if has_tag || field != OwnedStructFieldLayout::Scalar(OwnedStructScalarLayout::I64) {
                return Err("owned variant union must contain one i64 tag".to_owned());
            }
            has_tag = true;
            continue;
        }
        let OwnedStructFieldLayout::Struct(variant_layout) = field else {
            return Err(format!(
                "owned variant union field `{name}` must contain a struct layout"
            ));
        };
        if variant_layout.type_name != name {
            return Err(format!(
                "owned variant union field `{name}` does not match nested type `{}`",
                variant_layout.type_name
            ));
        }
        let Value::Struct(variant) = default_owned_layout_value(variant_layout)? else {
            return Err(format!(
                "owned variant union field `{name}` cannot contain another union root"
            ));
        };
        active_variant.get_or_insert_with(|| name.clone());
        if variants.insert(name.clone(), variant).is_some() {
            return Err(format!("owned variant union repeats field `{name}`"));
        }
    }
    if !has_tag {
        return Err("owned variant union is missing its i64 tag".to_owned());
    }
    Ok(Value::VariantUnion(VariantUnionValue {
        parent_type_name,
        active_variant: active_variant
            .ok_or_else(|| "owned variant union has no variants".to_owned())?,
        variants,
    }))
}

fn default_owned_field_value(field: OwnedStructFieldLayout) -> Result<Value, String> {
    match field {
        OwnedStructFieldLayout::Struct(layout) => default_owned_layout_value(layout),
        OwnedStructFieldLayout::Scalar(kind) => Ok(match kind {
            OwnedStructScalarLayout::Bool => Value::Bool(false),
            OwnedStructScalarLayout::I32 => Value::I32(0),
            OwnedStructScalarLayout::I64 => Value::Int(0),
            OwnedStructScalarLayout::F32 => Value::F32(0.0),
            OwnedStructScalarLayout::F64 => Value::F64(0.0),
            OwnedStructScalarLayout::String => Value::Symbol(String::new()),
            OwnedStructScalarLayout::Bytes => Value::OwnedBytes(Vec::new()),
        }),
    }
}
