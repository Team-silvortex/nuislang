use std::collections::BTreeSet;
use std::path::Path;

use nuis_artifact::BuildManifestDomainBuildUnit;
use nuis_runtime::{
    AdapterRegistry, BridgeExecutor, ClockProtocolRuntimeSummary, DomainAdapter,
    ExecutionClockValidation, ExecutionPhaseAction, ExecutionPhaseBinding, ExecutionPhaseContext,
    ExecutionPhaseOutcome, ExecutionPlan, ExecutionResourceBinding, ExecutionStateSnapshot,
    ExecutionTrace, ExecutionTraceEvent, Executor, RuntimeLoader, RuntimeRole,
};

use crate::execution_inspect::{
    execution_inspect_issues, ExecutionInspectDomainOverview, ExecutionInspectIssue,
    ExecutionInspectOverview,
};
use crate::{
    json_bool_field, json_escape, json_optional_string_field, json_string_array_field,
    json_string_field, json_usize_field, load_nuis_compiled_artifact,
};

struct InspectExecutionAdapter;

struct InspectExecutionSection {
    domain_family: String,
    plan: ExecutionPlan,
    trace: ExecutionTrace,
    clock_validation: Option<ExecutionClockValidation>,
    clock_validation_error: Option<String>,
}

struct InspectClockValidationSummary {
    valid: bool,
    checked_domains: usize,
    failed_domains: usize,
    initial_timestamps: Vec<String>,
    observed_emits: Vec<String>,
    final_timestamps: Vec<String>,
    errors: Vec<String>,
}

impl DomainAdapter for InspectExecutionAdapter {
    fn adapter_id(&self) -> &'static str {
        "nuisc-inspect-adapter"
    }

    fn supports(&self, _unit: &BuildManifestDomainBuildUnit) -> bool {
        true
    }

    fn phase_outcome(
        &self,
        _ctx: &ExecutionPhaseContext<'_>,
        _action: &nuis_runtime::ExecutionPhaseAction,
    ) -> Option<ExecutionPhaseOutcome> {
        None
    }
}

fn inspect_execution_sections(input: &Path) -> Result<Vec<InspectExecutionSection>, String> {
    let artifact = load_nuis_compiled_artifact(input)?;
    let loaded = RuntimeLoader
        .load_from_compiled_artifact(artifact)
        .map_err(|error| error.to_string())?;
    let mut adapters = AdapterRegistry::new();
    adapters.register(Box::new(InspectExecutionAdapter));

    let bridge = BridgeExecutor;
    let executor = Executor;
    let mut sections = Vec::new();
    let mut satisfied_timestamps = initial_clock_timestamps(&loaded);

    for unit in loaded.heterogeneous_units() {
        let prepared = bridge
            .prepare(&loaded, &adapters, &unit.domain_family)
            .map_err(|error| error.to_string())?;
        let plan = executor
            .plan(&prepared)
            .map_err(|error| error.to_string())?;
        let trace = executor
            .execute_prepared_plan(prepared.adapter, &plan)
            .map_err(|error| error.to_string())?;
        let validation_result = trace.validate_clock_gates(&satisfied_timestamps);
        let (clock_validation, clock_validation_error) = match validation_result {
            Ok(validation) => {
                satisfied_timestamps = validation.final_timestamps.clone();
                (Some(validation), None)
            }
            Err(error) => (None, Some(error.to_string())),
        };
        sections.push(InspectExecutionSection {
            domain_family: unit.domain_family.clone(),
            plan,
            trace,
            clock_validation,
            clock_validation_error,
        });
    }

    Ok(sections)
}

