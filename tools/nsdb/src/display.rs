use crate::model::NsdbInspectReport;
use crate::replay::build_replay_plan;

pub(crate) fn print_nsdb_inspect_report(report: &NsdbInspectReport) {
    println!("Nsdb YIR debug inspect");
    println!("  manifest: {}", report.manifest);
    println!("  output_dir: {}", report.output_dir);
    println!("  debug_model: {}", report.debug_model);
    println!(
        "  native_debugger_visibility: {}",
        report.native_debugger_visibility
    );
    println!("  nsdb_visibility: {}", report.nsdb_visibility);
    println!("  debug_readiness: {}", report.debug_readiness);
    println!("  yir_debuggable: {}", report.yir_debuggable);
    println!("  domain_count: {}", report.domain_count);
    println!("  hetero_domain_count: {}", report.hetero_domain_count);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  lowering_unit_count: {}", report.lowering_unit_count);
    println!("  sidecar_count: {}", report.sidecar_count);
    println!(
        "  payload_execution_event_filter_active: {}",
        report.payload_execution_event_filter.active()
    );
    println!(
        "  payload_execution_event_filter_status: {}",
        report
            .payload_execution_event_filter
            .status
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  payload_execution_event_filter_phase: {}",
        report
            .payload_execution_event_filter
            .phase
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  payload_execution_event_filter_trace_id: {}",
        report
            .payload_execution_event_filter
            .trace_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  payload_execution_handoff_available: {}",
        report.payload_execution_handoff.available
    );
    println!(
        "  payload_execution_handoff_status: {}",
        report.payload_execution_handoff.status
    );
    println!(
        "  payload_execution_handoff_protocol: {}",
        report.payload_execution_handoff.protocol
    );
    println!(
        "  payload_execution_handoff_record_count: {}",
        report.payload_execution_handoff.record_count
    );
    println!(
        "  payload_execution_handoff_first_trace_id: {}",
        report.payload_execution_handoff.first_trace_id
    );
    println!(
        "  payload_execution_handoff_first_entry_symbol: {}",
        report.payload_execution_handoff.first_entry_symbol
    );
    println!(
        "  payload_execution_event_count: {}",
        report.payload_execution_handoff.events.len()
    );
    for event in &report.payload_execution_handoff.events {
        println!(
            "  payload_execution_event: index={} trace={} phase={} status={} entry={} next={}",
            event.index,
            event.trace_id,
            event.execution_phase,
            event.status,
            event.entry_symbol,
            event.next_action
        );
    }
    println!(
        "  hetero_runtime_trace_available: {}",
        report.hetero_runtime_trace.available
    );
    println!(
        "  hetero_runtime_trace_status: {}",
        report.hetero_runtime_trace.status
    );
    println!(
        "  hetero_runtime_trace_protocol: {}",
        report.hetero_runtime_trace.protocol
    );
    println!(
        "  hetero_runtime_trace_record_count: {}",
        report.hetero_runtime_trace.record_count
    );
    println!(
        "  hetero_runtime_trace_first_trace_id: {}",
        report.hetero_runtime_trace.first_trace_id
    );
    for record in &report.hetero_runtime_trace.records {
        println!(
            "  hetero_runtime_trace_record: index={} trace={} role={} status={} domain={} backend={} target={} next={}",
            record.index,
            record.trace_id,
            record.trace_role,
            record.status,
            record.domain_family,
            record.backend_family,
            record.target_device,
            record.next_action
        );
    }
    for domain in &report.domains {
        println!(
            "  domain: {} package={} kind={} lowering={} backend={} scope={}",
            domain.domain_family,
            domain.package_id,
            domain.kind,
            domain.lowering_target,
            domain.backend_family,
            domain.debug_scope
        );
    }
    for edge in &report.clock_edges {
        println!(
            "  clock_edge: index={} from={} to={} relation={} source={}",
            edge.index, edge.from, edge.to, edge.relation, edge.source
        );
    }
    for segment in &report.data_segments {
        println!(
            "  data_segment: index={} id={} domain={} owner={} order={} phase={} source={}",
            segment.index,
            segment.segment_id,
            segment.domain_family,
            segment.owner_package,
            segment.order_key,
            segment.access_phase,
            segment.source_path
        );
    }
    for unit in &report.lowering_units {
        println!(
            "  lowering_unit: index={} package={} domain={} target={} backend={} sidecar={} role={}",
            unit.index,
            unit.package_id,
            unit.domain_family,
            unit.selected_lowering_target,
            unit.backend_family,
            unit.artifact_ir_sidecar_path,
            unit.packaging_role
        );
    }
    for sidecar in &report.sidecars {
        println!(
            "  sidecar: domain={} package={} schema={} owner={} frontend={} native={} dispatch={} transport={} entry={} stage={}",
            sidecar.domain_family,
            sidecar.package_id,
            sidecar.schema,
            sidecar.capability_owner,
            sidecar.frontend_ir,
            sidecar.native_ir,
            sidecar.dispatch_lowering,
            sidecar.transport_lowering,
            sidecar.entry_symbol,
            sidecar.stage_kind
        );
    }
    for item in &report.missing_metadata {
        println!("  missing_metadata: {item}");
    }
}

