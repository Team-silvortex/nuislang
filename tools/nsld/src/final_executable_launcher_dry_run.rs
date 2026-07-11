use super::{
    final_executable_launcher::nsld_verify_final_executable_launcher_manifest_report,
    final_executable_paths::nsld_final_executable_launcher_dry_run_path,
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableLauncherDryRunEmitReport, NsldFinalExecutableLauncherDryRunReport,
        NsldFinalExecutableLauncherDryRunVerifyReport,
    },
    toml,
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_launcher_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLauncherDryRunReport {
    let verify = nsld_verify_final_executable_launcher_manifest_report(manifest, plan);
    let nsb_path = verify.actual_nsb_path.clone();
    let nsb_bytes = nsb_path.as_deref().and_then(|path| fs::read(path).ok());
    let nsb_hash_actual = nsb_bytes.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let nsb_hash_expected = verify.actual_nsb_hash.clone();
    let nsb_hash_matches = nsb_hash_expected.is_some() && nsb_hash_actual == nsb_hash_expected;
    let mut blockers = Vec::new();
    if !verify.valid {
        blockers.push("host-launcher-manifest:invalid".to_owned());
        blockers.extend(
            verify
                .issues
                .iter()
                .map(|issue| format!("host-launcher-manifest:{issue}")),
        );
    }
    if nsb_path.is_none() {
        blockers.push("host-launcher:nsb-path-missing".to_owned());
    }
    if nsb_bytes.is_none() {
        blockers.push("host-launcher:nsb-unreadable".to_owned());
    }
    if !nsb_hash_matches {
        blockers.push("host-launcher:nsb-hash-mismatch".to_owned());
    }
    if verify.actual_image_header_valid != Some(true) {
        blockers.push("host-launcher:image-header-invalid".to_owned());
    }
    let launch_steps = if blockers.is_empty() {
        verify.actual_verification_steps.clone()
    } else {
        Vec::new()
    };
    let dry_run_ready = blockers.is_empty();
    NsldFinalExecutableLauncherDryRunReport {
        manifest: manifest.display().to_string(),
        launcher_manifest_path: verify.input_path,
        launcher_manifest_valid: verify.valid,
        nsb_path,
        nsb_readable: nsb_bytes.is_some(),
        nsb_hash_expected,
        nsb_hash_actual,
        nsb_hash_matches,
        image_header_valid: verify.actual_image_header_valid,
        entry_lifecycle_hook: verify.actual_entry_lifecycle_hook,
        scheduler_entry: verify.actual_scheduler_entry,
        dry_run_ready,
        would_enter_lifecycle_hook: dry_run_ready,
        launch_steps,
        blockers,
        notes: vec![
            "launcher-dry-run-is-non-executing".to_owned(),
            "launcher-dry-run-does-not-map-or-jump-into-payload-code".to_owned(),
        ],
    }
}

pub(crate) fn nsld_emit_final_executable_launcher_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableLauncherDryRunEmitReport, String> {
    let report = nsld_final_executable_launcher_dry_run_report(manifest, plan);
    let source = render_final_executable_launcher_dry_run(&report);
    let output_path = nsld_final_executable_launcher_dry_run_path(plan);
    fs::write(&output_path, &source).map_err(|error| {
        format!(
            "failed to write nsld final executable launcher dry-run `{}`: {error}",
            output_path.display()
        )
    })?;
    Ok(NsldFinalExecutableLauncherDryRunEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        dry_run_hash: fnv1a64_hex(source.as_bytes()),
        dry_run_ready: report.dry_run_ready,
        blocker_count: report.blockers.len(),
    })
}

