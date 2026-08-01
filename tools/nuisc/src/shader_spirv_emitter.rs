use std::collections::BTreeMap;

use crate::shader_canonical_compute::{
    parse_canonical_inline_wgsl_u32_compute, parse_u32_operation, CanonicalU32Compute,
    CanonicalU32Operation, CanonicalU32Output,
};

pub(crate) const SPIRV_COMPUTE_SOURCE_CONTRACT: &str = "nuis-spirv-compute-source-v1";
const SHADER_MODULE_BACKEND_PLAN_CONTRACT: &str = "nuis-yir.shader.backend-lowering-plan.v1";
const SHADER_MODULE_SUMMARY_SCHEMA: &str = "nuis-yir.shader.module-summary.v1";
const SHADER_MODULE_LOWERING_BOUNDARY: &str = "module-summary-to-native-ir";
const SPIRV_VULKAN_LOWERING_TARGET: &str = "spirv:vulkan-gpu";
const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;

pub(crate) fn lower_registered_compute_source_for_profile(
    source: &[u8],
    expected_entry: &str,
    expected_profile_lowering_target: &str,
) -> Result<Vec<u8>, String> {
    let source = std::str::from_utf8(source)
        .map_err(|_| "Nuis SPIR-V compute source must be UTF-8".to_owned())?;
    let source = parse_compute_source(source, expected_profile_lowering_target)?;
    if source.entry != expected_entry {
        return Err(format!(
            "Nuis SPIR-V source entry `{}` does not match registered entry `{expected_entry}`",
            source.entry
        ));
    }
    let words = emit_u32_module(&source);
    validate_module_shape(&words, expected_entry)?;
    Ok(words
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>())
}

pub(crate) fn lower_canonical_inline_wgsl_u32_for_profile(
    source: &[u8],
    expected_entry: &str,
    expected_profile_lowering_target: &str,
) -> Result<Vec<u8>, String> {
    let source = std::str::from_utf8(source)
        .map_err(|_| "canonical inline WGSL SPIR-V source must be UTF-8".to_owned())?;
    let plan = canonical_spirv_compute_plan(expected_profile_lowering_target);
    validate_module_lowering_plan(&plan, expected_profile_lowering_target)?;
    let compute = parse_canonical_inline_wgsl_u32_compute(source, expected_entry)?;
    let words = emit_u32_module(&compute);
    validate_module_shape(&words, expected_entry)?;
    Ok(words
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>())
}

fn parse_compute_source(
    source: &str,
    expected_profile_lowering_target: &str,
) -> Result<CanonicalU32Compute, String> {
    let mut fields = BTreeMap::new();
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("invalid Nuis SPIR-V source line `{line}`"))?;
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        if fields.insert(key, value).is_some() {
            return Err(format!("duplicate Nuis SPIR-V source field `{key}`"));
        }
    }
    if required_field(&fields, "contract")? != SPIRV_COMPUTE_SOURCE_CONTRACT
        || required_field(&fields, "spirv_version")? != "1.6"
    {
        return Err("unsupported Nuis SPIR-V compute source contract".to_owned());
    }
    let operation = parse_u32_operation(required_field(&fields, "operation")?)?;
    let plan = module_lowering_plan_from_fields(&fields)?;
    validate_module_lowering_plan(&plan, expected_profile_lowering_target)?;
    let entry = required_field(&fields, "entry")?;
    if !symbol_is_valid(entry) {
        return Err(format!("invalid Nuis SPIR-V entry `{entry}`"));
    }
    let local_size = parse_local_size(required_field(&fields, "local_size")?)?;
    let descriptor_set = parse_u32(&fields, "descriptor_set")?;
    let input_binding = parse_u32(&fields, "input_binding")?;
    let aux_input_binding = parse_optional_u32(&fields, "aux_input_binding")?;
    let output_binding = parse_u32(&fields, "output_binding")?;
    validate_u32_compute_bindings(operation, input_binding, aux_input_binding, output_binding)?;
    Ok(CanonicalU32Compute {
        entry: entry.to_owned(),
        local_size,
        descriptor_set,
        input_binding,
        aux_input_binding,
        outputs: vec![CanonicalU32Output {
            binding: output_binding,
            operation,
        }],
    })
}

