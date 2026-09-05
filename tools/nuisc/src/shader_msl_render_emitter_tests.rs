use super::*;

const RASTER_WGSL: &str = r#"
struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {
  var out: VsOut;
  let x: f32 = f32((vid << 1u) & 2u);
  let y: f32 = f32(vid & 2u);
  out.pos = vec4<f32>(x * 2.0 - 1.0, y * -2.0 + 1.0, 0.0, 1.0);
  out.uv = vec2<f32>(x, y);
  return out;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
  let luma: f32 = 0.18 + uv.x * 0.37 + uv.y * 0.23;
  let glow: f32 = 0.08 + uv.x * 0.11;
  return vec4<f32>(luma, luma + glow, luma, 1.0);
}
"#;

#[test]
fn rejects_vertex_semantics_that_the_canonical_emitter_would_discard() {
    for source in [
        RASTER_WGSL.replace("x * 2.0 - 1.0", "x * 0.5 - 1.0"),
        RASTER_WGSL.replace("vec2<f32>(x, y)", "vec2<f32>(y, x)"),
        RASTER_WGSL.replace("return out;", "out.pos.x = 0.0; return out;"),
    ] {
        let error = lower_canonical_inline_wgsl_render_for_profile(
            &source,
            "demo",
            "metal.apple-silicon-gpu",
        )
        .unwrap_err();
        assert!(error.contains("refusing to substitute fullscreen geometry"));
    }
}

#[test]
fn canonical_vertex_comments_do_not_change_lowering_or_fake_entry_detection() {
    let commented = format!(
        "// fn vs_main {{ discarded comment }}\n{}",
        RASTER_WGSL.replace(
            "return out;",
            "/* nested /* comment */ supported */ return out;"
        )
    );
    assert_eq!(
        lower_canonical_inline_wgsl_render_for_profile(
            &commented,
            "demo",
            "metal.apple-silicon-gpu"
        )
        .unwrap(),
        lower_canonical_inline_wgsl_render_for_profile(
            RASTER_WGSL,
            "demo",
            "metal.apple-silicon-gpu"
        )
        .unwrap()
    );
}

#[test]
fn rejects_vertex_input_semantics_drift() {
    let source = RASTER_WGSL.replace("builtin(vertex_index)", "builtin(instance_index)");
    assert!(lower_canonical_inline_wgsl_render_for_profile(
        &source,
        "demo",
        "metal.apple-silicon-gpu"
    )
    .unwrap_err()
    .contains("expected vertex_index input"));
}

#[test]
fn lowers_canonical_fullscreen_wgsl_to_msl() {
    let module = lower_canonical_inline_wgsl_render_for_profile(
        RASTER_WGSL,
        "pixelmagic_render_demo",
        "metal.apple-silicon-gpu",
    )
    .unwrap();

    assert_eq!(module.vertex_entry, "vs_main");
    assert_eq!(module.fragment_entry, "fs_main");
    assert!(module.source.contains("vertex NuisRasterOut vs_main"));
    assert!(module.source.contains("fragment float4 fs_main"));
    assert!(module
        .source
        .contains("float luma = 0.18 + uv.x * 0.37 + uv.y * 0.23;"));
    assert!(module
        .source
        .contains("return float4(luma, luma + glow, luma, 1.0);"));
}

#[test]
fn rejects_unregistered_target_and_nested_fragment_control_flow() {
    assert!(lower_canonical_inline_wgsl_render_for_profile(
        RASTER_WGSL,
        "demo",
        "vulkan.discrete-or-integrated-gpu"
    )
    .unwrap_err()
    .contains("unsupported"));

    let nested = RASTER_WGSL.replace(
        "return vec4<f32>(luma, luma + glow, luma, 1.0);",
        "if uv.x > 0.5 { return vec4<f32>(luma, luma, luma, 1.0); }",
    );
    assert!(lower_canonical_inline_wgsl_render_for_profile(
        &nested,
        "demo",
        "metal.apple-silicon-gpu"
    )
    .unwrap_err()
    .contains("nested blocks"));
}

