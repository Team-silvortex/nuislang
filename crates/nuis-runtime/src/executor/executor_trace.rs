use crate::{RuntimeError, RuntimeRole};

use super::{
    join_or_dash, validate_clock_gate_sequence, ExecutionClockGate, ExecutionClockValidation,
    ExecutionPhaseAction, ExecutionPhaseOutcome, ExecutionStateSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionTraceEvent {
    pub phase: String,
    pub role: RuntimeRole,
    pub adapter_id: String,
    pub bridge_surface: String,
    pub scheduler_binding: String,
    pub clock_gate: ExecutionClockGate,
    pub action: ExecutionPhaseAction,
    pub outcome: ExecutionPhaseOutcome,
    pub state_before: ExecutionStateSnapshot,
    pub state_after: ExecutionStateSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionTrace {
    pub domain_family: String,
    pub phase_count: usize,
    pub events: Vec<ExecutionTraceEvent>,
}

impl ExecutionTrace {
    pub fn validate_clock_gates(
        &self,
        initial_timestamps: &[String],
    ) -> Result<ExecutionClockValidation, RuntimeError> {
        validate_clock_gate_sequence(&self.events, initial_timestamps)
    }

    pub fn render_summary(&self) -> String {
        let mut lines = vec![
            format!("domain_family = {}", self.domain_family),
            format!("phase_count = {}", self.phase_count),
        ];

        for event in &self.events {
            lines.push(format!(
                "event {} role={:?} adapter={}",
                event.phase, event.role, event.adapter_id
            ));
            lines.push(format!("  bridge_surface = {}", event.bridge_surface));
            lines.push(format!("  scheduler_binding = {}", event.scheduler_binding));
            if !event.clock_gate.is_empty() {
                lines.push(format!(
                    "  clock_gate = wait_on:{} emits:{}",
                    join_or_dash(&event.clock_gate.wait_on),
                    join_or_dash(&event.clock_gate.emits)
                ));
            }
            lines.push(format!("  action_kind = {}", event.action.kind));
            lines.push(format!("  outcome_status = {}", event.outcome.status));
            lines.push(format!(
                "  state_before_handles = {}",
                join_or_dash(&event.state_before.available_handles)
            ));
            lines.push(format!(
                "  state_after_handles = {}",
                join_or_dash(&event.state_after.available_handles)
            ));
            for binding in &event.action.resolved_inputs {
                lines.push(format!(
                    "  resolved_input {} kind={:?} capability={} value={}",
                    binding.key,
                    binding.kind,
                    binding.capability_label.as_deref().unwrap_or("-"),
                    binding.value
                ));
            }
            for binding in &event.action.resolved_resources {
                lines.push(format!(
                    "  resolved_resource {} kind={:?} capability={} value={}",
                    binding.key,
                    binding.kind,
                    binding.capability_label.as_deref().unwrap_or("-"),
                    binding.value
                ));
            }
            for binding in &event.outcome.produced_slots {
                lines.push(format!(
                    "  produced_slot {} kind={:?} capability={} value={}",
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
