use super::*;

const CANONICAL_WGSL: &str = r#"
binding(0, 0) var<storage, read> input_values: array<u32>;

binding(0, 1) var<storage, read_write> output_values: array<u32>;

stage compute(workgroup_size(1, 1, 1)) {
  fn nuis_metal_copy_u32(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx: u32 = gid.x;
    output_values[idx] = input_values[idx];
  }
}
"#;

fn canonical_binary_wgsl(entry: &str, operator: &str) -> String {
    format!(
        r#"
binding(0, 0) var<storage, read> input_values: array<u32>;

binding(0, 1) var<storage, read_write> output_values: array<u32>;

stage compute(workgroup_size(1, 1, 1)) {{
  fn {entry}(@builtin(global_invocation_id) gid: vec3<u32>) {{
    let idx: u32 = gid.x;
    output_values[idx] = input_values[idx] {operator} input_values[idx];
  }}
}}
"#
    )
}

#[test]
fn emits_msl_copy_module_from_canonical_wgsl_body() {
    let lowered = lower_canonical_inline_wgsl_u32_for_profile(
        CANONICAL_WGSL.as_bytes(),
        "nuis_metal_copy_u32",
        "metal.apple-silicon-gpu",
    )
    .unwrap();
    let text = String::from_utf8(lowered).unwrap();

    assert!(text.contains("#include <metal_stdlib>"));
    assert!(text.contains("kernel void nuis_metal_copy_u32("));
    assert!(text.contains("device const uint* input_values [[buffer(0)]]"));
    assert!(text.contains("device uint* output_values [[buffer(1)]]"));
    assert!(text.contains("output_values[gid] = value;"));
}

#[test]
fn emits_binary_u32_msl_from_shared_canonical_body_contract() {
    for (entry, operator, expression) in [
        ("nuis_metal_add_u32", "+", "value + value"),
        ("nuis_metal_sub_u32", "-", "value - value"),
        ("nuis_metal_mul_u32", "*", "value * value"),
    ] {
        let source = canonical_binary_wgsl(entry, operator);
        let lowered = lower_canonical_inline_wgsl_u32_for_profile(
            source.as_bytes(),
            entry,
            "metal.mac-discrete-or-integrated-gpu",
        )
        .unwrap();
        let text = String::from_utf8(lowered).unwrap();

        assert!(text.contains(&format!("kernel void {entry}(")));
        assert!(text.contains(&format!("output_values[gid] = {expression};")));
    }
}

#[test]
fn rejects_non_metal_target_or_body_drift() {
    assert!(lower_canonical_inline_wgsl_u32_for_profile(
        CANONICAL_WGSL.as_bytes(),
        "nuis_metal_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .is_err());

    let drifted = CANONICAL_WGSL.replace(
        "output_values[idx] = input_values[idx];",
        "output_values[idx] = 0u;",
    );
    assert!(lower_canonical_inline_wgsl_u32_for_profile(
        drifted.as_bytes(),
        "nuis_metal_copy_u32",
        "metal.apple-silicon-gpu",
    )
    .is_err());
}
