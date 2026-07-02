use super::{
    fnv1a64_hex,
    link_units::{
        nsld_link_input_summary, nsld_link_unit_report, nsld_sidecar_capability_diagnostics,
    },
    reports::{NsldLinkBundleEmitReport, NsldLinkBundleReport, NsldLinkBundleVerifyReport},
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_link_bundle_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkBundleReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let unit_report = nsld_link_unit_report(manifest, plan);
    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let mut issues = Vec::new();
    if !plan.artifact_lowering_alignment.consistent {
        issues.push("artifact-lowering-alignment-mismatch".to_owned());
    }
    if !plan.clock_protocol.validation.valid {
        issues.push("clock-protocol-invalid".to_owned());
    }
    if !plan.hetero_calculate.validation.valid {
        issues.push("hetero-calculate-invalid".to_owned());
    }
    if !plan.hetero_calculate.static_link {
        issues.push("hetero-calculate-not-static-link".to_owned());
    }
    if !plan.hetero_calculate.lifecycle_driven {
        issues.push("hetero-calculate-not-lifecycle-driven".to_owned());
    }
    for capability in &sidecar_capabilities {
        for issue in &capability.issues {
            issues.push(format!(
                "sidecar-capability:{}:{}:{}",
                capability.package_id, capability.domain_family, issue
            ));
        }
    }

    let bundle_ready = issues.is_empty();
    let bundle_hash = nsld_link_bundle_hash(
        &unit_report,
        &link_input_summary,
        plan,
        host_wrapper_required,
        bundle_ready,
    );
    let bundle_id = format!("lb.{}", bundle_hash.trim_start_matches("0x"));

    NsldLinkBundleReport {
        manifest: manifest.display().to_string(),
        bundle_id,
        bundle_hash,
        bundle_ready,
        unit_count: unit_report.unit_count,
        hetero_unit_count: unit_report.hetero_unit_count,
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
        unit_table_hash: unit_report.unit_table_hash,
        clock_edge_count: plan.clock_protocol.edges.len(),
        data_segment_count: plan.hetero_calculate.data_segments.len(),
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        host_wrapper_required,
        compiled_artifact_path: plan.compiled_artifact.path.clone(),
        native_output_path: plan.final_stage.output_path.clone(),
        issues,
    }
}

pub(crate) fn nsld_emit_link_bundle_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldLinkBundleEmitReport, String> {
    let report = nsld_link_bundle_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-bundle.toml");
    fs::write(&output_path, toml::render_link_bundle(&report)).map_err(|error| {
        format!(
            "failed to write nsld link bundle `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldLinkBundleEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        bundle_id: report.bundle_id,
        bundle_hash: report.bundle_hash,
        bundle_ready: report.bundle_ready,
    })
}

pub(crate) fn nsld_verify_link_bundle_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkBundleVerifyReport {
    let expected_report = nsld_link_bundle_report(manifest, plan);
    let expected = toml::render_link_bundle(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-bundle.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_link_bundle `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_bundle_id, actual_bundle_hash) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "bundle_id"),
            toml::string_value(source, "bundle_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("link-bundle-content-mismatch".to_owned());
        }
        if actual_bundle_id.as_deref() != Some(expected_report.bundle_id.as_str()) {
            issues.push(format!(
                "bundle_id mismatch: expected {}, found {}",
                expected_report.bundle_id,
                actual_bundle_id
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_bundle_hash.as_deref() != Some(expected_report.bundle_hash.as_str()) {
            issues.push(format!(
                "bundle_hash mismatch: expected {}, found {}",
                expected_report.bundle_hash,
                actual_bundle_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldLinkBundleVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_bundle_id: expected_report.bundle_id,
        expected_bundle_hash: expected_report.bundle_hash,
        actual_bundle_id,
        actual_bundle_hash,
        issues,
    }
}

fn nsld_link_bundle_hash(
    unit_report: &super::reports::NsldLinkUnitReport,
    link_input_summary: &super::reports::NsldLinkInputSummary,
    plan: &nuisc::linker::LinkPlan,
    host_wrapper_required: bool,
    bundle_ready: bool,
) -> String {
    let material = format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
        unit_report.unit_count,
        unit_report.hetero_unit_count,
        link_input_summary.count,
        link_input_summary.total_bytes,
        link_input_summary.table_hash,
        unit_report.unit_table_hash,
        plan.clock_protocol.edges.len(),
        plan.hetero_calculate.data_segments.len(),
        plan.final_stage.link_mode,
        host_wrapper_required,
        bundle_ready
    );
    fnv1a64_hex(material.as_bytes())
}
