use std::collections::BTreeMap;

use yir_core::{EdgeKind, Node, YirModule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelLoweringContract {
    pub stages: Vec<KernelStageContract>,
    pub graphs: Vec<KernelComputeGraphContract>,
    pub fabric_handle_tables: Vec<FabricHandleTableContract>,
    pub fabric_core_bindings: Vec<FabricCoreBindingContract>,
}

impl KernelLoweringContract {
    pub fn has_kernel_work(&self) -> bool {
        !self.stages.is_empty()
    }

    pub fn has_backend_eligible_work(&self) -> bool {
        self.stages
            .iter()
            .any(|stage| stage.lowering == KernelLoweringMode::BackendEligible)
    }

    pub fn requires_cpu_fallback(&self) -> bool {
        self.stages
            .iter()
            .any(|stage| stage.lowering == KernelLoweringMode::CpuFallbackOnly)
    }

    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();
        for table in &self.fabric_handle_tables {
            lines.push(format!("fabric_handle_table={}", table.node));
            for entry in &table.entries {
                lines.push(format!("  handle {} -> {}", entry.slot, entry.resource));
            }
        }
        for binding in &self.fabric_core_bindings {
            lines.push(format!(
                "fabric_core_binding={} resource={} core={}",
                binding.node, binding.resource, binding.core_index
            ));
        }
        for graph in &self.graphs {
            lines.push(format!(
                "graph={} resource={} lowering={} reason={}",
                graph.id,
                graph.resource,
                graph.lowering.as_str(),
                graph.reason
            ));
            lines.push(format!("  function={}", graph.function));
            lines.push(format!("  node_kind={}", graph.node_kind));
            lines.push(format!("  execution_domain={}", graph.execution_domain));
            lines.push(format!("  time_mode={}", graph.time_mode));
            if let Some(runtime) = &graph.target_runtime {
                lines.push(format!("  target_runtime={runtime}"));
            }
            if let Some(arch) = &graph.target_arch {
                lines.push(format!("  target_arch={arch}"));
            }
            if let Some(width) = graph.lane_width {
                lines.push(format!("  lane_width={width}"));
            }
            lines.push(format!("  stage_count={}", graph.stages.len()));
            for stage in &graph.stages {
                lines.push(format!("  stage={stage}"));
            }
        }
        for stage in &self.stages {
            lines.push(format!(
                "stage={} op={} resource={} lowering={} reason={}",
                stage.node,
                stage.op,
                stage.resource,
                stage.lowering.as_str(),
                stage.reason
            ));
            lines.push(format!("  function={}", stage.function));
            lines.push(format!("  node_kind={}", stage.node_kind));
            lines.push(format!("  execution_domain={}", stage.execution_domain));
            lines.push(format!("  time_mode={}", stage.time_mode));
            if let Some(runtime) = &stage.target_runtime {
                lines.push(format!("  target_runtime={runtime}"));
            }
            if let Some(arch) = &stage.target_arch {
                lines.push(format!("  target_arch={arch}"));
            }
            if let Some(width) = stage.lane_width {
                lines.push(format!("  lane_width={width}"));
            }
            if let Some(axis) = &stage.axis {
                lines.push(format!("  axis={axis}"));
            }
            if let Some(k) = stage.topk {
                lines.push(format!("  topk={k}"));
            }
            if let Some(rows) = stage.rows {
                lines.push(format!("  rows={rows}"));
            }
            if let Some(cols) = stage.cols {
                lines.push(format!("  cols={cols}"));
            }
            for input in &stage.inputs {
                lines.push(format!("  input={input}"));
            }
            if let Some(table) = &stage.fabric_handle_table {
                lines.push(format!("  fabric_handle_table={table}"));
            }
        }
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" }
    }

    pub fn render_package_manifest(&self) -> String {
        let mut out = String::new();
        out.push_str("manifest_version = 1\n");
        out.push_str("package_kind = \"kernel_package\"\n");
        out.push_str(&format!("graph_count = {}\n", self.graphs.len()));
        out.push_str(&format!("stage_count = {}\n", self.stages.len()));
        out.push_str(&format!(
            "fabric_handle_table_count = {}\n",
            self.fabric_handle_tables.len()
        ));
        out.push_str(&format!(
            "fabric_core_binding_count = {}\n",
            self.fabric_core_bindings.len()
        ));
        out.push_str(&format!(
            "backend_eligible = {}\n",
            self.has_backend_eligible_work()
        ));
        out.push_str(&format!(
            "requires_cpu_fallback = {}\n",
            self.requires_cpu_fallback()
        ));

        for table in &self.fabric_handle_tables {
            out.push_str("\n[[fabric_handle_table]]\n");
            out.push_str(&format!("id = \"{}\"\n", table.node));
            for entry in &table.entries {
                out.push_str("\n[[fabric_handle_table.entry]]\n");
                out.push_str(&format!("slot = \"{}\"\n", escape_toml(&entry.slot)));
                out.push_str(&format!(
                    "resource = \"{}\"\n",
                    escape_toml(&entry.resource)
                ));
            }
        }

        for binding in &self.fabric_core_bindings {
            out.push_str("\n[[fabric_core_binding]]\n");
            out.push_str(&format!("id = \"{}\"\n", binding.node));
            out.push_str(&format!(
                "resource = \"{}\"\n",
                escape_toml(&binding.resource)
            ));
            out.push_str(&format!("core_index = {}\n", binding.core_index));
        }

        for graph in &self.graphs {
            out.push_str("\n[[graph]]\n");
            out.push_str(&format!("id = \"{}\"\n", graph.id));
            out.push_str(&format!(
                "function = \"{}\"\n",
                escape_toml(&graph.function)
            ));
            out.push_str(&format!(
                "node_kind = \"{}\"\n",
                escape_toml(&graph.node_kind)
            ));
            out.push_str(&format!(
                "execution_domain = \"{}\"\n",
                escape_toml(&graph.execution_domain)
            ));
            out.push_str(&format!(
                "time_mode = \"{}\"\n",
                escape_toml(&graph.time_mode)
            ));
            out.push_str(&format!(
                "resource = \"{}\"\n",
                escape_toml(&graph.resource)
            ));
            out.push_str(&format!("lowering = \"{}\"\n", graph.lowering.as_str()));
            out.push_str(&format!("reason = \"{}\"\n", escape_toml(&graph.reason)));
            if let Some(runtime) = &graph.target_runtime {
                out.push_str(&format!("target_runtime = \"{}\"\n", escape_toml(runtime)));
            }
            if let Some(arch) = &graph.target_arch {
                out.push_str(&format!("target_arch = \"{}\"\n", escape_toml(arch)));
            }
            if let Some(width) = graph.lane_width {
                out.push_str(&format!("lane_width = {}\n", width));
            }
            for stage in &graph.stages {
                out.push_str("\n[[graph.stage]]\n");
                out.push_str(&format!("id = \"{}\"\n", escape_toml(stage)));
            }
            for variant in graph.backend_variants() {
                out.push_str(&render_kernel_variant("graph.variant", &variant));
            }
        }

        for stage in &self.stages {
            out.push_str("\n[[stage]]\n");
            out.push_str(&format!("id = \"{}\"\n", stage.node));
            out.push_str(&format!(
                "function = \"{}\"\n",
                escape_toml(&stage.function)
            ));
            out.push_str(&format!(
                "node_kind = \"{}\"\n",
                escape_toml(&stage.node_kind)
            ));
            out.push_str(&format!(
                "execution_domain = \"{}\"\n",
                escape_toml(&stage.execution_domain)
            ));
            out.push_str(&format!(
                "time_mode = \"{}\"\n",
                escape_toml(&stage.time_mode)
            ));
            out.push_str(&format!("op = \"{}\"\n", stage.op));
            out.push_str(&format!(
                "resource = \"{}\"\n",
                escape_toml(&stage.resource)
            ));
            out.push_str(&format!("lowering = \"{}\"\n", stage.lowering.as_str()));
            out.push_str(&format!("reason = \"{}\"\n", escape_toml(&stage.reason)));
            if let Some(runtime) = &stage.target_runtime {
                out.push_str(&format!("target_runtime = \"{}\"\n", escape_toml(runtime)));
            }
            if let Some(arch) = &stage.target_arch {
                out.push_str(&format!("target_arch = \"{}\"\n", escape_toml(arch)));
            }
            if let Some(width) = stage.lane_width {
                out.push_str(&format!("lane_width = {}\n", width));
            }
            if let Some(axis) = &stage.axis {
                out.push_str(&format!("axis = \"{}\"\n", escape_toml(axis)));
            }
            if let Some(k) = stage.topk {
                out.push_str(&format!("topk = {}\n", k));
            }
            if let Some(rows) = stage.rows {
                out.push_str(&format!("rows = {}\n", rows));
            }
            if let Some(cols) = stage.cols {
                out.push_str(&format!("cols = {}\n", cols));
            }
            if let Some(table) = &stage.fabric_handle_table {
                out.push_str(&format!(
                    "fabric_handle_table = \"{}\"\n",
                    escape_toml(table)
                ));
            }
            for input in &stage.inputs {
                out.push_str("\n[[stage.input]]\n");
                out.push_str(&format!("source = \"{}\"\n", escape_toml(input)));
            }
            for variant in stage.backend_variants() {
                out.push_str(&render_kernel_variant("stage.variant", &variant));
            }
        }

        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLoweringContract {
    pub stages: Vec<ShaderStageContract>,
    pub fabric_handle_tables: Vec<FabricHandleTableContract>,
    pub fabric_core_bindings: Vec<FabricCoreBindingContract>,
}

impl ShaderLoweringContract {
    pub fn has_shader_work(&self) -> bool {
        !self.stages.is_empty()
    }

    pub fn has_backend_eligible_work(&self) -> bool {
        self.stages
            .iter()
            .any(|stage| stage.lowering == ShaderLoweringMode::BackendEligible)
    }

    pub fn requires_prerender_fallback(&self) -> bool {
        self.stages
            .iter()
            .any(|stage| stage.lowering == ShaderLoweringMode::PrerenderOnly)
    }

    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();
        for table in &self.fabric_handle_tables {
            lines.push(format!("fabric_handle_table={}", table.node));
            for entry in &table.entries {
                lines.push(format!("  handle {} -> {}", entry.slot, entry.resource));
            }
        }
        for binding in &self.fabric_core_bindings {
            lines.push(format!(
                "fabric_core_binding={} resource={} core={}",
                binding.node, binding.resource, binding.core_index
            ));
        }
        for stage in &self.stages {
            lines.push(format!(
                "stage={} op={} resource={} lowering={} reason={}",
                stage.node,
                stage.op,
                stage.resource,
                stage.lowering.as_str(),
                stage.reason
            ));
            if let Some(table) = &stage.fabric_handle_table {
                lines.push(format!("  fabric_handle_table={}", table));
            }

            if let Some(pipeline) = &stage.pipeline {
                lines.push(format!("  pipeline={}", pipeline));
            }
            if let Some(target_format) = &stage.target_format {
                lines.push(format!("  target_format={}", target_format));
            }
            if let Some(topology) = &stage.topology {
                lines.push(format!("  topology={}", topology));
            }
            if let Some(entry) = &stage.wgsl_entry {
                lines.push(format!("  wgsl_entry={entry}"));
            }
            if let Some(source) = &stage.wgsl_source {
                lines.push(format!("  wgsl_source_lines={}", source.lines().count()));
            }
            for shader_ir in &stage.shader_ir_stages {
                lines.push(format!("  shader_ir_stage={}", shader_ir.stage));
                lines.push(format!("  shader_ir_function={}", shader_ir.function));
                lines.push(format!("  shader_ir_node_kind={}", shader_ir.node_kind));
                lines.push(format!(
                    "  shader_ir_execution_domain={}",
                    shader_ir.execution_domain
                ));
                lines.push(format!("  shader_ir_time_mode={}", shader_ir.time_mode));
                lines.push(format!(
                    "  shader_ir_contract_family={}",
                    shader_ir.contract_family
                ));
                lines.push(format!("  shader_ir_time_domain={}", shader_ir.time_domain));
                lines.push(format!("  shader_ir_glm_scope={}", shader_ir.glm_scope));
                lines.push(format!(
                    "  shader_ir_instruction_count={}",
                    shader_ir.instructions.len()
                ));
                for inst in &shader_ir.instructions {
                    lines.push(format!(
                        "  shader_ir_inst result={} op={} ty={}",
                        inst.result,
                        inst.op,
                        inst.ty.as_deref().unwrap_or("infer")
                    ));
                    lines.push(format!("    expr={}", inst.expr));
                    for arg in &inst.args {
                        lines.push(format!("    arg={arg}"));
                    }
                }
                lines.push(format!(
                    "  shader_ir_term op={} expr={}",
                    shader_ir.terminator.op, shader_ir.terminator.expr
                ));
            }
            for binding in &stage.bindings {
                lines.push(format!(
                    "  binding slot={} kind={} source={}",
                    binding.slot, binding.kind, binding.source
                ));
                if let Some(filter) = &binding.sampler_filter {
                    lines.push(format!(
                        "    sampler filter={} address_mode={}",
                        filter,
                        binding.sampler_address_mode.as_deref().unwrap_or("clamp")
                    ));
                }
                if let Some(format) = &binding.texture_format {
                    lines.push(format!(
                        "    texture format={} size={}x{}",
                        format,
                        binding.texture_width.unwrap_or(0),
                        binding.texture_height.unwrap_or(0)
                    ));
                }
            }
            if let Some(blend_mode) = &stage.blend_mode {
                lines.push(format!(
                    "  blend enabled={} mode={}",
                    stage.blend_enabled.unwrap_or(false),
                    blend_mode
                ));
            }
            if let Some(compare) = &stage.depth_compare {
                lines.push(format!(
                    "  depth test={} write={} compare={}",
                    stage.depth_test_enabled.unwrap_or(false),
                    stage.depth_write_enabled.unwrap_or(false),
                    compare
                ));
            }
            if let Some(cull_mode) = &stage.cull_mode {
                lines.push(format!(
                    "  raster cull={} front={}",
                    cull_mode,
                    stage.front_face.as_deref().unwrap_or("ccw")
                ));
            }
        }
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" }
    }

    pub fn render_package_manifest(&self) -> String {
        let mut out = String::new();
        out.push_str("manifest_version = 1\n");
        out.push_str("package_kind = \"shader_package\"\n");
        out.push_str(&format!("stage_count = {}\n", self.stages.len()));
        out.push_str(&format!(
            "fabric_handle_table_count = {}\n",
            self.fabric_handle_tables.len()
        ));
        out.push_str(&format!(
            "fabric_core_binding_count = {}\n",
            self.fabric_core_bindings.len()
        ));
        out.push_str(&format!(
            "backend_eligible = {}\n",
            self.has_backend_eligible_work()
        ));
        out.push_str(&format!(
            "requires_prerender_fallback = {}\n",
            self.requires_prerender_fallback()
        ));

        for table in &self.fabric_handle_tables {
            out.push_str("\n[[fabric_handle_table]]\n");
            out.push_str(&format!("id = \"{}\"\n", table.node));
            for entry in &table.entries {
                out.push_str("\n[[fabric_handle_table.entry]]\n");
                out.push_str(&format!("slot = \"{}\"\n", escape_toml(&entry.slot)));
                out.push_str(&format!(
                    "resource = \"{}\"\n",
                    escape_toml(&entry.resource)
                ));
            }
        }

        for binding in &self.fabric_core_bindings {
            out.push_str("\n[[fabric_core_binding]]\n");
            out.push_str(&format!("id = \"{}\"\n", binding.node));
            out.push_str(&format!(
                "resource = \"{}\"\n",
                escape_toml(&binding.resource)
            ));
            out.push_str(&format!("core_index = {}\n", binding.core_index));
        }

        for stage in &self.stages {
            out.push_str("\n[[stage]]\n");
            out.push_str(&format!("id = \"{}\"\n", stage.node));
            out.push_str(&format!("op = \"{}\"\n", stage.op));
            out.push_str(&format!(
                "resource = \"{}\"\n",
                escape_toml(&stage.resource)
            ));
            out.push_str(&format!("lowering = \"{}\"\n", stage.lowering.as_str()));
            out.push_str(&format!("reason = \"{}\"\n", escape_toml(&stage.reason)));
            if let Some(table) = &stage.fabric_handle_table {
                out.push_str(&format!(
                    "fabric_handle_table = \"{}\"\n",
                    escape_toml(table)
                ));
            }
            if let Some(pipeline) = &stage.pipeline {
                out.push_str(&format!("pipeline = \"{}\"\n", escape_toml(pipeline)));
            }
            if let Some(target_format) = &stage.target_format {
                out.push_str(&format!(
                    "target_format = \"{}\"\n",
                    escape_toml(target_format)
                ));
            }
            if let Some(topology) = &stage.topology {
                out.push_str(&format!("topology = \"{}\"\n", escape_toml(topology)));
            }
            if let Some(entry) = &stage.wgsl_entry {
                out.push_str(&format!("wgsl_entry = \"{}\"\n", escape_toml(entry)));
            }
            if let Some(source) = &stage.wgsl_source {
                out.push_str(&format!("wgsl_source = \"{}\"\n", escape_toml(source)));
            }
            for shader_ir in &stage.shader_ir_stages {
                out.push_str(&format!(
                    "shader_ir_stage = \"{}\"\nshader_ir_function = \"{}\"\nshader_ir_node_kind = \"{}\"\nshader_ir_execution_domain = \"{}\"\nshader_ir_time_mode = \"{}\"\nshader_ir_contract_family = \"{}\"\nshader_ir_time_domain = \"{}\"\nshader_ir_glm_scope = \"{}\"\nshader_ir_instruction_count = {}\n",
                    escape_toml(&shader_ir.stage),
                    escape_toml(&shader_ir.function),
                    escape_toml(&shader_ir.node_kind),
                    escape_toml(&shader_ir.execution_domain),
                    escape_toml(&shader_ir.time_mode),
                    escape_toml(&shader_ir.contract_family),
                    escape_toml(&shader_ir.time_domain),
                    escape_toml(&shader_ir.glm_scope),
                    shader_ir.instructions.len()
                ));
                out.push_str(&format!(
                    "shader_ir_terminator_op = \"{}\"\nshader_ir_terminator_expr = \"{}\"\n",
                    escape_toml(&shader_ir.terminator.op),
                    escape_toml(&shader_ir.terminator.expr)
                ));
                for inst in &shader_ir.instructions {
                    out.push_str("\n[[stage.shader_ir_instruction]]\n");
                    out.push_str(&format!("result = \"{}\"\n", escape_toml(&inst.result)));
                    out.push_str(&format!("op = \"{}\"\n", escape_toml(&inst.op)));
                    if let Some(ty) = &inst.ty {
                        out.push_str(&format!("ty = \"{}\"\n", escape_toml(ty)));
                    }
                    out.push_str(&format!("expr = \"{}\"\n", escape_toml(&inst.expr)));
                    for arg in &inst.args {
                        out.push_str("\n[[stage.shader_ir_instruction.arg]]\n");
                        out.push_str(&format!("value = \"{}\"\n", escape_toml(arg)));
                    }
                }
            }
            if let Some(blend_mode) = &stage.blend_mode {
                out.push_str(&format!(
                    "blend_enabled = {}\nblend_mode = \"{}\"\n",
                    stage.blend_enabled.unwrap_or(false),
                    escape_toml(blend_mode)
                ));
            }
            if let Some(compare) = &stage.depth_compare {
                out.push_str(&format!(
                    "depth_test_enabled = {}\ndepth_write_enabled = {}\ndepth_compare = \"{}\"\n",
                    stage.depth_test_enabled.unwrap_or(false),
                    stage.depth_write_enabled.unwrap_or(false),
                    escape_toml(compare)
                ));
            }
            if let Some(cull_mode) = &stage.cull_mode {
                out.push_str(&format!(
                    "cull_mode = \"{}\"\nfront_face = \"{}\"\n",
                    escape_toml(cull_mode),
                    escape_toml(stage.front_face.as_deref().unwrap_or("ccw"))
                ));
            }
            for binding in &stage.bindings {
                out.push_str("\n[[stage.binding]]\n");
                out.push_str(&format!("slot = {}\n", binding.slot));
                out.push_str(&format!("kind = \"{}\"\n", escape_toml(&binding.kind)));
                out.push_str(&format!("source = \"{}\"\n", escape_toml(&binding.source)));
                if let Some(format) = &binding.texture_format {
                    out.push_str(&format!(
                        "texture_format = \"{}\"\ntexture_width = {}\ntexture_height = {}\n",
                        escape_toml(format),
                        binding.texture_width.unwrap_or(0),
                        binding.texture_height.unwrap_or(0)
                    ));
                }
                if let Some(filter) = &binding.sampler_filter {
                    out.push_str(&format!(
                        "sampler_filter = \"{}\"\nsampler_address_mode = \"{}\"\n",
                        escape_toml(filter),
                        escape_toml(binding.sampler_address_mode.as_deref().unwrap_or("clamp"))
                    ));
                }
            }

            for variant in stage.backend_variants() {
                out.push_str(&render_shader_variant("stage.variant", &variant));
            }

            if stage.lowering == ShaderLoweringMode::PrerenderOnly {
                out.push_str(&render_shader_variant(
                    "stage.variant",
                    &shader_backend_variant(
                        "reference",
                        "reference",
                        "host",
                        "host",
                        "ppm",
                        "prerender",
                        "reference-raster",
                        900,
                        "active",
                        stage.node.clone(),
                        "assets/<stage>.ppm".to_owned(),
                        "reference fallback artifact".to_owned(),
                    ),
                ));
            }
        }

        out
    }
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
struct ShaderInlineWgslModule {
    resource: String,
    entry: String,
    source: String,
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

impl ShaderStageContract {
    pub fn backend_variants(&self) -> Vec<ShaderBackendVariant> {
        if self.lowering != ShaderLoweringMode::BackendEligible {
            return Vec::new();
        }

        let stage_id = self.node.clone();
        vec![
            shader_backend_variant(
                "metal",
                "gpu",
                "macos",
                "apple-gpu",
                "msl",
                "metal-render-pipeline",
                "msl-source",
                10,
                "active",
                stage_id.clone(),
                format!("metal/{stage_id}.metal"),
                "Apple GPU backend source artifact".to_owned(),
            ),
            shader_backend_variant(
                "vulkan",
                "gpu",
                "cross-platform",
                "vulkan-device",
                "glsl450",
                "vulkan-graphics-pipeline",
                "glsl450-source",
                20,
                "active",
                stage_id.clone(),
                format!("vulkan/{stage_id}.vk.glsl"),
                "Portable Vulkan GLSL source artifact".to_owned(),
            ),
            shader_backend_variant(
                "directx",
                "gpu",
                "windows",
                "d3d12-device",
                "hlsl",
                "d3d12-graphics-pipeline",
                "hlsl-source",
                30,
                "active",
                stage_id.clone(),
                format!("directx/{stage_id}.hlsl"),
                "Windows DirectX backend source artifact".to_owned(),
            ),
            shader_backend_variant(
                "webgpu",
                "gpu",
                "cross-platform",
                "webgpu-device",
                "wgsl",
                "webgpu-render-pipeline",
                "wgsl-source",
                40,
                "planned",
                stage_id.clone(),
                format!("webgpu/{stage_id}.wgsl"),
                "WebGPU/WGSL portable backend artifact".to_owned(),
            ),
            shader_backend_variant(
                "opengl",
                "gpu",
                "cross-platform",
                "opengl-device",
                "glsl460",
                "opengl-graphics-pipeline",
                "glsl460-source",
                80,
                "active",
                stage_id,
                format!("opengl/{}.glsl", self.node),
                "OpenGL GLSL 460 source artifact".to_owned(),
            ),
        ]
    }
}

#[allow(clippy::too_many_arguments)]
fn shader_backend_variant(
    backend: &'static str,
    backend_family: &'static str,
    target_os: &'static str,
    target_device: &'static str,
    ir_format: &'static str,
    dispatch_abi: &'static str,
    kind: &'static str,
    priority: usize,
    status: &'static str,
    entry: String,
    artifact: String,
    notes: String,
) -> ShaderBackendVariant {
    ShaderBackendVariant {
        backend,
        backend_family,
        target_os,
        target_device,
        ir_format,
        dispatch_abi,
        kind,
        priority,
        status,
        verification: "contract-only",
        entry,
        artifact,
        notes,
    }
}

fn render_shader_variant(table: &str, variant: &ShaderBackendVariant) -> String {
    render_backend_variant(
        table,
        variant.backend,
        variant.backend_family,
        variant.target_os,
        variant.target_device,
        variant.ir_format,
        variant.dispatch_abi,
        variant.kind,
        variant.priority,
        variant.status,
        variant.verification,
        &variant.entry,
        &variant.artifact,
        &variant.notes,
    )
}

#[allow(clippy::too_many_arguments)]
fn kernel_backend_variant(
    backend: &'static str,
    backend_family: &'static str,
    target_os: &'static str,
    target_device: &'static str,
    ir_format: &'static str,
    dispatch_abi: &'static str,
    kind: &'static str,
    priority: usize,
    status: &'static str,
    entry: String,
    artifact: String,
    notes: String,
) -> KernelBackendVariant {
    KernelBackendVariant {
        backend,
        backend_family,
        target_os,
        target_device,
        ir_format,
        dispatch_abi,
        kind,
        priority,
        status,
        verification: "contract-only",
        entry,
        artifact,
        notes,
    }
}

fn render_kernel_variant(table: &str, variant: &KernelBackendVariant) -> String {
    render_backend_variant(
        table,
        variant.backend,
        variant.backend_family,
        variant.target_os,
        variant.target_device,
        variant.ir_format,
        variant.dispatch_abi,
        variant.kind,
        variant.priority,
        variant.status,
        variant.verification,
        &variant.entry,
        &variant.artifact,
        &variant.notes,
    )
}

#[allow(clippy::too_many_arguments)]
fn render_backend_variant(
    table: &str,
    backend: &str,
    backend_family: &str,
    target_os: &str,
    target_device: &str,
    ir_format: &str,
    dispatch_abi: &str,
    kind: &str,
    priority: usize,
    status: &str,
    verification: &str,
    entry: &str,
    artifact: &str,
    notes: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("\n[[{table}]]\n"));
    out.push_str(&format!("backend = \"{}\"\n", escape_toml(backend)));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml(backend_family)
    ));
    out.push_str(&format!("target_os = \"{}\"\n", escape_toml(target_os)));
    out.push_str(&format!(
        "target_device = \"{}\"\n",
        escape_toml(target_device)
    ));
    out.push_str(&format!("ir_format = \"{}\"\n", escape_toml(ir_format)));
    out.push_str(&format!(
        "dispatch_abi = \"{}\"\n",
        escape_toml(dispatch_abi)
    ));
    out.push_str(&format!("kind = \"{}\"\n", escape_toml(kind)));
    out.push_str(&format!("priority = {}\n", priority));
    out.push_str(&format!("status = \"{}\"\n", escape_toml(status)));
    out.push_str(&format!(
        "verification = \"{}\"\n",
        escape_toml(verification)
    ));
    out.push_str(&format!("entry = \"{}\"\n", escape_toml(entry)));
    out.push_str(&format!("artifact = \"{}\"\n", escape_toml(artifact)));
    out.push_str(&format!("notes = \"{}\"\n", escape_toml(notes)));
    out
}

