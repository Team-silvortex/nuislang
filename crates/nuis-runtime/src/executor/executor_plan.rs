use crate::RuntimeRole;

use super::{join_or_dash, ExecutionClockGate, ExecutionPhaseAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPhaseBinding {
    pub phase: String,
    pub role: RuntimeRole,
    pub bridge_surface: String,
    pub scheduler_binding: String,
    pub lowering_summary: String,
    pub backend_summary: String,
    pub bridge_summary: String,
    pub ir_sidecar_summary: Option<String>,
    pub clock_summary: Option<String>,
    pub clock_gate: ExecutionClockGate,
    pub action: ExecutionPhaseAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub domain_family: String,
    pub package_id: String,
    pub adapter_id: String,
    pub backend_family: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub clock_summary: Option<String>,
    pub clock_gate: ExecutionClockGate,
    pub phases: Vec<ExecutionPhaseBinding>,
}

impl ExecutionPlan {
    pub fn render_summary(&self) -> String {
        let mut lines = vec![
            format!("domain_family = {}", self.domain_family),
            format!("package_id = {}", self.package_id),
            format!("adapter_id = {}", self.adapter_id),
        ];
        if let Some(backend_family) = &self.backend_family {
            lines.push(format!("backend_family = {backend_family}"));
        }
        if let Some(target) = &self.selected_lowering_target {
            lines.push(format!("selected_lowering_target = {target}"));
        }
        if let Some(clock_summary) = &self.clock_summary {
            lines.push(format!("clock_summary = {clock_summary}"));
        }
        if !self.clock_gate.is_empty() {
            lines.push(format!(
                "clock_gate = wait_on:{} emits:{}",
                join_or_dash(&self.clock_gate.wait_on),
                join_or_dash(&self.clock_gate.emits)
            ));
        }
        lines.push(format!("phase_count = {}", self.phases.len()));

        for phase in &self.phases {
            lines.push(format!("phase {} role={:?}", phase.phase, phase.role));
            lines.push(format!("  bridge_surface = {}", phase.bridge_surface));
            lines.push(format!("  scheduler_binding = {}", phase.scheduler_binding));
            lines.push(format!("  lowering_summary = {}", phase.lowering_summary));
            lines.push(format!("  backend_summary = {}", phase.backend_summary));
            lines.push(format!("  bridge_summary = {}", phase.bridge_summary));
            if let Some(ir_sidecar_summary) = &phase.ir_sidecar_summary {
                lines.push(format!("  ir_sidecar_summary = {ir_sidecar_summary}"));
            }
            if let Some(clock_summary) = &phase.clock_summary {
                lines.push(format!("  clock_summary = {clock_summary}"));
            }
            if !phase.clock_gate.is_empty() {
                lines.push(format!(
                    "  clock_gate = wait_on:{} emits:{}",
                    join_or_dash(&phase.clock_gate.wait_on),
                    join_or_dash(&phase.clock_gate.emits)
                ));
            }
            lines.push(format!("  action_kind = {}", phase.action.kind));
            lines.push(format!(
                "  input_handles = {}",
                join_or_dash(&phase.action.input_handles)
            ));
            lines.push(format!(
                "  output_handles = {}",
                join_or_dash(&phase.action.output_handles)
            ));
            lines.push(format!(
                "  scheduler_keys = {}",
                join_or_dash(&phase.action.scheduler_keys)
            ));
            if let Some(hint) = &phase.action.adapter_hint {
                lines.push(format!("  adapter_hint = {hint}"));
            }
            for binding in &phase.action.resource_bindings {
                lines.push(format!(
                    "  resource {} kind={:?} capability={} value={}",
                    binding.key,
                    binding.kind,
                    binding.capability_label.as_deref().unwrap_or("-"),
                    binding.value
                ));
            }
        }

        lines.join("\n")
    }
}
