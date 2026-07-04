use super::{
    fnv1a64_hex,
    object_file_layout::nsld_object_file_layout_report,
    object_image_backend::{
        encode_object_image_for_backend, object_image_backend_family, object_image_backend_status,
    },
    reports::{
        NsldObjectImageDryRunEmitReport, NsldObjectImageDryRunReport,
        NsldObjectImageDryRunVerifyReport,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_object_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectImageDryRunReport {
    let file_layout = nsld_object_file_layout_report(manifest, plan);
    let image_result = encode_object_image_for_backend(manifest, plan, &file_layout);
    let mut blockers = file_layout.blockers.clone();
    if !file_layout.layout_ready {
        blockers.push("object-file-layout:not-ready".to_owned());
    }
    blockers.extend(image_result.blockers);
    let image = image_result.image;
    let image_size_bytes = image.as_ref().map(Vec::len);
    let image_hash = image.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let image_constructed = image.is_some();
    let image_ready = image_constructed && file_layout.layout_ready && blockers.is_empty();

    NsldObjectImageDryRunReport {
        manifest: manifest.display().to_string(),
        output_path: PathBuf::from(&plan.output_dir)
            .join("nuis.nsld.object-image-dry-run.toml")
            .display()
            .to_string(),
        image_path: object_image_dry_run_image_path(plan).display().to_string(),
        writer_target_id: file_layout.writer_target_id,
        writer_backend_kind: file_layout.writer_backend_kind.clone(),
        object_family: file_layout.object_family,
        backend_family: object_image_backend_family(&file_layout.writer_backend_kind).to_owned(),
        backend_status: object_image_backend_status(&file_layout.writer_backend_kind).to_owned(),
        backend_kind: file_layout.writer_backend_kind,
        object_format: file_layout.object_format,
        file_layout_hash: file_layout.file_layout_hash,
        record_count: file_layout.record_count,
        total_file_size_bytes: file_layout.total_file_size_bytes,
        image_constructed,
        image_ready,
        image_size_bytes,
        image_hash,
        blockers,
    }
}

pub(crate) fn nsld_emit_object_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldObjectImageDryRunEmitReport, String> {
    let report = nsld_object_image_dry_run_report(manifest, plan);
    let image = encode_object_image_dry_run(manifest, plan);
    let image_emitted = match image {
        Some(bytes) => {
            fs::write(&report.image_path, bytes).map_err(|error| {
                format!(
                    "failed to write nsld object image dry run bytes `{}`: {error}",
                    report.image_path
                )
            })?;
            true
        }
        None => false,
    };
    fs::write(
        &report.output_path,
        toml::render_object_image_dry_run(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld object image dry run `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldObjectImageDryRunEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        image_path: report.image_path,
        image_emitted,
        image_constructed: report.image_constructed,
        image_ready: report.image_ready,
        image_size_bytes: report.image_size_bytes,
        image_hash: report.image_hash,
    })
}

pub(crate) fn nsld_verify_object_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectImageDryRunVerifyReport {
    let expected_report = nsld_object_image_dry_run_report(manifest, plan);
    let expected = toml::render_object_image_dry_run(&expected_report);
    let input_path = PathBuf::from(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
    let image_path = object_image_dry_run_image_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_image_dry_run `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_file_layout_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_backend_family,
        actual_backend_status,
        actual_image_constructed,
        actual_image_ready,
        actual_image_size_bytes,
        actual_image_hash,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "file_layout_hash"),
            toml::string_value(source, "writer_backend_kind"),
            toml::string_value(source, "object_family"),
            toml::string_value(source, "backend_family"),
            toml::string_value(source, "backend_status"),
            toml::bool_value(source, "image_constructed"),
            toml::bool_value(source, "image_ready"),
            optional_usize_value(source, "image_size_bytes"),
            optional_string_value(source, "image_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected {
            issues.push("object-image-dry-run-content-mismatch".to_owned());
        }
        push_string_mismatch(
            &mut issues,
            "file_layout_hash",
            &expected_report.file_layout_hash,
            actual_file_layout_hash.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "writer_backend_kind",
            &expected_report.writer_backend_kind,
            actual_writer_backend_kind.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "object_family",
            &expected_report.object_family,
            actual_object_family.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "backend_family",
            &expected_report.backend_family,
            actual_backend_family.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "backend_status",
            &expected_report.backend_status,
            actual_backend_status.as_deref(),
        );
        push_bool_mismatch(
            &mut issues,
            "image_constructed",
            expected_report.image_constructed,
            actual_image_constructed,
        );
        push_bool_mismatch(
            &mut issues,
            "image_ready",
            expected_report.image_ready,
            actual_image_ready,
        );
        push_optional_usize_mismatch(
            &mut issues,
            "image_size_bytes",
            expected_report.image_size_bytes,
            actual_image_size_bytes,
        );
        push_optional_string_mismatch(
            &mut issues,
            "image_hash",
            expected_report.image_hash.as_deref(),
            actual_image_hash.as_deref(),
        );
    }
    let (actual_image_file_size_bytes, actual_image_file_hash) =
        image_file_size_and_hash(&image_path).unwrap_or_else(|error| {
            if expected_report.image_constructed {
                issues.push(error);
            }
            (None, None)
        });
    push_optional_usize_mismatch(
        &mut issues,
        "image_file_size_bytes",
        expected_report.image_size_bytes,
        actual_image_file_size_bytes,
    );
    push_optional_string_mismatch(
        &mut issues,
        "image_file_hash",
        expected_report.image_hash.as_deref(),
        actual_image_file_hash.as_deref(),
    );

    NsldObjectImageDryRunVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        image_path: image_path.display().to_string(),
        valid: issues.is_empty(),
        expected_writer_backend_kind: expected_report.writer_backend_kind,
        expected_object_family: expected_report.object_family,
        expected_backend_family: expected_report.backend_family,
        expected_backend_status: expected_report.backend_status,
        expected_file_layout_hash: expected_report.file_layout_hash,
        expected_image_constructed: expected_report.image_constructed,
        expected_image_ready: expected_report.image_ready,
        expected_image_size_bytes: expected_report.image_size_bytes,
        expected_image_hash: expected_report.image_hash,
        actual_file_layout_hash,
        actual_writer_backend_kind,
        actual_object_family,
        actual_backend_family,
        actual_backend_status,
        actual_image_constructed,
        actual_image_ready,
        actual_image_size_bytes,
        actual_image_hash,
        actual_image_file_size_bytes,
        actual_image_file_hash,
        issues,
    }
}

fn encode_object_image_dry_run(manifest: &Path, plan: &nuisc::linker::LinkPlan) -> Option<Vec<u8>> {
    let file_layout = nsld_object_file_layout_report(manifest, plan);
    encode_object_image_for_backend(manifest, plan, &file_layout).image
}

fn object_image_dry_run_image_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    PathBuf::from(&plan.output_dir).join("nuis.nsld.object-image-dry-run.bin")
}

