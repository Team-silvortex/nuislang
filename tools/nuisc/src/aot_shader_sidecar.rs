use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_profile::{
    derived_lowering_profile_for_unit, render_target_specific_lowering_fields,
    shader_supported_stages_for_profile,
};
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn render_domain_build_unit_shader_ir_sidecar(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let profile = derived_lowering_profile_for_unit(unit);
    let supported_stages = shader_supported_stages_for_profile(unit, &profile).unwrap_or(&[]);
    let has_stage = |stage: &str| supported_stages.contains(&stage);
    let mut out = String::new();
    out.push_str("schema = \"nuis-shader-ir-sidecar-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    if !supported_stages.is_empty() {
        out.push_str(&format!(
            "supported_stages = {}\n",
            render_string_array(
                &supported_stages
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    match profile.profile_key {
        "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.msl\"\n");
            out.push_str("entry_symbol = \"main0\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"argument-buffer\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("threadgroup_topology = \"tile\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"material.uniforms, frame.texture0\"\n");
            out.push_str("push_constants = \"fragment.params\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main0\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            out.push_str("header = \"#include <metal_stdlib>\\nusing namespace metal;\"\n");
            if has_stage("vertex") {
                out.push_str(
                    "vertex_body = \"vertex float4 vs_main(uint vid [[vertex_id]]) {\\n    return float4(float(vid & 1), float((vid >> 1) & 1), 0.0, 1.0);\\n}\"\n",
                );
            }
            if has_stage("fragment") {
                out.push_str(
                    "body = \"fragment float4 main0(float2 uv [[stage_in]]) {\\n    return float4(uv.x, uv.y, 0.0, 1.0);\\n}\"\n",
                );
            }
            if has_stage("compute") {
                out.push_str(
                    "compute_body = \"kernel void cs_main(uint2 gid [[thread_position_in_grid]]) {\\n    (void)gid;\\n}\"\n",
                );
            }
        }
        "vulkan.discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.spirv\"\n");
            out.push_str("entry_symbol = \"main\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"descriptor-set\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("threadgroup_topology = \"quad-fragment\"\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"set0.binding0.texture, set0.binding1.sampler\"\n");
            out.push_str("push_constants = \"fragment.params\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            out.push_str("capabilities = \"OpCapability Shader\"\n");
            if has_stage("vertex") {
                out.push_str("vertex_body = \"OpEntryPoint Vertex %vs_main \\\"vs_main\\\"\"\n");
            }
            if has_stage("fragment") {
                out.push_str(
                    "body = \"OpMemoryModel Logical GLSL450\\nOpEntryPoint Fragment %main \\\"main\\\"\\n%main = OpFunction %void None %fn\\nOpFunctionEnd\"\n",
                );
            }
            if has_stage("compute") {
                out.push_str(
                    "compute_body = \"OpEntryPoint GLCompute %cs_main \\\"cs_main\\\"\"\n",
                );
            }
        }
        "directx.discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.dxil\"\n");
            out.push_str("entry_symbol = \"main\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"root-signature\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"t0.texture, s0.sampler\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            if has_stage("vertex") {
                out.push_str("vertex_body = \"float4 vs_main(uint vid : SV_VertexID) : SV_Position { return float4(0, 0, 0, 1); }\"\n");
            }
            if has_stage("fragment") {
                out.push_str(
                    "body = \"float4 main() : SV_Target0 { return float4(0, 0, 0, 1); }\"\n",
                );
            }
            if has_stage("compute") {
                out.push_str("compute_body = \"[numthreads(8,8,1)] void cs_main(uint3 tid : SV_DispatchThreadID) { }\"\n");
            }
        }
        "opengl.discrete-or-integrated-gpu" => {
            out.push_str("ir_container = \"text.glsl\"\n");
            out.push_str("entry_symbol = \"main\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"uniform-slots\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"rgba8unorm\"]\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"sampler0, uniform0\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_main\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"main\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_main\"\n");
            }
            out.push_str("[source_stub]\n");
            out.push_str("header = \"#version 460 core\"\n");
            if has_stage("vertex") {
                out.push_str("vertex_body = \"void vs_main() { gl_Position = vec4(0.0, 0.0, 0.0, 1.0); }\"\n");
            }
            if has_stage("fragment") {
                out.push_str("body = \"out vec4 fragColor;\\nvoid main() { fragColor = vec4(0.0, 0.0, 0.0, 1.0); }\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute_body = \"layout(local_size_x = 8, local_size_y = 8) in;\\nvoid cs_main() { }\"\n");
            }
        }
        "cpu-fallback.cpu-host" => {
            out.push_str("ir_container = \"text.host-simd\"\n");
            out.push_str("entry_symbol = \"shade_stub\"\n");
            out.push_str("stage_kind = \"fragment\"\n");
            out.push_str("resource_layout = \"host-slices\"\n");
            out.push_str("[pipeline_layout]\n");
            out.push_str("color_targets = [\"host-rgba8\"]\n");
            out.push_str("[resource_bindings]\n");
            out.push_str("binding_table = \"tile.buffer, material.slice\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"vs_stub\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"shade_stub\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"cs_stub\"\n");
            }
            out.push_str("[source_stub]\n");
            if has_stage("vertex") {
                out.push_str(
                    "vertex_body = \"fn vs_stub(vid: u32) -> (f32, f32) { (vid as f32, 0.0) }\"\n",
                );
            }
            if has_stage("fragment") {
                out.push_str("body = \"fn shade_stub(tile: u32) -> u32 { tile }\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute_body = \"fn cs_stub(group: u32) -> u32 { group }\"\n");
            }
        }
        _ => {
            out.push_str("ir_container = \"text.unknown\"\n");
            out.push_str("entry_symbol = \"unimplemented\"\n");
            out.push_str("[entry_points]\n");
            if has_stage("vertex") {
                out.push_str("vertex = \"unimplemented\"\n");
            }
            if has_stage("fragment") {
                out.push_str("fragment = \"unimplemented\"\n");
            }
            if has_stage("compute") {
                out.push_str("compute = \"unimplemented\"\n");
            }
            out.push_str("[source_stub]\n");
            if has_stage("fragment") {
                out.push_str("body = \"unimplemented\"\n");
            }
        }
    }
    out
}
