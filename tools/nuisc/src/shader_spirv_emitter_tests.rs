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

const PAIR_ADD_SOURCE: &str = r#"
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
operation = "add-pair-u32"
entry = "nuis_vulkan_add_pair_u32"
local_size = "1x1x1"
descriptor_set = 0
input_binding = 0
aux_input_binding = 1
output_binding = 2
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

fn canonical_pair_reduced_fan_out_wgsl(entry: &str) -> String {
    canonical_pair_fan_out_wgsl(entry)
        .replace("xor_values: array<u32>;", "xor_values: array<u32, 2>;")
}

fn spirv_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}

fn has_instruction(words: &[u32], opcode: u16, operands: &[u32]) -> bool {
    let mut cursor = 5;
    while cursor < words.len() {
        let instruction = words[cursor];
        let word_count = usize::try_from(instruction >> 16).unwrap_or(0);
        if word_count == operands.len() + 1
            && instruction as u16 == opcode
            && &words[cursor + 1..cursor + word_count] == operands
        {
            return true;
        }
        if word_count == 0 {
            return false;
        }
        cursor += word_count;
    }
    false
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
fn emits_registered_add_pair_module_with_aux_input_binding() {
    let lowered = lower_registered_compute_source_for_profile(
        PAIR_ADD_SOURCE.as_bytes(),
        "nuis_vulkan_add_pair_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    let words = spirv_words(&lowered);

    assert_eq!(words[3], 25);
    assert!(has_instruction(&words, 71, &[19, 33, 1]));
    assert!(has_instruction(&words, 71, &[22, 33, 2]));
    assert!(has_instruction(&words, 128, &[3, 24, 18, 21]));
}

#[test]
fn rejects_add_pair_source_without_aux_input_binding() {
    let missing_aux = PAIR_ADD_SOURCE
        .lines()
        .filter(|line| !line.trim_start().starts_with("aux_input_binding"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(lower_registered_compute_source_for_profile(
        missing_aux.as_bytes(),
        "nuis_vulkan_add_pair_u32",
        "vulkan.discrete-or-integrated-gpu"
    )
    .is_err());
}

#[test]
fn emits_canonical_multi_output_spirv() {
    let entry = "nuis_vulkan_add_xor_pair_u32";
    let lowered = lower_canonical_inline_wgsl_u32_for_profile(
        canonical_pair_fan_out_wgsl(entry).as_bytes(),
        entry,
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    let words = spirv_words(&lowered);

    assert_eq!(words[3], 28);
    assert!(has_instruction(&words, 71, &[22, 33, 2]));
    assert!(has_instruction(&words, 71, &[25, 33, 3]));
    assert!(has_instruction(&words, 71, &[22, 25]));
    assert!(has_instruction(&words, 71, &[25, 25]));
    assert!(has_instruction(&words, 128, &[3, 24, 18, 21]));
    assert!(has_instruction(&words, 198, &[3, 27, 18, 21]));
    assert!(has_instruction(&words, 62, &[23, 24]));
    assert!(has_instruction(&words, 62, &[26, 27]));
}

#[test]
fn emits_bounds_safe_reduced_output_spirv() {
    let entry = "nuis_vulkan_add_xor_pair_reduced_u32";
    let lowered = lower_canonical_inline_wgsl_u32_for_profile(
        canonical_pair_reduced_fan_out_wgsl(entry).as_bytes(),
        entry,
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    let words = spirv_words(&lowered);

    assert_eq!(words[3], 33);
    assert!(has_instruction(&words, 20, &[32]));
    assert!(has_instruction(&words, 43, &[3, 28, 2]));
    assert!(has_instruction(&words, 176, &[32, 29, 16, 28]));
    assert!(has_instruction(&words, 247, &[31, 0]));
    assert!(has_instruction(&words, 250, &[29, 30, 31]));
    assert!(has_instruction(&words, 248, &[30]));
    assert!(has_instruction(&words, 65, &[12, 26, 25, 4, 16]));
    assert!(has_instruction(&words, 62, &[26, 27]));
    assert!(has_instruction(&words, 249, &[31]));
    assert!(has_instruction(&words, 248, &[31]));
}

#[test]
fn rejects_zero_output_write_extent() {
    let source = CANONICAL_WGSL.replace("array<u32>;", "array<u32, 0>;");
    assert!(lower_canonical_inline_wgsl_u32_for_profile(
        source.as_bytes(),
        "nuis_vulkan_copy_u32",
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap_err()
    .contains("element extent must be positive"));
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
        ("nuis_vulkan_xor_u32", "^", 198_u16),
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
fn emits_pair_add_u32_module_from_canonical_wgsl_body() {
    let entry = "nuis_vulkan_add_pair_u32";
    let source = canonical_pair_add_wgsl(entry);
    let lowered = lower_canonical_inline_wgsl_u32_for_profile(
        source.as_bytes(),
        entry,
        "vulkan.discrete-or-integrated-gpu",
    )
    .unwrap();
    let words = spirv_words(&lowered);

    assert!(has_instruction(&words, 71, &[19, 33, 1]));
    assert!(has_instruction(&words, 71, &[22, 33, 2]));
    assert!(has_instruction(&words, 128, &[3, 24, 18, 21]));
    assert!(lowered
        .windows(entry.len())
        .any(|window| window == entry.as_bytes()));
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
