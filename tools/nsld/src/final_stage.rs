pub(crate) use super::final_executable_emit::{
    nsld_emit_final_executable_report, nsld_verify_final_executable_emit_report,
};
pub(crate) use super::final_executable_host::{
    nsld_emit_final_executable_host_invoke_plan_report, nsld_final_executable_host_dry_run_report,
    nsld_final_executable_host_invoke_plan_report,
    nsld_verify_final_executable_host_invoke_plan_report,
};
pub(crate) use super::final_executable_image_stage::{
    nsld_emit_final_executable_image_dry_run_report, nsld_final_executable_image_dry_run_report,
    nsld_verify_final_executable_image_dry_run_report,
};
pub(crate) use super::final_executable_layout_stage::{
    nsld_emit_final_executable_layout_plan_report, nsld_final_executable_layout_plan_report,
    nsld_verify_final_executable_layout_plan_report,
};
pub(crate) use super::final_executable_summary::{
    nsld_final_executable_readiness_report, nsld_final_executable_writer_plan_report,
};
pub(crate) use super::final_executable_writer_input::{
    nsld_emit_final_executable_writer_input_report,
    nsld_verify_final_executable_writer_input_report,
};
use super::{
    artifact_chain::{nsld_artifact_stage_kind_path, NsldArtifactStageKind},
    closure::{nsld_closure_report, nsld_verify_closure_report},
    final_executable_paths::nsld_final_stage_plan_path,
    final_executable_render::render_final_stage_plan,
    final_stage_plan::{final_stage_input, final_stage_notes, nsld_final_stage_plan_hash},
    reports::{
        NsldFinalStagePlanEmitReport, NsldFinalStagePlanReport, NsldFinalStagePlanVerifyReport,
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