fn initial_clock_timestamps(loaded: &nuis_runtime::LoadedExecutable) -> Vec<String> {
    let Some(protocol) = loaded.clock_protocol.as_ref() else {
        return Vec::new();
    };
    let emitted = protocol
        .happens_before_edges()
        .map(|edge| edge.to.clone())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut initial = Vec::new();
    for edge in protocol.happens_before_edges() {
        for timestamp in edge
            .from
            .split('|')
            .map(str::trim)
            .filter(|timestamp| !timestamp.is_empty())
        {
            if !emitted.contains(timestamp) && seen.insert(timestamp.to_owned()) {
                initial.push(timestamp.to_owned());
            }
        }
    }
    initial
}

fn clock_validation_summary(sections: &[InspectExecutionSection]) -> InspectClockValidationSummary {
    let checked_domains = sections
        .iter()
        .filter(|section| {
            section.clock_validation.is_some() || section.clock_validation_error.is_some()
        })
        .count();
    let failed_domains = sections
        .iter()
        .filter(|section| section.clock_validation_error.is_some())
        .count();
    let mut initial_timestamps = Vec::new();
    let mut observed_emits = Vec::new();
    let mut final_timestamps = Vec::new();
    let mut seen_emits = BTreeSet::new();
    let mut errors = Vec::new();

    for section in sections {
        if let Some(validation) = &section.clock_validation {
            if initial_timestamps.is_empty() {
                initial_timestamps = validation.initial_timestamps.clone();
            }
            for timestamp in &validation.observed_emits {
                if seen_emits.insert(timestamp.clone()) {
                    observed_emits.push(timestamp.clone());
                }
            }
            final_timestamps = validation.final_timestamps.clone();
        }
        if let Some(error) = &section.clock_validation_error {
            errors.push(format!("{}: {}", section.domain_family, error));
        }
    }

    InspectClockValidationSummary {
        valid: failed_domains == 0,
        checked_domains,
        failed_domains,
        initial_timestamps,
        observed_emits,
        final_timestamps,
        errors,
    }
}

fn inspect_clock_protocol_summary(
    input: &Path,
) -> Result<Option<ClockProtocolRuntimeSummary>, String> {
    let artifact = load_nuis_compiled_artifact(input)?;
    let loaded = RuntimeLoader
        .load_from_compiled_artifact(artifact)
        .map_err(|error| error.to_string())?;
    Ok(loaded.clock_protocol_summary())
}

fn execution_overview_from_sections(
    sections: &[InspectExecutionSection],
) -> ExecutionInspectOverview {
    let domains = sections
        .iter()
        .map(|section| {
            let mut resource_keys = BTreeSet::new();
            let mut output_handles = BTreeSet::new();
            for phase in &section.plan.phases {
                for binding in &phase.action.resource_bindings {
                    resource_keys.insert(binding.key.clone());
                }
                for output in &phase.action.output_handles {
                    output_handles.insert(output.clone());
                }
            }
            ExecutionInspectDomainOverview {
                domain_family: section.domain_family.clone(),
                selected_lowering_target: section.plan.selected_lowering_target.clone(),
                phase_count: section.plan.phases.len(),
                event_count: section.trace.events.len(),
                resource_keys: resource_keys.into_iter().collect(),
                output_handles: output_handles.into_iter().collect(),
            }
        })
        .collect::<Vec<_>>();
    ExecutionInspectOverview {
        heterogeneous_domains: domains.len(),
        domains,
    }
}

pub(crate) fn inspect_execution_overview(input: &Path) -> Result<ExecutionInspectOverview, String> {
    let sections = inspect_execution_sections(input)?;
    Ok(execution_overview_from_sections(&sections))
}

