use super::{
    ShaderModuleBackendBindingEntry, ShaderModuleBackendLoweringPlan,
    ShaderModuleBackendStageEntry, ShaderModuleBindingContract, ShaderModuleContract,
    ShaderModuleStageContract,
};

const SHADER_MODULE_BACKEND_PLAN_CONTRACT: &str = "nuis-yir.shader.backend-lowering-plan.v1";
const SHADER_MODULE_LOWERING_BOUNDARY: &str = "module-summary-to-native-ir";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShaderBackendLoweringSpec {
    backend: &'static str,
    target: &'static str,
    lowering_target: &'static str,
    native_ir: &'static str,
    entry_prefix: &'static str,
    stage_model: ShaderStageModel,
    binding_model: ShaderBindingModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShaderStageModel {
    Spirv,
    Msl,
    Dxil,
    Glsl,
    HostSimd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShaderBindingModel {
    DescriptorSetBinding,
    MetalArgumentBuffer,
    RootSignatureSlot,
    UniformBindingSlot,
    HostTupleSlot,
}

const SHADER_BACKEND_LOWERING_SPECS: &[ShaderBackendLoweringSpec] = &[
    ShaderBackendLoweringSpec {
        backend: "spirv",
        target: "vulkan-gpu",
        lowering_target: "spirv:vulkan-gpu",
        native_ir: "spirv1.6",
        entry_prefix: "",
        stage_model: ShaderStageModel::Spirv,
        binding_model: ShaderBindingModel::DescriptorSetBinding,
    },
    ShaderBackendLoweringSpec {
        backend: "msl",
        target: "metal-gpu",
        lowering_target: "msl:metal-gpu",
        native_ir: "msl2.4",
        entry_prefix: "",
        stage_model: ShaderStageModel::Msl,
        binding_model: ShaderBindingModel::MetalArgumentBuffer,
    },
    ShaderBackendLoweringSpec {
        backend: "dxil",
        target: "directx-gpu",
        lowering_target: "dxil:directx-gpu",
        native_ir: "dxil6.8",
        entry_prefix: "",
        stage_model: ShaderStageModel::Dxil,
        binding_model: ShaderBindingModel::RootSignatureSlot,
    },
    ShaderBackendLoweringSpec {
        backend: "glsl",
        target: "opengl-gpu",
        lowering_target: "glsl:opengl-gpu",
        native_ir: "glsl460",
        entry_prefix: "",
        stage_model: ShaderStageModel::Glsl,
        binding_model: ShaderBindingModel::UniformBindingSlot,
    },
    ShaderBackendLoweringSpec {
        backend: "host-simd",
        target: "cpu-fallback",
        lowering_target: "host-simd:cpu-fallback",
        native_ir: "nuis-host-simd-yir",
        entry_prefix: "host_",
        stage_model: ShaderStageModel::HostSimd,
        binding_model: ShaderBindingModel::HostTupleSlot,
    },
];

pub(super) fn build_shader_module_backend_lowering_plans(
    module: &ShaderModuleContract,
) -> Vec<ShaderModuleBackendLoweringPlan> {
    SHADER_BACKEND_LOWERING_SPECS
        .iter()
        .map(|spec| ShaderModuleBackendLoweringPlan {
            contract: SHADER_MODULE_BACKEND_PLAN_CONTRACT.to_owned(),
            backend: spec.backend.to_owned(),
            target: spec.target.to_owned(),
            lowering_target: spec.lowering_target.to_owned(),
            native_ir: spec.native_ir.to_owned(),
            source_schema: module.schema.clone(),
            source_language: module.source_language.clone(),
            resource: module.resource.clone(),
            entry: module.entry.clone(),
            requires_translation: true,
            lowering_boundary: SHADER_MODULE_LOWERING_BOUNDARY.to_owned(),
            stage_entries: module
                .stages
                .iter()
                .map(|stage| build_stage_entry(stage, spec))
                .collect(),
            resource_bindings: module
                .bindings
                .iter()
                .map(|binding| build_binding_entry(binding, spec))
                .collect(),
        })
        .collect()
}

fn build_stage_entry(
    stage: &ShaderModuleStageContract,
    spec: &ShaderBackendLoweringSpec,
) -> ShaderModuleBackendStageEntry {
    ShaderModuleBackendStageEntry {
        stage: stage.stage.clone(),
        source_entry: stage.entry.clone(),
        target_entry: format!("{}{}", spec.entry_prefix, stage.entry),
        execution_model: execution_model_for_stage(&stage.stage, spec.stage_model),
    }
}

fn build_binding_entry(
    binding: &ShaderModuleBindingContract,
    spec: &ShaderBackendLoweringSpec,
) -> ShaderModuleBackendBindingEntry {
    ShaderModuleBackendBindingEntry {
        group: binding.group,
        binding: binding.binding,
        name: binding.name.clone(),
        kind: binding.kind.clone(),
        address_space: binding.address_space.clone(),
        ty: binding.ty.clone(),
        target_slot: target_slot_for_binding(binding, spec.binding_model),
    }
}

fn execution_model_for_stage(stage: &str, model: ShaderStageModel) -> String {
    match model {
        ShaderStageModel::Spirv => match stage {
            "vertex" => "Vertex".to_owned(),
            "fragment" => "Fragment".to_owned(),
            "compute" => "GLCompute".to_owned(),
            other => format!("User({other})"),
        },
        ShaderStageModel::Msl => match stage {
            "vertex" => "vertex".to_owned(),
            "fragment" => "fragment".to_owned(),
            "compute" => "kernel".to_owned(),
            other => format!("user({other})"),
        },
        ShaderStageModel::Dxil => match stage {
            "vertex" => "vs".to_owned(),
            "fragment" => "ps".to_owned(),
            "compute" => "cs".to_owned(),
            other => format!("lib({other})"),
        },
        ShaderStageModel::Glsl => match stage {
            "vertex" => "vertex".to_owned(),
            "fragment" => "fragment".to_owned(),
            "compute" => "compute".to_owned(),
            other => format!("shader({other})"),
        },
        ShaderStageModel::HostSimd => format!("host-{stage}"),
    }
}

fn target_slot_for_binding(
    binding: &ShaderModuleBindingContract,
    model: ShaderBindingModel,
) -> String {
    match model {
        ShaderBindingModel::DescriptorSetBinding => {
            format!("set{}.binding{}", binding.group, binding.binding)
        }
        ShaderBindingModel::MetalArgumentBuffer => {
            format!("argument-buffer[{}].slot{}", binding.group, binding.binding)
        }
        ShaderBindingModel::RootSignatureSlot => {
            format!("root-signature[{}].slot{}", binding.group, binding.binding)
        }
        ShaderBindingModel::UniformBindingSlot => {
            format!("uniform-binding[{}:{}]", binding.group, binding.binding)
        }
        ShaderBindingModel::HostTupleSlot => {
            format!("host.bind[{}:{}]", binding.group, binding.binding)
        }
    }
}