fn validate_u32_compute_bindings(
    operation: CanonicalU32Operation,
    input_binding: u32,
    aux_input_binding: Option<u32>,
    output_binding: u32,
) -> Result<(), String> {
    if operation.input_count() == 2 && aux_input_binding.is_none() {
        return Err("Nuis SPIR-V two-input u32 operation requires aux_input_binding".to_owned());
    }
    if operation.input_count() == 1 && aux_input_binding.is_some() {
        return Err(
            "Nuis SPIR-V one-input u32 operation must not declare aux_input_binding".to_owned(),
        );
    }
    if input_binding == output_binding
        || aux_input_binding.is_some_and(|aux| aux == input_binding || aux == output_binding)
    {
        return Err(
            "Nuis SPIR-V input, auxiliary input, and output bindings must differ".to_owned(),
        );
    }
    Ok(())
}

struct ModuleLoweringPlan {
    contract: String,
    source_schema: String,
    lowering_boundary: String,
    profile_lowering_target: String,
    lowering_target: String,
    native_ir: String,
    stage_kind: String,
    execution_model: String,
    binding_slot_model: String,
}

fn module_lowering_plan_from_fields(
    fields: &BTreeMap<&str, &str>,
) -> Result<ModuleLoweringPlan, String> {
    Ok(ModuleLoweringPlan {
        contract: required_field(fields, "module_lowering_plan_contract")?.to_owned(),
        source_schema: required_field(fields, "module_source_schema")?.to_owned(),
        lowering_boundary: required_field(fields, "module_lowering_boundary")?.to_owned(),
        profile_lowering_target: required_field(fields, "module_profile_lowering_target")?
            .to_owned(),
        lowering_target: required_field(fields, "module_lowering_target")?.to_owned(),
        native_ir: required_field(fields, "module_native_ir")?.to_owned(),
        stage_kind: required_field(fields, "module_stage_kind")?.to_owned(),
        execution_model: required_field(fields, "module_execution_model")?.to_owned(),
        binding_slot_model: required_field(fields, "module_binding_slot_model")?.to_owned(),
    })
}

fn canonical_spirv_compute_plan(expected_profile_lowering_target: &str) -> ModuleLoweringPlan {
    ModuleLoweringPlan {
        contract: SHADER_MODULE_BACKEND_PLAN_CONTRACT.to_owned(),
        source_schema: SHADER_MODULE_SUMMARY_SCHEMA.to_owned(),
        lowering_boundary: SHADER_MODULE_LOWERING_BOUNDARY.to_owned(),
        profile_lowering_target: expected_profile_lowering_target.to_owned(),
        lowering_target: SPIRV_VULKAN_LOWERING_TARGET.to_owned(),
        native_ir: "spirv1.6".to_owned(),
        stage_kind: "compute".to_owned(),
        execution_model: "GLCompute".to_owned(),
        binding_slot_model: "descriptor-set-binding".to_owned(),
    }
}

