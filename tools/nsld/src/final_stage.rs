use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    closure::{nsld_closure_report, nsld_verify_closure_report},
    final_executable_image::{
        encode_final_executable_image, parse_final_executable_image_header,
        verify_final_executable_image_payload_region, FINAL_EXECUTABLE_IMAGE_FORMAT,
        FINAL_EXECUTABLE_IMAGE_HEADER_SIZE, FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT,
        FINAL_EXECUTABLE_IMAGE_VERSION,
    },
    final_executable_layout::{
        final_executable_byte_map_entries, final_executable_payloads,
        nsld_final_executable_byte_map_hash, nsld_final_executable_layout_hash,
    },
    final_executable_paths::{
        nsld_final_executable_blocked_path, nsld_final_executable_host_invoke_plan_path,
        nsld_final_executable_image_dry_run_bytes_path, nsld_final_executable_image_dry_run_path,
        nsld_final_executable_layout_plan_path, nsld_final_executable_writer_input_path,
        nsld_final_stage_plan_path,
    },
    final_executable_render::{
        optional_bool_toml, optional_usize_toml, render_final_executable_blocked,
        render_final_executable_image_dry_run, render_final_executable_layout_plan,
        render_final_stage_plan,
    },
    final_executable_writer::{
        final_executable_writer_blockers, final_executable_writer_command_args,
        final_executable_writer_steps, render_final_executable_host_invoke_plan,
        render_final_executable_writer_input, resolve_host_driver_path,
    },
    final_stage_plan::{final_stage_input, final_stage_notes, nsld_final_stage_plan_hash},
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableEmitReport, NsldFinalExecutableEmitVerifyReport,
        NsldFinalExecutableHostDryRunReport, NsldFinalExecutableHostInvokePlanEmitReport,
        NsldFinalExecutableHostInvokePlanReport, NsldFinalExecutableHostInvokePlanVerifyReport,
        NsldFinalExecutableImageDryRunEmitReport, NsldFinalExecutableImageDryRunReport,
        NsldFinalExecutableImageDryRunVerifyReport, NsldFinalExecutableLayoutPlanEmitReport,
        NsldFinalExecutableLayoutPlanReport, NsldFinalExecutableLayoutPlanVerifyReport,
        NsldFinalExecutableWriterInputEmitReport, NsldFinalExecutableWriterInputVerifyReport,
        NsldFinalExecutableWriterPlanReport, NsldFinalStagePlanEmitReport,
        NsldFinalStagePlanReport, NsldFinalStagePlanVerifyReport,
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
    let invocation_policy = "dry-run-only".to_owned();
    let invocation_policy_reason = "alpha-host-finalizer-execution-disabled".to_owned();
    blockers.push(format!("host-finalizer-policy:{invocation_policy}"));
    let environment_ready = writer_input.valid && driver_available;
    let can_invoke_host_finalizer = environment_ready
        && writer_plan.writer_blockers.is_empty()
        && invocation_policy == "allow-host-invoke";
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
        invocation_policy,
        invocation_policy_reason,
        can_invoke_host_finalizer,
        blockers,
        notes,
    }
}

pub(crate) fn nsld_final_executable_host_invoke_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableHostInvokePlanReport {
    let dry_run = nsld_final_executable_host_dry_run_report(manifest, plan);
    let requires_explicit_allow = true;
    let explicit_allow_present = false;
    let mut blockers = dry_run.blockers.clone();
    if requires_explicit_allow && !explicit_allow_present {
        blockers.push("host-finalizer-explicit-allow:missing".to_owned());
    }
    let would_invoke = dry_run.can_invoke_host_finalizer && explicit_allow_present;
    let invocation_kind = "host-finalizer-command".to_owned();
    let mut notes = dry_run.notes.clone();
    notes.push("host-invoke-plan-is-non-mutating".to_owned());
    notes.push("host-finalizer-process-is-not-spawned".to_owned());

    NsldFinalExecutableHostInvokePlanReport {
        manifest: manifest.display().to_string(),
        output_path: plan.final_stage.output_path.clone(),
        writer_input_path: dry_run.writer_input_path,
        invocation_kind,
        invocation_policy: dry_run.invocation_policy,
        invocation_policy_reason: dry_run.invocation_policy_reason,
        requires_explicit_allow,
        explicit_allow_present,
        environment_ready: dry_run.environment_ready,
        driver_available: dry_run.driver_available,
        driver_resolved_path: dry_run.driver_resolved_path,
        can_invoke_host_finalizer: dry_run.can_invoke_host_finalizer,
        would_invoke,
        command_arg_count: dry_run.command_arg_count,
        command_args: dry_run.command_args,
        blockers,
        notes,
    }
}