pub(crate) fn render_execution_report(input: &Path) -> Result<String, String> {
    let artifact = load_nuis_compiled_artifact(input)?;
    let sections = inspect_execution_sections(input)?;
    let clock_summary = inspect_clock_protocol_summary(input)?;
    let clock_validation = clock_validation_summary(&sections);
    let mut lines = vec![
        format!("nuis execution: {}", input.display()),
        format!("  packaging_mode: {}", artifact.packaging_mode),
        format!("  binary_name: {}", artifact.binary_name),
        format!(
            "  domain_families: {}",
            artifact.envelope.domain_families.join(", ")
        ),
        format!("  heterogeneous_execution_domains: {}", sections.len()),
    ];
    if let Some(summary) = clock_summary {
        lines.push(format!(
            "  clock_protocol: schema={} mode={} domains={} edges={} happens_before={} valid={}",
            summary.schema,
            summary.mode,
            summary.domains,
            summary.edges,
            summary.happens_before_edges,
            summary.validation_valid
        ));
    } else {
        lines.push("  clock_protocol: <not available>".to_owned());
    }
    lines.push(format!(
        "  clock_validation: valid={} checked_domains={} failed_domains={} initial={} observed_emits={} final={}",
        clock_validation.valid,
        clock_validation.checked_domains,
        clock_validation.failed_domains,
        join_or_dash(&clock_validation.initial_timestamps),
        join_or_dash(&clock_validation.observed_emits),
        join_or_dash(&clock_validation.final_timestamps)
    ));
    for error in &clock_validation.errors {
        lines.push(format!("  clock_validation_error: {error}"));
    }

    if sections.is_empty() {
        lines.push("  execution_plan: <no heterogeneous domains available>".to_owned());
        return Ok(lines.join("\n"));
    }

    for section in sections {
        lines.push(format!("  domain: {}", section.domain_family));
        if let Some(validation) = &section.clock_validation {
            lines.push(format!(
                "    clock_validation: valid=true initial={} observed_emits={} final={}",
                join_or_dash(&validation.initial_timestamps),
                join_or_dash(&validation.observed_emits),
                join_or_dash(&validation.final_timestamps)
            ));
        } else if let Some(error) = &section.clock_validation_error {
            lines.push(format!("    clock_validation: valid=false error={error}"));
        }
        for line in section.plan.render_summary().lines() {
            lines.push(format!("    plan: {line}"));
        }
        for line in section.trace.render_summary().lines() {
            lines.push(format!("    trace: {line}"));
        }
    }

    Ok(lines.join("\n"))
}

fn runtime_role_json_value(role: RuntimeRole) -> String {
    format!("{role:?}")
}

fn execution_resource_binding_json(binding: &ExecutionResourceBinding) -> String {
    let fields = vec![
        json_string_field("key", &binding.key),
        json_string_field("kind", &format!("{:?}", binding.kind)),
        json_optional_string_field("capability_label", binding.capability_label.as_deref()),
        json_string_field("value", &binding.value),
    ];
    format!("{{{}}}", fields.join(","))
}

fn execution_state_snapshot_json(snapshot: &ExecutionStateSnapshot) -> String {
    let handle_slots = snapshot
        .handle_slots
        .iter()
        .map(execution_resource_binding_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_array_field("available_handles", &snapshot.available_handles),
        format!("\"handle_slots\":[{}]", handle_slots),
    ];
    format!("{{{}}}", fields.join(","))
}

fn execution_clock_gate_json(gate: &nuis_runtime::ExecutionClockGate) -> String {
    let fields = vec![
        json_string_array_field("wait_on", &gate.wait_on),
        json_string_array_field("emits", &gate.emits),
    ];
    format!("{{{}}}", fields.join(","))
}

fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_owned()
    } else {
        values.join(", ")
    }
}

fn execution_clock_validation_json(
    validation: Option<&ExecutionClockValidation>,
    error: Option<&str>,
) -> String {
    let Some(validation) = validation else {
        let fields = vec![
            json_bool_field("valid", false),
            json_optional_string_field("error", error),
        ];
        return format!("{{{}}}", fields.join(","));
    };
    let fields = vec![
        json_bool_field("valid", true),
        json_string_array_field("initial_timestamps", &validation.initial_timestamps),
        json_string_array_field("observed_emits", &validation.observed_emits),
        json_string_array_field("final_timestamps", &validation.final_timestamps),
        json_optional_string_field("error", None),
    ];
    format!("{{{}}}", fields.join(","))
}

