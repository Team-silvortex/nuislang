mod cli;
mod cursor;
mod cursor_lineage;
mod cursor_lineage_repair_journal;
mod digest_sha256;
mod display;
mod handoff;
mod hetero_trace;
mod json;
mod json_replay;
mod json_transcript;
mod model;
mod payload_decoder;
mod provider_completion_integrity;
mod provider_completion_signature;
mod provider_completion_trust_anchor;
mod provider_completion_trust_registry;
mod provider_output_comparison;
mod provider_request;
mod provider_runner_coreml;
mod provider_runner_metal;
mod provider_runner_registry;
mod provider_sample;
mod provider_sample_artifact;
mod provider_sample_execute;
mod provider_sample_execution;
mod provider_sample_materialize;
mod provider_sample_payload;
#[cfg(test)]
mod provider_sample_payload_tests;
mod provider_sample_runner;
mod replay;
#[cfg(test)]
mod replay_tests;
mod report;
mod sidecar;
mod transcript;

use crate::{
    cli::{parse_args, resolve_manifest_input, Command},
    display::{
        print_nsdb_events_report, print_nsdb_inspect_report, print_nsdb_replay_plan,
        print_nsdb_replay_transcript, print_nsdb_replay_transcript_with_control,
    },
    json::{nsdb_events_report_json, nsdb_inspect_report_json, nsdb_replay_plan_json},
    json_transcript::{nsdb_replay_transcript_json, nsdb_replay_transcript_json_with_control},
    provider_sample_execute::execute_provider_samples,
    provider_sample_materialize::{materialize_provider_samples, ProviderSampleMaterializeReport},
    report::nsdb_inspect_report,
};
use std::{env, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        Command::Status => {
            println!("Nsdb YIR debugger front-door");
            println!("  tool: nsdb");
            println!("  phase: alpha-0.6.0 debugger metadata boundary");
            println!("  debug_model: yir-metadata");
            println!("  native_debugger_visibility: host-shell-only");
            println!("  nsdb_visibility: yir domains, clock edges, data segments, lowering units");
        }
        Command::Inspect {
            input,
            json,
            event_filter,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if json {
                println!("{}", nsdb_inspect_report_json(&report));
            } else {
                print_nsdb_inspect_report(&report);
            }
        }
        Command::Events {
            input,
            json,
            event_filter,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if json {
                println!("{}", nsdb_events_report_json(&report));
            } else {
                print_nsdb_events_report(&report);
            }
        }
        Command::ReplayPlan {
            input,
            json,
            event_filter,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if json {
                println!("{}", nsdb_replay_plan_json(&report));
            } else {
                print_nsdb_replay_plan(&report);
            }
        }
        Command::Replay {
            input,
            json,
            event_filter,
            mut replay_control,
            cursor_input,
            cursor_output,
        } => {
            let manifest = resolve_manifest_input(&input)?;
            if let Some(path) = cursor_input.as_deref() {
                let loaded = crate::cursor::load_replay_cursor(path, &manifest)?;
                replay_control.resume_after_frame_id = loaded.resume_after_frame_id;
                replay_control.resume_next_frame_id = loaded.resume_next_frame_id;
            }
            let plan = nuisc::linker::build_link_plan_from_manifest(&manifest)?;
            let report = nsdb_inspect_report(&manifest, &plan, event_filter);
            if let Some(path) = cursor_output.as_deref() {
                let transcript = crate::transcript::build_replay_transcript_with_control(
                    &report,
                    &replay_control,
                );
                crate::cursor::persist_replay_cursor(path, &manifest, &transcript)?;
            }
            if json {
                let output = if replay_control == Default::default() {
                    nsdb_replay_transcript_json(&report)
                } else {
                    nsdb_replay_transcript_json_with_control(&report, &replay_control)
                };
                println!("{output}");
            } else if replay_control == Default::default() {
                print_nsdb_replay_transcript(&report);
            } else {
                print_nsdb_replay_transcript_with_control(&report, &replay_control);
            }
        }
        Command::CursorLineageRepair { input, json } => {
            let manifest = resolve_manifest_input(&input)?;
            let output_dir = manifest.parent().ok_or_else(|| {
                format!("manifest `{}` has no output directory", manifest.display())
            })?;
            let report = crate::cursor_lineage::repair_cursor_lineage(output_dir, &manifest)?;
            if json {
                println!(
                    "{{\"tool\":\"nsdb\",\"kind\":\"cursor_lineage_repair\",\"contract\":\"{}\",\"status\":\"{}\",\"mutated\":{},\"lineage_mutated\":{},\"repair_journal_mutated\":{},\"cursor_path\":\"{}\",\"lineage_path\":\"{}\",\"archived_path\":{},\"repair_journal_path\":\"{}\",\"archived_repair_journal_path\":{},\"entry_count\":{},\"latest_hash\":\"{}\"}}",
                    report.contract,
                    report.status,
                    report.mutated,
                    report.lineage_mutated,
                    report.repair_journal_mutated,
                    json_escape(&report.cursor_path),
                    json_escape(&report.lineage_path),
                    report
                        .archived_path
                        .as_deref()
                        .map(|path| format!("\"{}\"", json_escape(path)))
                        .unwrap_or_else(|| "null".to_owned()),
                    json_escape(&report.repair_journal_path),
                    report
                        .archived_repair_journal_path
                        .as_deref()
                        .map(|path| format!("\"{}\"", json_escape(path)))
                        .unwrap_or_else(|| "null".to_owned()),
                    report.entry_count,
                    report.latest_hash,
                );
            } else {
                println!("cursor_lineage_repair_contract: {}", report.contract);
                println!("cursor_lineage_repair_status: {}", report.status);
                println!("cursor_lineage_repair_mutated: {}", report.mutated);
                println!(
                    "cursor_lineage_repair_lineage_mutated: {}",
                    report.lineage_mutated
                );
                println!(
                    "cursor_lineage_repair_journal_mutated: {}",
                    report.repair_journal_mutated
                );
                println!("cursor_lineage_repair_cursor_path: {}", report.cursor_path);
                println!(
                    "cursor_lineage_repair_lineage_path: {}",
                    report.lineage_path
                );
                println!(
                    "cursor_lineage_repair_archived_path: {}",
                    report.archived_path.as_deref().unwrap_or("<none>")
                );
                println!(
                    "cursor_lineage_repair_journal_path: {}",
                    report.repair_journal_path
                );
                println!(
                    "cursor_lineage_repair_archived_journal_path: {}",
                    report
                        .archived_repair_journal_path
                        .as_deref()
                        .unwrap_or("<none>")
                );
                println!("cursor_lineage_repair_entry_count: {}", report.entry_count);
                println!("cursor_lineage_repair_latest_hash: {}", report.latest_hash);
            }
        }
        Command::MaterializeProviderSamples {
            output_dir,
            provider_family,
            json,
        } => {
            let report = materialize_provider_samples(&output_dir, provider_family.as_deref())?;
            if json {
                println!("{}", provider_sample_materialize_json(&report));
            } else {
                println!(
                    "device_provider_sample_materialize_status: {}",
                    report.status
                );
                println!("device_provider_sample_manifest_path: {}", report.path);
                println!(
                    "device_provider_sample_provider_family_filter: {}",
                    report.provider_family_filter.as_deref().unwrap_or("<none>")
                );
                println!(
                    "device_provider_sample_provider_families: {}",
                    report.provider_families.join(",")
                );
                println!(
                    "device_provider_sample_matched_record_count: {}",
                    report.matched_record_count
                );
                println!(
                    "device_provider_sample_materialized_record_count: {}",
                    report.materialized_record_count
                );
                println!(
                    "device_provider_sample_first_runner_adapter_contract: {}",
                    report.first_provider_runner_adapter_contract
                );
                println!(
                    "device_provider_sample_first_runner_adapter_id: {}",
                    report.first_provider_runner_adapter_id
                );
                println!(
                    "device_provider_sample_first_runner_adapter_capability_status: {}",
                    report.first_provider_runner_adapter_capability_status
                );
                println!(
                    "device_provider_sample_first_execution_comparison_status: {}",
                    report.first_provider_execution_comparison_status
                );
                println!(
                    "device_provider_sample_first_output_payload_status: {}",
                    report.first_provider_output_payload_status
                );
                println!(
                    "device_provider_sample_first_output_payload_evidence: {}",
                    report.first_provider_output_payload_evidence
                );
                println!(
                    "device_provider_sample_first_output_payload_path: {}",
                    report.first_provider_output_payload_path
                );
                println!(
                    "device_provider_sample_first_output_payload_hash: {}",
                    report.first_provider_output_payload_hash
                );
                println!(
                    "device_provider_sample_first_output_payload_attach_status: {}",
                    report.first_provider_output_payload_attach_status
                );
                println!("device_provider_sample_next_action: {}", report.next_action);
                println!(
                    "device_provider_sample_next_command: {}",
                    report.next_command
                );
                println!(
                    "device_provider_sample_return_contract: {}",
                    report.return_contract
                );
                println!(
                    "device_provider_sample_return_action: {}",
                    report.return_action
                );
                println!(
                    "device_provider_sample_return_command: {}",
                    report.return_command
                );
                println!(
                    "device_provider_sample_final_output_replay_contract: {}",
                    report.final_output_replay_contract
                );
            }
        }
        Command::ExecuteProviderSamples {
            output_dir,
            provider_family,
            json,
        } => {
            let report = execute_provider_samples(&output_dir, provider_family.as_deref())?;
            if json {
                println!("{}", provider_sample_execute_json(&report));
            } else {
                println!("device_provider_sample_execute_status: {}", report.status);
                println!(
                    "device_provider_sample_execute_output_payload_count: {}",
                    report.output_payload_count
                );
                println!(
                    "device_provider_sample_execute_matched_record_count: {}",
                    report.matched_record_count
                );
                println!(
                    "device_provider_sample_execute_first_provider_family: {}",
                    report.first_provider_family
                );
                println!(
                    "device_provider_sample_execute_first_runner_adapter_id: {}",
                    report.first_provider_runner_adapter_id
                );
                println!(
                    "device_provider_sample_execute_first_runner_adapter_capability_status: {}",
                    report.first_provider_runner_adapter_capability_status
                );
                println!(
                    "device_provider_sample_execute_first_runner_real_device_capable: {}",
                    report.first_provider_runner_real_device_capable
                );
                println!(
                    "device_provider_sample_execute_first_runner_real_device_probe_status: {}",
                    report.first_provider_runner_real_device_probe_status
                );
                println!(
                    "device_provider_sample_execute_first_execution_mode: {}",
                    report.first_provider_execution_mode
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_evidence: {}",
                    report.first_output_payload_evidence
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_comparison_contract: {}",
                    report.first_output_payload_comparison_contract
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_comparison_status: {}",
                    report.first_output_payload_comparison_status
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_input_evidence: {}",
                    report.first_output_payload_input_evidence
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_input_evidence_hash: {}",
                    report.first_output_payload_input_evidence_hash
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_output_kind: {}",
                    report.first_output_payload_native_output_kind
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_output_status: {}",
                    report.first_output_payload_native_output_status
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_output_bytes: {}",
                    report.first_output_payload_native_output_bytes
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_output_hash: {}",
                    report.first_output_payload_native_output_hash
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_execution_contract: {}",
                    report.first_output_payload_native_execution_contract
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_execution_status: {}",
                    report.first_output_payload_native_execution_status
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_device: {}",
                    report.first_output_payload_native_device
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_compute_plan_contract: {}",
                    report.first_output_payload_native_compute_plan_contract
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_compute_plan_status: {}",
                    report.first_output_payload_native_compute_plan_status
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_compute_plan_layer_count: {}",
                    report.first_output_payload_native_compute_plan_layer_count
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_compute_plan_preferred_devices: {}",
                    report.first_output_payload_native_compute_plan_preferred_devices
                );
                println!(
                    "device_provider_sample_execute_first_output_payload_native_compute_plan_supported_devices: {}",
                    report.first_output_payload_native_compute_plan_supported_devices
                );
                println!(
                    "device_provider_sample_execute_next_action: {}",
                    report.next_action
                );
                println!(
                    "device_provider_sample_execute_next_command: {}",
                    report.next_command
                );
            }
        }
    }
    Ok(())
}

