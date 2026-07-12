use super::packet_helpers::{find_packet_field, scalar_to_color_key};
use yir_core::StructValue;

pub(crate) fn field(
    packet: &StructValue,
    op: &str,
    flat_names: &[&str],
    group: &str,
    name: &str,
    default: i64,
) -> Result<i64, String> {
    field_with(packet, op, flat_names, &[group], &[name], || default)
}

pub(crate) fn field_with(
    packet: &StructValue,
    op: &str,
    flat_names: &[&str],
    groups: &[&str],
    names: &[&str],
    default: impl FnOnce() -> i64,
) -> Result<i64, String> {
    find_packet_field(packet, flat_names, groups, names)
        .map(|value| scalar_to_color_key(value, op))
        .transpose()
        .map(|value| value.unwrap_or_else(default))
}

pub(crate) fn scaled(value: f32, factor: f32) -> i64 {
    (value * factor).round() as i64
}
