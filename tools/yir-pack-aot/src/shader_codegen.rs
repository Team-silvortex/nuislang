mod expressions;
mod normalize;
mod syntax;
mod textures;

use self::expressions::{
    apply_backend_builtin_mapping, infer_backend_type, map_wgsl_expr_identifiers,
    map_wgsl_type_to_backend,
};
pub(super) use self::expressions::{compile_backend_expr, map_wgsl_expr_to_backend};
use crate::backend_descriptor::{render_shader_descriptor_lines, render_wgsl_comment_block};
use crate::shader_codegen::normalize::normalize_backend_expr;
use crate::shader_codegen::syntax::{replace_identifier_token, split_top_level_args};

pub(super) fn render_shader_glsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::OpenGl)
        .unwrap_or_else(default_glsl_vertex_body);
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::OpenGl, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"#version 460 core
// nuis shader backend scaffold
{descriptor}
{wgsl_comment}
//
// contract:
// {summary_comment}

#ifdef NUIS_STAGE_VERTEX
layout(location = 0) out vec2 v_uv;

void main() {{
{vertex_body}
}}
#endif

#ifdef NUIS_STAGE_FRAGMENT
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
{texture_uniforms}

void main() {{
{fragment_body}
}}
#endif
"#,
        descriptor = render_shader_descriptor_lines(variant, "// "),
        wgsl_comment = wgsl_comment,
        texture_uniforms = if uses_texture {
            "layout(binding = 2) uniform sampler2D u_texture0;"
        } else {
            ""
        },
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub(super) fn render_shader_metal_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::Metal)
        .unwrap_or_else(default_metal_vertex_body);
    let fragment_signature = if uses_texture {
        "fragment float4 frame_fs(VsOut in [[stage_in]], texture2d<float> tex [[texture(0)]], sampler samp [[sampler(0)]])"
    } else {
        "fragment float4 frame_fs(VsOut in [[stage_in]])"
    };
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::Metal, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"// nuis shader backend scaffold
{descriptor}
{wgsl_comment}
//
// contract:
// {summary_comment}

#include <metal_stdlib>
using namespace metal;

struct VsOut {{
    float4 position [[position]];
    float2 uv;
}};

vertex VsOut {entry}_vs(uint vid [[vertex_id]]) {{
    VsOut out;
{vertex_body}
    return out;
}}

{fragment_signature} {{
{fragment_body}
}}
"#,
        descriptor = render_shader_descriptor_lines(variant, "// "),
        entry = variant.entry,
        wgsl_comment = wgsl_comment,
        fragment_signature =
            fragment_signature.replace("frame_fs", &format!("{}_fs", variant.entry)),
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub(super) fn render_shader_hlsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::Hlsl)
        .unwrap_or_else(default_hlsl_vertex_body);
    let fragment_prelude = if uses_texture {
        "Texture2D shaderTexture : register(t0);\nSamplerState shaderSampler : register(s0);\n"
    } else {
        ""
    };
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::Hlsl, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"// nuis shader backend scaffold
{descriptor}
{wgsl_comment}
//
// contract:
// {summary_comment}

struct VsOut {{
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
}};

VsOut {entry}_vs(uint vid : SV_VertexID) {{
    VsOut outp;
{vertex_body}
    return outp;
}}

