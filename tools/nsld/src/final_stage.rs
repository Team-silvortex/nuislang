use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    closure::{nsld_closure_report, nsld_verify_closure_report},
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableEmitReport, NsldFinalExecutableEmitVerifyReport,
        NsldFinalStageInputDiagnostic, NsldFinalStagePlanEmitReport, NsldFinalStagePlanReport,
        NsldFinalStagePlanVerifyReport,
    },
    toml,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn nsld_final_stage_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalStagePlanReport {
    let closure = nsld_closure_report(manifest, plan);
    let closure_snapshot_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ClosureSnapshot);
    let native_object_path =
        nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectOutput);
    let host_wrapper_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let native_object_required = matches!(
        plan.final_stage.link_mode.as_str(),
        "host-toolchain-finalize" | "bundle-packaging"
    );
    let inputs = vec![
        final_stage_input(
            0,
            "fsi0000.container",
            "nsld-container",
            PathBuf::from(&plan.output_dir).join("nuis.nsld.container"),
            true,
        ),
        final_stage_input(
            1,
            "fsi0001.container-payload",
            "nsld-container-payload",
            PathBuf::from(&plan.output_dir).join("nuis.nsld.container.payload"),
            true,
        ),
        final_stage_input(
            2,
            "fsi0002.closure-snapshot",
            "nsld-closure-snapshot",
            closure_snapshot_path,
            true,
        ),
        final_stage_input(
            3,
            "fsi0003.native-object",
            "native-object-output",
            native_object_path,
            native_object_required,
        ),
    ];
    let mut blockers = Vec::new();
    for input in &inputs {
        if input.required && !input.present {
            blockers.push(format!("missing-final-stage-input:{}", input.input_id));
        }
    }
    let closure_snapshot_verify = nsld_verify_closure_report(manifest, plan);
    if !closure_snapshot_verify.valid {
        blockers.push("closure-snapshot-verification".to_owned());
        blockers.extend(
            closure_snapshot_verify
                .issues
                .iter()
                .map(|issue| format!("closure:{issue}")),
        );
    }
    if closure.container_loader_readiness == "blocked" {
        blockers.push("container-loader-blocked".to_owned());
    }
    if host_wrapper_required {
        blockers.push("self-owned-final-native-linker".to_owned());
    }

    let notes = final_stage_notes(plan, host_wrapper_required);
    let ready = blockers.is_empty();
    let plan_hash = nsld_final_stage_plan_hash(
        plan,
        &inputs,
        &closure.container_hash,
        &closure.payload_hash,
        &closure.linker_contract_hash,
        &blockers,
        &notes,
    );

    NsldFinalStagePlanReport {
        manifest: manifest.display().to_string(),
        ready,
        plan_hash,
        final_stage_kind: plan.final_stage.kind.clone(),
        final_stage_driver: plan.final_stage.driver.clone(),
        final_stage_link_mode: plan.final_stage.link_mode.clone(),
        final_output_path: plan.final_stage.output_path.clone(),
        host_wrapper_required,
        compatibility_mode: if host_wrapper_required {
            "host-assisted-wrapper".to_owned()
        } else {
            "self-contained".to_owned()
        },
        input_count: inputs.len(),
        inputs,
        container_hash: closure.container_hash,
        payload_hash: closure.payload_hash,
        linker_contract_hash: closure.linker_contract_hash,
        native_object_required,
        native_object_present: nsld_artifact_stage_kind_path(
            &plan.output_dir,
            NsldArtifactStageKind::ObjectOutput,
        )
        .exists(),
        blockers,
        notes,
    }
}

pub(crate) fn nsld_emit_final_stage_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalStagePlanEmitReport, String> {
    let report = nsld_final_stage_plan_report(manifest, plan);
    let output_path = nsld_final_stage_plan_path(plan);
    fs::write(&output_path, render_final_stage_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld final-stage plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldFinalStagePlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        ready: report.ready,
        plan_hash: report.plan_hash,
        input_count: report.input_count,
        blocker_count: report.blockers.len(),
    })
}

