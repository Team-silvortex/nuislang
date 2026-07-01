use super::{
    fnv1a64_hex,
    reports::{
        NsldDomainDiagnostic, NsldLinkInputDiagnostic, NsldLinkInputSummary,
        NsldLinkUnitDiagnostic, NsldLinkUnitReport, NsldSidecarCapabilityDiagnostic,
    },
    toml_string_array_value, toml_string_value,
};
use std::{fs, path::Path};

pub(crate) fn nsld_link_unit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkUnitReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut units = plan.domain_units.iter().collect::<Vec<_>>();
    units.sort_by(|left, right| {
        left.domain_family
            .cmp(&right.domain_family)
            .then_with(|| left.package_id.cmp(&right.package_id))
            .then_with(|| left.packaging_role.cmp(&right.packaging_role))
    });
    let units = units
        .into_iter()
        .enumerate()
        .map(|(index, unit)| {
            let link_input_ids = link_input_summary
                .inputs
                .iter()
                .filter(|input| {
                    input.domain_family == unit.domain_family && input.package_id == unit.package_id
                })
                .map(|input| input.input_id.clone())
                .collect::<Vec<_>>();
            let clock_edge_count = plan
                .clock_protocol
                .edges
                .iter()
                .filter(|edge| {
                    edge.from.contains(&unit.domain_family) || edge.to.contains(&unit.domain_family)
                })
                .count();
            let data_segment_count = plan
                .hetero_calculate
                .data_segments
                .iter()
                .filter(|segment| {
                    segment.domain_family == unit.domain_family
                        && segment.owner_package == unit.package_id
                })
                .count();
            let unit_kind = if unit.kind == "heterogeneous" {
                "hetero-domain"
            } else {
                "native-domain"
            }
            .to_owned();
            let deterministic_order_key =
                format!("{index:04}.{}.{}", unit.domain_family, unit.package_id);

            NsldLinkUnitDiagnostic {
                order_index: index,
                unit_id: format!("lu{index:04}.{}.{}", unit.domain_family, unit.package_id),
                unit_kind,
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                backend_family: unit
                    .backend_family
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                lowering_target: unit
                    .selected_lowering_target
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                packaging_role: unit.packaging_role.clone(),
                link_input_ids,
                clock_edge_count,
                data_segment_count,
                requires_host_wrapper: host_wrapper_required
                    && (unit.domain_family == "cpu" || unit.packaging_role.contains("launcher")),
                deterministic_order_key,
            }
        })
        .collect::<Vec<_>>();
    let unit_table_hash = nsld_link_unit_table_hash(&units);

    NsldLinkUnitReport {
        manifest: manifest.display().to_string(),
        unit_count: units.len(),
        hetero_unit_count: units
            .iter()
            .filter(|unit| unit.unit_kind == "hetero-domain")
            .count(),
        link_input_count: link_input_summary.count,
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        unit_table_hash,
        units,
    }
}

