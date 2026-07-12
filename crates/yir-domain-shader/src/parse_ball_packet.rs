use super::ball_packet::BallPacket;
use super::packet_helpers::{find_flat_packet_field, find_slider_packet_value, scalar_to_f32};
use super::parse_ball_packet_controls::parse_ball_packet_controls;
use super::parse_ball_packet_frame_sync::parse_ball_packet_frame_sync;
use super::parse_ball_packet_response::parse_ball_packet_response;
use super::parse_ball_packet_scene_core::parse_ball_packet_scene_core;
use super::parse_ball_packet_scene_runtime::parse_ball_packet_scene_runtime;
use super::parse_ball_packet_tuple::parse_ball_packet_tuple;
use yir_core::{StructValue, Value};

pub(crate) fn parse_ball_packet(value: &Value, op: &str) -> Result<BallPacket, String> {
    match value {
        Value::Tuple(items) if items.len() >= 2 => parse_ball_packet_tuple(items, op),
        Value::Struct(packet) => parse_ball_packet_struct(packet, op),
        _ => Err(format!(
            "{op} expects a packet tuple `(color, speed[, radius_scale])` or struct with `color` and `speed`"
        )),
    }
}

fn parse_ball_packet_struct(packet: &StructValue, op: &str) -> Result<BallPacket, String> {
    let color = find_slider_packet_value(packet, "color")
        .or_else(|| find_flat_packet_field(packet, &["color", "slider_color"]))
        .ok_or_else(|| format!("{op} struct packet is missing `color` field"))?;
    let speed = find_slider_packet_value(packet, "speed")
        .or_else(|| find_flat_packet_field(packet, &["speed", "slider_speed"]))
        .ok_or_else(|| format!("{op} struct packet is missing `speed` field"))?;
    let radius_scale = find_slider_packet_value(packet, "radius")
        .or_else(|| find_flat_packet_field(packet, &["radius_scale", "slider_radius"]))
        .map(|value| scalar_to_f32(value, op))
        .transpose()?
        .unwrap_or(1.0);

    let scene_core = parse_ball_packet_scene_core(packet, op, color, speed, radius_scale)?;
    let scene_runtime = parse_ball_packet_scene_runtime(
        packet,
        op,
        scene_core.scene_cluster_instance_group_slot,
        scene_core.instance_group_visible_count,
        scene_core.scene_node_visibility,
        scene_core.instance_group_phase_bias,
    )?;
    let frame_sync = parse_ball_packet_frame_sync(
        packet,
        op,
        radius_scale,
        scene_core.accent,
        scene_core.contrast,
        speed,
    )?;
    let response = parse_ball_packet_response(
        packet,
        op,
        radius_scale,
        scene_core.accent,
        scene_core.contrast,
        speed,
    )?;
    let controls = parse_ball_packet_controls(packet, op, radius_scale, scene_core.accent, speed)?;

    BallPacket::from_parts(
        color,
        speed,
        radius_scale,
        scene_core,
        scene_runtime,
        frame_sync,
        response,
        controls,
        op,
    )
}