pub(crate) fn print_nsdb_events_report(report: &NsdbInspectReport) {
    println!("Nsdb payload execution events");
    println!("  manifest: {}", report.manifest);
    println!(
        "  payload_execution_event_filter_active: {}",
        report.payload_execution_event_filter.active()
    );
    println!(
        "  payload_execution_event_filter_status: {}",
        report
            .payload_execution_event_filter
            .status
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  payload_execution_event_filter_phase: {}",
        report
            .payload_execution_event_filter
            .phase
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  payload_execution_event_filter_trace_id: {}",
        report
            .payload_execution_event_filter
            .trace_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  payload_execution_handoff_available: {}",
        report.payload_execution_handoff.available
    );
    println!(
        "  payload_execution_handoff_status: {}",
        report.payload_execution_handoff.status
    );
    println!(
        "  payload_execution_event_count: {}",
        report.payload_execution_handoff.events.len()
    );
    for event in &report.payload_execution_handoff.events {
        println!(
            "  payload_execution_event: index={} trace={} phase={} status={} entry={} next={}",
            event.index,
            event.trace_id,
            event.execution_phase,
            event.status,
            event.entry_symbol,
            event.next_action
        );
    }
}

pub(crate) fn print_nsdb_replay_plan(report: &NsdbInspectReport) {
    let plan = build_replay_plan(report);
    println!("Nsdb payload execution replay plan");
    println!("  manifest: {}", report.manifest);
    println!("  replay_protocol: {}", plan.protocol);
    println!("  replay_status: {}", plan.status);
    println!("  replay_checkpoint_count: {}", plan.checkpoint_count);
    println!(
        "  replayable_checkpoint_count: {}",
        plan.replayable_checkpoint_count
    );
    println!(
        "  replay_first_blocker: {}",
        plan.first_blocker.as_deref().unwrap_or("<none>")
    );
    for checkpoint in &plan.checkpoints {
        println!(
            "  replay_checkpoint: index={} trace={} kind={} status={} frame={} slot={} value_state={} value_sample_contract={} value_sample_ref={} value_sample_source={} value_sample_resolution={} value_sample_detail={} value_sample_materialization={} value_sample_payload={} value_sample_bridge_stub={} value_slot_id={} value_schema_status={} value_schema_hint={} value_snapshot_status={} value_snapshot_type={} value_snapshot_ref={} value_content_status={} value_content_type={} value_decoder_id={} value_decoder_status={} value_decoder_capability={} value_decoder_detail_level={} value_decoder_reads_file_summary={} value_decoder_format_probe_status={} value_decoder_format_probe_detail={} phase={} entry={} blocker={} next={}",
            checkpoint.index,
            checkpoint.trace_id,
            checkpoint.checkpoint_kind,
            checkpoint.replay_status,
            checkpoint.frame_id,
            checkpoint.slot_scope,
            checkpoint.value_state_status,
            checkpoint.value_sample_contract,
            checkpoint.value_sample_ref,
            checkpoint.value_sample_source,
            checkpoint.value_sample_resolution_status,
            checkpoint.value_sample_resolution_detail,
            checkpoint.value_sample_materialization_status,
            checkpoint.value_sample_payload_path,
            checkpoint.value_sample_bridge_stub_path,
            checkpoint.value_slot_id,
            checkpoint.value_schema_status,
            checkpoint.value_schema_hint,
            checkpoint.value_snapshot_status,
            checkpoint.value_snapshot_type,
            checkpoint.value_snapshot_ref,
            checkpoint.value_content_status,
            checkpoint.value_content_type,
            checkpoint.value_decoder_id,
            checkpoint.value_decoder_status,
            checkpoint.value_decoder_capability,
            checkpoint.value_decoder_detail_level,
            checkpoint.value_decoder_reads_file_summary,
            checkpoint.value_decoder_format_probe_status,
            checkpoint.value_decoder_format_probe_detail,
            checkpoint.execution_phase,
            checkpoint.entry_symbol,
            checkpoint.first_blocker.as_deref().unwrap_or("<none>"),
            checkpoint.next_action
        );
    }
}
