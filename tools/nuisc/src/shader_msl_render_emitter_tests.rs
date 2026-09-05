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
