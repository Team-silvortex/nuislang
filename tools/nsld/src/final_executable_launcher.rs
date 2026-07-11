use super::{
    final_executable_output::nsld_final_executable_output_report,
    final_executable_paths::nsld_final_executable_launcher_manifest_path,
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableLauncherDryRunReport, NsldFinalExecutableLauncherManifestEmitReport,
        NsldFinalExecutableLauncherManifestReport, NsldFinalExecutableLauncherManifestVerifyReport,
    },
    toml,
};
use std::{env, fs, path::Path};

pub(crate) fn nsld_final_executable_launcher_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLauncherManifestReport {
    let output = nsld_final_executable_output_report(manifest, plan);
    let launcher_manifest_path = nsld_final_executable_launcher_manifest_path(plan);
    let mut blockers = output.blockers.clone();
    if !output.runnable_candidate {
        blockers.push("host-launcher:nsb-not-runnable-candidate".to_owned());
    }
    if !output.output_image_header_valid {
        blockers.push("host-launcher:nsb-image-header-invalid".to_owned());
    }
    let ready = blockers.is_empty();
    NsldFinalExecutableLauncherManifestReport {
        manifest: manifest.display().to_string(),
        output_path: plan.final_stage.output_path.clone(),
        launcher_manifest_path: launcher_manifest_path.display().to_string(),
        ready,
        launcher_kind: "host-launcher-manifest".to_owned(),
        launcher_format: "nuis-host-launcher-manifest-v1".to_owned(),
        host_envelope_family: "host-process-envelope".to_owned(),
        host_os: env::consts::OS.to_owned(),
        host_arch: env::consts::ARCH.to_owned(),
        nsb_path: output.output_path,
        nsb_present: output.present,
        nsb_size_bytes: output.size_bytes,
        nsb_hash: output.output_hash,
        image_header_required: true,
        image_header_valid: output.output_image_header_valid,
        entry_lifecycle_hook: "on_process_start".to_owned(),
        scheduler_entry: "nuis.scheduler.loop.v1".to_owned(),
        verification_steps: vec![
            "read-nsb-header".to_owned(),
            "verify-nsb-magic-and-version".to_owned(),
            "verify-nsb-size-and-hash".to_owned(),
            "map-payload-region".to_owned(),
            "enter-lifecycle-hook:on_process_start".to_owned(),
        ],
        blockers,
        notes: vec![
            "launcher-manifest-is-non-executing".to_owned(),
            "host-launcher-implementation-remains-separate-from-nsld-core-linking".to_owned(),
        ],
    }
}

pub(crate) fn nsld_emit_final_executable_launcher_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableLauncherManifestEmitReport, String> {
    let report = nsld_final_executable_launcher_manifest_report(manifest, plan);
    let source = render_final_executable_launcher_manifest(&report);
    let output_path = nsld_final_executable_launcher_manifest_path(plan);
    fs::write(&output_path, &source).map_err(|error| {
        format!(
            "failed to write nsld final executable launcher manifest `{}`: {error}",
            output_path.display()
        )
    })?;
    Ok(NsldFinalExecutableLauncherManifestEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        launcher_manifest_hash: fnv1a64_hex(source.as_bytes()),
        ready: report.ready,
        blocker_count: report.blockers.len(),
    })
}

