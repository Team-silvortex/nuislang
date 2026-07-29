use super::*;

const SOURCE: &str = r#"
contract = "nuis-spirv-compute-source-v1"
module_lowering_plan_contract = "nuis-yir.shader.backend-lowering-plan.v1"
module_source_schema = "nuis-yir.shader.module-summary.v1"
module_lowering_boundary = "module-summary-to-native-ir"
module_profile_lowering_target = "vulkan.discrete-or-integrated-gpu"
module_lowering_target = "spirv:vulkan-gpu"
module_native_ir = "spirv1.6"
module_stage_kind = "compute"
module_execution_model = "GLCompute"
module_binding_slot_model = "descriptor-set-binding"
spirv_version = "1.6"
operation = "copy-u32"
entry = "nuis_vulkan_copy_u32"
local_size = "1x1x1"
descriptor_set = 0
input_binding = 0
output_binding = 1
"#;

const CANONICAL_WGSL: &str = r#"
binding(0, 0) var<storage, read> input_values: array<u32>;

binding(0, 1) var<storage, read_write> output_values: array<u32>;

stage compute(workgroup_size(1, 1, 1)) {
  fn nuis_vulkan_copy_u32(@builtin(global_invocation_id) gid: vec3<u32>) {
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

fn spirv_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}

#[test]
fn emits_deterministic_spirv_copy_module_without_external_tools() {
    let first = lower_registered_compute_source_for_profile(
        SOURCE.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    let repeated = lower_registered_compute_source_for_profile(
        SOURCE.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    assert_eq!(first, repeated);
    assert_eq!(first.len() % 4, 0);
    assert_eq!(
        u32::from_le_bytes(first[0..4].try_into().unwrap()),
        SPIRV_MAGIC
    );
    assert_eq!(
        u32::from_le_bytes(first[4..8].try_into().unwrap()),
        SPIRV_VERSION_1_6
    );
    assert!(first
        .windows("nuis_vulkan_copy_u32".len())
        .any(|window| window == b"nuis_vulkan_copy_u32"));
}

#[test]
fn rejects_entry_or_binding_drift() {
    assert!(lower_registered_compute_source_for_profile(
        SOURCE.as_bytes(),
        "other_entry",
        "vulkan.discrete-or-integrated-gpu"
    )
    .is_err());
    let duplicate_binding = SOURCE.replace("output_binding = 1", "output_binding = 0");
    assert!(lower_registered_compute_source_for_profile(
        duplicate_binding.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu"
    )
    .is_err());
}

#[test]
fn rejects_module_lowering_plan_drift() {
    let wrong_backend = SOURCE.replace(
        "module_lowering_target = \"spirv:vulkan-gpu\"",
        "module_lowering_target = \"msl:metal-gpu\"",
    );
    assert!(lower_registered_compute_source_for_profile(
        wrong_backend.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu"
    )
    .is_err());
    assert!(lower_registered_compute_source_for_profile(
        SOURCE.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.wrong-target"
    )
    .is_err());
}

#[test]
fn emits_registered_copy_module_from_canonical_wgsl_body() {
    let from_source = lower_registered_compute_source_for_profile(
        SOURCE.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    let from_wgsl = lower_canonical_inline_wgsl_u32_for_profile(
        CANONICAL_WGSL.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();

    assert_eq!(from_wgsl, from_source);
}

#[test]
fn emits_binary_u32_modules_from_canonical_wgsl_body() {
    let from_copy = lower_canonical_inline_wgsl_u32_for_profile(
        CANONICAL_WGSL.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    for (entry, operator, opcode) in [
        ("nuis_vulkan_add_u32", "+", 128_u16),
        ("nuis_vulkan_sub_u32", "-", 130_u16),
        ("nuis_vulkan_mul_u32", "*", 132_u16),
    ] {
        let source = canonical_binary_wgsl(entry, operator);
        let lowered = lower_canonical_inline_wgsl_u32_for_profile(
            source.as_bytes(),
            entry,
            "vulkan.discrete-or-integrated-gpu",
        )
        .unwrap();
        let words = spirv_words(&lowered);

        assert_ne!(lowered, from_copy);
        assert!(words.contains(&((5_u32 << 16) | u32::from(opcode))));
        assert!(lowered
            .windows(entry.len())
            .any(|window| window == entry.as_bytes()));
    }
}

#[test]
fn rejects_canonical_wgsl_body_drift() {
    let drifted = CANONICAL_WGSL.replace(
        "output_values[idx] = input_values[idx];",
        "output_values[idx] = 0u;",
    );
    assert!(lower_canonical_inline_wgsl_u32_for_profile(
        drifted.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu"
    )
    .is_err());
}
