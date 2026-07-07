use super::{
    artifact_chain::{nsld_artifact_stage_kind_path_for_plan, NsldArtifactStageKind},
    object_image_dry_run::{
        nsld_emit_object_image_dry_run_report, nsld_verify_object_image_dry_run_report,
    },
    object_plan::nsld_object_plan_report,
    object_writer_backend::{object_writer_backend, object_writer_backend_readiness},
    reports::{
        NsldObjectEmitReport, NsldObjectEmitVerifyReport, NsldObjectWriterReadinessReport,
        NsldObjectWriterStageDiagnostic,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
#[path = "object_emit_tests.rs"]
mod tests;

pub(crate) fn nsld_object_writer_readiness_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectWriterReadinessReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let backend = object_writer_backend(
        &object_plan.target_arch,
        &object_plan.target_os,
        &object_plan.object_format,
    );
    let readiness =
        object_writer_backend_readiness(&backend, object_plan.ready, &object_plan.blockers);
    NsldObjectWriterReadinessReport {
        manifest: object_plan.manifest,
        writer_target_id: readiness.target_id,
        writer_status: readiness.status,
        object_plan_hash: object_plan.object_plan_hash,
        section_count: object_plan.section_count,
        can_emit_object: readiness.can_emit_object,
        writer_stages: backend
            .writer_stages
            .into_iter()
            .map(|stage| NsldObjectWriterStageDiagnostic {
                stage_id: stage.stage_id,
                status: stage.status,
                required: stage.required,
            })
            .collect(),
        unsupported_features: readiness.unsupported_features,
        blockers: readiness.blockers,
    }
}

pub(crate) fn nsld_emit_object_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectEmitReport, String> {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let readiness = nsld_object_writer_readiness_report(manifest, plan);
    let writer_input_path =
        PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-input.toml");
    let blocked_report_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    fs::write(
        &writer_input_path,
        toml::render_object_writer_input(&object_plan),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object writer input `{}`: {error}",
            writer_input_path.display()
        )
    })?;
    let image_dry_run = nsld_emit_object_image_dry_run_report(manifest, plan)?;
    let output_path =
        nsld_artifact_stage_kind_path_for_plan(plan, NsldArtifactStageKind::ObjectOutput);
    let emitted = readiness.can_emit_object && image_dry_run.image_hash.is_some();
    if emitted {
        fs::copy(&image_dry_run.image_path, &output_path).map_err(|error| {
            format!(
                "failed to write nsld object `{}` from dry-run image `{}`: {error}",
                output_path.display(),
                image_dry_run.image_path
            )
        })?;
    }
    let report = NsldObjectEmitReport {
        manifest: readiness.manifest,
        output_path: output_path.display().to_string(),
        writer_input_path: writer_input_path.display().to_string(),
        blocked_report_path: blocked_report_path.display().to_string(),
        image_dry_run_report_path: image_dry_run.output_path,
        image_dry_run_path: image_dry_run.image_path,
        image_dry_run_hash: image_dry_run.image_hash,
        writer_target_id: readiness.writer_target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_plan_hash: readiness.object_plan_hash,
        emitted,
        can_emit_object: readiness.can_emit_object,
        blockers: readiness.blockers,
    };
    fs::write(
        &blocked_report_path,
        toml::render_object_emit_blocked(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object emit report `{}`: {error}",
            blocked_report_path.display()
        )
    })?;
    Ok(report)
}

pub(crate) fn nsld_verify_object_emit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectEmitVerifyReport {
    let expected = nsld_emit_object_report_shape(manifest, plan);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_emit_blocked `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_image_dry_run_hash,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "object_plan_hash"),
            toml::string_value(source, "writer_backend_kind"),
            toml::string_value(source, "object_family"),
            optional_string_value(source, "image_dry_run_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None)
        }
    };
    if actual.is_ok() {
        push_string_mismatch(
            &mut issues,
            "object_plan_hash",
            &expected.object_plan_hash,
            actual_object_plan_hash.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "writer_backend_kind",
            Some(&expected.writer_backend_kind),
            actual_writer_backend_kind.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "object_family",
            Some(&expected.object_family),
            actual_object_family.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "image_dry_run_hash",
            expected.image_dry_run_hash.as_deref(),
            actual_image_dry_run_hash.as_deref(),
        );
    }
    let image_verify = nsld_verify_object_image_dry_run_report(manifest, plan);
    if !image_verify.valid {
        issues.push("object-image-dry-run:invalid".to_owned());
        issues.extend(
            image_verify
                .issues
                .iter()
                .map(|issue| format!("object-image-dry-run:{issue}")),
        );
    }
    if image_verify.actual_image_file_hash != expected.image_dry_run_hash {
        push_optional_string_mismatch(
            &mut issues,
            "image_dry_run_file_hash",
            expected.image_dry_run_hash.as_deref(),
            image_verify.actual_image_file_hash.as_deref(),
        );
    }

    NsldObjectEmitVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_object_plan_hash: expected.object_plan_hash,
        expected_writer_backend_kind: expected.writer_backend_kind,
        expected_object_family: expected.object_family,
        expected_image_dry_run_hash: expected.image_dry_run_hash,
        actual_object_plan_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_image_dry_run_hash,
        image_dry_run_report_valid: image_verify.valid,
        issues,
    }
}

fn nsld_emit_object_report_shape(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectEmitReport {
    let object_plan = nsld_object_plan_report(manifest, plan);
    let readiness = nsld_object_writer_readiness_report(manifest, plan);
    let image_dry_run =
        super::object_image_dry_run::nsld_object_image_dry_run_report(manifest, plan);
    let emitted = readiness.can_emit_object && image_dry_run.image_hash.is_some();
    let writer_input_path =
        PathBuf::from(&plan.output_dir).join("nuis.nsld.object-writer-input.toml");
    let blocked_report_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object.blocked.toml");
    NsldObjectEmitReport {
        manifest: readiness.manifest,
        output_path: nsld_artifact_stage_kind_path_for_plan(
            plan,
            NsldArtifactStageKind::ObjectOutput,
        )
        .display()
        .to_string(),
        writer_input_path: writer_input_path.display().to_string(),
        blocked_report_path: blocked_report_path.display().to_string(),
        image_dry_run_report_path: image_dry_run.output_path,
        image_dry_run_path: image_dry_run.image_path,
        image_dry_run_hash: image_dry_run.image_hash,
        writer_target_id: readiness.writer_target_id,
        writer_backend_kind: object_plan.writer_backend_kind,
        object_family: object_plan.object_family,
        object_plan_hash: readiness.object_plan_hash,
        emitted,
        can_emit_object: readiness.can_emit_object,
        blockers: readiness.blockers,
    }
}

fn optional_string_value(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: Option<&str>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual.unwrap_or("missing")
        ));
    }
}

fn push_optional_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    if actual != expected {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            expected.unwrap_or("missing"),
            actual.unwrap_or("missing")
        ));
    }
}
