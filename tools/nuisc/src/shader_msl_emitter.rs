use crate::shader_canonical_compute::{
    parse_canonical_inline_wgsl_u32_compute, CanonicalU32Compute, CanonicalU32Operation,
};

const MSL_TARGETS: &[&str] = &[
    "metal.apple-silicon-gpu",
    "metal.mac-discrete-or-integrated-gpu",
];

pub(crate) fn lower_canonical_inline_wgsl_u32_for_profile(
    source: &[u8],
    expected_entry: &str,
    expected_profile_lowering_target: &str,
) -> Result<Vec<u8>, String> {
    if !MSL_TARGETS.contains(&expected_profile_lowering_target) {
        return Err(format!(
            "canonical inline WGSL MSL lowering target `{expected_profile_lowering_target}` is unsupported"
        ));
    }
    let source = std::str::from_utf8(source)
        .map_err(|_| "canonical inline WGSL MSL source must be UTF-8".to_owned())?;
    let compute = parse_canonical_inline_wgsl_u32_compute(source, expected_entry)?;
    Ok(render_u32_msl(&compute).into_bytes())
}

fn render_u32_msl(source: &CanonicalU32Compute) -> String {
    let input_binding = source.input_binding;
    let output_binding = source.output_binding;
    let entry = &source.entry;
    let expression = msl_u32_expression(source.operation);
    format!(
        "#include <metal_stdlib>\n\
         using namespace metal;\n\
         \n\
         kernel void {entry}(\n\
             device const uint* input_values [[buffer({input_binding})]],\n\
             device uint* output_values [[buffer({output_binding})]],\n\
             uint gid [[thread_position_in_grid]]) {{\n\
             uint value = input_values[gid];\n\
             output_values[gid] = {expression};\n\
         }}\n"
    )
}

fn msl_u32_expression(operation: CanonicalU32Operation) -> &'static str {
    match operation {
        CanonicalU32Operation::CopyU32 => "value",
        CanonicalU32Operation::AddU32 => "value + value",
        CanonicalU32Operation::SubU32 => "value - value",
        CanonicalU32Operation::MulU32 => "value * value",
    }
}

#[cfg(test)]
#[path = "shader_msl_emitter_tests.rs"]
mod tests;
