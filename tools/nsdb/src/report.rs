use crate::{
    handoff::read_payload_execution_handoff,
    model::{
        NsdbClockEdgeDebugInfo, NsdbDataSegmentDebugInfo, NsdbDomainDebugInfo, NsdbInspectReport,
        NsdbLoweringUnitDebugInfo, NsdbPayloadExecutionEventFilter,
    },
    sidecar::read_sidecar_debug_info,
};
use std::path::Path;

pub(crate) fn nsdb_inspect_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
    event_filter: NsdbPayloadExecutionEventFilter,
) -> NsdbInspectReport {
    let domains = plan
        .domain_units
        .iter()
        .map(|unit| NsdbDomainDebugInfo {
            domain_family: unit.domain_family.clone(),
            package_id: unit.package_id.clone(),
            kind: unit.kind.clone(),
            lowering_target: unit
                .selected_lowering_target
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            backend_family: unit
                .backend_family
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            debug_scope: if unit.kind == "host" {
                "host-shell+yir".to_owned()
            } else {
                "yir-domain".to_owned()
            },
        })
        .collect::<Vec<_>>();
    let clock_edges = plan
        .clock_protocol
        .edges
        .iter()
        .map(|edge| NsdbClockEdgeDebugInfo {
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
        .map(|segment| NsdbDataSegmentDebugInfo {
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
    let lowering_units = plan
        .compiled_artifact
        .lowering_units
        .iter()
        .enumerate()
        .map(|(index, unit)| NsdbLoweringUnitDebugInfo {
            index,
            package_id: unit.package_id.clone(),
            domain_family: unit.domain_family.clone(),
            backend_family: unit
                .backend_family
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            selected_lowering_target: unit
                .selected_lowering_target
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            artifact_ir_sidecar_path: unit
                .artifact_ir_sidecar_path
                .clone()
                .unwrap_or_else(|| "none".to_owned()),
            contract_family: unit.contract_family.clone(),
            packaging_role: unit.packaging_role.clone(),
        })
        .collect::<Vec<_>>();
    let sidecars = lowering_units
        .iter()
        .filter(|unit| unit.artifact_ir_sidecar_path != "none")
        .filter_map(read_sidecar_debug_info)
        .collect::<Vec<_>>();
    let mut payload_execution_handoff = read_payload_execution_handoff(Path::new(&plan.output_dir));
    let mut missing_metadata = Vec::new();
    if !plan.clock_protocol.validation.valid {
        missing_metadata.push("valid-clock-protocol".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        missing_metadata.push("valid-hetero-calculate-plan".to_owned());
    }
    if lowering_units.is_empty() {
        missing_metadata.push("compiled-artifact-lowering-units".to_owned());
    }
    let expected_sidecars = lowering_units
        .iter()
        .filter(|unit| unit.artifact_ir_sidecar_path != "none")
        .count();
    if sidecars.len() != expected_sidecars {
        missing_metadata.push("readable-ir-sidecars".to_owned());
    }
    if !payload_execution_handoff.available {
        missing_metadata.push("payload-execution-handoff".to_owned());
    } else if payload_execution_handoff.status != "ready" {
        missing_metadata.push("ready-payload-execution-handoff".to_owned());
    }
    if event_filter.active() {
        payload_execution_handoff
            .events
            .retain(|event| event_filter.matches(event));
    }

    NsdbInspectReport {
        manifest: manifest.display().to_string(),
        debug_model: "yir-metadata".to_owned(),
        native_debugger_visibility: "host-shell-only".to_owned(),
        nsdb_visibility: "domains+clock+segments+lowering-units".to_owned(),
        debug_readiness: if missing_metadata.is_empty() {
            "yir-debug-ready".to_owned()
        } else {
            "metadata-partial".to_owned()
        },
        yir_debuggable: missing_metadata.is_empty(),
        domain_count: plan.domain_units.len(),
        hetero_domain_count: plan
            .domain_units
            .iter()
            .filter(|unit| unit.kind == "heterogeneous")
            .count(),
        clock_edge_count: clock_edges.len(),
        data_segment_count: data_segments.len(),
        lowering_unit_count: lowering_units.len(),
        sidecar_count: sidecars.len(),
        payload_execution_event_filter: event_filter,
        payload_execution_handoff,
        domains,
        clock_edges,
        data_segments,
        lowering_units,
        sidecars,
        missing_metadata,
    }
}
