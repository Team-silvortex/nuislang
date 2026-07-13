use super::{
    final_executable_output::nsld_final_executable_output_report,
    final_executable_paths::nsld_final_executable_launcher_manifest_path,
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableLauncherManifestEmitReport, NsldFinalExecutableLauncherManifestReport,
        NsldFinalExecutableLauncherManifestVerifyReport,
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
    let image_header_required = output.output_kind == "nuis-image";
    let image_header_valid = !image_header_required || output.output_image_header_valid;
    let mut blockers = output.blockers.clone();
    if !output.runnable_candidate {
        blockers.push(format!(
            "host-launcher:{}-not-runnable-candidate",
            output.output_kind
        ));
    }
    if image_header_required && !output.output_image_header_valid {
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
        output_kind: output.output_kind,
        output_validation_mode: output.output_validation_mode,
        final_output_path: output.output_path.clone(),
        final_output_present: output.present,
        final_output_size_bytes: output.size_bytes,
        final_output_hash: output.output_hash.clone(),
        nsb_path: output.output_path,
        nsb_present: output.present,
        nsb_size_bytes: output.size_bytes,
        nsb_hash: output.output_hash,
        image_header_required,
        image_header_valid,
        entry_lifecycle_hook: "on_process_start".to_owned(),
        scheduler_entry: "nuis.scheduler.loop.v1".to_owned(),
        scheduler_metadata_payload_id: output.scheduler_metadata_payload_id,
        scheduler_metadata_present: output.scheduler_metadata_present,
        scheduler_metadata_offset: output.scheduler_metadata_offset,
        scheduler_metadata_hash: output.scheduler_metadata_hash,
        verification_steps: launcher_verification_steps(image_header_required),
        blockers,
        notes: vec![
            "launcher-manifest-is-non-executing".to_owned(),
            "host-launcher-implementation-remains-separate-from-nsld-core-linking".to_owned(),
        ],
    }
}

