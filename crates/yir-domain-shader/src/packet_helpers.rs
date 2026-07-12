use yir_core::{StructValue, Value};

pub(crate) fn find_packet_field<'a>(
    packet: &'a StructValue,
    flat_names: &[&str],
    nested_struct_names: &[&str],
    nested_field_names: &[&str],
) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| flat_names.iter().any(|candidate| name == candidate))
        .map(|(_, value)| value)
        .or_else(|| {
            packet
                .fields
                .iter()
                .find(|(name, _)| {
                    nested_struct_names
                        .iter()
                        .any(|candidate| name == candidate)
                })
                .and_then(|(_, value)| match value {
                    Value::Struct(inner) => inner
                        .fields
                        .iter()
                        .find(|(name, _)| {
                            nested_field_names.iter().any(|candidate| name == candidate)
                        })
                        .map(|(_, value)| value),
                    _ => None,
                })
        })
}

pub(crate) fn find_flat_packet_field<'a>(
    packet: &'a StructValue,
    flat_names: &[&str],
) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| flat_names.iter().any(|candidate| name == candidate))
        .map(|(_, value)| value)
}

pub(crate) fn find_slider_packet_value<'a>(
    packet: &'a StructValue,
    slider_name: &str,
) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| name == "sliders")
        .and_then(|(_, value)| match value {
            Value::Struct(group) => group
                .fields
                .iter()
                .find(|(name, _)| name == slider_name)
                .and_then(|(_, value)| match value {
                    Value::Struct(slider) => slider
                        .fields
                        .iter()
                        .find(|(name, _)| name == "value")
                        .map(|(_, value)| value),
                    _ => None,
                }),
            _ => None,
        })
}

pub(crate) fn find_slider_packet_field<'a>(
    packet: &'a StructValue,
    slider_name: &str,
    field_name: &str,
) -> Option<&'a Value> {
    packet
        .fields
        .iter()
        .find(|(name, _)| name == "sliders")
        .and_then(|(_, value)| match value {
            Value::Struct(group) => group
                .fields
                .iter()
                .find(|(name, _)| name == slider_name)
                .and_then(|(_, value)| match value {
                    Value::Struct(slider) => slider
                        .fields
                        .iter()
                        .find(|(name, _)| name == field_name)
                        .map(|(_, value)| value),
                    _ => None,
                }),
            _ => None,
        })
}

pub(crate) fn normalize_control_value(value: i64, min: i64, max: i64) -> usize {
    let max = max.max(min + 1);
    let clamped = value.clamp(min, max);
    (((clamped - min) * 127) / (max - min)) as usize
}

pub(crate) fn scalar_to_color_key(value: &Value, op: &str) -> Result<i64, String> {
    match value {
        Value::Bool(value) => Ok(if *value { 1 } else { 0 }),
        Value::I32(value) => Ok(*value as i64),
        Value::Int(value) => Ok(*value),
        Value::F32(value) => Ok(value.round() as i64),
        Value::F64(value) => Ok(value.round() as i64),
        other => Err(format!("{op} expects scalar `color` value, got {}", other)),
    }
}

pub(crate) fn scalar_to_f32(value: &Value, op: &str) -> Result<f32, String> {
    match value {
        Value::Bool(value) => Ok(if *value { 1.0 } else { 0.0 }),
        Value::I32(value) => Ok(*value as f32),
        Value::Int(value) => Ok(*value as f32),
        Value::F32(value) => Ok(*value),
        Value::F64(value) => Ok(*value as f32),
        other => Err(format!("{op} expects scalar numeric value, got {}", other)),
    }
}