impl KernelStageContract {
    pub fn backend_variants(&self) -> Vec<KernelBackendVariant> {
        let stage_id = self.node.clone();
        match self.lowering {
            KernelLoweringMode::BackendEligible => {
                let preferred_backend = self.target_runtime.as_deref();
                let mut variants = Vec::new();
                if matches!(preferred_backend, Some("coreml")) {
                    variants.push(kernel_backend_variant(
                        "coreml",
                        "npu",
                        "macos",
                        "apple-ane",
                        "mlmodel",
                        "coreml-predict",
                        "mlmodel",
                        10,
                        "planned",
                        stage_id.clone(),
                        format!("coreml/{stage_id}.mlmodel"),
                        "Apple ANE / CoreML compute artifact".to_owned(),
                    ));
                    variants.push(kernel_backend_variant(
                        "mps-graph",
                        "gpu",
                        "macos",
                        "apple-gpu",
                        "mps-graph-json",
                        "mps-graph-dispatch",
                        "graph",
                        20,
                        "planned",
                        stage_id.clone(),
                        format!("mps-graph/{stage_id}.json"),
                        "Apple GPU graph fallback artifact".to_owned(),
                    ));
                }
                if matches!(preferred_backend, Some("vulkan")) {
                    variants.push(kernel_backend_variant(
                        "vulkan",
                        "gpu",
                        "cross-platform",
                        "vulkan-device",
                        "spirv",
                        "vulkan-compute-pipeline",
                        "spirv",
                        30,
                        "planned",
                        stage_id.clone(),
                        format!("vulkan/{stage_id}.spv"),
                        "Portable Vulkan compute artifact".to_owned(),
                    ));
                }
                variants.push(kernel_backend_variant(
                    "cpu-fallback",
                    "cpu",
                    "cross-platform",
                    "host-cpu",
                    "llvm-bitcode",
                    "nuis-host-call",
                    "native",
                    900,
                    "planned",
                    stage_id,
                    format!("cpu-fallback/{}.bc", self.node),
                    "Host CPU fallback artifact".to_owned(),
                ));
                variants
            }
            KernelLoweringMode::CpuFallbackOnly => vec![kernel_backend_variant(
                "cpu-fallback",
                "cpu",
                "cross-platform",
                "host-cpu",
                "llvm-bitcode",
                "nuis-host-call",
                "native",
                900,
                "active",
                stage_id,
                format!("cpu-fallback/{}.bc", self.node),
                "Requires host CPU fallback because the op is outside the current backend portability subset".to_owned(),
            )],
        }
    }
}

