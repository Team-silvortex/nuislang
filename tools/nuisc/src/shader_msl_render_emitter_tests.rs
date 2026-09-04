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