pub(crate) fn nsld_emit_final_executable_host_invoke_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableHostInvokePlanEmitReport, String> {
    let report = nsld_final_executable_host_invoke_plan_report(manifest, plan);
    let source = render_final_executable_host_invoke_plan(&report);
    let output_path = nsld_final_executable_host_invoke_plan_path(plan);
    fs::write(&output_path, &source).map_err(|error| {
        format!(
            "failed to write nsld final executable host invoke plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldFinalExecutableHostInvokePlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        invoke_plan_hash: fnv1a64_hex(source.as_bytes()),
        invocation_policy: report.invocation_policy,
        requires_explicit_allow: report.requires_explicit_allow,
        explicit_allow_present: report.explicit_allow_present,
        would_invoke: report.would_invoke,
        blocker_count: report.blockers.len(),
    })
}

pub(crate) fn nsld_verify_final_executable_host_invoke_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableHostInvokePlanVerifyReport {
    let expected_report = nsld_final_executable_host_invoke_plan_report(manifest, plan);
    let expected_source = render_final_executable_host_invoke_plan(&expected_report);
    let expected_hash = fnv1a64_hex(expected_source.as_bytes());
    let input_path = nsld_final_executable_host_invoke_plan_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_host_invoke_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_hash,
        actual_invocation_policy,
        actual_requires_explicit_allow,
        actual_explicit_allow_present,
        actual_would_invoke,
        actual_command_arg_count,
        actual_command_args,
        actual_blocker_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::string_value(source, "invocation_policy"),
            toml::bool_value(source, "requires_explicit_allow"),
            toml::bool_value(source, "explicit_allow_present"),
            toml::bool_value(source, "would_invoke"),
            toml::usize_value(source, "command_arg_count"),
            toml::string_array_value(source, "command_args"),
            toml::usize_value(source, "blocker_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, Vec::new(), None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-host-invoke-plan-content-mismatch".to_owned());
        }
        if actual_invocation_policy.as_deref() != Some(expected_report.invocation_policy.as_str()) {
            issues.push(format!(
                "invocation_policy mismatch: expected {}, found {}",
                expected_report.invocation_policy,
                actual_invocation_policy
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_requires_explicit_allow != Some(expected_report.requires_explicit_allow) {
            issues.push(format!(
                "requires_explicit_allow mismatch: expected {}, found {}",
                expected_report.requires_explicit_allow,
                actual_requires_explicit_allow
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_explicit_allow_present != Some(expected_report.explicit_allow_present) {
            issues.push(format!(
                "explicit_allow_present mismatch: expected {}, found {}",
                expected_report.explicit_allow_present,
                actual_explicit_allow_present
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_would_invoke != Some(expected_report.would_invoke) {
            issues.push(format!(
                "would_invoke mismatch: expected {}, found {}",
                expected_report.would_invoke,
                actual_would_invoke
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_command_arg_count != Some(expected_report.command_args.len()) {
            issues.push(format!(
                "command_arg_count mismatch: expected {}, found {}",
                expected_report.command_args.len(),
                actual_command_arg_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_command_args != expected_report.command_args {
            issues.push(format!(
                "command_args mismatch: expected [{}], found [{}]",
                expected_report.command_args.join(", "),
                actual_command_args.join(", ")
            ));
        }
        if actual_blocker_count != Some(expected_report.blockers.len()) {
            issues.push(format!(
                "blocker_count mismatch: expected {}, found {}",
                expected_report.blockers.len(),
                actual_blocker_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldFinalExecutableHostInvokePlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_invoke_plan_hash: expected_hash,
        actual_invoke_plan_hash: actual_hash,
        expected_invocation_policy: expected_report.invocation_policy,
        actual_invocation_policy,
        expected_requires_explicit_allow: expected_report.requires_explicit_allow,
        actual_requires_explicit_allow,
        expected_explicit_allow_present: expected_report.explicit_allow_present,
        actual_explicit_allow_present,
        expected_would_invoke: expected_report.would_invoke,
        actual_would_invoke,
        expected_command_arg_count: expected_report.command_args.len(),
        actual_command_arg_count,
        expected_command_args: expected_report.command_args,
        actual_command_args,
        expected_blocker_count: expected_report.blockers.len(),
        actual_blocker_count,
        issues,
    }
}

pub(crate) fn nsld_final_executable_layout_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLayoutPlanReport {
    let final_stage = nsld_final_stage_plan_report(manifest, plan);
    let native_object = final_stage
        .inputs
        .iter()
        .find(|input| input.input_id == "fsi0003.native-object");
    let native_object_path = native_object
        .map(|input| input.path.clone())
        .unwrap_or_else(|| {
            nsld_artifact_stage_kind_path(&plan.output_dir, NsldArtifactStageKind::ObjectOutput)
                .display()
                .to_string()
        });
    let payloads = final_executable_payloads(&final_stage);
    let payload_names = payloads
        .iter()
        .map(|payload| payload.payload_kind.clone())
        .collect::<Vec<_>>();
    let byte_alignment = 16;
    let byte_map_entries = final_executable_byte_map_entries(&payloads, byte_alignment);
    let byte_span = byte_map_entries
        .last()
        .map(|entry| entry.offset + entry.size_bytes)
        .unwrap_or(0);
    let byte_map_hash = nsld_final_executable_byte_map_hash(&byte_map_entries);
    let platform_envelope_family = if plan.cpu_target.object_format.is_empty() {
        "host-native".to_owned()
    } else {
        plan.cpu_target.object_format.clone()
    };
    let platform_envelope_policy = if final_stage.host_wrapper_required {
        "compatibility-envelope".to_owned()
    } else {
        "self-contained-envelope".to_owned()
    };
    let internal_binary_format = "nuis-hetero-unified-binary".to_owned();
    let lifecycle_entry_hook = "on_process_start".to_owned();
    let scheduler_contract = "deterministic-lifecycle-hook-order".to_owned();
    let data_segment_ordering = "deterministic-data-segment-order".to_owned();
    let compatibility_domain = if final_stage.native_object_required {
        "cffi-native-object".to_owned()
    } else {
        "none".to_owned()
    };
    let compatibility_lifecycle_hook = if final_stage.native_object_required {
        "on_cffi_native_object".to_owned()
    } else {
        "none".to_owned()
    };
    let mut notes = final_stage.notes.clone();
    notes.push("final-executable-layout-is-nsld-owned-protocol".to_owned());
    notes.push("platform-envelope-is-compatibility-shell".to_owned());

    let layout_hash = nsld_final_executable_layout_hash(
        &final_stage.plan_hash,
        &final_stage.final_output_path,
        &final_stage.final_stage_link_mode,
        &platform_envelope_family,
        &platform_envelope_policy,
        &internal_binary_format,
        &lifecycle_entry_hook,
        &scheduler_contract,
        &data_segment_ordering,
        &native_object_path,
        final_stage.native_object_required,
        final_stage.native_object_present,
        &compatibility_domain,
        &compatibility_lifecycle_hook,
        &payloads,
        byte_alignment,
        byte_span,
        &byte_map_hash,
        &byte_map_entries,
        &notes,
    );

    NsldFinalExecutableLayoutPlanReport {
        manifest: final_stage.manifest,
        output_path: final_stage.final_output_path,
        layout_hash,
        final_stage_plan_hash: final_stage.plan_hash,
        final_stage_link_mode: final_stage.final_stage_link_mode,
        platform_envelope_family,
        platform_envelope_policy,
        internal_binary_format,
        lifecycle_entry_hook,
        scheduler_contract,
        data_segment_ordering,
        native_object_path,
        native_object_required: final_stage.native_object_required,
        native_object_present: final_stage.native_object_present,
        compatibility_domain,
        compatibility_lifecycle_hook,
        payload_count: payloads.len(),
        payloads,
        payload_names,
        byte_alignment,
        byte_span,
        byte_map_hash,
        byte_map_entries,
        notes,
    }
}

pub(crate) fn nsld_emit_final_executable_layout_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableLayoutPlanEmitReport, String> {
    let report = nsld_final_executable_layout_plan_report(manifest, plan);
    let output_path = nsld_final_executable_layout_plan_path(plan);
    fs::write(&output_path, render_final_executable_layout_plan(&report)).map_err(|error| {
        format!(
            "failed to write nsld final executable layout plan `{}`: {error}",
            output_path.display()
        )
    })?;

    Ok(NsldFinalExecutableLayoutPlanEmitReport {
        manifest: report.manifest,
        output_path: output_path.display().to_string(),
        layout_hash: report.layout_hash,
        final_stage_plan_hash: report.final_stage_plan_hash,
        payload_count: report.payload_count,
        native_object_present: report.native_object_present,
    })
}

pub(crate) fn nsld_verify_final_executable_layout_plan_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableLayoutPlanVerifyReport {
    let expected = nsld_final_executable_layout_plan_report(manifest, plan);
    let expected_source = render_final_executable_layout_plan(&expected);
    let input_path = nsld_final_executable_layout_plan_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_layout_plan `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_layout_hash,
        actual_payload_count,
        actual_byte_span,
        actual_byte_map_hash,
        actual_lifecycle_entry_hook,
        actual_platform_envelope_family,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "layout_hash"),
            toml::usize_value(source, "payload_count"),
            toml::usize_value(source, "byte_span"),
            toml::string_value(source, "byte_map_hash"),
            toml::string_value(source, "lifecycle_entry_hook"),
            toml::string_value(source, "platform_envelope_family"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-layout-plan-content-mismatch".to_owned());
        }
        if actual_layout_hash.as_deref() != Some(expected.layout_hash.as_str()) {
            issues.push(format!(
                "layout_hash mismatch: expected {}, found {}",
                expected.layout_hash,
                actual_layout_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payload_count != Some(expected.payload_count) {
            issues.push(format!(
                "payload_count mismatch: expected {}, found {}",
                expected.payload_count,
                actual_payload_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_byte_span != Some(expected.byte_span) {
            issues.push(format!(
                "byte_span mismatch: expected {}, found {}",
                expected.byte_span,
                actual_byte_span
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_byte_map_hash.as_deref() != Some(expected.byte_map_hash.as_str()) {
            issues.push(format!(
                "byte_map_hash mismatch: expected {}, found {}",
                expected.byte_map_hash,
                actual_byte_map_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_lifecycle_entry_hook.as_deref() != Some(expected.lifecycle_entry_hook.as_str()) {
            issues.push(format!(
                "lifecycle_entry_hook mismatch: expected {}, found {}",
                expected.lifecycle_entry_hook,
                actual_lifecycle_entry_hook
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_platform_envelope_family.as_deref()
            != Some(expected.platform_envelope_family.as_str())
        {
            issues.push(format!(
                "platform_envelope_family mismatch: expected {}, found {}",
                expected.platform_envelope_family,
                actual_platform_envelope_family
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }

    NsldFinalExecutableLayoutPlanVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        valid: issues.is_empty(),
        expected_layout_hash: expected.layout_hash,
        actual_layout_hash,
        expected_payload_count: expected.payload_count,
        actual_payload_count,
        expected_byte_span: expected.byte_span,
        actual_byte_span,
        expected_byte_map_hash: expected.byte_map_hash,
        actual_byte_map_hash,
        expected_lifecycle_entry_hook: expected.lifecycle_entry_hook,
        actual_lifecycle_entry_hook,
        expected_platform_envelope_family: expected.platform_envelope_family,
        actual_platform_envelope_family,
        issues,
    }
}

pub(crate) fn nsld_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableImageDryRunReport {
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let image = encode_final_executable_image(&layout);
    let mut blockers = Vec::new();
    for payload in &layout.payloads {
        if payload.required && !payload.present {
            blockers.push(format!(
                "missing-final-executable-payload:{}",
                payload.payload_id
            ));
        }
    }
    if layout.byte_map_entries.len() != layout.payloads.len() {
        blockers.push("final-executable-byte-map:payload-count-mismatch".to_owned());
    }
    let image_constructed = image.is_some();
    let image_ready = image_constructed && blockers.is_empty();
    let image_size_bytes = image.as_ref().map(Vec::len);
    let image_hash = image.as_ref().map(|bytes| fnv1a64_hex(bytes));

    NsldFinalExecutableImageDryRunReport {
        manifest: manifest.display().to_string(),
        output_path: nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        image_path: nsld_final_executable_image_dry_run_bytes_path(plan)
            .display()
            .to_string(),
        image_format: FINAL_EXECUTABLE_IMAGE_FORMAT.to_owned(),
        image_magic: FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT.to_owned(),
        image_header_size: FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        payload_byte_offset: FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
        payload_byte_span: layout.byte_span,
        layout_hash: layout.layout_hash,
        byte_map_hash: layout.byte_map_hash,
        payload_count: layout.payload_count,
        byte_span: layout.byte_span,
        image_constructed,
        image_ready,
        image_size_bytes,
        image_hash,
        blockers,
    }
}

pub(crate) fn nsld_emit_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> Result<NsldFinalExecutableImageDryRunEmitReport, String> {
    let report = nsld_final_executable_image_dry_run_report(manifest, plan);
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let image = encode_final_executable_image(&layout);
    let image_emitted = match image {
        Some(bytes) => {
            fs::write(&report.image_path, bytes).map_err(|error| {
                format!(
                    "failed to write nsld final executable image dry-run bytes `{}`: {error}",
                    report.image_path
                )
            })?;
            true
        }
        None => false,
    };
    fs::write(
        &report.output_path,
        render_final_executable_image_dry_run(&report),
    )
    .map_err(|error| {
        format!(
            "failed to write nsld final executable image dry-run `{}`: {error}",
            report.output_path
        )
    })?;

    Ok(NsldFinalExecutableImageDryRunEmitReport {
        manifest: report.manifest,
        output_path: report.output_path,
        image_path: report.image_path,
        image_emitted,
        image_constructed: report.image_constructed,
        image_ready: report.image_ready,
        image_format: report.image_format,
        image_header_size: report.image_header_size,
        payload_byte_offset: report.payload_byte_offset,
        image_size_bytes: report.image_size_bytes,
        image_hash: report.image_hash,
    })
}

pub(crate) fn nsld_verify_final_executable_image_dry_run_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableImageDryRunVerifyReport {
    let expected = nsld_final_executable_image_dry_run_report(manifest, plan);
    let layout = nsld_final_executable_layout_plan_report(manifest, plan);
    let expected_source = render_final_executable_image_dry_run(&expected);
    let input_path = nsld_final_executable_image_dry_run_path(plan);
    let image_path = nsld_final_executable_image_dry_run_bytes_path(plan);
    let mut issues = Vec::new();
    let actual = fs::read_to_string(&input_path).map_err(|error| {
        format!(
            "missing_or_unreadable_final_executable_image_dry_run `{}`: {error}",
            input_path.display()
        )
    });
    let (
        actual_layout_hash,
        actual_byte_map_hash,
        actual_image_header_size,
        actual_payload_byte_offset,
        actual_image_constructed,
        actual_image_ready,
        actual_image_size_bytes,
        actual_image_hash,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "layout_hash"),
            toml::string_value(source, "byte_map_hash"),
            toml::usize_value(source, "image_header_size"),
            toml::usize_value(source, "payload_byte_offset"),
            toml::bool_value(source, "image_constructed"),
            toml::bool_value(source, "image_ready"),
            optional_usize_value(source, "image_size_bytes"),
            non_empty_toml_string(source, "image_hash"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, None, None, None)
        }
    };
    if let Ok(actual) = actual {
        if actual != expected_source {
            issues.push("final-executable-image-dry-run-content-mismatch".to_owned());
        }
        push_optional_string_mismatch(
            &mut issues,
            "layout_hash",
            Some(expected.layout_hash.as_str()),
            actual_layout_hash.as_deref(),
        );
        push_optional_string_mismatch(
            &mut issues,
            "byte_map_hash",
            Some(expected.byte_map_hash.as_str()),
            actual_byte_map_hash.as_deref(),
        );
        if actual_image_header_size != Some(expected.image_header_size) {
            issues.push(format!(
                "image_header_size mismatch: expected {}, found {}",
                expected.image_header_size,
                actual_image_header_size
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_payload_byte_offset != Some(expected.payload_byte_offset) {
            issues.push(format!(
                "payload_byte_offset mismatch: expected {}, found {}",
                expected.payload_byte_offset,
                actual_payload_byte_offset
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_constructed != Some(expected.image_constructed) {
            issues.push(format!(
                "image_constructed mismatch: expected {}, found {}",
                expected.image_constructed,
                actual_image_constructed
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_ready != Some(expected.image_ready) {
            issues.push(format!(
                "image_ready mismatch: expected {}, found {}",
                expected.image_ready,
                actual_image_ready
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_size_bytes != expected.image_size_bytes {
            issues.push(format!(
                "image_size_bytes mismatch: expected {}, found {}",
                optional_usize_toml(expected.image_size_bytes),
                optional_usize_toml(actual_image_size_bytes)
            ));
        }
        if actual_image_hash != expected.image_hash {
            issues.push(format!(
                "image_hash mismatch: expected {}, found {}",
                expected
                    .image_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_image_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
    }
    if let Some(expected_hash) = expected.image_hash.as_deref() {
        match fs::read(&image_path) {
            Ok(bytes) => {
                let actual_hash = fnv1a64_hex(&bytes);
                if actual_hash != expected_hash {
                    issues.push(format!(
                        "image_bytes_hash mismatch: expected {expected_hash}, found {actual_hash}"
                    ));
                }
            }
            Err(error) => issues.push(format!(
                "missing_or_unreadable_final_executable_image_dry_run_bytes `{}`: {error}",
                image_path.display()
            )),
        }
    }
    let (
        actual_image_magic,
        actual_image_version,
        actual_header_image_size,
        actual_header_payload_offset,
        actual_header_payload_span,
        actual_header_layout_hash,
        actual_header_byte_map_hash,
    ) = match fs::read(&image_path) {
        Ok(bytes) => match parse_final_executable_image_header(&bytes) {
            Some(header) => {
                if header.magic != expected.image_magic {
                    issues.push(format!(
                        "image_header_magic mismatch: expected {}, found {}",
                        expected.image_magic, header.magic
                    ));
                }
                if header.version != FINAL_EXECUTABLE_IMAGE_VERSION {
                    issues.push(format!(
                        "image_header_version mismatch: expected {}, found {}",
                        FINAL_EXECUTABLE_IMAGE_VERSION, header.version
                    ));
                }
                if header.header_size != expected.image_header_size {
                    issues.push(format!(
                        "image_header_size_bytes mismatch: expected {}, found {}",
                        expected.image_header_size, header.header_size
                    ));
                }
                if header.payload_offset != expected.payload_byte_offset {
                    issues.push(format!(
                        "image_header_payload_offset mismatch: expected {}, found {}",
                        expected.payload_byte_offset, header.payload_offset
                    ));
                }
                if header.payload_span != expected.payload_byte_span {
                    issues.push(format!(
                        "image_header_payload_span mismatch: expected {}, found {}",
                        expected.payload_byte_span, header.payload_span
                    ));
                }
                if header.layout_hash != expected.layout_hash {
                    issues.push(format!(
                        "image_header_layout_hash mismatch: expected {}, found {}",
                        expected.layout_hash, header.layout_hash
                    ));
                }
                if header.byte_map_hash != expected.byte_map_hash {
                    issues.push(format!(
                        "image_header_byte_map_hash mismatch: expected {}, found {}",
                        expected.byte_map_hash, header.byte_map_hash
                    ));
                }
                (
                    Some(header.magic),
                    Some(header.version),
                    Some(header.header_size),
                    Some(header.payload_offset),
                    Some(header.payload_span),
                    Some(header.layout_hash),
                    Some(header.byte_map_hash),
                )
            }
            None => {
                issues.push("final-executable-image-header:invalid-or-too-short".to_owned());
                (None, None, None, None, None, None, None)
            }
        },
        Err(_) => (None, None, None, None, None, None, None),
    };
    let payload_region =
        verify_final_executable_image_payload_region(&layout, &image_path, &mut issues);

    NsldFinalExecutableImageDryRunVerifyReport {
        manifest: manifest.display().to_string(),
        input_path: input_path.display().to_string(),
        image_path: image_path.display().to_string(),
        valid: issues.is_empty(),
        expected_layout_hash: expected.layout_hash,
        actual_layout_hash,
        expected_byte_map_hash: expected.byte_map_hash,
        actual_byte_map_hash,
        expected_image_magic: expected.image_magic,
        actual_image_magic,
        expected_image_version: FINAL_EXECUTABLE_IMAGE_VERSION,
        actual_image_version,
        expected_image_header_size: expected.image_header_size,
        actual_image_header_size: actual_header_image_size.or(actual_image_header_size),
        expected_payload_byte_offset: expected.payload_byte_offset,
        actual_payload_byte_offset: actual_header_payload_offset.or(actual_payload_byte_offset),
        expected_payload_byte_span: expected.payload_byte_span,
        actual_payload_byte_span: actual_header_payload_span,
        actual_header_layout_hash,
        actual_header_byte_map_hash,
        expected_payload_region_count: layout.byte_map_entries.len(),
        actual_payload_region_count: payload_region.actual_count,
        expected_payload_region_hash: payload_region.expected_hash,
        actual_payload_region_hash: payload_region.actual_hash,
        expected_image_constructed: expected.image_constructed,
        actual_image_constructed,
        expected_image_ready: expected.image_ready,
        actual_image_ready,
        expected_image_size_bytes: expected.image_size_bytes,
        actual_image_size_bytes,
        expected_image_hash: expected.image_hash,
        actual_image_hash,
        issues,
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
        actual_writer_input_valid,
        actual_writer_input_hash,
        actual_writer_input_issues,
        actual_host_environment_ready,
        actual_host_driver_available,
        actual_host_can_invoke,
        actual_host_driver_resolved_path,
        actual_host_invocation_policy,
        actual_host_invocation_policy_reason,
        actual_host_dry_run_command_arg_count,
        actual_host_dry_run_command_args,
        actual_host_dry_run_blocker_count,
        actual_host_dry_run_blockers,
        actual_host_invoke_plan_valid,
        actual_host_invoke_plan_would_invoke,
        actual_host_invoke_plan_hash,
        actual_host_invoke_plan_issues,
        actual_layout_plan_valid,
        actual_layout_plan_hash,
        actual_layout_plan_issues,
        actual_image_dry_run_valid,
        actual_image_dry_run_hash,
        actual_image_dry_run_size_bytes,
        actual_image_dry_run_issues,
        actual_blocker_count,
    ) = match actual.as_ref() {
        Ok(source) => (
            toml::string_value(source, "final_stage_plan_hash"),
            toml::bool_value(source, "emitted"),
            toml::bool_value(source, "writer_input_valid"),
            non_empty_toml_string(source, "writer_input_hash"),
            toml::string_array_value(source, "writer_input_issues"),
            toml::bool_value(source, "host_dry_run_environment_ready"),
            toml::bool_value(source, "host_dry_run_driver_available"),
            toml::bool_value(source, "host_dry_run_can_invoke"),
            non_empty_toml_string(source, "host_dry_run_driver_resolved_path"),
            non_empty_toml_string(source, "host_dry_run_invocation_policy"),
            non_empty_toml_string(source, "host_dry_run_invocation_policy_reason"),
            toml::usize_value(source, "host_dry_run_command_arg_count"),
            toml::string_array_value(source, "host_dry_run_command_args"),
            toml::usize_value(source, "host_dry_run_blocker_count"),
            toml::string_array_value(source, "host_dry_run_blockers"),
            toml::bool_value(source, "host_invoke_plan_valid"),
            toml::bool_value(source, "host_invoke_plan_would_invoke"),
            non_empty_toml_string(source, "host_invoke_plan_hash"),
            toml::string_array_value(source, "host_invoke_plan_issues"),
            toml::bool_value(source, "layout_plan_valid"),
            non_empty_toml_string(source, "layout_plan_hash"),
            toml::string_array_value(source, "layout_plan_issues"),
            toml::bool_value(source, "image_dry_run_valid"),
            non_empty_toml_string(source, "image_dry_run_hash"),
            optional_usize_value(source, "image_dry_run_size_bytes"),
            toml::string_array_value(source, "image_dry_run_issues"),
            toml::usize_value(source, "blocker_count"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (
                None,       // final_stage_plan_hash
                None,       // emitted
                None,       // writer_input_valid
                None,       // writer_input_hash
                Vec::new(), // writer_input_issues
                None,       // host_dry_run_environment_ready
                None,       // host_dry_run_driver_available
                None,       // host_dry_run_can_invoke
                None,       // host_dry_run_driver_resolved_path
                None,       // host_dry_run_invocation_policy
                None,       // host_dry_run_invocation_policy_reason
                None,       // host_dry_run_command_arg_count
                Vec::new(), // host_dry_run_command_args
                None,       // host_dry_run_blocker_count
                Vec::new(), // host_dry_run_blockers
                None,       // host_invoke_plan_valid
                None,       // host_invoke_plan_would_invoke
                None,       // host_invoke_plan_hash
                Vec::new(), // host_invoke_plan_issues
                None,       // layout_plan_valid
                None,       // layout_plan_hash
                Vec::new(), // layout_plan_issues
                None,       // image_dry_run_valid
                None,       // image_dry_run_hash
                None,       // image_dry_run_size_bytes
                Vec::new(), // image_dry_run_issues
                None,       // blocker_count
            )
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
        if actual_writer_input_valid != expected.writer_input_valid {
            issues.push(format!(
                "writer_input_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.writer_input_valid),
                optional_bool_toml(actual_writer_input_valid)
            ));
        }
        if actual_writer_input_hash != expected.writer_input_hash {
            issues.push(format!(
                "writer_input_hash mismatch: expected {}, found {}",
                expected
                    .writer_input_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_writer_input_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_writer_input_issues != expected.writer_input_issues {
            issues.push(format!(
                "writer_input_issues mismatch: expected [{}], found [{}]",
                expected.writer_input_issues.join(", "),
                actual_writer_input_issues.join(", ")
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
        if actual_host_invocation_policy != expected.host_dry_run_invocation_policy {
            issues.push(format!(
                "host_dry_run_invocation_policy mismatch: expected {}, found {}",
                expected
                    .host_dry_run_invocation_policy
                    .as_deref()
                    .unwrap_or("missing"),
                actual_host_invocation_policy
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_host_invocation_policy_reason != expected.host_dry_run_invocation_policy_reason {
            issues.push(format!(
                "host_dry_run_invocation_policy_reason mismatch: expected {}, found {}",
                expected
                    .host_dry_run_invocation_policy_reason
                    .as_deref()
                    .unwrap_or("missing"),
                actual_host_invocation_policy_reason
                    .as_deref()
                    .unwrap_or("missing")
            ));
        }
        if actual_host_dry_run_command_arg_count != Some(expected.host_dry_run_command_args.len()) {
            issues.push(format!(
                "host_dry_run_command_arg_count mismatch: expected {}, found {}",
                expected.host_dry_run_command_args.len(),
                actual_host_dry_run_command_arg_count
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_dry_run_command_args != expected.host_dry_run_command_args {
            issues.push(format!(
                "host_dry_run_command_args mismatch: expected [{}], found [{}]",
                expected.host_dry_run_command_args.join(", "),
                actual_host_dry_run_command_args.join(", ")
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
        if actual_host_invoke_plan_valid != expected.host_invoke_plan_valid {
            issues.push(format!(
                "host_invoke_plan_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.host_invoke_plan_valid),
                optional_bool_toml(actual_host_invoke_plan_valid)
            ));
        }
        if actual_host_invoke_plan_would_invoke != expected.host_invoke_plan_would_invoke {
            issues.push(format!(
                "host_invoke_plan_would_invoke mismatch: expected {}, found {}",
                optional_bool_toml(expected.host_invoke_plan_would_invoke),
                optional_bool_toml(actual_host_invoke_plan_would_invoke)
            ));
        }
        if actual_host_invoke_plan_hash != expected.host_invoke_plan_hash {
            issues.push(format!(
                "host_invoke_plan_hash mismatch: expected {}, found {}",
                expected
                    .host_invoke_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_host_invoke_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_host_invoke_plan_issues != expected.host_invoke_plan_issues {
            issues.push(format!(
                "host_invoke_plan_issues mismatch: expected [{}], found [{}]",
                expected.host_invoke_plan_issues.join(", "),
                actual_host_invoke_plan_issues.join(", ")
            ));
        }
        if actual_layout_plan_valid != expected.layout_plan_valid {
            issues.push(format!(
                "layout_plan_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.layout_plan_valid),
                optional_bool_toml(actual_layout_plan_valid)
            ));
        }
        if actual_layout_plan_hash != expected.layout_plan_hash {
            issues.push(format!(
                "layout_plan_hash mismatch: expected {}, found {}",
                expected
                    .layout_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_layout_plan_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_layout_plan_issues != expected.layout_plan_issues {
            issues.push(format!(
                "layout_plan_issues mismatch: expected [{}], found [{}]",
                expected.layout_plan_issues.join(", "),
                actual_layout_plan_issues.join(", ")
            ));
        }
        if actual_image_dry_run_valid != expected.image_dry_run_valid {
            issues.push(format!(
                "image_dry_run_valid mismatch: expected {}, found {}",
                optional_bool_toml(expected.image_dry_run_valid),
                optional_bool_toml(actual_image_dry_run_valid)
            ));
        }
        if actual_image_dry_run_hash != expected.image_dry_run_hash {
            issues.push(format!(
                "image_dry_run_hash mismatch: expected {}, found {}",
                expected
                    .image_dry_run_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
                actual_image_dry_run_hash
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned())
            ));
        }
        if actual_image_dry_run_size_bytes != expected.image_dry_run_size_bytes {
            issues.push(format!(
                "image_dry_run_size_bytes mismatch: expected {}, found {}",
                optional_usize_toml(expected.image_dry_run_size_bytes),
                optional_usize_toml(actual_image_dry_run_size_bytes)
            ));
        }
        if actual_image_dry_run_issues != expected.image_dry_run_issues {
            issues.push(format!(
                "image_dry_run_issues mismatch: expected [{}], found [{}]",
                expected.image_dry_run_issues.join(", "),
                actual_image_dry_run_issues.join(", ")
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
        expected_writer_input_valid: expected.writer_input_valid,
        actual_writer_input_valid,
        expected_writer_input_hash: expected.writer_input_hash,
        actual_writer_input_hash,
        expected_writer_input_issues: expected.writer_input_issues,
        actual_writer_input_issues,
        expected_host_dry_run_environment_ready: expected.host_dry_run_environment_ready,
        actual_host_dry_run_environment_ready: actual_host_environment_ready,
        expected_host_dry_run_driver_available: expected.host_dry_run_driver_available,
        actual_host_dry_run_driver_available: actual_host_driver_available,
        expected_host_dry_run_can_invoke: expected.host_dry_run_can_invoke,
        actual_host_dry_run_can_invoke: actual_host_can_invoke,
        expected_host_dry_run_driver_resolved_path: expected.host_dry_run_driver_resolved_path,
        actual_host_dry_run_driver_resolved_path: actual_host_driver_resolved_path,
        expected_host_dry_run_invocation_policy: expected.host_dry_run_invocation_policy,
        actual_host_dry_run_invocation_policy: actual_host_invocation_policy,
        expected_host_dry_run_invocation_policy_reason: expected
            .host_dry_run_invocation_policy_reason,
        actual_host_dry_run_invocation_policy_reason: actual_host_invocation_policy_reason,
        expected_host_dry_run_command_arg_count: expected.host_dry_run_command_args.len(),
        actual_host_dry_run_command_arg_count,
        expected_host_dry_run_command_args: expected.host_dry_run_command_args,
        actual_host_dry_run_command_args,
        expected_host_dry_run_blocker_count: expected.host_dry_run_blockers.len(),
        actual_host_dry_run_blocker_count,
        expected_host_dry_run_blockers: expected.host_dry_run_blockers,
        actual_host_dry_run_blockers,
        expected_host_invoke_plan_valid: expected.host_invoke_plan_valid,
        actual_host_invoke_plan_valid,
        expected_host_invoke_plan_would_invoke: expected.host_invoke_plan_would_invoke,
        actual_host_invoke_plan_would_invoke,
        expected_host_invoke_plan_hash: expected.host_invoke_plan_hash,
        actual_host_invoke_plan_hash,
        expected_host_invoke_plan_issues: expected.host_invoke_plan_issues,
        actual_host_invoke_plan_issues,
        expected_layout_plan_valid: expected.layout_plan_valid,
        actual_layout_plan_valid,
        expected_layout_plan_hash: expected.layout_plan_hash,
        actual_layout_plan_hash,
        expected_layout_plan_issues: expected.layout_plan_issues,
        actual_layout_plan_issues,
        expected_image_dry_run_valid: expected.image_dry_run_valid,
        actual_image_dry_run_valid,
        expected_image_dry_run_hash: expected.image_dry_run_hash,
        actual_image_dry_run_hash,
        expected_image_dry_run_size_bytes: expected.image_dry_run_size_bytes,
        actual_image_dry_run_size_bytes,
        expected_image_dry_run_issues: expected.image_dry_run_issues,
        actual_image_dry_run_issues,
        expected_blocker_count: expected.blockers.len(),
        actual_blocker_count,
        issues,
    }
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
        host_dry_run_invocation_policy: None,
        host_dry_run_invocation_policy_reason: None,
        host_dry_run_command_arg_count: 0,
        host_dry_run_command_args: Vec::new(),
        host_dry_run_blocker_count: 0,
        host_dry_run_blockers: Vec::new(),
        host_invoke_plan_path: nsld_final_executable_host_invoke_plan_path(plan)
            .display()
            .to_string(),
        host_invoke_plan_valid: None,
        host_invoke_plan_hash: None,
        host_invoke_plan_would_invoke: None,
        host_invoke_plan_issues: Vec::new(),
        layout_plan_path: nsld_final_executable_layout_plan_path(plan)
            .display()
            .to_string(),
        layout_plan_valid: None,
        layout_plan_hash: None,
        layout_plan_issues: Vec::new(),
        image_dry_run_path: nsld_final_executable_image_dry_run_path(plan)
            .display()
            .to_string(),
        image_dry_run_bytes_path: nsld_final_executable_image_dry_run_bytes_path(plan)
            .display()
            .to_string(),
        image_dry_run_valid: None,
        image_dry_run_hash: None,
        image_dry_run_size_bytes: None,
        image_dry_run_issues: Vec::new(),
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
    let host_invoke_plan = nsld_verify_final_executable_host_invoke_plan_report(manifest, plan);
    let layout_plan = nsld_verify_final_executable_layout_plan_report(manifest, plan);
    let image_dry_run = nsld_verify_final_executable_image_dry_run_report(manifest, plan);
    report.writer_input_path = writer_input.input_path;
    report.writer_input_valid = Some(writer_input.valid);
    report.writer_input_hash = writer_input.actual_writer_input_hash;
    report.writer_input_issues = writer_input.issues;
    report.host_dry_run_environment_ready = Some(host_dry_run.environment_ready);
    report.host_dry_run_driver_available = Some(host_dry_run.driver_available);
    report.host_dry_run_driver_resolved_path = host_dry_run.driver_resolved_path;
    report.host_dry_run_can_invoke = Some(host_dry_run.can_invoke_host_finalizer);
    report.host_dry_run_invocation_policy = Some(host_dry_run.invocation_policy);
    report.host_dry_run_invocation_policy_reason = Some(host_dry_run.invocation_policy_reason);
    report.host_dry_run_command_arg_count = host_dry_run.command_args.len();
    report.host_dry_run_command_args = host_dry_run.command_args;
    report.host_dry_run_blocker_count = host_dry_run.blockers.len();
    report.host_dry_run_blockers = host_dry_run.blockers;
    report.host_invoke_plan_path = host_invoke_plan.input_path;
    report.host_invoke_plan_valid = Some(host_invoke_plan.valid);
    report.host_invoke_plan_hash = host_invoke_plan.actual_invoke_plan_hash;
    report.host_invoke_plan_would_invoke =
        Some(host_invoke_plan.actual_would_invoke.unwrap_or(false));
    report.host_invoke_plan_issues = host_invoke_plan.issues;
    report.layout_plan_path = layout_plan.input_path;
    report.layout_plan_valid = Some(layout_plan.valid);
    report.layout_plan_hash = layout_plan.actual_layout_hash;
    report.layout_plan_issues = layout_plan.issues;
    report.image_dry_run_path = image_dry_run.input_path;
    report.image_dry_run_bytes_path = image_dry_run.image_path;
    report.image_dry_run_valid = Some(image_dry_run.valid);
    report.image_dry_run_hash = image_dry_run.actual_image_hash;
    report.image_dry_run_size_bytes = image_dry_run.actual_image_size_bytes;
    report.image_dry_run_issues = image_dry_run.issues;
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
    if !host_invoke_plan.valid {
        report
            .blockers
            .push("host-finalizer-invoke-plan:invalid".to_owned());
        report.blockers.extend(
            report
                .host_invoke_plan_issues
                .iter()
                .map(|issue| format!("host-finalizer-invoke-plan:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    if host_invoke_plan.actual_would_invoke != Some(true) {
        report
            .blockers
            .push("host-finalizer-invoke-plan:not-allowed".to_owned());
        report.can_emit_final_executable = false;
    }
    if !layout_plan.valid {
        report
            .blockers
            .push("final-executable-layout-plan:invalid".to_owned());
        report.blockers.extend(
            report
                .layout_plan_issues
                .iter()
                .map(|issue| format!("final-executable-layout-plan:{issue}")),
        );
        report.can_emit_final_executable = false;
    }
    if !image_dry_run.valid {
        report
            .blockers
            .push("final-executable-image-dry-run:invalid".to_owned());
        report.blockers.extend(
            report
                .image_dry_run_issues
                .iter()
                .map(|issue| format!("final-executable-image-dry-run:{issue}")),
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

fn non_empty_toml_string(source: &str, key: &str) -> Option<String> {
    toml::string_value(source, key).filter(|value| !value.is_empty())
}

fn optional_usize_value(source: &str, key: &str) -> Option<usize> {
    toml::usize_value(source, key).filter(|value| *value != 0)
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