impl KernelComputeGraphContract {
    pub fn backend_variants(&self) -> Vec<KernelBackendVariant> {
        let entry = self.id.clone();
        match self.lowering {
            KernelLoweringMode::BackendEligible => {
                let preferred_backend = self.target_runtime.as_deref();
                let mut variants = Vec::new();
                if matches!(preferred_backend, Some("coreml")) {
                    variants.push(kernel_backend_variant(
                        "coreml",
                        "npu",
                        "macos",
                        "apple-ane",
                        "mlpackage",
                        "coreml-predict",
                        "mlpackage",
                        10,
                        "planned",
                        entry.clone(),
                        format!("coreml/{}.mlpackage", self.id),
                        "Fused kernel compute graph for Apple ANE / CoreML".to_owned(),
                    ));
                    variants.push(kernel_backend_variant(
                        "mps-graph",
                        "gpu",
                        "macos",
                        "apple-gpu",
                        "mps-graph-json",
                        "mps-graph-dispatch",
                        "graph",
                        20,
                        "planned",
                        entry.clone(),
                        format!("mps-graph/{}.json", self.id),
                        "Fused kernel compute graph for Apple GPU fallback".to_owned(),
                    ));
                }
                if matches!(preferred_backend, Some("vulkan")) {
                    variants.push(kernel_backend_variant(
                        "vulkan",
                        "gpu",
                        "cross-platform",
                        "vulkan-device",
                        "spirv",
                        "vulkan-compute-pipeline",
                        "spirv",
                        30,
                        "planned",
                        entry.clone(),
                        format!("vulkan/{}.spv", self.id),
                        "Fused Vulkan compute graph artifact".to_owned(),
                    ));
                }
                variants.push(kernel_backend_variant(
                    "cpu-fallback",
                    "cpu",
                    "cross-platform",
                    "host-cpu",
                    "llvm-bitcode",
                    "nuis-host-call",
                    "native",
                    900,
                    "planned",
                    entry,
                    format!("cpu-fallback/{}.bc", self.id),
                    "Fused host CPU fallback graph".to_owned(),
                ));
                variants
            }
            KernelLoweringMode::CpuFallbackOnly => vec![kernel_backend_variant(
                "cpu-fallback",
                "cpu",
                "cross-platform",
                "host-cpu",
                "llvm-bitcode",
                "nuis-host-call",
                "native",
                900,
                "active",
                entry,
                format!("cpu-fallback/{}.bc", self.id),
                "Graph requires host CPU fallback because one or more stages are outside the current backend portability subset".to_owned(),
            )],
        }
    }
}