fn provider_sample_materialize_json(report: &ProviderSampleMaterializeReport) -> String {
    format!(
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_materialize\",\"status\":\"{}\",\"path\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"materialized_record_count\":{},\"skipped_record_count\":{},\"first_provider_family\":\"{}\",\"first_provider_runner_contract\":\"{}\",\"first_provider_runner_adapter_contract\":\"{}\",\"first_provider_runner_adapter_id\":\"{}\",\"first_provider_runner_adapter_capability_status\":\"{}\",\"first_provider_runner_registry_protocol\":\"{}\",\"first_provider_runner_registry_source\":\"{}\",\"first_provider_runner_real_device_capable\":{},\"first_provider_runner_kind\":\"{}\",\"first_provider_execution_mode\":\"{}\",\"first_provider_execution_comparison_contract\":\"{}\",\"first_provider_execution_comparison_status\":\"{}\",\"first_provider_execution_evidence_status\":\"{}\",\"first_provider_output_payload_contract\":\"{}\",\"first_provider_output_payload_status\":\"{}\",\"first_provider_output_payload_evidence_status\":\"{}\",\"first_provider_output_payload_evidence\":\"{}\",\"first_provider_output_payload_detail\":\"{}\",\"first_provider_output_payload_path\":\"{}\",\"first_provider_output_payload_hash\":\"{}\",\"first_provider_output_payload_attach_status\":\"{}\",\"first_output_evidence\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\",\"return_contract\":\"{}\",\"return_action\":\"{}\",\"return_command\":\"{}\",\"final_output_replay_contract\":\"{}\"}}",
        json_escape(&report.status),
        json_escape(&report.path),
        json_optional_string(report.provider_family_filter.as_deref()),
        json_string_array(&report.provider_families),
        report.record_count,
        report.matched_record_count,
        report.materialized_record_count,
        report.skipped_record_count,
        json_escape(&report.first_provider_family),
        json_escape(&report.first_provider_runner_contract),
        json_escape(&report.first_provider_runner_adapter_contract),
        json_escape(&report.first_provider_runner_adapter_id),
        json_escape(&report.first_provider_runner_adapter_capability_status),
        json_escape(&report.first_provider_runner_registry_protocol),
        json_escape(&report.first_provider_runner_registry_source),
        report.first_provider_runner_real_device_capable,
        json_escape(&report.first_provider_runner_kind),
        json_escape(&report.first_provider_execution_mode),
        json_escape(&report.first_provider_execution_comparison_contract),
        json_escape(&report.first_provider_execution_comparison_status),
        json_escape(&report.first_provider_execution_evidence_status),
        json_escape(&report.first_provider_output_payload_contract),
        json_escape(&report.first_provider_output_payload_status),
        json_escape(&report.first_provider_output_payload_evidence_status),
        json_escape(&report.first_provider_output_payload_evidence),
        json_escape(&report.first_provider_output_payload_detail),
        json_escape(&report.first_provider_output_payload_path),
        json_escape(&report.first_provider_output_payload_hash),
        json_escape(&report.first_provider_output_payload_attach_status),
        json_escape(&report.first_output_evidence),
        json_escape(&report.next_action),
        json_escape(&report.next_command),
        json_escape(&report.return_contract),
        json_escape(&report.return_action),
        json_escape(&report.return_command),
        json_escape(&report.final_output_replay_contract),
    )
}

