use std::path::Path;

use crate::{
    aot, domain_build_contract_summary_json, json_bool_field, json_optional_string_field,
    json_string_array_field, json_string_field, json_usize_field, registry, NUSTAR_REGISTRY_ROOT,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainBuildContractDriftCheck {
    pub(crate) package_id: String,
    pub(crate) domain_family: String,
    pub(crate) consistent: bool,
    pub(crate) issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainBuildUnitVerificationVerdict {
    pub(crate) package_id: String,
    pub(crate) domain_family: String,
    pub(crate) kind: String,
    pub(crate) payload_blob_ok: bool,
    pub(crate) lowering_plan_ok: bool,
    pub(crate) backend_stub_ok: bool,
    pub(crate) bridge_plan_ok: bool,
    pub(crate) bridge_stub_ok: bool,
    pub(crate) bridge_registry_ok: bool,
    pub(crate) host_bridge_plan_ok: bool,
    pub(crate) registry_alignment_ok: bool,
    pub(crate) failure_reasons: Vec<String>,
    pub(crate) consistent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainBuildVerificationSummary {
    pub(crate) all_units_consistent: bool,
    pub(crate) total_units: usize,
    pub(crate) host_units_checked: usize,
    pub(crate) hetero_units_checked: usize,
    pub(crate) registry_drift_units: usize,
    pub(crate) failing_units: Vec<String>,
}

pub(crate) fn domain_build_unit_effective_contract_summary(
    unit: &aot::BuildManifestDomainBuildUnit,
) -> registry::NustarDomainBuildContractSummary {
    load_manifest_for_build_unit(unit)
        .map(|manifest| registry::domain_build_contract_summary(&manifest))
        .unwrap_or_else(|_| registry::domain_build_contract_summary_for_domain(&unit.domain_family))
}

pub(crate) fn load_manifest_for_build_unit(
    unit: &aot::BuildManifestDomainBuildUnit,
) -> Result<registry::NustarPackageManifest, String> {
    registry::load_manifest(Path::new(NUSTAR_REGISTRY_ROOT), &unit.package_id).or_else(|error| {
        registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &unit.domain_family)
            .map_err(|_| error)
    })
}

pub(crate) fn domain_build_unit_contract_json(unit: &aot::BuildManifestDomainBuildUnit) -> String {
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

pub(crate) fn domain_build_unit_contracts_json(
    units: &[aot::BuildManifestDomainBuildUnit],
) -> String {
    units
        .iter()
        .map(domain_build_unit_contract_json)
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn evaluate_domain_build_contract_drift(
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

pub(crate) fn domain_build_contract_drift_json(check: &DomainBuildContractDriftCheck) -> String {
    let fields = vec![
        json_string_field("package_id", &check.package_id),
        json_string_field("domain_family", &check.domain_family),
        json_bool_field("consistent", check.consistent),
        json_string_array_field("issues", &check.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn domain_build_contract_drift_checks(
    units: &[aot::BuildManifestDomainBuildUnit],
) -> Vec<DomainBuildContractDriftCheck> {
    units
        .iter()
        .map(evaluate_domain_build_contract_drift)
        .collect()
}

pub(crate) fn domain_build_unit_verification_verdict(
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

pub(crate) fn domain_build_unit_verification_verdict_json(
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

pub(crate) fn collect_domain_build_unit_verdicts(
    report: &aot::BuildManifestVerifyReport,
) -> Vec<DomainBuildUnitVerificationVerdict> {
    report
        .domain_build_units
        .iter()
        .map(|unit| domain_build_unit_verification_verdict(unit, report))
        .collect()
}

pub(crate) fn summarize_domain_build_verification(
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

pub(crate) fn domain_build_verification_summary_json(
    summary: &DomainBuildVerificationSummary,
) -> String {
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
