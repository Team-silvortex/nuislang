use super::{
    final_stage::{nsld_verify_final_executable_emit_report, nsld_verify_final_stage_plan_report},
    fnv1a64_hex,
    reports::NsldFinalExecutableOutputReport,
};
use std::{fs, path::Path};

pub(crate) fn nsld_final_executable_output_report(
    manifest: &Path,
    plan: &nuisc::linker::LinkPlan,
) -> NsldFinalExecutableOutputReport {
    let final_stage = nsld_verify_final_stage_plan_report(manifest, plan);
    let final_emit = nsld_verify_final_executable_emit_report(manifest, plan);
    let output_path = plan.final_stage.output_path.clone();
    let output_bytes = fs::read(&output_path);
    let present = output_bytes.is_ok();
    let size_bytes = output_bytes.as_ref().ok().map(Vec::len);
    let output_hash = output_bytes.as_ref().ok().map(|bytes| fnv1a64_hex(bytes));
    let mut blockers = Vec::new();
    let mut issues = Vec::new();

    if !final_stage.valid {
        blockers.push("final-stage-plan:invalid".to_owned());
        issues.extend(
            final_stage
                .issues
                .iter()
                .map(|issue| format!("final-stage-plan:{issue}")),
        );
    }
    if !final_emit.valid {
        blockers.push("final-executable-emit:invalid".to_owned());
        issues.extend(
            final_emit
                .issues
                .iter()
                .map(|issue| format!("final-executable-emit:{issue}")),
        );
    }
    if final_emit.actual_emitted != Some(true) {
        blockers.push("final-executable-emit:not-emitted".to_owned());
    }
    if let Err(error) = &output_bytes {
        blockers.push("final-executable-output:missing".to_owned());
        issues.push(format!(
            "missing_or_unreadable_final_executable_output `{output_path}`: {error}"
        ));
    }

    let runnable_candidate =
        present && final_stage.valid && final_emit.valid && final_emit.actual_emitted == Some(true);

    NsldFinalExecutableOutputReport {
        manifest: manifest.display().to_string(),
        output_path,
        present,
        size_bytes,
        output_hash,
        final_stage_plan_valid: final_stage.valid,
        final_stage_plan_hash: final_stage.actual_plan_hash,
        final_executable_emit_valid: final_emit.valid,
        final_executable_emitted: final_emit.actual_emitted,
        final_executable_blocker_count: final_emit.actual_blocker_count,
        runnable_candidate,
        blockers,
        issues,
    }
}
