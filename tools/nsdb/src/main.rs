mod cli;
mod display;
mod handoff;
mod hetero_trace;
mod json;
mod model;
mod payload_decoder;
mod provider_sample;
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
    provider_sample_materialize::materialize_provider_samples,
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
                println!("device_provider_sample_next_action: {}", report.next_action);
                println!(
                    "device_provider_sample_next_command: {}",
                    report.next_command
                );
                println!(
                    "device_provider_sample_return_action: {}",
                    report.return_action
                );
                println!(
                    "device_provider_sample_return_command: {}",
                    report.return_command
                );
            }
        }
    }
    Ok(())
}

fn provider_sample_materialize_json(
    report: &provider_sample_materialize::ProviderSampleMaterializeReport,
) -> String {
    format!(
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_materialize\",\"status\":\"{}\",\"path\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"materialized_record_count\":{},\"skipped_record_count\":{},\"first_provider_family\":\"{}\",\"first_output_evidence\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\",\"return_action\":\"{}\",\"return_command\":\"{}\"}}",
        json_escape(&report.status),
        json_escape(&report.path),
        json_optional_string(report.provider_family_filter.as_deref()),
        json_string_array(&report.provider_families),
        report.record_count,
        report.matched_record_count,
        report.materialized_record_count,
        report.skipped_record_count,
        json_escape(&report.first_provider_family),
        json_escape(&report.first_output_evidence),
        json_escape(&report.next_action),
        json_escape(&report.next_command),
        json_escape(&report.return_action),
        json_escape(&report.return_command),
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
    use super::provider_sample_materialize_json;
    use crate::provider_sample_materialize::ProviderSampleMaterializeReport;

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
            first_output_evidence: "metallib:pixelmagic.metallib".to_owned(),
            next_action: "replay-provider-sample".to_owned(),
            next_command: "nsdb replay-plan out --json".to_owned(),
            return_action: "resume-nsld-final-output-check".to_owned(),
            return_command: "nsld check out --json".to_owned(),
        };

        let json = provider_sample_materialize_json(&report);

        assert!(json.contains("\"provider_family_filter\":\"metal:apple-silicon-gpu\""));
        assert!(json
            .contains("\"provider_families\":[\"metal:apple-silicon-gpu\",\"spirv:vulkan-gpu\"]"));
        assert!(json.contains("\"matched_record_count\":1"));
        assert!(json.contains("\"return_action\":\"resume-nsld-final-output-check\""));
        assert!(json.contains("\"return_command\":\"nsld check out --json\""));
    }
}
