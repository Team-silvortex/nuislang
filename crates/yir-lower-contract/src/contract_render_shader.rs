use super::backend_variants::{render_shader_variant, shader_backend_variant};
use super::*;

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
