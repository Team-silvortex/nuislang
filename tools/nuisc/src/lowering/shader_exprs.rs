use super::*;

#[path = "shader_exprs_profile.rs"]
mod shader_exprs_profile;
#[path = "shader_exprs_resources.rs"]
mod shader_exprs_resources;
#[path = "shader_exprs_runtime.rs"]
mod shader_exprs_runtime;

use shader_exprs_profile::{
    lower_shader_profile_color_seed, lower_shader_profile_radius_seed, lower_shader_profile_render,
    lower_shader_profile_speed_seed,
};
use shader_exprs_resources::{
    lower_shader_inline_wgsl, lower_shader_pipeline, lower_shader_sampler, lower_shader_target,
    lower_shader_texture2d, lower_shader_uv, lower_shader_viewport,
};
use shader_exprs_runtime::{
    lower_shader_begin_pass, lower_shader_bind_set, lower_shader_binding,
    lower_shader_draw_instanced, lower_shader_sample, lower_shader_sample_uv,
};

pub(super) fn lower_shader_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Option<Result<String, String>> {
    match expr {
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => Some(
            lower_shader_profile_color_seed(unit, base, delta, state, bindings),
        ),
        NirExpr::ShaderProfileSpeedSeed {
            unit,
            delta,
            scale,
            base,
        } => Some(lower_shader_profile_speed_seed(
            unit, delta, scale, base, state, bindings,
        )),
        NirExpr::ShaderProfileRadiusSeed { unit, base, delta } => Some(
            lower_shader_profile_radius_seed(unit, base, delta, state, bindings),
        ),
        NirExpr::ShaderProfileRender { unit, packet } => {
            Some(lower_shader_profile_render(unit, packet, state, bindings))
        }
        NirExpr::ShaderProfileTargetRef { unit } => {
            Some(lower_project_profile_ref(state, "shader", unit, "target"))
        }
        NirExpr::ShaderProfileViewportRef { unit } => {
            Some(lower_project_profile_ref(state, "shader", unit, "viewport"))
        }
        NirExpr::ShaderProfilePipelineRef { unit } => {
            Some(lower_project_profile_ref(state, "shader", unit, "pipeline"))
        }
        NirExpr::ShaderProfileVertexCountRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "vertex_count",
        )),
        NirExpr::ShaderProfileInstanceCountRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "instance_count",
        )),
        NirExpr::ShaderProfilePacketColorSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_color_slot",
        )),
        NirExpr::ShaderProfilePacketSpeedSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_speed_slot",
        )),
        NirExpr::ShaderProfilePacketRadiusSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_radius_slot",
        )),
        NirExpr::ShaderProfileSliderColorSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "slider_color_slot",
        )),
        NirExpr::ShaderProfileSliderSpeedSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "slider_speed_slot",
        )),
        NirExpr::ShaderProfileSliderRadiusSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "slider_radius_slot",
        )),
        NirExpr::ShaderProfileHeaderAccentSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "header_accent_slot",
        )),
        NirExpr::ShaderProfileToggleLiveSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "toggle_live_slot",
        )),
        NirExpr::ShaderProfileFocusSlotRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "focus_slot",
        )),
        NirExpr::ShaderProfilePacketTagRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_tag",
        )),
        NirExpr::ShaderProfileMaterialModeRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "material_mode",
        )),
        NirExpr::ShaderProfilePassKindRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "pass_kind",
        )),
        NirExpr::ShaderProfilePacketFieldCountRef { unit } => Some(lower_project_profile_ref(
            state,
            "shader",
            unit,
            "packet_field_count",
        )),
        NirExpr::ShaderTarget {
            format,
            width,
            height,
        } => Some(Ok(lower_shader_target(format, *width, *height, state))),
        NirExpr::ShaderViewport { width, height } => {
            Some(Ok(lower_shader_viewport(*width, *height, state)))
        }
        NirExpr::ShaderPipeline {
            name: pipe_name,
            topology,
        } => Some(Ok(lower_shader_pipeline(pipe_name, topology, state))),
        NirExpr::ShaderTexture2d {
            format,
            width,
            height,
            texels,
        } => Some(Ok(lower_shader_texture2d(
            format, *width, *height, texels, state,
        ))),
        NirExpr::ShaderSampler {
            filter,
            address_mode,
        } => Some(Ok(lower_shader_sampler(filter, address_mode, state))),
        NirExpr::ShaderUv { u, v } => Some(Ok(lower_shader_uv(*u, *v, state))),
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            mode,
        } => Some(lower_shader_sample(
            texture, sampler, x, y, *mode, state, bindings,
        )),
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            mode,
        } => Some(lower_shader_sample_uv(
            texture, sampler, uv, *mode, state, bindings,
        )),
        NirExpr::ShaderBinding {
            kind,
            slot,
            layout,
            profile_contract,
            value,
        } => Some(lower_shader_binding(
            kind,
            *slot,
            layout.as_deref(),
            profile_contract.as_deref(),
            value,
            state,
            bindings,
        )),
        NirExpr::ShaderBindSet {
            pipeline,
            bindings: set_bindings,
        } => Some(lower_shader_bind_set(
            pipeline,
            set_bindings,
            state,
            bindings,
        )),
        NirExpr::ShaderInlineWgsl { entry, source } => {
            Some(lower_shader_inline_wgsl(entry, source, state))
        }
        NirExpr::ShaderResult { value, state: flow } => Some(lower_result_observe_node(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            value,
            "shader_result",
            flow.render(),
        )),
        NirExpr::ShaderPassReady(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_pass_ready",
            "is_pass_ready",
        )),
        NirExpr::ShaderFrameReady(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_frame_ready",
            "is_frame_ready",
        )),
        NirExpr::ShaderValue(result) => Some(lower_result_unary_value_effect(
            state,
            bindings,
            ResultLoweringDomain::Shader,
            result,
            "shader_value",
            "value",
        )),
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => Some(lower_shader_begin_pass(
            target, pipeline, viewport, state, bindings,
        )),
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => Some(lower_shader_draw_instanced(
            pass,
            packet,
            vertex_count,
            instance_count,
            state,
            bindings,
        )),
        _ => None,
    }
}
