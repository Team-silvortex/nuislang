use super::{
    assembly::nsld_section_manifest_report,
    container_verify::{self, TomlFieldKind},
    fnv1a64_hex,
    reports::{NsldObjectPlanEmitReport, NsldObjectPlanReport, NsldObjectPlanVerifyReport},
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectPlanReport {
    let section_manifest = nsld_section_manifest_report(manifest, plan);
    let output_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-plan.toml");
    let source_container_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container")
        .display()
        .to_string();
    let source_payload_path = PathBuf::from(&plan.output_dir)
        .join("nuis.nsld.container.payload")
        .display()
        .to_string();
    let mut blockers = section_manifest.blockers.clone();
    blockers.push("object-byte-emitter:not-implemented".to_owned());
    blockers.push("native-relocation-applier:not-implemented".to_owned());
    let object_sections = section_manifest
        .sections
        .iter()
        .map(|section| super::reports::NsldObjectSectionDiagnostic {
            order_index: section.order_index,
            source_section_id: section.section_id.clone(),
            source_section_kind: section.section_kind.clone(),
            object_section_name: object_section_name(&section.section_kind, section.order_index),
            object_section_role: object_section_role(&section.section_kind),
            source_path: section.source_path.clone(),
            source_hash: section.source_hash.clone(),
            payload_offset_seed: section.order_index,
            required: section.required,
        })
        .collect::<Vec<_>>();
    let object_plan_hash = nsld_object_plan_hash(
        &plan.cpu_target.machine_arch,
        &plan.cpu_target.machine_os,
        &plan.cpu_target.object_format,
        &section_manifest.section_table_hash,
        &source_container_path,
        &source_payload_path,
        &object_sections,
        &blockers,
    );

    NsldObjectPlanReport {
        manifest: manifest.display().to_string(),
        ready: section_manifest.ready && blockers.is_empty(),
        target_arch: plan.cpu_target.machine_arch.clone(),
        target_os: plan.cpu_target.machine_os.clone(),
        object_format: plan.cpu_target.object_format.clone(),
        calling_abi: plan.cpu_target.calling_abi.clone(),
        clang_target: plan.cpu_target.clang_target.clone(),
        output_path: output_path.display().to_string(),
        source_container_path,
        source_payload_path,
        section_count: section_manifest.section_count,
        section_table_hash: section_manifest.section_table_hash,
        object_plan_hash,
        emission_status: "plan-only".to_owned(),
        object_sections,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectPlanEmitReport, String> {
    let report = nsld_object_plan_report(manifest, plan);
    fs::write(&report.output_path, toml::render_object_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld object plan `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectPlanEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        ready: report.ready,
        object_plan_hash: report.object_plan_hash,
        section_count: report.section_count,
    })
}

pub(crate) fn nsld_verify_object_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectPlanVerifyReport {
    let expected_report = nsld_object_plan_report(manifest, plan);
    let expected = toml::render_object_plan(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-plan.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_object_plan_hash, actual_section_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::usize_value(source, "section_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-plan-content-mismatch".to_owned());
        }
        issues.extend(object_section_table_field_issues(&actual));
        if actual_object_plan_hash.as_deref() != Some(expected_report.object_plan_hash.as_str()) {
            issues.push(format!(
                "object_plan_hash mismatch: expected {}, found {}",
                expected_report.object_plan_hash,
                actual_object_plan_hash
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

    NsldObjectPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected_report.object_plan_hash,
        expected_section_count: expected_report.section_count,
        actual_object_plan_hash,
        actual_section_count,
        issues,
    }
}

fn nsld_object_plan_hash(
    target_arch: &str,
    target_os: &str,
    object_format: &str,
    section_table_hash: &str,
    source_container_path: &str,
    source_payload_path: &str,
    object_sections: &[super::reports::NsldObjectSectionDiagnostic],
    blockers: &[String],
) -> String {
    let section_material = object_sections
        .iter()
        .map(|section| {
            format!(
                "{}:{}:{}:{}:{}:{}",
                section.order_index,
                section.source_section_id,
                section.source_section_kind,
                section.object_section_name,
                section.object_section_role,
                section.payload_offset_seed
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let material = format!(
        "target_arch={target_arch}\ntarget_os={target_os}\nobject_format={object_format}\nsection_table_hash={section_table_hash}\nsource_container_path={source_container_path}\nsource_payload_path={source_payload_path}\nobject_sections={section_material}\nblockers={}\n",
        blockers.join("|")
    );
    fnv1a64_hex(material.as_bytes())
}

fn object_section_table_field_issues(source: &str) -> Vec<String> {
    container_verify::table_field_issues(
        source,
        "object_section",
        "object_section",
        &[
            ("order_index", TomlFieldKind::Usize),
            ("source_section_id", TomlFieldKind::String),
            ("source_section_kind", TomlFieldKind::String),
            ("object_section_name", TomlFieldKind::String),
            ("object_section_role", TomlFieldKind::String),
            ("source_path", TomlFieldKind::String),
            ("source_hash", TomlFieldKind::String),
            ("payload_offset_seed", TomlFieldKind::Usize),
            ("required", TomlFieldKind::Bool),
        ],
    )
}

fn object_section_name(section_kind: &str, order_index: usize) -> String {
    match section_kind {
        "compiled-artifact" => ".nuis.text.compiled".to_owned(),
        "nsld-link-input-table" => ".nuis.meta.link_inputs".to_owned(),
        "nsld-link-unit-table" => ".nuis.meta.link_units".to_owned(),
        "nsld-link-bundle" => ".nuis.meta.link_bundle".to_owned(),
        "lowering-sidecar-input" => format!(".nuis.ir.sidecar.{order_index:04}"),
        "hetero-data-segment" => format!(".nuis.data.hetero.{order_index:04}"),
        other => format!(
            ".nuis.section.{}.{}",
            order_index,
            sanitize_section_token(other)
        ),
    }
}

fn object_section_role(section_kind: &str) -> String {
    match section_kind {
        "compiled-artifact" => "native-bootstrap-input".to_owned(),
        "nsld-link-input-table"
        | "nsld-link-unit-table"
        | "nsld-link-bundle"
        | "lowering-sidecar-input" => "metadata".to_owned(),
        "hetero-data-segment" => "data".to_owned(),
        _ => "extension".to_owned(),
    }
}

fn sanitize_section_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{nsld_object_plan_report, nsld_verify_object_plan_report};
    use crate::main_test_support::empty_link_plan;
    use std::{fs, path::Path};

    #[test]
    fn object_plan_is_plan_only_until_object_writer_exists() {
        let plan = empty_link_plan();
        let report = nsld_object_plan_report(Path::new("nuis.build.manifest.toml"), &plan);

        assert_eq!(report.object_format, "mach-o");
        assert_eq!(report.emission_status, "plan-only");
        assert_eq!(
            report.object_sections[0].object_section_name,
            ".nuis.text.compiled"
        );
        assert_eq!(
            report.object_sections[0].object_section_role,
            "native-bootstrap-input"
        );
        assert!(report
            .blockers
            .contains(&"object-byte-emitter:not-implemented".to_owned()));
        assert!(report
            .blockers
            .contains(&"native-relocation-applier:not-implemented".to_owned()));
    }

    #[test]
    fn verify_object_plan_reports_missing_object_section_fields() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-plan-field-tamper-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();
        let report = nsld_object_plan_report(Path::new("manifest.toml"), &plan);
        let damaged = crate::toml::render_object_plan(&report)
            .replace("object_section_role = \"", "# object_section_role = \"");
        fs::write(dir.join("nuis.nsld.object-plan.toml"), damaged).unwrap();

        let verify = nsld_verify_object_plan_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!verify.valid);
        assert!(verify
            .issues
            .iter()
            .any(|issue| issue == "object_section[0].object_section_role missing"));
    }
}