fn validate_module_lowering_plan(
    plan: &ModuleLoweringPlan,
    expected_profile_lowering_target: &str,
) -> Result<(), String> {
    for (field, actual, expected) in [
        (
            "module_lowering_plan_contract",
            plan.contract.as_str(),
            SHADER_MODULE_BACKEND_PLAN_CONTRACT,
        ),
        (
            "module_source_schema",
            plan.source_schema.as_str(),
            SHADER_MODULE_SUMMARY_SCHEMA,
        ),
        (
            "module_lowering_boundary",
            plan.lowering_boundary.as_str(),
            SHADER_MODULE_LOWERING_BOUNDARY,
        ),
        (
            "module_profile_lowering_target",
            plan.profile_lowering_target.as_str(),
            expected_profile_lowering_target,
        ),
        (
            "module_lowering_target",
            plan.lowering_target.as_str(),
            SPIRV_VULKAN_LOWERING_TARGET,
        ),
        ("module_native_ir", plan.native_ir.as_str(), "spirv1.6"),
        ("module_stage_kind", plan.stage_kind.as_str(), "compute"),
        (
            "module_execution_model",
            plan.execution_model.as_str(),
            "GLCompute",
        ),
        (
            "module_binding_slot_model",
            plan.binding_slot_model.as_str(),
            "descriptor-set-binding",
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "Nuis SPIR-V module lowering plan field `{field}` is `{actual}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn emit_u32_module(source: &CanonicalU32Compute) -> Vec<u32> {
    const VOID: u32 = 1;
    const FUNCTION_TYPE: u32 = 2;
    const U32_TYPE: u32 = 3;
    const U32_ZERO: u32 = 4;
    const U32_VEC3: u32 = 5;
    const INPUT_VEC3_POINTER: u32 = 6;
    const GLOBAL_INVOCATION_ID: u32 = 7;
    const U32_RUNTIME_ARRAY: u32 = 8;
    const BUFFER_BLOCK: u32 = 9;
    const STORAGE_BUFFER_BLOCK_POINTER: u32 = 10;
    const INPUT_BUFFER: u32 = 11;
    const STORAGE_BUFFER_U32_POINTER: u32 = 12;
    const MAIN: u32 = 13;
    const LABEL: u32 = 14;
    const INVOCATION: u32 = 15;
    const INDEX: u32 = 16;
    const INPUT_ELEMENT: u32 = 17;
    const VALUE: u32 = 18;

    let mut next_id = 19;
    let aux_ids = source.aux_input_binding.map(|_| SpirvInputIds {
        variable: allocate_spirv_id(&mut next_id),
        element: allocate_spirv_id(&mut next_id),
        value: allocate_spirv_id(&mut next_id),
    });
    let output_ids = source
        .outputs
        .iter()
        .map(|output| SpirvOutputIds {
            variable: allocate_spirv_id(&mut next_id),
            element: allocate_spirv_id(&mut next_id),
            computed: spirv_u32_binary_opcode(output.operation)
                .map(|_| allocate_spirv_id(&mut next_id)),
        })
        .collect::<Vec<_>>();
    let id_bound = next_id;
    let mut words = vec![SPIRV_MAGIC, SPIRV_VERSION_1_6, 0, id_bound, 0];
    instruction(&mut words, 17, &[1]);
    instruction(&mut words, 14, &[0, 1]);
    let mut entry_operands = vec![5, MAIN];
    entry_operands.extend(encode_string(&source.entry));
    entry_operands.extend([GLOBAL_INVOCATION_ID, INPUT_BUFFER]);
    if let Some(aux) = &aux_ids {
        entry_operands.push(aux.variable);
    }
    entry_operands.extend(output_ids.iter().map(|output| output.variable));
    instruction(&mut words, 15, &entry_operands);
    instruction(
        &mut words,
        16,
        &[
            MAIN,
            17,
            source.local_size[0],
            source.local_size[1],
            source.local_size[2],
        ],
    );
    instruction(&mut words, 71, &[U32_RUNTIME_ARRAY, 6, 4]);
    instruction(&mut words, 72, &[BUFFER_BLOCK, 0, 35, 0]);
    instruction(&mut words, 71, &[BUFFER_BLOCK, 2]);
    instruction(&mut words, 71, &[GLOBAL_INVOCATION_ID, 11, 28]);
    instruction(&mut words, 71, &[INPUT_BUFFER, 34, source.descriptor_set]);
    instruction(&mut words, 71, &[INPUT_BUFFER, 33, source.input_binding]);
    instruction(&mut words, 71, &[INPUT_BUFFER, 24]);
    if let (Some(aux_input_binding), Some(aux)) = (source.aux_input_binding, &aux_ids) {
        instruction(&mut words, 71, &[aux.variable, 34, source.descriptor_set]);
        instruction(&mut words, 71, &[aux.variable, 33, aux_input_binding]);
        instruction(&mut words, 71, &[aux.variable, 24]);
    }
    for (output, ids) in source.outputs.iter().zip(&output_ids) {
        instruction(&mut words, 71, &[ids.variable, 34, source.descriptor_set]);
        instruction(&mut words, 71, &[ids.variable, 33, output.binding]);
        instruction(&mut words, 71, &[ids.variable, 25]);
    }
    instruction(&mut words, 19, &[VOID]);
    instruction(&mut words, 33, &[FUNCTION_TYPE, VOID]);
    instruction(&mut words, 21, &[U32_TYPE, 32, 0]);
    instruction(&mut words, 43, &[U32_TYPE, U32_ZERO, 0]);
    instruction(&mut words, 23, &[U32_VEC3, U32_TYPE, 3]);
    instruction(&mut words, 32, &[INPUT_VEC3_POINTER, 1, U32_VEC3]);
    instruction(
        &mut words,
        59,
        &[INPUT_VEC3_POINTER, GLOBAL_INVOCATION_ID, 1],
    );
    instruction(&mut words, 29, &[U32_RUNTIME_ARRAY, U32_TYPE]);
    instruction(&mut words, 30, &[BUFFER_BLOCK, U32_RUNTIME_ARRAY]);
    instruction(
        &mut words,
        32,
        &[STORAGE_BUFFER_BLOCK_POINTER, 12, BUFFER_BLOCK],
    );
    instruction(
        &mut words,
        59,
        &[STORAGE_BUFFER_BLOCK_POINTER, INPUT_BUFFER, 12],
    );
    if let Some(aux) = &aux_ids {
        instruction(
            &mut words,
            59,
            &[STORAGE_BUFFER_BLOCK_POINTER, aux.variable, 12],
        );
    }
    for output in &output_ids {
        instruction(
            &mut words,
            59,
            &[STORAGE_BUFFER_BLOCK_POINTER, output.variable, 12],
        );
    }
    instruction(&mut words, 32, &[STORAGE_BUFFER_U32_POINTER, 12, U32_TYPE]);
    instruction(&mut words, 54, &[VOID, MAIN, 0, FUNCTION_TYPE]);
    instruction(&mut words, 248, &[LABEL]);
    instruction(
        &mut words,
        61,
        &[U32_VEC3, INVOCATION, GLOBAL_INVOCATION_ID],
    );
    instruction(&mut words, 81, &[U32_TYPE, INDEX, INVOCATION, 0]);
    instruction(
        &mut words,
        65,
        &[
            STORAGE_BUFFER_U32_POINTER,
            INPUT_ELEMENT,
            INPUT_BUFFER,
            U32_ZERO,
            INDEX,
        ],
    );
    instruction(&mut words, 61, &[U32_TYPE, VALUE, INPUT_ELEMENT]);
    if let Some(aux) = &aux_ids {
        instruction(
            &mut words,
            65,
            &[
                STORAGE_BUFFER_U32_POINTER,
                aux.element,
                aux.variable,
                U32_ZERO,
                INDEX,
            ],
        );
        instruction(&mut words, 61, &[U32_TYPE, aux.value, aux.element]);
    }
    for (output, ids) in source.outputs.iter().zip(&output_ids) {
        instruction(
            &mut words,
            65,
            &[
                STORAGE_BUFFER_U32_POINTER,
                ids.element,
                ids.variable,
                U32_ZERO,
                INDEX,
            ],
        );
        match (spirv_u32_binary_opcode(output.operation), ids.computed) {
            (None, None) => instruction(&mut words, 62, &[ids.element, VALUE]),
            (Some(opcode), Some(computed)) => {
                let rhs = aux_ids.as_ref().map(|aux| aux.value).unwrap_or(VALUE);
                instruction(&mut words, opcode, &[U32_TYPE, computed, VALUE, rhs]);
                instruction(&mut words, 62, &[ids.element, computed]);
            }
            _ => unreachable!("SPIR-V output IDs follow the registered operation"),
        }
    }
    instruction(&mut words, 253, &[]);
    instruction(&mut words, 56, &[]);
    words
}

struct SpirvInputIds {
    variable: u32,
    element: u32,
    value: u32,
}

struct SpirvOutputIds {
    variable: u32,
    element: u32,
    computed: Option<u32>,
}

fn allocate_spirv_id(next: &mut u32) -> u32 {
    let id = *next;
    *next += 1;
    id
}

fn spirv_u32_binary_opcode(operation: CanonicalU32Operation) -> Option<u16> {
    match operation {
        CanonicalU32Operation::CopyU32 => None,
        CanonicalU32Operation::AddU32 => Some(128),
        CanonicalU32Operation::SubU32 => Some(130),
        CanonicalU32Operation::MulU32 => Some(132),
        CanonicalU32Operation::XorU32 => Some(198),
        CanonicalU32Operation::AddPairU32 => Some(128),
        CanonicalU32Operation::XorPairU32 => Some(198),
    }
}

fn required_field<'a>(fields: &BTreeMap<&'a str, &'a str>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .copied()
        .ok_or_else(|| format!("missing Nuis SPIR-V source field `{key}`"))
}

fn parse_u32(fields: &BTreeMap<&str, &str>, key: &str) -> Result<u32, String> {
    required_field(fields, key)?
        .parse()
        .map_err(|_| format!("Nuis SPIR-V source field `{key}` must be u32"))
}

fn parse_optional_u32(fields: &BTreeMap<&str, &str>, key: &str) -> Result<Option<u32>, String> {
    fields
        .get(key)
        .map(|value| {
            value
                .parse()
                .map_err(|_| format!("Nuis SPIR-V source field `{key}` must be u32"))
        })
        .transpose()
}

fn parse_local_size(value: &str) -> Result<[u32; 3], String> {
    let values = value
        .split('x')
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Nuis SPIR-V local_size must use positive u32 dimensions".to_owned())?;
    let [x, y, z] = values.as_slice() else {
        return Err("Nuis SPIR-V local_size must contain three dimensions".to_owned());
    };
    if [x, y, z].into_iter().any(|value| *value == 0)
        || u64::from(*x) * u64::from(*y) * u64::from(*z) > 1024
    {
        return Err("Nuis SPIR-V local_size exceeds the portable compute limit".to_owned());
    }
    Ok([*x, *y, *z])
}

fn symbol_is_valid(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn instruction(words: &mut Vec<u32>, opcode: u16, operands: &[u32]) {
    let word_count = u32::try_from(operands.len() + 1).expect("SPIR-V instruction is bounded");
    words.push((word_count << 16) | u32::from(opcode));
    words.extend_from_slice(operands);
}

fn encode_string(value: &str) -> Vec<u32> {
    let mut bytes = value.as_bytes().to_vec();
    bytes.push(0);
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes(chunk.try_into().expect("four-byte SPIR-V word")))
        .collect()
}

fn validate_module_shape(words: &[u32], entry: &str) -> Result<(), String> {
    if words.len() < 6 || words[0] != SPIRV_MAGIC || words[1] != SPIRV_VERSION_1_6 || words[3] < 21
    {
        return Err("Nuis SPIR-V emitter produced an invalid module header".to_owned());
    }
    let encoded_entry = encode_string(entry);
    let mut cursor = 5;
    let mut found_entry = false;
    while cursor < words.len() {
        let instruction = words[cursor];
        let word_count = usize::try_from(instruction >> 16).unwrap_or(0);
        let opcode = instruction as u16;
        if word_count == 0 || cursor + word_count > words.len() {
            return Err("Nuis SPIR-V emitter produced an invalid instruction span".to_owned());
        }
        if opcode == 15
            && word_count >= 3 + encoded_entry.len()
            && words[cursor + 1] == 5
            && words[cursor + 2] > 0
            && words[cursor + 2] < words[3]
            && words[cursor + 3..cursor + 3 + encoded_entry.len()] == encoded_entry
        {
            found_entry = true;
        }
        cursor += word_count;
    }
    if cursor != words.len() || !found_entry {
        return Err("Nuis SPIR-V emitter did not preserve the registered entry".to_owned());
    }
    Ok(())
}

#[cfg(test)]
#[path = "shader_spirv_emitter_tests.rs"]
mod tests;
