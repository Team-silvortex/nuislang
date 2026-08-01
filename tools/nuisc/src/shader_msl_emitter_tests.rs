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

fn canonical_pair_add_wgsl(entry: &str) -> String {
    format!(
        r#"
binding(0, 0) var<storage, read> left_values: array<u32>;

binding(0, 1) var<storage, read> right_values: array<u32>;

binding(0, 2) var<storage, read_write> output_values: array<u32>;

stage compute(workgroup_size(1, 1, 1)) {{
  fn {entry}(@builtin(global_invocation_id) gid: vec3<u32>) {{
    let idx: u32 = gid.x;
    output_values[idx] = left_values[idx] + right_values[idx];
  }}
}}
"#
    )
}

fn canonical_pair_fan_out_wgsl(entry: &str) -> String {
    format!(
        r#"
binding(0, 0) var<storage, read> left_values: array<u32>;

binding(0, 1) var<storage, read> right_values: array<u32>;

binding(0, 2) var<storage, read_write> sum_values: array<u32>;

binding(0, 3) var<storage, read_write> xor_values: array<u32>;

stage compute(workgroup_size(1, 1, 1)) {{
  fn {entry}(@builtin(global_invocation_id) gid: vec3<u32>) {{
    let idx: u32 = gid.x;
    sum_values[idx] = left_values[idx] + right_values[idx];
    xor_values[idx] = left_values[idx] ^ right_values[idx];
  }}
}}
"#
    )
}

fn assert_msl_plan_proof(text: &str, profile: &str) {
    assert!(text.contains(
        "// nuis-module-lowering-plan contract=nuis-yir.shader.backend-lowering-plan.v1"
    ));
    assert!(text.contains("// nuis-module-source-schema nuis-yir.shader.module-summary.v1"));
    assert!(text.contains("// nuis-module-lowering-boundary module-summary-to-native-ir"));
    assert!(text.contains(&format!("// nuis-module-profile-lowering-target {profile}")));
    assert!(text.contains("// nuis-module-lowering-target msl:metal-gpu"));
    assert!(text.contains("// nuis-module-native-ir msl2.4"));
    assert!(text.contains(
        "// nuis-module-stage kind=compute execution_model=kernel binding_slot_model=argument-buffer-slot"
    ));
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

    assert_msl_plan_proof(&text, "metal.apple-silicon-gpu");
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
        ("nuis_metal_xor_u32", "^", "value ^ value"),
    ] {
        let source = canonical_binary_wgsl(entry, operator);
        let lowered = lower_canonical_inline_wgsl_u32_for_profile(
            source.as_bytes(),
            entry,
            "metal.mac-discrete-or-integrated-gpu",
        )
        .unwrap();
        let text = String::from_utf8(lowered).unwrap();

        assert_msl_plan_proof(&text, "metal.mac-discrete-or-integrated-gpu");
        assert!(text.contains(&format!("kernel void {entry}(")));
        assert!(text.contains(&format!("output_values[gid] = {expression};")));
    }
}

#[test]
fn emits_pair_add_u32_msl_from_shared_canonical_body_contract() {
    let entry = "nuis_metal_add_pair_u32";
    let source = canonical_pair_add_wgsl(entry);
    let lowered = lower_canonical_inline_wgsl_u32_for_profile(
        source.as_bytes(),
        entry,
        "metal.apple-silicon-gpu",
    )
    .unwrap();
    let text = String::from_utf8(lowered).unwrap();

    assert_msl_plan_proof(&text, "metal.apple-silicon-gpu");
    assert!(text.contains("kernel void nuis_metal_add_pair_u32("));
    assert!(text.contains("device const uint* input_values [[buffer(0)]]"));
    assert!(text.contains("device const uint* right_values [[buffer(1)]]"));
    assert!(text.contains("device uint* output_values [[buffer(2)]]"));
    assert!(text.contains("uint rhs = right_values[gid];"));
    assert!(text.contains("output_values[gid] = value + rhs;"));
}

#[test]
fn emits_ordered_multi_output_u32_msl() {
    let entry = "nuis_metal_add_xor_pair_u32";
    let source = canonical_pair_fan_out_wgsl(entry);
    let lowered = lower_canonical_inline_wgsl_u32_for_profile(
        source.as_bytes(),
        entry,
        "metal.apple-silicon-gpu",
    )
    .unwrap();
    let text = String::from_utf8(lowered).unwrap();

    assert!(text.contains("device uint* output_values [[buffer(2)]]"));
    assert!(text.contains("device uint* output_values_1 [[buffer(3)]]"));
    assert!(text.contains("output_values[gid] = value + rhs;"));
    assert!(text.contains("output_values_1[gid] = value ^ rhs;"));
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

#[test]
fn rejects_msl_module_lowering_plan_drift() {
    let mut plan = canonical_msl_compute_plan("metal.apple-silicon-gpu");
    plan.lowering_target = "spirv:vulkan-gpu";
    assert!(validate_msl_module_lowering_plan(&plan, "metal.apple-silicon-gpu").is_err());

    let mut plan = canonical_msl_compute_plan("metal.apple-silicon-gpu");
    plan.binding_slot_model = "descriptor-set-binding";
    assert!(validate_msl_module_lowering_plan(&plan, "metal.apple-silicon-gpu").is_err());

    let plan = canonical_msl_compute_plan("metal.apple-silicon-gpu");
    assert!(
        validate_msl_module_lowering_plan(&plan, "metal.mac-discrete-or-integrated-gpu").is_err()
    );
}
