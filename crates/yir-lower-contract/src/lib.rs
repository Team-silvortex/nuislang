use yir_core::YirModule;

mod backend_variants;
mod contract_render;
mod contract_render_shader;
mod kernel_analysis;
mod shader_analysis;
mod shader_backend_plan;
mod shader_ir;

pub use kernel_analysis::analyze_kernel_lowering;
pub use shader_analysis::analyze_shader_lowering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelLoweringContract {
    pub stages: Vec<KernelStageContract>,
    pub graphs: Vec<KernelComputeGraphContract>,
    pub fabric_handle_tables: Vec<FabricHandleTableContract>,
    pub fabric_core_bindings: Vec<FabricCoreBindingContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLoweringContract {
    pub stages: Vec<ShaderStageContract>,
    pub fabric_handle_tables: Vec<FabricHandleTableContract>,
    pub fabric_core_bindings: Vec<FabricCoreBindingContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderStageContract {
    pub node: String,
    pub op: String,
    pub resource: String,
    pub lowering: ShaderLoweringMode,
    pub reason: String,
    pub pipeline: Option<String>,
    pub target_format: Option<String>,
    pub topology: Option<String>,
    pub wgsl_entry: Option<String>,
    pub wgsl_source: Option<String>,
    pub fabric_handle_table: Option<String>,
    pub bindings: Vec<ShaderResourceBinding>,
    pub blend_mode: Option<String>,
    pub blend_enabled: Option<bool>,
    pub depth_compare: Option<String>,
    pub depth_test_enabled: Option<bool>,
    pub depth_write_enabled: Option<bool>,
    pub cull_mode: Option<String>,
    pub front_face: Option<String>,
    pub shader_module: Option<ShaderModuleContract>,
    pub shader_module_lowering_plans: Vec<ShaderModuleBackendLoweringPlan>,
    pub shader_ir_stages: Vec<ShaderIrStageContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModuleContract {
    pub schema: String,
    pub resource: String,
    pub entry: String,
    pub source_language: String,
    pub stages: Vec<ShaderModuleStageContract>,
    pub bindings: Vec<ShaderModuleBindingContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModuleStageContract {
    pub stage: String,
    pub entry: String,
    pub attributes: Vec<String>,
    pub workgroup_size: Option<String>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModuleBindingContract {
    pub group: u32,
    pub binding: u32,
    pub name: String,
    pub kind: String,
    pub address_space: Option<String>,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModuleBackendLoweringPlan {
    pub contract: String,
    pub backend: String,
    pub target: String,
    pub lowering_target: String,
    pub native_ir: String,
    pub source_schema: String,
    pub source_language: String,
    pub resource: String,
    pub entry: String,
    pub requires_translation: bool,
    pub lowering_boundary: String,
    pub stage_entries: Vec<ShaderModuleBackendStageEntry>,
    pub resource_bindings: Vec<ShaderModuleBackendBindingEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModuleBackendStageEntry {
    pub stage: String,
    pub source_entry: String,
    pub target_entry: String,
    pub execution_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderModuleBackendBindingEntry {
    pub group: u32,
    pub binding: u32,
    pub name: String,
    pub kind: String,
    pub address_space: Option<String>,
    pub ty: String,
    pub target_slot: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarContractStage {
    pub stage: String,
    pub function: String,
    pub node_kind: String,
    pub execution_domain: String,
    pub time_mode: String,
    pub contract_family: String,
    pub time_domain: String,
    pub glm_scope: String,
    pub instructions: Vec<NustarContractInstruction>,
    pub terminator: NustarContractTerminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarContractInstruction {
    pub result: String,
    pub ty: Option<String>,
    pub op: String,
    pub args: Vec<String>,
    pub expr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarContractTerminator {
    pub op: String,
    pub expr: String,
}

pub type ShaderIrStageContract = NustarContractStage;
pub type ShaderIrInstruction = NustarContractInstruction;
pub type ShaderIrTerminator = NustarContractTerminator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricHandleTableContract {
    pub node: String,
    pub entries: Vec<FabricHandleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricHandleEntry {
    pub slot: String,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FabricCoreBindingContract {
    pub node: String,
    pub resource: String,
    pub core_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderResourceBinding {
    pub slot: usize,
    pub kind: String,
    pub source: String,
    pub texture_format: Option<String>,
    pub texture_width: Option<usize>,
    pub texture_height: Option<usize>,
    pub sampler_filter: Option<String>,
    pub sampler_address_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderBackendVariant {
    pub backend: &'static str,
    pub backend_family: &'static str,
    pub target_os: &'static str,
    pub target_device: &'static str,
    pub ir_format: &'static str,
    pub dispatch_abi: &'static str,
    pub kind: &'static str,
    pub priority: usize,
    pub status: &'static str,
    pub verification: &'static str,
    pub entry: String,
    pub artifact: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelStageContract {
    pub node: String,
    pub function: String,
    pub node_kind: String,
    pub execution_domain: String,
    pub time_mode: String,
    pub op: String,
    pub resource: String,
    pub lowering: KernelLoweringMode,
    pub reason: String,
    pub target_arch: Option<String>,
    pub target_runtime: Option<String>,
    pub lane_width: Option<usize>,
    pub rows: Option<usize>,
    pub cols: Option<usize>,
    pub axis: Option<String>,
    pub topk: Option<usize>,
    pub inputs: Vec<String>,
    pub fabric_handle_table: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelBackendVariant {
    pub backend: &'static str,
    pub backend_family: &'static str,
    pub target_os: &'static str,
    pub target_device: &'static str,
    pub ir_format: &'static str,
    pub dispatch_abi: &'static str,
    pub kind: &'static str,
    pub priority: usize,
    pub status: &'static str,
    pub verification: &'static str,
    pub entry: String,
    pub artifact: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelComputeGraphContract {
    pub id: String,
    pub function: String,
    pub node_kind: String,
    pub execution_domain: String,
    pub time_mode: String,
    pub resource: String,
    pub lowering: KernelLoweringMode,
    pub reason: String,
    pub target_arch: Option<String>,
    pub target_runtime: Option<String>,
    pub lane_width: Option<usize>,
    pub stages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderLoweringMode {
    BackendEligible,
    PrerenderOnly,
}

impl ShaderLoweringMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackendEligible => "backend_eligible",
            Self::PrerenderOnly => "prerender_only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelLoweringMode {
    BackendEligible,
    CpuFallbackOnly,
}

impl KernelLoweringMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackendEligible => "backend_eligible",
            Self::CpuFallbackOnly => "cpu_fallback_only",
        }
    }
}

fn collect_fabric_handle_tables(module: &YirModule) -> Vec<FabricHandleTableContract> {
    module
        .nodes
        .iter()
        .filter(|node| {
            matches!(node.op.module.as_str(), "data" | "fabric")
                && node.op.instruction == "handle_table"
        })
        .map(|node| FabricHandleTableContract {
            node: node.name.clone(),
            entries: node
                .op
                .args
                .iter()
                .filter_map(|entry| entry.split_once('='))
                .map(|(slot, resource)| FabricHandleEntry {
                    slot: slot.trim().to_owned(),
                    resource: resource.trim().to_owned(),
                })
                .collect(),
        })
        .collect()
}

fn collect_fabric_core_bindings(module: &YirModule) -> Vec<FabricCoreBindingContract> {
    module
        .nodes
        .iter()
        .filter(|node| {
            matches!(node.op.module.as_str(), "data" | "fabric")
                && node.op.instruction == "bind_core"
                && node.op.args.len() == 1
        })
        .filter_map(|node| {
            node.op.args[0]
                .parse::<usize>()
                .ok()
                .map(|core_index| FabricCoreBindingContract {
                    node: node.name.clone(),
                    resource: node.resource.clone(),
                    core_index,
                })
        })
        .collect()
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_module(source: &str) -> YirModule {
        yir_syntax::parse_module(source).expect("module should parse")
    }

    #[test]
    fn kernel_contract_marks_coreml_matmul_pipeline_backend_eligible() {
        let module = parse_module(
            r#"yir 0.1

resource kernel0 kernel.apple
resource fabric0 data.fabric

data.handle_table handles fabric0 host=cpu0,compute=kernel0
kernel.target_config profile kernel0 apple_ane coreml 128
kernel.tensor input kernel0 1 3 2,4,6
kernel.tensor weights kernel0 3 2 1,-2,3,0,2,1
kernel.matmul projected kernel0 input weights
kernel.print trace kernel0 projected
"#,
        );

        let contract = analyze_kernel_lowering(&module);
        assert!(contract.has_kernel_work());
        assert!(contract.has_backend_eligible_work());
        assert!(!contract.requires_cpu_fallback());
        assert_eq!(contract.stages.len(), 3);
        assert_eq!(contract.graphs.len(), 1);
        assert_eq!(contract.stages[0].node_kind, "function-node");
        assert_eq!(contract.stages[0].execution_domain, "kernel");
        assert_eq!(contract.stages[0].time_mode, "logical");
        assert!(contract.stages[0].function.starts_with("kernel."));
        assert_eq!(
            contract.graphs[0].lowering,
            KernelLoweringMode::BackendEligible
        );
        assert_eq!(contract.graphs[0].node_kind, "function-graph");
        assert_eq!(contract.graphs[0].execution_domain, "kernel");
        assert_eq!(contract.graphs[0].time_mode, "logical");
        assert!(contract.graphs[0].function.starts_with("kernel.graph."));
        assert!(contract
            .render_package_manifest()
            .contains("package_kind = \"kernel_package\""));
        assert!(contract
            .render_package_manifest()
            .contains("execution_domain = \"kernel\""));
        let manifest = contract.render_package_manifest();
        assert!(manifest.contains("backend = \"coreml\""));
        assert!(manifest.contains("backend_family = \"npu\""));
        assert!(manifest.contains("target_device = \"apple-ane\""));
        assert!(manifest.contains("ir_format = \"mlpackage\""));
        assert!(manifest.contains("dispatch_abi = \"coreml-predict\""));
        assert!(manifest.contains("priority = 10"));
        assert!(manifest.contains("verification = \"contract-only\""));
        assert!(manifest.contains("[[graph]]"));
    }

    #[test]
    fn kernel_contract_marks_topk_as_cpu_fallback_only() {
        let module = parse_module(
            r#"yir 0.1

resource kernel0 kernel.apple

kernel.target_config profile kernel0 apple_ane coreml 128
kernel.tensor base kernel0 2 4 9,2,7,5,4,8,1,6
kernel.topk top_rows kernel0 base 2
kernel.print trace kernel0 top_rows
"#,
        );

        let contract = analyze_kernel_lowering(&module);
        assert!(contract.has_kernel_work());
        assert!(contract.has_backend_eligible_work());
        assert!(contract.requires_cpu_fallback());
        assert_eq!(contract.stages.len(), 2);
        assert_eq!(contract.graphs.len(), 1);
        assert_eq!(
            contract.graphs[0].lowering,
            KernelLoweringMode::CpuFallbackOnly
        );
        let topk_stage = contract
            .stages
            .iter()
            .find(|stage| stage.node == "top_rows")
            .expect("topk stage should be present");
        assert_eq!(topk_stage.node_kind, "function-node");
        assert_eq!(topk_stage.execution_domain, "kernel");
        assert_eq!(topk_stage.lowering, KernelLoweringMode::CpuFallbackOnly);
        assert!(contract.render_text().contains("node_kind=function-graph"));
        assert!(contract.render_text().contains("cpu_fallback_only"));
        assert!(contract
            .render_package_manifest()
            .contains("backend = \"cpu-fallback\""));
    }

    #[test]
    fn kernel_contract_exposes_cuda_ptx_variant_without_provider_coupling() {
        let module = parse_module(
            r#"yir 0.1

resource kernel0 kernel.cuda

kernel.target_config profile kernel0 nvidia_gpu cuda 256
kernel.tensor lhs kernel0 1 4 1,2,3,4
kernel.tensor rhs kernel0 1 4 10,20,30,40
kernel.add output kernel0 lhs rhs
"#,
        );

        let contract = analyze_kernel_lowering(&module);
        assert!(contract.has_backend_eligible_work());
        assert!(!contract.requires_cpu_fallback());
        let manifest = contract.render_package_manifest();
        assert!(manifest.contains("backend = \"cuda\""));
        assert!(manifest.contains("target_device = \"nvidia-gpu\""));
        assert!(manifest.contains("ir_format = \"ptx8.0\""));
        assert!(manifest.contains("dispatch_abi = \"cuda-driver-launch\""));
        assert!(manifest.contains("artifact = \"cuda/"));
    }

    #[test]
    fn shader_contract_extracts_fragment_shader_ir() {
        let module = parse_module(
            r#"yir 0.1

resource shader0 shader.render

shader.target main_target shader0 rgba8_unorm 40 24
shader.viewport main_view shader0 40 24
shader.pipeline lit_pipe shader0 lit_sphere triangle_strip
shader.inline_wgsl lit_pipe_wgsl shader0 lit_sphere "@group(0)\n@binding(0)\nvar albedo_sampler: sampler;\n@group(0)\n@binding(1)\nvar albedo_texture: texture_2d<f32>;\nstruct VsOut {\n  @builtin(position) pos: vec4<f32>,\n  @location(0) uv: vec2<f32>,\n};\n\n@vertex\nfn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {\n  var out: VsOut;\n  out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);\n  out.uv = vec2<f32>(0.0, 0.0);\n  return out;\n}\n\n@fragment\nfn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {\n  let uv2: vec2<f32> = clamp(uv, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));\n  let sampled: vec4<f32> = textureSample(albedo_texture, albedo_sampler, uv2);\n  let mixed: vec3<f32> = mix(sampled.xyz, vec3<f32>(uv2.xy, 1.0), 0.35);\n  return vec4<f32>(mixed.xyz, sampled.w);\n}"
shader.texture2d checker shader0 r8_unorm 2 2 8,16,24,32
shader.sampler clamp_sampler shader0 nearest clamp
shader.uniform material_uniform shader0 0 lit_pipe
shader.attachment color_attachment shader0 1 main_target
shader.texture_binding albedo_texture shader0 2 checker
shader.sampler_binding albedo_sampler shader0 3 clamp_sampler
shader.bind_set material_bindings shader0 lit_pipe material_uniform color_attachment albedo_texture albedo_sampler
shader.begin_pass main_pass shader0 main_target lit_pipe main_view
shader.draw_instanced frame shader0 main_pass lit_pipe 4 1 material_bindings
"#,
        );

        let contract = analyze_shader_lowering(&module);
        let stage = contract
            .stages
            .iter()
            .find(|stage| stage.node == "frame")
            .expect("frame stage should be present");
        let shader_ir = stage
            .shader_ir_stages
            .iter()
            .find(|shader_ir| shader_ir.stage == "fragment")
            .expect("fragment shader ir should exist");
        assert_eq!(shader_ir.stage, "fragment");
        assert_eq!(shader_ir.function, "shader.fragment");
        assert_eq!(shader_ir.node_kind, "function-node");
        assert_eq!(shader_ir.execution_domain, "shader");
        assert_eq!(shader_ir.time_mode, "logical");
        assert_eq!(shader_ir.contract_family, "nustar.shader");
        assert_eq!(shader_ir.time_domain, "shader.stage.fragment");
        assert_eq!(shader_ir.glm_scope, "shader::fragment");
        assert_eq!(shader_ir.instructions.len(), 3);
        assert_eq!(shader_ir.instructions[0].result, "uv2");
        assert_eq!(shader_ir.instructions[0].op, "clamp");
        assert_eq!(shader_ir.instructions[1].op, "sample_texture");
        assert_eq!(shader_ir.terminator.op, "return");
        let shader_module = stage
            .shader_module
            .as_ref()
            .expect("shader module summary should exist");
        assert_eq!(shader_module.schema, "nuis-yir.shader.module-summary.v1");
        assert_eq!(shader_module.entry, "lit_sphere");
        assert_eq!(shader_module.source_language, "wgsl");
        assert_eq!(shader_module.stages.len(), 2);
        assert_eq!(shader_module.bindings.len(), 2);
        assert!(shader_module
            .stages
            .iter()
            .any(|stage| stage.stage == "vertex" && stage.entry == "vs_main"));
        assert!(shader_module.bindings.iter().any(|binding| {
            binding.group == 0
                && binding.binding == 0
                && binding.name == "albedo_sampler"
                && binding.kind == "sampler"
        }));
        assert!(shader_module.bindings.iter().any(|binding| {
            binding.group == 0
                && binding.binding == 1
                && binding.name == "albedo_texture"
                && binding.kind == "texture"
        }));
        assert_eq!(stage.shader_module_lowering_plans.len(), 5);
        let spirv_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "spirv:vulkan-gpu")
            .expect("SPIR-V lowering plan should exist");
        assert_eq!(
            spirv_plan.contract,
            "nuis-yir.shader.backend-lowering-plan.v1"
        );
        assert_eq!(spirv_plan.native_ir, "spirv1.6");
        assert_eq!(spirv_plan.source_schema, shader_module.schema);
        assert!(spirv_plan.stage_entries.iter().any(|entry| {
            entry.stage == "fragment"
                && entry.source_entry == "fs_main"
                && entry.execution_model == "Fragment"
        }));
        assert!(spirv_plan.resource_bindings.iter().any(|binding| {
            binding.name == "albedo_texture" && binding.target_slot == "set0.binding1"
        }));
        let msl_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "msl:metal-gpu")
            .expect("MSL lowering plan should exist");
        assert!(msl_plan.resource_bindings.iter().any(|binding| {
            binding.name == "albedo_texture" && binding.target_slot == "argument-buffer[0].slot1"
        }));
        let host_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "host-simd:cpu-fallback")
            .expect("host SIMD fallback lowering plan should exist");
        assert!(host_plan
            .stage_entries
            .iter()
            .any(|entry| entry.target_entry == "host_vs_main"));
        let dxil_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "dxil:directx-gpu")
            .expect("DXIL lowering plan should exist");
        assert!(dxil_plan.resource_bindings.iter().any(|binding| {
            binding.name == "albedo_texture" && binding.target_slot == "root-signature[0].slot1"
        }));
        let glsl_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "glsl:opengl-gpu")
            .expect("GLSL lowering plan should exist");
        assert!(glsl_plan.resource_bindings.iter().any(|binding| {
            binding.name == "albedo_texture" && binding.target_slot == "uniform-binding[0:1]"
        }));
        assert!(stage
            .shader_ir_stages
            .iter()
            .any(|shader_ir| shader_ir.stage == "vertex"));
        assert!(contract.render_text().contains("shader_ir_stage=fragment"));
        assert!(contract
            .render_text()
            .contains("shader_module_schema=nuis-yir.shader.module-summary.v1"));
        assert!(contract
            .render_text()
            .contains("shader_module_binding group=0 binding=1 name=albedo_texture"));
        assert!(contract
            .render_text()
            .contains("shader_module_lowering_plan contract=nuis-yir.shader.backend-lowering-plan.v1 lowering_target=spirv:vulkan-gpu"));
        assert!(contract.render_text().contains(
            "lowering_binding group=0 binding=1 name=albedo_texture target_slot=set0.binding1"
        ));
        assert!(contract
            .render_text()
            .contains("shader_ir_function=shader.fragment"));
        assert!(contract
            .render_text()
            .contains("shader_ir_contract_family=nustar.shader"));
        let manifest = contract.render_package_manifest();
        assert!(manifest.contains("shader_ir_instruction_count = 3"));
        assert!(manifest.contains("shader_module_schema = \"nuis-yir.shader.module-summary.v1\""));
        assert!(manifest.contains("shader_module_stage_count = 2"));
        assert!(manifest.contains("shader_module_binding_count = 2"));
        assert!(manifest.contains("[[stage.shader_module_binding]]"));
        assert!(manifest.contains("[[stage.shader_module_lowering_plan]]"));
        assert!(manifest.contains("contract = \"nuis-yir.shader.backend-lowering-plan.v1\""));
        assert!(manifest.contains("lowering_target = \"spirv:vulkan-gpu\""));
        assert!(manifest.contains("target_slot = \"argument-buffer[0].slot1\""));
        assert!(manifest.contains("shader_ir_execution_domain = \"shader\""));
        assert!(manifest.contains("shader_ir_time_domain = \"shader.stage.fragment\""));
        assert!(manifest.contains("backend = \"webgpu\""));
        assert!(manifest.contains("backend_family = \"gpu\""));
        assert!(manifest.contains("target_device = \"webgpu-device\""));
        assert!(manifest.contains("ir_format = \"wgsl\""));
        assert!(manifest.contains("dispatch_abi = \"webgpu-render-pipeline\""));
        assert!(manifest.contains("priority = 40"));
        assert!(manifest.contains("verification = \"contract-only\""));
    }

    #[test]
    fn shader_contract_extracts_compute_shader_ir_stage() {
        let module = parse_module(
            r#"yir 0.1

resource shader0 shader.render

shader.target main_target shader0 rgba8_unorm 40 24
shader.viewport main_view shader0 40 24
shader.pipeline lit_pipe shader0 lit_sphere triangle_strip
shader.inline_wgsl lit_pipe_wgsl shader0 lit_sphere "// @compute\n// fn fake_cs() {}\n@vertex\nfn vs_main(@builtin(vertex_index) vid: u32) -> vec4<f32> {\n  return vec4<f32>(f32(vid), 0.0, 0.0, 1.0);\n}\n\n@fragment\nfn fs_main() -> @location(0) vec4<f32> {\n  return vec4<f32>(1.0, 1.0, 1.0, 1.0);\n}\n\n@compute @workgroup_size(8, 1, 1)\nfn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {\n  let idx: u32 = gid.x;\n}"
shader.begin_pass main_pass shader0 main_target lit_pipe main_view
shader.draw_instanced frame shader0 main_pass lit_pipe 4 1 lit_pipe
"#,
        );

        let contract = analyze_shader_lowering(&module);
        let stage = contract
            .stages
            .iter()
            .find(|stage| stage.node == "frame")
            .expect("frame stage should be present");
        let compute_ir = stage
            .shader_ir_stages
            .iter()
            .find(|shader_ir| shader_ir.stage == "compute")
            .expect("compute shader ir should exist");

        assert_eq!(
            stage
                .shader_ir_stages
                .iter()
                .filter(|shader_ir| shader_ir.stage == "compute")
                .count(),
            1
        );
        assert_eq!(compute_ir.function, "shader.compute");
        assert_eq!(compute_ir.time_domain, "shader.stage.compute");
        assert_eq!(compute_ir.glm_scope, "shader::compute");
        assert_eq!(compute_ir.instructions.len(), 1);
        assert_eq!(compute_ir.instructions[0].result, "idx");
        assert_eq!(compute_ir.terminator.op, "end");
        assert_eq!(compute_ir.terminator.expr, "void");
        let shader_module = stage
            .shader_module
            .as_ref()
            .expect("shader module summary should exist");
        assert_eq!(shader_module.stages.len(), 3);
        assert!(shader_module.stages.iter().any(|stage| {
            stage.stage == "compute"
                && stage.entry == "cs_main"
                && stage.workgroup_size.as_deref() == Some("8, 1, 1")
        }));
        let spirv_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "spirv:vulkan-gpu")
            .expect("SPIR-V lowering plan should exist");
        assert!(spirv_plan.stage_entries.iter().any(|entry| {
            entry.stage == "compute"
                && entry.source_entry == "cs_main"
                && entry.execution_model == "GLCompute"
        }));
        let msl_plan = stage
            .shader_module_lowering_plans
            .iter()
            .find(|plan| plan.lowering_target == "msl:metal-gpu")
            .expect("MSL lowering plan should exist");
        assert!(msl_plan.stage_entries.iter().any(|entry| {
            entry.stage == "compute"
                && entry.source_entry == "cs_main"
                && entry.execution_model == "kernel"
        }));
        assert!(contract.render_text().contains("shader_ir_stage=compute"));
        let manifest = contract.render_package_manifest();
        assert!(manifest.contains("shader_ir_stage = \"compute\""));
        assert!(manifest.contains("kind = \"compute\""));
        assert!(manifest.contains("workgroup_size = \"8, 1, 1\""));
        assert!(manifest.contains("shader_ir_terminator_op = \"end\""));
    }
}