fn provider_sample_execute_json(
    report: &provider_sample_execute::ProviderSampleExecuteReport,
) -> String {
    format!(
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_execute\",\"status\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"executable_record_count\":{},\"output_payload_count\":{},\"first_provider_family\":\"{}\",\"first_provider_runner_adapter_id\":\"{}\",\"first_provider_runner_adapter_capability_status\":\"{}\",\"first_provider_runner_real_device_capable\":{},\"first_provider_runner_real_device_probe_status\":\"{}\",\"first_provider_execution_mode\":\"{}\",\"first_output_payload_evidence\":\"{}\",\"first_output_payload_comparison_contract\":\"{}\",\"first_output_payload_comparison_status\":\"{}\",\"first_output_payload_input_evidence\":\"{}\",\"first_output_payload_input_evidence_hash\":\"{}\",\"first_output_payload_native_output_kind\":\"{}\",\"first_output_payload_native_output_status\":\"{}\",\"first_output_payload_native_output_bytes\":\"{}\",\"first_output_payload_native_output_hash\":\"{}\",\"first_output_payload_native_execution_contract\":\"{}\",\"first_output_payload_native_execution_status\":\"{}\",\"first_output_payload_native_device\":\"{}\",\"first_output_payload_native_compute_plan_contract\":\"{}\",\"first_output_payload_native_compute_plan_status\":\"{}\",\"first_output_payload_native_compute_plan_layer_count\":\"{}\",\"first_output_payload_native_compute_plan_preferred_devices\":\"{}\",\"first_output_payload_native_compute_plan_supported_devices\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\"}}",
        json_escape(&report.status),
        json_optional_string(report.provider_family_filter.as_deref()),
        json_string_array(&report.provider_families),
        report.record_count,
        report.matched_record_count,
        report.executable_record_count,
        report.output_payload_count,
        json_escape(&report.first_provider_family),
        json_escape(&report.first_provider_runner_adapter_id),
        json_escape(&report.first_provider_runner_adapter_capability_status),
        report.first_provider_runner_real_device_capable,
        json_escape(&report.first_provider_runner_real_device_probe_status),
        json_escape(&report.first_provider_execution_mode),
        json_escape(&report.first_output_payload_evidence),
        json_escape(&report.first_output_payload_comparison_contract),
        json_escape(&report.first_output_payload_comparison_status),
        json_escape(&report.first_output_payload_input_evidence),
        json_escape(&report.first_output_payload_input_evidence_hash),
        json_escape(&report.first_output_payload_native_output_kind),
        json_escape(&report.first_output_payload_native_output_status),
        json_escape(&report.first_output_payload_native_output_bytes),
        json_escape(&report.first_output_payload_native_output_hash),
        json_escape(&report.first_output_payload_native_execution_contract),
        json_escape(&report.first_output_payload_native_execution_status),
        json_escape(&report.first_output_payload_native_device),
        json_escape(&report.first_output_payload_native_compute_plan_contract),
        json_escape(&report.first_output_payload_native_compute_plan_status),
        json_escape(&report.first_output_payload_native_compute_plan_layer_count),
        json_escape(&report.first_output_payload_native_compute_plan_preferred_devices),
        json_escape(&report.first_output_payload_native_compute_plan_supported_devices),
        json_escape(&report.next_action),
        json_escape(&report.next_command),
    )
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

#[cfg(test)]
mod tests {
    use super::{provider_sample_execute_json, provider_sample_materialize_json};
    use crate::{
        provider_sample_execute::ProviderSampleExecuteReport,
        provider_sample_materialize::ProviderSampleMaterializeReport,
    };

    #[test]
    fn materialize_provider_samples_json_exposes_provider_family_discovery() {
        let report = ProviderSampleMaterializeReport {
            path: "out/nuis.nsdb.device-provider-samples.toml".to_owned(),
            provider_family_filter: Some("metal:apple-silicon-gpu".to_owned()),
            provider_families: vec![
                "metal:apple-silicon-gpu".to_owned(),
                "spirv:vulkan-gpu".to_owned(),
            ],
            status: "awaiting-provider-materialization".to_owned(),
            record_count: 2,
            matched_record_count: 1,
            materialized_record_count: 1,
            skipped_record_count: 1,
            first_provider_family: "metal:apple-silicon-gpu".to_owned(),
            first_provider_runner_contract: "nuis-provider-runner-v1".to_owned(),
            first_provider_runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
            first_provider_runner_adapter_id: "metal.apple-silicon-gpu.host-simulated".to_owned(),
            first_provider_runner_adapter_capability_status: "registered-host-simulated".to_owned(),
            first_provider_runner_registry_protocol: "nuis-provider-runner-registry-v1".to_owned(),
            first_provider_runner_registry_source: "builtin-nustar-provider-runner-registry"
                .to_owned(),
            first_provider_runner_real_device_capable: false,
            first_provider_runner_kind: "metal-host-simulated-runner".to_owned(),
            first_provider_execution_mode: "host-simulated-provider-runner".to_owned(),
            first_provider_execution_comparison_contract: "nuis-provider-execution-comparison-v1"
                .to_owned(),
            first_provider_execution_comparison_status: "host-fallback-output-comparable"
                .to_owned(),
            first_provider_execution_evidence_status: "host-simulated-output-anchor".to_owned(),
            first_provider_output_payload_contract: "nuis-provider-output-payload-handoff-v1"
                .to_owned(),
            first_provider_output_payload_status: "host-fallback-output-payload-ready".to_owned(),
            first_provider_output_payload_evidence_status: "deterministic-provider-output-anchor"
                .to_owned(),
            first_provider_output_payload_evidence: "nuis.nsdb.provider-output.metal.toml:hash=0x1:status=written".to_owned(),
            first_provider_output_payload_detail:
                "deterministic-provider-output-payload:nuis.nsdb.provider-output.metal.toml:0x1:written".to_owned(),
            first_provider_output_payload_path: "nuis.nsdb.provider-output.metal.toml".to_owned(),
            first_provider_output_payload_hash: "0x1".to_owned(),
            first_provider_output_payload_attach_status: "written".to_owned(),
            first_output_evidence: "metallib:pixelmagic.metallib".to_owned(),
            next_action: "replay-provider-sample".to_owned(),
            next_command: "nsdb replay-plan out --json".to_owned(),
            return_contract: "nsld-final-output-boundary-return-v1".to_owned(),
            return_action: "resume-nsld-final-output-check".to_owned(),
            return_command: "nsld check out --json".to_owned(),
            final_output_replay_contract: "nsdb-payload-execution-replay-plan-v1".to_owned(),
        };

        let json = provider_sample_materialize_json(&report);

        assert!(json.contains("\"provider_family_filter\":\"metal:apple-silicon-gpu\""));
        assert!(json
            .contains("\"provider_families\":[\"metal:apple-silicon-gpu\",\"spirv:vulkan-gpu\"]"));
        assert!(json.contains("\"matched_record_count\":1"));
        assert!(json.contains("\"first_provider_runner_contract\":\"nuis-provider-runner-v1\""));
        assert!(json.contains(
            "\"first_provider_runner_adapter_contract\":\"nuis-provider-runner-adapter-v1\""
        ));
        assert!(json.contains(
            "\"first_provider_runner_adapter_id\":\"metal.apple-silicon-gpu.host-simulated\""
        ));
        assert!(json.contains(
            "\"first_provider_runner_adapter_capability_status\":\"registered-host-simulated\""
        ));
        assert!(json.contains(
            "\"first_provider_runner_registry_protocol\":\"nuis-provider-runner-registry-v1\""
        ));
        assert!(json.contains(
            "\"first_provider_runner_registry_source\":\"builtin-nustar-provider-runner-registry\""
        ));
        assert!(json.contains("\"first_provider_runner_real_device_capable\":false"));
        assert!(json.contains("\"first_provider_runner_kind\":\"metal-host-simulated-runner\""));
        assert!(
            json.contains("\"first_provider_execution_mode\":\"host-simulated-provider-runner\"")
        );
        assert!(json.contains(
            "\"first_provider_execution_comparison_contract\":\"nuis-provider-execution-comparison-v1\""
        ));
        assert!(json.contains(
            "\"first_provider_execution_comparison_status\":\"host-fallback-output-comparable\""
        ));
        assert!(json.contains(
            "\"first_provider_execution_evidence_status\":\"host-simulated-output-anchor\""
        ));
        assert!(json.contains(
            "\"first_provider_output_payload_contract\":\"nuis-provider-output-payload-handoff-v1\""
        ));
        assert!(json.contains(
            "\"first_provider_output_payload_status\":\"host-fallback-output-payload-ready\""
        ));
        assert!(json.contains("\"first_provider_output_payload_evidence\":\"nuis.nsdb.provider-output.metal.toml:hash=0x1:status=written\""));
        assert!(json.contains(
            "\"first_provider_output_payload_path\":\"nuis.nsdb.provider-output.metal.toml\""
        ));
        assert!(json.contains("\"first_provider_output_payload_hash\":\"0x1\""));
        assert!(json.contains("\"first_provider_output_payload_attach_status\":\"written\""));
        assert!(json.contains("\"return_contract\":\"nsld-final-output-boundary-return-v1\""));
        assert!(json.contains("\"return_action\":\"resume-nsld-final-output-check\""));
        assert!(json.contains("\"return_command\":\"nsld check out --json\""));
        assert!(json.contains(
            "\"final_output_replay_contract\":\"nsdb-payload-execution-replay-plan-v1\""
        ));
    }

    #[test]
    fn execute_provider_samples_json_exposes_runner_boundary() {
        let report = ProviderSampleExecuteReport {
            status: "provider-output-payloads-ready".to_owned(),
            provider_family_filter: Some("coreml:apple-ane".to_owned()),
            provider_families: vec!["coreml:apple-ane".to_owned()],
            record_count: 1,
            matched_record_count: 1,
            executable_record_count: 1,
            output_payload_count: 1,
            first_provider_family: "coreml:apple-ane".to_owned(),
            first_provider_runner_adapter_id: "coreml.apple-ane.real-device".to_owned(),
            first_provider_runner_adapter_capability_status: "registered-real-device".to_owned(),
            first_provider_runner_real_device_capable: true,
            first_provider_runner_real_device_probe_status: "real-device-candidate-available"
                .to_owned(),
            first_provider_execution_mode: "real-device-provider-runner".to_owned(),
            first_output_payload_evidence:
                "nuis.nsdb.provider-output.coreml-apple-ane.toml:hash=0x1:status=written".to_owned(),
            first_output_payload_comparison_contract: "nuis-provider-execution-comparison-v1"
                .to_owned(),
            first_output_payload_comparison_status: "ready-for-comparison".to_owned(),
            first_output_payload_input_evidence: "tensor:shape=1x4".to_owned(),
            first_output_payload_input_evidence_hash: "0x1234".to_owned(),
            first_output_payload_native_output_kind: "none".to_owned(),
            first_output_payload_native_output_status: "none".to_owned(),
            first_output_payload_native_output_bytes: "none".to_owned(),
            first_output_payload_native_output_hash: "none".to_owned(),
            first_output_payload_native_execution_contract: "none".to_owned(),
            first_output_payload_native_execution_status: "none".to_owned(),
            first_output_payload_native_device: "none".to_owned(),
            first_output_payload_native_compute_plan_contract: "none".to_owned(),
            first_output_payload_native_compute_plan_status: "none".to_owned(),
            first_output_payload_native_compute_plan_layer_count: "0".to_owned(),
            first_output_payload_native_compute_plan_preferred_devices: "none".to_owned(),
            first_output_payload_native_compute_plan_supported_devices: "none".to_owned(),
            next_action: "materialize-provider-samples".to_owned(),
            next_command: "nsdb materialize-provider-samples out --json".to_owned(),
        };

        let json = provider_sample_execute_json(&report);

        assert!(json.contains("\"matched_record_count\":1"));
        assert!(json.contains("\"first_provider_family\":\"coreml:apple-ane\""));
        assert!(
            json.contains("\"first_provider_runner_adapter_id\":\"coreml.apple-ane.real-device\"")
        );
        assert!(json.contains(
            "\"first_provider_runner_adapter_capability_status\":\"registered-real-device\""
        ));
        assert!(json.contains("\"first_provider_runner_real_device_capable\":true"));
        assert!(json.contains(
            "\"first_provider_runner_real_device_probe_status\":\"real-device-candidate-available\""
        ));
        assert!(json.contains("\"first_provider_execution_mode\":\"real-device-provider-runner\""));
        assert!(json.contains(
            "\"first_output_payload_comparison_contract\":\"nuis-provider-execution-comparison-v1\""
        ));
        assert!(
            json.contains("\"first_output_payload_comparison_status\":\"ready-for-comparison\"")
        );
        assert!(json.contains("\"first_output_payload_input_evidence\":\"tensor:shape=1x4\""));
        assert!(json.contains("\"first_output_payload_input_evidence_hash\":\"0x1234\""));
        assert!(json.contains("\"first_output_payload_native_output_kind\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_output_hash\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_execution_contract\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_execution_status\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_device\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_compute_plan_status\":\"none\""));
    }
}