pub fn analyze_shader_lowering(module: &YirModule) -> ShaderLoweringContract {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let incoming = module
        .edges
        .iter()
        .filter(|edge| matches!(edge.kind, EdgeKind::Dep | EdgeKind::Effect))
        .fold(BTreeMap::<&str, Vec<&str>>::new(), |mut acc, edge| {
            acc.entry(edge.to.as_str())
                .or_default()
                .push(edge.from.as_str());
            acc
        });

    let mut stages = Vec::new();
    let fabric_handle_tables: Vec<FabricHandleTableContract> = module
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
        .collect();
    let fabric_core_bindings: Vec<FabricCoreBindingContract> = module
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
        .collect();
    let inline_wgsl_modules: Vec<ShaderInlineWgslModule> = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "shader"
                && node.op.instruction == "inline_wgsl"
                && node.op.args.len() == 2
        })
        .map(|node| ShaderInlineWgslModule {
            resource: node.resource.clone(),
            entry: node.op.args[0].clone(),
            source: decode_inline_shader_source(&node.op.args[1]),
        })
        .collect();

    for node in &module.nodes {
        if node.op.module != "shader" {
            continue;
        }

        match node.op.instruction.as_str() {
            "draw_instanced" => stages.push(analyze_draw_instanced(
                node,
                &nodes,
                &incoming,
                &fabric_handle_tables,
                &inline_wgsl_modules,
            )),
            "draw_ball" | "draw_sphere" => stages.push(ShaderStageContract {
                node: node.name.clone(),
                op: node.op.full_name(),
                resource: node.resource.clone(),
                lowering: ShaderLoweringMode::PrerenderOnly,
                reason: "legacy reference raster op is only available through prerender fallback"
                    .to_owned(),
                pipeline: None,
                target_format: None,
                topology: None,
                wgsl_entry: None,
                wgsl_source: None,
                fabric_handle_table: None,
                bindings: Vec::new(),
                blend_mode: None,
                blend_enabled: None,
                depth_compare: None,
                depth_test_enabled: None,
                depth_write_enabled: None,
                cull_mode: None,
                front_face: None,
                shader_ir_stages: Vec::new(),
            }),
            "dispatch" => stages.push(ShaderStageContract {
                node: node.name.clone(),
                op: node.op.full_name(),
                resource: node.resource.clone(),
                lowering: ShaderLoweringMode::PrerenderOnly,
                reason: "generic shader.dispatch lacks a backend ABI contract".to_owned(),
                pipeline: None,
                target_format: None,
                topology: None,
                wgsl_entry: None,
                wgsl_source: None,
                fabric_handle_table: None,
                bindings: Vec::new(),
                blend_mode: None,
                blend_enabled: None,
                depth_compare: None,
                depth_test_enabled: None,
                depth_write_enabled: None,
                cull_mode: None,
                front_face: None,
                shader_ir_stages: Vec::new(),
            }),
            _ => {}
        }
    }

    ShaderLoweringContract {
        stages,
        fabric_handle_tables,
        fabric_core_bindings,
    }
}

