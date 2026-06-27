pub mod aot;
pub mod cache;
pub mod cli;
pub mod codegen_wasm;
pub mod data_markers;
pub mod engine;
pub mod errors;
pub mod fmt;
pub mod frontend;
pub mod linker;
pub mod lowering;
pub mod nir_verify;
pub mod nustar_binary;
pub mod optimize;
pub mod pipeline;
pub mod project;
pub mod registry;
pub mod render;
pub mod shader_source;
pub mod stdlib_registry;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use nuis_artifact::BuildManifestDomainBuildUnit;
use nuis_runtime::{
    AdapterRegistry, BridgeExecutor, DomainAdapter, ExecutionPhaseAction, ExecutionPhaseBinding,
    ExecutionPhaseContext, ExecutionPhaseOutcome, ExecutionPlan, ExecutionResourceBinding,
    ExecutionStateSnapshot, ExecutionTrace, ExecutionTraceEvent, Executor, RuntimeLoader,
    RuntimeRole,
};

pub use cli::CommandKind;

const NUSTAR_REGISTRY_ROOT: &str = "nustar-packages";

struct CompiledCommandInput {
    resolved: pipeline::ResolvedCompileInput,
    artifacts: pipeline::PipelineArtifacts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkInventoryEntry {
    symbol: String,
    label: String,
    is_async: bool,
    return_type: String,
    warmup_iters: Option<i64>,
    measure_iters: Option<i64>,
    timeout_ms: Option<i64>,
    clock_domain: Option<String>,
    clock_policy: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocIndexModuleSummary {
    module_path: String,
    item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GalaxyDocModuleSummary {
    library_module: String,
    module_path: String,
    documented_item_count: usize,
    doc_index: frontend::AstDocIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GalaxyDocSummary {
    galaxy: String,
    package_id: String,
    library_module_count: usize,
    documented_library_module_count: usize,
    documented_item_count: usize,
    modules: Vec<GalaxyDocModuleSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StdlibDocSummary {
    galaxy_count: usize,
    documented_galaxy_count: usize,
    documented_item_count: usize,
    galaxies: Vec<GalaxyDocSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectMetadataSummary {
    source_kind: String,
    project_name: Option<String>,
    project_root: Option<String>,
    manifest_path: Option<String>,
    build_manifest_path: Option<String>,
    artifact_path: Option<String>,
    docs_index_path: Option<String>,
    docs_module_count: usize,
    docs_documented_module_count: usize,
    docs_documented_item_count: usize,
    imports_index_path: Option<String>,
    imports_library_count: usize,
    imports_visible_library_count: usize,
    imports_visible_module_count: usize,
    imports_documented_visible_module_count: usize,
    imports_documented_visible_item_count: usize,
    galaxy_index_path: Option<String>,
    galaxy_count: usize,
    documented_galaxy_count: usize,
    documented_galaxy_library_module_count: usize,
    documented_galaxy_item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DomainBuildContractDriftCheck {
    package_id: String,
    domain_family: String,
    consistent: bool,
    issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DomainBuildUnitVerificationVerdict {
    package_id: String,
    domain_family: String,
    kind: String,
    payload_blob_ok: bool,
    lowering_plan_ok: bool,
    backend_stub_ok: bool,
    bridge_plan_ok: bool,
    bridge_stub_ok: bool,
    bridge_registry_ok: bool,
    host_bridge_plan_ok: bool,
    registry_alignment_ok: bool,
    failure_reasons: Vec<String>,
    consistent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DomainBuildVerificationSummary {
    all_units_consistent: bool,
    total_units: usize,
    host_units_checked: usize,
    hetero_units_checked: usize,
    registry_drift_units: usize,
    failing_units: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutionInspectDomainOverview {
    domain_family: String,
    selected_lowering_target: Option<String>,
    phase_count: usize,
    event_count: usize,
    resource_keys: Vec<String>,
    output_handles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutionInspectOverview {
    heterogeneous_domains: usize,
    domains: Vec<ExecutionInspectDomainOverview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutionInspectIssue {
    domain_family: String,
    issue: String,
}

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

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", name, value)
}

fn json_i64_field(name: &str, value: i64) -> String {
    format!("\"{}\":{}", name, value)
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

fn json_optional_i64_field(name: &str, value: Option<i64>) -> String {
    match value {
        Some(value) => json_i64_field(name, value),
        None => format!("\"{}\":null", name),
    }
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(name, value),
        None => format!("\"{}\":null", name),
    }
}

fn success_logs_enabled() -> bool {
    std::env::var_os("NUIS_TEST_QUIET_SUCCESS_LOGS").is_none()
}

fn domain_build_contract_summary_json(
    summary: &registry::NustarDomainBuildContractSummary,
) -> String {
    let lowering_fields = vec![
        json_string_field("lane_policy", &summary.lowering.lane_policy),
        json_string_field("bridge_surface", &summary.lowering.bridge_surface),
        json_string_field("emission_kind", &summary.lowering.emission_kind),
    ];
    let backend_fields = vec![
        json_string_field("stub_kind", &summary.backend.stub_kind),
        json_string_field("bridge_entry", &summary.backend.bridge_entry),
        json_string_field("submission_mode", &summary.backend.submission_mode),
        json_string_field("wake_policy", &summary.backend.wake_policy),
        json_string_field("scheduler_binding", &summary.backend.scheduler_binding),
        json_optional_string_field("phase_bind", summary.backend.phase_bind.as_deref()),
        json_optional_string_field("phase_submit", summary.backend.phase_submit.as_deref()),
        json_optional_string_field("phase_wait", summary.backend.phase_wait.as_deref()),
        json_optional_string_field("phase_finalize", summary.backend.phase_finalize.as_deref()),
        json_optional_string_field(
            "transport_model",
            summary.backend.transport_model.as_deref(),
        ),
        json_optional_string_field("request_shape", summary.backend.request_shape.as_deref()),
        json_optional_string_field("response_shape", summary.backend.response_shape.as_deref()),
        json_optional_string_field("dispatch_shape", summary.backend.dispatch_shape.as_deref()),
        json_optional_string_field("memory_binding", summary.backend.memory_binding.as_deref()),
        json_optional_string_field(
            "resource_binding",
            summary.backend.resource_binding.as_deref(),
        ),
        json_optional_string_field(
            "completion_model",
            summary.backend.completion_model.as_deref(),
        ),
    ];
    let bridge_fields = vec![
        json_string_field("bridge_surface", &summary.bridge.bridge_surface),
        json_string_field("bridge_entry", &summary.bridge.bridge_entry),
        json_string_field("scheduler_binding", &summary.bridge.scheduler_binding),
        json_string_field("phase_bind", &summary.bridge.phase_bind),
        json_string_field("phase_submit", &summary.bridge.phase_submit),
        json_string_field("phase_wait", &summary.bridge.phase_wait),
        json_string_field("phase_finalize", &summary.bridge.phase_finalize),
        json_string_field("bridge_kind", &summary.bridge.bridge_kind),
    ];
    let host_bridge_fields = vec![
        json_string_field("host_ffi_surface", &summary.host_bridge.host_ffi_surface),
        json_string_field("handle_family", &summary.host_bridge.handle_family),
        json_string_array_field("phase_order", &summary.host_bridge.phase_order),
        json_string_array_field("phase_bind_inputs", &summary.host_bridge.phase_bind_inputs),
        json_string_array_field(
            "phase_bind_outputs",
            &summary.host_bridge.phase_bind_outputs,
        ),
        json_string_array_field(
            "phase_submit_inputs",
            &summary.host_bridge.phase_submit_inputs,
        ),
        json_string_array_field(
            "phase_submit_outputs",
            &summary.host_bridge.phase_submit_outputs,
        ),
        json_string_array_field("phase_wait_inputs", &summary.host_bridge.phase_wait_inputs),
        json_string_array_field(
            "phase_wait_outputs",
            &summary.host_bridge.phase_wait_outputs,
        ),
        json_string_array_field(
            "phase_finalize_inputs",
            &summary.host_bridge.phase_finalize_inputs,
        ),
        json_string_array_field(
            "phase_finalize_outputs",
            &summary.host_bridge.phase_finalize_outputs,
        ),
        json_string_field("phase_bind_wake", &summary.host_bridge.phase_bind_wake),
        json_string_field("phase_submit_wake", &summary.host_bridge.phase_submit_wake),
        json_string_field("phase_wait_wake", &summary.host_bridge.phase_wait_wake),
        json_string_field(
            "phase_finalize_wake",
            &summary.host_bridge.phase_finalize_wake,
        ),
        json_bool_field("bridge_plan_begin", summary.host_bridge.bridge_plan_begin),
        json_bool_field("bridge_plan_end", summary.host_bridge.bridge_plan_end),
    ];
    format!(
        "{{\"lowering\":{{{}}},\"backend\":{{{}}},\"bridge\":{{{}}},\"host_bridge\":{{{}}}}}",
        lowering_fields.join(","),
        backend_fields.join(","),
        bridge_fields.join(","),
        host_bridge_fields.join(","),
    )
}

fn domain_registry_json(
    registration: &registry::NustarDomainRegistration,
    manifest: &registry::NustarPackageManifest,
) -> String {
    let mut fields = registry::domain_registration_json(registration);
    fields.pop();
    fields.push_str(&format!(
        ",\"build_contract\":{}",
        domain_build_contract_summary_json(&registry::domain_build_contract_summary(manifest))
    ));
    fields.push('}');
    fields
}

fn domain_build_unit_json(unit: &aot::BuildManifestDomainBuildUnit) -> String {
    let fields = vec![
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_optional_string_field("abi", unit.abi.as_deref()),
        json_optional_string_field("machine_arch", unit.machine_arch.as_deref()),
        json_optional_string_field("machine_os", unit.machine_os.as_deref()),
        json_optional_string_field("backend_family", unit.backend_family.as_deref()),
        json_optional_string_field(
            "selected_lowering_target",
            unit.selected_lowering_target.as_deref(),
        ),
        json_optional_string_field("artifact_stub_path", unit.artifact_stub_path.as_deref()),
        json_optional_string_field(
            "artifact_payload_path",
            unit.artifact_payload_path.as_deref(),
        ),
        json_optional_string_field(
            "artifact_bridge_stub_path",
            unit.artifact_bridge_stub_path.as_deref(),
        ),
        json_optional_string_field(
            "artifact_payload_blob_path",
            unit.artifact_payload_blob_path.as_deref(),
        ),
        match unit.artifact_payload_blob_bytes {
            Some(value) => json_usize_field("artifact_payload_blob_bytes", value),
            None => "\"artifact_payload_blob_bytes\":null".to_owned(),
        },
        json_optional_string_field(
            "artifact_payload_format",
            unit.artifact_payload_format.as_deref(),
        ),
        json_string_field("contract_family", &unit.contract_family),
        json_string_field("packaging_role", &unit.packaging_role),
    ];
    format!("{{{}}}", fields.join(","))
}

fn domain_build_unit_effective_contract_summary(
    unit: &aot::BuildManifestDomainBuildUnit,
) -> registry::NustarDomainBuildContractSummary {
    load_manifest_for_build_unit(unit)
        .map(|manifest| registry::domain_build_contract_summary(&manifest))
        .unwrap_or_else(|_| registry::domain_build_contract_summary_for_domain(&unit.domain_family))
}

fn load_manifest_for_build_unit(
    unit: &aot::BuildManifestDomainBuildUnit,
) -> Result<registry::NustarPackageManifest, String> {
    registry::load_manifest(Path::new(NUSTAR_REGISTRY_ROOT), &unit.package_id).or_else(|error| {
        registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &unit.domain_family)
            .map_err(|_| error)
    })
}

fn domain_build_unit_contract_json(unit: &aot::BuildManifestDomainBuildUnit) -> String {
    let fields = vec![
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_optional_string_field("abi", unit.abi.as_deref()),
        json_optional_string_field(
            "selected_lowering_target",
            unit.selected_lowering_target.as_deref(),
        ),
        format!(
            "\"build_contract\":{}",
            domain_build_contract_summary_json(&domain_build_unit_effective_contract_summary(unit))
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn domain_build_unit_contracts_json(units: &[aot::BuildManifestDomainBuildUnit]) -> String {
    units
        .iter()
        .map(domain_build_unit_contract_json)
        .collect::<Vec<_>>()
        .join(",")
}

fn evaluate_domain_build_contract_drift(
    unit: &aot::BuildManifestDomainBuildUnit,
) -> DomainBuildContractDriftCheck {
    let mut issues = Vec::new();
    match load_manifest_for_build_unit(unit) {
        Ok(manifest) => {
            if manifest.domain_family != unit.domain_family {
                issues.push(format!(
                    "registry domain_family={} but build unit recorded {}",
                    manifest.domain_family, unit.domain_family
                ));
            }
            let execution = registry::execution_summary(&manifest);
            if execution.contract_family != unit.contract_family {
                issues.push(format!(
                    "registry contract_family={} but build unit recorded {}",
                    execution.contract_family, unit.contract_family
                ));
            }
            if let Some(target) = unit.selected_lowering_target.as_deref() {
                if !manifest.lowering_targets.iter().any(|item| item == target) {
                    issues.push(format!(
                        "selected_lowering_target={} is not registered in lowering_targets",
                        target
                    ));
                }
            }
            if let Some(backend_family) = unit.backend_family.as_deref() {
                if !manifest.lowering_targets.is_empty()
                    && !manifest
                        .lowering_targets
                        .iter()
                        .any(|item| item == backend_family)
                {
                    issues.push(format!(
                        "backend_family={} is not registered in lowering_targets",
                        backend_family
                    ));
                }
            }
            if let (Some(backend_family), Some(target)) = (
                unit.backend_family.as_deref(),
                unit.selected_lowering_target.as_deref(),
            ) {
                if backend_family != target && !target.starts_with(&format!("{backend_family}.")) {
                    issues.push(format!(
                        "backend_family={} diverges from selected_lowering_target={}",
                        backend_family, target
                    ));
                }
            }
        }
        Err(error) => issues.push(format!(
            "failed to load current registry manifest for {}: {}",
            unit.package_id, error
        )),
    }
    DomainBuildContractDriftCheck {
        package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        consistent: issues.is_empty(),
        issues,
    }
}

fn domain_build_contract_drift_json(check: &DomainBuildContractDriftCheck) -> String {
    let fields = vec![
        json_string_field("package_id", &check.package_id),
        json_string_field("domain_family", &check.domain_family),
        json_bool_field("consistent", check.consistent),
        json_string_array_field("issues", &check.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn domain_build_contract_drift_checks(
    units: &[aot::BuildManifestDomainBuildUnit],
) -> Vec<DomainBuildContractDriftCheck> {
    units
        .iter()
        .map(evaluate_domain_build_contract_drift)
        .collect()
}

fn domain_build_unit_verification_verdict(
    unit: &aot::BuildManifestDomainBuildUnit,
    report: &aot::BuildManifestVerifyReport,
) -> DomainBuildUnitVerificationVerdict {
    let is_heterogeneous = unit.domain_family != "cpu";
    let kind = if is_heterogeneous { "hetero" } else { "host" }.to_owned();
    let drift = evaluate_domain_build_contract_drift(unit);
    let mut failure_reasons = Vec::new();
    let payload_blob_ok = if is_heterogeneous {
        unit.artifact_payload_blob_path.is_some() && report.domain_payload_blobs_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !payload_blob_ok {
        failure_reasons.push("payload_blob_missing_or_unverified".to_owned());
    }
    let lowering_plan_ok = if is_heterogeneous {
        unit.artifact_payload_blob_path.is_some()
            && report.domain_payload_lowering_plans_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !lowering_plan_ok {
        failure_reasons.push("lowering_plan_missing_or_unverified".to_owned());
    }
    let backend_stub_ok = if is_heterogeneous {
        unit.artifact_payload_blob_path.is_some() && report.domain_payload_backend_stubs_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !backend_stub_ok {
        failure_reasons.push("backend_stub_missing_or_unverified".to_owned());
    }
    let bridge_plan_ok = if is_heterogeneous {
        unit.artifact_payload_blob_path.is_some() && report.domain_payload_bridge_plans_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !bridge_plan_ok {
        failure_reasons.push("bridge_plan_missing_or_unverified".to_owned());
    }
    let bridge_stub_ok = if is_heterogeneous {
        unit.artifact_bridge_stub_path.is_some() && report.domain_bridge_stubs_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !bridge_stub_ok {
        failure_reasons.push("bridge_stub_missing_or_unverified".to_owned());
    }
    let bridge_registry_ok = if is_heterogeneous {
        report.bridge_registry_checked > 0 && report.bridge_registry_entries_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !bridge_registry_ok {
        failure_reasons.push("bridge_registry_missing_or_unverified".to_owned());
    }
    let host_bridge_plan_ok = if is_heterogeneous {
        report.host_bridge_plan_checked > 0 && report.host_bridge_plan_entries_checked > 0
    } else {
        true
    };
    if is_heterogeneous && !host_bridge_plan_ok {
        failure_reasons.push("host_bridge_plan_missing_or_unverified".to_owned());
    }
    let registry_alignment_ok = drift.consistent;
    if !registry_alignment_ok {
        failure_reasons.push("registry_alignment_drift".to_owned());
    }
    let consistent = payload_blob_ok
        && lowering_plan_ok
        && backend_stub_ok
        && bridge_plan_ok
        && bridge_stub_ok
        && bridge_registry_ok
        && host_bridge_plan_ok
        && registry_alignment_ok;
    DomainBuildUnitVerificationVerdict {
        package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        kind,
        payload_blob_ok,
        lowering_plan_ok,
        backend_stub_ok,
        bridge_plan_ok,
        bridge_stub_ok,
        bridge_registry_ok,
        host_bridge_plan_ok,
        registry_alignment_ok,
        failure_reasons,
        consistent,
    }
}

fn domain_build_unit_verification_verdict_json(
    verdict: &DomainBuildUnitVerificationVerdict,
) -> String {
    let fields = vec![
        json_string_field("package_id", &verdict.package_id),
        json_string_field("domain_family", &verdict.domain_family),
        json_string_field("kind", &verdict.kind),
        json_bool_field("payload_blob_ok", verdict.payload_blob_ok),
        json_bool_field("lowering_plan_ok", verdict.lowering_plan_ok),
        json_bool_field("backend_stub_ok", verdict.backend_stub_ok),
        json_bool_field("bridge_plan_ok", verdict.bridge_plan_ok),
        json_bool_field("bridge_stub_ok", verdict.bridge_stub_ok),
        json_bool_field("bridge_registry_ok", verdict.bridge_registry_ok),
        json_bool_field("host_bridge_plan_ok", verdict.host_bridge_plan_ok),
        json_bool_field("registry_alignment_ok", verdict.registry_alignment_ok),
        json_string_array_field("failure_reasons", &verdict.failure_reasons),
        json_bool_field("consistent", verdict.consistent),
    ];
    format!("{{{}}}", fields.join(","))
}

fn collect_domain_build_unit_verdicts(
    report: &aot::BuildManifestVerifyReport,
) -> Vec<DomainBuildUnitVerificationVerdict> {
    report
        .domain_build_units
        .iter()
        .map(|unit| domain_build_unit_verification_verdict(unit, report))
        .collect()
}

fn summarize_domain_build_verification(
    verdicts: &[DomainBuildUnitVerificationVerdict],
) -> DomainBuildVerificationSummary {
    let total_units = verdicts.len();
    let host_units_checked = verdicts
        .iter()
        .filter(|verdict| verdict.kind == "host")
        .count();
    let hetero_units_checked = verdicts
        .iter()
        .filter(|verdict| verdict.kind == "hetero")
        .count();
    let registry_drift_units = verdicts
        .iter()
        .filter(|verdict| !verdict.registry_alignment_ok)
        .count();
    let failing_units = verdicts
        .iter()
        .filter(|verdict| !verdict.consistent)
        .map(|verdict| format!("{}[{}]", verdict.package_id, verdict.domain_family))
        .collect::<Vec<_>>();
    DomainBuildVerificationSummary {
        all_units_consistent: failing_units.is_empty(),
        total_units,
        host_units_checked,
        hetero_units_checked,
        registry_drift_units,
        failing_units,
    }
}

fn domain_build_verification_summary_json(summary: &DomainBuildVerificationSummary) -> String {
    let fields = vec![
        json_bool_field("all_units_consistent", summary.all_units_consistent),
        json_usize_field("total_units", summary.total_units),
        json_usize_field("host_units_checked", summary.host_units_checked),
        json_usize_field("hetero_units_checked", summary.hetero_units_checked),
        json_usize_field("registry_drift_units", summary.registry_drift_units),
        json_string_array_field("failing_units", &summary.failing_units),
    ];
    format!("{{{}}}", fields.join(","))
}

fn link_plan_domain_unit_json(unit: &linker::LinkPlanDomainUnit) -> String {
    let mut fields = vec![
        json_string_field("kind", &unit.kind),
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_string_field("contract_family", &unit.contract_family),
        json_string_field("packaging_role", &unit.packaging_role),
    ];
    if let Some(value) = unit.abi.as_deref() {
        fields.push(json_string_field("abi", value));
    }
    if let Some(value) = unit.backend_family.as_deref() {
        fields.push(json_string_field("backend_family", value));
    }
    if let Some(value) = unit.vendor.as_deref() {
        fields.push(json_string_field("vendor", value));
    }
    if let Some(value) = unit.device_class.as_deref() {
        fields.push(json_string_field("device_class", value));
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        fields.push(json_string_field("selected_lowering_target", value));
    }
    format!("{{{}}}", fields.join(","))
}

fn link_plan_json(plan: &linker::LinkPlan) -> String {
    let domain_units = plan
        .domain_units
        .iter()
        .map(link_plan_domain_unit_json)
        .collect::<Vec<_>>()
        .join(",");
    let mut fields = vec![
        json_string_field("schema", &plan.schema),
        json_string_field("packaging_mode", &plan.packaging_mode),
        json_string_field("final_stage_kind", &plan.final_stage.kind),
        json_string_field("final_stage_driver", &plan.final_stage.driver),
        json_string_field("final_stage_link_mode", &plan.final_stage.link_mode),
        json_string_field("final_stage_output", &plan.final_stage.output_path),
        json_string_array_field("final_stage_inputs", &plan.final_stage.inputs),
        json_string_array_field("final_stage_notes", &plan.final_stage.notes),
        json_optional_string_field(
            "artifact_container_kind",
            plan.compiled_artifact.container_kind.as_deref(),
        ),
        match plan.compiled_artifact.container_version {
            Some(version) => format!("\"artifact_container_version\":{}", version),
            None => "\"artifact_container_version\":null".to_owned(),
        },
        match plan.compiled_artifact.section_count {
            Some(count) => json_usize_field("artifact_section_count", count),
            None => "\"artifact_section_count\":null".to_owned(),
        },
        json_string_array_field(
            "artifact_section_names",
            &plan.compiled_artifact.section_names,
        ),
        match plan.compiled_artifact.section_table_valid {
            Some(valid) => json_bool_field("artifact_section_table_valid", valid),
            None => "\"artifact_section_table_valid\":null".to_owned(),
        },
        json_usize_field("domain_unit_count", plan.domain_units.len()),
        format!("\"domain_units\":[{}]", domain_units),
    ];
    if let Some(path) = &plan.bridge_registry_path {
        fields.push(json_string_field("bridge_registry_path", path));
    }
    if let Some(path) = &plan.host_bridge_plan_index_path {
        fields.push(json_string_field("host_bridge_plan_index_path", path));
    }
    format!("{{{}}}", fields.join(","))
}

fn artifact_report_summary_lines(
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    verification_summary: &DomainBuildVerificationSummary,
    link_plan: Option<&linker::LinkPlan>,
    manifest_verify_reconstructed: bool,
    execution_overview: Option<&ExecutionInspectOverview>,
    doc_indexes: Option<&[frontend::AstDocIndex]>,
    project_metadata: Option<&ProjectMetadataSummary>,
) -> Vec<String> {
    let mut lines = vec![
        format!(
            "summary: artifact_roundtrip={} lifecycle={} runtime_flags={} all_units_consistent={}",
            if artifact_verify.artifact_roundtrip_verified {
                "ok"
            } else {
                "failed"
            },
            if artifact_verify.lifecycle_contract_consistent {
                "ok"
            } else {
                "failed"
            },
            if artifact_verify.lifecycle_runtime_capability_flags_consistent {
                "ok"
            } else {
                "failed"
            },
            if verification_summary.all_units_consistent {
                "true"
            } else {
                "false"
            }
        ),
        format!(
            "summary_units: total={} host={} hetero={} drift={} failing={}",
            verification_summary.total_units,
            verification_summary.host_units_checked,
            verification_summary.hetero_units_checked,
            verification_summary.registry_drift_units,
            if verification_summary.failing_units.is_empty() {
                "<none>".to_owned()
            } else {
                verification_summary.failing_units.join(", ")
            }
        ),
        format!(
            "summary_manifest: reconstructed={}",
            if manifest_verify_reconstructed {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if let Some(plan) = link_plan {
        lines.push(format!(
            "summary_link: final_stage={} driver={} link_mode={} output={}",
            plan.final_stage.kind,
            plan.final_stage.driver,
            plan.final_stage.link_mode,
            plan.final_stage.output_path
        ));
    }
    if let Some(overview) = execution_overview {
        let issues = execution_inspect_issues(overview);
        lines.push(format!(
            "summary_execution: hetero_domains={} domains={}",
            overview.heterogeneous_domains,
            if overview.domains.is_empty() {
                "<none>".to_owned()
            } else {
                overview
                    .domains
                    .iter()
                    .map(|domain| {
                        let target = domain
                            .selected_lowering_target
                            .as_deref()
                            .unwrap_or("<none>");
                        format!(
                            "{}(target={} phases={} events={})",
                            domain.domain_family, target, domain.phase_count, domain.event_count
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
        lines.push(format!(
            "summary_execution_issues: {}",
            if issues.is_empty() {
                "<none>".to_owned()
            } else {
                issues
                    .iter()
                    .map(|issue| format!("{}:{}", issue.domain_family, issue.issue))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    }
    if let Some(indexes) = doc_indexes {
        let module_count = indexes.len();
        let item_count = indexes.iter().map(|index| index.items.len()).sum::<usize>();
        lines.push(format!(
            "summary_docs: modules={} documented_items={} documented_modules={}",
            module_count,
            item_count,
            if indexes.is_empty() {
                "<none>".to_owned()
            } else {
                indexes
                    .iter()
                    .map(|index| index.module_path.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    }
    if let Some(project) = project_metadata {
        lines.push(format!(
            "summary_project: docs={}/{}/{} imports={}/{}/{}/{}/{} galaxies={}/{}/{}/{}",
            project.docs_module_count,
            project.docs_documented_module_count,
            project.docs_documented_item_count,
            project.imports_library_count,
            project.imports_visible_library_count,
            project.imports_visible_module_count,
            project.imports_documented_visible_module_count,
            project.imports_documented_visible_item_count,
            project.galaxy_count,
            project.documented_galaxy_count,
            project.documented_galaxy_library_module_count,
            project.documented_galaxy_item_count
        ));
    }
    lines
}

fn execution_inspect_issues(overview: &ExecutionInspectOverview) -> Vec<ExecutionInspectIssue> {
    let mut issues = Vec::new();
    for domain in &overview.domains {
        if domain.selected_lowering_target.is_none() {
            issues.push(ExecutionInspectIssue {
                domain_family: domain.domain_family.clone(),
                issue: "missing_target".to_owned(),
            });
        }
        if domain.phase_count == 0 {
            issues.push(ExecutionInspectIssue {
                domain_family: domain.domain_family.clone(),
                issue: "zero_phases".to_owned(),
            });
        }
        if domain.phase_count != domain.event_count {
            issues.push(ExecutionInspectIssue {
                domain_family: domain.domain_family.clone(),
                issue: format!(
                    "phase_event_mismatch({}->{})",
                    domain.phase_count, domain.event_count
                ),
            });
        }
        let has_resource = |key: &str| domain.resource_keys.iter().any(|item| item == key);
        let has_output = |key: &str| domain.output_handles.iter().any(|item| item == key);
        match domain.domain_family.as_str() {
            "network" => {
                if !has_resource("request_packet") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_network_request_packet".to_owned(),
                    });
                }
                if !has_resource("active_response") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_network_active_response".to_owned(),
                    });
                }
                if !has_output("response.handle") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_network_response_handle".to_owned(),
                    });
                }
            }
            "shader" => {
                if !has_resource("shader_buffer") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_shader_buffer".to_owned(),
                    });
                }
                if !has_resource("frame_target") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_shader_frame_target".to_owned(),
                    });
                }
                if !has_output("draw.handle") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_shader_draw_handle".to_owned(),
                    });
                }
            }
            "kernel" => {
                if !has_resource("kernel_buffer") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_kernel_buffer".to_owned(),
                    });
                }
                if !has_resource("dispatch_handle") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_kernel_dispatch_handle".to_owned(),
                    });
                }
                if !has_resource("result_buffer") {
                    issues.push(ExecutionInspectIssue {
                        domain_family: domain.domain_family.clone(),
                        issue: "missing_kernel_result_buffer".to_owned(),
                    });
                }
            }
            _ => {}
        }
    }
    issues
}

fn verdict_status(ok: bool, hetero_expected: bool) -> &'static str {
    if !hetero_expected {
        "skipped"
    } else if ok {
        "ok"
    } else {
        "missing"
    }
}

fn collect_benchmark_inventory(
    artifacts: &pipeline::PipelineArtifacts,
) -> Vec<BenchmarkInventoryEntry> {
    frontend::collect_nir_benchmarks(&artifacts.nir)
        .into_iter()
        .map(|function| BenchmarkInventoryEntry {
            symbol: format!(
                "{}::{}::{}",
                artifacts.nir.domain, artifacts.nir.unit, function.name
            ),
            label: function
                .benchmark_name
                .clone()
                .unwrap_or_else(|| function.name.clone()),
            is_async: function.is_async,
            return_type: function
                .return_type
                .as_ref()
                .map(|ty| ty.render())
                .unwrap_or_else(|| "()".to_owned()),
            warmup_iters: function.benchmark_warmup_iters,
            measure_iters: function.benchmark_measure_iters,
            timeout_ms: function.benchmark_timeout_ms,
            clock_domain: function
                .benchmark_clock_domain
                .map(|domain| domain.as_str().to_owned()),
            clock_policy: function
                .benchmark_clock_policy
                .map(|policy| policy.as_str().to_owned()),
        })
        .collect()
}

fn inspect_benchmarks_json(input: &Path, artifacts: &pipeline::PipelineArtifacts) -> String {
    let benchmarks = collect_benchmark_inventory(artifacts);
    let entries = benchmarks
        .iter()
        .map(|entry| {
            let fields = vec![
                json_string_field("symbol", &entry.symbol),
                json_string_field("label", &entry.label),
                json_bool_field("async", entry.is_async),
                json_string_field("return_type", &entry.return_type),
                json_optional_i64_field("warmup_iters", entry.warmup_iters),
                json_optional_i64_field("measure_iters", entry.measure_iters),
                json_optional_i64_field("timeout_ms", entry.timeout_ms),
                json_optional_string_field("clock_domain", entry.clock_domain.as_deref()),
                json_optional_string_field("clock_policy", entry.clock_policy.as_deref()),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = vec![
        json_string_field("kind", "nuis_benchmark_inventory"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("domain", &artifacts.nir.domain),
        json_string_field("unit", &artifacts.nir.unit),
        json_usize_field("benchmark_count", benchmarks.len()),
        format!("\"benchmarks\":[{}]", entries),
    ];
    format!("{{{}}}", fields.join(","))
}

fn collect_doc_indexes(input: &Path) -> Result<Vec<frontend::AstDocIndex>, String> {
    if project::is_project_input(input) {
        let project = project::load_project(input)?;
        let mut indexes = project
            .modules
            .iter()
            .map(|module| frontend::extract_ast_doc_index(&module.ast))
            .collect::<Vec<_>>();
        indexes.sort_by(|lhs, rhs| lhs.module_path.cmp(&rhs.module_path));
        return Ok(indexes);
    }

    let source = std::fs::read_to_string(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    let ast = frontend::parse_nuis_ast(&source)?;
    Ok(vec![frontend::extract_ast_doc_index(&ast)])
}

fn summarize_doc_indexes(indexes: &[frontend::AstDocIndex]) -> Vec<DocIndexModuleSummary> {
    indexes
        .iter()
        .map(|index| DocIndexModuleSummary {
            module_path: index.module_path.clone(),
            item_count: index.items.len(),
        })
        .collect()
}

fn inspect_docs_json(input: &Path, indexes: &[frontend::AstDocIndex]) -> String {
    let modules = indexes
        .iter()
        .map(|index| {
            let items = index
                .items
                .iter()
                .map(|item| {
                    let fields = vec![
                        json_string_field("kind", &item.kind),
                        json_string_field("path", &item.path),
                        json_string_array_field("docs", &item.docs),
                        json_optional_string_field("signature", item.signature.as_deref()),
                    ];
                    format!("{{{}}}", fields.join(","))
                })
                .collect::<Vec<_>>()
                .join(",");
            let fields = vec![
                json_string_field("module_path", &index.module_path),
                json_usize_field("item_count", index.items.len()),
                format!("\"items\":[{}]", items),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let total_items = indexes.iter().map(|index| index.items.len()).sum::<usize>();
    let fields = vec![
        json_string_field("kind", "nuis_doc_index"),
        json_string_field("input", &input.display().to_string()),
        json_usize_field("module_count", indexes.len()),
        json_usize_field("documented_item_count", total_items),
        format!("\"modules\":[{}]", modules),
    ];
    format!("{{{}}}", fields.join(","))
}

fn write_json_output(path: &Path, payload: &str) -> Result<(), String> {
    std::fs::write(path, payload)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))
}

fn inspect_galaxy_doc_summary(galaxy: &str) -> Result<GalaxyDocSummary, String> {
    let stdlib_root = stdlib_registry::resolve_stdlib_root()?;
    let manifest = stdlib_registry::load_stdlib_module_manifest(&stdlib_root, galaxy)?;
    let module_root = stdlib_root.join(galaxy);
    let mut modules = Vec::new();

    for library_module in &manifest.library_modules {
        let path = module_root.join(library_module);
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let ast = frontend::parse_nuis_ast(&source)?;
        let doc_index = frontend::extract_ast_doc_index(&ast);
        let documented_item_count = doc_index.items.len();
        modules.push(GalaxyDocModuleSummary {
            library_module: library_module.clone(),
            module_path: doc_index.module_path.clone(),
            documented_item_count,
            doc_index,
        });
    }

    let documented_library_module_count = modules
        .iter()
        .filter(|module| module.documented_item_count > 0)
        .count();
    let documented_item_count = modules
        .iter()
        .map(|module| module.documented_item_count)
        .sum::<usize>();

    Ok(GalaxyDocSummary {
        galaxy: manifest.name,
        package_id: manifest.package_id,
        library_module_count: modules.len(),
        documented_library_module_count,
        documented_item_count,
        modules,
    })
}

fn inspect_galaxy_docs_json(summary: &GalaxyDocSummary) -> String {
    let modules = summary
        .modules
        .iter()
        .map(|module| {
            let items = module
                .doc_index
                .items
                .iter()
                .map(|item| {
                    format!(
                        "{{{},{},{},{}}}",
                        json_string_field("kind", &item.kind),
                        json_string_field("path", &item.path),
                        json_string_array_field("docs", &item.docs),
                        json_optional_string_field("signature", item.signature.as_deref()),
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{{},{},{},{}}}",
                json_string_field("library_module", &module.library_module),
                json_string_field("module_path", &module.module_path),
                json_usize_field("documented_item_count", module.documented_item_count),
                format!("\"items\":[{}]", items)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{},{},{},{},{},{}}}",
        json_string_field("kind", "nuis_galaxy_doc_index"),
        json_string_field("galaxy", &summary.galaxy),
        json_string_field("package_id", &summary.package_id),
        json_usize_field("library_module_count", summary.library_module_count),
        json_usize_field(
            "documented_library_module_count",
            summary.documented_library_module_count
        ),
        format!(
            "{},\"modules\":[{}]",
            json_usize_field("documented_item_count", summary.documented_item_count),
            modules
        )
    )
}

fn inspect_stdlib_doc_summary() -> Result<StdlibDocSummary, String> {
    let stdlib_root = stdlib_registry::resolve_stdlib_root()?;
    let layout = stdlib_registry::load_stdlib_layout(&stdlib_root)?;
    let mut galaxies = Vec::new();
    for module in layout.modules {
        galaxies.push(inspect_galaxy_doc_summary(&module.name)?);
    }
    let documented_galaxy_count = galaxies
        .iter()
        .filter(|galaxy| galaxy.documented_item_count > 0)
        .count();
    let documented_item_count = galaxies
        .iter()
        .map(|galaxy| galaxy.documented_item_count)
        .sum::<usize>();
    Ok(StdlibDocSummary {
        galaxy_count: galaxies.len(),
        documented_galaxy_count,
        documented_item_count,
        galaxies,
    })
}

fn inspect_stdlib_docs_json(summary: &StdlibDocSummary) -> String {
    let galaxies = summary
        .galaxies
        .iter()
        .map(|galaxy| {
            format!(
                "{{{},{},{},{},{}}}",
                json_string_field("galaxy", &galaxy.galaxy),
                json_string_field("package_id", &galaxy.package_id),
                json_usize_field("library_module_count", galaxy.library_module_count),
                json_usize_field(
                    "documented_library_module_count",
                    galaxy.documented_library_module_count
                ),
                json_usize_field("documented_item_count", galaxy.documented_item_count)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{},{},{},{},{}}}",
        json_string_field("kind", "nuis_stdlib_doc_index"),
        json_usize_field("galaxy_count", summary.galaxy_count),
        json_usize_field("documented_galaxy_count", summary.documented_galaxy_count),
        json_usize_field("documented_item_count", summary.documented_item_count),
        format!("\"galaxies\":[{}]", galaxies)
    )
}

fn project_metadata_summary_from_manifest_report(
    source_kind: &str,
    manifest_path: Option<&Path>,
    artifact_path: Option<&Path>,
    report: &aot::BuildManifestVerifyReport,
) -> ProjectMetadataSummary {
    ProjectMetadataSummary {
        source_kind: source_kind.to_owned(),
        project_name: None,
        project_root: Path::new(&report.input)
            .parent()
            .map(|path| path.display().to_string()),
        manifest_path: manifest_path.map(|path| path.display().to_string()),
        build_manifest_path: manifest_path.map(|path| path.display().to_string()),
        artifact_path: artifact_path
            .map(|path| path.display().to_string())
            .or_else(|| {
                if report.artifact_path.is_empty() {
                    None
                } else {
                    Some(report.artifact_path.clone())
                }
            }),
        docs_index_path: report.project_docs_index.clone(),
        docs_module_count: report.project_docs_module_count,
        docs_documented_module_count: report.project_docs_documented_module_count,
        docs_documented_item_count: report.project_docs_documented_item_count,
        imports_index_path: report.project_imports_index.clone(),
        imports_library_count: report.project_imports_library_count,
        imports_visible_library_count: report.project_imports_visible_library_count,
        imports_visible_module_count: report.project_imports_visible_module_count,
        imports_documented_visible_module_count: report
            .project_imports_documented_visible_module_count,
        imports_documented_visible_item_count: report.project_imports_documented_visible_item_count,
        galaxy_index_path: report.project_galaxy_index.clone(),
        galaxy_count: report.project_galaxy_count,
        documented_galaxy_count: report.project_documented_galaxy_count,
        documented_galaxy_library_module_count: report
            .project_documented_galaxy_library_module_count,
        documented_galaxy_item_count: report.project_documented_galaxy_item_count,
    }
}

fn inspect_project_metadata_from_source(input: &Path) -> Result<ProjectMetadataSummary, String> {
    let project = project::load_project(input)?;
    let docs_summary = project::project_docs_summary(&project);
    let imports_summary = project::project_imports_summary(&project);
    let galaxy_summary = project::project_galaxy_summary(&project);
    Ok(ProjectMetadataSummary {
        source_kind: "project-source".to_owned(),
        project_name: Some(project.manifest.name.clone()),
        project_root: Some(project.root.display().to_string()),
        manifest_path: Some(project.manifest_path.display().to_string()),
        build_manifest_path: None,
        artifact_path: None,
        docs_index_path: None,
        docs_module_count: docs_summary.modules,
        docs_documented_module_count: docs_summary.documented_modules,
        docs_documented_item_count: docs_summary.documented_items,
        imports_index_path: None,
        imports_library_count: imports_summary.libraries,
        imports_visible_library_count: imports_summary.visible_libraries,
        imports_visible_module_count: imports_summary.visible_modules,
        imports_documented_visible_module_count: imports_summary.documented_visible_modules,
        imports_documented_visible_item_count: imports_summary.documented_visible_items,
        galaxy_index_path: None,
        galaxy_count: galaxy_summary.galaxies,
        documented_galaxy_count: galaxy_summary.documented_galaxies,
        documented_galaxy_library_module_count: galaxy_summary.documented_library_modules,
        documented_galaxy_item_count: galaxy_summary.documented_items,
    })
}

fn inspect_project_metadata(input: &Path) -> Result<ProjectMetadataSummary, String> {
    if input.is_dir() {
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            let report = aot::verify_build_manifest(&manifest_path)?;
            let artifact_path = input.join("nuis.compiled.artifact");
            return Ok(project_metadata_summary_from_manifest_report(
                "build-output-dir",
                Some(&manifest_path),
                artifact_path.is_file().then_some(artifact_path.as_path()),
                &report,
            ));
        }
    }
    let is_manifest = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let is_artifact = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.compiled.artifact")
        .unwrap_or(false);
    if is_manifest {
        let report = aot::verify_build_manifest(input)?;
        return Ok(project_metadata_summary_from_manifest_report(
            "build-manifest",
            Some(input),
            None,
            &report,
        ));
    }
    if is_artifact {
        let artifact = load_nuis_compiled_artifact(input)?;
        let (manifest_path, report) = reconstruct_manifest_report_from_artifact(input, &artifact)?;
        return Ok(project_metadata_summary_from_manifest_report(
            "compiled-artifact",
            Some(&manifest_path),
            Some(input),
            &report,
        ));
    }
    inspect_project_metadata_from_source(input)
}

fn resolve_build_manifest_path(input: &Path) -> Result<PathBuf, String> {
    if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        return Ok(input.to_path_buf());
    }
    if input.is_dir() {
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            return Ok(manifest_path);
        }
        return Err(format!(
            "`{}` does not contain `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    Err(format!(
        "expected a build manifest path or output directory, got `{}`",
        input.display()
    ))
}

fn resolve_artifact_report_inputs(
    input: &Path,
) -> Result<
    (
        PathBuf,
        aot::NuisCompiledArtifact,
        PathBuf,
        aot::BuildManifestVerifyReport,
        bool,
    ),
    String,
> {
    let is_manifest_input = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let is_output_dir_input = input.is_dir() && input.join("nuis.build.manifest.toml").is_file();
    let artifact = load_nuis_compiled_artifact(input)?;
    if is_manifest_input || is_output_dir_input {
        let manifest_input = if is_manifest_input {
            input.to_path_buf()
        } else {
            input.join("nuis.build.manifest.toml")
        };
        let report = aot::verify_build_manifest(&manifest_input)?;
        return Ok((
            manifest_input,
            artifact,
            PathBuf::from(&report.artifact_path),
            report,
            false,
        ));
    }
    let (manifest_input, manifest_verify) =
        reconstruct_manifest_report_from_artifact(input, &artifact)?;
    Ok((
        manifest_input,
        artifact,
        input.to_path_buf(),
        manifest_verify,
        true,
    ))
}

fn repair_project_metadata_target(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if input.is_dir() {
        let manifest_path = resolve_build_manifest_path(input)?;
        let report = aot::verify_build_manifest(&manifest_path)?;
        let project_input = PathBuf::from(&report.input);
        if !project_input.exists() {
            return Err(format!(
                "cannot repair project metadata from `{}` because the original compile input `{}` no longer exists; try `nuisc inspect-project-metadata \"{}\"` or `nuisc verify-build-manifest \"{}\"` instead",
                input.display(),
                project_input.display(),
                input.display(),
                input.display()
            ));
        }
        return Ok((project_input, PathBuf::from(report.output_dir)));
    }
    let is_manifest = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let is_artifact = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.compiled.artifact")
        .unwrap_or(false);
    if is_manifest {
        let report = aot::verify_build_manifest(input)?;
        let project_input = PathBuf::from(&report.input);
        if !project_input.exists() {
            return Err(format!(
                "cannot repair project metadata from `{}` because the original compile input `{}` no longer exists; try `nuisc inspect-project-metadata \"{}\"` or `nuisc verify-build-manifest \"{}\"` instead",
                input.display(),
                project_input.display(),
                input.display(),
                input.display()
            ));
        }
        return Ok((project_input, PathBuf::from(report.output_dir)));
    }
    if is_artifact {
        let artifact = load_nuis_compiled_artifact(input)?;
        let (_manifest_path, report) = reconstruct_manifest_report_from_artifact(input, &artifact)?;
        let project_input = PathBuf::from(&report.input);
        if !project_input.exists() {
            return Err(format!(
                "cannot repair project metadata from `{}` because the original compile input `{}` no longer exists; try `nuisc inspect-project-metadata \"{}\"` instead",
                input.display(),
                project_input.display(),
                input.display()
            ));
        }
        return Ok((project_input, PathBuf::from(report.output_dir)));
    }
    Err(
        "usage: nuisc repair-project-metadata [--dry-run] <output-dir|nuis.build.manifest.toml|nuis.compiled.artifact>"
            .to_owned(),
    )
}

fn inspect_project_metadata_json(summary: &ProjectMetadataSummary) -> String {
    let mut fields = vec![
        json_string_field("kind", "nuis_project_metadata"),
        json_string_field("source_kind", &summary.source_kind),
        json_usize_field("docs_module_count", summary.docs_module_count),
        json_usize_field(
            "docs_documented_module_count",
            summary.docs_documented_module_count,
        ),
        json_usize_field(
            "docs_documented_item_count",
            summary.docs_documented_item_count,
        ),
        json_usize_field("imports_library_count", summary.imports_library_count),
        json_usize_field(
            "imports_visible_library_count",
            summary.imports_visible_library_count,
        ),
        json_usize_field(
            "imports_visible_module_count",
            summary.imports_visible_module_count,
        ),
        json_usize_field(
            "imports_documented_visible_module_count",
            summary.imports_documented_visible_module_count,
        ),
        json_usize_field(
            "imports_documented_visible_item_count",
            summary.imports_documented_visible_item_count,
        ),
        json_usize_field("galaxy_count", summary.galaxy_count),
        json_usize_field("documented_galaxy_count", summary.documented_galaxy_count),
        json_usize_field(
            "documented_galaxy_library_module_count",
            summary.documented_galaxy_library_module_count,
        ),
        json_usize_field(
            "documented_galaxy_item_count",
            summary.documented_galaxy_item_count,
        ),
    ];
    if let Some(value) = &summary.project_name {
        fields.push(json_string_field("project_name", value));
    }
    if let Some(value) = &summary.project_root {
        fields.push(json_string_field("project_root", value));
    }
    if let Some(value) = &summary.manifest_path {
        fields.push(json_string_field("manifest_path", value));
    }
    if let Some(value) = &summary.build_manifest_path {
        fields.push(json_string_field("build_manifest_path", value));
    }
    if let Some(value) = &summary.artifact_path {
        fields.push(json_string_field("artifact_path", value));
    }
    if let Some(value) = &summary.docs_index_path {
        fields.push(json_string_field("docs_index_path", value));
    }
    if let Some(value) = &summary.imports_index_path {
        fields.push(json_string_field("imports_index_path", value));
    }
    if let Some(value) = &summary.galaxy_index_path {
        fields.push(json_string_field("galaxy_index_path", value));
    }
    format!("{{{}}}", fields.join(","))
}

fn render_project_metadata_summary(summary: &ProjectMetadataSummary) -> String {
    let mut lines = vec!["project metadata".to_owned()];
    lines.push(format!("  source_kind: {}", summary.source_kind));
    if let Some(value) = &summary.project_name {
        lines.push(format!("  project_name: {}", value));
    }
    if let Some(value) = &summary.project_root {
        lines.push(format!("  project_root: {}", value));
    }
    if let Some(value) = &summary.manifest_path {
        lines.push(format!("  manifest_path: {}", value));
    }
    if let Some(value) = &summary.build_manifest_path {
        lines.push(format!("  build_manifest_path: {}", value));
    }
    if let Some(value) = &summary.artifact_path {
        lines.push(format!("  artifact_path: {}", value));
    }
    lines.push(format!(
        "  docs: modules={} documented_modules={} documented_items={}",
        summary.docs_module_count,
        summary.docs_documented_module_count,
        summary.docs_documented_item_count
    ));
    if let Some(value) = &summary.docs_index_path {
        lines.push(format!("  docs_index_path: {}", value));
    }
    lines.push(format!(
        "  imports: libraries={} visible_libraries={} visible_modules={} documented_visible_modules={} documented_visible_items={}",
        summary.imports_library_count,
        summary.imports_visible_library_count,
        summary.imports_visible_module_count,
        summary.imports_documented_visible_module_count,
        summary.imports_documented_visible_item_count
    ));
    if let Some(value) = &summary.imports_index_path {
        lines.push(format!("  imports_index_path: {}", value));
    }
    lines.push(format!(
        "  galaxies: total={} documented={} documented_library_modules={} documented_items={}",
        summary.galaxy_count,
        summary.documented_galaxy_count,
        summary.documented_galaxy_library_module_count,
        summary.documented_galaxy_item_count
    ));
    if let Some(value) = &summary.galaxy_index_path {
        lines.push(format!("  galaxy_index_path: {}", value));
    }
    lines.join("\n")
}

fn render_project_metadata_compact_summary(summary: &ProjectMetadataSummary) -> String {
    format!(
        "project metadata summary: source_kind={} project={} docs={}/{}/{} imports={}/{}/{}/{}/{} galaxies={}/{}/{}/{}",
        summary.source_kind,
        summary.project_name.as_deref().unwrap_or("<none>"),
        summary.docs_module_count,
        summary.docs_documented_module_count,
        summary.docs_documented_item_count,
        summary.imports_library_count,
        summary.imports_visible_library_count,
        summary.imports_visible_module_count,
        summary.imports_documented_visible_module_count,
        summary.imports_documented_visible_item_count,
        summary.galaxy_count,
        summary.documented_galaxy_count,
        summary.documented_galaxy_library_module_count,
        summary.documented_galaxy_item_count
    )
}

fn render_project_metadata_paths(summary: &ProjectMetadataSummary) -> String {
    let mut lines = Vec::new();
    if let Some(value) = &summary.project_root {
        lines.push(format!("project_root={}", value));
    }
    if let Some(value) = &summary.manifest_path {
        lines.push(format!("manifest_path={}", value));
    }
    if let Some(value) = &summary.build_manifest_path {
        lines.push(format!("build_manifest_path={}", value));
    }
    if let Some(value) = &summary.artifact_path {
        lines.push(format!("artifact_path={}", value));
    }
    if let Some(value) = &summary.docs_index_path {
        lines.push(format!("docs_index_path={}", value));
    }
    if let Some(value) = &summary.imports_index_path {
        lines.push(format!("imports_index_path={}", value));
    }
    if let Some(value) = &summary.galaxy_index_path {
        lines.push(format!("galaxy_index_path={}", value));
    }
    lines.join("\n")
}

fn collect_doc_indexes_from_manifest_input(
    manifest_verify: &aot::BuildManifestVerifyReport,
) -> Result<Vec<frontend::AstDocIndex>, String> {
    collect_doc_indexes(Path::new(&manifest_verify.input))
}

fn write_compile_doc_index(
    input: &Path,
    output_dir: &Path,
) -> Result<aot::BuildManifestDocIndexInfo, String> {
    let indexes = collect_doc_indexes(input)?;
    let payload = inspect_docs_json(input, &indexes);
    let output_path = output_dir.join("nuis.doc-index.json");
    write_json_output(&output_path, &payload)?;
    Ok(aot::BuildManifestDocIndexInfo {
        path: output_path.display().to_string(),
        module_count: indexes.len(),
        documented_item_count: indexes.iter().map(|index| index.items.len()).sum(),
    })
}

pub fn project_compile_workflow_brief() -> &'static str {
    "health -> structure -> scheduler -> abi_lock -> check -> test -> build -> project_metadata_inspect -> artifact_doctor -> metadata_repair -> run_artifact -> release_check"
}

pub fn nuisc_compile_pipeline_brief() -> &'static str {
    "resolve_input -> resolve_cpu_target -> compile_plan -> nir_verify -> project_link_validate -> yir_lower -> project_link_apply -> project_abi_validate -> codegen_prune -> llvm_emit -> aot_link -> project_metadata -> build_manifest -> compiled_artifact"
}

pub fn project_compile_samples_brief() -> &'static str {
    "health=nuis project-doctor <project-dir>; structure=nuis project-status <project-dir>; scheduler=nuis scheduler-view <project-dir>; abi_lock=nuis project-lock-abi <project-dir>; compile=nuis check <project-dir> -> nuis test <project-dir> -> nuis build <project-dir> <output-dir> -> nuisc inspect-project-metadata --summary <output-dir> -> nuis artifact-doctor <output-dir> -> nuisc repair-project-metadata --dry-run <output-dir> -> nuis run-artifact <output-dir> -> nuis release-check <project-dir> <output-dir>"
}

pub fn project_test_workflow_brief() -> &'static str {
    "list=nuis test --list <project-dir>; exact=nuis test --exact <project-dir> <test-name>; ignored=nuis test --ignored <project-dir>; include_ignored=nuis test --include-ignored <project-dir>"
}

pub fn project_galaxy_workflow_brief() -> &'static str {
    "galaxy=nuis galaxy init <project-dir> -> nuis galaxy check <project-dir> -> nuis galaxy lock-deps <project-dir> -> nuis galaxy sync-deps <project-dir> -> nuis project-doctor <project-dir> -> nuisc inspect-project-metadata --summary <project-dir>"
}

fn resolve_compile_input(input: &Path) -> Result<pipeline::ResolvedCompileInput, String> {
    pipeline::resolve_compile_input(input)
}

fn compile_command_input(input: &Path) -> Result<CompiledCommandInput, String> {
    let resolved = resolve_compile_input(input)?;
    let artifacts = resolved.compile()?;
    Ok(CompiledCommandInput {
        resolved,
        artifacts,
    })
}

fn load_nuis_executable_envelope(input: &Path) -> Result<aot::NuisExecutableEnvelope, String> {
    let bytes = std::fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    if bytes.starts_with(b"NENV") {
        aot::decode_nuis_executable_envelope_binary(&bytes)
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        let report = aot::verify_build_manifest(input)?;
        aot::parse_nuis_executable_envelope(Path::new(&report.envelope_path))
    } else {
        aot::parse_nuis_executable_envelope(input)
    }
}

fn load_nuis_compiled_artifact(input: &Path) -> Result<aot::NuisCompiledArtifact, String> {
    if input.is_dir() {
        let artifact_path = input.join("nuis.compiled.artifact");
        if artifact_path.is_file() {
            return aot::parse_nuis_compiled_artifact(&artifact_path);
        }
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            let report = aot::verify_build_manifest(&manifest_path)?;
            return aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path));
        }
        return Err(format!(
            "`{}` does not contain `nuis.compiled.artifact` or `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    let bytes = std::fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    if bytes.starts_with(b"NART") {
        aot::decode_nuis_compiled_artifact_binary(&bytes)
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        let report = aot::verify_build_manifest(input)?;
        aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path))
    } else {
        aot::parse_nuis_compiled_artifact(input)
    }
}

fn inspect_artifact_container_for_input(
    input: &Path,
    manifest_verify: Option<&aot::BuildManifestVerifyReport>,
) -> Result<Option<aot::NuisCompiledArtifactContainerInspect>, String> {
    let artifact_path = if input.is_dir() {
        let direct = input.join("nuis.compiled.artifact");
        if direct.is_file() {
            Some(direct)
        } else {
            manifest_verify.map(|report| PathBuf::from(&report.artifact_path))
        }
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        manifest_verify.map(|report| PathBuf::from(&report.artifact_path))
    } else {
        let bytes = std::fs::read(input)
            .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
        if bytes.starts_with(b"NART") {
            Some(input.to_path_buf())
        } else {
            None
        }
    };
    match artifact_path {
        Some(path) => Ok(Some(aot::inspect_nuis_compiled_artifact_container(&path)?)),
        None => Ok(None),
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

fn inspect_execution_overview(input: &Path) -> Result<ExecutionInspectOverview, String> {
    let sections = inspect_execution_sections(input)?;
    Ok(execution_overview_from_sections(&sections))
}

fn render_execution_report(input: &Path) -> Result<String, String> {
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

fn inspect_execution_json(input: &Path) -> Result<String, String> {
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

fn inspect_artifact_json(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
    container: Option<&aot::NuisCompiledArtifactContainerInspect>,
    manifest_verify: Option<&aot::BuildManifestVerifyReport>,
) -> String {
    let mut fields = vec![
        json_string_field("kind", "nuis_artifact_inspect"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &artifact.schema),
        json_string_field("packaging_mode", &artifact.packaging_mode),
        json_string_field("cpu_target_abi", &artifact.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &artifact.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &artifact.cpu_target_machine_os),
        json_string_field(
            "cpu_target_object_format",
            &artifact.cpu_target_object_format,
        ),
        json_string_field("cpu_target_calling_abi", &artifact.cpu_target_calling_abi),
        json_string_field("binary_name", &artifact.binary_name),
        json_usize_field("binary_bytes", artifact.binary_bytes),
        json_usize_field("build_manifest_bytes", artifact.build_manifest_bytes),
        json_string_field("envelope_schema", &artifact.envelope.schema),
        json_string_array_field(
            "envelope_contract_families",
            &artifact.envelope.contract_families,
        ),
        json_string_field("lifecycle_schema", &artifact.lifecycle.schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &artifact.lifecycle.bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &artifact.lifecycle.tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &artifact.lifecycle.shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &artifact.lifecycle.yalivia_rpc),
        json_usize_field(
            "lifecycle_hook_count",
            artifact.lifecycle.hook_surface.len(),
        ),
        json_string_array_field("lifecycle_hook_surface", &artifact.lifecycle.hook_surface),
        json_usize_field(
            "lifecycle_export_count",
            artifact.lifecycle.export_surface.len(),
        ),
        json_string_array_field(
            "lifecycle_export_surface",
            &artifact.lifecycle.export_surface,
        ),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &artifact.lifecycle.runtime_capability_flags,
        ),
    ];
    if let Some(container) = container {
        fields.push(json_string_field(
            "artifact_container_magic",
            &container.magic,
        ));
        fields.push(json_usize_field(
            "artifact_container_version",
            container.binary_version as usize,
        ));
        fields.push(json_string_field(
            "artifact_container_kind",
            &container.container_kind,
        ));
        fields.push(json_usize_field(
            "artifact_section_count",
            container.section_count,
        ));
        fields.push(json_string_array_field(
            "artifact_section_names",
            &container.section_names,
        ));
        fields.push(json_bool_field(
            "artifact_section_table_valid",
            container.section_table_valid,
        ));
    }
    if let Some(report) = manifest_verify {
        let link_plan = linker::build_link_plan(report, artifact);
        let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
        let drift_check_count = drift_checks.len();
        let drift_mismatch_count = drift_checks
            .iter()
            .filter(|check| !check.consistent)
            .count();
        let verdicts = collect_domain_build_unit_verdicts(report);
        let summary = summarize_domain_build_verification(&verdicts);
        fields.push(json_usize_field(
            "domain_build_unit_count",
            report.domain_build_unit_count,
        ));
        fields.push(json_usize_field(
            "domain_payload_blobs_checked",
            report.domain_payload_blobs_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_blob_sections_checked",
            report.domain_payload_blob_sections_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_contract_sections_checked",
            report.domain_payload_contract_sections_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_lowering_plans_checked",
            report.domain_payload_lowering_plans_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_backend_stubs_checked",
            report.domain_payload_backend_stubs_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_bridge_plans_checked",
            report.domain_payload_bridge_plans_checked,
        ));
        fields.push(json_usize_field(
            "domain_bridge_stubs_checked",
            report.domain_bridge_stubs_checked,
        ));
        fields.push(format!(
            "\"domain_build_units\":[{}]",
            report
                .domain_build_units
                .iter()
                .map(domain_build_unit_json)
                .collect::<Vec<_>>()
                .join(",")
        ));
        fields.push(format!(
            "\"domain_build_contracts\":[{}]",
            domain_build_unit_contracts_json(&report.domain_build_units)
        ));
        fields.push(json_usize_field(
            "domain_build_contract_drift_checked",
            drift_check_count,
        ));
        fields.push(json_usize_field(
            "domain_build_contract_drift_mismatches",
            drift_mismatch_count,
        ));
        fields.push(json_bool_field(
            "domain_build_contracts_consistent",
            drift_mismatch_count == 0,
        ));
        fields.push(json_usize_field(
            "bridge_registry_entries_checked",
            report.bridge_registry_entries_checked,
        ));
        fields.push(json_usize_field(
            "host_bridge_plan_entries_checked",
            report.host_bridge_plan_entries_checked,
        ));
        fields.push(format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ));
        fields.push(format!(
            "\"domain_build_unit_verdicts\":[{}]",
            verdicts
                .iter()
                .map(domain_build_unit_verification_verdict_json)
                .collect::<Vec<_>>()
                .join(",")
        ));
        fields.push(format!(
            "\"domain_build_contract_drift\":[{}]",
            drift_checks
                .iter()
                .map(domain_build_contract_drift_json)
                .collect::<Vec<_>>()
                .join(",")
        ));
        fields.push(format!("\"link_plan\":{}", link_plan_json(&link_plan)));
    }
    format!("{{{}}}", fields.join(","))
}

fn verify_artifact_json(input: &Path, report: &aot::NuisCompiledArtifactVerifyReport) -> String {
    let fields = vec![
        json_string_field("kind", "nuis_artifact_verify"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &report.schema),
        json_string_field("artifact_container_kind", &report.artifact_container_kind),
        json_usize_field(
            "artifact_container_version",
            report.artifact_container_version as usize,
        ),
        json_usize_field("artifact_section_count", report.artifact_section_count),
        json_string_array_field("artifact_section_names", &report.artifact_section_names),
        json_bool_field(
            "artifact_section_table_valid",
            report.artifact_section_table_valid,
        ),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("binary_name", &report.binary_name),
        json_usize_field("binary_bytes", report.binary_bytes),
        json_usize_field("build_manifest_bytes", report.build_manifest_bytes),
        json_string_field("envelope_schema", &report.envelope_schema),
        json_usize_field("envelope_package_count", report.envelope_package_count),
        json_string_field("lifecycle_schema", &report.lifecycle_schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &report.lifecycle_bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &report.lifecycle_shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
        json_usize_field("lifecycle_hook_count", report.lifecycle_hook_count),
        json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
        json_usize_field("lifecycle_export_count", report.lifecycle_export_count),
        json_string_array_field("lifecycle_export_surface", &report.lifecycle_export_surface),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &report.lifecycle_runtime_capability_flags,
        ),
        json_bool_field(
            "lifecycle_contract_consistent",
            report.lifecycle_contract_consistent,
        ),
        json_bool_field(
            "lifecycle_runtime_capability_flags_consistent",
            report.lifecycle_runtime_capability_flags_consistent,
        ),
        json_usize_field(
            "execution_contracts_checked",
            report.execution_contracts_checked,
        ),
        json_string_field("cpu_target_abi", &report.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &report.cpu_target_machine_os),
        json_string_field("cpu_target_object_format", &report.cpu_target_object_format),
        json_string_field("cpu_target_calling_abi", &report.cpu_target_calling_abi),
        json_bool_field(
            "artifact_roundtrip_verified",
            report.artifact_roundtrip_verified,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn verify_build_manifest_json(input: &Path, report: &aot::BuildManifestVerifyReport) -> String {
    let domain_build_units = report
        .domain_build_units
        .iter()
        .map(domain_build_unit_json)
        .collect::<Vec<_>>()
        .join(",");
    let domain_build_contracts = domain_build_unit_contracts_json(&report.domain_build_units);
    let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
    let drift_mismatch_count = drift_checks
        .iter()
        .filter(|check| !check.consistent)
        .count();
    let verdicts = collect_domain_build_unit_verdicts(report);
    let summary = summarize_domain_build_verification(&verdicts);
    let fields = vec![
        json_string_field("kind", "nuis_build_manifest_verify"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &report.schema),
        json_string_field("manifest_input", &report.input),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("envelope_path", &report.envelope_path),
        json_string_field("envelope_schema", &report.envelope_schema),
        json_usize_field("envelope_package_count", report.envelope_package_count),
        json_string_field("artifact_path", &report.artifact_path),
        json_string_field("artifact_schema", &report.artifact_schema),
        json_string_field("artifact_binary_name", &report.artifact_binary_name),
        json_usize_field("artifact_binary_bytes", report.artifact_binary_bytes),
        json_string_field("lifecycle_schema", &report.lifecycle_schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &report.lifecycle_bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &report.lifecycle_shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
        json_usize_field("lifecycle_hook_count", report.lifecycle_hook_count),
        json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
        json_usize_field("lifecycle_export_count", report.lifecycle_export_count),
        json_string_array_field("lifecycle_export_surface", &report.lifecycle_export_surface),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &report.lifecycle_runtime_capability_flags,
        ),
        json_usize_field(
            "execution_contracts_checked",
            report.execution_contracts_checked,
        ),
        json_usize_field("domain_build_unit_count", report.domain_build_unit_count),
        json_usize_field(
            "heterogeneous_domain_count",
            report.heterogeneous_domain_count,
        ),
        json_usize_field(
            "domain_payload_blobs_checked",
            report.domain_payload_blobs_checked,
        ),
        json_usize_field(
            "domain_payload_blob_sections_checked",
            report.domain_payload_blob_sections_checked,
        ),
        json_usize_field(
            "domain_payload_contract_sections_checked",
            report.domain_payload_contract_sections_checked,
        ),
        json_usize_field(
            "domain_payload_lowering_plans_checked",
            report.domain_payload_lowering_plans_checked,
        ),
        json_usize_field(
            "domain_payload_backend_stubs_checked",
            report.domain_payload_backend_stubs_checked,
        ),
        json_usize_field(
            "domain_payload_bridge_plans_checked",
            report.domain_payload_bridge_plans_checked,
        ),
        json_usize_field(
            "domain_bridge_stubs_checked",
            report.domain_bridge_stubs_checked,
        ),
        format!("\"domain_build_units\":[{}]", domain_build_units),
        format!("\"domain_build_contracts\":[{}]", domain_build_contracts),
        json_usize_field("domain_build_contract_drift_checked", drift_checks.len()),
        json_usize_field(
            "domain_build_contract_drift_mismatches",
            drift_mismatch_count,
        ),
        json_bool_field(
            "domain_build_contracts_consistent",
            drift_mismatch_count == 0,
        ),
        format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ),
        format!(
            "\"domain_build_contract_drift\":[{}]",
            drift_checks
                .iter()
                .map(domain_build_contract_drift_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_optional_string_field(
            "bridge_registry_path",
            report.bridge_registry_path.as_deref(),
        ),
        json_usize_field("bridge_registry_units", report.bridge_registry_units),
        json_usize_field("bridge_registry_checked", report.bridge_registry_checked),
        json_usize_field(
            "bridge_registry_entries_checked",
            report.bridge_registry_entries_checked,
        ),
        json_optional_string_field(
            "host_bridge_plan_index_path",
            report.host_bridge_plan_index_path.as_deref(),
        ),
        json_usize_field("host_bridge_plan_units", report.host_bridge_plan_units),
        json_usize_field("host_bridge_plan_checked", report.host_bridge_plan_checked),
        json_usize_field(
            "host_bridge_plan_entries_checked",
            report.host_bridge_plan_entries_checked,
        ),
        json_optional_string_field("doc_index_path", report.doc_index_path.as_deref()),
        json_usize_field("doc_index_module_count", report.doc_index_module_count),
        json_usize_field(
            "doc_index_documented_item_count",
            report.doc_index_documented_item_count,
        ),
        json_usize_field("doc_index_checked", report.doc_index_checked),
        json_optional_string_field("project_docs_index", report.project_docs_index.as_deref()),
        json_usize_field(
            "project_docs_module_count",
            report.project_docs_module_count,
        ),
        json_usize_field(
            "project_docs_documented_module_count",
            report.project_docs_documented_module_count,
        ),
        json_usize_field(
            "project_docs_documented_item_count",
            report.project_docs_documented_item_count,
        ),
        json_optional_string_field(
            "project_imports_index",
            report.project_imports_index.as_deref(),
        ),
        json_usize_field(
            "project_imports_library_count",
            report.project_imports_library_count,
        ),
        json_usize_field(
            "project_imports_visible_library_count",
            report.project_imports_visible_library_count,
        ),
        json_usize_field(
            "project_imports_visible_module_count",
            report.project_imports_visible_module_count,
        ),
        json_usize_field(
            "project_imports_documented_visible_module_count",
            report.project_imports_documented_visible_module_count,
        ),
        json_usize_field(
            "project_imports_documented_visible_item_count",
            report.project_imports_documented_visible_item_count,
        ),
        json_optional_string_field(
            "project_galaxy_index",
            report.project_galaxy_index.as_deref(),
        ),
        json_usize_field("project_galaxy_count", report.project_galaxy_count),
        json_usize_field(
            "project_documented_galaxy_count",
            report.project_documented_galaxy_count,
        ),
        json_usize_field(
            "project_documented_galaxy_library_module_count",
            report.project_documented_galaxy_library_module_count,
        ),
        json_usize_field(
            "project_documented_galaxy_item_count",
            report.project_documented_galaxy_item_count,
        ),
        format!(
            "\"domain_build_unit_verdicts\":[{}]",
            verdicts
                .iter()
                .map(domain_build_unit_verification_verdict_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_string_field("cpu_target_abi", &report.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &report.cpu_target_machine_os),
        json_string_field("cpu_target_object_format", &report.cpu_target_object_format),
        json_string_field("cpu_target_calling_abi", &report.cpu_target_calling_abi),
        json_string_field("cpu_target_clang", &report.cpu_target_clang),
        json_bool_field("cpu_target_cross", report.cpu_target_cross),
        json_usize_field("artifacts_checked", report.artifacts_checked),
        json_usize_field("project_metadata_checked", report.project_metadata_checked),
    ];
    format!("{{{}}}", fields.join(","))
}

fn reconstruct_manifest_report_from_artifact(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
) -> Result<(PathBuf, aot::BuildManifestVerifyReport), String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nuis_artifact_report_{nonce}"));
    std::fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create `{}`: {error}", temp_root.display()))?;

    let manifest_path = temp_root.join("nuis.build.manifest.toml");
    let envelope_path = temp_root.join("nuis.executable.envelope.toml");
    let artifact_path = temp_root.join("nuis.compiled.artifact");
    let binary_path = temp_root.join(&artifact.binary_name);

    let result = (|| {
        std::fs::write(&binary_path, &artifact.binary_blob)
            .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
        aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
        let relocated_manifest = aot::render_relocated_unpacked_build_manifest(
            artifact,
            &temp_root,
            &envelope_path,
            &artifact_path,
            &binary_path,
        )?;
        let mut relocated_artifact = artifact.clone();
        relocated_artifact.build_manifest_source = relocated_manifest.clone();
        relocated_artifact.build_manifest_bytes = relocated_manifest.len();
        aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
        std::fs::write(&manifest_path, relocated_manifest)
            .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
        let report = aot::verify_build_manifest(&manifest_path)?;
        Ok((manifest_path.clone(), report))
    })();

    let _ = std::fs::remove_dir_all(&temp_root);
    result.map_err(|error: String| {
        format!(
            "failed to reconstruct build manifest context for `{}`: {error}",
            input.display()
        )
    })
}

fn artifact_report_json(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
    artifact_verify_input: &Path,
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    manifest_input: &Path,
    manifest_verify: &aot::BuildManifestVerifyReport,
    manifest_verify_reconstructed: bool,
) -> String {
    let verdicts = collect_domain_build_unit_verdicts(manifest_verify);
    let summary = summarize_domain_build_verification(&verdicts);
    let link_plan = linker::build_link_plan(manifest_verify, artifact);
    let doc_indexes =
        collect_doc_indexes_from_manifest_input(manifest_verify).unwrap_or_else(|_| Vec::new());
    let execution_inspect = inspect_execution_json(manifest_input).unwrap_or_else(|error| {
        format!(
            "{{{},{},{}}}",
            json_string_field("kind", "nuis_execution_inspect_error"),
            json_string_field("input", &manifest_input.display().to_string()),
            json_string_field("error", &error)
        )
    });
    let project_metadata =
        inspect_project_metadata_json(&project_metadata_summary_from_manifest_report(
            "build-manifest",
            Some(manifest_input),
            Some(artifact_verify_input),
            manifest_verify,
        ));
    let artifact_container =
        aot::inspect_nuis_compiled_artifact_container(artifact_verify_input).ok();
    let fields = vec![
        json_string_field("kind", "nuis_artifact_report"),
        json_string_field("input", &input.display().to_string()),
        json_bool_field(
            "manifest_verify_reconstructed",
            manifest_verify_reconstructed,
        ),
        format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ),
        format!(
            "\"artifact_inspect\":{}",
            inspect_artifact_json(
                input,
                artifact,
                artifact_container.as_ref(),
                Some(manifest_verify),
            )
        ),
        format!(
            "\"artifact_verify\":{}",
            verify_artifact_json(artifact_verify_input, artifact_verify)
        ),
        format!(
            "\"manifest_verify\":{}",
            verify_build_manifest_json(manifest_input, manifest_verify)
        ),
        format!("\"project_metadata\":{}", project_metadata),
        format!(
            "\"doc_index\":{}",
            inspect_docs_json(Path::new(&manifest_verify.input), &doc_indexes)
        ),
        format!("\"execution_inspect\":{}", execution_inspect),
        format!("\"link_plan\":{}", link_plan_json(&link_plan)),
    ];
    format!("{{{}}}", fields.join(","))
}

fn print_project_context(resolved: &pipeline::ResolvedCompileInput) {
    if let Some(project) = &resolved.project {
        eprintln!("nuisc: {}", project::describe_project(project));
    }
}

fn print_required_nustar_context(artifacts: &pipeline::PipelineArtifacts) -> Result<(), String> {
    let required =
        registry::load_required_manifests(Path::new(NUSTAR_REGISTRY_ROOT), &artifacts.yir)?;
    registry::validate_unit_binding(&required, &artifacts.ast.domain, &artifacts.ast.unit)?;
    eprintln!(
        "nuisc: lazily loaded nustar = {}",
        required
            .iter()
            .map(|manifest| manifest.package_id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

pub fn run(command: CommandKind) -> Result<(), String> {
    let frontend = frontend::frontend_name();
    let backend = codegen_wasm::backend_name();
    let engine = engine::default_engine();

    match command {
        CommandKind::Status => {
            let index = registry::load_index(Path::new("nustar-packages"))?;
            println!(
                "nuisc compiler core: topology-first scheduler frontend ({frontend} -> {backend}, yir={}, profile={}, indexed_nustar={})",
                engine.version,
                engine.profile,
                index.len()
            );
            for entry in index {
                println!(
                    "  - {} [{}] -> {}",
                    entry.package_id,
                    entry.domain_family,
                    registry::manifest_path(Path::new("nustar-packages"), &entry).display()
                );
            }
        }
        CommandKind::Registry { json } => {
            let registrations = registry::load_registered_domains(Path::new("nustar-packages"))?;
            if registrations.is_empty() {
                let placeholder_error = errors::NuiscError {
                    message: "no nustar packages discovered",
                };
                return Err(placeholder_error.message.to_owned());
            }
            if json {
                let contracts = registrations
                    .iter()
                    .map(|registration| {
                        let manifest = registry::load_manifest_for_domain(
                            Path::new("nustar-packages"),
                            &registration.domain_family,
                        )?;
                        Ok(domain_registry_json(registration, &manifest))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                println!(
                    "{{{},{},{}}}",
                    format!(
                        "\"contract_schema\":\"{}\"",
                        registry::NUSTAR_DOMAIN_CONTRACT_SCHEMA
                    ),
                    json_bool_field("registry_indexed", true),
                    format!("\"domains\":[{}]", contracts.join(","))
                );
                return Ok(());
            }
            for registration in registrations {
                let manifest = registry::load_manifest_for_domain(
                    Path::new("nustar-packages"),
                    &registration.domain_family,
                )?;
                let capability = registry::capability_summary(&manifest);
                let execution = registry::execution_summary(&manifest);
                let scheduler = registry::scheduler_summary(&manifest);
                let build_contract = registry::domain_build_contract_summary(&manifest);
                println!("package: {}", manifest.package_id);
                println!("  schema: {}", manifest.manifest_schema);
                println!("  domain: {}", manifest.domain_family);
                println!("  frontend: {}", manifest.frontend);
                println!("  crate: {}", manifest.entry_crate);
                println!("  ast_entry: {}", manifest.ast_entry);
                println!("  nir_entry: {}", manifest.nir_entry);
                println!("  yir_lowering_entry: {}", manifest.yir_lowering_entry);
                println!("  part_verify_entry: {}", manifest.part_verify_entry);
                println!("  ast_surface: {}", manifest.ast_surface.join(", "));
                println!("  nir_surface: {}", manifest.nir_surface.join(", "));
                println!("  yir_lowering: {}", manifest.yir_lowering.join(", "));
                println!("  part_verify: {}", manifest.part_verify.join(", "));
                println!("  binary_extension: {}", manifest.binary_extension);
                println!("  package_layout: {}", manifest.package_layout);
                println!("  machine_abi_policy: {}", manifest.machine_abi_policy);
                if !manifest.abi_profiles.is_empty() {
                    println!("  abi_profiles: {}", manifest.abi_profiles.join(", "));
                }
                if !manifest.abi_capabilities.is_empty() {
                    println!(
                        "  abi_capabilities: {}",
                        manifest.abi_capabilities.join(", ")
                    );
                }
                println!(
                    "  implementation_kinds: {}",
                    manifest.implementation_kinds.join(", ")
                );
                println!("  loader_entry: {}", manifest.loader_entry);
                println!("  loader_abi: {}", manifest.loader_abi);
                if !manifest.host_ffi_surface.is_empty() {
                    println!(
                        "  host_ffi_surface: {}",
                        manifest.host_ffi_surface.join(", ")
                    );
                    println!("  host_ffi_abis: {}", manifest.host_ffi_abis.join(", "));
                    println!("  host_ffi_bridge: {}", manifest.host_ffi_bridge);
                }
                if !capability.support_surface.is_empty() {
                    println!(
                        "  support_surface: {}",
                        capability.support_surface.join(", ")
                    );
                }
                if !capability.support_profile_slots.is_empty() {
                    println!(
                        "  support_profile_slots: {}",
                        capability.support_profile_slots.join(", ")
                    );
                }
                if !capability.default_lanes.is_empty() {
                    println!("  default_lanes: {}", capability.default_lanes.join(", "));
                }
                println!("  clock_domain_id: {}", capability.clock.domain_id);
                println!("  clock_kind: {}", capability.clock.kind);
                println!("  clock_epoch_kind: {}", capability.clock.epoch_kind);
                println!("  clock_resolution: {}", capability.clock.resolution);
                println!(
                    "  clock_bridge_default: {}",
                    capability.clock.bridge_default
                );
                println!(
                    "  execution_skeleton_version: {}",
                    execution.skeleton_version
                );
                println!("  execution_function_kind: {}", execution.function_kind);
                println!("  execution_graph_kind: {}", execution.graph_kind);
                println!("  execution_domain: {}", execution.execution_domain);
                println!(
                    "  execution_default_time_mode: {}",
                    execution.default_time_mode
                );
                println!("  execution_contract_family: {}", execution.contract_family);
                println!("  scheduler_contract_stack: {}", scheduler.contract_stack);
                println!("  scheduler_result_roles: {}", scheduler.result_roles);
                if let Some(navigation) = scheduler.sample_navigation {
                    println!("  scheduler_sample_navigation: {}", navigation);
                }
                if let Some(samples) = scheduler.result_samples {
                    println!("  scheduler_result_samples: {}", samples);
                }
                if let Some(samples) = scheduler.transport_samples {
                    println!("  scheduler_transport_samples: {}", samples);
                }
                println!("  scheduler_summary_api: {}", scheduler.summary_api);
                if let Some(samples) = scheduler.summary_samples {
                    println!("  scheduler_summary_samples: {}", samples);
                }
                println!(
                    "  scheduler_observer_classes: {}",
                    scheduler.observer_classes
                );
                println!(
                    "  build_lowering_lane_policy: {}",
                    build_contract.lowering.lane_policy
                );
                println!(
                    "  build_lowering_bridge_surface: {}",
                    build_contract.lowering.bridge_surface
                );
                println!(
                    "  build_lowering_emission_kind: {}",
                    build_contract.lowering.emission_kind
                );
                println!(
                    "  build_backend_stub_kind: {}",
                    build_contract.backend.stub_kind
                );
                println!(
                    "  build_backend_bridge_entry: {}",
                    build_contract.backend.bridge_entry
                );
                println!(
                    "  build_backend_submission_mode: {}",
                    build_contract.backend.submission_mode
                );
                println!(
                    "  build_backend_wake_policy: {}",
                    build_contract.backend.wake_policy
                );
                println!(
                    "  build_backend_scheduler_binding: {}",
                    build_contract.backend.scheduler_binding
                );
                if let Some(phase_bind) = build_contract.backend.phase_bind.as_deref() {
                    println!("  build_backend_phase_bind: {}", phase_bind);
                }
                if let Some(phase_submit) = build_contract.backend.phase_submit.as_deref() {
                    println!("  build_backend_phase_submit: {}", phase_submit);
                }
                if let Some(phase_wait) = build_contract.backend.phase_wait.as_deref() {
                    println!("  build_backend_phase_wait: {}", phase_wait);
                }
                if let Some(phase_finalize) = build_contract.backend.phase_finalize.as_deref() {
                    println!("  build_backend_phase_finalize: {}", phase_finalize);
                }
                if let Some(transport_model) = build_contract.backend.transport_model.as_deref() {
                    println!("  build_backend_transport_model: {}", transport_model);
                }
                if let Some(request_shape) = build_contract.backend.request_shape.as_deref() {
                    println!("  build_backend_request_shape: {}", request_shape);
                }
                if let Some(response_shape) = build_contract.backend.response_shape.as_deref() {
                    println!("  build_backend_response_shape: {}", response_shape);
                }
                if let Some(dispatch_shape) = build_contract.backend.dispatch_shape.as_deref() {
                    println!("  build_backend_dispatch_shape: {}", dispatch_shape);
                }
                if let Some(memory_binding) = build_contract.backend.memory_binding.as_deref() {
                    println!("  build_backend_memory_binding: {}", memory_binding);
                }
                if let Some(resource_binding) = build_contract.backend.resource_binding.as_deref() {
                    println!("  build_backend_resource_binding: {}", resource_binding);
                }
                if let Some(completion_model) = build_contract.backend.completion_model.as_deref() {
                    println!("  build_backend_completion_model: {}", completion_model);
                }
                println!(
                    "  build_bridge_surface: {}",
                    build_contract.bridge.bridge_surface
                );
                println!(
                    "  build_bridge_entry: {}",
                    build_contract.bridge.bridge_entry
                );
                println!(
                    "  build_bridge_scheduler_binding: {}",
                    build_contract.bridge.scheduler_binding
                );
                println!(
                    "  build_bridge_phase_bind: {}",
                    build_contract.bridge.phase_bind
                );
                println!(
                    "  build_bridge_phase_submit: {}",
                    build_contract.bridge.phase_submit
                );
                println!(
                    "  build_bridge_phase_wait: {}",
                    build_contract.bridge.phase_wait
                );
                println!(
                    "  build_bridge_phase_finalize: {}",
                    build_contract.bridge.phase_finalize
                );
                println!("  build_bridge_kind: {}", build_contract.bridge.bridge_kind);
                println!(
                    "  host_bridge_host_ffi_surface: {}",
                    build_contract.host_bridge.host_ffi_surface
                );
                println!(
                    "  host_bridge_handle_family: {}",
                    build_contract.host_bridge.handle_family
                );
                println!(
                    "  host_bridge_phase_order: {}",
                    build_contract.host_bridge.phase_order.join(", ")
                );
                println!(
                    "  host_bridge_phase_bind_inputs: {}",
                    build_contract.host_bridge.phase_bind_inputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_bind_outputs: {}",
                    build_contract.host_bridge.phase_bind_outputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_submit_inputs: {}",
                    build_contract.host_bridge.phase_submit_inputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_submit_outputs: {}",
                    build_contract.host_bridge.phase_submit_outputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_wait_inputs: {}",
                    build_contract.host_bridge.phase_wait_inputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_wait_outputs: {}",
                    build_contract.host_bridge.phase_wait_outputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_finalize_inputs: {}",
                    build_contract.host_bridge.phase_finalize_inputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_finalize_outputs: {}",
                    build_contract.host_bridge.phase_finalize_outputs.join(", ")
                );
                println!(
                    "  host_bridge_phase_bind_wake: {}",
                    build_contract.host_bridge.phase_bind_wake
                );
                println!(
                    "  host_bridge_phase_submit_wake: {}",
                    build_contract.host_bridge.phase_submit_wake
                );
                println!(
                    "  host_bridge_phase_wait_wake: {}",
                    build_contract.host_bridge.phase_wait_wake
                );
                println!(
                    "  host_bridge_phase_finalize_wake: {}",
                    build_contract.host_bridge.phase_finalize_wake
                );
                println!(
                    "  host_bridge_plan_begin: {}",
                    build_contract.host_bridge.bridge_plan_begin
                );
                println!(
                    "  host_bridge_plan_end: {}",
                    build_contract.host_bridge.bridge_plan_end
                );
                println!("  profiles: {}", manifest.profiles.join(", "));
                println!(
                    "  resource_families: {}",
                    manifest.resource_families.join(", ")
                );
                println!(
                    "  unit_types: {}",
                    if manifest.unit_types.is_empty() {
                        "<any>".to_owned()
                    } else {
                        manifest.unit_types.join(", ")
                    }
                );
                println!(
                    "  lowering_targets: {}",
                    manifest.lowering_targets.join(", ")
                );
                println!("  ops: {}", manifest.ops.join(", "));
            }
        }
        CommandKind::Fmt { input } => {
            let report = fmt::format_input(&input)?;
            println!("formatted nuis input: {}", input.display());
            println!("  total_files: {}", report.total_files);
            println!("  changed_files: {}", report.changed_files.len());
            for file in report.changed_files {
                println!("  - {}", file);
            }
        }
        CommandKind::Bindings { input } => {
            let compiled = compile_command_input(&input)?;
            let artifacts = &compiled.artifacts;
            let declared_used_units = artifacts
                .ast
                .uses
                .iter()
                .map(|item| (item.domain.clone(), item.unit.clone()))
                .collect::<Vec<_>>();
            let declared_externs = artifacts
                .ast
                .externs
                .iter()
                .map(|item| (item.abi.clone(), item.name.clone()))
                .chain(
                    artifacts
                        .ast
                        .extern_interfaces
                        .iter()
                        .flat_map(|interface| {
                            interface.methods.iter().map(move |method| {
                                (
                                    method.abi.clone(),
                                    format!("{}__{}", interface.name, method.name),
                                )
                            })
                        }),
                )
                .collect::<Vec<_>>();
            let plan = registry::plan_bindings(
                Path::new("nustar-packages"),
                &artifacts.nir,
                &artifacts.yir,
                &artifacts.ast.domain,
                &artifacts.ast.unit,
                &declared_used_units,
                &declared_externs,
            )?;
            println!("binding plan for: {}", input.display());
            if let Some(project) = &compiled.resolved.project {
                println!("project: {}", project::describe_project(project));
            }
            for binding in plan.bindings {
                println!("package: {}", binding.package_id);
                println!("  domain: {}", binding.domain_family);
                println!("  frontend: {}", binding.frontend);
                println!("  crate: {}", binding.entry_crate);
                println!("  ast_entry: {}", binding.ast_entry);
                println!("  nir_entry: {}", binding.nir_entry);
                println!("  yir_lowering_entry: {}", binding.yir_lowering_entry);
                println!("  part_verify_entry: {}", binding.part_verify_entry);
                println!("  machine_abi_policy: {}", binding.machine_abi_policy);
                if !binding.abi_profiles.is_empty() {
                    println!("  abi_profiles: {}", binding.abi_profiles.join(", "));
                }
                if !binding.abi_capabilities.is_empty() {
                    println!(
                        "  abi_capabilities: {}",
                        binding.abi_capabilities.join(", ")
                    );
                }
                println!("  ast_surface: {}", binding.ast_surface.join(", "));
                println!("  nir_surface: {}", binding.nir_surface.join(", "));
                println!("  yir_lowering: {}", binding.yir_lowering.join(", "));
                println!("  part_verify: {}", binding.part_verify.join(", "));
                if !binding.support_surface.is_empty() {
                    println!("  support_surface: {}", binding.support_surface.join(", "));
                }
                if !binding.support_profile_slots.is_empty() {
                    println!(
                        "  support_profile_slots: {}",
                        binding.support_profile_slots.join(", ")
                    );
                }
                if !binding.capability_tags.is_empty() {
                    println!("  capability_tags: {}", binding.capability_tags.join(", "));
                }
                if !binding.default_lanes.is_empty() {
                    println!("  default_lanes: {}", binding.default_lanes.join(", "));
                }
                println!(
                    "  execution_skeleton_version: {}",
                    binding.execution.skeleton_version
                );
                println!(
                    "  execution_function_kind: {}",
                    binding.execution.function_kind
                );
                println!("  execution_graph_kind: {}", binding.execution.graph_kind);
                println!("  execution_domain: {}", binding.execution.execution_domain);
                println!(
                    "  execution_default_time_mode: {}",
                    binding.execution.default_time_mode
                );
                println!(
                    "  execution_contract_family: {}",
                    binding.execution.contract_family
                );
                if !binding.execution.lowering_targets.is_empty() {
                    println!(
                        "  execution_lowering_targets: {}",
                        binding.execution.lowering_targets.join(", ")
                    );
                }
                if !binding.matched_support_surface.is_empty() {
                    println!(
                        "  matched_support_surface: {}",
                        binding.matched_support_surface.join(", ")
                    );
                }
                if !binding.matched_support_profile_slots.is_empty() {
                    println!(
                        "  matched_support_profile_slots: {}",
                        binding.matched_support_profile_slots.join(", ")
                    );
                }
                if !binding.covered_support_profile_slots.is_empty() {
                    println!(
                        "  covered_support_profile_slots: {}",
                        binding.covered_support_profile_slots.join(", ")
                    );
                }
                if !binding.uncovered_support_profile_slots.is_empty() {
                    println!(
                        "  uncovered_support_profile_slots: {}",
                        binding.uncovered_support_profile_slots.join(", ")
                    );
                }
                println!(
                    "  registered_units: {}",
                    if binding.registered_units.is_empty() {
                        "<registry-only>".to_owned()
                    } else {
                        binding.registered_units.join(", ")
                    }
                );
                if let Some(bound_unit) = &binding.bound_unit {
                    println!("  bound_unit: {}", bound_unit);
                }
                if !binding.used_units.is_empty() {
                    println!("  used_units: {}", binding.used_units.join(", "));
                }
                if !binding.instantiated_units.is_empty() {
                    println!(
                        "  instantiated_units: {}",
                        binding.instantiated_units.join(", ")
                    );
                }
                if !binding.used_host_ffi_abis.is_empty() {
                    println!(
                        "  used_host_ffi_abis: {}",
                        binding.used_host_ffi_abis.join(", ")
                    );
                }
                if !binding.used_host_ffi_symbols.is_empty() {
                    println!(
                        "  used_host_ffi_symbols: {}",
                        binding.used_host_ffi_symbols.join(", ")
                    );
                }
                println!(
                    "  matched_resources: {}",
                    if binding.matched_resources.is_empty() {
                        "<none>".to_owned()
                    } else {
                        binding.matched_resources.join(", ")
                    }
                );
                println!(
                    "  matched_ops: {}",
                    if binding.matched_ops.is_empty() {
                        "<none>".to_owned()
                    } else {
                        binding.matched_ops.join(", ")
                    }
                );
                if !binding.undeclared_ops.is_empty() {
                    println!("  undeclared_ops: {}", binding.undeclared_ops.join(", "));
                }
            }
        }
        CommandKind::PackNustar { package_id, output } => {
            let manifest = registry::load_manifest(Path::new("nustar-packages"), &package_id)?;
            nustar_binary::validate_manifest_for_packaging(&manifest)?;
            let blob = format!(
                "nustar_impl_stub\npackage={}\nfrontend={}\nentry_crate={}\n",
                manifest.package_id, manifest.frontend, manifest.entry_crate
            )
            .into_bytes();
            let binary = nustar_binary::default_binary(manifest, blob);
            nustar_binary::write_to_path(&output, &binary)?;
            println!("packed nustar binary: {}", output.display());
            println!("  package: {}", binary.manifest.package_id);
            println!("  extension: .nustar");
            println!("  format_version: {}", binary.format_version);
            println!("  abi: {}", binary.abi_tag);
            println!("  machine_arch: {}", binary.machine_arch);
            println!("  machine_os: {}", binary.machine_os);
            println!("  object_format: {}", binary.object_format);
            println!("  calling_abi: {}", binary.calling_abi);
            println!("  format: {}", binary.implementation_format);
            println!("  checksum: {}", binary.implementation_checksum);
            println!(
                "  abi_profiles: {}",
                binary.manifest.abi_profiles.join(", ")
            );
            println!(
                "  abi_capabilities: {}",
                binary.manifest.abi_capabilities.join(", ")
            );
            if !binary.manifest.abi_targets.is_empty() {
                println!("  abi_targets: {}", binary.manifest.abi_targets.join(", "));
            }
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::InspectNustar { input } => {
            let binary = nustar_binary::read_from_path(&input)?;
            let capability = registry::capability_summary(&binary.manifest);
            println!("nustar binary: {}", input.display());
            println!("  package: {}", binary.manifest.package_id);
            println!("  domain: {}", binary.manifest.domain_family);
            println!("  frontend: {}", binary.manifest.frontend);
            println!("  crate: {}", binary.manifest.entry_crate);
            println!("  ast_entry: {}", binary.manifest.ast_entry);
            println!("  nir_entry: {}", binary.manifest.nir_entry);
            println!(
                "  yir_lowering_entry: {}",
                binary.manifest.yir_lowering_entry
            );
            println!("  part_verify_entry: {}", binary.manifest.part_verify_entry);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
            if !binary.manifest.abi_profiles.is_empty() {
                println!(
                    "  abi_profiles: {}",
                    binary.manifest.abi_profiles.join(", ")
                );
            }
            if !binary.manifest.abi_capabilities.is_empty() {
                println!(
                    "  abi_capabilities: {}",
                    binary.manifest.abi_capabilities.join(", ")
                );
            }
            if !binary.manifest.abi_targets.is_empty() {
                println!("  abi_targets: {}", binary.manifest.abi_targets.join(", "));
            }
            if !binary.manifest.host_ffi_surface.is_empty() {
                println!(
                    "  host_ffi_surface: {}",
                    binary.manifest.host_ffi_surface.join(", ")
                );
                println!(
                    "  host_ffi_abis: {}",
                    binary.manifest.host_ffi_abis.join(", ")
                );
                println!("  host_ffi_bridge: {}", binary.manifest.host_ffi_bridge);
            }
            if !capability.support_surface.is_empty() {
                println!(
                    "  support_surface: {}",
                    capability.support_surface.join(", ")
                );
            }
            if !capability.support_profile_slots.is_empty() {
                println!(
                    "  support_profile_slots: {}",
                    capability.support_profile_slots.join(", ")
                );
            }
            if !capability.default_lanes.is_empty() {
                println!("  default_lanes: {}", capability.default_lanes.join(", "));
            }
            println!("  clock_domain_id: {}", capability.clock.domain_id);
            println!("  clock_kind: {}", capability.clock.kind);
            println!("  clock_epoch_kind: {}", capability.clock.epoch_kind);
            println!("  clock_resolution: {}", capability.clock.resolution);
            println!(
                "  clock_bridge_default: {}",
                capability.clock.bridge_default
            );
            println!("  format_version: {}", binary.format_version);
            println!("  abi: {}", binary.abi_tag);
            println!("  machine_arch: {}", binary.machine_arch);
            println!("  machine_os: {}", binary.machine_os);
            println!("  object_format: {}", binary.object_format);
            println!("  calling_abi: {}", binary.calling_abi);
            println!(
                "  machine_abi_compatible_with_host: {}",
                nustar_binary::machine_abi_matches_host(&binary)
            );
            println!("  format: {}", binary.implementation_format);
            println!("  checksum: {}", binary.implementation_checksum);
            println!("  profiles: {}", binary.manifest.profiles.join(", "));
            println!(
                "  resource_families: {}",
                binary.manifest.resource_families.join(", ")
            );
            println!(
                "  unit_types: {}",
                if binary.manifest.unit_types.is_empty() {
                    "<any>".to_owned()
                } else {
                    binary.manifest.unit_types.join(", ")
                }
            );
            println!(
                "  lowering_targets: {}",
                binary.manifest.lowering_targets.join(", ")
            );
            println!("  ops: {}", binary.manifest.ops.join(", "));
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::LoaderContract { package_id } => {
            let manifest = registry::load_manifest(Path::new("nustar-packages"), &package_id)?;
            let binary = nustar_binary::default_binary(manifest, Vec::new());
            let capability = registry::capability_summary(&binary.manifest);
            println!("loader contract: {}", binary.manifest.package_id);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
            if !capability.support_surface.is_empty() {
                println!(
                    "  support_surface: {}",
                    capability.support_surface.join(", ")
                );
            }
            if !capability.support_profile_slots.is_empty() {
                println!(
                    "  support_profile_slots: {}",
                    capability.support_profile_slots.join(", ")
                );
            }
            if !capability.default_lanes.is_empty() {
                println!("  default_lanes: {}", capability.default_lanes.join(", "));
            }
            println!("  clock_domain_id: {}", capability.clock.domain_id);
            println!("  clock_kind: {}", capability.clock.kind);
            println!("  clock_epoch_kind: {}", capability.clock.epoch_kind);
            println!("  clock_resolution: {}", capability.clock.resolution);
            println!(
                "  clock_bridge_default: {}",
                capability.clock.bridge_default
            );
            println!(
                "  canonical_entry_signature: {}",
                nustar_binary::CANONICAL_ENTRY_SIGNATURE
            );
            println!(
                "  canonical_host_abi_struct: {}",
                nustar_binary::CANONICAL_HOST_ABI_STRUCT
            );
            println!(
                "  canonical_result_struct: {}",
                nustar_binary::CANONICAL_RESULT_STRUCT
            );
            println!(
                "  loader_status_convention: {}",
                nustar_binary::CANONICAL_LOADER_STATUS_CONVENTION
            );
            println!(
                "  machine_abi_policy: {}",
                binary.manifest.machine_abi_policy
            );
            println!("  host_machine_arch: {}", binary.machine_arch);
            println!("  host_machine_os: {}", binary.machine_os);
            println!("  host_object_format: {}", binary.object_format);
            println!("  host_calling_abi: {}", binary.calling_abi);
            for contract in nustar_binary::implementation_contracts(&binary) {
                println!("  kind: {}", contract.kind);
                println!("    loader_abi: {}", contract.loader_abi);
                println!("    entry_symbol: {}", contract.entry_symbol);
                println!("    entry_signature: {}", contract.entry_signature);
                println!("    host_abi_struct: {}", contract.host_abi_struct);
                println!("    result_struct: {}", contract.result_struct);
                println!("    status_convention: {}", contract.status_convention);
                println!("    artifact_container: {}", contract.artifact_container);
                println!(
                    "    implementation_section: {}",
                    contract.implementation_section
                );
                println!(
                    "    required_exports: {}",
                    contract.required_exports.join(", ")
                );
                println!(
                    "    required_metadata: {}",
                    contract.required_metadata.join(", ")
                );
                println!("    link_mode: {}", contract.link_mode);
                println!("    machine_abi_policy: {}", contract.machine_abi_policy);
                println!("    notes: {}", contract.notes);
            }
        }
        CommandKind::PackEnvelope { input, output } => {
            let envelope = load_nuis_executable_envelope(&input)?;
            let encoded = aot::encode_nuis_executable_envelope_binary(&envelope)?;
            std::fs::write(&output, encoded)
                .map_err(|error| format!("failed to write `{}`: {error}", output.display()))?;
            println!("packed nuis envelope: {}", output.display());
            println!("  source: {}", input.display());
            println!("  schema: {}", envelope.schema);
            println!("  executable_kind: {}", envelope.executable_kind);
            println!("  package_count: {}", envelope.package_count);
        }
        CommandKind::UnpackEnvelope { input, output } => {
            let envelope = load_nuis_executable_envelope(&input)?;
            aot::write_nuis_executable_envelope(&output, &envelope)?;
            println!("unpacked nuis envelope: {}", output.display());
            println!("  source: {}", input.display());
            println!("  schema: {}", envelope.schema);
            println!("  executable_kind: {}", envelope.executable_kind);
            println!("  package_count: {}", envelope.package_count);
        }
        CommandKind::InspectEnvelope { input } => {
            let envelope = load_nuis_executable_envelope(&input)?;
            println!("nuis envelope: {}", input.display());
            println!("  schema: {}", envelope.schema);
            println!("  executable_kind: {}", envelope.executable_kind);
            println!("  package_count: {}", envelope.package_count);
            println!("  domain_families: {}", envelope.domain_families.join(", "));
            println!(
                "  contract_families: {}",
                envelope.contract_families.join(", ")
            );
            println!("  function_kind: {}", envelope.function_kind);
            println!("  graph_kind: {}", envelope.graph_kind);
            println!("  default_time_mode: {}", envelope.default_time_mode);
        }
        CommandKind::InspectArtifact { input, json } => {
            let artifact = load_nuis_compiled_artifact(&input)?;
            let is_manifest_input = input
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == "nuis.build.manifest.toml")
                .unwrap_or(false);
            let manifest_verify = if is_manifest_input {
                Some(aot::verify_build_manifest(&input)?)
            } else {
                Some(reconstruct_manifest_report_from_artifact(&input, &artifact)?.1)
            };
            let container = inspect_artifact_container_for_input(&input, manifest_verify.as_ref())?;
            if json {
                println!(
                    "{}",
                    inspect_artifact_json(
                        &input,
                        &artifact,
                        container.as_ref(),
                        manifest_verify.as_ref(),
                    )
                );
                return Ok(());
            }
            println!("nuis artifact: {}", input.display());
            if let Some(container) = &container {
                println!(
                    "  artifact_container: {} version {}",
                    container.container_kind, container.binary_version
                );
                println!("  artifact_section_count: {}", container.section_count);
                if !container.section_names.is_empty() {
                    println!(
                        "  artifact_section_names: {}",
                        container.section_names.join(", ")
                    );
                }
                println!(
                    "  artifact_section_table_valid: {}",
                    container.section_table_valid
                );
            }
            println!("  schema: {}", artifact.schema);
            println!("  packaging_mode: {}", artifact.packaging_mode);
            println!("  cpu_target_abi: {}", artifact.cpu_target_abi);
            println!(
                "  cpu_target_machine: {}-{}",
                artifact.cpu_target_machine_arch, artifact.cpu_target_machine_os
            );
            println!(
                "  cpu_target_object_format: {}",
                artifact.cpu_target_object_format
            );
            println!(
                "  cpu_target_calling_abi: {}",
                artifact.cpu_target_calling_abi
            );
            println!("  binary_name: {}", artifact.binary_name);
            println!("  binary_bytes: {}", artifact.binary_bytes);
            println!("  build_manifest_bytes: {}", artifact.build_manifest_bytes);
            println!("  envelope_schema: {}", artifact.envelope.schema);
            println!(
                "  envelope_contract_families: {}",
                artifact.envelope.contract_families.join(", ")
            );
            println!("  lifecycle_schema: {}", artifact.lifecycle.schema);
            println!(
                "  lifecycle_bootstrap_entry: {}",
                artifact.lifecycle.bootstrap_entry
            );
            println!(
                "  lifecycle_tick_policy: {}",
                artifact.lifecycle.tick_policy
            );
            println!(
                "  lifecycle_shutdown_policy: {}",
                artifact.lifecycle.shutdown_policy
            );
            println!(
                "  lifecycle_yalivia_rpc: {}",
                artifact.lifecycle.yalivia_rpc
            );
            println!(
                "  lifecycle_hook_count: {}",
                artifact.lifecycle.hook_surface.len()
            );
            println!(
                "  lifecycle_hook_surface: {}",
                artifact.lifecycle.hook_surface.join(", ")
            );
            println!(
                "  lifecycle_export_count: {}",
                artifact.lifecycle.export_surface.len()
            );
            println!(
                "  lifecycle_export_surface: {}",
                artifact.lifecycle.export_surface.join(", ")
            );
            println!(
                "  lifecycle_runtime_capability_flags: {}",
                artifact.lifecycle.runtime_capability_flags.join(", ")
            );
            if let Some(report) = &manifest_verify {
                let link_plan = linker::build_link_plan(report, &artifact);
                let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
                let drift_mismatch_count = drift_checks
                    .iter()
                    .filter(|check| !check.consistent)
                    .count();
                println!(
                    "  domain_build_unit_count: {}",
                    report.domain_build_unit_count
                );
                println!(
                    "  heterogeneous_domain_count: {}",
                    report.heterogeneous_domain_count
                );
                println!(
                    "  domain_payload_blobs_checked: {}",
                    report.domain_payload_blobs_checked
                );
                println!(
                    "  domain_payload_blob_sections_checked: {}",
                    report.domain_payload_blob_sections_checked
                );
                println!(
                    "  domain_payload_contract_sections_checked: {}",
                    report.domain_payload_contract_sections_checked
                );
                println!(
                    "  domain_payload_lowering_plans_checked: {}",
                    report.domain_payload_lowering_plans_checked
                );
                println!(
                    "  domain_payload_backend_stubs_checked: {}",
                    report.domain_payload_backend_stubs_checked
                );
                println!(
                    "  domain_payload_bridge_plans_checked: {}",
                    report.domain_payload_bridge_plans_checked
                );
                println!(
                    "  domain_bridge_stubs_checked: {}",
                    report.domain_bridge_stubs_checked
                );
                println!(
                    "  domain_build_contract_drift_checked: {}",
                    drift_checks.len()
                );
                println!(
                    "  domain_build_contract_drift_mismatches: {}",
                    drift_mismatch_count
                );
                println!(
                    "  domain_build_contracts_consistent: {}",
                    if drift_mismatch_count == 0 {
                        "true"
                    } else {
                        "false"
                    }
                );
                println!(
                    "  bridge_registry_entries_checked: {}",
                    report.bridge_registry_entries_checked
                );
                println!(
                    "  host_bridge_plan_entries_checked: {}",
                    report.host_bridge_plan_entries_checked
                );
                println!("  link_plan_final_stage: {}", link_plan.final_stage.kind);
                println!("  link_plan_final_driver: {}", link_plan.final_stage.driver);
                println!(
                    "  link_plan_final_link_mode: {}",
                    link_plan.final_stage.link_mode
                );
                println!(
                    "  link_plan_final_output: {}",
                    link_plan.final_stage.output_path
                );
                println!("  link_plan_domain_units: {}", link_plan.domain_units.len());
                for unit in &report.domain_build_units {
                    let verdict = domain_build_unit_verification_verdict(unit, report);
                    let build_contract = domain_build_unit_effective_contract_summary(unit);
                    println!(
                        "  domain_build_contract: {} [{}]",
                        unit.package_id, unit.domain_family
                    );
                    if let Some(abi) = unit.abi.as_deref() {
                        println!("    abi: {}", abi);
                    }
                    if let Some(target) = unit.selected_lowering_target.as_deref() {
                        println!("    selected_lowering_target: {}", target);
                    }
                    println!(
                        "    lowering: lane_policy={}, bridge_surface={}, emission_kind={}",
                        build_contract.lowering.lane_policy,
                        build_contract.lowering.bridge_surface,
                        build_contract.lowering.emission_kind
                    );
                    println!(
                        "    backend: stub_kind={}, bridge_entry={}, submission_mode={}, wake_policy={}, scheduler_binding={}",
                        build_contract.backend.stub_kind,
                        build_contract.backend.bridge_entry,
                        build_contract.backend.submission_mode,
                        build_contract.backend.wake_policy,
                        build_contract.backend.scheduler_binding
                    );
                    println!(
                        "    bridge: bridge_surface={}, bridge_entry={}, scheduler_binding={}, phase_bind={}, phase_submit={}, phase_wait={}, phase_finalize={}, bridge_kind={}",
                        build_contract.bridge.bridge_surface,
                        build_contract.bridge.bridge_entry,
                        build_contract.bridge.scheduler_binding,
                        build_contract.bridge.phase_bind,
                        build_contract.bridge.phase_submit,
                        build_contract.bridge.phase_wait,
                        build_contract.bridge.phase_finalize,
                        build_contract.bridge.bridge_kind
                    );
                    println!(
                        "    host_bridge: host_ffi_surface={}, handle_family={}, phase_order={}, phase_bind_wake={}, phase_submit_wake={}, phase_wait_wake={}, phase_finalize_wake={}, bridge_plan_begin={}, bridge_plan_end={}",
                        build_contract.host_bridge.host_ffi_surface,
                        build_contract.host_bridge.handle_family,
                        build_contract.host_bridge.phase_order.join(", "),
                        build_contract.host_bridge.phase_bind_wake,
                        build_contract.host_bridge.phase_submit_wake,
                        build_contract.host_bridge.phase_wait_wake,
                        build_contract.host_bridge.phase_finalize_wake,
                        build_contract.host_bridge.bridge_plan_begin,
                        build_contract.host_bridge.bridge_plan_end
                    );
                    let drift = evaluate_domain_build_contract_drift(unit);
                    println!(
                        "    registry_alignment: {}",
                        if drift.consistent { "ok" } else { "drift" }
                    );
                    println!(
                        "    verification_verdict: kind={} payload_blob={} lowering_plan={} backend_stub={} bridge_plan={} bridge_stub={} bridge_registry={} host_bridge_plan={} registry_alignment={} consistent={}",
                        verdict.kind,
                        verdict_status(verdict.payload_blob_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.lowering_plan_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.backend_stub_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.bridge_plan_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.bridge_stub_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.bridge_registry_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.host_bridge_plan_ok, verdict.kind == "hetero"),
                        if verdict.registry_alignment_ok { "ok" } else { "drift" },
                        if verdict.consistent { "true" } else { "false" }
                    );
                    if !verdict.failure_reasons.is_empty() {
                        println!(
                            "      failure_reasons: {}",
                            verdict.failure_reasons.join(", ")
                        );
                    }
                    for issue in drift.issues {
                        println!("      issue: {}", issue);
                    }
                }
            }
        }
        CommandKind::InspectExecution { input, json } => {
            if json {
                println!("{}", inspect_execution_json(&input)?);
            } else {
                println!("{}", render_execution_report(&input)?);
            }
        }
        CommandKind::ArtifactReport {
            input,
            json,
            summary,
        } => {
            let (
                manifest_input,
                artifact,
                artifact_verify_input,
                manifest_verify,
                manifest_verify_reconstructed,
            ) = resolve_artifact_report_inputs(&input)?;
            let artifact_verify = aot::verify_nuis_compiled_artifact(&artifact_verify_input)?;
            if json {
                println!(
                    "{}",
                    artifact_report_json(
                        &input,
                        &artifact,
                        &artifact_verify_input,
                        &artifact_verify,
                        &manifest_input,
                        &manifest_verify,
                        manifest_verify_reconstructed,
                    )
                );
                return Ok(());
            }
            let verdicts = collect_domain_build_unit_verdicts(&manifest_verify);
            let summary_view = summarize_domain_build_verification(&verdicts);
            let execution_overview = inspect_execution_overview(&manifest_input).ok();
            let doc_indexes = collect_doc_indexes_from_manifest_input(&manifest_verify).ok();
            let project_metadata = project_metadata_summary_from_manifest_report(
                "build-manifest",
                Some(&manifest_input),
                Some(&artifact_verify_input),
                &manifest_verify,
            );
            if summary {
                println!("nuis artifact report summary: {}", input.display());
                for line in artifact_report_summary_lines(
                    &artifact_verify,
                    &summary_view,
                    Some(&linker::build_link_plan(&manifest_verify, &artifact)),
                    manifest_verify_reconstructed,
                    execution_overview.as_ref(),
                    doc_indexes.as_deref(),
                    Some(&project_metadata),
                ) {
                    println!("  {}", line);
                }
                return Ok(());
            }
            println!("nuis artifact report: {}", input.display());
            println!("  artifact_schema: {}", artifact.schema);
            println!("  packaging_mode: {}", artifact.packaging_mode);
            println!("  binary_name: {}", artifact.binary_name);
            println!(
                "  artifact_roundtrip_verified: {}",
                if artifact_verify.artifact_roundtrip_verified {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  lifecycle_contract_consistent: {}",
                if artifact_verify.lifecycle_contract_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  lifecycle_runtime_capability_flags_consistent: {}",
                if artifact_verify.lifecycle_runtime_capability_flags_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!("  manifest_schema: {}", manifest_verify.schema);
            println!("  manifest_input: {}", manifest_input.display());
            println!(
                "  manifest_verify_reconstructed: {}",
                if manifest_verify_reconstructed {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  manifest_artifact_path: {}",
                manifest_verify.artifact_path
            );
            if let Some(indexes) = &doc_indexes {
                println!(
                    "  documented_modules: {}",
                    indexes
                        .iter()
                        .filter(|index| !index.items.is_empty())
                        .count()
                );
                println!(
                    "  documented_items: {}",
                    indexes.iter().map(|index| index.items.len()).sum::<usize>()
                );
            }
            println!(
                "  execution_contracts_checked: {}",
                manifest_verify.execution_contracts_checked
            );
            let summary = summary_view;
            for line in artifact_report_summary_lines(
                &artifact_verify,
                &summary,
                Some(&linker::build_link_plan(&manifest_verify, &artifact)),
                manifest_verify_reconstructed,
                execution_overview.as_ref(),
                doc_indexes.as_deref(),
                Some(&project_metadata),
            ) {
                println!("  {}", line);
            }
            println!(
                "  all_units_consistent: {}",
                if summary.all_units_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!("  total_units: {}", summary.total_units);
            println!("  host_units_checked: {}", summary.host_units_checked);
            println!("  hetero_units_checked: {}", summary.hetero_units_checked);
            println!("  registry_drift_units: {}", summary.registry_drift_units);
            println!(
                "  failing_units: {}",
                if summary.failing_units.is_empty() {
                    "<none>".to_owned()
                } else {
                    summary.failing_units.join(", ")
                }
            );
            println!(
                "  domain_payload_blobs_checked: {}",
                manifest_verify.domain_payload_blobs_checked
            );
            println!(
                "  domain_payload_blob_sections_checked: {}",
                manifest_verify.domain_payload_blob_sections_checked
            );
            println!(
                "  domain_payload_lowering_plans_checked: {}",
                manifest_verify.domain_payload_lowering_plans_checked
            );
            println!(
                "  domain_payload_backend_stubs_checked: {}",
                manifest_verify.domain_payload_backend_stubs_checked
            );
            println!(
                "  domain_payload_bridge_plans_checked: {}",
                manifest_verify.domain_payload_bridge_plans_checked
            );
            println!(
                "  domain_bridge_stubs_checked: {}",
                manifest_verify.domain_bridge_stubs_checked
            );
            println!(
                "  bridge_registry_entries_checked: {}",
                manifest_verify.bridge_registry_entries_checked
            );
            println!(
                "  host_bridge_plan_entries_checked: {}",
                manifest_verify.host_bridge_plan_entries_checked
            );
            println!(
                "  lifecycle_runtime_capability_flags: {}",
                manifest_verify
                    .lifecycle_runtime_capability_flags
                    .join(", ")
            );
        }
        CommandKind::VerifyArtifact { input, json } => {
            let artifact_input = if input.is_dir() {
                let artifact_path = input.join("nuis.compiled.artifact");
                if artifact_path.is_file() {
                    artifact_path
                } else {
                    let manifest_path = resolve_build_manifest_path(&input)?;
                    let report = aot::verify_build_manifest(&manifest_path)?;
                    PathBuf::from(report.artifact_path)
                }
            } else {
                input.clone()
            };
            let report = aot::verify_nuis_compiled_artifact(&artifact_input)?;
            if json {
                println!("{}", verify_artifact_json(&artifact_input, &report));
                return Ok(());
            }
            println!("nuis artifact verified: {}", artifact_input.display());
            println!("  schema: {}", report.schema);
            println!(
                "  artifact_container_kind: {}",
                report.artifact_container_kind
            );
            println!(
                "  artifact_container_version: {}",
                report.artifact_container_version
            );
            println!(
                "  artifact_section_count: {}",
                report.artifact_section_count
            );
            if !report.artifact_section_names.is_empty() {
                println!(
                    "  artifact_section_names: {}",
                    report.artifact_section_names.join(", ")
                );
            }
            println!(
                "  artifact_section_table_valid: {}",
                report.artifact_section_table_valid
            );
            println!("  packaging_mode: {}", report.packaging_mode);
            println!("  binary_name: {}", report.binary_name);
            println!("  binary_bytes: {}", report.binary_bytes);
            println!("  build_manifest_bytes: {}", report.build_manifest_bytes);
            println!("  envelope_schema: {}", report.envelope_schema);
            println!(
                "  envelope_package_count: {}",
                report.envelope_package_count
            );
            println!("  lifecycle_schema: {}", report.lifecycle_schema);
            println!(
                "  lifecycle_bootstrap_entry: {}",
                report.lifecycle_bootstrap_entry
            );
            println!("  lifecycle_tick_policy: {}", report.lifecycle_tick_policy);
            println!(
                "  lifecycle_shutdown_policy: {}",
                report.lifecycle_shutdown_policy
            );
            println!("  lifecycle_yalivia_rpc: {}", report.lifecycle_yalivia_rpc);
            println!("  lifecycle_hook_count: {}", report.lifecycle_hook_count);
            println!(
                "  lifecycle_hook_surface: {}",
                report.lifecycle_hook_surface.join(", ")
            );
            println!(
                "  lifecycle_export_count: {}",
                report.lifecycle_export_count
            );
            println!(
                "  lifecycle_export_surface: {}",
                report.lifecycle_export_surface.join(", ")
            );
            println!(
                "  lifecycle_runtime_capability_flags: {}",
                report.lifecycle_runtime_capability_flags.join(", ")
            );
            println!(
                "  lifecycle_contract_consistent: {}",
                if report.lifecycle_contract_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  lifecycle_runtime_capability_flags_consistent: {}",
                if report.lifecycle_runtime_capability_flags_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  execution_contracts_checked: {}",
                report.execution_contracts_checked
            );
            println!("  cpu_target_abi: {}", report.cpu_target_abi);
            println!(
                "  cpu_target_machine: {}-{}",
                report.cpu_target_machine_arch, report.cpu_target_machine_os
            );
            println!(
                "  cpu_target_object_format: {}",
                report.cpu_target_object_format
            );
            println!(
                "  cpu_target_calling_abi: {}",
                report.cpu_target_calling_abi
            );
            println!(
                "  artifact_roundtrip_verified: {}",
                if report.artifact_roundtrip_verified {
                    "true"
                } else {
                    "false"
                }
            );
        }
        CommandKind::UnpackArtifact { input, output_dir } => {
            let artifact = load_nuis_compiled_artifact(&input)?;
            std::fs::create_dir_all(&output_dir)
                .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
            let envelope_path = output_dir.join("nuis.executable.envelope.toml");
            let manifest_path = output_dir.join("nuis.build.manifest.toml");
            let artifact_path = output_dir.join("nuis.compiled.artifact");
            let binary_path = output_dir.join(&artifact.binary_name);
            aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
            std::fs::write(&binary_path, &artifact.binary_blob)
                .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
            let relocated_manifest = aot::render_relocated_unpacked_build_manifest(
                &artifact,
                &output_dir,
                &envelope_path,
                &artifact_path,
                &binary_path,
            )?;
            let mut relocated_artifact = artifact.clone();
            relocated_artifact.build_manifest_source = relocated_manifest.clone();
            relocated_artifact.build_manifest_bytes = relocated_manifest.len();
            aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
            std::fs::write(&manifest_path, relocated_manifest).map_err(|error| {
                format!("failed to write `{}`: {error}", manifest_path.display())
            })?;
            println!("unpacked nuis artifact: {}", output_dir.display());
            println!("  source: {}", input.display());
            println!("  manifest: {}", manifest_path.display());
            println!("  envelope: {}", envelope_path.display());
            println!("  artifact: {}", artifact_path.display());
            println!("  binary: {}", binary_path.display());
            println!("  packaging_mode: {}", artifact.packaging_mode);
        }
        CommandKind::VerifyBuildManifest { manifest, json } => {
            let manifest = resolve_build_manifest_path(&manifest)?;
            let report = aot::verify_build_manifest(&manifest)?;
            if json {
                println!("{}", verify_build_manifest_json(&manifest, &report));
                return Ok(());
            }
            if success_logs_enabled() {
                println!("build manifest verified: {}", manifest.display());
                println!("  schema: {}", report.schema);
                println!("  input: {}", report.input);
                println!("  output_dir: {}", report.output_dir);
                println!("  packaging_mode: {}", report.packaging_mode);
                println!("  envelope_path: {}", report.envelope_path);
                println!("  envelope_schema: {}", report.envelope_schema);
                println!(
                    "  envelope_package_count: {}",
                    report.envelope_package_count
                );
                println!("  artifact_path: {}", report.artifact_path);
                println!("  artifact_schema: {}", report.artifact_schema);
                println!("  artifact_binary_name: {}", report.artifact_binary_name);
                println!("  artifact_binary_bytes: {}", report.artifact_binary_bytes);
                println!("  lifecycle_schema: {}", report.lifecycle_schema);
                println!(
                    "  lifecycle_bootstrap_entry: {}",
                    report.lifecycle_bootstrap_entry
                );
                println!("  lifecycle_tick_policy: {}", report.lifecycle_tick_policy);
                println!(
                    "  lifecycle_shutdown_policy: {}",
                    report.lifecycle_shutdown_policy
                );
                println!("  lifecycle_yalivia_rpc: {}", report.lifecycle_yalivia_rpc);
                println!("  lifecycle_hook_count: {}", report.lifecycle_hook_count);
                println!(
                    "  lifecycle_hook_surface: {}",
                    report.lifecycle_hook_surface.join(", ")
                );
                println!(
                    "  lifecycle_export_count: {}",
                    report.lifecycle_export_count
                );
                println!(
                    "  lifecycle_export_surface: {}",
                    report.lifecycle_export_surface.join(", ")
                );
                println!(
                    "  lifecycle_runtime_capability_flags: {}",
                    report.lifecycle_runtime_capability_flags.join(", ")
                );
                println!(
                    "  execution_contracts_checked: {}",
                    report.execution_contracts_checked
                );
                println!(
                    "  domain_build_unit_count: {}",
                    report.domain_build_unit_count
                );
                println!(
                    "  heterogeneous_domain_count: {}",
                    report.heterogeneous_domain_count
                );
                println!(
                    "  domain_payload_blobs_checked: {}",
                    report.domain_payload_blobs_checked
                );
                println!(
                    "  domain_payload_blob_sections_checked: {}",
                    report.domain_payload_blob_sections_checked
                );
                println!(
                    "  domain_payload_contract_sections_checked: {}",
                    report.domain_payload_contract_sections_checked
                );
                println!(
                    "  domain_payload_lowering_plans_checked: {}",
                    report.domain_payload_lowering_plans_checked
                );
                println!(
                    "  domain_payload_backend_stubs_checked: {}",
                    report.domain_payload_backend_stubs_checked
                );
                println!(
                    "  domain_payload_bridge_plans_checked: {}",
                    report.domain_payload_bridge_plans_checked
                );
                println!(
                    "  domain_bridge_stubs_checked: {}",
                    report.domain_bridge_stubs_checked
                );
                let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
                let drift_mismatch_count = drift_checks
                    .iter()
                    .filter(|check| !check.consistent)
                    .count();
                println!(
                    "  domain_build_contract_drift_checked: {}",
                    drift_checks.len()
                );
                println!(
                    "  domain_build_contract_drift_mismatches: {}",
                    drift_mismatch_count
                );
                println!(
                    "  domain_build_contracts_consistent: {}",
                    if drift_mismatch_count == 0 {
                        "true"
                    } else {
                        "false"
                    }
                );
                for unit in &report.domain_build_units {
                    let verdict = domain_build_unit_verification_verdict(unit, &report);
                    let build_contract = domain_build_unit_effective_contract_summary(unit);
                    println!(
                        "  domain_build_contract: {} [{}]",
                        unit.package_id, unit.domain_family
                    );
                    if let Some(abi) = unit.abi.as_deref() {
                        println!("    abi: {}", abi);
                    }
                    if let Some(target) = unit.selected_lowering_target.as_deref() {
                        println!("    selected_lowering_target: {}", target);
                    }
                    println!(
                        "    lowering: lane_policy={}, bridge_surface={}, emission_kind={}",
                        build_contract.lowering.lane_policy,
                        build_contract.lowering.bridge_surface,
                        build_contract.lowering.emission_kind
                    );
                    println!(
                        "    backend: stub_kind={}, bridge_entry={}, submission_mode={}, wake_policy={}, scheduler_binding={}",
                        build_contract.backend.stub_kind,
                        build_contract.backend.bridge_entry,
                        build_contract.backend.submission_mode,
                        build_contract.backend.wake_policy,
                        build_contract.backend.scheduler_binding
                    );
                    println!(
                        "    bridge: bridge_surface={}, bridge_entry={}, scheduler_binding={}, phase_bind={}, phase_submit={}, phase_wait={}, phase_finalize={}, bridge_kind={}",
                        build_contract.bridge.bridge_surface,
                        build_contract.bridge.bridge_entry,
                        build_contract.bridge.scheduler_binding,
                        build_contract.bridge.phase_bind,
                        build_contract.bridge.phase_submit,
                        build_contract.bridge.phase_wait,
                        build_contract.bridge.phase_finalize,
                        build_contract.bridge.bridge_kind
                    );
                    println!(
                        "    host_bridge: host_ffi_surface={}, handle_family={}, phase_order={}, phase_bind_wake={}, phase_submit_wake={}, phase_wait_wake={}, phase_finalize_wake={}, bridge_plan_begin={}, bridge_plan_end={}",
                        build_contract.host_bridge.host_ffi_surface,
                        build_contract.host_bridge.handle_family,
                        build_contract.host_bridge.phase_order.join(", "),
                        build_contract.host_bridge.phase_bind_wake,
                        build_contract.host_bridge.phase_submit_wake,
                        build_contract.host_bridge.phase_wait_wake,
                        build_contract.host_bridge.phase_finalize_wake,
                        build_contract.host_bridge.bridge_plan_begin,
                        build_contract.host_bridge.bridge_plan_end
                    );
                    let drift = evaluate_domain_build_contract_drift(unit);
                    println!(
                        "    registry_alignment: {}",
                        if drift.consistent { "ok" } else { "drift" }
                    );
                    println!(
                        "    verification_verdict: kind={} payload_blob={} lowering_plan={} backend_stub={} bridge_plan={} bridge_stub={} bridge_registry={} host_bridge_plan={} registry_alignment={} consistent={}",
                        verdict.kind,
                        verdict_status(verdict.payload_blob_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.lowering_plan_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.backend_stub_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.bridge_plan_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.bridge_stub_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.bridge_registry_ok, verdict.kind == "hetero"),
                        verdict_status(verdict.host_bridge_plan_ok, verdict.kind == "hetero"),
                        if verdict.registry_alignment_ok { "ok" } else { "drift" },
                        if verdict.consistent { "true" } else { "false" }
                    );
                    if !verdict.failure_reasons.is_empty() {
                        println!(
                            "      failure_reasons: {}",
                            verdict.failure_reasons.join(", ")
                        );
                    }
                    for issue in drift.issues {
                        println!("      issue: {}", issue);
                    }
                }
                if let Some(path) = &report.bridge_registry_path {
                    println!("  bridge_registry_path: {}", path);
                }
                println!("  bridge_registry_units: {}", report.bridge_registry_units);
                println!(
                    "  bridge_registry_checked: {}",
                    report.bridge_registry_checked
                );
                println!(
                    "  bridge_registry_entries_checked: {}",
                    report.bridge_registry_entries_checked
                );
                if let Some(path) = &report.host_bridge_plan_index_path {
                    println!("  host_bridge_plan_index_path: {}", path);
                }
                println!(
                    "  host_bridge_plan_units: {}",
                    report.host_bridge_plan_units
                );
                println!(
                    "  host_bridge_plan_checked: {}",
                    report.host_bridge_plan_checked
                );
                println!(
                    "  host_bridge_plan_entries_checked: {}",
                    report.host_bridge_plan_entries_checked
                );
                if let Some(path) = &report.doc_index_path {
                    println!("  doc_index_path: {}", path);
                }
                println!(
                    "  doc_index_module_count: {}",
                    report.doc_index_module_count
                );
                println!(
                    "  doc_index_documented_item_count: {}",
                    report.doc_index_documented_item_count
                );
                println!("  doc_index_checked: {}", report.doc_index_checked);
                if let Some(path) = &report.project_docs_index {
                    println!("  project_docs_index: {}", path);
                }
                println!(
                    "  project_docs_module_count: {}",
                    report.project_docs_module_count
                );
                println!(
                    "  project_docs_documented_module_count: {}",
                    report.project_docs_documented_module_count
                );
                println!(
                    "  project_docs_documented_item_count: {}",
                    report.project_docs_documented_item_count
                );
                if let Some(path) = &report.project_imports_index {
                    println!("  project_imports_index: {}", path);
                }
                println!(
                    "  project_imports_library_count: {}",
                    report.project_imports_library_count
                );
                println!(
                    "  project_imports_visible_library_count: {}",
                    report.project_imports_visible_library_count
                );
                println!(
                    "  project_imports_visible_module_count: {}",
                    report.project_imports_visible_module_count
                );
                println!(
                    "  project_imports_documented_visible_module_count: {}",
                    report.project_imports_documented_visible_module_count
                );
                println!(
                    "  project_imports_documented_visible_item_count: {}",
                    report.project_imports_documented_visible_item_count
                );
                if let Some(path) = &report.project_galaxy_index {
                    println!("  project_galaxy_index: {}", path);
                }
                println!("  project_galaxy_count: {}", report.project_galaxy_count);
                println!(
                    "  project_documented_galaxy_count: {}",
                    report.project_documented_galaxy_count
                );
                println!(
                    "  project_documented_galaxy_library_module_count: {}",
                    report.project_documented_galaxy_library_module_count
                );
                println!(
                    "  project_documented_galaxy_item_count: {}",
                    report.project_documented_galaxy_item_count
                );
                for unit in &report.domain_build_units {
                    let payload_blob_bytes = unit
                        .artifact_payload_blob_bytes
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "<none>".to_owned());
                    println!(
                        "  domain_build_unit: {} package={} abi={} lowering={} backend={} role={} stub={} payload={} bridge_stub={} payload_blob={} payload_blob_bytes={} payload_format={}",
                        unit.domain_family,
                        unit.package_id,
                        unit.abi.as_deref().unwrap_or("<none>"),
                        unit.selected_lowering_target.as_deref().unwrap_or("<none>"),
                        unit.backend_family.as_deref().unwrap_or("<none>"),
                        unit.packaging_role,
                        unit.artifact_stub_path.as_deref().unwrap_or("<none>"),
                        unit.artifact_payload_path.as_deref().unwrap_or("<none>"),
                        unit.artifact_bridge_stub_path.as_deref().unwrap_or("<none>"),
                        unit.artifact_payload_blob_path.as_deref().unwrap_or("<none>"),
                        payload_blob_bytes,
                        unit.artifact_payload_format.as_deref().unwrap_or("<none>")
                    );
                }
                println!("  cpu_target_abi: {}", report.cpu_target_abi);
                println!(
                    "  cpu_target_machine: {}-{}",
                    report.cpu_target_machine_arch, report.cpu_target_machine_os
                );
                println!(
                    "  cpu_target_object_format: {}",
                    report.cpu_target_object_format
                );
                println!(
                    "  cpu_target_calling_abi: {}",
                    report.cpu_target_calling_abi
                );
                println!("  cpu_target_clang: {}", report.cpu_target_clang);
                println!(
                    "  cpu_target_cross: {}",
                    if report.cpu_target_cross {
                        "true"
                    } else {
                        "false"
                    }
                );
                if let Some(status) = report.compile_cache_status {
                    println!("  compile_cache_status: {}", status);
                }
                if let Some(key) = report.compile_cache_key {
                    println!("  compile_cache_key: {}", key);
                }
                if let Some(root) = report.compile_cache_root {
                    println!("  compile_cache_root: {}", root);
                }
                if let Some(plan_index) = report.project_plan_index {
                    println!("  project_plan_index: {}", plan_index);
                }
                if let Some(packet_index) = report.project_packet_index {
                    println!("  project_packet_index: {}", packet_index);
                }
                println!("  artifacts_checked: {}", report.artifacts_checked);
                println!(
                    "  project_metadata_checked: {}",
                    report.project_metadata_checked
                );
            }
        }
        CommandKind::InspectBenchmarks { input, json } => {
            let compiled = compile_command_input(&input)?;
            let benchmarks = collect_benchmark_inventory(&compiled.artifacts);
            if json {
                println!("{}", inspect_benchmarks_json(&input, &compiled.artifacts));
                return Ok(());
            }
            print_project_context(&compiled.resolved);
            println!("benchmark inventory: {}", input.display());
            println!(
                "  domain_unit: {}::{}",
                compiled.artifacts.nir.domain, compiled.artifacts.nir.unit
            );
            println!("  benchmark_count: {}", benchmarks.len());
            for entry in benchmarks {
                println!("  benchmark: {}", entry.symbol);
                println!("    label: {}", entry.label);
                println!(
                    "    async: {}",
                    if entry.is_async { "true" } else { "false" }
                );
                println!("    return_type: {}", entry.return_type);
                println!(
                    "    warmup_iters: {}",
                    entry
                        .warmup_iters
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                );
                println!(
                    "    measure_iters: {}",
                    entry
                        .measure_iters
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                );
                println!(
                    "    timeout_ms: {}",
                    entry
                        .timeout_ms
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                );
                println!(
                    "    clock_domain: {}",
                    entry.clock_domain.as_deref().unwrap_or("-")
                );
                println!(
                    "    clock_policy: {}",
                    entry.clock_policy.as_deref().unwrap_or("-")
                );
            }
        }
        CommandKind::InspectDocs {
            input,
            json,
            output,
        } => {
            let indexes = collect_doc_indexes(&input)?;
            if json {
                let payload = inspect_docs_json(&input, &indexes);
                if let Some(path) = output {
                    write_json_output(&path, &payload)?;
                    println!("wrote doc index: {}", path.display());
                    println!("  source: {}", input.display());
                    println!("  bytes: {}", payload.len());
                } else {
                    println!("{payload}");
                }
                return Ok(());
            }
            if project::is_project_input(&input) {
                let resolved = resolve_compile_input(&input)?;
                print_project_context(&resolved);
            }
            let summaries = summarize_doc_indexes(&indexes);
            let total_items = summaries
                .iter()
                .map(|summary| summary.item_count)
                .sum::<usize>();
            println!("doc index: {}", input.display());
            println!("  module_count: {}", summaries.len());
            println!("  documented_item_count: {}", total_items);
            for (index, summary) in indexes.iter().zip(summaries.iter()) {
                println!("  module: {}", summary.module_path);
                println!("    documented_items: {}", summary.item_count);
                for item in &index.items {
                    println!("    item: {} {}", item.kind, item.path);
                    if let Some(signature) = &item.signature {
                        println!("      signature: {}", signature);
                    }
                    for line in &item.docs {
                        println!("      doc: {}", line);
                    }
                }
            }
        }
        CommandKind::InspectGalaxyDocs { galaxy, json } => {
            let summary = inspect_galaxy_doc_summary(&galaxy)?;
            if json {
                println!("{}", inspect_galaxy_docs_json(&summary));
                return Ok(());
            }
            println!("galaxy doc index: {}", summary.galaxy);
            println!("  package_id: {}", summary.package_id);
            println!("  library_module_count: {}", summary.library_module_count);
            println!(
                "  documented_library_module_count: {}",
                summary.documented_library_module_count
            );
            println!("  documented_item_count: {}", summary.documented_item_count);
            for module in summary.modules {
                println!("  library_module: {}", module.library_module);
                println!("    module_path: {}", module.module_path);
                println!("    documented_items: {}", module.documented_item_count);
                for item in module.doc_index.items {
                    println!("    item: {} {}", item.kind, item.path);
                    if let Some(signature) = item.signature {
                        println!("      signature: {}", signature);
                    }
                    for line in item.docs {
                        println!("      doc: {}", line);
                    }
                }
            }
        }
        CommandKind::InspectStdlibDocs { json } => {
            let summary = inspect_stdlib_doc_summary()?;
            if json {
                println!("{}", inspect_stdlib_docs_json(&summary));
                return Ok(());
            }
            println!("stdlib doc index");
            println!("  galaxy_count: {}", summary.galaxy_count);
            println!(
                "  documented_galaxy_count: {}",
                summary.documented_galaxy_count
            );
            println!("  documented_item_count: {}", summary.documented_item_count);
            for galaxy in summary.galaxies {
                println!("  galaxy: {}", galaxy.galaxy);
                println!("    package_id: {}", galaxy.package_id);
                println!("    library_module_count: {}", galaxy.library_module_count);
                println!(
                    "    documented_library_module_count: {}",
                    galaxy.documented_library_module_count
                );
                println!(
                    "    documented_item_count: {}",
                    galaxy.documented_item_count
                );
            }
        }
        CommandKind::InspectProjectMetadata {
            input,
            json,
            summary,
            paths_only,
        } => {
            let metadata = inspect_project_metadata(&input)?;
            if json {
                println!("{}", inspect_project_metadata_json(&metadata));
                return Ok(());
            }
            if summary {
                println!("{}", render_project_metadata_compact_summary(&metadata));
                return Ok(());
            }
            if paths_only {
                println!("{}", render_project_metadata_paths(&metadata));
                return Ok(());
            }
            println!("{}", render_project_metadata_summary(&metadata));
        }
        CommandKind::RepairProjectMetadata { input, dry_run } => {
            let (project_input, output_dir) = repair_project_metadata_target(&input)?;
            if dry_run {
                println!("project metadata repair plan");
                println!("  source: {}", input.display());
                println!("  input: {}", project_input.display());
                println!("  output_dir: {}", output_dir.display());
                println!(
                    "  command: nuisc compile \"{}\" \"{}\"",
                    project_input.display(),
                    output_dir.display()
                );
                return Ok(());
            }
            run(CommandKind::Compile {
                input: project_input.clone(),
                output_dir: output_dir.clone(),
                verbose_cache: false,
                cpu_abi: None,
                target: None,
            })?;
            let repaired_manifest = output_dir.join("nuis.build.manifest.toml");
            let repaired_summary = inspect_project_metadata(&repaired_manifest)?;
            println!(
                "project metadata repaired: input={} output_dir={}",
                project_input.display(),
                output_dir.display()
            );
            println!(
                "{}",
                render_project_metadata_compact_summary(&repaired_summary)
            );
        }
        CommandKind::CacheStatus {
            input,
            all,
            verbose_cache,
            json,
        } => {
            if all {
                let workspace_root = std::env::current_dir()
                    .map_err(|error| format!("failed to resolve current directory: {error}"))?;
                let summary = cache::compile_cache_inventory_summary(&workspace_root)?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_inventory\",\"workspace_root\":\"{}\",\"roots_count\":{},\"entries\":{},\"files\":{},\"bytes\":{},\"roots\":[",
                        json_escape(&summary.workspace_root.display().to_string()),
                        summary.roots.len(),
                        summary.total_entries,
                        summary.total_files,
                        summary.total_bytes
                    );
                    for (root_index, inventory) in summary.roots.iter().enumerate() {
                        if root_index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"root\":\"{}\",\"entries\":{},\"files\":{},\"bytes\":{}",
                            json_escape(&inventory.root.display().to_string()),
                            inventory.entry_count,
                            inventory.total_files,
                            inventory.total_bytes
                        );
                        if verbose_cache {
                            print!(",\"items\":[");
                            for (entry_index, entry) in inventory.entries.iter().enumerate() {
                                if entry_index > 0 {
                                    print!(",");
                                }
                                print!(
                                    "{{\"key\":\"{}\",\"files\":{},\"bytes\":{},\"dir\":\"{}\"}}",
                                    json_escape(&entry.key),
                                    entry.file_count,
                                    entry.total_bytes,
                                    json_escape(&entry.entry_dir.display().to_string())
                                );
                            }
                            print!("]");
                        }
                        print!("}}");
                    }
                    println!("]}}");
                } else {
                    println!("compile cache inventory");
                    println!("  workspace_root: {}", summary.workspace_root.display());
                    println!("  roots: {}", summary.roots.len());
                    println!("  entries: {}", summary.total_entries);
                    println!("  files: {}", summary.total_files);
                    println!("  bytes: {}", summary.total_bytes);
                    for inventory in summary.roots {
                        println!("  root: {}", inventory.root.display());
                        println!("    entries: {}", inventory.entry_count);
                        println!("    files: {}", inventory.total_files);
                        println!("    bytes: {}", inventory.total_bytes);
                        if verbose_cache {
                            for entry in inventory.entries {
                                println!(
                                    "    entry: {} files={} bytes={} dir={}",
                                    entry.key,
                                    entry.file_count,
                                    entry.total_bytes,
                                    entry.entry_dir.display()
                                );
                            }
                        }
                    }
                }
            } else {
                let input = input.expect("cache-status input must exist when --all is not set");
                let resolved = resolve_compile_input(&input)?;
                let status = cache::compile_cache_status_with_plan(
                    &input,
                    resolved.project.as_ref(),
                    resolved.project_plan.as_ref(),
                )?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_status\",\"input\":\"{}\",\"root\":\"{}\",\"key\":\"{}\",\"state\":\"{}\",\"entry_dir\":\"{}\",\"files\":{},\"bytes\":{},\"fingerprint_inputs\":{}",
                        json_escape(&input.display().to_string()),
                        json_escape(&status.root.display().to_string()),
                        json_escape(&status.key),
                        if status.entry_exists { "present" } else { "missing" },
                        json_escape(&status.entry_dir.display().to_string()),
                        status.file_count,
                        status.total_bytes,
                        status.input_labels.len()
                    );
                    if verbose_cache {
                        print!(",\"inputs\":[");
                        for (index, label) in status.input_labels.iter().enumerate() {
                            if index > 0 {
                                print!(",");
                            }
                            print!("\"{}\"", json_escape(label));
                        }
                        print!("]");
                    }
                    println!("}}");
                } else {
                    println!("compile cache status: {}", input.display());
                    println!("  root: {}", status.root.display());
                    println!("  key: {}", status.key);
                    println!(
                        "  state: {}",
                        if status.entry_exists {
                            "present"
                        } else {
                            "missing"
                        }
                    );
                    println!("  entry_dir: {}", status.entry_dir.display());
                    println!("  files: {}", status.file_count);
                    println!("  bytes: {}", status.total_bytes);
                    println!("  fingerprint_inputs: {}", status.input_labels.len());
                    if verbose_cache {
                        for label in status.input_labels {
                            println!("  input: {}", label);
                        }
                    }
                }
            }
        }
        CommandKind::CleanCache { input, all, json } => {
            if all {
                let workspace_root = std::env::current_dir()
                    .map_err(|error| format!("failed to resolve current directory: {error}"))?;
                let cleaned = cache::clean_compile_cache_summary(&workspace_root)?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_cleaned\",\"workspace_root\":\"{}\",\"cleaned_roots\":{},\"removed_entries\":{},\"removed_bytes\":{},\"roots\":[",
                        json_escape(&cleaned.workspace_root.display().to_string()),
                        cleaned.cleaned_roots.len(),
                        cleaned.removed_entries,
                        cleaned.removed_bytes
                    );
                    for (index, root) in cleaned.cleaned_roots.iter().enumerate() {
                        if index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"root\":\"{}\",\"removed_entries\":{},\"removed_bytes\":{}}}",
                            json_escape(&root.root.display().to_string()),
                            root.removed_entries,
                            root.removed_bytes
                        );
                    }
                    println!("]}}");
                } else {
                    println!("compile cache cleaned");
                    println!("  workspace_root: {}", cleaned.workspace_root.display());
                    println!("  cleaned_roots: {}", cleaned.cleaned_roots.len());
                    println!("  removed_entries: {}", cleaned.removed_entries);
                    println!("  removed_bytes: {}", cleaned.removed_bytes);
                    for root in cleaned.cleaned_roots {
                        println!("  root: {}", root.root.display());
                        println!("    removed_entries: {}", root.removed_entries);
                        println!("    removed_bytes: {}", root.removed_bytes);
                    }
                }
            } else {
                let input = input.expect("clean-cache input must exist when --all is not set");
                let resolved = resolve_compile_input(&input)?;
                let cleaned = cache::clean_compile_cache_with_plan(
                    &input,
                    resolved.project.as_ref(),
                    resolved.project_plan.as_ref(),
                )?;
                if json {
                    println!(
                        "{{\"kind\":\"compile_cache_cleaned\",\"input\":\"{}\",\"root\":\"{}\",\"removed_entries\":{},\"removed_bytes\":{}}}",
                        json_escape(&input.display().to_string()),
                        json_escape(&cleaned.root.display().to_string()),
                        cleaned.removed_entries,
                        cleaned.removed_bytes
                    );
                } else {
                    println!("compile cache cleaned: {}", input.display());
                    println!("  root: {}", cleaned.root.display());
                    println!("  removed_entries: {}", cleaned.removed_entries);
                    println!("  removed_bytes: {}", cleaned.removed_bytes);
                }
            }
        }
        CommandKind::PruneCache {
            input,
            all,
            keep,
            json,
        } => {
            if all {
                let workspace_root = std::env::current_dir()
                    .map_err(|error| format!("failed to resolve current directory: {error}"))?;
                let pruned = cache::prune_compile_cache_summary(&workspace_root, keep)?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_pruned\",\"workspace_root\":\"{}\",\"keep\":{},\"pruned_roots\":{},\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{},\"roots\":[",
                        json_escape(&pruned.workspace_root.display().to_string()),
                        keep,
                        pruned.pruned_roots.len(),
                        pruned.kept_entries,
                        pruned.removed_entries,
                        pruned.removed_bytes
                    );
                    for (index, root) in pruned.pruned_roots.iter().enumerate() {
                        if index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"root\":\"{}\",\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{}}}",
                            json_escape(&root.root.display().to_string()),
                            root.kept_entries,
                            root.removed_entries,
                            root.removed_bytes
                        );
                    }
                    println!("]}}");
                } else {
                    println!("compile cache pruned");
                    println!("  workspace_root: {}", pruned.workspace_root.display());
                    println!("  keep: {}", keep);
                    println!("  pruned_roots: {}", pruned.pruned_roots.len());
                    println!("  kept_entries: {}", pruned.kept_entries);
                    println!("  removed_entries: {}", pruned.removed_entries);
                    println!("  removed_bytes: {}", pruned.removed_bytes);
                    for root in pruned.pruned_roots {
                        println!("  root: {}", root.root.display());
                        println!("    kept_entries: {}", root.kept_entries);
                        println!("    removed_entries: {}", root.removed_entries);
                        println!("    removed_bytes: {}", root.removed_bytes);
                    }
                }
            } else {
                let input = input.expect("cache-prune input must exist when --all is not set");
                let resolved = resolve_compile_input(&input)?;
                let pruned = cache::prune_compile_cache_with_plan(
                    &input,
                    resolved.project.as_ref(),
                    resolved.project_plan.as_ref(),
                    keep,
                )?;
                if json {
                    println!(
                        "{{\"kind\":\"compile_cache_pruned\",\"input\":\"{}\",\"root\":\"{}\",\"keep\":{},\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{}}}",
                        json_escape(&input.display().to_string()),
                        json_escape(&pruned.root.display().to_string()),
                        keep,
                        pruned.kept_entries,
                        pruned.removed_entries,
                        pruned.removed_bytes
                    );
                } else {
                    println!("compile cache pruned: {}", input.display());
                    println!("  root: {}", pruned.root.display());
                    println!("  keep: {}", keep);
                    println!("  kept_entries: {}", pruned.kept_entries);
                    println!("  removed_entries: {}", pruned.removed_entries);
                    println!("  removed_bytes: {}", pruned.removed_bytes);
                }
            }
        }
        CommandKind::DumpAst { input } => {
            let compiled = compile_command_input(&input)?;
            print_project_context(&compiled.resolved);
            print!("{}", render::render_ast(&compiled.artifacts.ast));
        }
        CommandKind::DumpNir { input } => {
            let compiled = compile_command_input(&input)?;
            print_project_context(&compiled.resolved);
            print_required_nustar_context(&compiled.artifacts)?;
            print!("{}", render::render_nir(&compiled.artifacts.nir));
        }
        CommandKind::DumpYir { input } => {
            let compiled = compile_command_input(&input)?;
            print_project_context(&compiled.resolved);
            print_required_nustar_context(&compiled.artifacts)?;
            print!("{}", render::render_yir(&compiled.artifacts.yir));
        }
        CommandKind::Check { input } => {
            let resolved = resolve_compile_input(&input)?;
            let artifacts = resolved.compile()?;
            let benchmarks = collect_benchmark_inventory(&artifacts);
            if success_logs_enabled() {
                println!("checked nuis source: {}", input.display());
                if let Some(project) = &resolved.project {
                    println!("project: {}", project::describe_project(project));
                }
                if let Some(plan) = &resolved.project_plan {
                    println!(
                        "project_plan: {}",
                        project::describe_project_compilation_plan(plan)
                    );
                    println!(
                        "project_abi_graph: {}",
                        project::render_project_abi_graph_line(&plan.abi_resolution)
                    );
                }
                println!(
                    "loaded_nustar: {}",
                    artifacts
                        .loaded_nustar
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("nir_functions: {}", artifacts.nir.functions.len());
                println!("nir_benchmarks: {}", benchmarks.len());
                if !benchmarks.is_empty() {
                    println!(
                        "benchmark_symbols: {}",
                        benchmarks
                            .iter()
                            .map(|entry| entry.symbol.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                println!("yir_nodes: {}", artifacts.yir.nodes.len());
                println!("yir_edges: {}", artifacts.yir.edges.len());
                println!("llvm_ir_bytes: {}", artifacts.llvm_ir.len());
            }
        }
        CommandKind::Compile {
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
        } => {
            let resolved = resolve_compile_input(&input)?;
            let cpu_target = aot::resolve_cpu_build_target(
                Path::new("nustar-packages"),
                resolved
                    .project_plan
                    .as_ref()
                    .map(|plan| &plan.abi_resolution),
                cpu_abi.as_deref(),
                target.as_deref(),
            )?;
            let cache_key = cache::compute_compile_cache_key_with_plan(
                &input,
                resolved.project.as_ref(),
                resolved.project_plan.as_ref(),
            )?;
            let cache_hit = cache::lookup_compile_cache(&cache_key)?;
            let compile_fresh = || -> Result<(aot::CompileArtifacts, Vec<String>), String> {
                let artifacts =
                    resolved.compile_with_options(&pipeline::PipelineCompileOptions {
                        lowering_target: Some(
                            lowering::LoweringTargetConfig::from_cpu_build_target(&cpu_target),
                        ),
                    })?;
                let written = aot::write_and_link(
                    &resolved.effective_input_path,
                    &output_dir,
                    &artifacts.ast,
                    &artifacts.nir,
                    &artifacts.yir,
                    &artifacts.llvm_ir,
                    &cpu_target,
                )?;
                let _ = cache::store_compile_cache(&cache_key, &output_dir)?;
                Ok((written, artifacts.loaded_nustar))
            };
            let (written, loaded_nustar, used_cache_restore) = if let Some(entry) = &cache_hit {
                match cache::restore_compile_cache(entry, &output_dir).and_then(|_| {
                    aot::verify_build_manifest(&output_dir.join("nuis.build.manifest.toml"))
                }) {
                    Ok(restored_manifest) => {
                        let written = aot::compile_artifacts_for_output_dir_with_packaging_mode(
                            &resolved.effective_input_path,
                            &output_dir,
                            &restored_manifest.packaging_mode,
                        )?;
                        (written, restored_manifest.loaded_nustar, true)
                    }
                    Err(_) => {
                        let (written, loaded_nustar) = compile_fresh()?;
                        (written, loaded_nustar, false)
                    }
                }
            } else {
                let (written, loaded_nustar) = compile_fresh()?;
                (written, loaded_nustar, false)
            };
            let project_metadata =
                if let (Some(project), Some(plan)) = (&resolved.project, &resolved.project_plan) {
                    Some(project::write_project_metadata(&output_dir, project, plan)?)
                } else {
                    None
                };
            let project_text_handle_rewrite = resolved
                .project
                .as_ref()
                .map(project::summarize_project_text_handle_rewrites)
                .transpose()?;
            let doc_index = write_compile_doc_index(&input, &output_dir)?;
            let build_manifest = aot::write_build_manifest(
                &output_dir,
                &written,
                &aot::BuildManifestContext {
                    input_path: input.display().to_string(),
                    output_dir: output_dir.display().to_string(),
                    loaded_nustar: loaded_nustar.clone(),
                    compile_cache: Some(aot::BuildManifestCacheInfo {
                        status: if used_cache_restore {
                            "hit".to_owned()
                        } else {
                            "miss".to_owned()
                        },
                        key: cache_key.key.clone(),
                        root: cache_key.root.display().to_string(),
                    }),
                    project: resolved
                        .project
                        .as_ref()
                        .zip(resolved.project_plan.as_ref())
                        .map(|(project, plan)| aot::BuildManifestProjectInfo {
                            name: project.manifest.name.clone(),
                            abi_mode: if plan.abi_resolution.explicit {
                                "explicit".to_owned()
                            } else {
                                "auto-recommended".to_owned()
                            },
                            abi_graph_summary: Some(project::render_project_abi_graph_line(
                                &plan.abi_resolution,
                            )),
                            abi_entries: plan
                                .abi_resolution
                                .requirements
                                .iter()
                                .map(|item| (item.domain.clone(), item.abi.clone()))
                                .collect::<Vec<_>>(),
                            plan_summary: Some(project::describe_project_compilation_plan(plan)),
                            effective_input: Some(plan.effective_input_path.display().to_string()),
                            text_handle_rewrite_helper_hits: project_text_handle_rewrite
                                .map(|summary| summary.helper_hits)
                                .unwrap_or(0),
                            text_handle_rewrite_local_hits: project_text_handle_rewrite
                                .map(|summary| summary.local_hits)
                                .unwrap_or(0),
                            manifest_copy_path: project_metadata
                                .as_ref()
                                .map(|item| item.manifest_copy_path.clone()),
                            plan_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.plan_index_path.clone()),
                            organization_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.organization_index_path.clone()),
                            exchange_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.exchange_index_path.clone()),
                            modules_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.modules_index_path.clone()),
                            docs_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.docs_index_path.clone()),
                            docs_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.docs_summary.modules)
                                .unwrap_or(0),
                            docs_documented_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.docs_summary.documented_modules)
                                .unwrap_or(0),
                            docs_documented_item_count: project_metadata
                                .as_ref()
                                .map(|item| item.docs_summary.documented_items)
                                .unwrap_or(0),
                            imports_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.imports_index_path.clone()),
                            imports_library_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.libraries)
                                .unwrap_or(0),
                            imports_visible_library_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.visible_libraries)
                                .unwrap_or(0),
                            imports_visible_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.visible_modules)
                                .unwrap_or(0),
                            imports_documented_visible_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.documented_visible_modules)
                                .unwrap_or(0),
                            imports_documented_visible_item_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.documented_visible_items)
                                .unwrap_or(0),
                            galaxy_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_index_path.clone()),
                            galaxy_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.galaxies)
                                .unwrap_or(0),
                            galaxy_documented_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.documented_galaxies)
                                .unwrap_or(0),
                            galaxy_documented_library_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.documented_library_modules)
                                .unwrap_or(0),
                            galaxy_documented_item_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.documented_items)
                                .unwrap_or(0),
                            links_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.links_index_path.clone()),
                            packet_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.packet_index_path.clone()),
                            host_ffi_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.host_ffi_index_path.clone()),
                            abi_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.abi_index_path.clone()),
                        }),
                    doc_index: Some(doc_index.clone()),
                    cpu_target: cpu_target.clone(),
                },
            )?;
            if success_logs_enabled() {
                println!("compiled nuis source: {}", input.display());
                println!(
                    "compile_cache: {} ({})",
                    if used_cache_restore { "hit" } else { "miss" },
                    cache_key.key
                );
                println!("compile_cache_inputs: {}", cache_key.input_labels.len());
                if verbose_cache {
                    for label in &cache_key.input_labels {
                        println!("  compile_cache_input: {}", label);
                    }
                }
                if let Some(project) = &resolved.project {
                    println!("project: {}", project::describe_project(project));
                    if let Ok(graph) = project::describe_project_abi_graph(project) {
                        println!("project_abi_graph: {}", graph);
                    }
                }
                if let Some(plan) = &resolved.project_plan {
                    println!(
                        "project_plan: {}",
                        project::describe_project_compilation_plan(plan)
                    );
                    println!(
                        "project_abi_graph: {}",
                        project::render_project_abi_graph_line(&plan.abi_resolution)
                    );
                }
                println!(
                    "loaded_nustar: {}",
                    loaded_nustar
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("cpu_target_abi: {}", cpu_target.abi);
                println!(
                    "cpu_target_machine: {}-{}",
                    cpu_target.machine_arch, cpu_target.machine_os
                );
                println!("cpu_target_clang: {}", cpu_target.clang_target);
                println!(
                    "cpu_target_cross: {}",
                    if cpu_target.cross_compile {
                        "true"
                    } else {
                        "false"
                    }
                );
                if let Some(plan) = &resolved.project_plan {
                    for item in &plan.abi_resolution.requirements {
                        println!("abi: {}={}", item.domain, item.abi);
                        if let Ok(manifest) = registry::load_manifest_for_domain(
                            Path::new("nustar-packages"),
                            &item.domain,
                        ) {
                            if let Ok(target) =
                                registry::registered_abi_target(&manifest, &item.abi)
                            {
                                println!(
                                    "  abi_target_machine: {}-{}",
                                    target.machine_arch, target.machine_os
                                );
                                println!("  abi_target_object: {}", target.object_format);
                                println!("  abi_target_calling: {}", target.calling_abi);
                                println!("  abi_target_clang: {}", target.clang_target);
                                if let Some(backend) = target.backend_family {
                                    println!("  abi_target_backend: {}", backend);
                                }
                                if let Some(vendor) = target.vendor {
                                    println!("  abi_target_vendor: {}", vendor);
                                }
                                if let Some(device_class) = target.device_class {
                                    println!("  abi_target_device: {}", device_class);
                                }
                                println!(
                                    "  abi_target_host_adaptive: {}",
                                    if target.host_adaptive {
                                        "true"
                                    } else {
                                        "false"
                                    }
                                );
                            }
                        }
                    }
                }
                println!("ast: {}", written.ast_path);
                println!("nir: {}", written.nir_path);
                println!("yir: {}", written.yir_path);
                println!("llvm_ir: {}", written.llvm_ir_path);
                println!("packaging_mode: {}", written.packaging_mode);
                println!("binary: {}", written.binary_path);
                println!(
                    "compiled_artifact: {}",
                    output_dir.join("nuis.compiled.artifact").display()
                );
                println!("doc_index: {}", doc_index.path);
                println!("doc_index_modules: {}", doc_index.module_count);
                println!(
                    "doc_index_documented_items: {}",
                    doc_index.documented_item_count
                );
                println!("build_manifest: {}", build_manifest);
                if let Some(metadata) = &project_metadata {
                    println!("project_manifest: {}", metadata.manifest_copy_path);
                    println!("project_plan_index: {}", metadata.plan_index_path);
                    println!("project_organization: {}", metadata.organization_index_path);
                    println!("project_exchange: {}", metadata.exchange_index_path);
                    println!("project_modules: {}", metadata.modules_index_path);
                    println!("project_docs: {}", metadata.docs_index_path);
                    println!("project_imports: {}", metadata.imports_index_path);
                    println!("project_galaxy: {}", metadata.galaxy_index_path);
                    println!("project_links: {}", metadata.links_index_path);
                    println!("project_packet: {}", metadata.packet_index_path);
                    println!("project_host_ffi: {}", metadata.host_ffi_index_path);
                    println!("project_abi: {}", metadata.abi_index_path);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("nuisc_{label}_{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_temp_project_fixture(name: &str, manifest: &str, entry_source: &str) -> PathBuf {
        let root = temp_dir(name);
        fs::write(root.join("nuis.toml"), manifest).unwrap();
        fs::write(root.join("main.ns"), entry_source).unwrap();
        root
    }

    #[test]
    fn domain_contract_json_exposes_grouped_contract_sections() {
        let contract =
            registry::load_domain_contract_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
                .expect("expected network domain contract");
        let json = registry::domain_contract_json(&contract);

        assert!(json.contains("\"contract_schema\":\"nustar-domain-contract-v1\""));
        assert!(json.contains("\"contract\":{"));
        assert!(json.contains("\"schema\":\"nustar-domain-contract-v1\""));
        assert!(json.contains("\"groups\":[\"package_identity\""));
        assert!(json.contains("\"package_identity\":{"));
        assert!(json.contains("\"loader_contract\":{"));
        assert!(json.contains("\"abi_contract\":{"));
        assert!(json.contains("\"host_bridge_contract\":{"));
        assert!(json.contains("\"runtime_capability_contract\":{"));
        assert!(json.contains("\"capability_tags\":[\"io-reactor\""));
        assert!(json.contains("\"scheduler_contract\":{"));
        assert!(json.contains("\"std_net_extension\":{"));
        assert!(json.contains("\"domain\":\"network\""));
    }

    #[test]
    fn domain_registration_json_exposes_registration_section() {
        let registration = registry::load_registered_domains(Path::new(NUSTAR_REGISTRY_ROOT))
            .expect("expected registered domains")
            .into_iter()
            .find(|item| item.domain_family == "network")
            .expect("expected network registration");
        let json = registry::domain_registration_json(&registration);

        assert!(json.contains("\"registration\":{"));
        assert!(json.contains("\"manifest_path\":"));
        assert!(json.contains("\"entry_crate\":"));
        assert!(json.contains("\"ast_entry\":"));
        assert!(json.contains("\"nir_entry\":"));
        assert!(json.contains("\"yir_lowering_entry\":"));
        assert!(json.contains("\"part_verify_entry\":"));
        assert!(json.contains("\"ast_surface\":["));
        assert!(json.contains("\"nir_surface\":["));
        assert!(json.contains("\"ops\":["));
    }

    #[test]
    fn domain_build_contract_summary_json_exposes_grouped_sections() {
        let manifest =
            registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
                .expect("expected network manifest");
        let json =
            domain_build_contract_summary_json(&registry::domain_build_contract_summary(&manifest));

        assert!(json.contains("\"lowering\":{"));
        assert!(json.contains("\"backend\":{"));
        assert!(json.contains("\"bridge\":{"));
        assert!(json.contains("\"host_bridge\":{"));
        assert!(json.contains("\"lane_policy\":\"dispatch-lanes.io-bound\""));
        assert!(json.contains("\"bridge_entry\":\"nuis.network.bridge.dispatch.v1\""));
        assert!(json.contains("\"transport_model\":\"client-session\""));
        assert!(json.contains("\"phase_order\":[\"bind\",\"submit\",\"wait\",\"finalize\"]"));
        assert!(json.contains("\"bridge_plan_begin\":true"));
        assert!(json.contains("\"bridge_plan_end\":true"));
    }

    #[test]
    fn domain_registry_json_includes_effective_build_contract() {
        let registration = registry::load_registered_domains(Path::new(NUSTAR_REGISTRY_ROOT))
            .expect("expected registered domains")
            .into_iter()
            .find(|item| item.domain_family == "network")
            .expect("expected network registration");
        let manifest =
            registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
                .expect("expected network manifest");
        let json = domain_registry_json(&registration, &manifest);

        assert!(json.contains("\"registration\":{"));
        assert!(json.contains("\"build_contract\":{"));
        assert!(json.contains("\"backend\":{"));
        assert!(json.contains("\"host_bridge\":{"));
        assert!(json.contains("\"scheduler_binding\":\"network-poll-bridge\""));
        assert!(json.contains("\"host_ffi_surface\":\"socket,urlsession\""));
    }

    #[test]
    fn domain_build_unit_contract_json_includes_effective_build_contract() {
        let unit = aot::BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: Some("network.socket.macos.arm64.v1".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("urlsession".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("socket-io".to_owned()),
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
            packaging_role: "domain-sidecar".to_owned(),
        };
        let json = domain_build_unit_contract_json(&unit);

        assert!(json.contains("\"package_id\":\"official.network\""));
        assert!(json.contains("\"domain_family\":\"network\""));
        assert!(json.contains("\"build_contract\":{"));
        assert!(json.contains("\"lane_policy\":\"dispatch-lanes.io-bound\""));
        assert!(json.contains("\"bridge_entry\":\"nuis.network.bridge.dispatch.v1\""));
    }

    #[test]
    fn domain_build_contract_drift_check_accepts_current_registry_alignment() {
        let unit = aot::BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: Some("network.socket.macos.arm64.v1".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("urlsession".to_owned()),
            vendor: Some("apple".to_owned()),
            device_class: Some("socket-io".to_owned()),
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
            packaging_role: "domain-sidecar".to_owned(),
        };
        let drift = evaluate_domain_build_contract_drift(&unit);

        assert!(drift.consistent);
        assert!(drift.issues.is_empty());
    }

    #[test]
    fn domain_build_contract_drift_check_reports_registry_mismatch() {
        let unit = aot::BuildManifestDomainBuildUnit {
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            abi: Some("network.socket.macos.arm64.v1".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("imaginary-backend".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("imaginary-target".to_owned()),
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
            contract_family: "nustar.network.drifted".to_owned(),
            packaging_role: "domain-sidecar".to_owned(),
        };
        let drift = evaluate_domain_build_contract_drift(&unit);

        assert!(!drift.consistent);
        assert!(drift
            .issues
            .iter()
            .any(|issue| issue.contains("contract_family")));
        assert!(drift
            .issues
            .iter()
            .any(|issue| issue.contains("selected_lowering_target")));
        assert!(drift
            .issues
            .iter()
            .any(|issue| issue.contains("backend_family")));
    }

    #[test]
    fn domain_build_unit_verification_verdict_marks_cpu_unit_consistent() {
        let unit = aot::BuildManifestDomainBuildUnit {
            package_id: "official.cpu".to_owned(),
            domain_family: "cpu".to_owned(),
            abi: Some("cpu.arm64.apple_aapcs64".to_owned()),
            machine_arch: Some("arm64".to_owned()),
            machine_os: Some("darwin".to_owned()),
            backend_family: Some("llvm".to_owned()),
            vendor: None,
            device_class: None,
            selected_lowering_target: Some("llvm".to_owned()),
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
            contract_family: "nustar.cpu".to_owned(),
            packaging_role: "host-binary".to_owned(),
        };
        let report = aot::BuildManifestVerifyReport {
            schema: "nuis-build-manifest-v1".to_owned(),
            input: "main.ns".to_owned(),
            output_dir: "out".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            envelope_path: "out/nuis.executable.envelope.toml".to_owned(),
            envelope_schema: "nuis-executable-envelope-v1".to_owned(),
            envelope_package_count: 1,
            artifact_path: "out/nuis.compiled.artifact".to_owned(),
            artifact_schema: "nuis-compiled-artifact-v1".to_owned(),
            artifact_binary_name: "demo".to_owned(),
            artifact_binary_bytes: 1,
            lifecycle_schema: "nuis-lifecycle-contract-v1".to_owned(),
            lifecycle_bootstrap_entry: "main".to_owned(),
            lifecycle_tick_policy: "poll".to_owned(),
            lifecycle_shutdown_policy: "flush".to_owned(),
            lifecycle_yalivia_rpc: "disabled".to_owned(),
            lifecycle_hook_count: 0,
            lifecycle_hook_surface: Vec::new(),
            lifecycle_export_count: 0,
            lifecycle_export_surface: Vec::new(),
            lifecycle_runtime_capability_flags: Vec::new(),
            execution_contracts_checked: 1,
            domain_build_unit_count: 1,
            heterogeneous_domain_count: 0,
            domain_payload_blobs_checked: 0,
            domain_payload_blob_sections_checked: 0,
            domain_payload_contract_sections_checked: 0,
            domain_payload_lowering_plans_checked: 0,
            domain_payload_backend_stubs_checked: 0,
            domain_payload_bridge_plans_checked: 0,
            domain_bridge_stubs_checked: 0,
            domain_build_units: vec![unit.clone()],
            cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
            cpu_target_machine_arch: "arm64".to_owned(),
            cpu_target_machine_os: "darwin".to_owned(),
            cpu_target_object_format: "mach-o".to_owned(),
            cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
            cpu_target_clang: "aarch64-apple-darwin".to_owned(),
            cpu_target_cross: false,
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache_status: None,
            compile_cache_key: None,
            compile_cache_root: None,
            doc_index_path: None,
            doc_index_module_count: 0,
            doc_index_documented_item_count: 0,
            doc_index_checked: 0,
            project_text_handle_rewrite_helper_hits: 0,
            project_text_handle_rewrite_local_hits: 0,
            project_plan_index: None,
            project_docs_index: None,
            project_docs_module_count: 0,
            project_docs_documented_module_count: 0,
            project_docs_documented_item_count: 0,
            project_imports_index: None,
            project_imports_library_count: 0,
            project_imports_visible_library_count: 0,
            project_imports_visible_module_count: 0,
            project_imports_documented_visible_module_count: 0,
            project_imports_documented_visible_item_count: 0,
            project_galaxy_index: None,
            project_galaxy_count: 0,
            project_documented_galaxy_count: 0,
            project_documented_galaxy_library_module_count: 0,
            project_documented_galaxy_item_count: 0,
            project_packet_index: None,
            bridge_registry_path: None,
            bridge_registry_units: 0,
            bridge_registry_checked: 0,
            bridge_registry_entries_checked: 0,
            host_bridge_plan_index_path: None,
            host_bridge_plan_units: 0,
            host_bridge_plan_checked: 0,
            host_bridge_plan_entries_checked: 0,
            artifacts_checked: 0,
            project_metadata_checked: 0,
        };
        let verdict = domain_build_unit_verification_verdict(&unit, &report);

        assert_eq!(verdict.kind, "host");
        assert!(verdict.payload_blob_ok);
        assert!(verdict.bridge_registry_ok);
        assert!(verdict.host_bridge_plan_ok);
        assert!(verdict.registry_alignment_ok);
        assert!(verdict.failure_reasons.is_empty());
        assert!(verdict.consistent);
    }

    #[test]
    fn verify_build_manifest_json_includes_domain_build_contracts() {
        let project_name = "verify_build_manifest_contract_json";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "verify_build_manifest_contract_json"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        );
        let output_dir = temp_dir("verify_build_manifest_contract_json_outputs");

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let report = aot::verify_build_manifest(&manifest_path).unwrap();
        let json = verify_build_manifest_json(&manifest_path, &report);

        assert!(json.contains("\"domain_build_units\":["));
        assert!(json.contains("\"domain_build_contracts\":["));
        assert!(json.contains("\"domain_payload_blobs_checked\":0"));
        assert!(json.contains("\"domain_payload_blob_sections_checked\":0"));
        assert!(json.contains("\"domain_payload_lowering_plans_checked\":0"));
        assert!(json.contains("\"domain_payload_backend_stubs_checked\":0"));
        assert!(json.contains("\"domain_payload_bridge_plans_checked\":0"));
        assert!(json.contains("\"domain_bridge_stubs_checked\":0"));
        assert!(json.contains("\"bridge_registry_entries_checked\":0"));
        assert!(json.contains("\"host_bridge_plan_entries_checked\":0"));
        assert!(json.contains("\"doc_index_path\":"));
        assert!(json.contains("\"doc_index_module_count\":1"));
        assert!(json.contains("\"doc_index_documented_item_count\":0"));
        assert!(json.contains("\"doc_index_checked\":1"));
        assert!(json.contains("\"domain_build_contract_drift_checked\":"));
        assert!(json.contains("\"domain_build_contract_drift_mismatches\":0"));
        assert!(json.contains("\"domain_build_contracts_consistent\":true"));
        assert!(json.contains("\"domain_build_contract_drift\":["));
        assert!(json.contains("\"domain_build_unit_verdicts\":["));
        assert!(json.contains("\"domain_build_verification_summary\":{"));
        assert!(json.contains("\"all_units_consistent\":true"));
        assert!(json.contains("\"failing_units\":[]"));
        assert!(json.contains("\"kind\":\"host\""));
        assert!(json.contains("\"failure_reasons\":[]"));
        assert!(json.contains("\"registry_alignment_ok\":true"));
        assert!(json.contains("\"consistent\":true"));
        assert!(json.contains("\"package_id\":\"official.cpu\""));
        assert!(json.contains("\"build_contract\":{"));
    }

    #[test]
    fn inspect_artifact_json_includes_domain_build_contracts_when_manifest_is_available() {
        let project_name = "inspect_artifact_contract_json";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "inspect_artifact_contract_json"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        );
        let output_dir = temp_dir("inspect_artifact_contract_json_outputs");

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
        let report = aot::verify_build_manifest(&manifest_path).unwrap();
        let container = inspect_artifact_container_for_input(&manifest_path, Some(&report))
            .unwrap()
            .unwrap();
        let json =
            inspect_artifact_json(&manifest_path, &artifact, Some(&container), Some(&report));

        assert!(json.contains("\"domain_build_unit_count\":"));
        assert!(json.contains("\"domain_build_units\":["));
        assert!(json.contains("\"domain_build_contracts\":["));
        assert!(json.contains("\"domain_payload_blobs_checked\":0"));
        assert!(json.contains("\"domain_payload_blob_sections_checked\":0"));
        assert!(json.contains("\"domain_payload_lowering_plans_checked\":0"));
        assert!(json.contains("\"domain_payload_backend_stubs_checked\":0"));
        assert!(json.contains("\"domain_payload_bridge_plans_checked\":0"));
        assert!(json.contains("\"domain_bridge_stubs_checked\":0"));
        assert!(json.contains("\"bridge_registry_entries_checked\":0"));
        assert!(json.contains("\"host_bridge_plan_entries_checked\":0"));
        assert!(json.contains("\"domain_build_contract_drift_checked\":"));
        assert!(json.contains("\"domain_build_contract_drift_mismatches\":0"));
        assert!(json.contains("\"domain_build_contracts_consistent\":true"));
        assert!(json.contains("\"domain_build_contract_drift\":["));
        assert!(json.contains("\"domain_build_unit_verdicts\":["));
        assert!(json.contains("\"domain_build_verification_summary\":{"));
        assert!(json.contains("\"all_units_consistent\":true"));
        assert!(json.contains("\"failing_units\":[]"));
        assert!(json.contains("\"kind\":\"host\""));
        assert!(json.contains("\"failure_reasons\":[]"));
        assert!(json.contains("\"registry_alignment_ok\":true"));
        assert!(json.contains("\"consistent\":true"));
        assert!(json.contains("\"package_id\":\"official.cpu\""));
        assert!(json.contains("\"link_plan\":{"));
        assert!(json.contains("\"final_stage_driver\":\"clang\""));
        assert!(json.contains("\"final_stage_kind\":\"host-native-link\""));
        assert!(json.contains("\"final_stage_link_mode\":\"host-toolchain-finalize\""));
        assert!(json.contains("\"artifact_container_kind\":\"compiled-artifact-v1\""));
        assert!(json.contains("\"artifact_container_version\":1"));
        assert!(json.contains("\"artifact_section_table_valid\":true"));
        assert!(json.contains("\"link_plan\":{\"schema\":\"nuis-link-plan-v1\""));
        assert!(json.contains("\"artifact_section_count\":0"));
    }

    #[test]
    fn inspect_artifact_json_accepts_section_table_artifact_container() {
        let project_name = "inspect_artifact_v2_section_table_json";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "inspect_artifact_v2_section_table_json"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 7;
              }
            }
            "#,
        );
        let output_dir = temp_dir("inspect_artifact_v2_section_table_outputs");

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
        let v2_path = output_dir.join("nuis.compiled.v2.artifact");
        let v2_bytes = aot::encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
        std::fs::write(&v2_path, v2_bytes).unwrap();

        let decoded = load_nuis_compiled_artifact(&v2_path).unwrap();
        let container = inspect_artifact_container_for_input(&v2_path, None)
            .unwrap()
            .unwrap();
        let json = inspect_artifact_json(&v2_path, &decoded, Some(&container), None);
        let verify_report = aot::verify_nuis_compiled_artifact(&v2_path).unwrap();
        let verify_json = verify_artifact_json(&v2_path, &verify_report);

        assert_eq!(decoded.binary_name, artifact.binary_name);
        assert!(json.contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\""));
        assert!(json.contains("\"artifact_container_version\":2"));
        assert!(json.contains("\"artifact_section_count\":6"));
        assert!(json.contains("\"metadata_toml\""));
        assert!(json.contains("\"envelope_binary\""));
        assert!(json.contains("\"lifecycle_toml\""));
        assert!(json.contains("\"build_manifest_toml\""));
        assert!(json.contains("\"lowering_index_toml\""));
        assert!(json.contains("\"host_binary\""));
        assert!(json.contains("\"artifact_section_table_valid\":true"));
        assert!(verify_json
            .contains("\"artifact_container_kind\":\"compiled-artifact-section-table-v2\""));
        assert!(verify_json.contains("\"artifact_container_version\":2"));
        assert!(verify_json.contains("\"artifact_section_count\":6"));
        assert!(verify_json.contains("\"artifact_roundtrip_verified\":true"));
    }

    #[test]
    fn artifact_report_json_includes_top_level_verification_summary() {
        let project_name = "artifact_report_contract_summary_json";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "artifact_report_contract_summary_json"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        );
        let output_dir = temp_dir("artifact_report_contract_summary_json_outputs");

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
        let artifact_verify =
            aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
                .unwrap();
        let manifest_verify = aot::verify_build_manifest(&manifest_path).unwrap();
        let json = artifact_report_json(
            &manifest_path,
            &artifact,
            output_dir.join("nuis.compiled.artifact").as_path(),
            &artifact_verify,
            &manifest_path,
            &manifest_verify,
            false,
        );

        assert!(json.contains("\"domain_build_verification_summary\":{"));
        assert!(json.contains("\"all_units_consistent\":true"));
        assert!(json.contains("\"host_units_checked\":1"));
        assert!(json.contains("\"hetero_units_checked\":0"));
        assert!(json.contains("\"failing_units\":[]"));
        assert!(json.contains("\"execution_inspect\":{"));
        assert!(json.contains("\"kind\":\"nuis_execution_inspect\""));
        assert!(json.contains("\"heterogeneous_execution_domains\":0"));
        assert!(json.contains("\"execution_inspect\":{\"kind\":\"nuis_execution_inspect\""));
        assert!(json.contains("\"issues\":[]"));
        assert!(json.contains("\"project_metadata\":{"));
        assert!(json.contains("\"kind\":\"nuis_project_metadata\""));
        assert!(json.contains("\"source_kind\":\"build-manifest\""));
        assert!(json.contains("\"sections\":[]"));
        assert!(json.contains("\"doc_index\":{"));
        assert!(json.contains("\"kind\":\"nuis_doc_index\""));
        assert!(json.contains("\"module_count\":1"));
        assert!(json.contains("\"link_plan\":{"));
        assert!(json.contains("\"final_stage_driver\":\"clang\""));
    }

    #[test]
    fn benchmark_report_file_tooling_outputs_support_inspect_and_verify_json() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_file_demo",
        );
        let output_dir = temp_dir("benchmark_report_file_artifact_json_outputs");

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let artifact_path = output_dir.join("nuis.compiled.artifact");
        let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
        let manifest_verify = aot::verify_build_manifest(&manifest_path).unwrap();
        let artifact_verify = aot::verify_nuis_compiled_artifact(&artifact_path).unwrap();

        let container =
            inspect_artifact_container_for_input(&manifest_path, Some(&manifest_verify))
                .unwrap()
                .unwrap();
        let inspect_json = inspect_artifact_json(
            &manifest_path,
            &artifact,
            Some(&container),
            Some(&manifest_verify),
        );
        assert!(inspect_json.contains("\"kind\":\"nuis_artifact_inspect\""));
        assert!(inspect_json.contains("\"binary_name\":\"benchmark_report_file_demo\""));
        assert!(inspect_json.contains("\"packaging_mode\":\"native-cpu-llvm\""));
        assert!(inspect_json.contains("\"artifact_container_kind\":\"compiled-artifact-v1\""));
        assert!(inspect_json.contains("\"domain_build_units\":["));
        assert!(inspect_json.contains("\"domain_build_contracts\":["));
        assert!(inspect_json.contains("\"link_plan\":{"));
        assert!(inspect_json.contains("\"artifact_container_version\":1"));
        assert!(inspect_json.contains("\"artifact_section_count\":0"));
        assert!(inspect_json.contains("\"final_stage_driver\":\"clang\""));

        let verify_manifest_json = verify_build_manifest_json(&manifest_path, &manifest_verify);
        assert!(verify_manifest_json.contains("\"kind\":\"nuis_build_manifest_verify\""));
        assert!(verify_manifest_json
            .contains("\"artifact_binary_name\":\"benchmark_report_file_demo\""));
        assert!(verify_manifest_json.contains("\"project_metadata_checked\":"));
        assert!(verify_manifest_json.contains("\"domain_build_verification_summary\":{"));
        assert!(verify_manifest_json.contains("\"all_units_consistent\":true"));

        let verify_artifact_json_text = verify_artifact_json(&artifact_path, &artifact_verify);
        assert!(verify_artifact_json_text.contains("\"kind\":\"nuis_artifact_verify\""));
        assert!(
            verify_artifact_json_text.contains("\"binary_name\":\"benchmark_report_file_demo\"")
        );
        assert!(verify_artifact_json_text
            .contains("\"artifact_container_kind\":\"compiled-artifact-v1\""));
        assert!(verify_artifact_json_text.contains("\"artifact_container_version\":1"));
        assert!(verify_artifact_json_text.contains("\"artifact_section_count\":0"));
        assert!(verify_artifact_json_text.contains("\"artifact_roundtrip_verified\":true"));
        assert!(verify_artifact_json_text.contains("\"lifecycle_contract_consistent\":true"));

        let artifact_report = artifact_report_json(
            &manifest_path,
            &artifact,
            &artifact_path,
            &artifact_verify,
            &manifest_path,
            &manifest_verify,
            false,
        );
        assert!(artifact_report.contains("\"kind\":\"nuis_artifact_report\""));
        assert!(artifact_report.contains("\"manifest_verify_reconstructed\":false"));
        assert!(artifact_report.contains("\"execution_inspect\":{"));
        assert!(artifact_report.contains("\"kind\":\"nuis_execution_inspect\""));
        assert!(artifact_report.contains("\"sections\":[]"));
        assert!(artifact_report.contains("\"project_metadata\":{"));
        assert!(artifact_report.contains("\"kind\":\"nuis_project_metadata\""));
        assert!(artifact_report.contains("\"doc_index\":{"));
        assert!(artifact_report.contains("\"kind\":\"nuis_doc_index\""));
        assert!(artifact_report.contains("\"artifact_inspect\":{"));
        assert!(artifact_report.contains("\"artifact_verify\":{"));
        assert!(artifact_report.contains("\"manifest_verify\":{"));
        assert!(artifact_report.contains("\"binary_name\":\"benchmark_report_file_demo\""));
        assert!(artifact_report.contains("\"all_units_consistent\":true"));
    }

    #[test]
    fn inspect_project_metadata_json_reports_source_project_summaries() {
        let project_name = "inspect_project_metadata_source_json";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "inspect_project_metadata_source_json"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["pixelmagic=workspace"]
"#
            .trim_start(),
            r#"
            use cpu PixelMagicContracts;

            mod cpu Main {
              fn main() -> i64 {
                return PixelMagicContracts.blur_op_kind();
              }
            }
            "#,
        );
        let summary = inspect_project_metadata(&project_root).unwrap();
        let json = inspect_project_metadata_json(&summary);
        assert!(json.contains("\"kind\":\"nuis_project_metadata\""));
        assert!(json.contains("\"source_kind\":\"project-source\""));
        assert!(json.contains("\"project_name\":\"inspect_project_metadata_source_json\""));
        assert!(json.contains("\"imports_library_count\":8"));
        assert!(json.contains("\"galaxy_count\":3"));
    }

    #[test]
    fn inspect_project_metadata_output_dir_reports_build_output_summary() {
        let project_root = write_temp_project_fixture(
            "inspect_project_metadata_output_dir",
            r#"
name = "inspect_project_metadata_output_dir"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 7;
  }
}
"#,
        );
        let output_dir = temp_dir("inspect_project_metadata_output_dir_outputs");
        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let summary = inspect_project_metadata(&output_dir).unwrap();
        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let artifact_path = output_dir.join("nuis.compiled.artifact");
        assert_eq!(summary.source_kind, "build-output-dir");
        assert_eq!(
            summary.build_manifest_path.as_deref(),
            Some(manifest_path.to_string_lossy().as_ref())
        );
        assert_eq!(
            summary.artifact_path.as_deref(),
            Some(artifact_path.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn project_metadata_render_helpers_expose_summary_and_paths() {
        let summary = ProjectMetadataSummary {
            source_kind: "build-manifest".to_owned(),
            project_name: Some("demo".to_owned()),
            project_root: Some("/tmp/demo".to_owned()),
            manifest_path: Some("/tmp/demo/nuis.toml".to_owned()),
            build_manifest_path: Some("/tmp/demo/build/nuis.build.manifest.toml".to_owned()),
            artifact_path: Some("/tmp/demo/build/nuis.compiled.artifact".to_owned()),
            docs_index_path: Some("/tmp/demo/build/nuis.project.docs.txt".to_owned()),
            docs_module_count: 4,
            docs_documented_module_count: 3,
            docs_documented_item_count: 12,
            imports_index_path: Some("/tmp/demo/build/nuis.project.imports.txt".to_owned()),
            imports_library_count: 6,
            imports_visible_library_count: 5,
            imports_visible_module_count: 7,
            imports_documented_visible_module_count: 4,
            imports_documented_visible_item_count: 10,
            galaxy_index_path: Some("/tmp/demo/build/nuis.project.galaxy.txt".to_owned()),
            galaxy_count: 3,
            documented_galaxy_count: 2,
            documented_galaxy_library_module_count: 5,
            documented_galaxy_item_count: 10,
        };
        let compact = render_project_metadata_compact_summary(&summary);
        assert!(compact.contains("source_kind=build-manifest"));
        assert!(compact.contains("project=demo"));
        assert!(compact.contains("docs=4/3/12"));
        assert!(compact.contains("imports=6/5/7/4/10"));
        assert!(compact.contains("galaxies=3/2/5/10"));

        let paths = render_project_metadata_paths(&summary);
        assert!(paths.contains("project_root=/tmp/demo"));
        assert!(paths.contains("manifest_path=/tmp/demo/nuis.toml"));
        assert!(paths.contains("build_manifest_path=/tmp/demo/build/nuis.build.manifest.toml"));
        assert!(paths.contains("artifact_path=/tmp/demo/build/nuis.compiled.artifact"));
        assert!(paths.contains("docs_index_path=/tmp/demo/build/nuis.project.docs.txt"));
        assert!(paths.contains("imports_index_path=/tmp/demo/build/nuis.project.imports.txt"));
        assert!(paths.contains("galaxy_index_path=/tmp/demo/build/nuis.project.galaxy.txt"));
    }

    #[test]
    fn repair_project_metadata_target_rejects_non_manifest_inputs() {
        let error = repair_project_metadata_target(Path::new("examples/demo")).unwrap_err();
        assert!(error.contains("usage: nuisc repair-project-metadata"));
    }

    #[test]
    fn resolve_build_manifest_path_accepts_output_dir() {
        let output_dir = temp_dir("resolve_build_manifest_path_accepts_output_dir");
        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        fs::write(&manifest_path, "schema = \"demo\"\n").unwrap();
        let resolved = resolve_build_manifest_path(&output_dir).unwrap();
        assert_eq!(resolved, manifest_path);
    }

    #[test]
    fn repair_project_metadata_target_reports_missing_original_input() {
        let project_root = write_temp_project_fixture(
            "repair_project_metadata_missing_input",
            r#"
name = "repair_project_metadata_missing_input"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let output_dir = temp_dir("repair_project_metadata_missing_input_outputs");
        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();
        fs::remove_dir_all(&project_root).unwrap();
        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let error = repair_project_metadata_target(&manifest_path).unwrap_err();
        assert!(error.contains("cannot repair project metadata"));
        assert!(error.contains("no longer exists"));
        assert!(error.contains("inspect-project-metadata"));
    }

    #[test]
    fn repair_project_metadata_target_resolves_manifest_to_input_and_output_dir() {
        let project_root = write_temp_project_fixture(
            "repair_project_metadata_target_resolves",
            r#"
name = "repair_project_metadata_target_resolves"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let output_dir = temp_dir("repair_project_metadata_target_resolves_outputs");
        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();
        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let (resolved_input, resolved_output_dir) =
            repair_project_metadata_target(&manifest_path).unwrap();
        assert_eq!(resolved_input, project_root);
        assert_eq!(resolved_output_dir, output_dir);
    }

    #[test]
    fn repair_project_metadata_target_accepts_output_dir() {
        let project_root = write_temp_project_fixture(
            "repair_project_metadata_target_output_dir",
            r#"
name = "repair_project_metadata_target_output_dir"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        );
        let output_dir = temp_dir("repair_project_metadata_target_output_dir_outputs");
        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();
        let (resolved_input, resolved_output_dir) =
            repair_project_metadata_target(&output_dir).unwrap();
        assert_eq!(resolved_input, project_root);
        assert_eq!(resolved_output_dir, output_dir);
    }

    #[test]
    fn artifact_report_summary_lines_expose_compact_overview() {
        let artifact_verify = aot::NuisCompiledArtifactVerifyReport {
            schema: "nuis-compiled-artifact-v1".to_owned(),
            artifact_container_kind: "compiled-artifact-v1".to_owned(),
            artifact_container_version: 1,
            artifact_section_count: 0,
            artifact_section_names: Vec::new(),
            artifact_section_table_valid: true,
            packaging_mode: "native-cpu-llvm".to_owned(),
            binary_name: "demo".to_owned(),
            binary_bytes: 1,
            build_manifest_bytes: 1,
            envelope_schema: "nuis-executable-envelope-v1".to_owned(),
            envelope_package_count: 1,
            lifecycle_schema: "nuis-lifecycle-contract-v1".to_owned(),
            lifecycle_bootstrap_entry: "main".to_owned(),
            lifecycle_tick_policy: "poll".to_owned(),
            lifecycle_shutdown_policy: "flush".to_owned(),
            lifecycle_yalivia_rpc: "disabled".to_owned(),
            lifecycle_hook_count: 0,
            lifecycle_hook_surface: Vec::new(),
            lifecycle_export_count: 0,
            lifecycle_export_surface: Vec::new(),
            lifecycle_runtime_capability_flags: Vec::new(),
            lifecycle_contract_consistent: true,
            lifecycle_runtime_capability_flags_consistent: true,
            execution_contracts_checked: 1,
            cpu_target_abi: "cpu.arm64.apple_aapcs64".to_owned(),
            cpu_target_machine_arch: "arm64".to_owned(),
            cpu_target_machine_os: "darwin".to_owned(),
            cpu_target_object_format: "mach-o".to_owned(),
            cpu_target_calling_abi: "aapcs64-darwin".to_owned(),
            artifact_roundtrip_verified: true,
        };
        let summary = DomainBuildVerificationSummary {
            all_units_consistent: true,
            total_units: 1,
            host_units_checked: 1,
            hetero_units_checked: 0,
            registry_drift_units: 0,
            failing_units: Vec::new(),
        };
        let link_plan = linker::LinkPlan {
            schema: linker::LINK_PLAN_SCHEMA.to_owned(),
            input: "main.ns".to_owned(),
            output_dir: "out".to_owned(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            cpu_target: linker::LinkPlanCpuTarget {
                abi: "cpu.arm64.apple_aapcs64".to_owned(),
                machine_arch: "arm64".to_owned(),
                machine_os: "darwin".to_owned(),
                object_format: "mach-o".to_owned(),
                calling_abi: "aapcs64-darwin".to_owned(),
                clang_target: "aarch64-apple-darwin".to_owned(),
                cross_compile: false,
            },
            lifecycle: linker::LinkPlanLifecycle {
                bootstrap_entry: "main".to_owned(),
                tick_policy: "poll".to_owned(),
                shutdown_policy: "flush".to_owned(),
                yalivia_rpc: "disabled".to_owned(),
                hook_surface: Vec::new(),
                export_surface: Vec::new(),
                runtime_capability_flags: Vec::new(),
            },
            envelope: linker::LinkPlanEnvelope {
                schema: "nuis-executable-envelope-v1".to_owned(),
                package_count: 1,
                contract_families: vec!["nustar.cpu".to_owned()],
                domain_families: vec!["cpu".to_owned()],
                function_kind: "federated-function".to_owned(),
                graph_kind: "federated-graph".to_owned(),
                default_time_mode: "global".to_owned(),
            },
            compiled_artifact: linker::LinkPlanArtifact {
                path: "out/nuis.compiled.artifact".to_owned(),
                binary_name: "demo".to_owned(),
                binary_path: "out/demo".to_owned(),
                binary_bytes: 1,
                build_manifest_bytes: 1,
                container_kind: Some("compiled-artifact-v1".to_owned()),
                container_version: Some(1),
                section_count: Some(0),
                section_names: Vec::new(),
                section_table_valid: Some(true),
            },
            bridge_registry_path: None,
            host_bridge_plan_index_path: None,
            domain_units: vec![linker::LinkPlanDomainUnit {
                kind: "host".to_owned(),
                package_id: "official.cpu".to_owned(),
                domain_family: "cpu".to_owned(),
                abi: Some("cpu.arm64.apple_aapcs64".to_owned()),
                machine_arch: Some("arm64".to_owned()),
                machine_os: Some("darwin".to_owned()),
                backend_family: Some("llvm".to_owned()),
                vendor: None,
                device_class: None,
                selected_lowering_target: Some("llvm".to_owned()),
                contract_family: "nustar.cpu".to_owned(),
                packaging_role: "host-binary".to_owned(),
                artifact_stub_path: None,
                artifact_stub_inline: None,
                artifact_payload_path: None,
                artifact_bridge_stub_path: None,
                artifact_bridge_stub_inline: None,
                artifact_payload_blob_path: None,
                artifact_payload_blob_bytes: None,
                artifact_payload_format: None,
                artifact_payload_blob_inline: None,
            }],
            final_stage: linker::LinkPlanFinalStage {
                kind: "host-native-link".to_owned(),
                driver: "clang".to_owned(),
                link_mode: "host-toolchain-finalize".to_owned(),
                output_path: "out/demo".to_owned(),
                inputs: vec![
                    "out/nuis.compiled.artifact".to_owned(),
                    "out/nuis.executable.envelope.toml".to_owned(),
                ],
                notes: vec!["demo".to_owned()],
            },
        };
        let execution_overview = ExecutionInspectOverview {
            heterogeneous_domains: 1,
            domains: vec![ExecutionInspectDomainOverview {
                domain_family: "network".to_owned(),
                selected_lowering_target: Some("urlsession.socket-io".to_owned()),
                phase_count: 4,
                event_count: 4,
                resource_keys: vec![
                    "active_response".to_owned(),
                    "active_session".to_owned(),
                    "request_packet".to_owned(),
                ],
                output_handles: vec![
                    "response.handle".to_owned(),
                    "session.handle".to_owned(),
                    "status.code".to_owned(),
                    "task.handle".to_owned(),
                ],
            }],
        };
        let lines = artifact_report_summary_lines(
            &artifact_verify,
            &summary,
            Some(&link_plan),
            false,
            Some(&execution_overview),
            Some(&[frontend::AstDocIndex {
                module_path: "cpu.Main".to_owned(),
                items: vec![frontend::AstDocIndexItem {
                    kind: "function".to_owned(),
                    path: "cpu.Main::main".to_owned(),
                    docs: vec!["entry docs".to_owned()],
                    signature: Some("fn main() -> i64".to_owned()),
                }],
            }]),
            None,
        );

        assert_eq!(lines.len(), 7);
        assert!(lines[0].contains("artifact_roundtrip=ok"));
        assert!(lines[0].contains("lifecycle=ok"));
        assert!(lines[0].contains("runtime_flags=ok"));
        assert!(lines[0].contains("all_units_consistent=true"));
        assert!(lines[1].contains("total=1"));
        assert!(lines[1].contains("host=1"));
        assert!(lines[1].contains("hetero=0"));
        assert!(lines[1].contains("drift=0"));
        assert!(lines[1].contains("failing=<none>"));
        assert_eq!(lines[2], "summary_manifest: reconstructed=false");
        assert!(lines[3].contains("final_stage=host-native-link"));
        assert!(lines[3].contains("driver=clang"));
        assert!(lines[4].contains("summary_execution: hetero_domains=1"));
        assert!(lines[4].contains("network(target=urlsession.socket-io phases=4 events=4)"));
        assert_eq!(lines[5], "summary_execution_issues: <none>");
        assert_eq!(
            lines[6],
            "summary_docs: modules=1 documented_items=1 documented_modules=cpu.Main"
        );
    }

    #[test]
    fn execution_inspect_issues_flag_missing_target_and_phase_mismatch() {
        let overview = ExecutionInspectOverview {
            heterogeneous_domains: 2,
            domains: vec![
                ExecutionInspectDomainOverview {
                    domain_family: "network".to_owned(),
                    selected_lowering_target: None,
                    phase_count: 0,
                    event_count: 0,
                    resource_keys: vec![],
                    output_handles: vec![],
                },
                ExecutionInspectDomainOverview {
                    domain_family: "shader".to_owned(),
                    selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
                    phase_count: 4,
                    event_count: 3,
                    resource_keys: vec!["shader_buffer".to_owned()],
                    output_handles: vec![],
                },
            ],
        };

        let issues = execution_inspect_issues(&overview);

        assert_eq!(
            issues,
            vec![
                ExecutionInspectIssue {
                    domain_family: "network".to_owned(),
                    issue: "missing_target".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "network".to_owned(),
                    issue: "zero_phases".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "network".to_owned(),
                    issue: "missing_network_request_packet".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "network".to_owned(),
                    issue: "missing_network_active_response".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "network".to_owned(),
                    issue: "missing_network_response_handle".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "shader".to_owned(),
                    issue: "phase_event_mismatch(4->3)".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "shader".to_owned(),
                    issue: "missing_shader_frame_target".to_owned(),
                },
                ExecutionInspectIssue {
                    domain_family: "shader".to_owned(),
                    issue: "missing_shader_draw_handle".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn compile_command_writes_end_to_end_project_outputs() {
        let project_name = "compile_command_smoke";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "compile_command_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        );
        let output_dir = temp_dir("compile_command_outputs");
        let output_stem = project_name.to_owned();

        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        for path in [
            output_dir.join(format!("{output_stem}.ast.txt")),
            output_dir.join(format!("{output_stem}.nir.txt")),
            output_dir.join(format!("{output_stem}.yir")),
            output_dir.join(format!("{output_stem}.ll")),
            output_dir.join(&output_stem),
            output_dir.join("nuis.doc-index.json"),
            output_dir.join("nuis.build.manifest.toml"),
            output_dir.join("nuis.executable.envelope.toml"),
            output_dir.join("nuis.compiled.artifact"),
            output_dir.join("nuis.project.toml"),
            output_dir.join("nuis.project.plan.txt"),
            output_dir.join("nuis.project.organization.txt"),
            output_dir.join("nuis.project.exchange.txt"),
            output_dir.join("nuis.project.modules.txt"),
            output_dir.join("nuis.project.docs.txt"),
            output_dir.join("nuis.project.imports.txt"),
            output_dir.join("nuis.project.galaxy.txt"),
            output_dir.join("nuis.project.links.txt"),
            output_dir.join("nuis.project.packet.txt"),
            output_dir.join("nuis.project.host_ffi.txt"),
            output_dir.join("nuis.project.abi.txt"),
        ] {
            assert!(path.exists(), "expected output `{}`", path.display());
        }

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let manifest_text = fs::read_to_string(&manifest_path).unwrap();
        assert!(manifest_text.contains("manifest_schema = \"nuis-build-manifest-v1\""));
        assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
        assert!(manifest_text.contains("loaded_nustar = [\"official.cpu\"]"));
        assert!(manifest_text.contains("doc_index_path = "));
        assert!(manifest_text.contains("doc_index_module_count = 1"));
        assert!(manifest_text.contains("doc_index_documented_item_count = 0"));
        assert!(manifest_text.contains("[[domain_build_unit]]"));
        assert!(manifest_text.contains(&format!("name = \"{project_name}\"")));
        assert!(manifest_text.contains("manifest_copy = "));
        assert!(manifest_text.contains("plan_index = "));
        assert!(manifest_text.contains("organization_index = "));
        assert!(manifest_text.contains("exchange_index = "));
        assert!(manifest_text.contains("modules_index = "));
        assert!(manifest_text.contains("docs_index = "));
        assert!(manifest_text.contains("docs_module_count = 1"));
        assert!(manifest_text.contains("docs_documented_module_count = 0"));
        assert!(manifest_text.contains("docs_documented_item_count = 0"));
        assert!(manifest_text.contains("imports_index = "));
        assert!(manifest_text.contains("imports_library_count = 0"));
        assert!(manifest_text.contains("imports_visible_library_count = 0"));
        assert!(manifest_text.contains("imports_visible_module_count = 1"));
        assert!(manifest_text.contains("imports_documented_visible_module_count = 0"));
        assert!(manifest_text.contains("imports_documented_visible_item_count = 0"));
        assert!(manifest_text.contains("galaxy_index = "));
        assert!(manifest_text.contains("galaxy_count = 0"));
        assert!(manifest_text.contains("documented_galaxy_count = 0"));
        assert!(manifest_text.contains("documented_galaxy_library_module_count = 0"));
        assert!(manifest_text.contains("documented_galaxy_item_count = 0"));
        assert!(manifest_text.contains("links_index = "));
        assert!(manifest_text.contains("packet_index = "));
        assert!(manifest_text.contains("host_ffi_index = "));
        assert!(manifest_text.contains("abi_index = "));

        let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
        assert!(manifest_report
            .doc_index_path
            .as_deref()
            .is_some_and(|path| path.ends_with("nuis.doc-index.json")));
        assert_eq!(manifest_report.doc_index_module_count, 1);
        assert_eq!(manifest_report.doc_index_documented_item_count, 0);
        assert_eq!(manifest_report.doc_index_checked, 1);
        assert_eq!(manifest_report.project_docs_module_count, 1);
        assert_eq!(manifest_report.project_docs_documented_module_count, 0);
        assert_eq!(manifest_report.project_docs_documented_item_count, 0);
        assert_eq!(manifest_report.project_imports_library_count, 0);
        assert_eq!(manifest_report.project_imports_visible_library_count, 0);
        assert_eq!(manifest_report.project_imports_visible_module_count, 1);
        assert_eq!(
            manifest_report.project_imports_documented_visible_module_count,
            0
        );
        assert_eq!(
            manifest_report.project_imports_documented_visible_item_count,
            0
        );
        assert_eq!(manifest_report.project_galaxy_count, 0);
        assert_eq!(manifest_report.project_documented_galaxy_count, 0);
        assert_eq!(
            manifest_report.project_documented_galaxy_library_module_count,
            0
        );
        assert_eq!(manifest_report.project_documented_galaxy_item_count, 0);
        assert_eq!(
            manifest_report.envelope_schema,
            "nuis-executable-envelope-v1"
        );
        assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
        assert_eq!(manifest_report.artifact_binary_name, output_stem);
        assert!(Path::new(&manifest_report.envelope_path).exists());
        assert!(Path::new(&manifest_report.artifact_path).exists());
        assert!(manifest_report.project_metadata_checked >= 2);

        let artifact_report =
            aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
                .unwrap();
        assert_eq!(artifact_report.binary_name, output_stem);
        assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
        assert!(artifact_report.lifecycle_contract_consistent);
        assert!(artifact_report.artifact_roundtrip_verified);
    }

    #[test]
    fn compile_command_reuses_cached_project_outputs_without_recompiling() {
        let project_name = "compile_command_cache_hit_smoke";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "compile_command_cache_hit_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return 7;
              }
            }
            "#,
        );
        let output_dir = temp_dir("compile_command_cache_hit_outputs");

        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let first_report = aot::verify_build_manifest(&manifest_path).unwrap();
        assert_eq!(first_report.compile_cache_status.as_deref(), Some("miss"));
        assert_eq!(first_report.loaded_nustar, vec!["official.cpu".to_owned()]);

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        let second_report = aot::verify_build_manifest(&manifest_path).unwrap();
        assert_eq!(second_report.compile_cache_status.as_deref(), Some("hit"));
        assert_eq!(second_report.loaded_nustar, vec!["official.cpu".to_owned()]);
        assert_eq!(second_report.packaging_mode, "native-cpu-llvm");
        assert!(Path::new(&second_report.artifact_path).exists());
    }

    #[test]
    fn compile_command_writes_host_file_ffi_project_outputs() {
        let project_name = "compile_command_host_file_smoke";
        let project_root = write_temp_project_fixture(
            project_name,
            r#"
name = "compile_command_host_file_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
            .trim_start(),
            r#"
            mod cpu Main {
              extern "c" fn host_file_open(path_handle: i64, flags: i64) -> i64;
              extern "c" fn host_file_read(file_handle: i64, buffer_handle: i64, len: i64) -> i64;
              extern "c" fn host_file_write(file_handle: i64, text_handle: i64) -> i64;
              extern "c" fn host_file_close(file_handle: i64) -> i64;
              extern "c" fn host_path_copy(src_handle: i64, dst_handle: i64) -> i64;
              extern "c" fn host_fs_exists(path_handle: i64) -> i64;

              fn main() -> i64 {
                let handle: i64 = host_file_open(2103, 1);
                let backing: ref Buffer = alloc_buffer(8, 0);
                host_file_read(handle, host_buffer_handle(backing), 8);
                host_file_write(handle, 777);
                host_file_close(handle);
                host_path_copy(2103, 2109);
                host_fs_exists(2109);
                return 0;
              }
            }
            "#,
        );
        let output_dir = temp_dir("compile_command_host_file_outputs");
        let output_stem = project_name.to_owned();

        run(CommandKind::Compile {
            input: project_root.clone(),
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        for path in [
            output_dir.join(format!("{output_stem}.ll")),
            output_dir.join(&output_stem),
            output_dir.join("nuis.build.manifest.toml"),
            output_dir.join("nuis.compiled.artifact"),
            output_dir.join("nuis.project.host_ffi.txt"),
        ] {
            assert!(path.exists(), "expected output `{}`", path.display());
        }

        let manifest_text =
            fs::read_to_string(output_dir.join("nuis.build.manifest.toml")).unwrap();
        assert!(manifest_text.contains("host_ffi_index = "));

        let host_ffi_text =
            fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
        assert!(host_ffi_text.contains("host_file_open"));
        assert!(host_ffi_text.contains("host_file_read"));
        assert!(host_ffi_text.contains("host_file_write"));
        assert!(host_ffi_text.contains("host_file_close"));
        assert!(host_ffi_text.contains("host_path_copy"));
        assert!(host_ffi_text.contains("host_fs_exists"));

        let artifact_report =
            aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
                .unwrap();
        assert_eq!(artifact_report.binary_name, output_stem);
        assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
        assert!(artifact_report.lifecycle_contract_consistent);
        assert!(artifact_report.artifact_roundtrip_verified);

        let status = Command::new(output_dir.join(&output_stem))
            .status()
            .expect("expected compiled binary to launch");
        assert!(
            status.success(),
            "expected compiled binary to exit successfully"
        );
    }

    #[test]
    fn compile_command_writes_benchmark_report_file_tooling_outputs() {
        let project_root = PathBuf::from(
            "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_file_demo",
        );
        let output_dir = temp_dir("compile_command_benchmark_report_file_outputs");
        let output_stem = "benchmark_report_file_demo".to_owned();

        run(CommandKind::Compile {
            input: project_root,
            output_dir: output_dir.clone(),
            verbose_cache: false,
            cpu_abi: None,
            target: None,
        })
        .unwrap();

        for path in [
            output_dir.join(format!("{output_stem}.ll")),
            output_dir.join(&output_stem),
            output_dir.join("nuis.build.manifest.toml"),
            output_dir.join("nuis.compiled.artifact"),
            output_dir.join("nuis.project.host_ffi.txt"),
            output_dir.join("nuis.project.plan.txt"),
        ] {
            assert!(path.exists(), "expected output `{}`", path.display());
        }

        let manifest_path = output_dir.join("nuis.build.manifest.toml");
        let manifest_text = fs::read_to_string(&manifest_path).unwrap();
        assert!(manifest_text.contains("name = \"benchmark_report_file_demo\""));
        assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
        assert!(manifest_text.contains("host_ffi_index = "));

        let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
        assert_eq!(manifest_report.artifact_binary_name, output_stem);
        assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
        assert!(manifest_report.project_metadata_checked >= 2);

        let host_ffi_text =
            fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
        assert!(host_ffi_text.contains("host_monotonic_time_ns"));
        assert!(host_ffi_text.contains("host_sleep_ns"));
        assert!(host_ffi_text.contains("host_file_open"));
        assert!(host_ffi_text.contains("host_file_write"));
        assert!(host_ffi_text.contains("host_file_close"));
        assert!(host_ffi_text.contains("host_temp_file_handle"));

        let artifact_report =
            aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
                .unwrap();
        assert_eq!(artifact_report.binary_name, output_stem);
        assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
        assert!(artifact_report.lifecycle_contract_consistent);
        assert!(artifact_report.artifact_roundtrip_verified);

        let status = Command::new(output_dir.join(&output_stem))
            .status()
            .expect("expected compiled benchmark report binary to launch");
        assert!(
            status.success(),
            "expected compiled benchmark report binary to exit successfully"
        );
    }

    #[test]
    fn benchmark_inventory_collects_declared_benchmarks() {
        let artifacts = pipeline::compile_source(
            r#"
            mod cpu Main {
              benchmark("sum_loop", warmup_iters=4, measure_iters=32, timeout_ms=25, clock_domain="global", clock_policy="bridge")
              async fn sum_loop() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let entries = collect_benchmark_inventory(&artifacts);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].symbol, "cpu::Main::sum_loop");
        assert_eq!(entries[0].label, "sum_loop");
        assert!(entries[0].is_async);
        assert_eq!(entries[0].return_type, "i64");
        assert_eq!(entries[0].warmup_iters, Some(4));
        assert_eq!(entries[0].measure_iters, Some(32));
        assert_eq!(entries[0].timeout_ms, Some(25));
        assert_eq!(entries[0].clock_domain.as_deref(), Some("global"));
        assert_eq!(entries[0].clock_policy.as_deref(), Some("bridge"));
    }

    #[test]
    fn inspect_benchmarks_json_exposes_metadata() {
        let artifacts = pipeline::compile_source(
            r#"
            mod cpu Main {
              benchmark("sum_loop", measure_iters=32)
              fn sum_loop() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return sum_loop();
              }
            }
            "#,
        )
        .unwrap();

        let json = inspect_benchmarks_json(Path::new("main.ns"), &artifacts);
        assert!(json.contains("\"kind\":\"nuis_benchmark_inventory\""));
        assert!(json.contains("\"input\":\"main.ns\""));
        assert!(json.contains("\"benchmark_count\":1"));
        assert!(json.contains("\"symbol\":\"cpu::Main::sum_loop\""));
        assert!(json.contains("\"label\":\"sum_loop\""));
        assert!(json.contains("\"measure_iters\":32"));
    }

    #[test]
    fn inspect_docs_json_exposes_documented_items() {
        let ast = frontend::parse_nuis_ast(
            r#"
            /// module docs
            mod cpu Docs {
              /// function docs
              fn answer() -> i32 {
                42
              }
            }
            "#,
        )
        .unwrap();

        let indexes = vec![frontend::extract_ast_doc_index(&ast)];
        let json = inspect_docs_json(Path::new("main.ns"), &indexes);
        assert!(json.contains("\"kind\":\"nuis_doc_index\""));
        assert!(json.contains("\"input\":\"main.ns\""));
        assert!(json.contains("\"module_count\":1"));
        assert!(json.contains("\"documented_item_count\":2"));
        assert!(json.contains("\"module_path\":\"cpu.Docs\""));
        assert!(json.contains("\"kind\":\"module\""));
        assert!(json.contains("\"path\":\"cpu.Docs\""));
        assert!(json.contains("\"docs\":[\"module docs\"]"));
        assert!(json.contains("\"signature\":\"mod cpu Docs\""));
        assert!(json.contains("\"kind\":\"function\""));
        assert!(json.contains("\"path\":\"cpu.Docs::answer\""));
        assert!(json.contains("\"docs\":[\"function docs\"]"));
        assert!(json.contains("\"signature\":\"fn answer() -> i32\""));
    }

    #[test]
    fn collect_doc_indexes_reads_single_source_input() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nuis_doc_index_{nonce}.ns"));
        std::fs::write(
            &path,
            r#"
            /// module docs
            mod cpu Docs {
              /// value docs
              const ANSWER: i32 = 42;
            }
            "#,
        )
        .unwrap();

        let indexes = collect_doc_indexes(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(indexes.len(), 1);
        assert_eq!(indexes[0].module_path, "cpu.Docs");
        assert_eq!(indexes[0].items.len(), 2);
        assert_eq!(indexes[0].items[0].path, "cpu.Docs");
        assert_eq!(indexes[0].items[1].path, "cpu.Docs::ANSWER");
    }

    #[test]
    fn write_json_output_persists_payload() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("nuis_doc_index_output_{nonce}.json"));
        write_json_output(&path, "{\"kind\":\"nuis_doc_index\"}").unwrap();
        let written = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(written, "{\"kind\":\"nuis_doc_index\"}");
    }

    #[test]
    fn inspect_galaxy_docs_json_reports_documented_library_modules() {
        let summary = inspect_galaxy_doc_summary("pixelmagic").unwrap();
        let json = inspect_galaxy_docs_json(&summary);

        assert!(json.contains("\"kind\":\"nuis_galaxy_doc_index\""));
        assert!(json.contains("\"galaxy\":\"pixelmagic\""));
        assert!(json.contains("\"package_id\":\"nuis.pixelmagic\""));
        assert!(json.contains("\"documented_library_module_count\":"));
        assert!(json.contains("\"documented_item_count\":"));
        assert!(json.contains("\"library_module\":\"lib/image_contracts.ns\""));
        assert!(json.contains("\"module_path\":\"cpu.PixelMagicContracts\""));
    }

    #[test]
    fn inspect_stdlib_docs_json_reports_all_official_galaxies() {
        let summary = inspect_stdlib_doc_summary().unwrap();
        let json = inspect_stdlib_docs_json(&summary);

        assert!(json.contains("\"kind\":\"nuis_stdlib_doc_index\""));
        assert!(json.contains("\"galaxy_count\":5"));
        assert!(json.contains("\"galaxy\":\"core\""));
        assert!(json.contains("\"galaxy\":\"std\""));
        assert!(json.contains("\"galaxy\":\"pixelmagic\""));
        assert!(json.contains("\"galaxy\":\"witsage\""));
        assert!(json.contains("\"galaxy\":\"ns-nova\""));
    }
}