pub(crate) fn nsld_verify_final_executable_launcher_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLauncherDryRunVerifyReport {
    let expected = nsld_final_executable_launcher_dry_run_report(manifest, plan);
    let expected_source = render_final_executable_launcher_dry_run(&expected);
    let expected_hash = fnv1a64_hex(expected_source.as_bytes());
    let input_path = nsld_final_executable_launcher_dry_run_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_launcher_dry_run `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_hash,
        actual_dry_run_ready,
        actual_would_enter_lifecycle_hook,
        actual_nsb_hash_actual,
        actual_launch_steps,
        actual_blocker_count,
        actual_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::bool_value(source, "dry_run_ready"),
            toml::bool_value(source, "would_enter_lifecycle_hook"),
            non_empty_toml_string(source, "nsb_hash_actual"),
            toml::string_array_value(source, "launch_steps"),
            toml::usize_value(source, "blocker_count"),
            toml::string_array_value(source, "blockers"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, Vec::new(), None, Vec::new())
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-launcher-dry-run-content-mismatch".to_owned());
        }
        push_bool_mismatch(
            &mut issues,
            "dry_run_ready",
            expected.dry_run_ready,
            actual_dry_run_ready,
        );
        push_bool_mismatch(
            &mut issues,
            "would_enter_lifecycle_hook",
            expected.would_enter_lifecycle_hook,
            actual_would_enter_lifecycle_hook,
        );
        if actual_nsb_hash_actual != expected.nsb_hash_actual {
            issues.push(format!(
                "nsb_hash_actual mismatch: expected {}, found {}",
                expected.nsb_hash_actual.as_deref().unwrap_or("missing"),
                actual_nsb_hash_actual.as_deref().unwrap_or("missing")
            ));
        }
        if actual_launch_steps != expected.launch_steps {
            issues.push(format!(
                "launch_steps mismatch: expected [{}], found [{}]",
                expected.launch_steps.join(", "),
                actual_launch_steps.join(", ")
            ));
        }
        if actual_blocker_count != Some(expected.blockers.len()) {
            issues.push(format!(
                "blocker_count mismatch: expected {}, found {}",
                expected.blockers.len(),
                actual_blocker_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_blockers != expected.blockers {
            issues.push(format!(
                "blockers mismatch: expected [{}], found [{}]",
                expected.blockers.join(", "),
                actual_blockers.join(", ")
            ));
        }
    }
    NsldFinalExecutableLauncherDryRunVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_dry_run_hash: expected_hash,
        actual_dry_run_hash: actual_hash,
        expected_dry_run_ready: expected.dry_run_ready,
        actual_dry_run_ready,
        expected_would_enter_lifecycle_hook: expected.would_enter_lifecycle_hook,
        actual_would_enter_lifecycle_hook,
        expected_nsb_hash_actual: expected.nsb_hash_actual,
        actual_nsb_hash_actual,
        expected_launch_steps: expected.launch_steps,
        actual_launch_steps,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
    }
}

fn render_final_executable_launcher_dry_run(
    report: &NsldFinalExecutableLauncherDryRunReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-host-launcher-dry-run-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    push_str_field(&mut out, "manifest", &report.manifest);
    push_str_field(
        &mut out,
        "launcher_manifest_path",
        &report.launcher_manifest_path,
    );
    out.push_str(&format!(
        "launcher_manifest_valid = {}\n",
        report.launcher_manifest_valid
    ));
    push_str_field(
        &mut out,
        "nsb_path",
        report.nsb_path.as_deref().unwrap_or(""),
    );
    out.push_str(&format!("nsb_readable = {}\n", report.nsb_readable));
    push_str_field(
        &mut out,
        "nsb_hash_expected",
        report.nsb_hash_expected.as_deref().unwrap_or(""),
    );
    push_str_field(
        &mut out,
        "nsb_hash_actual",
        report.nsb_hash_actual.as_deref().unwrap_or(""),
    );
    out.push_str(&format!("nsb_hash_matches = {}\n", report.nsb_hash_matches));
    out.push_str(&format!(
        "image_header_valid = {}\n",
        report.image_header_valid.unwrap_or(false)
    ));
    push_str_field(
        &mut out,
        "entry_lifecycle_hook",
        report.entry_lifecycle_hook.as_deref().unwrap_or(""),
    );
    push_str_field(
        &mut out,
        "scheduler_entry",
        report.scheduler_entry.as_deref().unwrap_or(""),
    );
    out.push_str(&format!("dry_run_ready = {}\n", report.dry_run_ready));
    out.push_str(&format!(
        "would_enter_lifecycle_hook = {}\n",
        report.would_enter_lifecycle_hook
    ));
    out.push_str(&format!(
        "launch_steps = [{}]\n",
        toml::toml_string_array_literal(&report.launch_steps)
    ));
    out.push_str(&format!("blocker_count = {}\n", report.blockers.len()));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml::toml_string_array_literal(&report.blockers)
    ));
    out.push_str(&format!(
        "notes = [{}]\n",
        toml::toml_string_array_literal(&report.notes)
    ));
    out
}

fn push_str_field(out: &mut String, key: &str, value: &str) {
    out.push_str(&format!(
        "{key} = \"{}\"\n",
        toml::escape_toml_string(value)
    ));
}

fn non_empty_toml_string(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn push_bool_mismatch(issues: &mut Vec<String>, field: &str, expected: bool, actual: Option<bool>) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            expected,
            actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_owned())
        ));
    }
}
