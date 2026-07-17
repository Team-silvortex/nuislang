mod cli;
mod display;
mod handoff;
mod hetero_trace;
mod json;
mod model;
mod payload_decoder;
mod provider_runner_registry;
mod provider_sample;
mod provider_sample_execute;
mod provider_sample_execution;
mod provider_sample_materialize;
mod replay;
#[cfg(test)]
mod replay_tests;
mod report;
mod sidecar;

use crate::{
    cli::{parse_args, resolve_manifest_input, Command},
    display::{print_nsdb_events_report, print_nsdb_inspect_report, print_nsdb_replay_plan},
    json::{nsdb_events_report_json, nsdb_inspect_report_json, nsdb_replay_plan_json},
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
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_materialize\",\"status\":\"{}\",\"path\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"materialized_record_count\":{},\"skipped_record_count\":{},\"first_provider_family\":\"{}\",\"first_provider_runner_contract\":\"{}\",\"first_provider_runner_adapter_contract\":\"{}\",\"first_provider_runner_adapter_id\":\"{}\",\"first_provider_runner_adapter_capability_status\":\"{}\",\"first_provider_runner_registry_protocol\":\"{}\",\"first_provider_runner_registry_source\":\"{}\",\"first_provider_runner_real_device_capable\":{},\"first_provider_runner_kind\":\"{}\",\"first_provider_execution_mode\":\"{}\",\"first_provider_execution_comparison_contract\":\"{}\",\"first_provider_execution_comparison_status\":\"{}\",\"first_provider_execution_evidence_status\":\"{}\",\"first_provider_output_payload_contract\":\"{}\",\"first_provider_output_payload_status\":\"{}\",\"first_provider_output_payload_evidence_status\":\"{}\",\"first_provider_output_payload_evidence\":\"{}\",\"first_provider_output_payload_detail\":\"{}\",\"first_output_evidence\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\",\"return_contract\":\"{}\",\"return_action\":\"{}\",\"return_command\":\"{}\",\"final_output_replay_contract\":\"{}\"}}",
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
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_execute\",\"status\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"executable_record_count\":{},\"output_payload_count\":{},\"first_provider_family\":\"{}\",\"first_provider_runner_adapter_id\":\"{}\",\"first_provider_runner_adapter_capability_status\":\"{}\",\"first_provider_runner_real_device_capable\":{},\"first_provider_runner_real_device_probe_status\":\"{}\",\"first_provider_execution_mode\":\"{}\",\"first_output_payload_evidence\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\"}}",
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
    }
}
