use yir_core::YirModule;

mod backend_variants;
mod contract_render;
mod contract_render_shader;
mod kernel_analysis;
mod shader_analysis;
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
    pub shader_ir_stages: Vec<ShaderIrStageContract>,
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
    fn shader_contract_extracts_fragment_shader_ir() {
        let module = parse_module(
            r#"yir 0.1

resource shader0 shader.render

shader.target main_target shader0 rgba8_unorm 40 24
shader.viewport main_view shader0 40 24
shader.pipeline lit_pipe shader0 lit_sphere triangle_strip
shader.inline_wgsl lit_pipe_wgsl shader0 lit_sphere "struct VsOut {\n  @builtin(position) pos: vec4<f32>,\n  @location(0) uv: vec2<f32>,\n};\n\n@vertex\nfn vs_main(@builtin(vertex_index) vid: u32) -> VsOut {\n  var out: VsOut;\n  out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);\n  out.uv = vec2<f32>(0.0, 0.0);\n  return out;\n}\n\n@fragment\nfn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {\n  let uv2: vec2<f32> = clamp(uv, vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0));\n  let sampled: vec4<f32> = textureSample(albedo_texture, albedo_sampler, uv2);\n  let mixed: vec3<f32> = mix(sampled.xyz, vec3<f32>(uv2.xy, 1.0), 0.35);\n  return vec4<f32>(mixed.xyz, sampled.w);\n}"
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
        assert!(stage
            .shader_ir_stages
            .iter()
            .any(|shader_ir| shader_ir.stage == "vertex"));
        assert!(contract.render_text().contains("shader_ir_stage=fragment"));
        assert!(contract
            .render_text()
            .contains("shader_ir_function=shader.fragment"));
        assert!(contract
            .render_text()
            .contains("shader_ir_contract_family=nustar.shader"));
        let manifest = contract.render_package_manifest();
        assert!(manifest.contains("shader_ir_instruction_count = 3"));
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
}