{fragment_prelude}
float4 {entry}_ps(VsOut input) : SV_Target0 {{
{fragment_body}
}}
"#,
        descriptor = render_shader_descriptor_lines(variant, "// "),
        entry = variant.entry,
        wgsl_comment = wgsl_comment,
        fragment_prelude = fragment_prelude,
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub(super) fn render_shader_vulkan_glsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let uses_texture = shader_stage_uses_texture(stage);
    let vertex_body = shader_vertex_body(stage, ShaderTargetFlavor::VulkanGlsl)
        .unwrap_or_else(default_vulkan_vertex_body);
    let fragment_body = shader_fragment_body(stage, ShaderTargetFlavor::VulkanGlsl, uses_texture);
    let wgsl_comment = render_wgsl_comment_block(
        stage.wgsl_entry.as_deref(),
        stage.wgsl_source.as_deref(),
        "// ",
    );
    format!(
        r#"#version 450
// nuis shader backend scaffold
{descriptor}
{wgsl_comment}
//
// contract:
// {summary_comment}

#ifdef NUIS_STAGE_VERTEX
layout(location = 0) out vec2 v_uv;

void main() {{
{vertex_body}
}}
#endif

#ifdef NUIS_STAGE_FRAGMENT
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;
{texture_uniforms}

void main() {{
{fragment_body}
}}
#endif
"#,
        descriptor = render_shader_descriptor_lines(variant, "// "),
        wgsl_comment = wgsl_comment,
        texture_uniforms = if uses_texture {
            "layout(set = 0, binding = 2) uniform sampler2D u_texture0;"
        } else {
            ""
        },
        vertex_body = vertex_body,
        fragment_body = fragment_body,
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub(super) fn render_shader_wgsl_scaffold(
    stage: &yir_lower_contract::ShaderStageContract,
    variant: &yir_lower_contract::ShaderBackendVariant,
    summary: &str,
) -> String {
    let source = stage.wgsl_source.as_deref().unwrap_or(
        "@vertex\nfn vs_main(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4<f32> {\n  let x = f32(i32(vid) - 1);\n  let y = select(-1.0, 1.0, vid == 2u);\n  return vec4<f32>(x, y, 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n  return vec4<f32>(1.0, 0.0, 1.0, 1.0);\n}\n",
    );
    format!(
        r#"// nuis shader backend scaffold
{descriptor}
// contract:
// {summary_comment}
//
// original_wgsl_begin
{source}
// original_wgsl_end
"#,
        descriptor = render_shader_descriptor_lines(variant, "// "),
        summary_comment = summary
            .lines()
            .map(|line| format!("// {line}"))
            .collect::<Vec<_>>()
            .join("\n"),
        source = source
    )
}

fn shader_stage_uses_texture(stage: &yir_lower_contract::ShaderStageContract) -> bool {
    let has_texture = stage
        .bindings
        .iter()
        .any(|binding| binding.kind == "texture_binding");
    let has_sampler = stage
        .bindings
        .iter()
        .any(|binding| binding.kind == "sampler_binding");
    has_texture && has_sampler
}

#[derive(Clone, Copy)]
pub(super) enum ShaderTargetFlavor {
    OpenGl,
    VulkanGlsl,
    Metal,
    Hlsl,
}

fn shader_vertex_body(
    stage: &yir_lower_contract::ShaderStageContract,
    flavor: ShaderTargetFlavor,
) -> Option<String> {
    let shader_ir = stage
        .shader_ir_stages
        .iter()
        .find(|shader_ir| shader_ir.stage == "vertex")?;
    let mut lines = Vec::new();
    for inst in &shader_ir.instructions {
        let expr = compile_backend_vertex_expr(&inst.expr, flavor)?;
        match inst.result.as_str() {
            "out.pos" => lines.push(vertex_output_assign("position", &expr, flavor)),
            "out.uv" => lines.push(vertex_output_assign("uv", &expr, flavor)),
            other => {
                let ty = inst
                    .ty
                    .as_deref()
                    .map(|ty| map_wgsl_type_to_backend(ty, flavor))
                    .unwrap_or_else(|| infer_backend_type(&inst.expr, flavor));
                lines.push(format!("    {ty} {other} = {expr};"));
            }
        }
    }
    Some(lines.join("\n"))
}

fn compile_backend_vertex_expr(expr: &str, flavor: ShaderTargetFlavor) -> Option<String> {
    let mapped = apply_backend_builtin_mapping(expr, flavor)
        .replace("f32(", "float(")
        .replace("1u", "1")
        .replace("2u", "2");
    let mapped = replace_identifier_token(
        &mapped,
        "vid",
        match flavor {
            ShaderTargetFlavor::OpenGl => "gl_VertexID",
            ShaderTargetFlavor::VulkanGlsl => "gl_VertexIndex",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "vid",
        },
    );
    Some(normalize_backend_expr(&mapped))
}

fn vertex_output_assign(field: &str, expr: &str, flavor: ShaderTargetFlavor) -> String {
    match (field, flavor) {
        ("position", ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl) => {
            format!("    gl_Position = {expr};")
        }
        ("uv", ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl) => {
            format!("    v_uv = {expr};")
        }
        ("position", ShaderTargetFlavor::Metal) => format!("    out.position = {expr};"),
        ("uv", ShaderTargetFlavor::Metal) => format!("    out.uv = {expr};"),
        ("position", ShaderTargetFlavor::Hlsl) => format!("    outp.position = {expr};"),
        ("uv", ShaderTargetFlavor::Hlsl) => format!("    outp.uv = {expr};"),
        _ => format!("    {field} = {expr};"),
    }
}

fn default_glsl_vertex_body() -> String {
    "    vec2 corners[4] = vec2[](\n        vec2(-1.0, -1.0),\n        vec2( 1.0, -1.0),\n        vec2(-1.0,  1.0),\n        vec2( 1.0,  1.0)\n    );\n    vec2 uvs[4] = vec2[](\n        vec2(0.0, 0.0),\n        vec2(1.0, 0.0),\n        vec2(0.0, 1.0),\n        vec2(1.0, 1.0)\n    );\n    int idx = min(gl_VertexID, 3);\n    gl_Position = vec4(corners[idx], 0.0, 1.0);\n    v_uv = uvs[idx];".to_owned()
}

fn default_vulkan_vertex_body() -> String {
    "    vec2 corners[4] = vec2[](\n        vec2(-1.0, -1.0),\n        vec2( 1.0, -1.0),\n        vec2(-1.0,  1.0),\n        vec2( 1.0,  1.0)\n    );\n    vec2 uvs[4] = vec2[](\n        vec2(0.0, 0.0),\n        vec2(1.0, 0.0),\n        vec2(0.0, 1.0),\n        vec2(1.0, 1.0)\n    );\n    int idx = min(gl_VertexIndex, 3);\n    gl_Position = vec4(corners[idx], 0.0, 1.0);\n    v_uv = uvs[idx];".to_owned()
}

fn default_metal_vertex_body() -> String {
    "    float2 corners[4] = {\n        float2(-1.0, -1.0),\n        float2( 1.0, -1.0),\n        float2(-1.0,  1.0),\n        float2( 1.0,  1.0),\n    };\n    float2 xy = corners[min(vid, 3u)];\n    out.position = float4(xy, 0.0, 1.0);\n    float2 uvs[4] = {\n        float2(0.0, 0.0),\n        float2(1.0, 0.0),\n        float2(0.0, 1.0),\n        float2(1.0, 1.0),\n    };\n    out.uv = uvs[min(vid, 3u)];".to_owned()
}

fn default_hlsl_vertex_body() -> String {
    "    float2 corners[4] = {\n        float2(-1.0, -1.0),\n        float2( 1.0, -1.0),\n        float2(-1.0,  1.0),\n        float2( 1.0,  1.0),\n    };\n    float2 xy = corners[min(vid, 3u)];\n    outp.position = float4(xy, 0.0, 1.0);\n    float2 uvs[4] = {\n        float2(0.0, 0.0),\n        float2(1.0, 0.0),\n        float2(0.0, 1.0),\n        float2(1.0, 1.0),\n    };\n    outp.uv = uvs[min(vid, 3u)];".to_owned()
}

fn shader_fragment_body(
    stage: &yir_lower_contract::ShaderStageContract,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> String {
    if let Some(shader_ir) = stage
        .shader_ir_stages
        .iter()
        .find(|shader_ir| shader_ir.stage == "fragment")
    {
        if let Some(mapped) = render_shader_ir_fragment(shader_ir, flavor, uses_texture) {
            return mapped;
        }
    }

    if let Some(wgsl) = stage.wgsl_source.as_deref() {
        if let Some(model) = extract_wgsl_fragment_model(wgsl) {
            if let Some(mapped) = render_wgsl_fragment_model(&model, flavor, uses_texture) {
                return mapped;
            }
        }
    }

    if let Some(mapped) = try_map_simple_wgsl_vec4_return(stage, flavor) {
        return mapped;
    }

    if let Some(wgsl) = stage.wgsl_source.as_deref() {
        if let Some(return_expr) = extract_wgsl_fragment_lowerable_expr(wgsl) {
            if let Some(mapped) = map_wgsl_expr_to_backend(&return_expr, flavor, uses_texture) {
                return mapped;
            }
        }
    }

    match (flavor, uses_texture) {
        (ShaderTargetFlavor::OpenGl, true) | (ShaderTargetFlavor::VulkanGlsl, true) => {
            "    vec4 sampled = texture(u_texture0, v_uv);\n    outColor = sampled;".to_owned()
        }
        (ShaderTargetFlavor::OpenGl, false) | (ShaderTargetFlavor::VulkanGlsl, false) => {
            "    outColor = vec4(0.82, 0.88, 0.97, 1.0);".to_owned()
        }
        (ShaderTargetFlavor::Metal, true) => "    return tex.sample(samp, in.uv);".to_owned(),
        (ShaderTargetFlavor::Metal, false) => {
            "    return float4(0.82, 0.88, 0.97, 1.0);".to_owned()
        }
        (ShaderTargetFlavor::Hlsl, true) => {
            "    return shaderTexture.Sample(shaderSampler, input.uv);".to_owned()
        }
        (ShaderTargetFlavor::Hlsl, false) => "    return float4(0.82, 0.88, 0.97, 1.0);".to_owned(),
    }
}

struct WgslFragmentBinding {
    name: String,
    ty: Option<String>,
    expr: String,
}

struct WgslFragmentModel {
    bindings: Vec<WgslFragmentBinding>,
    return_expr: String,
}

fn try_map_simple_wgsl_vec4_return(
    stage: &yir_lower_contract::ShaderStageContract,
    flavor: ShaderTargetFlavor,
) -> Option<String> {
    let wgsl = stage.wgsl_source.as_deref()?;
    let expr = extract_wgsl_fragment_lowerable_expr(wgsl)?;
    let inner = expr
        .strip_prefix("vec4<f32>(")
        .or_else(|| expr.strip_prefix("vec4("))?
        .strip_suffix(')')?;
    let comps = split_top_level_args(inner);
    if comps.len() != 4 {
        return None;
    }
    if comps
        .iter()
        .any(|component| !is_simple_wgsl_component_expr(component.trim()))
    {
        return None;
    }

    let mapped = comps
        .iter()
        .map(|component| map_wgsl_component_ref(component.trim(), flavor))
        .collect::<Vec<_>>()
        .join(", ");

    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            format!("    outColor = vec4({mapped});")
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            let ctor = match flavor {
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
                _ => "vec4",
            };
            format!("    return {ctor}({mapped});")
        }
    })
}

