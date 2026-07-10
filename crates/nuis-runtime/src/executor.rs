use crate::{DomainAdapter, PreparedDomainExecution, RuntimeError, RuntimeRole};

mod executor_clock;
mod executor_plan;
mod executor_resources;
mod executor_trace;

use executor_clock::{execution_clock_gate, execution_clock_summary, validate_clock_gate_sequence};
pub use executor_clock::{ExecutionClockGate, ExecutionClockValidation};
pub use executor_plan::{ExecutionPhaseBinding, ExecutionPlan};
use executor_resources::{
    apply_phase_outcome, default_phase_action, default_phase_outcome, materialize_phase_action,
    phase_role,
};
#[cfg(test)]
use executor_resources::{
    domain_resource_capability_label, slot_resource_capability_label, slot_resource_kind,
};
pub use executor_trace::{ExecutionTrace, ExecutionTraceEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContract {
    pub yir_version: &'static str,
    pub fabric_abi_version: &'static str,
    pub profile: ExecutionProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionProfile {
    Aot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionResourceKind {
    Handle,
    Packet,
    Response,
    Buffer,
    Scheduler,
    Bridge,
    Metadata,
    Slot,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResourceBinding {
    pub key: String,
    pub kind: ExecutionResourceKind,
    pub capability_label: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPhaseAction {
    pub kind: String,
    pub input_handles: Vec<String>,
    pub resolved_inputs: Vec<ExecutionResourceBinding>,
    pub output_handles: Vec<String>,
    pub resource_bindings: Vec<ExecutionResourceBinding>,
    pub resolved_resources: Vec<ExecutionResourceBinding>,
    pub scheduler_keys: Vec<String>,
    pub adapter_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPhaseOutcome {
    pub status: String,
    pub produced_handles: Vec<String>,
    pub produced_slots: Vec<ExecutionResourceBinding>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionStateSnapshot {
    pub available_handles: Vec<String>,
    pub handle_slots: Vec<ExecutionResourceBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionPhaseContext<'a> {
    pub phase: &'a str,
    pub role: RuntimeRole,
    pub domain_family: &'a str,
    pub package_id: &'a str,
    pub adapter_id: &'a str,
    pub backend_family: Option<&'a str>,
    pub selected_lowering_target: Option<&'a str>,
    pub bridge_surface: &'a str,
    pub scheduler_binding: &'a str,
    pub lowering_summary: &'a str,
    pub backend_summary: &'a str,
    pub bridge_summary: &'a str,
    pub ir_sidecar_summary: Option<&'a str>,
    pub clock_summary: Option<&'a str>,
    pub clock_gate: &'a ExecutionClockGate,
    pub available_handles: &'a [String],
    pub available_slots: &'a [ExecutionResourceBinding],
}

#[derive(Debug, Default)]
pub struct Executor;

impl Executor {
    pub fn verify(&self, contract: &ExecutionContract) -> Result<(), &'static str> {
        if contract.yir_version.is_empty() || contract.fabric_abi_version.is_empty() {
            return Err("execution contract is incomplete");
        }

        Ok(())
    }

    pub fn plan<'a>(
        &self,
        prepared: &PreparedDomainExecution<'a>,
    ) -> Result<ExecutionPlan, RuntimeError> {
        let phase_order = prepared.phase_order().ok_or_else(|| {
            RuntimeError::new(format!(
                "missing phase order for domain `{}`",
                prepared.unit.domain_family
            ))
        })?;
        let host_plan = prepared.host_bridge_plan_entry.ok_or_else(|| {
            RuntimeError::new(format!(
                "missing host bridge plan for domain `{}`",
                prepared.unit.domain_family
            ))
        })?;
        let lowering_summary =
            normalized_summary(prepared.lowering_plan_text(), "lowering_plan", prepared)?;
        let backend_summary =
            normalized_summary(prepared.backend_stub_text(), "backend_stub", prepared)?;
        let bridge_summary =
            normalized_summary(prepared.bridge_plan_text(), "bridge_plan", prepared)?;
        let ir_sidecar_summary = optional_summary(prepared.ir_sidecar_text(), prepared)?;
        let clock_summary = execution_clock_summary(prepared);
        let clock_gate = execution_clock_gate(prepared);

        let phases = phase_order
            .iter()
            .map(|phase| {
                let role = phase_role(phase);
                let ctx = ExecutionPhaseContext {
                    phase,
                    role,
                    domain_family: &prepared.unit.domain_family,
                    package_id: &prepared.unit.package_id,
                    adapter_id: prepared.adapter.adapter_id(),
                    backend_family: prepared.unit.backend_family.as_deref(),
                    selected_lowering_target: prepared.unit.selected_lowering_target.as_deref(),
                    bridge_surface: &host_plan.bridge_surface,
                    scheduler_binding: &host_plan.scheduler_binding,
                    lowering_summary: &lowering_summary,
                    backend_summary: &backend_summary,
                    bridge_summary: &bridge_summary,
                    ir_sidecar_summary: ir_sidecar_summary.as_deref(),
                    clock_summary: clock_summary.as_deref(),
                    clock_gate: &clock_gate,
                    available_handles: &[],
                    available_slots: &[],
                };
                ExecutionPhaseBinding {
                    phase: phase.clone(),
                    role,
                    bridge_surface: host_plan.bridge_surface.clone(),
                    scheduler_binding: host_plan.scheduler_binding.clone(),
                    lowering_summary: lowering_summary.clone(),
                    backend_summary: backend_summary.clone(),
                    bridge_summary: bridge_summary.clone(),
                    ir_sidecar_summary: ir_sidecar_summary.clone(),
                    clock_summary: clock_summary.clone(),
                    clock_gate: clock_gate.clone(),
                    action: prepared
                        .adapter
                        .phase_action(&ctx)
                        .unwrap_or_else(|| default_phase_action(&ctx)),
                }
            })
            .collect::<Vec<_>>();

        Ok(ExecutionPlan {
            domain_family: prepared.unit.domain_family.clone(),
            package_id: prepared.unit.package_id.clone(),
            adapter_id: prepared.adapter.adapter_id().to_owned(),
            backend_family: prepared.unit.backend_family.clone(),
            selected_lowering_target: prepared.unit.selected_lowering_target.clone(),
            clock_summary,
            clock_gate,
            phases,
        })
    }

    pub fn execute_plan(&self, plan: &ExecutionPlan) -> Result<ExecutionTrace, RuntimeError> {
        if plan.phases.is_empty() {
            return Err(RuntimeError::new(format!(
                "execution plan for domain `{}` has no phases",
                plan.domain_family
            )));
        }

        let mut state = ExecutionStateSnapshot::default();
        let events = plan
            .phases
            .iter()
            .map(|binding| {
                let state_before = state.clone();
                let action = materialize_phase_action(binding, &state_before);
                let outcome = default_phase_outcome(&action, binding);
                apply_phase_outcome(&mut state, &outcome);
                ExecutionTraceEvent {
                    phase: binding.phase.clone(),
                    role: binding.role,
                    adapter_id: plan.adapter_id.clone(),
                    bridge_surface: binding.bridge_surface.clone(),
                    scheduler_binding: binding.scheduler_binding.clone(),
                    clock_gate: binding.clock_gate.clone(),
                    action,
                    outcome,
                    state_before,
                    state_after: state.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(ExecutionTrace {
            domain_family: plan.domain_family.clone(),
            phase_count: events.len(),
            events,
        })
    }

    pub fn execute_prepared<'a>(
        &self,
        prepared: &PreparedDomainExecution<'a>,
    ) -> Result<ExecutionTrace, RuntimeError> {
        let plan = self.plan(prepared)?;
        self.execute_prepared_plan(prepared.adapter, &plan)
    }

    pub fn execute_prepared_plan(
        &self,
        adapter: &dyn DomainAdapter,
        plan: &ExecutionPlan,
    ) -> Result<ExecutionTrace, RuntimeError> {
        if plan.phases.is_empty() {
            return Err(RuntimeError::new(format!(
                "execution plan for domain `{}` has no phases",
                plan.domain_family
            )));
        }

        let mut state = ExecutionStateSnapshot::default();
        let events = plan
            .phases
            .iter()
            .map(|binding| {
                let state_before = state.clone();
                let action = materialize_phase_action(binding, &state_before);
                let ctx = ExecutionPhaseContext {
                    phase: &binding.phase,
                    role: binding.role,
                    domain_family: &plan.domain_family,
                    package_id: &plan.package_id,
                    adapter_id: &plan.adapter_id,
                    backend_family: plan.backend_family.as_deref(),
                    selected_lowering_target: plan.selected_lowering_target.as_deref(),
                    bridge_surface: &binding.bridge_surface,
                    scheduler_binding: &binding.scheduler_binding,
                    lowering_summary: &binding.lowering_summary,
                    backend_summary: &binding.backend_summary,
                    bridge_summary: &binding.bridge_summary,
                    ir_sidecar_summary: binding.ir_sidecar_summary.as_deref(),
                    clock_summary: binding.clock_summary.as_deref(),
                    clock_gate: &binding.clock_gate,
                    available_handles: &state_before.available_handles,
                    available_slots: &state_before.handle_slots,
                };
                let outcome = adapter
                    .phase_outcome(&ctx, &action)
                    .unwrap_or_else(|| default_phase_outcome(&action, binding));
                apply_phase_outcome(&mut state, &outcome);
                ExecutionTraceEvent {
                    phase: binding.phase.clone(),
                    role: binding.role,
                    adapter_id: plan.adapter_id.clone(),
                    bridge_surface: binding.bridge_surface.clone(),
                    scheduler_binding: binding.scheduler_binding.clone(),
                    clock_gate: binding.clock_gate.clone(),
                    action,
                    outcome,
                    state_before,
                    state_after: state.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(ExecutionTrace {
            domain_family: plan.domain_family.clone(),
            phase_count: events.len(),
            events,
        })
    }
}

fn normalized_summary(
    section: Option<Result<&str, std::str::Utf8Error>>,
    section_name: &str,
    prepared: &PreparedDomainExecution<'_>,
) -> Result<String, RuntimeError> {
    let text = section.ok_or_else(|| {
        RuntimeError::new(format!(
            "missing `{section_name}` section for domain `{}`",
            prepared.unit.domain_family
        ))
    })?;
    text.map(normalize_summary).map_err(|error| {
        RuntimeError::new(format!(
            "invalid `{section_name}` text for domain `{}`: {error}",
            prepared.unit.domain_family
        ))
    })
}

fn optional_summary(
    section: Option<Result<&str, std::str::Utf8Error>>,
    prepared: &PreparedDomainExecution<'_>,
) -> Result<Option<String>, RuntimeError> {
    match section {
        Some(Ok(text)) => Ok(Some(normalize_ir_sidecar_summary(text))),
        Some(Err(error)) => Err(RuntimeError::new(format!(
            "invalid ir sidecar text for domain `{}`: {error}",
            prepared.unit.domain_family
        ))),
        None => Ok(None),
    }
}

fn normalize_summary(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("")
        .to_owned()
}

fn normalize_ir_sidecar_summary(text: &str) -> String {
    let mut in_lowering_capabilities = false;
    let mut fields = Vec::new();
    for line in text.lines().map(str::trim) {
        if line == "[lowering_capabilities]" {
            in_lowering_capabilities = true;
            continue;
        }
        if in_lowering_capabilities && line.starts_with('[') {
            break;
        }
        if !in_lowering_capabilities || line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(" = ") else {
            continue;
        };
        if matches!(
            key,
            "capability_owner"
                | "native_ir"
                | "pipeline_lowering"
                | "resource_lowering"
                | "dispatch_lowering"
                | "tensor_lowering"
                | "texture_lowering"
                | "result_lowering"
        ) {
            fields.push(format!("{key}={}", value.trim_matches('"')));
        }
    }
    if fields.is_empty() {
        normalize_summary(text)
    } else {
        fields.join(" ")
    }
}

fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_owned()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests;
