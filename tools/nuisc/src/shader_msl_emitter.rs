use crate::shader_canonical_compute::{
    parse_canonical_inline_wgsl_u32_compute, CanonicalU32Compute, CanonicalU32Operation,
};

const MSL_TARGETS: &[&str] = &[
    "metal.apple-silicon-gpu",
    "metal.mac-discrete-or-integrated-gpu",
];
const SHADER_MODULE_BACKEND_PLAN_CONTRACT: &str = "nuis-yir.shader.backend-lowering-plan.v1";
const SHADER_MODULE_SUMMARY_SCHEMA: &str = "nuis-yir.shader.module-summary.v1";
const SHADER_MODULE_LOWERING_BOUNDARY: &str = "module-summary-to-native-ir";
const MSL_LOWERING_TARGET: &str = "msl:metal-gpu";
const MSL_NATIVE_IR: &str = "msl2.4";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleLoweringPlan {
    contract: &'static str,
    source_schema: &'static str,
    lowering_boundary: &'static str,
    profile_lowering_target: String,
    lowering_target: &'static str,
    native_ir: &'static str,
    stage_kind: &'static str,
    execution_model: &'static str,
    binding_slot_model: &'static str,
}

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
    let plan = canonical_msl_compute_plan(expected_profile_lowering_target);
    validate_msl_module_lowering_plan(&plan, expected_profile_lowering_target)?;
    let compute = parse_canonical_inline_wgsl_u32_compute(source, expected_entry)?;
    Ok(render_u32_msl(&compute, &plan).into_bytes())
}

fn canonical_msl_compute_plan(expected_profile_lowering_target: &str) -> ModuleLoweringPlan {
    ModuleLoweringPlan {
        contract: SHADER_MODULE_BACKEND_PLAN_CONTRACT,
        source_schema: SHADER_MODULE_SUMMARY_SCHEMA,
        lowering_boundary: SHADER_MODULE_LOWERING_BOUNDARY,
        profile_lowering_target: expected_profile_lowering_target.to_owned(),
        lowering_target: MSL_LOWERING_TARGET,
        native_ir: MSL_NATIVE_IR,
        stage_kind: "compute",
        execution_model: "kernel",
        binding_slot_model: "argument-buffer-slot",
    }
}

fn validate_msl_module_lowering_plan(
    plan: &ModuleLoweringPlan,
    expected_profile_lowering_target: &str,
) -> Result<(), String> {
    for (field, actual, expected) in [
        (
            "module_lowering_plan_contract",
            plan.contract,
            SHADER_MODULE_BACKEND_PLAN_CONTRACT,
        ),
        (
            "module_source_schema",
            plan.source_schema,
            SHADER_MODULE_SUMMARY_SCHEMA,
        ),
        (
            "module_lowering_boundary",
            plan.lowering_boundary,
            SHADER_MODULE_LOWERING_BOUNDARY,
        ),
        (
            "module_profile_lowering_target",
            plan.profile_lowering_target.as_str(),
            expected_profile_lowering_target,
        ),
        (
            "module_lowering_target",
            plan.lowering_target,
            MSL_LOWERING_TARGET,
        ),
        ("module_native_ir", plan.native_ir, MSL_NATIVE_IR),
        ("module_stage_kind", plan.stage_kind, "compute"),
        ("module_execution_model", plan.execution_model, "kernel"),
        (
            "module_binding_slot_model",
            plan.binding_slot_model,
            "argument-buffer-slot",
        ),
    ] {
        if actual != expected {
            return Err(format!(
                "Nuis MSL module lowering plan field `{field}` is `{actual}`, expected `{expected}`"
            ));
        }
    }
    Ok(())
}

fn render_u32_msl(source: &CanonicalU32Compute, plan: &ModuleLoweringPlan) -> String {
    let input_binding = source.input_binding;
    let aux_parameter = source
        .aux_input_binding
        .map(|binding| format!("    device const uint* right_values [[buffer({binding})]],\n"))
        .unwrap_or_default();
    let aux_load = source
        .aux_input_binding
        .map(|_| "    uint rhs = right_values[gid];\n")
        .unwrap_or_default();
    let output_parameters = source
        .outputs
        .iter()
        .enumerate()
        .map(|(index, output)| {
            format!(
                "    device uint* {} [[buffer({})]],\n",
                msl_output_name(index),
                output.binding
            )
        })
        .collect::<String>();
    let output_stores = source
        .outputs
        .iter()
        .enumerate()
        .map(|(index, output)| {
            format!(
                "    {}[gid] = {};\n",
                msl_output_name(index),
                msl_u32_expression(output.operation)
            )
        })
        .collect::<String>();
    let entry = &source.entry;
    format!(
        "// nuis-module-lowering-plan contract={}\n\
         // nuis-module-source-schema {}\n\
         // nuis-module-lowering-boundary {}\n\
         // nuis-module-profile-lowering-target {}\n\
         // nuis-module-lowering-target {}\n\
         // nuis-module-native-ir {}\n\
         // nuis-module-stage kind={} execution_model={} binding_slot_model={}\n\
         #include <metal_stdlib>\n\
         using namespace metal;\n\
         \n\
         kernel void {entry}(\n\
             device const uint* input_values [[buffer({input_binding})]],\n\
{aux_parameter}\
{output_parameters}\
             uint gid [[thread_position_in_grid]]) {{\n\
             uint value = input_values[gid];\n\
{aux_load}\
{output_stores}\
         }}\n",
        plan.contract,
        plan.source_schema,
        plan.lowering_boundary,
        plan.profile_lowering_target,
        plan.lowering_target,
        plan.native_ir,
        plan.stage_kind,
        plan.execution_model,
        plan.binding_slot_model
    )
}

fn msl_output_name(index: usize) -> String {
    if index == 0 {
        "output_values".to_owned()
    } else {
        format!("output_values_{index}")
    }
}

fn msl_u32_expression(operation: CanonicalU32Operation) -> &'static str {
    match operation {
        CanonicalU32Operation::CopyU32 => "value",
        CanonicalU32Operation::AddU32 => "value + value",
        CanonicalU32Operation::SubU32 => "value - value",
        CanonicalU32Operation::MulU32 => "value * value",
        CanonicalU32Operation::XorU32 => "value ^ value",
        CanonicalU32Operation::AddPairU32 => "value + rhs",
        CanonicalU32Operation::XorPairU32 => "value ^ rhs",
    }
}

#[cfg(test)]
#[path = "shader_msl_emitter_tests.rs"]
mod tests;