fn map_wgsl_component_ref(component: &str, flavor: ShaderTargetFlavor) -> String {
    map_wgsl_expr_identifiers(component, flavor)
}

fn is_simple_wgsl_component_expr(component: &str) -> bool {
    !component.contains('(') && !component.contains(')')
}

fn extract_wgsl_fragment_return_expr(wgsl: &str) -> Option<String> {
    let fragment_pos = wgsl.find("@fragment")?;
    let fragment_src = &wgsl[fragment_pos..];
    let return_pos = fragment_src.find("return")?;
    let after_return = &fragment_src[return_pos + "return".len()..];
    let semicolon_pos = after_return.find(';')?;
    Some(after_return[..semicolon_pos].trim().to_owned())
}

fn extract_wgsl_fragment_model(wgsl: &str) -> Option<WgslFragmentModel> {
    let fragment_pos = wgsl.find("@fragment")?;
    let fragment_src = &wgsl[fragment_pos..];
    let return_expr = extract_wgsl_fragment_return_expr(wgsl)?;
    let mut bindings = Vec::new();
    for raw_line in fragment_src.lines() {
        let line = raw_line.trim();
        if !line.starts_with("let ") {
            continue;
        }
        let Some(eq_pos) = line.find('=') else {
            continue;
        };
        let lhs = line["let ".len()..eq_pos].trim();
        let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
        if rhs.is_empty() {
            continue;
        }
        let (name, ty) = if let Some(colon_pos) = lhs.find(':') {
            (
                lhs[..colon_pos].trim().to_owned(),
                Some(lhs[colon_pos + 1..].trim().to_owned()),
            )
        } else {
            (lhs.to_owned(), None)
        };
        if name.is_empty() {
            continue;
        }
        bindings.push(WgslFragmentBinding {
            name,
            ty,
            expr: rhs.to_owned(),
        });
    }
    Some(WgslFragmentModel {
        bindings,
        return_expr,
    })
}

