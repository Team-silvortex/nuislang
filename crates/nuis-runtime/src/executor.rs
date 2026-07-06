use crate::{DomainAdapter, PreparedDomainExecution, RuntimeError, RuntimeRole};
use std::collections::{BTreeMap, BTreeSet};

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
pub struct ExecutionClockGate {
    pub wait_on: Vec<String>,
    pub emits: Vec<String>,
}

impl ExecutionClockGate {
    pub fn is_empty(&self) -> bool {
        self.wait_on.is_empty() && self.emits.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionClockValidation {
    pub initial_timestamps: Vec<String>,
    pub observed_emits: Vec<String>,
    pub final_timestamps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionStateSnapshot {
    pub available_handles: Vec<String>,
    pub handle_slots: Vec<ExecutionResourceBinding>,
}

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

fn normalized_summary<'a>(
    section: Option<Result<&'a str, std::str::Utf8Error>>,
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

fn optional_summary<'a>(
    section: Option<Result<&'a str, std::str::Utf8Error>>,
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

fn execution_clock_summary(prepared: &PreparedDomainExecution<'_>) -> Option<String> {
    let domain = prepared.clock_domain?;
    let mut fields = vec![
        format!("clock_domain={}", domain.clock_domain_id),
        format!("kind={}", domain.clock_kind),
        format!("epoch={}", domain.clock_epoch_kind),
        format!("resolution={}", domain.clock_resolution),
        format!("bridge={}", domain.clock_bridge_default),
        format!("hook={}", domain.lifecycle_hook),
    ];
    if !prepared.clock_edges.is_empty() {
        fields.push(format!(
            "happens_before={}",
            prepared
                .clock_edges
                .iter()
                .map(|edge| format!("{}->{}", edge.from, edge.to))
                .collect::<Vec<_>>()
                .join("|")
        ));
    }
    Some(fields.join(" "))
}

fn execution_clock_gate(prepared: &PreparedDomainExecution<'_>) -> ExecutionClockGate {
    let mut wait_on = Vec::new();
    let mut emits = Vec::new();
    let mut seen_wait = BTreeSet::new();
    let mut seen_emit = BTreeSet::new();
    for edge in &prepared.clock_edges {
        for wait in edge
            .from
            .split('|')
            .map(str::trim)
            .filter(|wait| !wait.is_empty())
        {
            if seen_wait.insert(wait.to_owned()) {
                wait_on.push(wait.to_owned());
            }
        }
        if !edge.to.is_empty() && seen_emit.insert(edge.to.clone()) {
            emits.push(edge.to.clone());
        }
    }
    ExecutionClockGate { wait_on, emits }
}

fn validate_clock_gate_sequence(
    events: &[ExecutionTraceEvent],
    initial_timestamps: &[String],
) -> Result<ExecutionClockValidation, RuntimeError> {
    let mut satisfied = BTreeSet::new();
    let mut final_timestamps = Vec::new();
    for timestamp in initial_timestamps {
        if satisfied.insert(timestamp.clone()) {
            final_timestamps.push(timestamp.clone());
        }
    }

    let mut observed_emits = Vec::new();
    let mut seen_emits = BTreeSet::new();
    for event in events {
        for required in &event.clock_gate.wait_on {
            if !satisfied.contains(required) {
                return Err(RuntimeError::new(format!(
                    "clock gate violation in phase `{}`: missing timestamp `{}`",
                    event.phase, required
                )));
            }
        }
        for emitted in &event.clock_gate.emits {
            if satisfied.insert(emitted.clone()) {
                final_timestamps.push(emitted.clone());
            }
            if seen_emits.insert(emitted.clone()) {
                observed_emits.push(emitted.clone());
            }
        }
    }

    Ok(ExecutionClockValidation {
        initial_timestamps: initial_timestamps.to_vec(),
        observed_emits,
        final_timestamps,
    })
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

fn default_phase_action(ctx: &ExecutionPhaseContext<'_>) -> ExecutionPhaseAction {
    ExecutionPhaseAction {
        kind: format!("phase.{}", ctx.phase),
        input_handles: default_input_handles(ctx),
        resolved_inputs: Vec::new(),
        output_handles: default_output_handles(ctx),
        resource_bindings: default_resource_bindings(ctx),
        resolved_resources: Vec::new(),
        scheduler_keys: vec![
            ctx.scheduler_binding.to_owned(),
            ctx.domain_family.to_owned(),
            ctx.phase.to_owned(),
        ],
        adapter_hint: None,
    }
}

fn materialize_phase_action(
    binding: &ExecutionPhaseBinding,
    state: &ExecutionStateSnapshot,
) -> ExecutionPhaseAction {
    let mut action = binding.action.clone();
    let slot_map = state
        .handle_slots
        .iter()
        .map(|slot| (slot.key.as_str(), slot.value.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut known = BTreeSet::new();
    action.input_handles.retain(|key| known.insert(key.clone()));
    action.resolved_inputs = action
        .input_handles
        .iter()
        .filter_map(|key| {
            slot_map
                .get(key.as_str())
                .map(|value| ExecutionResourceBinding {
                    key: key.clone(),
                    kind: slot_resource_kind(key),
                    capability_label: Some(slot_resource_capability_label(key)),
                    value: (*value).to_owned(),
                })
        })
        .collect();
    action.resolved_resources = action
        .resource_bindings
        .iter()
        .map(|binding| ExecutionResourceBinding {
            key: binding.key.clone(),
            kind: binding.kind.clone(),
            capability_label: binding.capability_label.clone(),
            value: resolve_resource_binding_value(&binding.value, &slot_map),
        })
        .collect();
    action
}

fn resolve_resource_binding_value(value: &str, slot_map: &BTreeMap<&str, &str>) -> String {
    if let Some(slot_key) = value.strip_prefix("slot:") {
        slot_map
            .get(slot_key)
            .map(|resolved| (*resolved).to_owned())
            .unwrap_or_else(|| format!("unresolved:{slot_key}"))
    } else {
        value.to_owned()
    }
}

fn default_phase_outcome(
    action: &ExecutionPhaseAction,
    binding: &ExecutionPhaseBinding,
) -> ExecutionPhaseOutcome {
    ExecutionPhaseOutcome {
        status: "mock-complete".to_owned(),
        produced_handles: action.output_handles.clone(),
        produced_slots: action
            .output_handles
            .iter()
            .map(|key| ExecutionResourceBinding {
                key: key.clone(),
                kind: slot_resource_kind(key),
                capability_label: Some(slot_resource_capability_label(key)),
                value: format!("mock://{}/{}", binding.phase, key),
            })
            .collect(),
        notes: vec![
            format!("phase={} completed in mock runtime", binding.phase),
            format!("scheduler={}", binding.scheduler_binding),
        ],
    }
}

fn apply_phase_outcome(state: &mut ExecutionStateSnapshot, outcome: &ExecutionPhaseOutcome) {
    let mut known = state
        .available_handles
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for handle in &outcome.produced_handles {
        if known.insert(handle.clone()) {
            state.available_handles.push(handle.clone());
        }
    }
    let mut slot_map = state
        .handle_slots
        .iter()
        .map(|slot| (slot.key.clone(), slot.value.clone()))
        .collect::<BTreeMap<_, _>>();
    for slot in &outcome.produced_slots {
        slot_map.insert(slot.key.clone(), slot.value.clone());
    }
    state.handle_slots = slot_map
        .into_iter()
        .map(|(key, value)| ExecutionResourceBinding {
            kind: slot_resource_kind(&key),
            capability_label: Some(slot_resource_capability_label(&key)),
            key,
            value,
        })
        .collect();
}

fn slot_resource_kind(key: &str) -> ExecutionResourceKind {
    if key.ends_with(".handle") {
        ExecutionResourceKind::Handle
    } else if key.ends_with(".packet") || key.contains("packet") {
        ExecutionResourceKind::Packet
    } else if key.ends_with(".response") || key.contains("response") {
        ExecutionResourceKind::Response
    } else if key.ends_with(".buffer") || key.contains("buffer") {
        ExecutionResourceKind::Buffer
    } else if key.contains("scheduler") {
        ExecutionResourceKind::Scheduler
    } else if key.contains("bridge") {
        ExecutionResourceKind::Bridge
    } else {
        ExecutionResourceKind::Slot
    }
}

fn slot_resource_capability_label(key: &str) -> String {
    if key.ends_with(".handle") {
        format!("cap.handle.{key}")
    } else if key.ends_with(".packet") || key.contains("packet") {
        format!("cap.packet.{key}")
    } else if key.ends_with(".buffer") || key.contains("buffer") {
        format!("cap.buffer.{key}")
    } else if key.ends_with(".response") || key.contains("response") || key.ends_with(".target") {
        format!("cap.response.{key}")
    } else {
        format!("cap.slot.{key}")
    }
}

fn domain_resource_capability_label(
    domain_family: &str,
    selected_lowering_target: Option<&str>,
    key: &str,
    kind: &ExecutionResourceKind,
) -> String {
    let scope = capability_scope(domain_family, selected_lowering_target);
    match (domain_family, kind) {
        ("network", ExecutionResourceKind::Packet) => format!("cap.{scope}.packet.{key}"),
        ("network", ExecutionResourceKind::Response) => format!("cap.{scope}.response.{key}"),
        ("network", ExecutionResourceKind::Handle) => format!("cap.{scope}.handle.{key}"),
        ("kernel", ExecutionResourceKind::Buffer) => format!("cap.{scope}.buffer.{key}"),
        ("kernel", ExecutionResourceKind::Handle) => format!("cap.{scope}.dispatch.{key}"),
        ("shader", ExecutionResourceKind::Buffer) => format!("cap.{scope}.buffer.{key}"),
        ("shader", ExecutionResourceKind::Handle) => format!("cap.{scope}.draw.{key}"),
        ("shader", ExecutionResourceKind::Response) => format!("cap.{scope}.frame.{key}"),
        (_, ExecutionResourceKind::Bridge) => format!("cap.{scope}.bridge.{key}"),
        (_, ExecutionResourceKind::Scheduler) => format!("cap.{scope}.scheduler.{key}"),
        (_, ExecutionResourceKind::Metadata) => format!("cap.{scope}.meta.{key}"),
        _ => format!("cap.{scope}.{key}"),
    }
}

fn capability_scope(domain_family: &str, selected_lowering_target: Option<&str>) -> String {
    if let Some(target) = selected_lowering_target {
        let slug = target.replace('.', "_").replace('-', "_");
        format!("{domain_family}.{slug}")
    } else {
        domain_family.to_owned()
    }
}

fn default_input_handles(ctx: &ExecutionPhaseContext<'_>) -> Vec<String> {
    match (ctx.domain_family, ctx.phase) {
        ("network", "bind") => vec!["authority.text".to_owned()],
        ("network", "submit") => vec!["session.handle".to_owned(), "request.packet".to_owned()],
        ("network", "wait") => vec!["task.handle".to_owned()],
        ("network", "finalize") => vec!["response.handle".to_owned()],
        ("kernel", "bind") => vec!["kernel.buffer".to_owned(), "queue.slot".to_owned()],
        ("kernel", "submit") => vec!["kernel.buffer".to_owned(), "dispatch.grid".to_owned()],
        ("kernel", "wait") => vec!["dispatch.handle".to_owned()],
        ("kernel", "finalize") => vec!["result.buffer".to_owned()],
        ("shader", "bind") => vec!["shader.buffer".to_owned(), "frame.target".to_owned()],
        ("shader", "submit") => vec!["shader.buffer".to_owned(), "draw.list".to_owned()],
        ("shader", "wait") => vec!["draw.handle".to_owned()],
        ("shader", "finalize") => vec!["frame.target".to_owned()],
        (_, "bind") => vec!["bridge.surface".to_owned()],
        (_, "submit") => vec!["phase.submit".to_owned()],
        (_, "wait") => vec!["phase.wait".to_owned()],
        (_, "finalize") => vec!["phase.finalize".to_owned()],
        _ => vec!["phase.input".to_owned()],
    }
}

fn default_output_handles(ctx: &ExecutionPhaseContext<'_>) -> Vec<String> {
    match (ctx.domain_family, ctx.phase) {
        ("network", "bind") => vec!["session.handle".to_owned()],
        ("network", "submit") => vec!["task.handle".to_owned()],
        ("network", "wait") => vec!["response.handle".to_owned()],
        ("network", "finalize") => vec!["status.code".to_owned()],
        ("kernel", "bind") => vec!["kernel.buffer".to_owned()],
        ("kernel", "submit") => vec!["dispatch.handle".to_owned()],
        ("kernel", "wait") => vec!["result.buffer".to_owned()],
        ("kernel", "finalize") => vec!["completion.fence".to_owned()],
        ("shader", "bind") => vec!["shader.buffer".to_owned()],
        ("shader", "submit") => vec!["draw.handle".to_owned()],
        ("shader", "wait") => vec!["frame.target".to_owned()],
        ("shader", "finalize") => vec!["present.fence".to_owned()],
        (_, "bind") => vec!["phase.bind".to_owned()],
        (_, "submit") => vec!["phase.submit".to_owned()],
        (_, "wait") => vec!["phase.wait".to_owned()],
        (_, "finalize") => vec!["phase.finalize".to_owned()],
        _ => vec!["phase.output".to_owned()],
    }
}

fn default_resource_bindings(ctx: &ExecutionPhaseContext<'_>) -> Vec<ExecutionResourceBinding> {
    let mut bindings = vec![
        ExecutionResourceBinding {
            key: "bridge_surface".to_owned(),
            kind: ExecutionResourceKind::Bridge,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "bridge_surface",
                &ExecutionResourceKind::Bridge,
            )),
            value: ctx.bridge_surface.to_owned(),
        },
        ExecutionResourceBinding {
            key: "scheduler_binding".to_owned(),
            kind: ExecutionResourceKind::Scheduler,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "scheduler_binding",
                &ExecutionResourceKind::Scheduler,
            )),
            value: ctx.scheduler_binding.to_owned(),
        },
        ExecutionResourceBinding {
            key: "backend_summary".to_owned(),
            kind: ExecutionResourceKind::Metadata,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "backend_summary",
                &ExecutionResourceKind::Metadata,
            )),
            value: ctx.backend_summary.to_owned(),
        },
    ];
    if let Some(ir_sidecar_summary) = ctx.ir_sidecar_summary {
        bindings.push(ExecutionResourceBinding {
            key: "lowering_capabilities".to_owned(),
            kind: ExecutionResourceKind::Metadata,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "lowering_capabilities",
                &ExecutionResourceKind::Metadata,
            )),
            value: ir_sidecar_summary.to_owned(),
        });
    }
    if let Some(clock_summary) = ctx.clock_summary {
        bindings.push(ExecutionResourceBinding {
            key: "clock_protocol".to_owned(),
            kind: ExecutionResourceKind::Metadata,
            capability_label: Some(domain_resource_capability_label(
                ctx.domain_family,
                ctx.selected_lowering_target,
                "clock_protocol",
                &ExecutionResourceKind::Metadata,
            )),
            value: clock_summary.to_owned(),
        });
    }
    match ctx.domain_family {
        "network" => {
            bindings.push(ExecutionResourceBinding {
                key: "active_session".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "active_session",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:session.handle".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "request_packet".to_owned(),
                kind: ExecutionResourceKind::Packet,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "request_packet",
                    &ExecutionResourceKind::Packet,
                )),
                value: "slot:request.packet".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "active_response".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "active_response",
                    &ExecutionResourceKind::Response,
                )),
                value: "slot:response.handle".to_owned(),
            });
        }
        "kernel" => {
            bindings.push(ExecutionResourceBinding {
                key: "kernel_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "kernel_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:kernel.buffer".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "dispatch_handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "dispatch_handle",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:dispatch.handle".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "result_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "result_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:result.buffer".to_owned(),
            });
        }
        "shader" => {
            bindings.push(ExecutionResourceBinding {
                key: "shader_buffer".to_owned(),
                kind: ExecutionResourceKind::Buffer,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "shader_buffer",
                    &ExecutionResourceKind::Buffer,
                )),
                value: "slot:shader.buffer".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "draw_handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "draw_handle",
                    &ExecutionResourceKind::Handle,
                )),
                value: "slot:draw.handle".to_owned(),
            });
            bindings.push(ExecutionResourceBinding {
                key: "frame_target".to_owned(),
                kind: ExecutionResourceKind::Response,
                capability_label: Some(domain_resource_capability_label(
                    ctx.domain_family,
                    ctx.selected_lowering_target,
                    "frame_target",
                    &ExecutionResourceKind::Response,
                )),
                value: "slot:frame.target".to_owned(),
            });
        }
        _ => {}
    }
    bindings
}

