mod ball_packet;
mod ball_packet_from_parts;
mod control_panel_extended_summary;
mod control_panel_layout;
mod control_panel_summary;
mod control_panel_surface;
mod control_panel_widgets;
mod describe;
mod execute_core;
mod execute_effects;
mod flow_state;
mod frame_surface;
mod geometry_overlay;
mod packet_helpers;
mod parse_ball_packet;
mod parse_ball_packet_controls;
mod parse_ball_packet_frame_sync;
mod parse_ball_packet_response;
mod parse_ball_packet_scene_core;
mod parse_ball_packet_scene_core_fields;
mod parse_ball_packet_scene_core_helpers;
mod parse_ball_packet_scene_runtime;
mod parse_ball_packet_tuple;
mod render_pass;
mod scene_preview;
mod scene_runtime_overlay;
mod sphere_render;
mod surface_primitives;
mod texture_sampling;

use ball_packet::BallPacket;
use control_panel_surface::draw_control_panel_surface;
use describe::describe_shader_node;
use execute_core::execute_shader_core_node;
use execute_effects::execute_shader_effect_node;
use flow_state::parse_shader_flow_state;
use parse_ball_packet::parse_ball_packet;
use render_pass::draw_render_pass_surface;
use yir_core::{ExecutionState, InstructionSemantics, Node, RegisteredMod, Resource, Value};

pub struct ShaderMod;

impl RegisteredMod for ShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        describe_shader_node(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if let Some(value) = execute_shader_core_node(node, resource, state)? {
            return Ok(value);
        }

        if let Some(value) = execute_shader_effect_node(node, resource, state)? {
            return Ok(value);
        }

        Err(format!(
            "unknown shader instruction `{}`",
            node.op.instruction
        ))
    }
}