#[test]
fn lowers_one_typed_fragment_uniform_into_content_bound_msl_reflection() {
    let source = format!("@group(0) @binding(2) var<uniform> tint: vec4<f32>;\n{RASTER_WGSL}")
        .replace(
            "vec4<f32>(luma, luma + glow, luma, 1.0)",
            "vec4<f32>(luma * tint.x, luma * tint.y, luma * tint.z, tint.w)",
        );
    let lowered =
        lower_canonical_inline_wgsl_render_for_profile(&source, "demo", "metal.apple-silicon-gpu")
            .unwrap();
    assert!(lowered
        .source
        .contains("constant float4& tint [[buffer(2)]]"));
    assert!(lowered.source.contains("luma * tint.x"));
    assert_eq!(
        yir_domain_shader::fragment_uniform_capability(&lowered.source).unwrap(),
        Some(2)
    );
    for invalid in [
        source.replace("@group(0)", "@group(1)"),
        source.replace("@binding(2)", "@binding(31)"),
        source.replace("var<uniform>", "var<storage, read_write>"),
        source.replace("tint: vec4<f32>;", "tint: vec4<u32>;"),
        source.replace("tint: vec4<f32>;", "tint: vec4<f32> = vec4<f32>(1.0);"),
        format!("@group(0) @binding(3) var<uniform> other: vec4<f32>;\n{source}"),
    ] {
        assert!(lower_canonical_inline_wgsl_render_for_profile(
            &invalid,
            "demo",
            "metal.apple-silicon-gpu"
        )
        .is_err());
    }
}

#[test]
fn native_binding_normalization_is_idempotent_before_resource_reflection() {
    let source = format!("binding(0, 2) var<uniform> tint: vec4<f32>;\n{RASTER_WGSL}");
    let normalized = crate::shader_source::normalize_inline_wgsl_source(&source).unwrap();
    assert_eq!(
        crate::shader_source::normalize_inline_wgsl_source(&normalized).unwrap(),
        normalized
    );
    let lowered =
        lower_canonical_inline_wgsl_render_for_profile(&source, "demo", "metal.apple-silicon-gpu")
            .unwrap();
    assert_eq!(
        yir_domain_shader::fragment_uniform_capability(&lowered.source).unwrap(),
        Some(2)
    );
}

#[test]
fn lowers_read_only_storage_with_checked_gpu_indexing_and_fixed_layout_reflection() {
    let source = format!("binding(0, 3) var<storage, read> pixels: array<u32, 768>;\n{RASTER_WGSL}")
        .replace("let luma: f32 = 0.18 + uv.x * 0.37 + uv.y * 0.23;",
            "let index: u32 = u32(uv.x * 32.0); let pixel: u32 = pixels[index]; let luma: f32 = f32(pixel & 255u) / 255.0;");
    let lowered =
        lower_canonical_inline_wgsl_render_for_profile(&source, "image", "metal.apple-silicon-gpu")
            .unwrap();
    assert_eq!(
        yir_domain_shader::fragment_storage_capability(&lowered.source).unwrap(),
        Some(yir_domain_shader::ShaderFragmentStorageCapability {
            slot: 3,
            element_count: 768
        })
    );
    assert!(lowered
        .source
        .contains("const device NuisFragmentStorage& pixels [[buffer(3)]]"));
    assert!(lowered.source.contains("uint values[768]"));
    assert!(lowered
        .source
        .contains("index < 768u ? buffer.values[index] : 0u"));
    assert!(lowered.source.contains("nuis_storage_read(pixels, index)"));
    for invalid in [
        source.replace("storage, read", "storage, read_write"),
        source.replace("array<u32, 768>", "array<u32>"),
        source.replace("array<u32, 768>", "array<f32, 768>"),
        source.replace("array<u32, 768>", "array<u32, 4194305>"),
        source.replace("pixels[index]", "other[index]"),
        source.replace("pixels[index]", "pixels[pixels[index]]"),
    ] {
        assert!(lower_canonical_inline_wgsl_render_for_profile(
            &invalid,
            "image",
            "metal.apple-silicon-gpu"
        )
        .is_err());
    }
}
