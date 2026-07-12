use yir_core::ShaderFlowState;

pub(crate) fn parse_shader_flow_state(raw: &str) -> Result<ShaderFlowState, String> {
    match raw {
        "pass_ready" => Ok(ShaderFlowState::PassReady),
        "frame_ready" => Ok(ShaderFlowState::FrameReady),
        other => Err(format!("unknown shader flow state `{other}`")),
    }
}
