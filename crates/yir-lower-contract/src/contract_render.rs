use super::backend_variants::render_kernel_variant;
use super::*;

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
