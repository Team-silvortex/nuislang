use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    fnv1a64_hex,
    reports::NsldObjectOutputVerifyReport,
};
use std::{fs, path::Path};

pub(crate) fn nsld_verify_object_output_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldObjectOutputVerifyReport {
    let object_output_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectOutput);
    let image_dry_run_path = nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::ObjectImageDryRunBytes,
    );
    let object_bytes = fs::read(&object_output_path);
    let image_bytes = fs::read(&image_dry_run_path);
    let expected_size_bytes = image_bytes.as_ref().ok().map(Vec::len);
    let actual_size_bytes = object_bytes.as_ref().ok().map(Vec::len);
    let expected_hash = image_bytes.as_ref().ok().map(|bytes| fnv1a64_hex(bytes));
    let actual_hash = object_bytes.as_ref().ok().map(|bytes| fnv1a64_hex(bytes));
    let mut issues = Vec::new();
    if let Err(error) = &object_bytes {
        issues.push(format!(
            "missing_or_unreadable_object_output `{}`: {error}",
            object_output_path.display()
        ));
    }
    if let Err(error) = &image_bytes {
        issues.push(format!(
            "missing_or_unreadable_object_image_dry_run_bytes `{}`: {error}",
            image_dry_run_path.display()
        ));
    }
    if let (Some(expected), Some(actual)) = (expected_size_bytes, actual_size_bytes) {
        if expected != actual {
            issues.push(format!(
                "object_output_size mismatch: expected {expected}, found {actual}"
            ));
        }
    }
    if let (Some(expected), Some(actual)) = (expected_hash.as_deref(), actual_hash.as_deref()) {
        if expected != actual {
            issues.push(format!(
                "object_output_hash mismatch: expected {expected}, found {actual}"
            ));
        }
    }

    NsldObjectOutputVerifyReport {
        manifest: manifest.display().to_string(),
        object_output_path: object_output_path.display().to_string(),
        image_dry_run_path: image_dry_run_path.display().to_string(),
        valid: issues.is_empty(),
        expected_size_bytes,
        actual_size_bytes,
        expected_hash,
        actual_hash,
        issues,
    }
}

pub(crate) fn nsld_object_output_issues(plan: &nuisc::linker::LinkPlan) -> Vec<String> {
    nsld_verify_object_output_report(Path::new("manifest.toml"), plan).issues
}

#[cfg(test)]
mod tests {
    use super::nsld_verify_object_output_report;
    use crate::{main_test_support::empty_link_plan, prepare::nsld_prepare_report};
    use std::{fs, path::Path};

    #[test]
    fn verify_object_output_accepts_prepared_mach_o_output() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-output-verify-ok-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();

        nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
        let report = nsld_verify_object_output_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(report.valid, "{:?}", report.issues);
        assert!(report.object_output_path.ends_with("nuis.nsld.mach-o"));
        assert!(report
            .image_dry_run_path
            .ends_with("nuis.nsld.object-image-dry-run.bin"));
        assert_eq!(report.expected_size_bytes, report.actual_size_bytes);
        assert_eq!(report.expected_hash, report.actual_hash);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn verify_object_output_rejects_drifted_mach_o_output() {
        let dir = std::env::temp_dir().join(format!(
            "nsld-object-output-verify-drift-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let artifact_path = dir.join("nuis.compiled.artifact");
        fs::write(&artifact_path, b"compiled-artifact").unwrap();
        let mut plan = empty_link_plan();
        plan.output_dir = dir.display().to_string();
        plan.compiled_artifact.path = artifact_path.display().to_string();

        nsld_prepare_report(Path::new("manifest.toml"), &plan).unwrap();
        fs::write(dir.join("nuis.nsld.mach-o"), b"drifted-object").unwrap();
        let report = nsld_verify_object_output_report(Path::new("manifest.toml"), &plan);
        fs::remove_dir_all(dir).unwrap();

        assert!(!report.valid);
        assert_ne!(report.expected_hash, report.actual_hash);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.contains("object_output_hash mismatch")));
    }
}