pub(crate) fn nsld_verify_final_stage_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalStagePlanVerifyReport {
    let expected_report = nsld_final_stage_plan_report(manifest, plan);
    let input_path = nsld_final_stage_plan_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_stage_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_plan_hash, actual_input_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "plan_hash"),
            toml::usize_value(source, "input_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None)
        }
    };
    if let Ok(actual) = actual {
        let expected = render_final_stage_plan(&expected_report);
        if actual != expected {
            issues.push("final-stage-plan-content-mismatch".to_owned());
        }
        if actual_plan_hash.as_deref() != Some(expected_report.plan_hash.as_str()) {
            issues.push(format!(
                "plan_hash mismatch: expected {}, found {}",
                expected_report.plan_hash,
                actual_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_input_count != Some(expected_report.input_count) {
            issues.push(format!(
                "input_count mismatch: expected {}, found {}",
                expected_report.input_count,
                actual_input_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldFinalStagePlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_plan_hash: expected_report.plan_hash,
        expected_input_count: expected_report.input_count,
        actual_plan_hash,
        actual_input_count,
        issues,
    }
}

pub(crate) fn nsld_emit_final_executable_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableEmitReport, String> {
    let report = nsld_final_executable_readiness_report(manifest, plan);
    let blocked_report_path = nsld_final_executable_blocked_path(plan);
    fs::write(
        &blocked_report_path,
        render_final_executable_blocked(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld final executable blocked report `{}`: {error}",
            blocked_report_path.display()
        )
    })?;
    Ok(report)
}

pub(crate) fn nsld_verify_final_executable_emit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitVerifyReport {
    let expected = nsld_final_executable_readiness_report(manifest, plan);
    let input_path = nsld_final_executable_blocked_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_blocked `{}`: {error}",
            input_path.display()
        )
    });
    let (actual_plan_hash, actual_emitted, actual_blocker_count) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "final_stage_plan_hash"),
            toml::bool_value(source, "emitted"),
            toml::usize_value(source, "blocker_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None)
        }
    };
    if let Ok(actual) = actual {
        let expected_source = render_final_executable_blocked(&expected);
        if actual != expected_source {
            issues.push("final-executable-blocked-content-mismatch".to_owned());
        }
        if actual_plan_hash.as_deref() != Some(expected.final_stage_plan_hash.as_str()) {
            issues.push(format!(
                "final_stage_plan_hash mismatch: expected {}, found {}",
                expected.final_stage_plan_hash,
                actual_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_emitted != Some(expected.emitted) {
            issues.push(format!(
                "emitted mismatch: expected {}, found {}",
                expected.emitted,
                actual_emitted
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
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
    }

    NsldFinalExecutableEmitVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_final_stage_plan_hash: expected.final_stage_plan_hash,
        actual_final_stage_plan_hash: actual_plan_hash,
        expected_emitted: expected.emitted,
        actual_emitted,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        issues,
    }
}

pub(crate) fn render_final_stage_plan(report: &NsldFinalStagePlanReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-stage-plan-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("plan_kind = \"deterministic-final-stage-plan\"\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.6.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "plan_hash = \"{}\"\n",
        toml::escape_toml_string(&report.plan_hash)
    ));
    out.push_str(&format!(
        "final_stage_kind = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_kind)
    ));
    out.push_str(&format!(
        "final_stage_driver = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_driver)
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "final_output_path = \"{}\"\n",
        toml::escape_toml_string(&report.final_output_path)
    ));
    out.push_str(&format!(
        "host_wrapper_required = {}\n",
        report.host_wrapper_required
    ));
    out.push_str(&format!(
        "compatibility_mode = \"{}\"\n",
        toml::escape_toml_string(&report.compatibility_mode)
    ));
    out.push_str(&format!("input_count = {}\n", report.input_count));
    out.push_str(&format!(
        "container_hash = \"{}\"\n",
        toml::escape_toml_string(&report.container_hash)
    ));
    out.push_str(&format!(
        "payload_hash = \"{}\"\n",
        toml::escape_toml_string(&report.payload_hash)
    ));
    out.push_str(&format!(
        "linker_contract_hash = \"{}\"\n",
        toml::escape_toml_string(&report.linker_contract_hash)
    ));
    out.push_str(&format!(
        "native_object_required = {}\n",
        report.native_object_required
    ));
    out.push_str(&format!(
        "native_object_present = {}\n",
        report.native_object_present
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml::toml_string_array_literal(&report.blockers)
    ));
    out.push_str(&format!(
        "notes = [{}]\n",
        toml::toml_string_array_literal(&report.notes)
    ));
    for input in &report.inputs {
        out.push_str("\n[[final_stage_input]]\n");
        out.push_str(&format!("order_index = {}\n", input.order_index));
        out.push_str(&format!(
            "input_id = \"{}\"\n",
            toml::escape_toml_string(&input.input_id)
        ));
        out.push_str(&format!(
            "input_kind = \"{}\"\n",
            toml::escape_toml_string(&input.input_kind)
        ));
        out.push_str(&format!(
            "path = \"{}\"\n",
            toml::escape_toml_string(&input.path)
        ));
        out.push_str(&format!(
            "content_hash = \"{}\"\n",
            toml::escape_toml_string(&input.content_hash)
        ));
        out.push_str(&format!("required = {}\n", input.required));
        out.push_str(&format!("present = {}\n", input.present));
    }
    out
}

pub(crate) fn render_final_executable_blocked(report: &NsldFinalExecutableEmitReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-blocked-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.6.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        toml::escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "blocked_report_path = \"{}\"\n",
        toml::escape_toml_string(&report.blocked_report_path)
    ));
    out.push_str(&format!("emitted = {}\n", report.emitted));
    out.push_str(&format!(
        "can_emit_final_executable = {}\n",
        report.can_emit_final_executable
    ));
    out.push_str(&format!(
        "final_stage_ready = {}\n",
        report.final_stage_ready
    ));
    out.push_str(&format!(
        "final_stage_plan_hash = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_plan_hash)
    ));
    out.push_str(&format!(
        "final_stage_driver = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_driver)
    ));
    out.push_str(&format!(
        "final_stage_link_mode = \"{}\"\n",
        toml::escape_toml_string(&report.final_stage_link_mode)
    ));
    out.push_str(&format!(
        "host_wrapper_required = {}\n",
        report.host_wrapper_required
    ));
    out.push_str(&format!("input_count = {}\n", report.input_count));
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

pub(crate) fn nsld_final_stage_plan_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::FinalStagePlan)
}