pub(crate) fn nsld_link_unit_table_hash(units: &[NsldLinkUnitDiagnostic]) -> String {
    let mut material = String::new();
    for unit in units {
        material.push_str(&unit.order_index.to_string());
        material.push('\t');
        material.push_str(&unit.unit_id);
        material.push('\t');
        material.push_str(&unit.unit_kind);
        material.push('\t');
        material.push_str(&unit.domain_family);
        material.push('\t');
        material.push_str(&unit.package_id);
        material.push('\t');
        material.push_str(&unit.backend_family);
        material.push('\t');
        material.push_str(&unit.lowering_target);
        material.push('\t');
        material.push_str(&unit.packaging_role);
        material.push('\t');
        material.push_str(&unit.link_input_ids.join("|"));
        material.push('\t');
        material.push_str(&unit.clock_edge_count.to_string());
        material.push('\t');
        material.push_str(&unit.data_segment_count.to_string());
        material.push('\t');
        material.push_str(if unit.requires_host_wrapper {
            "host-wrapper"
        } else {
            "self-contained"
        });
        material.push('\t');
        material.push_str(&unit.deterministic_order_key);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

pub(crate) fn nsld_domain_diagnostics(plan: &nuisc::linker::LinkPlan) -> Vec<NsldDomainDiagnostic> {
    plan.domain_units
        .iter()
        .map(|unit| {
            let alignment = plan
                .artifact_lowering_alignment
                .checks
                .iter()
                .find(|check| {
                    check.package_id == unit.package_id && check.domain_family == unit.domain_family
                });
            NsldDomainDiagnostic {
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                kind: unit.kind.clone(),
                packaging_role: unit.packaging_role.clone(),
                lowering_target: unit
                    .selected_lowering_target
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                backend_family: unit
                    .backend_family
                    .clone()
                    .unwrap_or_else(|| "none".to_owned()),
                alignment_consistent: alignment.map(|check| check.consistent).unwrap_or(true),
                alignment_issues: alignment
                    .map(|check| check.issues.clone())
                    .unwrap_or_default(),
            }
        })
        .collect()
}

pub(crate) fn nsld_sidecar_capability_diagnostics(
    plan: &nuisc::linker::LinkPlan,
) -> Vec<NsldSidecarCapabilityDiagnostic> {
    plan.domain_units
        .iter()
        .filter(|unit| unit.kind == "heterogeneous")
        .filter(|unit| unit.artifact_ir_sidecar_path.is_some())
        .map(|unit| {
            let path = unit
                .artifact_ir_sidecar_path
                .clone()
                .unwrap_or_else(|| "none".to_owned());
            let Some(source) = unit
                .artifact_ir_sidecar_path
                .as_deref()
                .and_then(|path| fs::read_to_string(path).ok())
            else {
                return NsldSidecarCapabilityDiagnostic {
                    domain_family: unit.domain_family.clone(),
                    package_id: unit.package_id.clone(),
                    path,
                    content_bytes: 0,
                    content_hash: "missing".to_owned(),
                    valid: false,
                    capability_owner: "missing".to_owned(),
                    frontend_ir: "missing".to_owned(),
                    native_ir: "missing".to_owned(),
                    dispatch_lowering: "missing".to_owned(),
                    validation_contracts: Vec::new(),
                    issues: vec!["missing_or_unreadable_ir_sidecar".to_owned()],
                };
            };

            let capability_owner =
                toml_string_value(&source, "capability_owner").unwrap_or_else(|| "missing".to_owned());
            let frontend_ir =
                toml_string_value(&source, "frontend_ir").unwrap_or_else(|| "missing".to_owned());
            let native_ir =
                toml_string_value(&source, "native_ir").unwrap_or_else(|| "missing".to_owned());
            let dispatch_lowering =
                toml_string_value(&source, "dispatch_lowering").unwrap_or_else(|| "missing".to_owned());
            let validation_contracts = toml_string_array_value(&source, "validation_contracts");
            let mut issues = Vec::new();
            let expected_owner = format!("{}-nustar", unit.domain_family);
            if capability_owner != expected_owner {
                issues.push(format!(
                    "capability_owner mismatch: expected `{expected_owner}`, found `{capability_owner}`"
                ));
            }
            let expected_frontend = format!("nuis-yir.{}", unit.domain_family);
            if frontend_ir != expected_frontend {
                issues.push(format!(
                    "frontend_ir mismatch: expected `{expected_frontend}`, found `{frontend_ir}`"
                ));
            }
            if native_ir == "missing" || native_ir == "unknown" || native_ir == "unimplemented" {
                issues.push(format!("native_ir is not link-ready: `{native_ir}`"));
            }
            if dispatch_lowering == "missing" || dispatch_lowering == "unimplemented" {
                issues.push(format!(
                    "dispatch_lowering is not link-ready: `{dispatch_lowering}`"
                ));
            }
            if validation_contracts.is_empty() {
                issues.push("validation_contracts is empty".to_owned());
            }

            NsldSidecarCapabilityDiagnostic {
                domain_family: unit.domain_family.clone(),
                package_id: unit.package_id.clone(),
                path,
                content_bytes: source.len(),
                content_hash: fnv1a64_hex(source.as_bytes()),
                valid: issues.is_empty(),
                capability_owner,
                frontend_ir,
                native_ir,
                dispatch_lowering,
                validation_contracts,
                issues,
            }
        })
        .collect()
}

pub(crate) fn nsld_link_input_diagnostics(
    capabilities: &[NsldSidecarCapabilityDiagnostic],
) -> Vec<NsldLinkInputDiagnostic> {
    let mut capabilities = capabilities
        .iter()
        .filter(|capability| capability.valid)
        .collect::<Vec<_>>();
    capabilities.sort_by(|left, right| {
        left.domain_family
            .cmp(&right.domain_family)
            .then_with(|| left.package_id.cmp(&right.package_id))
            .then_with(|| left.path.cmp(&right.path))
    });
    capabilities
        .into_iter()
        .enumerate()
        .map(|(index, capability)| NsldLinkInputDiagnostic {
            order_index: index,
            input_id: format!(
                "li{:04}.{}.{}",
                index, capability.domain_family, capability.package_id
            ),
            input_kind: "lowering-ir-sidecar".to_owned(),
            domain_family: capability.domain_family.clone(),
            package_id: capability.package_id.clone(),
            path: capability.path.clone(),
            native_ir: capability.native_ir.clone(),
            dispatch_lowering: capability.dispatch_lowering.clone(),
            contract_count: capability.validation_contracts.len(),
            content_bytes: capability.content_bytes,
            content_hash: capability.content_hash.clone(),
        })
        .collect()
}

pub(crate) fn nsld_link_input_summary(
    capabilities: &[NsldSidecarCapabilityDiagnostic],
) -> NsldLinkInputSummary {
    let inputs = nsld_link_input_diagnostics(capabilities);
    let count = inputs.len();
    let total_bytes = inputs
        .iter()
        .map(|input| input.content_bytes)
        .sum::<usize>();
    let table_hash = nsld_link_input_table_hash(&inputs);
    NsldLinkInputSummary {
        inputs,
        count,
        total_bytes,
        table_hash,
    }
}

pub(crate) fn nsld_link_input_table_hash(inputs: &[NsldLinkInputDiagnostic]) -> String {
    let mut material = String::new();
    for input in inputs {
        material.push_str(&input.order_index.to_string());
        material.push('\t');
        material.push_str(&input.input_id);
        material.push('\t');
        material.push_str(&input.input_kind);
        material.push('\t');
        material.push_str(&input.domain_family);
        material.push('\t');
        material.push_str(&input.package_id);
        material.push('\t');
        material.push_str(&input.native_ir);
        material.push('\t');
        material.push_str(&input.dispatch_lowering);
        material.push('\t');
        material.push_str(&input.contract_count.to_string());
        material.push('\t');
        material.push_str(&input.content_bytes.to_string());
        material.push('\t');
        material.push_str(&input.content_hash);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}