fn inspect_clock_validation_summary_json(summary: &InspectClockValidationSummary) -> String {
    let fields = vec![
        json_bool_field("valid", summary.valid),
        json_usize_field("checked_domains", summary.checked_domains),
        json_usize_field("failed_domains", summary.failed_domains),
        json_string_array_field("initial_timestamps", &summary.initial_timestamps),
        json_string_array_field("observed_emits", &summary.observed_emits),
        json_string_array_field("final_timestamps", &summary.final_timestamps),
        json_string_array_field("errors", &summary.errors),
    ];
    format!("\"clock_validation\":{{{}}}", fields.join(","))
}

fn execution_phase_action_json(action: &ExecutionPhaseAction) -> String {
    let resolved_inputs = action
        .resolved_inputs
        .iter()
        .map(execution_resource_binding_json)
        .collect::<Vec<_>>()
        .join(",");
    let resource_bindings = action
        .resource_bindings
        .iter()
        .map(execution_resource_binding_json)
        .collect::<Vec<_>>()
        .join(",");
    let resolved_resources = action
        .resolved_resources
        .iter()
        .map(execution_resource_binding_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("kind", &action.kind),
        json_string_array_field("input_handles", &action.input_handles),
        format!("\"resolved_inputs\":[{}]", resolved_inputs),
        json_string_array_field("output_handles", &action.output_handles),
        format!("\"resource_bindings\":[{}]", resource_bindings),
        format!("\"resolved_resources\":[{}]", resolved_resources),
        json_string_array_field("scheduler_keys", &action.scheduler_keys),
        json_optional_string_field("adapter_hint", action.adapter_hint.as_deref()),
    ];
    format!("{{{}}}", fields.join(","))
}