pub(crate) fn nsld_final_executable_blocked_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableBlocked,
    )
}

pub(crate) fn nsld_final_executable_readiness_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitReport {
    let final_stage = nsld_final_stage_plan_report(manifest, plan);
    let mut blockers = final_stage.blockers.clone();
    if final_stage.ready {
        blockers.push("final-executable-emitter:not-implemented".to_owned());
    }
    let emitted = false;
    let can_emit_final_executable = blockers.is_empty();
    let mut notes = final_stage.notes.clone();
    notes.push("final-executable-emit-is-contract-only".to_owned());
    if final_stage.host_wrapper_required {
        notes.push("host-wrapper-remains-cffi-compatibility-domain".to_owned());
    }

    NsldFinalExecutableEmitReport {
        manifest: final_stage.manifest,
        output_path: final_stage.final_output_path,
        blocked_report_path: nsld_final_executable_blocked_path(plan)
            .display()
            .to_string(),
        emitted,
        can_emit_final_executable,
        final_stage_ready: final_stage.ready,
        final_stage_plan_hash: final_stage.plan_hash,
        final_stage_driver: final_stage.final_stage_driver,
        final_stage_link_mode: final_stage.final_stage_link_mode,
        host_wrapper_required: final_stage.host_wrapper_required,
        input_count: final_stage.input_count,
        blockers,
        notes,
    }
}

fn final_stage_input(
    order_index: usize,
    input_id: &str,
    input_kind: &str,
    path: PathBuf,
    required: bool,
) -> NsldFinalStageInputDiagnostic {
    let present = path.exists();
    let content_hash = fs::read(&path)
        .map(|bytes| fnv1a64_hex(&bytes))
        .unwrap_or_else(|_| "missing".to_owned());
    NsldFinalStageInputDiagnostic {
        order_index,
        input_id: input_id.to_owned(),
        input_kind: input_kind.to_owned(),
        path: path.display().to_string(),
        content_hash,
        required,
        present,
    }
}

fn final_stage_notes(plan: &nuisc::linker::LinkPlan, host_wrapper_required: bool) -> Vec<String> {
    let mut notes = Vec::new();
    if host_wrapper_required {
        notes.push(format!(
            "host-final-stage-driver:{}",
            plan.final_stage.driver
        ));
    }
    if !plan.cpu_target.object_format.is_empty() {
        notes.push(format!("object-format:{}", plan.cpu_target.object_format));
    }
    if !plan.cpu_target.clang_target.is_empty() {
        notes.push(format!("clang-target:{}", plan.cpu_target.clang_target));
    }
    notes
}

fn nsld_final_stage_plan_hash(
    plan: &nuisc::linker::LinkPlan,
    inputs: &[NsldFinalStageInputDiagnostic],
    container_hash: &str,
    payload_hash: &str,
    linker_contract_hash: &str,
    blockers: &[String],
    notes: &[String],
) -> String {
    let mut material = String::new();
    material.push_str(&plan.final_stage.kind);
    material.push('\t');
    material.push_str(&plan.final_stage.driver);
    material.push('\t');
    material.push_str(&plan.final_stage.link_mode);
    material.push('\t');
    material.push_str(&plan.final_stage.output_path);
    material.push('\n');
    material.push_str(container_hash);
    material.push('\t');
    material.push_str(payload_hash);
    material.push('\t');
    material.push_str(linker_contract_hash);
    material.push('\n');
    for input in inputs {
        material.push_str(&input.input_id);
        material.push('\t');
        material.push_str(&input.input_kind);
        material.push('\t');
        material.push_str(&input.path);
        material.push('\t');
        material.push_str(&input.content_hash);
        material.push('\t');
        material.push_str(if input.required {
            "required"
        } else {
            "optional"
        });
        material.push('\t');
        material.push_str(if input.present { "present" } else { "missing" });
        material.push('\n');
    }
    for blocker in blockers {
        material.push_str("blocker\t");
        material.push_str(blocker);
        material.push('\n');
    }
    for note in notes {
        material.push_str("note\t");
        material.push_str(note);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}