fn extract_wgsl_fragment_lowerable_expr(wgsl: &str) -> Option<String> {
    let return_expr = extract_wgsl_fragment_return_expr(wgsl)?;
    let fragment_pos = wgsl.find("@fragment")?;
    let fragment_src = &wgsl[fragment_pos..];
    let let_bindings = extract_wgsl_fragment_let_bindings(fragment_src);
    Some(expand_wgsl_expr_bindings(&return_expr, &let_bindings))
}

fn extract_wgsl_fragment_let_bindings(fragment_src: &str) -> Vec<(String, String)> {
    let mut bindings = Vec::new();
    for raw_line in fragment_src.lines() {
        let line = raw_line.trim();
        if !line.starts_with("let ") {
            continue;
        }
        let Some(eq_pos) = line.find('=') else {
            continue;
        };
        let lhs = line["let ".len()..eq_pos].trim();
        let name = lhs.split(':').next().map(str::trim).unwrap_or(lhs);
        if name.is_empty() {
            continue;
        }
        let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
        if rhs.is_empty() {
            continue;
        }
        bindings.push((name.to_owned(), rhs.to_owned()));
    }
    bindings
}

fn expand_wgsl_expr_bindings(expr: &str, bindings: &[(String, String)]) -> String {
    let mut expanded = expr.to_owned();
    for _ in 0..bindings.len() {
        let mut changed = false;
        for (name, value) in bindings {
            let next = replace_identifier_token(&expanded, name, &format!("({value})"));
            if next != expanded {
                expanded = next;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    expanded
}

fn render_wgsl_fragment_model(
    model: &WgslFragmentModel,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    let mut lines = Vec::new();
    for binding in &model.bindings {
        let ty = binding
            .ty
            .as_deref()
            .map(|ty| map_wgsl_type_to_backend(ty, flavor))
            .unwrap_or_else(|| infer_backend_type(&binding.expr, flavor));
        let expr = compile_backend_expr(&binding.expr, flavor, uses_texture)?;
        lines.push(format!("    {ty} {} = {expr};", binding.name));
    }
    let return_expr = compile_backend_expr(&model.return_expr, flavor, uses_texture)?;
    match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            lines.push(format!("    outColor = {return_expr};"));
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            lines.push(format!("    return {return_expr};"));
        }
    }
    Some(lines.join("\n"))
}

fn render_shader_ir_fragment(
    shader_ir: &yir_lower_contract::NustarContractStage,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    if shader_ir.stage != "fragment" {
        return None;
    }

    let mut lines = Vec::new();
    for inst in &shader_ir.instructions {
        let ty = inst
            .ty
            .as_deref()
            .map(|ty| map_wgsl_type_to_backend(ty, flavor))
            .unwrap_or_else(|| infer_backend_type(&inst.expr, flavor));
        let expr = compile_backend_expr(&inst.expr, flavor, uses_texture)?;
        lines.push(format!("    {ty} {} = {expr};", inst.result));
    }

    let return_expr = compile_backend_expr(&shader_ir.terminator.expr, flavor, uses_texture)?;
    match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            lines.push(format!("    outColor = {return_expr};"));
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            lines.push(format!("    return {return_expr};"));
        }
    }

    Some(lines.join("\n"))
}