fn execution_phase_outcome_json(outcome: &ExecutionPhaseOutcome) -> String {
    let produced_slots = outcome
        .produced_slots
        .iter()
        .map(execution_resource_binding_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("status", &outcome.status),
        json_string_array_field("produced_handles", &outcome.produced_handles),
        format!("\"produced_slots\":[{}]", produced_slots),
        json_string_array_field("notes", &outcome.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

fn execution_phase_binding_json(phase: &ExecutionPhaseBinding) -> String {
    let fields = vec![
        json_string_field("phase", &phase.phase),
        json_string_field("role", &runtime_role_json_value(phase.role)),
        json_string_field("bridge_surface", &phase.bridge_surface),
        json_string_field("scheduler_binding", &phase.scheduler_binding),
        json_string_field("lowering_summary", &phase.lowering_summary),
        json_string_field("backend_summary", &phase.backend_summary),
        json_string_field("bridge_summary", &phase.bridge_summary),
        json_optional_string_field("ir_sidecar_summary", phase.ir_sidecar_summary.as_deref()),
        format!(
            "\"clock_gate\":{}",
            execution_clock_gate_json(&phase.clock_gate)
        ),
        format!("\"action\":{}", execution_phase_action_json(&phase.action)),
    ];
    format!("{{{}}}", fields.join(","))
}

fn execution_trace_event_json(event: &ExecutionTraceEvent) -> String {
    let fields = vec![
        json_string_field("phase", &event.phase),
        json_string_field("role", &runtime_role_json_value(event.role)),
        json_string_field("adapter_id", &event.adapter_id),
        json_string_field("bridge_surface", &event.bridge_surface),
        json_string_field("scheduler_binding", &event.scheduler_binding),
        format!(
            "\"clock_gate\":{}",
            execution_clock_gate_json(&event.clock_gate)
        ),
        format!("\"action\":{}", execution_phase_action_json(&event.action)),
        format!(
            "\"outcome\":{}",
            execution_phase_outcome_json(&event.outcome)
        ),
        format!(
            "\"state_before\":{}",
            execution_state_snapshot_json(&event.state_before)
        ),
        format!(
            "\"state_after\":{}",
            execution_state_snapshot_json(&event.state_after)
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn execution_inspect_issue_json(issue: &ExecutionInspectIssue) -> String {
    let fields = vec![
        json_string_field("domain_family", &issue.domain_family),
        json_string_field("issue", &issue.issue),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn inspect_execution_json(input: &Path) -> Result<String, String> {
    let artifact = load_nuis_compiled_artifact(input)?;
    let sections = inspect_execution_sections(input)?;
    let clock_summary = inspect_clock_protocol_summary(input)?;
    let clock_validation = clock_validation_summary(&sections);
    let overview = execution_overview_from_sections(&sections);
    let all_issues = execution_inspect_issues(&overview);
    let section_json = sections
        .iter()
        .map(|section| {
            let section_issues = all_issues
                .iter()
                .filter(|issue| issue.domain_family == section.domain_family)
                .map(execution_inspect_issue_json)
                .collect::<Vec<_>>()
                .join(",");
            let phases = section
                .plan
                .phases
                .iter()
                .map(execution_phase_binding_json)
                .collect::<Vec<_>>()
                .join(",");
            let events = section
                .trace
                .events
                .iter()
                .map(execution_trace_event_json)
                .collect::<Vec<_>>()
                .join(",");
            let fields = vec![
                json_string_field("domain_family", &section.domain_family),
                json_usize_field("plan_phase_count", section.plan.phases.len()),
                json_usize_field("trace_phase_count", section.trace.events.len()),
                format!(
                    "\"backend_family\":{}",
                    match section.plan.backend_family.as_deref() {
                        Some(value) => format!("\"{}\"", json_escape(value)),
                        None => "null".to_owned(),
                    }
                ),
                format!(
                    "\"selected_lowering_target\":{}",
                    match section.plan.selected_lowering_target.as_deref() {
                        Some(value) => format!("\"{}\"", json_escape(value)),
                        None => "null".to_owned(),
                    }
                ),
                format!(
                    "\"clock_validation\":{}",
                    execution_clock_validation_json(
                        section.clock_validation.as_ref(),
                        section.clock_validation_error.as_deref()
                    )
                ),
                format!("\"phases\":[{}]", phases),
                format!("\"events\":[{}]", events),
                format!("\"issues\":[{}]", section_issues),
                json_string_field("plan_summary", &section.plan.render_summary()),
                json_string_field("trace_summary", &section.trace.render_summary()),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let issues_json = all_issues
        .iter()
        .map(execution_inspect_issue_json)
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("kind", "nuis_execution_inspect"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("packaging_mode", &artifact.packaging_mode),
        json_string_field("binary_name", &artifact.binary_name),
        json_string_array_field("domain_families", &artifact.envelope.domain_families),
        json_usize_field("heterogeneous_execution_domains", sections.len()),
        clock_protocol_summary_json(clock_summary.as_ref()),
        inspect_clock_validation_summary_json(&clock_validation),
        format!("\"issues\":[{}]", issues_json),
        format!("\"sections\":[{}]", section_json),
    ];
    Ok(format!("{{{}}}", fields.join(",")))
}

fn clock_protocol_summary_json(summary: Option<&ClockProtocolRuntimeSummary>) -> String {
    let Some(summary) = summary else {
        return "\"clock_protocol\":null".to_owned();
    };
    let fields = vec![
        json_string_field("schema", &summary.schema),
        json_string_field("mode", &summary.mode),
        json_usize_field("domains", summary.domains),
        json_usize_field("edges", summary.edges),
        json_usize_field("happens_before_edges", summary.happens_before_edges),
        json_bool_field("validation_valid", summary.validation_valid),
    ];
    format!("\"clock_protocol\":{{{}}}", fields.join(","))
}