fn phase_role(phase: &str) -> RuntimeRole {
    match phase {
        "bind" => RuntimeRole::Bind,
        "submit" | "wait" | "finalize" => RuntimeRole::Execute,
        _ => RuntimeRole::Execute,
    }
}

#[cfg(test)]
mod tests {
    use nuis_artifact::{
        BridgeRegistryEntry, BuildManifestDomainBuildUnit, ClockDomain, ClockEdge,
        DomainBuildUnitPayloadBlob, DomainBuildUnitPayloadBlobSection, HostBridgePlanEntry,
    };

    use crate::{
        DomainAdapter, ExecutionPhaseAction, ExecutionPhaseContext, ExecutionPhaseOutcome,
        ExecutionResourceBinding, PreparedDomainExecution,
    };

    use super::{
        domain_resource_capability_label, slot_resource_capability_label, slot_resource_kind,
        ExecutionContract, ExecutionProfile, ExecutionResourceKind, Executor,
    };

    struct NetworkAdapter;
    struct PassiveAdapter;

    impl DomainAdapter for NetworkAdapter {
        fn adapter_id(&self) -> &'static str {
            "network-test-adapter"
        }

        fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool {
            unit.domain_family == "network"
        }

        fn phase_action(&self, ctx: &ExecutionPhaseContext<'_>) -> Option<ExecutionPhaseAction> {
            Some(ExecutionPhaseAction {
                kind: format!("network.{}", ctx.phase),
                input_handles: match ctx.phase {
                    "bind" => vec!["authority.text".to_owned()],
                    "submit" => vec!["session.handle".to_owned(), "request.packet".to_owned()],
                    "wait" => vec!["task.handle".to_owned()],
                    "finalize" => vec!["response.handle".to_owned()],
                    _ => vec!["phase.input".to_owned()],
                },
                resolved_inputs: Vec::new(),
                output_handles: match ctx.phase {
                    "bind" => vec!["session.handle".to_owned()],
                    "submit" => vec!["task.handle".to_owned()],
                    "wait" => vec!["response.handle".to_owned()],
                    "finalize" => vec!["status.code".to_owned()],
                    _ => vec!["phase.output".to_owned()],
                },
                resource_bindings: vec![
                    ExecutionResourceBinding {
                        key: "bridge_surface".to_owned(),
                        kind: ExecutionResourceKind::Bridge,
                        capability_label: Some(domain_resource_capability_label(
                            ctx.domain_family,
                            ctx.selected_lowering_target,
                            "bridge_surface",
                            &ExecutionResourceKind::Bridge,
                        )),
                        value: ctx.bridge_surface.to_owned(),
                    },
                    ExecutionResourceBinding {
                        key: "scheduler_binding".to_owned(),
                        kind: ExecutionResourceKind::Scheduler,
                        capability_label: Some(domain_resource_capability_label(
                            ctx.domain_family,
                            ctx.selected_lowering_target,
                            "scheduler_binding",
                            &ExecutionResourceKind::Scheduler,
                        )),
                        value: ctx.scheduler_binding.to_owned(),
                    },
                    ExecutionResourceBinding {
                        key: "backend_summary".to_owned(),
                        kind: ExecutionResourceKind::Metadata,
                        capability_label: Some(domain_resource_capability_label(
                            ctx.domain_family,
                            ctx.selected_lowering_target,
                            "backend_summary",
                            &ExecutionResourceKind::Metadata,
                        )),
                        value: ctx.backend_summary.to_owned(),
                    },
                    ExecutionResourceBinding {
                        key: "active_session".to_owned(),
                        kind: ExecutionResourceKind::Handle,
                        capability_label: Some(domain_resource_capability_label(
                            ctx.domain_family,
                            ctx.selected_lowering_target,
                            "active_session",
                            &ExecutionResourceKind::Handle,
                        )),
                        value: "slot:session.handle".to_owned(),
                    },
                    ExecutionResourceBinding {
                        key: "active_task".to_owned(),
                        kind: ExecutionResourceKind::Handle,
                        capability_label: Some(domain_resource_capability_label(
                            ctx.domain_family,
                            ctx.selected_lowering_target,
                            "active_task",
                            &ExecutionResourceKind::Handle,
                        )),
                        value: "slot:task.handle".to_owned(),
                    },
                    ExecutionResourceBinding {
                        key: "active_response".to_owned(),
                        kind: ExecutionResourceKind::Response,
                        capability_label: Some(domain_resource_capability_label(
                            ctx.domain_family,
                            ctx.selected_lowering_target,
                            "active_response",
                            &ExecutionResourceKind::Response,
                        )),
                        value: "slot:response.handle".to_owned(),
                    },
                ],
                resolved_resources: Vec::new(),
                scheduler_keys: vec![
                    ctx.scheduler_binding.to_owned(),
                    ctx.phase.to_owned(),
                    "network".to_owned(),
                ],
                adapter_hint: Some(match ctx.phase {
                    "bind" => "adapter.bind.session-open".to_owned(),
                    "submit" => "adapter.submit.request-dispatch".to_owned(),
                    "wait" => "adapter.wait.callback-poll".to_owned(),
                    "finalize" => "adapter.finalize.response-commit".to_owned(),
                    _ => "adapter.execute.generic".to_owned(),
                }),
            })
        }