fn launcher_verification_steps(image_header_required: bool) -> Vec<String> {
    if image_header_required {
        vec![
            "read-nsb-header".to_owned(),
            "verify-nsb-magic-and-version".to_owned(),
            "verify-nsb-size-and-hash".to_owned(),
            "map-payload-region".to_owned(),
            "enter-lifecycle-hook:on_process_start".to_owned(),
        ]
    } else {
        vec![
            "verify-host-native-output-presence".to_owned(),
            "verify-host-native-output-hash".to_owned(),
            "verify-host-native-invoke-plan".to_owned(),
            "enter-host-native-process-boundary".to_owned(),
        ]
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
        actual_output_kind,
        actual_output_validation_mode,
        actual_final_output_path,
        actual_final_output_size_bytes,
        actual_final_output_hash,
        actual_image_header_required,
        actual_image_header_valid,
        actual_entry_lifecycle_hook,
        actual_scheduler_entry,
        actual_scheduler_metadata_payload_id,
        actual_scheduler_metadata_present,
        actual_scheduler_metadata_offset,
        actual_scheduler_metadata_hash,
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
            non_empty_toml_string(source, "output_kind"),
            non_empty_toml_string(source, "output_validation_mode"),
            non_empty_toml_string(source, "final_output_path")
                .or_else(|| non_empty_toml_string(source, "nsb_path")),
            optional_usize_value(source, "final_output_size_bytes")
                .or_else(|| optional_usize_value(source, "nsb_size_bytes")),
            non_empty_toml_string(source, "final_output_hash")
                .or_else(|| non_empty_toml_string(source, "nsb_hash")),
            toml::bool_value(source, "image_header_required"),
            toml::bool_value(source, "image_header_valid"),
            non_empty_toml_string(source, "entry_lifecycle_hook"),
            non_empty_toml_string(source, "scheduler_entry"),
            non_empty_toml_string(source, "scheduler_metadata_payload_id"),
            toml::bool_value(source, "scheduler_metadata_present"),
            optional_usize_value(source, "scheduler_metadata_offset"),
            non_empty_toml_string(source, "scheduler_metadata_hash"),
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
                None,
                None,
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
        push_string_mismatch(
            &mut issues,
            "output_kind",
            expected.output_kind.as_str(),
            actual_output_kind.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "output_validation_mode",
            expected.output_validation_mode.as_str(),
            actual_output_validation_mode.as_deref(),
        );
        push_string_mismatch(
            &mut issues,
            "final_output_path",
            expected.final_output_path.as_str(),
            actual_final_output_path.as_deref(),
        );
        if actual_final_output_size_bytes != expected.final_output_size_bytes {
            issues.push(format!(
                "final_output_size_bytes mismatch: expected {}, found {}",
                optional_usize_text(expected.final_output_size_bytes),
                optional_usize_text(actual_final_output_size_bytes)
            ));
        }
        if actual_final_output_hash != expected.final_output_hash {
            issues.push(format!(
                "final_output_hash mismatch: expected {}, found {}",
                expected.final_output_hash.as_deref().unwrap_or("missing"),
                actual_final_output_hash.as_deref().unwrap_or("missing")
            ));
        }
        push_bool_mismatch(
            &mut issues,
            "image_header_required",
            expected.image_header_required,
            actual_image_header_required,
        );
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
        push_optional_string_mismatch(
            &mut issues,
            "scheduler_metadata_payload_id",
            expected.scheduler_metadata_payload_id.as_deref(),
            actual_scheduler_metadata_payload_id.as_deref(),
        );
        if actual_scheduler_metadata_present != expected.scheduler_metadata_present {
            issues.push(format!(
                "scheduler_metadata_present mismatch: expected {}, found {}",
                optional_bool_text(expected.scheduler_metadata_present),
                optional_bool_text(actual_scheduler_metadata_present)
            ));
        }
        if actual_scheduler_metadata_offset != expected.scheduler_metadata_offset {
            issues.push(format!(
                "scheduler_metadata_offset mismatch: expected {}, found {}",
                optional_usize_text(expected.scheduler_metadata_offset),
                optional_usize_text(actual_scheduler_metadata_offset)
            ));
        }
        if actual_scheduler_metadata_hash != expected.scheduler_metadata_hash {
            issues.push(format!(
                "scheduler_metadata_hash mismatch: expected {}, found {}",
                expected
                    .scheduler_metadata_hash
                    .as_deref()
                    .unwrap_or("missing"),
                actual_scheduler_metadata_hash
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
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
        expected_output_kind: expected.output_kind,
        actual_output_kind,
        expected_output_validation_mode: expected.output_validation_mode,
        actual_output_validation_mode,
        expected_final_output_path: expected.final_output_path,
        actual_final_output_path,
        expected_final_output_size_bytes: expected.final_output_size_bytes,
        actual_final_output_size_bytes,
        expected_final_output_hash: expected.final_output_hash,
        actual_final_output_hash,
        expected_image_header_required: expected.image_header_required,
        actual_image_header_required,
        expected_image_header_valid: expected.image_header_valid,
        actual_image_header_valid,
        expected_entry_lifecycle_hook: expected.entry_lifecycle_hook,
        actual_entry_lifecycle_hook,
        expected_scheduler_entry: expected.scheduler_entry,
        actual_scheduler_entry,
        expected_scheduler_metadata_payload_id: expected.scheduler_metadata_payload_id,
        actual_scheduler_metadata_payload_id,
        expected_scheduler_metadata_present: expected.scheduler_metadata_present,
        actual_scheduler_metadata_present,
        expected_scheduler_metadata_offset: expected.scheduler_metadata_offset,
        actual_scheduler_metadata_offset,
        expected_scheduler_metadata_hash: expected.scheduler_metadata_hash,
        actual_scheduler_metadata_hash,
        expected_verification_steps: expected.verification_steps,
        actual_verification_steps,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        expected_blockers: expected.blockers,
        actual_blockers,
        issues,
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
    push_str_field(&mut out, "output_kind", &report.output_kind);
    push_str_field(
        &mut out,
        "output_validation_mode",
        &report.output_validation_mode,
    );
    push_str_field(&mut out, "final_output_path", &report.final_output_path);
    out.push_str(&format!(
        "final_output_present = {}\n",
        report.final_output_present
    ));
    out.push_str(&format!(
        "final_output_size_bytes = {}\n",
        report
            .final_output_size_bytes
            .map(|value| value.to_string())
            .unwrap_or_else(|| "0".to_owned())
    ));
    push_str_field(
        &mut out,
        "final_output_hash",
        report.final_output_hash.as_deref().unwrap_or(""),
    );
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
    push_str_field(
        &mut out,
        "scheduler_metadata_payload_id",
        report
            .scheduler_metadata_payload_id
            .as_deref()
            .unwrap_or(""),
    );
    out.push_str(&format!(
        "scheduler_metadata_present = {}\n",
        report.scheduler_metadata_present.unwrap_or(false)
    ));
    out.push_str(&format!(
        "scheduler_metadata_offset = {}\n",
        report
            .scheduler_metadata_offset
            .map(|value| value.to_string())
            .unwrap_or_else(|| "0".to_owned())
    ));
    push_str_field(
        &mut out,
        "scheduler_metadata_hash",
        report.scheduler_metadata_hash.as_deref().unwrap_or(""),
    );
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

fn optional_bool_text(value: Option<bool>) -> String {
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
