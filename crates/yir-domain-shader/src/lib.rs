mod ball_packet;
mod ball_packet_from_parts;
mod control_panel_extended_summary;
mod control_panel_layout;
mod control_panel_summary;
mod control_panel_surface;
mod control_panel_widgets;
mod describe;
mod draw_request;
mod execute_core;
mod execute_effects;
mod flow_state;
mod fragment_uniform;
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
pub use draw_request::{ShaderDrawArguments, ShaderDrawDescriptor, SHADER_UNBOUND_DRAW_CONTRACT};
use execute_core::execute_shader_core_node;
use execute_effects::execute_shader_effect_node;
use flow_state::parse_shader_flow_state;
pub use fragment_uniform::{
    fragment_uniform_capability, ShaderFragmentUniform, FRAGMENT_UNIFORM_CAPABILITY_MARKER,
    SHADER_FRAGMENT_UNIFORM_CONTRACT,
};
use parse_ball_packet::parse_ball_packet;
use yir_core::{
    ExecutionState, InstructionSemantics, Node, ProviderCompletionRegistration, RegisteredMod,
    Resource, Value, YirResultFamily,
};

pub struct ShaderMod;

impl RegisteredMod for ShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn provider_completion_registration(
        &self,
        node: &Node,
    ) -> Option<ProviderCompletionRegistration> {
        matches!(
            node.op.instruction.as_str(),
            "begin_pass" | "draw_instanced"
        )
        .then_some(ProviderCompletionRegistration::new(
            YirResultFamily::Shader,
            "shader.clock.frame.v1",
        ))
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
