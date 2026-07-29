use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_profile::DerivedLoweringProfile;
use crate::aot_toml::{escape_toml_string, render_string_array};

const MODULE_SUMMARY_SCHEMA: &str = "nuis-yir.shader.module-summary.v1";
const MODULE_LOWERING_PLAN_CONTRACT: &str = "nuis-yir.shader.backend-lowering-plan.v1";
const MODULE_LOWERING_BOUNDARY: &str = "module-summary-to-native-ir";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShaderModulePlanSpec {
    profile: &'static str,
    backend: &'static str,
    target: &'static str,
    lowering_target: &'static str,
    native_ir: &'static str,
    stage_model: StageModel,
    binding_slot_model: &'static str,
    target_entry_policy: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StageModel {
    Msl,
    Spirv,
    Dxil,
    Glsl,
    HostSimd,
}

const SHADER_MODULE_PLAN_SPECS: &[ShaderModulePlanSpec] = &[
    ShaderModulePlanSpec {
        profile: "metal.apple-silicon-gpu",
        backend: "msl",
        target: "metal-gpu",
        lowering_target: "msl:metal-gpu",
        native_ir: "msl2.4",
        stage_model: StageModel::Msl,
        binding_slot_model: "argument-buffer-slot",
        target_entry_policy: "preserve-source-entry",
    },
    ShaderModulePlanSpec {
        profile: "metal.mac-discrete-or-integrated-gpu",
        backend: "msl",
        target: "metal-gpu",
        lowering_target: "msl:metal-gpu",
        native_ir: "msl2.4",
        stage_model: StageModel::Msl,
        binding_slot_model: "argument-buffer-slot",
        target_entry_policy: "preserve-source-entry",
    },
    ShaderModulePlanSpec {
        profile: "vulkan.discrete-or-integrated-gpu",
        backend: "spirv",
        target: "vulkan-gpu",
        lowering_target: "spirv:vulkan-gpu",
        native_ir: "spirv1.6",
        stage_model: StageModel::Spirv,
        binding_slot_model: "descriptor-set-binding",
        target_entry_policy: "preserve-source-entry",
    },
    ShaderModulePlanSpec {
        profile: "directx.discrete-or-integrated-gpu",
        backend: "dxil",
        target: "directx-gpu",
        lowering_target: "dxil:directx-gpu",
        native_ir: "dxil6.8",
        stage_model: StageModel::Dxil,
        binding_slot_model: "root-signature-slot",
        target_entry_policy: "preserve-source-entry",
    },
    ShaderModulePlanSpec {
        profile: "opengl.discrete-or-integrated-gpu",
        backend: "glsl",
        target: "opengl-gpu",
        lowering_target: "glsl:opengl-gpu",
        native_ir: "glsl460",
        stage_model: StageModel::Glsl,
        binding_slot_model: "uniform-binding-slot",
        target_entry_policy: "preserve-source-entry",
    },
    ShaderModulePlanSpec {
        profile: "cpu-fallback.cpu-host",
        backend: "host-simd",
        target: "cpu-fallback",
        lowering_target: "host-simd:cpu-fallback",
        native_ir: "nuis-host-simd-yir",
        stage_model: StageModel::HostSimd,
        binding_slot_model: "host-tuple-slot",
        target_entry_policy: "prefix:host_",
    },
];

pub(crate) fn render_shader_module_lowering_plan(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
    supported_stages: &[&str],
) -> String {
    if unit.domain_family != "shader" {
        return String::new();
    }
    let Some(spec) = SHADER_MODULE_PLAN_SPECS
        .iter()
        .find(|spec| spec.profile == profile.profile_key)
    else {
        return String::new();
    };

    let mut out = String::new();
    out.push_str("[module_lowering_plan]\n");
    out.push_str(&format!(
        "contract = \"{}\"\n",
        escape_toml_string(MODULE_LOWERING_PLAN_CONTRACT)
    ));
    out.push_str(&format!(
        "source_schema = \"{}\"\n",
        escape_toml_string(MODULE_SUMMARY_SCHEMA)
    ));
    out.push_str("source_language = \"wgsl\"\n");
    out.push_str(&format!(
        "lowering_boundary = \"{}\"\n",
        escape_toml_string(MODULE_LOWERING_BOUNDARY)
    ));
    out.push_str(&format!(
        "profile_lowering_target = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "backend = \"{}\"\n",
        escape_toml_string(spec.backend)
    ));
    out.push_str(&format!(
        "target = \"{}\"\n",
        escape_toml_string(spec.target)
    ));
    out.push_str(&format!(
        "lowering_target = \"{}\"\n",
        escape_toml_string(spec.lowering_target)
    ));
    out.push_str(&format!(
        "native_ir = \"{}\"\n",
        escape_toml_string(spec.native_ir)
    ));
    out.push_str("requires_translation = true\n");
    out.push_str(&format!(
        "binding_slot_model = \"{}\"\n",
        escape_toml_string(spec.binding_slot_model)
    ));
    out.push_str(&format!(
        "target_entry_policy = \"{}\"\n",
        escape_toml_string(spec.target_entry_policy)
    ));
    out.push_str(&format!(
        "accepted_stage_kinds = {}\n",
        render_string_array(
            &supported_stages
                .iter()
                .map(|stage| (*stage).to_owned())
                .collect::<Vec<_>>()
        )
    ));
    for stage in supported_stages {
        out.push_str("\n[[module_lowering_plan.stage]]\n");
        out.push_str(&format!("kind = \"{}\"\n", escape_toml_string(stage)));
        out.push_str(&format!(
            "execution_model = \"{}\"\n",
            escape_toml_string(&execution_model_for_stage(stage, spec.stage_model))
        ));
        out.push_str(&format!(
            "target_entry_policy = \"{}\"\n",
            escape_toml_string(spec.target_entry_policy)
        ));
    }
    out
}

fn execution_model_for_stage(stage: &str, model: StageModel) -> String {
    match model {
        StageModel::Msl => match stage {
            "vertex" => "vertex".to_owned(),
            "fragment" => "fragment".to_owned(),
            "compute" => "kernel".to_owned(),
            other => format!("user({other})"),
        },
        StageModel::Spirv => match stage {
            "vertex" => "Vertex".to_owned(),
            "fragment" => "Fragment".to_owned(),
            "compute" => "GLCompute".to_owned(),
            other => format!("User({other})"),
        },
        StageModel::Dxil => match stage {
            "vertex" => "vs".to_owned(),
            "fragment" => "ps".to_owned(),
            "compute" => "cs".to_owned(),
            other => format!("lib({other})"),
        },
        StageModel::Glsl => match stage {
            "vertex" => "vertex".to_owned(),
            "fragment" => "fragment".to_owned(),
            "compute" => "compute".to_owned(),
            other => format!("shader({other})"),
        },
        StageModel::HostSimd => format!("host-{stage}"),
    }
}
