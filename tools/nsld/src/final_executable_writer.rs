use super::{
    reports::{
        NsldFinalExecutableHostInvokePlanReport, NsldFinalExecutableWriterPlanReport,
        NsldFinalStagePlanReport,
    },
    toml,
};
use std::{env, path::Path};

pub(crate) fn final_executable_writer_blockers(
    final_stage: &NsldFinalStagePlanReport,
) -> Vec<String> {
    if final_stage.host_wrapper_required {
        vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
    } else {
        Vec::new()
    }
}

pub(crate) fn final_executable_writer_steps(final_stage: &NsldFinalStagePlanReport) -> Vec<String> {
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

pub(crate) fn final_executable_writer_command_args(
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

pub(crate) fn resolve_host_driver_path(driver: &str) -> Option<String> {
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

pub(crate) fn render_final_executable_host_invoke_plan(
    report: &NsldFinalExecutableHostInvokePlanReport,
) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-final-executable-host-invoke-plan-v1\"\n");
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
        "writer_input_path = \"{}\"\n",
        toml::escape_toml_string(&report.writer_input_path)
    ));
    out.push_str(&format!(
        "invocation_kind = \"{}\"\n",
        toml::escape_toml_string(&report.invocation_kind)
    ));
    out.push_str(&format!(
        "invocation_policy = \"{}\"\n",
        toml::escape_toml_string(&report.invocation_policy)
    ));
    out.push_str(&format!(
        "invocation_policy_reason = \"{}\"\n",
        toml::escape_toml_string(&report.invocation_policy_reason)
    ));
    out.push_str(&format!(
        "requires_explicit_allow = {}\n",
        report.requires_explicit_allow
    ));
    out.push_str(&format!(
        "explicit_allow_present = {}\n",
        report.explicit_allow_present
    ));
    out.push_str(&format!(
        "environment_ready = {}\n",
        report.environment_ready
    ));
    out.push_str(&format!("driver_available = {}\n", report.driver_available));
    out.push_str(&format!(
        "driver_resolved_path = \"{}\"\n",
        toml::escape_toml_string(report.driver_resolved_path.as_deref().unwrap_or(""))
    ));
    out.push_str(&format!(
        "can_invoke_host_finalizer = {}\n",
        report.can_invoke_host_finalizer
    ));
    out.push_str(&format!("would_invoke = {}\n", report.would_invoke));
    out.push_str(&format!(
        "command_arg_count = {}\n",
        report.command_arg_count
    ));
    out.push_str(&format!(
        "command_args = [{}]\n",
        toml::toml_string_array_literal(&report.command_args)
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
