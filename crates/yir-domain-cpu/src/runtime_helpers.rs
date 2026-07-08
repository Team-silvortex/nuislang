use std::collections::BTreeMap;
use yir_core::{
    DataHandleTable, DataMarker, Node, RenderPipeline, Resource, StructValue, SurfaceTarget,
    TaskLifecycleState, Value, VariantUnionValue, Viewport,
};

pub(crate) fn unwrap_present_frame_payload(value: Value) -> Value {
    match value {
        Value::DataWindow(window) => (*window.base).clone(),
        other => other,
    }
}

pub(crate) fn task_lifecycle_state(task: &yir_core::TaskHandle) -> TaskLifecycleState {
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

pub(crate) fn task_lifecycle_state_for_thread(
    thread: &yir_core::ThreadHandle,
) -> TaskLifecycleState {
    match thread.state {
        TaskLifecycleState::Cancelled => TaskLifecycleState::Cancelled,
        TaskLifecycleState::TimedOut => TaskLifecycleState::TimedOut,
        TaskLifecycleState::Completed | TaskLifecycleState::Pending => {
            TaskLifecycleState::Completed
        }
    }
}

pub(crate) fn require_cpu_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("cpu") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses cpu mod on non-cpu resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

pub(crate) fn cpu_struct_field_is_literal(value: &str) -> bool {
    matches!(value, "true" | "false")
        || value.parse::<i64>().is_ok()
        || value.parse::<f64>().is_ok()
}

pub(crate) fn normalize_channel(channel: &str) -> String {
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

pub(crate) fn resolve_project_profile_ref(node: &Node) -> Result<Value, String> {
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

pub(crate) fn execute_extern_i64(abi: &str, symbol: &str, args: &[i64]) -> Result<i64, String> {
    if abi == "libc" {
        return match symbol {
            "getpid" => {
                let [] = args else {
                    return Err("libc getpid expects 0 args".to_owned());
                };
                Ok(1)
            }
            "usleep" => {
                let [_usec] = args else {
                    return Err("libc usleep expects 1 arg".to_owned());
                };
                Ok(0)
            }
            "puts" => {
                let [_message] = args else {
                    return Err("libc puts expects 1 arg".to_owned());
                };
                Ok(0)
            }
            "strlen" => {
                let [_message] = args else {
                    return Err("libc strlen expects 1 arg".to_owned());
                };
                Ok(0)
            }
            "write" => {
                let [_fd, _message, len] = args else {
                    return Err("libc write expects 3 args".to_owned());
                };
                Ok(*len)
            }
            "close" => {
                let [_fd] = args else {
                    return Err("libc close expects 1 arg".to_owned());
                };
                Ok(0)
            }
            "read" => {
                let [_fd, _buffer, _len] = args else {
                    return Err("libc read expects 3 args".to_owned());
                };
                Ok(-1)
            }
            _ => Err("unknown libc extern symbol".to_owned()),
        };
    }
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

pub(crate) fn execute_extern_i32(abi: &str, symbol: &str, args: &[i64]) -> Result<i32, String> {
    let value = execute_extern_i64(abi, symbol, args)?;
    Ok(value as i32)
}

pub(crate) fn variant_parent_name(type_name: &str) -> Option<&str> {
    type_name.rsplit_once('.').map(|(parent, _)| parent)
}

pub(crate) fn struct_as_variant_union(
    struct_value: &StructValue,
    active_variant: String,
) -> Option<VariantUnionValue> {
    let parent_type_name = variant_parent_name(&struct_value.type_name)?.to_owned();
    let mut variants = BTreeMap::new();
    variants.insert(struct_value.type_name.clone(), struct_value.clone());
    Some(VariantUnionValue {
        parent_type_name,
        active_variant,
        variants,
    })
}

pub(crate) fn merge_variant_maps(
    lhs: &BTreeMap<String, StructValue>,
    rhs: &BTreeMap<String, StructValue>,
) -> BTreeMap<String, StructValue> {
    let mut merged = lhs.clone();
    for (name, value) in rhs {
        merged.entry(name.clone()).or_insert_with(|| value.clone());
    }
    merged
}

pub(crate) fn select_variant_union(
    cond: bool,
    then_value: &Value,
    else_value: &Value,
) -> Option<VariantUnionValue> {
    match (then_value, else_value) {
        (Value::Struct(then_struct), Value::Struct(else_struct))
            if then_struct.type_name != else_struct.type_name =>
        {
            let then_parent = variant_parent_name(&then_struct.type_name)?;
            let else_parent = variant_parent_name(&else_struct.type_name)?;
            if then_parent != else_parent {
                return None;
            }
            let then_union = struct_as_variant_union(
                then_struct,
                if cond {
                    then_struct.type_name.clone()
                } else {
                    else_struct.type_name.clone()
                },
            )?;
            let else_union =
                struct_as_variant_union(else_struct, then_union.active_variant.clone())?;
            Some(VariantUnionValue {
                parent_type_name: then_parent.to_owned(),
                active_variant: then_union.active_variant,
                variants: merge_variant_maps(&then_union.variants, &else_union.variants),
            })
        }
        (Value::VariantUnion(then_union), Value::VariantUnion(else_union)) => {
            if then_union.parent_type_name != else_union.parent_type_name {
                return None;
            }
            Some(VariantUnionValue {
                parent_type_name: then_union.parent_type_name.clone(),
                active_variant: if cond {
                    then_union.active_variant.clone()
                } else {
                    else_union.active_variant.clone()
                },
                variants: merge_variant_maps(&then_union.variants, &else_union.variants),
            })
        }
        (Value::VariantUnion(union), Value::Struct(struct_value))
        | (Value::Struct(struct_value), Value::VariantUnion(union)) => {
            let parent = variant_parent_name(&struct_value.type_name)?;
            if parent != union.parent_type_name {
                return None;
            }
            let mut variants = union.variants.clone();
            variants
                .entry(struct_value.type_name.clone())
                .or_insert_with(|| struct_value.clone());
            let active_variant = if matches!(then_value, Value::VariantUnion(_)) {
                if cond {
                    union.active_variant.clone()
                } else {
                    struct_value.type_name.clone()
                }
            } else if cond {
                struct_value.type_name.clone()
            } else {
                union.active_variant.clone()
            };
            Some(VariantUnionValue {
                parent_type_name: union.parent_type_name.clone(),
                active_variant,
                variants,
            })
        }
        _ => None,
    }
}
