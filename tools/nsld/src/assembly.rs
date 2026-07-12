pub(crate) use super::link_bundle_pipeline::{
    nsld_emit_link_bundle_report, nsld_link_bundle_report, nsld_verify_link_bundle_report,
};

use super::{
    fnv1a64_hex,
    link_units::{nsld_link_input_summary, nsld_sidecar_capability_diagnostics},
    reports::{
        NsldAssemblePlanEmitReport, NsldAssemblePlanReport, NsldAssemblePlanVerifyReport,
        NsldAssembleSectionDiagnostic, NsldSectionManifestEmitReport, NsldSectionManifestReport,
        NsldSectionManifestVerifyReport,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_assemble_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldAssemblePlanReport {
    let bundle = nsld_link_bundle_report(manifest, plan);
    let sidecar_capabilities = nsld_sidecar_capability_diagnostics(plan);
    let link_input_summary = nsld_link_input_summary(&sidecar_capabilities);
    let mut blockers = bundle.issues.clone();
    let mut sections = Vec::new();

    push_assemble_section(
        &mut sections,
        "compiled-artifact",
        &plan.compiled_artifact.path,
        true,
    );
    push_assemble_section(
        &mut sections,
        "nsld-link-input-table",
        &PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.link-inputs.toml")
            .display()
            .to_string(),
        true,
    );
    push_assemble_section(
        &mut sections,
        "nsld-link-unit-table",
        &PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.link-units.toml")
            .display()
            .to_string(),
        true,
    );
    push_assemble_section(
        &mut sections,
        "nsld-link-bundle",
        &PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.link-bundle.toml")
            .display()
            .to_string(),
        true,
    );
    for input in &link_input_summary.inputs {
        push_assemble_section(
            &mut sections,
            &lowering_sidecar_section_kind(&input.domain_family),
            &input.path,
            true,
        );
    }
    for segment in &plan.hetero_calculate.data_segments {
        if let Some(source_path) = &segment.source_path {
            push_assemble_section(&mut sections, "hetero-data-segment", source_path, true);
        } else {
            blockers.push(format!(
                "data-segment:{}:{}:missing-source-path",
                segment.owner_package, segment.segment_id
            ));
        }
    }

    for section in &sections {
        if section.required && section.source_hash == "missing" {
            blockers.push(format!(
                "section:{}:{}:missing-source",
                section.section_kind, section.source_path
            ));
        }
    }

    let assemble_plan_hash =
        nsld_assemble_plan_hash(&bundle.bundle_id, &bundle.bundle_hash, &sections, &blockers);

    NsldAssemblePlanReport {
        manifest: manifest.display().to_string(),
        ready: bundle.bundle_ready && blockers.is_empty(),
        bundle_id: bundle.bundle_id,
        bundle_hash: bundle.bundle_hash,
        assemble_plan_hash,
        section_count: sections.len(),
        sections,
        blockers,
    }
}

pub(crate) fn nsld_emit_assemble_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldAssemblePlanEmitReport, String> {
    let report = nsld_assemble_plan_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.assemble-plan.toml");
    fs::write(&output_path, toml::render_assemble_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld assemble plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldAssemblePlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        assemble_plan_hash: report.assemble_plan_hash,
        section_count: report.section_count,
    })
}

pub(crate) fn nsld_verify_assemble_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldAssemblePlanVerifyReport {
    let expected_report = nsld_assemble_plan_report(manifest, plan);
    let expected = toml::render_assemble_plan(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.assemble-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_assemble_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_assemble_plan_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "assemble_plan_hash"),
            toml::usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("assemble-plan-content-mismatch".to_owned());
        }
        if actual_assemble_plan_hash.as_deref() != Some(expected_report.assemble_plan_hash.as_str())
        {
            issues.push(format!(
                "assemble_plan_hash mismatch: expected {}, found {}",
                expected_report.assemble_plan_hash,
                actual_assemble_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldAssemblePlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_assemble_plan_hash: expected_report.assemble_plan_hash,
        expected_section_count: expected_report.section_count,
        actual_assemble_plan_hash,
        actual_section_count,
        issues,
    }
}

pub(crate) fn nsld_section_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldSectionManifestReport {
    let assemble_plan = nsld_assemble_plan_report(manifest, plan);
    let section_table_hash = nsld_section_table_hash(&assemble_plan.sections);
    NsldSectionManifestReport {
        manifest: manifest.display().to_string(),
        ready: assemble_plan.ready,
        assemble_plan_hash: assemble_plan.assemble_plan_hash,
        section_count: assemble_plan.section_count,
        section_table_hash,
        sections: assemble_plan.sections,
        blockers: assemble_plan.blockers,
    }
}

pub(crate) fn nsld_emit_section_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldSectionManifestEmitReport, String> {
    let report = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.section-manifest.toml");
    fs::write(&output_path, toml::render_section_manifest(&report)).map_err(|error| {
        format!(
            "failed to write nsld section manifest `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldSectionManifestEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        section_count: report.section_count,
        section_table_hash: report.section_table_hash,
    })
}

pub(crate) fn nsld_verify_section_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldSectionManifestVerifyReport {
    let expected_report = nsld_section_manifest_report(manifest, plan);
    let expected = toml::render_section_manifest(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.section-manifest.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_section_manifest `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_section_count, actual_section_table_hash) = match actual.as_ref() {
        Ok(source) => (
            toml::usize_value(source, "section_count"),
            toml::string_value(source, "section_table_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("section-manifest-content-mismatch".to_owned());
        }
        if actual_section_count != Some(expected_report.section_count) {
            issues.push(format!(
                "section_count mismatch: expected {}, found {}",
                expected_report.section_count,
                actual_section_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_section_table_hash.as_deref() != Some(expected_report.section_table_hash.as_str())
        {
            issues.push(format!(
                "section_table_hash mismatch: expected {}, found {}",
                expected_report.section_table_hash,
                actual_section_table_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldSectionManifestVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_section_count: expected_report.section_count,
        expected_section_table_hash: expected_report.section_table_hash,
        actual_section_count,
        actual_section_table_hash,
        issues,
    }
}

pub(crate) fn nsld_section_table_hash(sections: &[NsldAssembleSectionDiagnostic]) -> String {
    let mut material = String::new();
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn nsld_assemble_plan_hash(
    bundle_id: &str,
    bundle_hash: &str,
    sections: &[NsldAssembleSectionDiagnostic],
    blockers: &[String],
) -> String {
    let mut material = String::new();
    material.push_str(bundle_id);
    material.push('\t');
    material.push_str(bundle_hash);
    material.push('\n');
    for section in sections {
        material.push_str(&section.order_index.to_string());
        material.push('\t');
        material.push_str(&section.section_id);
        material.push('\t');
        material.push_str(&section.section_kind);
        material.push('\t');
        material.push_str(&section.source_path);
        material.push('\t');
        material.push_str(&section.source_hash);
        material.push('\t');
        material.push_str(if section.required {
            "required"
        } else {
            "optional"
        });
        material.push('\n');
    }
    for blocker in blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}

fn push_assemble_section(
    sections: &mut Vec<NsldAssembleSectionDiagnostic>,
    section_kind: &str,
    source_path: &str,
    required: bool,
) {
    let order_index = sections.len();
    let source_hash = fs::read(source_path)
        .map(|bytes| fnv1a64_hex(&bytes))
        .unwrap_or_else(|_| "missing".to_owned());
    sections.push(NsldAssembleSectionDiagnostic {
        order_index,
        section_id: format!("sec{order_index:04}.{section_kind}"),
        section_kind: section_kind.to_owned(),
        source_path: source_path.to_owned(),
        source_hash,
        required,
    });
}

fn lowering_sidecar_section_kind(domain_family: &str) -> String {
    match domain_family {
        "shader" => "shader-lowering-sidecar-input".to_owned(),
        "kernel" => "kernel-lowering-sidecar-input".to_owned(),
        _ => "lowering-sidecar-input".to_owned(),
    }
}