pub(crate) fn nsld_verify_final_executable_launcher_manifest_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLauncherManifestVerifyReport {
    let expected = nsld_final_executable_launcher_manifest_report(manifest, plan);
    let expected_source = render_final_executable_launcher_manifest(&expected);
    let expected_hash = fnv1a64_hex(expected_source.as_bytes());
    let input_path = nsld_final_executable_launcher_manifest_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_launcher_manifest `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_hash,
        actual_ready,
        actual_nsb_path,
        actual_nsb_size_bytes,
        actual_nsb_hash,
        actual_image_header_valid,
        actual_entry_lifecycle_hook,
        actual_scheduler_entry,
        actual_verification_steps,
        actual_blocker_count,
        actual_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::bool_value(source, "ready"),
            non_empty_toml_string(source, "nsb_path"),
            optional_usize_value(source, "nsb_size_bytes"),
            non_empty_toml_string(source, "nsb_hash"),
            toml::bool_value(source, "image_header_valid"),
            non_empty_toml_string(source, "entry_lifecycle_hook"),
            non_empty_toml_string(source, "scheduler_entry"),
            toml::string_array_value(source, "verification_steps"),
            toml::usize_value(source, "blocker_count"),
            toml::string_array_value(source, "blockers"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Vec::new(),
                None,
                Vec::new(),
            )
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-launcher-manifest-content-mismatch".to_owned());
        }
        push_bool_mismatch(&mut issues, "ready", expected.ready, actual_ready);
        push_string_mismatch(
            &mut issues,
            "nsb_path",
            expected.nsb_path.as_str(),
            actual_nsb_path.as_deref(),
        );
        if actual_nsb_size_bytes != expected.nsb_size_bytes {
            issues.push(format!(
                "nsb_size_bytes mismatch: expected {}, found {}",
                optional_usize_text(expected.nsb_size_bytes),
                optional_usize_text(actual_nsb_size_bytes)
            ));
        }
        if actual_nsb_hash != expected.nsb_hash {
            issues.push(format!(
                "nsb_hash mismatch: expected {}, found {}",
                expected.nsb_hash.as_deref().unwrap_or("missing"),
                actual_nsb_hash.as_deref().unwrap_or("missing")
            ));
        }
        push_bool_mismatch(
            &mut issues,
            "image_header_valid",
            expected.image_header_valid,
            actual_image_header_valid,
        );
        push_string_mismatch(
            &mut issues,
            "entry_lifecycle_hook",
            expected.entry_lifecycle_hook.as_str(),
            actual_entry_lifecycle_hook.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "scheduler_entry",
            expected.scheduler_entry.as_str(),
            actual_scheduler_entry.as_deref(),
        );
        if actual_verification_steps != expected.verification_steps {
            issues.push(format!(
                "verification_steps mismatch: expected [{}], found [{}]",
                expected.verification_steps.join(", "),
                actual_verification_steps.join(", ")
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

    NsldFinalExecutableLauncherManifestVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_launcher_manifest_hash: expected_hash,
        actual_launcher_manifest_hash: actual_hash,
        expected_ready: expected.ready,
        actual_ready,
        expected_nsb_path: expected.nsb_path,
        actual_nsb_path,
        expected_nsb_size_bytes: expected.nsb_size_bytes,
        actual_nsb_size_bytes,
        expected_nsb_hash: expected.nsb_hash,
        actual_nsb_hash,
        expected_image_header_valid: expected.image_header_valid,
        actual_image_header_valid,
        expected_entry_lifecycle_hook: expected.entry_lifecycle_hook,
        actual_entry_lifecycle_hook,
        expected_scheduler_entry: expected.scheduler_entry,
        actual_scheduler_entry,
        expected_verification_steps: expected.verification_steps,
        actual_verification_steps,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
    }
}

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

pub(crate) fn render_final_executable_launcher_manifest(
    report: &NsldFinalExecutableLauncherManifestReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-host-launcher-manifest-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    push_str_field(&mut out, "manifest", &report.manifest);
    push_str_field(&mut out, "output_path", &report.output_path);
    push_str_field(
        &mut out,
        "launcher_manifest_path",
        &report.launcher_manifest_path,
    );
    out.push_str(&format!("ready = {}\n", report.ready));
    push_str_field(&mut out, "launcher_kind", &report.launcher_kind);
    push_str_field(&mut out, "launcher_format", &report.launcher_format);
    push_str_field(
        &mut out,
        "host_envelope_family",
        &report.host_envelope_family,
    );
    push_str_field(&mut out, "host_os", &report.host_os);
    push_str_field(&mut out, "host_arch", &report.host_arch);
    push_str_field(&mut out, "nsb_path", &report.nsb_path);
    out.push_str(&format!("nsb_present = {}\n", report.nsb_present));
    out.push_str(&format!(
        "nsb_size_bytes = {}\n",
        report
            .nsb_size_bytes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "0".to_owned())
    ));
    push_str_field(
        &mut out,
        "nsb_hash",
        report.nsb_hash.as_deref().unwrap_or(""),
    );
    out.push_str(&format!(
        "image_header_required = {}\n",
        report.image_header_required
    ));
    out.push_str(&format!(
        "image_header_valid = {}\n",
        report.image_header_valid
    ));
    push_str_field(
        &mut out,
        "entry_lifecycle_hook",
        &report.entry_lifecycle_hook,
    );
    push_str_field(&mut out, "scheduler_entry", &report.scheduler_entry);
    out.push_str(&format!(
        "verification_steps = [{}]\n",
        toml::toml_string_array_literal(&report.verification_steps)
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

fn optional_usize_value(source: &str, key: &str) -> Option<usize> {
    toml::usize_value(source, key).filter(|value| *value != 0)
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
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

fn push_string_mismatch(
    issues: &mut Vec<String>,
    field: &str,
    expected: &str,
    actual: Option<&str>,
) {
    if actual != Some(expected) {
        issues.push(format!(
            "{field} mismatch: expected {}, found {}",
            expected,
            actual.unwrap_or("missing")
        ));
    }
}