pub fn analyze_kernel_lowering(module: &YirModule) -> KernelLoweringContract {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let fabric_handle_tables = collect_fabric_handle_tables(module);
    let fabric_core_bindings = collect_fabric_core_bindings(module);
    let target_profiles = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "kernel"
                && node.op.instruction == "target_config"
                && node.op.args.len() == 3
        })
        .map(|node| {
            (
                node.resource.clone(),
                KernelTargetProfile {
                    arch: Some(node.op.args[0].clone()),
                    runtime: Some(node.op.args[1].clone()),
                    lane_width: node.op.args[2].parse::<usize>().ok(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    let stages = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "kernel")
        .filter_map(|node| {
            analyze_kernel_stage(node, &nodes, &target_profiles, &fabric_handle_tables)
        })
        .collect::<Vec<_>>();
    let graphs = build_kernel_graphs(&stages);

    KernelLoweringContract {
        stages,
        graphs,
        fabric_handle_tables,
        fabric_core_bindings,
    }
}

fn analyze_draw_instanced(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
    incoming: &BTreeMap<&str, Vec<&str>>,
    fabric_handle_tables: &[FabricHandleTableContract],
    inline_wgsl_modules: &[ShaderInlineWgslModule],
) -> ShaderStageContract {
    let Some(pass_name) = node.op.args.first() else {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            resource: node.resource.clone(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: "draw_instanced is missing its render pass input".to_owned(),
            pipeline: None,
            target_format: None,
            topology: None,
            wgsl_entry: None,
            wgsl_source: None,
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
            shader_ir_stages: Vec::new(),
        };
    };

    let Some(pass_node) = nodes.get(pass_name.as_str()).copied() else {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            resource: node.resource.clone(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: format!("render pass `{pass_name}` is not present in the graph"),
            pipeline: None,
            target_format: None,
            topology: None,
            wgsl_entry: None,
            wgsl_source: None,
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
            shader_ir_stages: Vec::new(),
        };
    };

    if pass_node.op.instruction != "begin_pass" || pass_node.op.args.len() != 3 {
        return ShaderStageContract {
            node: node.name.clone(),
            op: node.op.full_name(),
            resource: node.resource.clone(),
            lowering: ShaderLoweringMode::PrerenderOnly,
            reason: format!("render pass `{pass_name}` does not resolve to shader.begin_pass"),
            pipeline: None,
            target_format: None,
            topology: None,
            wgsl_entry: None,
            wgsl_source: None,
            fabric_handle_table: None,
            bindings: Vec::new(),
            blend_mode: None,
            blend_enabled: None,
            depth_compare: None,
            depth_test_enabled: None,
            depth_write_enabled: None,
            cull_mode: None,
            front_face: None,
            shader_ir_stages: Vec::new(),
        };
    }

    let target_node = nodes.get(pass_node.op.args[0].as_str()).copied();
    let pipeline_node = nodes.get(pass_node.op.args[1].as_str()).copied();

    let target_format = target_node.and_then(parse_target_format);
    let (pipeline_name, topology) = pipeline_node
        .and_then(parse_pipeline_signature)
        .unwrap_or((None, None));
    let (wgsl_entry, wgsl_source) = find_matching_inline_wgsl(
        &node.resource,
        pipeline_name.as_deref(),
        inline_wgsl_modules,
    )
    .map(|module| (Some(module.entry.clone()), Some(module.source.clone())))
    .unwrap_or((None, None));

    let (lowering, reason) = classify_backend_eligibility(
        target_format.as_deref(),
        pipeline_name.as_deref(),
        topology.as_deref(),
    );
    let bindings = incoming
        .get(node.name.as_str())
        .into_iter()
        .flat_map(|names| names.iter().copied())
        .filter_map(|name| nodes.get(name).copied())
        .filter(|candidate| {
            candidate.op.module == "shader" && candidate.op.instruction == "bind_set"
        })
        .flat_map(|bind_set| extract_bindings(bind_set, nodes))
        .collect();
    let render_state = incoming
        .get(node.name.as_str())
        .into_iter()
        .flat_map(|names| names.iter().copied())
        .filter_map(|name| nodes.get(name).copied())
        .find(|candidate| {
            candidate.op.module == "shader" && candidate.op.instruction == "render_state"
        });
    let (
        blend_mode,
        blend_enabled,
        depth_compare,
        depth_test_enabled,
        depth_write_enabled,
        cull_mode,
        front_face,
    ) = render_state
        .and_then(|state| extract_render_state(state, nodes))
        .unwrap_or((None, None, None, None, None, None, None));
    let fabric_handle_table = fabric_handle_tables
        .iter()
        .find(|table| {
            table
                .entries
                .iter()
                .any(|entry| entry.resource == node.resource)
        })
        .map(|table| table.node.clone());
    let shader_ir_stages = wgsl_source
        .as_deref()
        .map(build_shader_ir_stage_contracts)
        .unwrap_or_default();

    ShaderStageContract {
        node: node.name.clone(),
        op: node.op.full_name(),
        resource: node.resource.clone(),
        lowering,
        reason: reason.to_owned(),
        pipeline: pipeline_name,
        target_format,
        topology,
        wgsl_entry,
        wgsl_source,
        fabric_handle_table,
        bindings,
        blend_mode,
        blend_enabled,
        depth_compare,
        depth_test_enabled,
        depth_write_enabled,
        cull_mode,
        front_face,
        shader_ir_stages,
    }
}

fn build_shader_ir_stage_contracts(wgsl_source: &str) -> Vec<NustarContractStage> {
    let mut stages = Vec::new();
    if let Some(vertex_src) = extract_shader_stage_source(wgsl_source, "@vertex", "@fragment") {
        if let Some(stage) = build_shader_ir_stage_contract("vertex", &vertex_src) {
            stages.push(stage);
        }
    }
    if let Some(fragment_src) = extract_shader_stage_source(wgsl_source, "@fragment", "") {
        if let Some(stage) = build_shader_ir_stage_contract("fragment", &fragment_src) {
            stages.push(stage);
        }
    }
    stages
}

fn build_shader_ir_stage_contract(
    stage_name: &str,
    stage_src: &str,
) -> Option<NustarContractStage> {
    let mut instructions = Vec::new();
    for raw_line in stage_src.lines() {
        let line = raw_line.trim();
        if line.starts_with("let ") {
            let Some(eq_pos) = line.find('=') else {
                continue;
            };
            let lhs = line["let ".len()..eq_pos].trim();
            let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
            if rhs.is_empty() {
                continue;
            }
            let (result, ty) = if let Some(colon_pos) = lhs.find(':') {
                (
                    lhs[..colon_pos].trim().to_owned(),
                    Some(lhs[colon_pos + 1..].trim().to_owned()),
                )
            } else {
                (lhs.to_owned(), None)
            };
            if result.is_empty() {
                continue;
            }
            instructions.push(NustarContractInstruction {
                result,
                ty,
                op: classify_shader_ir_op(rhs),
                args: collect_shader_ir_args(rhs),
                expr: rhs.to_owned(),
            });
        } else if line.contains('=') && line.ends_with(';') && !line.starts_with("return ") {
            let eq_pos = line.find('=').expect("checked contains =");
            let lhs = line[..eq_pos].trim();
            let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
            if lhs.is_empty() || rhs.is_empty() {
                continue;
            }
            instructions.push(NustarContractInstruction {
                result: lhs.to_owned(),
                ty: None,
                op: "assign".to_owned(),
                args: collect_shader_ir_args(rhs),
                expr: rhs.to_owned(),
            });
        }
    }

    let return_expr = extract_fragment_return_expr_from_source(stage_src)?;
    Some(NustarContractStage {
        stage: stage_name.to_owned(),
        function: format!("shader.{stage_name}"),
        node_kind: "function-node".to_owned(),
        execution_domain: "shader".to_owned(),
        time_mode: "logical".to_owned(),
        contract_family: "nustar.shader".to_owned(),
        time_domain: format!("shader.stage.{stage_name}"),
        glm_scope: format!("shader::{stage_name}"),
        instructions,
        terminator: NustarContractTerminator {
            op: "return".to_owned(),
            expr: return_expr,
        },
    })
}

fn extract_shader_stage_source(
    wgsl_source: &str,
    stage_marker: &str,
    next_marker: &str,
) -> Option<String> {
    let start = wgsl_source.find(stage_marker)?;
    let tail = &wgsl_source[start..];
    if next_marker.is_empty() {
        return Some(tail.to_owned());
    }
    let end = tail.find(next_marker)?;
    Some(tail[..end].to_owned())
}

fn classify_shader_ir_op(expr: &str) -> String {
    if expr.contains("textureSample(") {
        "sample_texture".to_owned()
    } else if expr.contains("smoothstep(") {
        "smoothstep".to_owned()
    } else if expr.contains("normalize(") {
        "normalize".to_owned()
    } else if expr.contains("dot(") {
        "dot".to_owned()
    } else if expr.contains("clamp(") {
        "clamp".to_owned()
    } else if expr.contains("fract(") {
        "fract".to_owned()
    } else if expr.contains("mix(") {
        "mix".to_owned()
    } else if expr.contains("vec4") || expr.contains("vec3") || expr.contains("vec2") {
        "construct".to_owned()
    } else {
        "expr".to_owned()
    }
}

fn collect_shader_ir_args(expr: &str) -> Vec<String> {
    if let Some(open) = expr.find('(') {
        if let Some(close) = expr.rfind(')') {
            if close > open {
                return expr[open + 1..close]
                    .split(',')
                    .map(str::trim)
                    .filter(|arg| !arg.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
            }
        }
    }
    Vec::new()
}

fn extract_fragment_return_expr_from_source(fragment_src: &str) -> Option<String> {
    let return_pos = fragment_src.find("return")?;
    let after_return = &fragment_src[return_pos + "return".len()..];
    let semicolon_pos = after_return.find(';')?;
    Some(after_return[..semicolon_pos].trim().to_owned())
}

fn extract_bindings(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Vec<ShaderResourceBinding> {
    node.op.args[1..]
        .iter()
        .filter_map(|binding_name| {
            let binding = nodes.get(binding_name.as_str()).copied()?;
            if binding.op.module != "shader" {
                return None;
            }
            let kind = binding.op.instruction.as_str();
            if !matches!(
                kind,
                "uniform"
                    | "storage"
                    | "attachment"
                    | "texture_binding"
                    | "sampler_binding"
                    | "vertex_layout_binding"
                    | "vertex_binding"
                    | "index_binding"
            ) || binding.op.args.len() != 2
            {
                return None;
            }
            let slot = binding.op.args[0].parse::<usize>().ok()?;
            let source = binding.op.args[1].clone();
            let (
                texture_format,
                texture_width,
                texture_height,
                sampler_filter,
                sampler_address_mode,
            ) = extract_binding_metadata(kind, source.as_str(), nodes);
            Some(ShaderResourceBinding {
                slot,
                kind: kind.to_owned(),
                source,
                texture_format,
                texture_width,
                texture_height,
                sampler_filter,
                sampler_address_mode,
            })
        })
        .collect()
}

fn find_matching_inline_wgsl<'a>(
    resource: &str,
    pipeline_name: Option<&str>,
    inline_wgsl_modules: &'a [ShaderInlineWgslModule],
) -> Option<&'a ShaderInlineWgslModule> {
    inline_wgsl_modules
        .iter()
        .find(|module| {
            module.resource == resource
                && pipeline_name
                    .map(|pipeline| pipeline == module.entry)
                    .unwrap_or(true)
        })
        .or_else(|| {
            inline_wgsl_modules
                .iter()
                .find(|module| module.resource == resource)
        })
}

fn decode_inline_shader_source(raw: &str) -> String {
    fn decode_once(raw: &str) -> String {
        let mut out = String::new();
        let mut chars = raw.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        }
        out
    }

    let mut current = raw.to_owned();
    for _ in 0..2 {
        let decoded = decode_once(&current);
        if decoded == current {
            break;
        }
        current = decoded;
    }
    current
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct KernelTargetProfile {
    arch: Option<String>,
    runtime: Option<String>,
    lane_width: Option<usize>,
}

fn analyze_kernel_stage(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
    target_profiles: &BTreeMap<String, KernelTargetProfile>,
    fabric_handle_tables: &[FabricHandleTableContract],
) -> Option<KernelStageContract> {
    if matches!(
        node.op.instruction.as_str(),
        "target_config"
            | "observe"
            | "is_config_ready"
            | "value"
            | "const_bool"
            | "const_i32"
            | "const_i64"
            | "const_f32"
            | "const_f64"
            | "print"
    ) {
        return None;
    }

    let target_profile = target_profiles.get(&node.resource);
    let runtime = target_profile.and_then(|profile| profile.runtime.clone());
    let arch = target_profile.and_then(|profile| profile.arch.clone());
    let lane_width = target_profile.and_then(|profile| profile.lane_width);
    let rows = infer_kernel_rows(node, nodes);
    let cols = infer_kernel_cols(node, nodes);
    let axis = infer_kernel_axis(node);
    let topk = infer_kernel_topk(node);
    let inputs = infer_kernel_inputs(node);
    let fabric_handle_table = fabric_handle_tables
        .iter()
        .find(|table| {
            table
                .entries
                .iter()
                .any(|entry| entry.resource == node.resource)
        })
        .map(|table| table.node.clone());

    let (lowering, reason) = classify_kernel_backend_eligibility(
        node.op.instruction.as_str(),
        runtime.as_deref(),
        rows,
        cols,
        axis.as_deref(),
    );

    Some(KernelStageContract {
        node: node.name.clone(),
        function: format!("kernel.{}", node.name),
        node_kind: "function-node".to_owned(),
        execution_domain: "kernel".to_owned(),
        time_mode: "logical".to_owned(),
        op: node.op.full_name(),
        resource: node.resource.clone(),
        lowering,
        reason: reason.to_owned(),
        target_arch: arch,
        target_runtime: runtime,
        lane_width,
        rows,
        cols,
        axis,
        topk,
        inputs,
        fabric_handle_table,
    })
}

fn infer_kernel_rows(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Option<usize> {
    match node.op.instruction.as_str() {
        "tensor" | "fill" | "splat" if node.op.args.len() >= 2 => {
            node.op.args[0].parse::<usize>().ok()
        }
        "reshape" if node.op.args.len() >= 3 => node.op.args[1].parse::<usize>().ok(),
        "slice" if node.op.args.len() >= 5 => node.op.args[3].parse::<usize>().ok(),
        "broadcast" if node.op.args.len() >= 3 => node.op.args[1].parse::<usize>().ok(),
        "row" => Some(1),
        "col" => nodes
            .get(node.op.args.first()?.as_str())
            .copied()
            .and_then(|source| infer_kernel_rows(source, nodes)),
        "shape" => Some(1),
        "rows" | "cols" | "reduce_sum" | "reduce_mean" | "reduce_max" | "reduce_min" => Some(1),
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "topk_axis" | "sort_axis" => {
            let source = nodes.get(node.op.args.first()?.as_str()).copied()?;
            match node.op.args.get(1).map(|value| value.as_str()) {
                Some("rows") => Some(1),
                Some("cols") => infer_kernel_rows(source, nodes),
                _ => None,
            }
        }
        "argmax" | "argmin" | "element_at" => Some(1),
        _ => None,
    }
}

fn infer_kernel_cols(node: &Node, nodes: &BTreeMap<&str, &Node>) -> Option<usize> {
    match node.op.instruction.as_str() {
        "tensor" | "fill" | "splat" if node.op.args.len() >= 2 => {
            node.op.args[1].parse::<usize>().ok()
        }
        "reshape" if node.op.args.len() >= 3 => node.op.args[2].parse::<usize>().ok(),
        "slice" if node.op.args.len() >= 5 => node.op.args[4].parse::<usize>().ok(),
        "broadcast" if node.op.args.len() >= 3 => node.op.args[2].parse::<usize>().ok(),
        "row" => nodes
            .get(node.op.args.first()?.as_str())
            .copied()
            .and_then(|source| infer_kernel_cols(source, nodes)),
        "col" => Some(1),
        "shape" => Some(2),
        "rows" | "cols" | "reduce_sum" | "reduce_mean" | "reduce_max" | "reduce_min" => Some(1),
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "topk_axis" | "sort_axis" => {
            let source = nodes.get(node.op.args.first()?.as_str()).copied()?;
            match node.op.args.get(1).map(|value| value.as_str()) {
                Some("rows") => infer_kernel_cols(source, nodes),
                Some("cols") => Some(1),
                _ => None,
            }
        }
        "argmax" | "argmin" | "element_at" => Some(1),
        "topk" => node.op.args.get(1).and_then(|k| k.parse::<usize>().ok()),
        _ => None,
    }
}

fn infer_kernel_axis(node: &Node) -> Option<String> {
    match node.op.instruction.as_str() {
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "topk_axis" | "sort_axis" | "relu_axis"
        | "add_scalar_axis" | "mul_scalar_axis" => node.op.args.last().cloned(),
        _ => None,
    }
}

fn infer_kernel_topk(node: &Node) -> Option<usize> {
    match node.op.instruction.as_str() {
        "topk" => node
            .op
            .args
            .get(1)
            .and_then(|value| value.parse::<usize>().ok()),
        "topk_axis" => node
            .op
            .args
            .get(1)
            .and_then(|value| value.parse::<usize>().ok()),
        _ => None,
    }
}

fn infer_kernel_inputs(node: &Node) -> Vec<String> {
    match node.op.instruction.as_str() {
        "tensor" | "fill" | "splat" | "const_bool" | "const_i32" | "const_i64" | "const_f32"
        | "const_f64" | "target_config" => Vec::new(),
        "reshape" => node.op.args.iter().take(1).cloned().collect(),
        "slice" => node.op.args.iter().take(1).cloned().collect(),
        "broadcast" => node.op.args.iter().take(1).cloned().collect(),
        "topk" => node.op.args.iter().take(1).cloned().collect(),
        "topk_axis" => node.op.args.iter().take(1).cloned().collect(),
        "reduce_sum_axis" | "reduce_mean_axis" | "reduce_max_axis" | "reduce_min_axis"
        | "argmax_axis" | "argmin_axis" | "sort_axis" | "relu_axis" | "add_scalar_axis"
        | "mul_scalar_axis" => node.op.args.iter().take(1).cloned().collect(),
        "element_at" => node.op.args.iter().take(1).cloned().collect(),
        "print" => node.op.args.iter().take(1).cloned().collect(),
        _ => node.op.args.clone(),
    }
}

fn classify_kernel_backend_eligibility(
    op: &str,
    runtime: Option<&str>,
    rows: Option<usize>,
    cols: Option<usize>,
    axis: Option<&str>,
) -> (KernelLoweringMode, &'static str) {
    let Some(runtime) = runtime else {
        return (
            KernelLoweringMode::CpuFallbackOnly,
            "missing kernel.target_config runtime contract",
        );
    };

    let portable_tensor_subset = matches!(
        op,
        "tensor"
            | "fill"
            | "splat"
            | "add"
            | "mul"
            | "add_scalar"
            | "mul_scalar"
            | "matmul"
            | "add_bias"
            | "relu"
            | "reshape"
            | "slice"
            | "broadcast"
            | "reduce_sum"
            | "reduce_mean"
            | "reduce_max"
            | "reduce_min"
            | "reduce_sum_axis"
            | "reduce_mean_axis"
            | "reduce_max_axis"
            | "reduce_min_axis"
            | "row"
            | "col"
            | "shape"
            | "rows"
            | "cols"
            | "element_at"
    );

    if !portable_tensor_subset {
        return (
            KernelLoweringMode::CpuFallbackOnly,
            "op is outside the current portable kernel lowering subset",
        );
    }

    if rows == Some(0) || cols == Some(0) {
        return (
            KernelLoweringMode::CpuFallbackOnly,
            "zero-shaped kernel work cannot be lowered portably",
        );
    }

    if let Some(axis) = axis {
        if !matches!(axis, "rows" | "cols") {
            return (
                KernelLoweringMode::CpuFallbackOnly,
                "axis contract is outside the current portable kernel lowering subset",
            );
        }
    }

    match runtime {
        "coreml" => (
            KernelLoweringMode::BackendEligible,
            "stage fits the current CoreML/MPS graph lowering subset",
        ),
        "vulkan" => (
            KernelLoweringMode::BackendEligible,
            "stage fits the current Vulkan compute lowering subset",
        ),
        _ => (
            KernelLoweringMode::CpuFallbackOnly,
            "runtime is outside the current portable kernel lowering subset",
        ),
    }
}

fn build_kernel_graphs(stages: &[KernelStageContract]) -> Vec<KernelComputeGraphContract> {
    let mut by_resource = BTreeMap::<String, Vec<&KernelStageContract>>::new();
    for stage in stages {
        by_resource
            .entry(stage.resource.clone())
            .or_default()
            .push(stage);
    }

    by_resource
        .into_iter()
        .enumerate()
        .map(|(index, (resource, resource_stages))| {
            let lowering = if resource_stages
                .iter()
                .all(|stage| stage.lowering == KernelLoweringMode::BackendEligible)
            {
                KernelLoweringMode::BackendEligible
            } else {
                KernelLoweringMode::CpuFallbackOnly
            };
            let reason = if lowering == KernelLoweringMode::BackendEligible {
                "graph fits the current fused kernel backend portability subset".to_owned()
            } else {
                "graph includes one or more stages outside the current fused kernel backend portability subset".to_owned()
            };
            let target_arch = resource_stages
                .iter()
                .find_map(|stage| stage.target_arch.clone());
            let target_runtime = resource_stages
                .iter()
                .find_map(|stage| stage.target_runtime.clone());
            let lane_width = resource_stages.iter().find_map(|stage| stage.lane_width);
            let graph_name = resource
                .rsplit('@')
                .next()
                .unwrap_or(resource.as_str())
                .replace('.', "_");
            let stages = resource_stages
                .iter()
                .map(|stage| stage.node.clone())
                .collect::<Vec<_>>();

            KernelComputeGraphContract {
                id: format!("kernel_graph_{}_{}", index + 1, graph_name),
                function: format!("kernel.graph.{}", graph_name),
                node_kind: "function-graph".to_owned(),
                execution_domain: "kernel".to_owned(),
                time_mode: "logical".to_owned(),
                resource,
                lowering,
                reason,
                target_arch,
                target_runtime,
                lane_width,
                stages,
            }
        })
        .collect()
}

fn extract_binding_metadata(
    kind: &str,
    source: &str,
    nodes: &BTreeMap<&str, &Node>,
) -> (
    Option<String>,
    Option<usize>,
    Option<usize>,
    Option<String>,
    Option<String>,
) {
    let Some(source_node) = nodes.get(source).copied() else {
        return (None, None, None, None, None);
    };

    match kind {
        "texture_binding"
            if source_node.op.module == "shader"
                && source_node.op.instruction == "texture2d"
                && source_node.op.args.len() == 4 =>
        {
            let width = source_node.op.args[1].parse::<usize>().ok();
            let height = source_node.op.args[2].parse::<usize>().ok();
            (
                Some(source_node.op.args[0].clone()),
                width,
                height,
                None,
                None,
            )
        }
        "sampler_binding"
            if source_node.op.module == "shader"
                && source_node.op.instruction == "sampler"
                && source_node.op.args.len() == 2 =>
        {
            (
                None,
                None,
                None,
                Some(source_node.op.args[0].clone()),
                Some(source_node.op.args[1].clone()),
            )
        }
        _ => (None, None, None, None, None),
    }
}

fn extract_render_state(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
) -> Option<(
    Option<String>,
    Option<bool>,
    Option<String>,
    Option<bool>,
    Option<bool>,
    Option<String>,
    Option<String>,
)> {
    if node.op.args.len() != 4 {
        return None;
    }
    let blend = nodes.get(node.op.args[1].as_str()).copied()?;
    let depth = nodes.get(node.op.args[2].as_str()).copied()?;
    let raster = nodes.get(node.op.args[3].as_str()).copied()?;

    let (blend_enabled, blend_mode) = if blend.op.module == "shader"
        && blend.op.instruction == "blend_state"
        && blend.op.args.len() == 2
    {
        Some((
            parse_bool_literal(&blend.op.args[0])?,
            Some(blend.op.args[1].clone()),
        ))
    } else {
        None
    }?;
    let (depth_test_enabled, depth_write_enabled, depth_compare) = if depth.op.module == "shader"
        && depth.op.instruction == "depth_state"
        && depth.op.args.len() == 3
    {
        Some((
            parse_bool_literal(&depth.op.args[0])?,
            parse_bool_literal(&depth.op.args[1])?,
            Some(depth.op.args[2].clone()),
        ))
    } else {
        None
    }?;
    let (cull_mode, front_face) = if raster.op.module == "shader"
        && raster.op.instruction == "raster_state"
        && raster.op.args.len() == 2
    {
        Some((
            Some(raster.op.args[0].clone()),
            Some(raster.op.args[1].clone()),
        ))
    } else {
        None
    }?;

    Some((
        blend_mode,
        Some(blend_enabled),
        depth_compare,
        Some(depth_test_enabled),
        Some(depth_write_enabled),
        cull_mode,
        front_face,
    ))
}

fn parse_bool_literal(raw: &str) -> Option<bool> {
    match raw {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

fn parse_target_format(node: &Node) -> Option<String> {
    if node.op.module == "shader" && node.op.instruction == "target" && node.op.args.len() == 3 {
        Some(node.op.args[0].clone())
    } else {
        None
    }
}

fn parse_pipeline_signature(node: &Node) -> Option<(Option<String>, Option<String>)> {
    if node.op.module == "shader" && node.op.instruction == "pipeline" && node.op.args.len() == 2 {
        Some((Some(node.op.args[0].clone()), Some(node.op.args[1].clone())))
    } else {
        None
    }
}

fn classify_backend_eligibility(
    target_format: Option<&str>,
    shading_model: Option<&str>,
    topology: Option<&str>,
) -> (ShaderLoweringMode, &'static str) {
    let Some(target_format) = target_format else {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "missing shader.target format contract",
        );
    };
    let Some(shading_model) = shading_model else {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "missing shader.pipeline shading model contract",
        );
    };
    let Some(topology) = topology else {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "missing shader.pipeline topology contract",
        );
    };

    let supported_format = matches!(target_format, "rgba8_unorm" | "bgra8_unorm");
    if !supported_format {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "target format is outside the current backend portability subset",
        );
    }

    let supported_topology = matches!(topology, "triangle" | "triangle_strip");
    if !supported_topology {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "pipeline topology is outside the current backend portability subset",
        );
    }

    let supported_shading_model = matches!(
        shading_model,
        "flat_color" | "ball" | "sphere" | "lit_sphere"
    );
    if !supported_shading_model {
        return (
            ShaderLoweringMode::PrerenderOnly,
            "shading model is outside the current backend portability subset",
        );
    }

    (
        ShaderLoweringMode::BackendEligible,
        "stage fits the current Metal/Vulkan common lowering subset",
    )
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
