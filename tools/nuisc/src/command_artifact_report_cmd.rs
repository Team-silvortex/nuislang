use std::path::PathBuf;

use crate::artifact_report::{artifact_report_json, artifact_report_summary_lines};
use crate::domain_build_report::{
    collect_domain_build_unit_verdicts, summarize_domain_build_verification,
};
use crate::execution_inspect_report::inspect_execution_overview;
use crate::inspect_report::collect_doc_indexes_from_manifest_input;
use crate::project_metadata_report::{
    project_metadata_summary_from_manifest_report, resolve_artifact_report_inputs,
};
use crate::{aot, linker};

pub(crate) fn run_artifact_report(input: PathBuf, json: bool, summary: bool) -> Result<(), String> {
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

    Ok(())
}
