use super::{
    link_units::{nsld_link_input_summary, nsld_sidecar_capability_diagnostics},
    reports::{NsldLinkInputsEmitReport, NsldLinkInputsVerifyReport},
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_emit_link_inputs_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldLinkInputsEmitReport, String> {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let invalid = sidecar_capabilities
        .iter()
        .filter(|capability| !capability.valid)
        .flat_map(|capability| {
            capability.issues.iter().map(|issue| {
                format!(
                    "{}:{}:{}",
                    capability.package_id, capability.domain_family, issue
                )
            })
        })
        .collect::<Vec<_>>();
    if !invalid.is_empty() {
        return Err(format!(
            "cannot emit nsld link inputs while sidecar capabilities are invalid: {}",
            invalid.join(", ")
        ));
    }
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    fs::write(
        &output_path,
        toml::render_link_input_table(
            &link_input_summary.inputs,
            link_input_summary.total_bytes,
            &link_input_summary.table_hash,
        ),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld link input table `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldLinkInputsEmitReport {
        manifest: manifest.display().to_string(),
        output_path: output_path.display().to_string(),
        link_input_count: link_input_summary.count,
        link_input_total_bytes: link_input_summary.total_bytes,
        link_input_table_hash: link_input_summary.table_hash,
    })
}

pub(crate) fn nsld_verify_link_inputs_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldLinkInputsVerifyReport {
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let expected = toml::render_link_input_table(
        &link_input_summary.inputs,
        link_input_summary.total_bytes,
        &link_input_summary.table_hash,
    );
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.link-inputs.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_link_input_table `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_link_input_count, actual_link_input_total_bytes, actual_link_input_table_hash) =
        match actual.as_ref() {
            Ok(source) => (
                toml::usize_value(source, "link_input_count"),
                toml::usize_value(source, "link_input_total_bytes"),
                toml::string_value(source, "link_input_table_hash"),
            ),
            Err(error) => {
                issues.push(error.clone());
                (None, None, None)
            }
        };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("link-input-table-content-mismatch".to_owned());
        }
        if actual_link_input_count != Some(link_input_summary.count) {
            issues.push(format!(
                "link_input_count mismatch: expected {}, found {}",
                link_input_summary.count,
                actual_link_input_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_total_bytes != Some(link_input_summary.total_bytes) {
            issues.push(format!(
                "link_input_total_bytes mismatch: expected {}, found {}",
                link_input_summary.total_bytes,
                actual_link_input_total_bytes
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_link_input_table_hash.as_deref() != Some(link_input_summary.table_hash.as_str()) {
            issues.push(format!(
                "link_input_table_hash mismatch: expected {}, found {}",
                link_input_summary.table_hash,
                actual_link_input_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldLinkInputsVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_link_input_count: link_input_summary.count,
        expected_link_input_total_bytes: link_input_summary.total_bytes,
        expected_link_input_table_hash: link_input_summary.table_hash,
        actual_link_input_count,
        actual_link_input_total_bytes,
        actual_link_input_table_hash,
        issues,
    }
}
