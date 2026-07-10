use super::{
    link_units::{nsld_domain_diagnostics, nsld_sidecar_capability_diagnostics},
    reports::{
        NsldClockEdgeDiagnostic, NsldDataSegmentDiagnostic, NsldDomainDiagnostic,
        NsldSidecarCapabilityDiagnostic,
    },
};

pub(crate) struct NsldCheckCoreSnapshot {
    pub(crate) artifact_lowering_alignment_consistent: bool,
    pub(crate) artifact_lowering_alignment_mismatches: usize,
    pub(crate) clock_protocol_valid: bool,
    pub(crate) clock_protocol_issues: Vec<String>,
    pub(crate) hetero_calculate_valid: bool,
    pub(crate) hetero_calculate_issues: Vec<String>,
    pub(crate) static_link: bool,
    pub(crate) lifecycle_driven: bool,
    pub(crate) sidecar_capability_valid: bool,
    pub(crate) sidecar_capability_issues: Vec<String>,
    pub(crate) domains: Vec<NsldDomainDiagnostic>,
    pub(crate) sidecar_capabilities: Vec<NsldSidecarCapabilityDiagnostic>,
    pub(crate) clock_edges: Vec<NsldClockEdgeDiagnostic>,
    pub(crate) data_segments: Vec<NsldDataSegmentDiagnostic>,
}

pub(crate) fn nsld_check_core_snapshot(plan: &nuisc::linker::LinkPlan) -> NsldCheckCoreSnapshot {
    let artifact_lowering_alignment_consistent = plan.artifact_lowering_alignment.consistent;
    let artifact_lowering_alignment_mismatches = plan.artifact_lowering_alignment.mismatches;
    let clock_protocol_valid = plan.clock_protocol.validation.valid;
    let clock_protocol_issues = plan.clock_protocol.validation.issues.clone();
    let hetero_calculate_valid = plan.hetero_calculate.validation.valid;
    let hetero_calculate_issues = plan.hetero_calculate.validation.issues.clone();
    let static_link = plan.hetero_calculate.static_link;
    let lifecycle_driven = plan.hetero_calculate.lifecycle_driven;
    let domains = nsld_domain_diagnostics(plan);
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let sidecar_capability_issues = sidecar_capabilities
        .iter()
        .flat_map(|capability| {
            capability
                .issues
                .iter()
                .map(|issue| {
                    format!(
                        "{}:{}: {}",
                        capability.package_id, capability.domain_family, issue
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let sidecar_capability_valid = sidecar_capability_issues.is_empty();
    let clock_edges = plan
        .clock_protocol
        .edges
        .iter()
        .map(|edge| NsldClockEdgeDiagnostic {
            index: edge.index,
            from: edge.from.clone(),
            to: edge.to.clone(),
            relation: edge.relation.clone(),
            source: edge.source.clone(),
        })
        .collect::<Vec<_>>();
    let data_segments = plan
        .hetero_calculate
        .data_segments
        .iter()
        .map(|segment| NsldDataSegmentDiagnostic {
            index: segment.index,
            segment_id: segment.segment_id.clone(),
            domain_family: segment.domain_family.clone(),
            owner_package: segment.owner_package.clone(),
            order_key: segment.order_key.clone(),
            access_phase: segment.access_phase.clone(),
            source_path: segment
                .source_path
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
        })
        .collect::<Vec<_>>();

    NsldCheckCoreSnapshot {
        artifact_lowering_alignment_consistent,
        artifact_lowering_alignment_mismatches,
        clock_protocol_valid,
        clock_protocol_issues,
        hetero_calculate_valid,
        hetero_calculate_issues,
        static_link,
        lifecycle_driven,
        sidecar_capability_valid,
        sidecar_capability_issues,
        domains,
        sidecar_capabilities,
        clock_edges,
        data_segments,
    }
}

pub(crate) fn push_optional_check_failure(
    issues: &mut Vec<String>,
    valid: Option<bool>,
    headline: &str,
    details: &[String],
) {
    if valid == Some(false) {
        issues.push(headline.to_owned());
        issues.extend(details.iter().cloned());
    }
}
