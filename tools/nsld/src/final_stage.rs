use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    closure::{nsld_closure_report, nsld_verify_closure_report},
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableEmitReport, NsldFinalExecutableEmitVerifyReport,
        NsldFinalExecutableHostDryRunReport, NsldFinalExecutableWriterInputEmitReport,
        NsldFinalExecutableWriterInputVerifyReport, NsldFinalExecutableWriterPlanReport,
        NsldFinalStageInputDiagnostic, NsldFinalStagePlanEmitReport, NsldFinalStagePlanReport,
        NsldFinalStagePlanVerifyReport,
    },
    toml,
};
use std::{
    env, fs,
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
    let report = nsld_final_executable_emit_report_shape(manifest, plan);
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

pub(crate) fn nsld_emit_final_executable_writer_input_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableWriterInputEmitReport, String> {
    let writer_plan = nsld_final_executable_writer_plan_report(manifest, plan);
    let source = render_final_executable_writer_input(&writer_plan, plan);
    let output_path = nsld_final_executable_writer_input_path(plan);
    let command_arg_count = final_executable_writer_command_args(&writer_plan, plan).len();
    fs::write(&output_path, &source).map_err(|error| {
        format!(
            "failed to write nsld final executable writer input `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldFinalExecutableWriterInputEmitReport {
        manifest: writer_plan.manifest,
        output_path: output_path.display().to_string(),
        writer_input_hash: fnv1a64_hex(source.as_bytes()),
        writer_kind: writer_plan.writer_kind,
        writer_status: writer_plan.writer_status,
        final_stage_plan_hash: writer_plan.final_stage_plan_hash,
        final_stage_driver: writer_plan.final_stage_driver,
        final_stage_link_mode: writer_plan.final_stage_link_mode,
        host_wrapper_required: writer_plan.host_wrapper_required,
        command_arg_count,
        writer_blockers: writer_plan.writer_blockers,
    })
}

pub(crate) fn nsld_verify_final_executable_writer_input_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableWriterInputVerifyReport {
    let expected_plan = nsld_final_executable_writer_plan_report(manifest, plan);
    let expected_source = render_final_executable_writer_input(&expected_plan, plan);
    let expected_hash = fnv1a64_hex(expected_source.as_bytes());
    let expected_command_args = final_executable_writer_command_args(&expected_plan, plan);
    let expected_command_arg_count = expected_command_args.len();
    let input_path = nsld_final_executable_writer_input_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_writer_input `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_hash,
        actual_final_stage_plan_hash,
        actual_writer_kind,
        actual_writer_status,
        actual_command_arg_count,
        actual_command_args,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::string_value(source, "final_stage_plan_hash"),
            toml::string_value(source, "writer_kind"),
            toml::string_value(source, "writer_status"),
            toml::usize_value(source, "command_arg_count"),
            toml::string_array_value(source, "command_args"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, Vec::new())
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-writer-input-content-mismatch".to_owned());
        }
        if actual_final_stage_plan_hash.as_deref()
            != Some(expected_plan.final_stage_plan_hash.as_str())
        {
            issues.push(format!(
                "final_stage_plan_hash mismatch: expected {}, found {}",
                expected_plan.final_stage_plan_hash,
                actual_final_stage_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_writer_kind.as_deref() != Some(expected_plan.writer_kind.as_str()) {
            issues.push(format!(
                "writer_kind mismatch: expected {}, found {}",
                expected_plan.writer_kind,
                actual_writer_kind
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_writer_status.as_deref() != Some(expected_plan.writer_status.as_str()) {
            issues.push(format!(
                "writer_status mismatch: expected {}, found {}",
                expected_plan.writer_status,
                actual_writer_status
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        let expected_arg_count = final_executable_writer_command_args(&expected_plan, plan).len();
        if actual_command_arg_count != Some(expected_arg_count) {
            issues.push(format!(
                "command_arg_count mismatch: expected {}, found {}",
                expected_arg_count,
                actual_command_arg_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_command_args != expected_command_args {
            issues.push(format!(
                "command_args mismatch: expected [{}], found [{}]",
                expected_command_args.join(", "),
                actual_command_args.join(", ")
            ));
        }
    }

    NsldFinalExecutableWriterInputVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_writer_input_hash: expected_hash,
        actual_writer_input_hash: actual_hash,
        expected_final_stage_plan_hash: expected_plan.final_stage_plan_hash,
        actual_final_stage_plan_hash,
        expected_writer_kind: expected_plan.writer_kind,
        actual_writer_kind,
        expected_writer_status: expected_plan.writer_status,
        actual_writer_status,
        expected_command_arg_count,
        actual_command_arg_count,
        expected_command_args,
        actual_command_args,
        issues,
    }
}

pub(crate) fn nsld_final_executable_host_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableHostDryRunReport {
    let writer_plan = nsld_final_executable_writer_plan_report(manifest, plan);
    let writer_input = nsld_verify_final_executable_writer_input_report(manifest, plan);
    let command_args = if writer_input.actual_command_args.is_empty() {
        writer_input.expected_command_args.clone()
    } else {
        writer_input.actual_command_args.clone()
    };
    let driver = command_args
        .first()
        .cloned()
        .unwrap_or_else(|| writer_plan.final_stage_driver.clone());
    let driver_resolved_path = resolve_host_driver_path(&driver);
    let driver_available = driver_resolved_path.is_some();
    let mut blockers = Vec::new();
    if !writer_input.valid {
        blockers.push("final-executable-writer-input:invalid".to_owned());
        blockers.extend(
            writer_input
                .issues
                .iter()
                .map(|issue| format!("final-executable-writer-input:{issue}")),
        );
    }
    if !driver_available {
        blockers.push(format!("host-finalizer-driver-unavailable:{driver}"));
    }
    blockers.extend(writer_plan.writer_blockers.iter().cloned());
    let environment_ready = writer_input.valid && driver_available;
    let can_invoke_host_finalizer = environment_ready && writer_plan.writer_blockers.is_empty();
    let mut notes = writer_plan.notes.clone();
    notes.push("host-finalizer-dry-run-is-non-mutating".to_owned());
    notes.push("host-finalizer-is-not-invoked".to_owned());

    NsldFinalExecutableHostDryRunReport {
        manifest: manifest.display().to_string(),
        writer_input_path: writer_input.input_path,
        writer_input_valid: writer_input.valid,
        writer_input_hash: writer_input.actual_writer_input_hash,
        driver,
        driver_available,
        driver_resolved_path,
        command_arg_count: command_args.len(),
        command_args,
        environment_ready,
        can_invoke_host_finalizer,
        blockers,
        notes,
    }
}

pub(crate) fn nsld_verify_final_executable_emit_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitVerifyReport {
    let expected = nsld_final_executable_emit_report_shape(manifest, plan);
    let input_path = nsld_final_executable_blocked_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_blocked `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_plan_hash,
        actual_emitted,
        actual_host_environment_ready,
        actual_host_driver_available,
        actual_host_can_invoke,
        actual_host_driver_resolved_path,
        actual_host_dry_run_blocker_count,
        actual_host_dry_run_blockers,
        actual_blocker_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "final_stage_plan_hash"),
            toml::bool_value(source, "emitted"),
            toml::bool_value(source, "host_dry_run_environment_ready"),
            toml::bool_value(source, "host_dry_run_driver_available"),
            toml::bool_value(source, "host_dry_run_can_invoke"),
            non_empty_toml_string(source, "host_dry_run_driver_resolved_path"),
            toml::usize_value(source, "host_dry_run_blocker_count"),
            toml::string_array_value(source, "host_dry_run_blockers"),
            toml::usize_value(source, "blocker_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, None, Vec::new(), None)
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
        if actual_host_environment_ready != expected.host_dry_run_environment_ready {
            issues.push(format!(
                "host_dry_run_environment_ready mismatch: expected {}, found {}",
                expected
                    .host_dry_run_environment_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_environment_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_driver_available != expected.host_dry_run_driver_available {
            issues.push(format!(
                "host_dry_run_driver_available mismatch: expected {}, found {}",
                expected
                    .host_dry_run_driver_available
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_driver_available
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_can_invoke != expected.host_dry_run_can_invoke {
            issues.push(format!(
                "host_dry_run_can_invoke mismatch: expected {}, found {}",
                expected
                    .host_dry_run_can_invoke
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_can_invoke
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_driver_resolved_path != expected.host_dry_run_driver_resolved_path {
            issues.push(format!(
                "host_dry_run_driver_resolved_path mismatch: expected {}, found {}",
                expected
                    .host_dry_run_driver_resolved_path
                    .as_deref()
                    .unwrap_or("missing"),
                actual_host_driver_resolved_path
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_host_dry_run_blockers != expected.host_dry_run_blockers {
            issues.push(format!(
                "host_dry_run_blockers mismatch: expected [{}], found [{}]",
                expected.host_dry_run_blockers.join(", "),
                actual_host_dry_run_blockers.join(", ")
            ));
        }
        if actual_host_dry_run_blocker_count != Some(expected.host_dry_run_blockers.len()) {
            issues.push(format!(
                "host_dry_run_blocker_count mismatch: expected {}, found {}",
                expected.host_dry_run_blockers.len(),
                actual_host_dry_run_blocker_count
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
        expected_host_dry_run_environment_ready: expected.host_dry_run_environment_ready,
        actual_host_dry_run_environment_ready: actual_host_environment_ready,
        expected_host_dry_run_driver_available: expected.host_dry_run_driver_available,
        actual_host_dry_run_driver_available: actual_host_driver_available,
        expected_host_dry_run_can_invoke: expected.host_dry_run_can_invoke,
        actual_host_dry_run_can_invoke: actual_host_can_invoke,
        expected_host_dry_run_driver_resolved_path: expected.host_dry_run_driver_resolved_path,
        actual_host_dry_run_driver_resolved_path: actual_host_driver_resolved_path,
        expected_host_dry_run_blocker_count: expected.host_dry_run_blockers.len(),
        actual_host_dry_run_blocker_count,
        expected_host_dry_run_blockers: expected.host_dry_run_blockers,
        actual_host_dry_run_blockers,
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
    out.push_str("producer_phase = \"alpha-0.8.0\"\n");
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
    out.push_str("producer_phase = \"alpha-0.8.0\"\n");
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
    out.push_str(&format!(
        "writer_kind = \"{}\"\n",
        toml::escape_toml_string(&report.writer_kind)
    ));
    out.push_str(&format!(
        "writer_status = \"{}\"\n",
        toml::escape_toml_string(&report.writer_status)
    ));
    out.push_str(&format!(
        "writer_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.writer_blockers)
    ));
    out.push_str(&format!(
        "writer_input_path = \"{}\"\n",
        toml::escape_toml_string(&report.writer_input_path)
    ));
    out.push_str(&format!(
        "writer_input_valid = {}\n",
        optional_bool_toml(report.writer_input_valid)
    ));
    out.push_str(&format!(
        "writer_input_hash = \"{}\"\n",
        toml::escape_toml_string(report.writer_input_hash.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "writer_input_issues = [{}]\n",
        toml::toml_string_array_literal(&report.writer_input_issues)
    ));
    out.push_str(&format!(
        "host_dry_run_environment_ready = {}\n",
        optional_bool_toml(report.host_dry_run_environment_ready)
    ));
    out.push_str(&format!(
        "host_dry_run_driver_available = {}\n",
        optional_bool_toml(report.host_dry_run_driver_available)
    ));
    out.push_str(&format!(
        "host_dry_run_driver_resolved_path = \"{}\"\n",
        toml::escape_toml_string(
            report
                .host_dry_run_driver_resolved_path
                .as_deref()
                .unwrap_or("")
        )
    ));
    out.push_str(&format!(
        "host_dry_run_can_invoke = {}\n",
        optional_bool_toml(report.host_dry_run_can_invoke)
    ));
    out.push_str(&format!(
        "host_dry_run_blocker_count = {}\n",
        report.host_dry_run_blockers.len()
    ));
    out.push_str(&format!(
        "host_dry_run_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.host_dry_run_blockers)
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

pub(crate) fn render_final_executable_writer_input(
    report: &NsldFinalExecutableWriterPlanReport,
    plan: &nuisc::linker::LinkPlan,
) -> String {
    let command_args = final_executable_writer_command_args(report, plan);
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-writer-input-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.8.0\"\n");
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        toml::escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        toml::escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "writer_kind = \"{}\"\n",
        toml::escape_toml_string(&report.writer_kind)
    ));
    out.push_str(&format!(
        "writer_status = \"{}\"\n",
        toml::escape_toml_string(&report.writer_status)
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
    out.push_str(&format!("command_arg_count = {}\n", command_args.len()));
    out.push_str(&format!(
        "command_args = [{}]\n",
        toml::toml_string_array_literal(&command_args)
    ));
    out.push_str(&format!(
        "writer_steps = [{}]\n",
        toml::toml_string_array_literal(&report.writer_steps)
    ));
    out.push_str(&format!(
        "writer_blockers = [{}]\n",
        toml::toml_string_array_literal(&report.writer_blockers)
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

pub(crate) fn nsld_final_stage_plan_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::FinalStagePlan)
}

pub(crate) fn nsld_final_executable_writer_input_path(plan: &nuisc::linker::LinkPlan) -> PathBuf {
    nsld_artifact_stage_kind_path(
        &plan.output_dir,
        NsldArtifactStageKind::FinalExecutableWriterInput,
    )
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
    let writer_kind = if final_stage.host_wrapper_required {
        "host-assisted-final-executable"
    } else {
        "self-contained-final-executable"
    }
    .to_owned();
    let writer_status = "blocked".to_owned();
    let writer_blockers = final_executable_writer_blockers(&final_stage);
    blockers.extend(writer_blockers.iter().cloned());
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
        writer_kind,
        writer_status,
        writer_blockers,
        writer_input_path: nsld_final_executable_writer_input_path(plan)
            .display()
            .to_string(),
        writer_input_valid: None,
        writer_input_hash: None,
        writer_input_issues: Vec::new(),
        host_dry_run_environment_ready: None,
        host_dry_run_driver_available: None,
        host_dry_run_driver_resolved_path: None,
        host_dry_run_can_invoke: None,
        host_dry_run_blocker_count: 0,
        host_dry_run_blockers: Vec::new(),
        input_count: final_stage.input_count,
        blockers,
        notes,
    }
}

fn nsld_final_executable_emit_report_shape(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableEmitReport {
    let mut report = nsld_final_executable_readiness_report(manifest, plan);
    let writer_input = nsld_verify_final_executable_writer_input_report(manifest, plan);
    let host_dry_run = nsld_final_executable_host_dry_run_report(manifest, plan);
    report.writer_input_path = writer_input.input_path;
    report.writer_input_valid = Some(writer_input.valid);
    report.writer_input_hash = writer_input.actual_writer_input_hash;
    report.writer_input_issues = writer_input.issues;
    report.host_dry_run_environment_ready = Some(host_dry_run.environment_ready);
    report.host_dry_run_driver_available = Some(host_dry_run.driver_available);
    report.host_dry_run_driver_resolved_path = host_dry_run.driver_resolved_path;
    report.host_dry_run_can_invoke = Some(host_dry_run.can_invoke_host_finalizer);
    report.host_dry_run_blocker_count = host_dry_run.blockers.len();
    report.host_dry_run_blockers = host_dry_run.blockers;
    if !writer_input.valid {
        report
            .blockers
            .push("final-executable-writer-input:invalid".to_owned());
        report.blockers.extend(
            report
                .writer_input_issues
                .iter()
                .map(|issue| format!("final-executable-writer-input:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    if !host_dry_run.environment_ready {
        report
            .blockers
            .push("host-finalizer-environment:not-ready".to_owned());
        report.blockers.extend(
            report
                .host_dry_run_blockers
                .iter()
                .map(|blocker| format!("host-finalizer-dry-run:{blocker}")),
        );
        report.can_emit_final_executable = false;
    }
    report
}

pub(crate) fn nsld_final_executable_writer_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableWriterPlanReport {
    let final_stage = nsld_final_stage_plan_report(manifest, plan);
    let writer_kind = if final_stage.host_wrapper_required {
        "host-assisted-final-executable"
    } else {
        "self-contained-final-executable"
    }
    .to_owned();
    let writer_status = "blocked".to_owned();
    let writer_blockers = final_executable_writer_blockers(&final_stage);
    let writer_steps = final_executable_writer_steps(&final_stage);
    let mut notes = final_stage.notes.clone();
    notes.push("final-executable-writer-plan-is-non-mutating".to_owned());

    NsldFinalExecutableWriterPlanReport {
        manifest: final_stage.manifest,
        output_path: final_stage.final_output_path,
        writer_kind,
        writer_status,
        final_stage_plan_hash: final_stage.plan_hash,
        final_stage_driver: final_stage.final_stage_driver,
        final_stage_link_mode: final_stage.final_stage_link_mode,
        host_wrapper_required: final_stage.host_wrapper_required,
        input_count: final_stage.input_count,
        inputs: final_stage.inputs,
        writer_steps,
        writer_blockers,
        notes,
    }
}

fn final_executable_writer_blockers(final_stage: &NsldFinalStagePlanReport) -> Vec<String> {
    if final_stage.host_wrapper_required {
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    } else {
        vec!["final-executable-writer:self-contained:not-implemented".to_owned()]
    }
}

fn final_executable_writer_steps(final_stage: &NsldFinalStagePlanReport) -> Vec<String> {
    if final_stage.host_wrapper_required {
        vec![
            "consume-native-object-output".to_owned(),
            "consume-nsld-container-and-payload".to_owned(),
            "consume-closure-snapshot".to_owned(),
            "prepare-host-assisted-entry-wrapper".to_owned(),
            "invoke-host-finalizer-driver".to_owned(),
            "verify-final-executable-boundary".to_owned(),
        ]
    } else {
        vec![
            "consume-nsld-container-and-payload".to_owned(),
            "consume-closure-snapshot".to_owned(),
            "assemble-self-contained-entrypoint".to_owned(),
            "verify-final-executable-boundary".to_owned(),
        ]
    }
}

fn final_executable_writer_command_args(
    report: &NsldFinalExecutableWriterPlanReport,
    plan: &nuisc::linker::LinkPlan,
) -> Vec<String> {
    let mut args = Vec::new();
    args.push(report.final_stage_driver.clone());
    if !plan.cpu_target.clang_target.is_empty() {
        args.push("-target".to_owned());
        args.push(plan.cpu_target.clang_target.clone());
    }
    if let Some(native_object) = report
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0003.native-object")
    {
        args.push(native_object.path.clone());
    }
    args.push("-o".to_owned());
    args.push(report.output_path.clone());
    args
}

fn resolve_host_driver_path(driver: &str) -> Option<String> {
    if driver.is_empty() {
        return None;
    }
    let driver_path = Path::new(driver);
    if driver_path.components().count() > 1 {
        return driver_path
            .is_file()
            .then(|| driver_path.display().to_string());
    }
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths).find_map(|dir| {
        let candidate = dir.join(driver);
        candidate.is_file().then(|| candidate.display().to_string())
    })
}

fn non_empty_toml_string(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn optional_bool_toml(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "false".to_owned())
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
