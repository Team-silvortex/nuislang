use super::{
    final_executable_paths::nsld_final_executable_writer_input_path,
    final_executable_summary::nsld_final_executable_writer_plan_report,
    final_executable_writer::{
        final_executable_writer_command_args, render_final_executable_writer_input,
    },
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableWriterInputEmitReport, NsldFinalExecutableWriterInputVerifyReport,
    },
    toml,
};
use std::{fs, path::Path};

#[cfg(test)]
#[path = "final_executable_writer_input_tests.rs"]
mod tests;

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
        actual_writer_blockers,
    ) = match actual.as_ref() {
        Ok(source) => (
            Some(fnv1a64_hex(source.as_bytes())),
            toml::string_value(source, "final_stage_plan_hash"),
            toml::string_value(source, "writer_kind"),
            toml::string_value(source, "writer_status"),
            toml::usize_value(source, "command_arg_count"),
            toml::string_array_value(source, "command_args"),
            toml::string_array_value(source, "writer_blockers"),
        ),
        Err(error) => {
            issues.push(error.clone());
            (None, None, None, None, None, Vec::new(), Vec::new())
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
        if actual_writer_blockers != expected_plan.writer_blockers {
            issues.push(format!(
                "writer_blockers mismatch: expected [{}], found [{}]",
                expected_plan.writer_blockers.join(", "),
                actual_writer_blockers.join(", ")
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
        expected_writer_blockers: expected_plan.writer_blockers,
        actual_writer_blockers,
        issues,
    }
}
