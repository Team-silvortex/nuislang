use std::collections::BTreeMap;

pub(crate) const SPIRV_COMPUTE_SOURCE_CONTRACT: &str = "nuis-spirv-compute-source-v1";
const SPIRV_MAGIC: u32 = 0x0723_0203;
const SPIRV_VERSION_1_6: u32 = 0x0001_0600;

struct ComputeSource {
    entry: String,
    local_size: [u32; 3],
    descriptor_set: u32,
    input_binding: u32,
    output_binding: u32,
}

pub(crate) fn lower_registered_compute_source(
    source: &[u8],
    expected_entry: &str,
) -> Result<Vec<u8>, String> {
    let source = std::str::from_utf8(source)
        .map_err(|_| "Nuis SPIR-V compute source must be UTF-8".to_owned())?;
    let source = parse_compute_source(source)?;
    if source.entry != expected_entry {
        return Err(format!(
            "Nuis SPIR-V source entry `{}` does not match registered entry `{expected_entry}`",
            source.entry
        ));
    }
    let words = emit_copy_u32_module(&source);
    validate_module_shape(&words, expected_entry)?;
    Ok(words
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>())
}

fn parse_compute_source(source: &str) -> Result<ComputeSource, String> {
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
        || required_field(&fields, "operation")? != "copy-u32"
    {
        return Err("unsupported Nuis SPIR-V compute source contract or operation".to_owned());
    }
    let entry = required_field(&fields, "entry")?;
    if !symbol_is_valid(entry) {
        return Err(format!("invalid Nuis SPIR-V entry `{entry}`"));
    }
    let local_size = parse_local_size(required_field(&fields, "local_size")?)?;
    let descriptor_set = parse_u32(&fields, "descriptor_set")?;
    let input_binding = parse_u32(&fields, "input_binding")?;
    let output_binding = parse_u32(&fields, "output_binding")?;
    if input_binding == output_binding {
        return Err("Nuis SPIR-V input and output bindings must differ".to_owned());
    }
    Ok(ComputeSource {
        entry: entry.to_owned(),
        local_size,
        descriptor_set,
        input_binding,
        output_binding,
    })
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

fn emit_copy_u32_module(source: &ComputeSource) -> Vec<u32> {
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
    const OUTPUT_BUFFER: u32 = 12;
    const STORAGE_BUFFER_U32_POINTER: u32 = 13;
    const MAIN: u32 = 14;
    const LABEL: u32 = 15;
    const INVOCATION: u32 = 16;
    const INDEX: u32 = 17;
    const INPUT_ELEMENT: u32 = 18;
    const VALUE: u32 = 19;
    const OUTPUT_ELEMENT: u32 = 20;

    let mut words = vec![SPIRV_MAGIC, SPIRV_VERSION_1_6, 0, 21, 0];
    instruction(&mut words, 17, &[1]);
    instruction(&mut words, 14, &[0, 1]);
    let mut entry_operands = vec![5, MAIN];
    entry_operands.extend(encode_string(&source.entry));
    entry_operands.extend([GLOBAL_INVOCATION_ID, INPUT_BUFFER, OUTPUT_BUFFER]);
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
    instruction(&mut words, 71, &[OUTPUT_BUFFER, 34, source.descriptor_set]);
    instruction(&mut words, 71, &[OUTPUT_BUFFER, 33, source.output_binding]);
    instruction(&mut words, 71, &[OUTPUT_BUFFER, 25]);
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
    instruction(
        &mut words,
        59,
        &[STORAGE_BUFFER_BLOCK_POINTER, OUTPUT_BUFFER, 12],
    );
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
    instruction(
        &mut words,
        65,
        &[
            STORAGE_BUFFER_U32_POINTER,
            OUTPUT_ELEMENT,
            OUTPUT_BUFFER,
            U32_ZERO,
            INDEX,
        ],
    );
    instruction(&mut words, 62, &[OUTPUT_ELEMENT, VALUE]);
    instruction(&mut words, 253, &[]);
    instruction(&mut words, 56, &[]);
    words
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
    if words.len() < 6 || words[0] != SPIRV_MAGIC || words[1] != SPIRV_VERSION_1_6 || words[3] != 21
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
            && words[cursor + 2] == 14
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
mod tests {
    use super::*;

    const SOURCE: &str = r#"
contract = "nuis-spirv-compute-source-v1"
spirv_version = "1.6"
operation = "copy-u32"
entry = "nuis_vulkan_copy_u32"
local_size = "1x1x1"
descriptor_set = 0
input_binding = 0
output_binding = 1
"#;

    #[test]
    fn emits_deterministic_spirv_copy_module_without_external_tools() {
        let first =
            lower_registered_compute_source(SOURCE.as_bytes(), "nuis_vulkan_copy_u32").unwrap();
        let repeated =
            lower_registered_compute_source(SOURCE.as_bytes(), "nuis_vulkan_copy_u32").unwrap();
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
        assert!(lower_registered_compute_source(SOURCE.as_bytes(), "other_entry").is_err());
        let duplicate_binding = SOURCE.replace("output_binding = 1", "output_binding = 0");
        assert!(lower_registered_compute_source(
            duplicate_binding.as_bytes(),
            "nuis_vulkan_copy_u32"
        )
        .is_err());
    }
}