        fn phase_outcome(
            &self,
            ctx: &ExecutionPhaseContext<'_>,
            action: &ExecutionPhaseAction,
        ) -> Option<ExecutionPhaseOutcome> {
            Some(ExecutionPhaseOutcome {
                status: format!("adapter-{}", ctx.phase),
                produced_handles: action.output_handles.clone(),
                produced_slots: action
                    .output_handles
                    .iter()
                    .map(|key| ExecutionResourceBinding {
                        key: key.clone(),
                        kind: slot_resource_kind(key),
                        capability_label: Some(slot_resource_capability_label(key)),
                        value: format!("network://{}/{}", ctx.phase, key),
                    })
                    .collect(),
                notes: vec![
                    format!("domain={}", ctx.domain_family),
                    format!("kind={}", action.kind),
                ],
            })
        }
    }

    impl DomainAdapter for PassiveAdapter {
        fn adapter_id(&self) -> &'static str {
            "passive-adapter"
        }

        fn supports(&self, _unit: &BuildManifestDomainBuildUnit) -> bool {
            true
        }
    }

    fn prepared_network_execution<'a>(
        adapter: &'a dyn DomainAdapter,
        payload_blob: &'a DomainBuildUnitPayloadBlob,
        host_plan: &'a HostBridgePlanEntry,
        bridge_registry: &'a BridgeRegistryEntry,
        unit: &'a BuildManifestDomainBuildUnit,
    ) -> PreparedDomainExecution<'a> {
        PreparedDomainExecution {
            unit,
            payload_blob: Some(payload_blob),
            adapter,
            bridge_registry_entry: Some(bridge_registry),
            host_bridge_plan_entry: Some(host_plan),
            clock_domain: None,
            clock_edges: Vec::new(),
        }
    }

    fn sample_network_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("urlsession".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("socket-io".to_owned()),
            target_device: Some("urlsession-stack".to_owned()),
            ir_format: Some("host-ffi-plan".to_owned()),
            dispatch_abi: Some("nuis-host-call".to_owned()),
            backend_priority: Some(700),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("urlsession.socket-io".to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    fn sample_network_payload() -> DomainBuildUnitPayloadBlob {
        DomainBuildUnitPayloadBlob {
            domain_family: "network".to_owned(),
            package_id: "official.network".to_owned(),
            backend_family: Some("urlsession".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("socket-io".to_owned()),
            target_device: Some("urlsession-stack".to_owned()),
            ir_format: Some("host-ffi-plan".to_owned()),
            dispatch_abi: Some("nuis-host-call".to_owned()),
            backend_priority: Some(700),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("urlsession.socket-io".to_owned()),
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "toml".to_owned(),
            sections: vec![
                DomainBuildUnitPayloadBlobSection {
                    name: "lowering_plan".to_owned(),
                    bytes: b"execution_route = \"foundation-session-reactor\"\nphase_bind = \"socket-bind-or-session-open\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "backend_stub".to_owned(),
                    bytes: b"transport_ir = \"foundation-url-request\"\ntransport_entry_model = \"urlsession-task\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "bridge_plan".to_owned(),
                    bytes: b"phase_submit = \"packet-write-dispatch\"\nphase_wait = \"callback-or-read-ready\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "network_ir_sidecar".to_owned(),
                    bytes: b"schema = \"nuis-network-ir-sidecar-v1\"\nrequest = \"http-client-session\"".to_vec(),
                },
            ],
        }
    }

    fn sample_network_host_plan() -> HostBridgePlanEntry {
        HostBridgePlanEntry {
            domain_family: "network".to_owned(),
            package_id: "official.network".to_owned(),
            bridge_stub_path: "/tmp/network.bridge.stub.txt".to_owned(),
            bridge_surface: "host-ffi.bridge.network".to_owned(),
            scheduler_binding: "network-poll-bridge".to_owned(),
            phase_order: vec![
                "bind".to_owned(),
                "submit".to_owned(),
                "wait".to_owned(),
                "finalize".to_owned(),
            ],
            plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
        }
    }

    fn sample_network_bridge_registry() -> BridgeRegistryEntry {
        BridgeRegistryEntry {
            domain_family: "network".to_owned(),
            package_id: "official.network".to_owned(),
            backend_family: "urlsession".to_owned(),
            selected_lowering_target: "urlsession.socket-io".to_owned(),
            bridge_stub_path: "/tmp/network.bridge.stub.txt".to_owned(),
            payload_blob_path: "/tmp/network.payload.bin".to_owned(),
            plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
        }
    }

    fn sample_kernel_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.kernel".to_owned(),
            domain_family: "kernel".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("vulkan".to_owned()),
            vendor: Some("cross-vendor".to_owned()),
            device_class: Some("discrete-or-integrated-gpu".to_owned()),
            target_device: Some("vulkan-device".to_owned()),
            ir_format: Some("spirv".to_owned()),
            dispatch_abi: Some("vulkan-compute-pipeline".to_owned()),
            backend_priority: Some(30),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("vulkan.discrete-or-integrated-gpu".to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: "nustar.kernel".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    fn sample_kernel_payload() -> DomainBuildUnitPayloadBlob {
        DomainBuildUnitPayloadBlob {
            domain_family: "kernel".to_owned(),
            package_id: "official.kernel".to_owned(),
            backend_family: Some("vulkan".to_owned()),
            vendor: Some("cross-vendor".to_owned()),
            device_class: Some("discrete-or-integrated-gpu".to_owned()),
            target_device: Some("vulkan-device".to_owned()),
            ir_format: Some("spirv".to_owned()),
            dispatch_abi: Some("vulkan-compute-pipeline".to_owned()),
            backend_priority: Some(30),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("vulkan.discrete-or-integrated-gpu".to_owned()),
            contract_family: "nustar.kernel".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "toml".to_owned(),
            sections: vec![
                DomainBuildUnitPayloadBlobSection {
                    name: "lowering_plan".to_owned(),
                    bytes: b"dispatch_shape = \"grid-launch\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "backend_stub".to_owned(),
                    bytes: b"kernel_ir = \"spirv1.6\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "bridge_plan".to_owned(),
                    bytes: b"phase_submit = \"queue-dispatch-submit\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "kernel_ir_sidecar".to_owned(),
                    bytes: b"schema = \"nuis-kernel-ir-sidecar-v1\"\n[lowering_capabilities]\ncapability_owner = \"kernel-nustar\"\nnative_ir = \"spirv1.6\"\ntensor_lowering = \"storage-buffer-tensor-view\"\ndispatch_lowering = \"compute-grid-or-indirect\"\nresult_lowering = \"storage-buffer-result\"".to_vec(),
                },
            ],
        }
    }

    fn sample_shader_unit() -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: Some("metal".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("apple-silicon-gpu".to_owned()),
            target_device: Some("apple-gpu".to_owned()),
            ir_format: Some("msl".to_owned()),
            dispatch_abi: Some("metal-render-pipeline".to_owned()),
            backend_priority: Some(10),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            artifact_stub_path: None,
            artifact_stub_inline: None,
            artifact_payload_path: None,
            artifact_bridge_stub_path: None,
            artifact_ir_sidecar_path: None,
            artifact_bridge_stub_inline: None,
            artifact_payload_blob_path: None,
            artifact_payload_blob_bytes: None,
            artifact_payload_format: None,
            artifact_payload_blob_inline: None,
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    fn sample_shader_payload() -> DomainBuildUnitPayloadBlob {
        DomainBuildUnitPayloadBlob {
            domain_family: "shader".to_owned(),
            package_id: "official.shader".to_owned(),
            backend_family: Some("metal".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("apple-silicon-gpu".to_owned()),
            target_device: Some("apple-gpu".to_owned()),
            ir_format: Some("msl".to_owned()),
            dispatch_abi: Some("metal-render-pipeline".to_owned()),
            backend_priority: Some(10),
            verification: Some("contract-only".to_owned()),
            selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
            contract_family: "nustar.shader".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
            payload_kind: "contract-sidecar".to_owned(),
            payload_format: "toml".to_owned(),
            sections: vec![
                DomainBuildUnitPayloadBlobSection {
                    name: "lowering_plan".to_owned(),
                    bytes: b"dispatch_encoding_model = \"tile-and-threadgroup\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "backend_stub".to_owned(),
                    bytes: b"shader_ir = \"msl2.4\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "bridge_plan".to_owned(),
                    bytes: b"phase_submit = \"render-submit-bridge\"".to_vec(),
                },
                DomainBuildUnitPayloadBlobSection {
                    name: "shader_ir_sidecar".to_owned(),
                    bytes: b"schema = \"nuis-shader-ir-sidecar-v1\"\n[lowering_capabilities]\ncapability_owner = \"shader-nustar\"\nnative_ir = \"msl2.4\"\npipeline_lowering = \"metal-render-pipeline-state\"\nresource_lowering = \"argument-buffer-table\"\ntexture_lowering = \"texture2d-sampler-argument\"".to_vec(),
                },
            ],
        }
    }

    fn sample_host_plan(
        domain_family: &str,
        package_id: &str,
        scheduler: &str,
    ) -> HostBridgePlanEntry {
        HostBridgePlanEntry {
            domain_family: domain_family.to_owned(),
            package_id: package_id.to_owned(),
            bridge_stub_path: format!("/tmp/{domain_family}.bridge.stub.txt"),
            bridge_surface: format!("host-ffi.bridge.{domain_family}"),
            scheduler_binding: scheduler.to_owned(),
            phase_order: vec![
                "bind".to_owned(),
                "submit".to_owned(),
                "wait".to_owned(),
                "finalize".to_owned(),
            ],
            plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
        }
    }

    fn sample_bridge_registry(
        domain_family: &str,
        package_id: &str,
        backend: &str,
        target: &str,
    ) -> BridgeRegistryEntry {
        BridgeRegistryEntry {
            domain_family: domain_family.to_owned(),
            package_id: package_id.to_owned(),
            backend_family: backend.to_owned(),
            selected_lowering_target: target.to_owned(),
            bridge_stub_path: format!("/tmp/{domain_family}.bridge.stub.txt"),
            payload_blob_path: format!("/tmp/{domain_family}.payload.bin"),
            plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
        }
    }

    #[test]
    fn verify_rejects_incomplete_contract() {
        let contract = ExecutionContract {
            yir_version: "",
            fabric_abi_version: "federated-abi-v1",
            profile: ExecutionProfile::Aot,
        };

        assert_eq!(
            Executor.verify(&contract),
            Err("execution contract is incomplete")
        );
    }

    #[test]
    fn executor_plans_phase_bindings_from_prepared_execution() {
        let adapter = NetworkAdapter;
        let unit = sample_network_unit();
        let payload = sample_network_payload();
        let host_plan = sample_network_host_plan();
        let bridge_registry = sample_network_bridge_registry();
        let prepared =
            prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

        let plan = Executor.plan(&prepared).unwrap();

        assert_eq!(plan.domain_family, "network");
        assert_eq!(plan.package_id, "official.network");
        assert_eq!(plan.adapter_id, "network-test-adapter");
        assert_eq!(
            plan.selected_lowering_target.as_deref(),
            Some("urlsession.socket-io")
        );
        assert_eq!(plan.phases.len(), 4);
        assert_eq!(plan.phases[0].phase, "bind");
        assert_eq!(plan.phases[0].role, crate::RuntimeRole::Bind);
        assert_eq!(plan.phases[0].bridge_surface, "host-ffi.bridge.network");
        assert_eq!(plan.phases[0].scheduler_binding, "network-poll-bridge");
        assert_eq!(
            plan.phases[0].lowering_summary,
            "execution_route = \"foundation-session-reactor\""
        );
        assert_eq!(
            plan.phases[0].backend_summary,
            "transport_ir = \"foundation-url-request\""
        );
        assert_eq!(
            plan.phases[0].bridge_summary,
            "phase_submit = \"packet-write-dispatch\""
        );
        assert_eq!(
            plan.phases[0].ir_sidecar_summary.as_deref(),
            Some("schema = \"nuis-network-ir-sidecar-v1\"")
        );
        assert_eq!(plan.phases[0].action.kind, "network.bind");
        assert_eq!(
            plan.phases[0].action.input_handles,
            vec!["authority.text".to_owned()]
        );
        assert_eq!(
            plan.phases[0].action.output_handles,
            vec!["session.handle".to_owned()]
        );
        assert_eq!(
            plan.phases[0].action.resolved_inputs,
            Vec::<ExecutionResourceBinding>::new()
        );
        assert_eq!(
            plan.phases[0].action.resolved_resources,
            Vec::<ExecutionResourceBinding>::new()
        );
        assert_eq!(
            plan.phases[0].action.adapter_hint.as_deref(),
            Some("adapter.bind.session-open")
        );
        assert_eq!(plan.phases[2].action.kind, "network.wait");
        assert_eq!(
            plan.phases[2].action.adapter_hint.as_deref(),
            Some("adapter.wait.callback-poll")
        );

        let summary = plan.render_summary();
        assert!(summary.contains("selected_lowering_target = urlsession.socket-io"));
        assert!(summary.contains("phase bind role=Bind"));
        assert!(summary.contains(
            "resource bridge_surface kind=Bridge capability=cap.network.urlsession_socket_io.bridge.bridge_surface"
        ));
        assert!(summary.contains(
            "resource active_session kind=Handle capability=cap.network.urlsession_socket_io.handle.active_session"
        ));
    }

    #[test]
    fn executor_exposes_clock_protocol_as_phase_metadata_resource() {
        let adapter = PassiveAdapter;
        let unit = sample_network_unit();
        let payload = sample_network_payload();
        let host_plan = sample_network_host_plan();
        let bridge_registry = sample_network_bridge_registry();
        let clock_domain = ClockDomain {
            index: 0,
            domain_family: "network".to_owned(),
            package_id: "official.network".to_owned(),
            clock_domain_id: "network.clock.io.v1".to_owned(),
            clock_kind: "io-monotonic".to_owned(),
            clock_epoch_kind: "io-epoch".to_owned(),
            clock_resolution: "io-ready-step".to_owned(),
            clock_bridge_default: "global->io:bridge".to_owned(),
            lifecycle_hook: "on_network_bridge_progress".to_owned(),
        };
        let clock_edge = ClockEdge {
            index: 1,
            from: "t0000.nuis.bootstrap.lifecycle.v1".to_owned(),
            to: "t0001.network".to_owned(),
            relation: "happens-before".to_owned(),
            source: "hetero.node.0".to_owned(),
        };
        let mut prepared =
            prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);
        prepared.clock_domain = Some(&clock_domain);
        prepared.clock_edges = vec![&clock_edge];

        let plan = Executor.plan(&prepared).unwrap();

        assert!(plan
            .clock_summary
            .as_deref()
            .unwrap()
            .contains("clock_domain=network.clock.io.v1"));
        assert!(plan.phases[0]
            .clock_summary
            .as_deref()
            .unwrap()
            .contains("t0000.nuis.bootstrap.lifecycle.v1->t0001.network"));
        assert_eq!(
            plan.clock_gate.wait_on,
            vec!["t0000.nuis.bootstrap.lifecycle.v1".to_owned()]
        );
        assert_eq!(plan.clock_gate.emits, vec!["t0001.network".to_owned()]);
        assert_eq!(plan.phases[0].clock_gate, plan.clock_gate);
        let clock_binding = plan.phases[0]
            .action
            .resource_bindings
            .iter()
            .find(|binding| binding.key == "clock_protocol")
            .expect("default phase action should expose clock protocol metadata");
        assert_eq!(clock_binding.kind, ExecutionResourceKind::Metadata);
        assert!(clock_binding.value.contains("bridge=global->io:bridge"));

        let trace = Executor.execute_prepared_plan(&adapter, &plan).unwrap();
        assert_eq!(trace.events[0].clock_gate, plan.clock_gate);
        let validation = trace
            .validate_clock_gates(&["t0000.nuis.bootstrap.lifecycle.v1".to_owned()])
            .unwrap();
        assert_eq!(
            validation.initial_timestamps,
            vec!["t0000.nuis.bootstrap.lifecycle.v1".to_owned()]
        );
        assert_eq!(validation.observed_emits, vec!["t0001.network".to_owned()]);
        assert_eq!(
            validation.final_timestamps,
            vec![
                "t0000.nuis.bootstrap.lifecycle.v1".to_owned(),
                "t0001.network".to_owned()
            ]
        );
        let error = trace.validate_clock_gates(&[]).unwrap_err();
        assert!(error
            .to_string()
            .contains("missing timestamp `t0000.nuis.bootstrap.lifecycle.v1`"));
    }

    #[test]
    fn executor_emits_trace_for_prepared_execution() {
        let adapter = NetworkAdapter;
        let unit = sample_network_unit();
        let payload = sample_network_payload();
        let host_plan = sample_network_host_plan();
        let bridge_registry = sample_network_bridge_registry();
        let prepared =
            prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

        let trace = Executor.execute_prepared(&prepared).unwrap();

        assert_eq!(trace.domain_family, "network");
        assert_eq!(trace.phase_count, 4);
        assert_eq!(trace.events[0].phase, "bind");
        assert_eq!(trace.events[0].role, crate::RuntimeRole::Bind);
        assert_eq!(trace.events[1].phase, "submit");
        assert_eq!(trace.events[1].role, crate::RuntimeRole::Execute);
        assert_eq!(trace.events[1].action.kind, "network.submit");
        assert_eq!(
            trace.events[1].action.adapter_hint.as_deref(),
            Some("adapter.submit.request-dispatch")
        );
        assert_eq!(
            trace.events[0].state_before.available_handles,
            Vec::<String>::new()
        );
        assert_eq!(
            trace.events[0].state_after.available_handles,
            vec!["session.handle".to_owned()]
        );
        assert_eq!(
            trace.events[0].state_after.handle_slots,
            vec![ExecutionResourceBinding {
                key: "session.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.session.handle".to_owned()),
                value: "network://bind/session.handle".to_owned()
            }]
        );
        assert_eq!(
            trace.events[1].state_before.available_handles,
            vec!["session.handle".to_owned()]
        );
        assert_eq!(
            trace.events[1].state_before.handle_slots,
            vec![ExecutionResourceBinding {
                key: "session.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.session.handle".to_owned()),
                value: "network://bind/session.handle".to_owned()
            }]
        );
        assert_eq!(
            trace.events[1].action.input_handles,
            vec!["session.handle".to_owned(), "request.packet".to_owned()]
        );
        assert_eq!(
            trace.events[1].action.resolved_inputs,
            vec![ExecutionResourceBinding {
                key: "session.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.session.handle".to_owned()),
                value: "network://bind/session.handle".to_owned()
            }]
        );
        assert_eq!(
            trace.events[1].action.resolved_resources,
            vec![
                ExecutionResourceBinding {
                    key: "bridge_surface".to_owned(),
                    kind: ExecutionResourceKind::Bridge,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "bridge_surface",
                        &ExecutionResourceKind::Bridge,
                    )),
                    value: "host-ffi.bridge.network".to_owned()
                },
                ExecutionResourceBinding {
                    key: "scheduler_binding".to_owned(),
                    kind: ExecutionResourceKind::Scheduler,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "scheduler_binding",
                        &ExecutionResourceKind::Scheduler,
                    )),
                    value: "network-poll-bridge".to_owned()
                },
                ExecutionResourceBinding {
                    key: "backend_summary".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "backend_summary",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: "transport_ir = \"foundation-url-request\"".to_owned()
                },
                ExecutionResourceBinding {
                    key: "active_session".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "active_session",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "network://bind/session.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "active_task".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "active_task",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "unresolved:task.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "active_response".to_owned(),
                    kind: ExecutionResourceKind::Response,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "active_response",
                        &ExecutionResourceKind::Response,
                    )),
                    value: "unresolved:response.handle".to_owned()
                }
            ]
        );
        assert_eq!(trace.events[1].outcome.status, "adapter-submit");
        assert_eq!(
            trace.events[1].outcome.produced_handles,
            vec!["task.handle".to_owned()]
        );
        assert_eq!(
            trace.events[1].outcome.produced_slots,
            vec![ExecutionResourceBinding {
                key: "task.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.task.handle".to_owned()),
                value: "network://submit/task.handle".to_owned()
            }]
        );
        assert_eq!(
            trace.events[1].state_after.available_handles,
            vec!["session.handle".to_owned(), "task.handle".to_owned()]
        );
        assert_eq!(
            trace.events[1].state_after.handle_slots,
            vec![
                ExecutionResourceBinding {
                    key: "session.handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some("cap.handle.session.handle".to_owned()),
                    value: "network://bind/session.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "task.handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some("cap.handle.task.handle".to_owned()),
                    value: "network://submit/task.handle".to_owned()
                }
            ]
        );
        assert_eq!(trace.events[3].phase, "finalize");
        assert_eq!(trace.events[3].bridge_surface, "host-ffi.bridge.network");
        assert_eq!(trace.events[3].scheduler_binding, "network-poll-bridge");
        assert_eq!(trace.events[3].adapter_id, "network-test-adapter");
        assert_eq!(
            trace.events[3].action.adapter_hint.as_deref(),
            Some("adapter.finalize.response-commit")
        );
        assert_eq!(trace.events[3].outcome.status, "adapter-finalize");
        assert_eq!(
            trace.events[3].outcome.notes,
            vec![
                "domain=network".to_owned(),
                "kind=network.finalize".to_owned()
            ]
        );
        assert_eq!(
            trace.events[3].state_before.available_handles,
            vec![
                "session.handle".to_owned(),
                "task.handle".to_owned(),
                "response.handle".to_owned()
            ]
        );
        assert_eq!(
            trace.events[3].action.resolved_inputs,
            vec![ExecutionResourceBinding {
                key: "response.handle".to_owned(),
                kind: ExecutionResourceKind::Handle,
                capability_label: Some("cap.handle.response.handle".to_owned()),
                value: "network://wait/response.handle".to_owned()
            }]
        );
        assert_eq!(
            trace.events[3].action.resolved_resources,
            vec![
                ExecutionResourceBinding {
                    key: "bridge_surface".to_owned(),
                    kind: ExecutionResourceKind::Bridge,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "bridge_surface",
                        &ExecutionResourceKind::Bridge,
                    )),
                    value: "host-ffi.bridge.network".to_owned()
                },
                ExecutionResourceBinding {
                    key: "scheduler_binding".to_owned(),
                    kind: ExecutionResourceKind::Scheduler,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "scheduler_binding",
                        &ExecutionResourceKind::Scheduler,
                    )),
                    value: "network-poll-bridge".to_owned()
                },
                ExecutionResourceBinding {
                    key: "backend_summary".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "backend_summary",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: "transport_ir = \"foundation-url-request\"".to_owned()
                },
                ExecutionResourceBinding {
                    key: "active_session".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "active_session",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "network://bind/session.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "active_task".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "active_task",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "network://submit/task.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "active_response".to_owned(),
                    kind: ExecutionResourceKind::Response,
                    capability_label: Some(domain_resource_capability_label(
                        "network",
                        Some("urlsession.socket-io"),
                        "active_response",
                        &ExecutionResourceKind::Response,
                    )),
                    value: "network://wait/response.handle".to_owned()
                }
            ]
        );
        assert_eq!(
            trace.events[3].state_after.available_handles,
            vec![
                "session.handle".to_owned(),
                "task.handle".to_owned(),
                "response.handle".to_owned(),
                "status.code".to_owned()
            ]
        );
        assert_eq!(
            trace.events[3].state_after.handle_slots,
            vec![
                ExecutionResourceBinding {
                    key: "response.handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some("cap.handle.response.handle".to_owned()),
                    value: "network://wait/response.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "session.handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some("cap.handle.session.handle".to_owned()),
                    value: "network://bind/session.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "status.code".to_owned(),
                    kind: ExecutionResourceKind::Slot,
                    capability_label: Some("cap.slot.status.code".to_owned()),
                    value: "network://finalize/status.code".to_owned()
                },
                ExecutionResourceBinding {
                    key: "task.handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some("cap.handle.task.handle".to_owned()),
                    value: "network://submit/task.handle".to_owned()
                }
            ]
        );

        let summary = trace.render_summary();
        assert!(summary.contains("event submit role=Execute adapter=network-test-adapter"));
        assert!(summary.contains(
            "resolved_resource active_session kind=Handle capability=cap.network.urlsession_socket_io.handle.active_session value=network://bind/session.handle"
        ));
        assert!(summary.contains(
            "produced_slot task.handle kind=Handle capability=cap.handle.task.handle value=network://submit/task.handle"
        ));
    }

    #[test]
    fn executor_default_kernel_plan_uses_buffer_and_dispatch_resources() {
        let adapter = PassiveAdapter;
        let unit = sample_kernel_unit();
        let payload = sample_kernel_payload();
        let host_plan = sample_host_plan("kernel", "official.kernel", "hetero-submit-bridge");
        let bridge_registry = sample_bridge_registry(
            "kernel",
            "official.kernel",
            "vulkan",
            "vulkan.discrete-or-integrated-gpu",
        );
        let prepared =
            prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

        let plan = Executor.plan(&prepared).unwrap();

        assert_eq!(
            plan.phases[0].ir_sidecar_summary.as_deref(),
            Some(
                "capability_owner=kernel-nustar native_ir=spirv1.6 tensor_lowering=storage-buffer-tensor-view dispatch_lowering=compute-grid-or-indirect result_lowering=storage-buffer-result"
            )
        );
        assert_eq!(
            plan.phases[0].action.input_handles,
            vec!["kernel.buffer".to_owned(), "queue.slot".to_owned()]
        );
        assert_eq!(
            plan.phases[1].action.output_handles,
            vec!["dispatch.handle".to_owned()]
        );
        assert_eq!(
            plan.phases[1].action.resource_bindings,
            vec![
                ExecutionResourceBinding {
                    key: "bridge_surface".to_owned(),
                    kind: ExecutionResourceKind::Bridge,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "bridge_surface",
                        &ExecutionResourceKind::Bridge,
                    )),
                    value: "host-ffi.bridge.kernel".to_owned()
                },
                ExecutionResourceBinding {
                    key: "scheduler_binding".to_owned(),
                    kind: ExecutionResourceKind::Scheduler,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "scheduler_binding",
                        &ExecutionResourceKind::Scheduler,
                    )),
                    value: "hetero-submit-bridge".to_owned()
                },
                ExecutionResourceBinding {
                    key: "backend_summary".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "backend_summary",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: "kernel_ir = \"spirv1.6\"".to_owned()
                },
                ExecutionResourceBinding {
                    key: "lowering_capabilities".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "lowering_capabilities",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: "capability_owner=kernel-nustar native_ir=spirv1.6 tensor_lowering=storage-buffer-tensor-view dispatch_lowering=compute-grid-or-indirect result_lowering=storage-buffer-result".to_owned()
                },
                ExecutionResourceBinding {
                    key: "kernel_buffer".to_owned(),
                    kind: ExecutionResourceKind::Buffer,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "kernel_buffer",
                        &ExecutionResourceKind::Buffer,
                    )),
                    value: "slot:kernel.buffer".to_owned()
                },
                ExecutionResourceBinding {
                    key: "dispatch_handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "dispatch_handle",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "slot:dispatch.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "result_buffer".to_owned(),
                    kind: ExecutionResourceKind::Buffer,
                    capability_label: Some(domain_resource_capability_label(
                        "kernel",
                        Some("vulkan.discrete-or-integrated-gpu"),
                        "result_buffer",
                        &ExecutionResourceKind::Buffer,
                    )),
                    value: "slot:result.buffer".to_owned()
                }
            ]
        );
    }

    #[test]
    fn executor_default_shader_plan_uses_shader_and_frame_resources() {
        let adapter = PassiveAdapter;
        let unit = sample_shader_unit();
        let payload = sample_shader_payload();
        let host_plan = sample_host_plan("shader", "official.shader", "render-submit-bridge");
        let bridge_registry = sample_bridge_registry(
            "shader",
            "official.shader",
            "metal",
            "metal.apple-silicon-gpu",
        );
        let prepared =
            prepared_network_execution(&adapter, &payload, &host_plan, &bridge_registry, &unit);

        let trace = Executor.execute_prepared(&prepared).unwrap();
        let plan = Executor.plan(&prepared).unwrap();

        assert_eq!(
            plan.phases[0].ir_sidecar_summary.as_deref(),
            Some(
                "capability_owner=shader-nustar native_ir=msl2.4 pipeline_lowering=metal-render-pipeline-state resource_lowering=argument-buffer-table texture_lowering=texture2d-sampler-argument"
            )
        );

        assert_eq!(
            trace.events[0].action.input_handles,
            vec!["shader.buffer".to_owned(), "frame.target".to_owned()]
        );
        assert_eq!(
            trace.events[1].action.output_handles,
            vec!["draw.handle".to_owned()]
        );
        assert_eq!(
            trace.events[3].action.resolved_resources,
            vec![
                ExecutionResourceBinding {
                    key: "bridge_surface".to_owned(),
                    kind: ExecutionResourceKind::Bridge,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "bridge_surface",
                        &ExecutionResourceKind::Bridge,
                    )),
                    value: "host-ffi.bridge.shader".to_owned()
                },
                ExecutionResourceBinding {
                    key: "scheduler_binding".to_owned(),
                    kind: ExecutionResourceKind::Scheduler,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "scheduler_binding",
                        &ExecutionResourceKind::Scheduler,
                    )),
                    value: "render-submit-bridge".to_owned()
                },
                ExecutionResourceBinding {
                    key: "backend_summary".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "backend_summary",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: "shader_ir = \"msl2.4\"".to_owned()
                },
                ExecutionResourceBinding {
                    key: "lowering_capabilities".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "lowering_capabilities",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: "capability_owner=shader-nustar native_ir=msl2.4 pipeline_lowering=metal-render-pipeline-state resource_lowering=argument-buffer-table texture_lowering=texture2d-sampler-argument".to_owned()
                },
                ExecutionResourceBinding {
                    key: "shader_buffer".to_owned(),
                    kind: ExecutionResourceKind::Buffer,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "shader_buffer",
                        &ExecutionResourceKind::Buffer,
                    )),
                    value: "mock://bind/shader.buffer".to_owned()
                },
                ExecutionResourceBinding {
                    key: "draw_handle".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "draw_handle",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "mock://submit/draw.handle".to_owned()
                },
                ExecutionResourceBinding {
                    key: "frame_target".to_owned(),
                    kind: ExecutionResourceKind::Response,
                    capability_label: Some(domain_resource_capability_label(
                        "shader",
                        Some("metal.apple-silicon-gpu"),
                        "frame_target",
                        &ExecutionResourceKind::Response,
                    )),
                    value: "mock://wait/frame.target".to_owned()
                }
            ]
        );

        let summary = trace.render_summary();
        assert!(summary.contains("event finalize role=Execute adapter=passive-adapter"));
        assert!(summary.contains(
            "resolved_resource shader_buffer kind=Buffer capability=cap.shader.metal_apple_silicon_gpu.buffer.shader_buffer value=mock://bind/shader.buffer"
        ));
        assert!(summary.contains(
            "resolved_resource frame_target kind=Response capability=cap.shader.metal_apple_silicon_gpu.frame.frame_target value=mock://wait/frame.target"
        ));
    }
}