fn image_file_size_and_hash(path: &Path) -> Result<(Option<usize>, Option<String>), String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "missing_or_unreadable_object_image_dry_run_bytes `{}`: {error}",
            path.display()
        )
    })?;
    Ok((Some(bytes.len()), Some(fnv1a64_hex(&bytes))))
}

fn optional_string_value(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn optional_usize_value(source: &str, key: &str) -> Option<usize> {
    toml::usize_value(source, key).filter(|value| *value != 0)
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

fn push_bool_mismatch(issues: &mut Vec<String>, field: &str, expected: bool, actual: Option<bool>) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {expected}, found {}",
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}

fn push_optional_usize_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: Option<usize>,
    actual: Option<usize>,
) {
    if actual != expected {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            optional_usize_text(expected),
            optional_usize_text(actual)
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

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        nsld_emit_object_image_dry_run_report, nsld_object_image_dry_run_report,
        nsld_verify_object_image_dry_run_report,
    };
    use crate::main_test_support::empty_link_plan;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn reports_mach_o_image_hash_when_encoder_can_construct_image() {
        let plan = empty_link_plan();
        let report = nsld_object_image_dry_run_report(Path::new("manifest.toml"), &plan);

        assert_eq!(report.writer_backend_kind, "mach-o-arm64");
        assert_eq!(report.object_family, "mach-o");
        assert_eq!(report.backend_kind, "mach-o-arm64");
        assert_eq!(report.backend_family, "mach-o");
        assert_eq!(report.backend_status, "ready");
        assert!(report.image_constructed);
        assert_eq!(report.image_size_bytes, Some(report.total_file_size_bytes));
        assert!(report.image_hash.as_deref().unwrap().starts_with("0x"));
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker.starts_with("object-file-layout:")));
    }

    #[test]
    fn object_image_dry_run_serializes_writer_identity() {
        let plan = empty_link_plan();
        let report = nsld_object_image_dry_run_report(Path::new("manifest.toml"), &plan);
        let rendered = crate::toml::render_object_image_dry_run(&report);
        let json = crate::json_object_image::nsld_object_image_dry_run_report_json(&report);

        assert!(rendered.contains("writer_backend_kind = \"mach-o-arm64\""));
        assert!(rendered.contains("object_family = \"mach-o\""));
        assert!(json.contains("\"writer_backend_kind\":\"mach-o-arm64\""));
        assert!(json.contains("\"object_family\":\"mach-o\""));
    }

    #[test]
    fn verify_object_image_dry_run_reports_writer_identity_drift() {
        let mut plan = empty_link_plan();
        plan.output_dir = temp_output_dir("nsld-object-image-dry-run-identity-drift");
        fs::create_dir_all(&plan.output_dir).unwrap();
        let manifest = Path::new("manifest.toml");
        nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();

        let path = Path::new(&plan.output_dir).join("nuis.nsld.object-image-dry-run.toml");
        let damaged = fs::read_to_string(&path).unwrap().replace(
            "writer_backend_kind = \"mach-o-arm64\"",
            "writer_backend_kind = \"elf-amd64\"",
        );
        fs::write(&path, damaged).unwrap();

        let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
        fs::remove_dir_all(&plan.output_dir).unwrap();

        assert!(!verify.valid);
        assert!(verify.issues.iter().any(|issue| {
            issue == "writer_backend_kind mismatch: expected mach-o-arm64, found elf-amd64"
        }));
    }

    #[test]
    fn emits_and_verifies_object_image_dry_run_report() {
        let mut plan = empty_link_plan();
        plan.output_dir = temp_output_dir("nsld-object-image-dry-run");
        fs::create_dir_all(&plan.output_dir).unwrap();
        let manifest = Path::new("manifest.toml");

        let emit = nsld_emit_object_image_dry_run_report(manifest, &plan).unwrap();
        assert!(emit
            .output_path
            .ends_with("nuis.nsld.object-image-dry-run.toml"));
        assert!(emit
            .image_path
            .ends_with("nuis.nsld.object-image-dry-run.bin"));
        assert!(emit.image_emitted);
        assert!(emit.image_constructed);
        assert!(Path::new(&emit.image_path).exists());

        let verify = nsld_verify_object_image_dry_run_report(manifest, &plan);
        assert!(verify.valid, "{:?}", verify.issues);
        assert_eq!(
            verify.actual_image_hash.as_deref(),
            verify.expected_image_hash.as_deref()
        );
        assert_eq!(verify.actual_backend_family.as_deref(), Some("mach-o"));
        assert_eq!(verify.actual_backend_status.as_deref(), Some("ready"));
        assert_eq!(
            verify.actual_image_file_hash.as_deref(),
            verify.expected_image_hash.as_deref()
        );
        assert_eq!(
            verify.actual_image_file_size_bytes,
            verify.expected_image_size_bytes
        );
    }

    fn temp_output_dir(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from(std::env::temp_dir())
            .join(format!("{prefix}-{nanos}"))
            .display()
            .to_string()
    }
}
