use std::collections::BTreeSet;
use std::path::Path;

use nuis_artifact::BuildManifestDomainBuildUnit;
use nuis_runtime::{
    AdapterRegistry, BridgeExecutor, DomainAdapter, ExecutionPhaseAction, ExecutionPhaseBinding,
    ExecutionPhaseContext, ExecutionPhaseOutcome, ExecutionPlan, ExecutionResourceBinding,
    ExecutionStateSnapshot, ExecutionTrace, ExecutionTraceEvent, Executor, RuntimeLoader,
    RuntimeRole,
};

use crate::execution_inspect::{
    execution_inspect_issues, ExecutionInspectDomainOverview, ExecutionInspectIssue,
    ExecutionInspectOverview,
};
use crate::{
    json_escape, json_optional_string_field, json_string_array_field, json_string_field,
    json_usize_field, load_nuis_compiled_artifact,
};

struct InspectExecutionAdapter;

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

fn inspect_execution_sections(
    input: &Path,
) -> Result<Vec<(String, ExecutionPlan, ExecutionTrace)>, String> {
    let artifact = load_nuis_compiled_artifact(input)?;
    let loaded = RuntimeLoader
        .load_from_compiled_artifact(artifact)
        .map_err(|error| error.to_string())?;
    let mut adapters = AdapterRegistry::new();
    adapters.register(Box::new(InspectExecutionAdapter));

    let bridge = BridgeExecutor;
    let executor = Executor;
    let mut sections = Vec::new();

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
        sections.push((unit.domain_family.clone(), plan, trace));
    }

    Ok(sections)
}

fn execution_overview_from_sections(
    sections: &[(String, ExecutionPlan, ExecutionTrace)],
) -> ExecutionInspectOverview {
    let domains = sections
        .iter()
        .map(|(domain_family, plan, trace)| {
            let mut resource_keys = BTreeSet::new();
            let mut output_handles = BTreeSet::new();
            for phase in &plan.phases {
                for binding in &phase.action.resource_bindings {
                    resource_keys.insert(binding.key.clone());
                }
                for output in &phase.action.output_handles {
                    output_handles.insert(output.clone());
                }
            }
            ExecutionInspectDomainOverview {
                domain_family: domain_family.clone(),
                selected_lowering_target: plan.selected_lowering_target.clone(),
                phase_count: plan.phases.len(),
                event_count: trace.events.len(),
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

    if sections.is_empty() {
        lines.push("  execution_plan: <no heterogeneous domains available>".to_owned());
        return Ok(lines.join("\n"));
    }

    for (domain_family, plan, trace) in sections {
        lines.push(format!("  domain: {domain_family}"));
        for line in plan.render_summary().lines() {
            lines.push(format!("    plan: {line}"));
        }
        for line in trace.render_summary().lines() {
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
    let overview = execution_overview_from_sections(&sections);
    let all_issues = execution_inspect_issues(&overview);
    let section_json = sections
        .iter()
        .map(|(domain_family, plan, trace)| {
            let section_issues = all_issues
                .iter()
                .filter(|issue| issue.domain_family == *domain_family)
                .map(execution_inspect_issue_json)
                .collect::<Vec<_>>()
                .join(",");
            let phases = plan
                .phases
                .iter()
                .map(execution_phase_binding_json)
                .collect::<Vec<_>>()
                .join(",");
            let events = trace
                .events
                .iter()
                .map(execution_trace_event_json)
                .collect::<Vec<_>>()
                .join(",");
            let fields = vec![
                json_string_field("domain_family", domain_family),
                json_usize_field("plan_phase_count", plan.phases.len()),
                json_usize_field("trace_phase_count", trace.events.len()),
                format!(
                    "\"backend_family\":{}",
                    match plan.backend_family.as_deref() {
                        Some(value) => format!("\"{}\"", json_escape(value)),
                        None => "null".to_owned(),
                    }
                ),
                format!(
                    "\"selected_lowering_target\":{}",
                    match plan.selected_lowering_target.as_deref() {
                        Some(value) => format!("\"{}\"", json_escape(value)),
                        None => "null".to_owned(),
                    }
                ),
                format!("\"phases\":[{}]", phases),
                format!("\"events\":[{}]", events),
                format!("\"issues\":[{}]", section_issues),
                json_string_field("plan_summary", &plan.render_summary()),
                json_string_field("trace_summary", &trace.render_summary()),
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
        format!("\"issues\":[{}]", issues_json),
        format!("\"sections\":[{}]", section_json),
    ];
    Ok(format!("{{{}}}", fields.join(",")))
}
