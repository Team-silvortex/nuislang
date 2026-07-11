use super::{
    reports::{
        NsldFinalExecutableHostInvokePlanReport, NsldFinalExecutableWriterPlanReport,
        NsldFinalStagePlanReport,
    },
    toml,
};
use std::{env, fmt::Write as _, path::Path};

const HOST_FINALIZER_ALLOW_ENV: &str = "NUIS_NSLD_ALLOW_HOST_FINALIZER";
const HOST_FINALIZER_POLICY_ENV: &str = "NUIS_NSLD_HOST_FINALIZER_POLICY";

pub(crate) fn final_executable_writer_blockers(
    final_stage: &NsldFinalStagePlanReport,
) -> Vec<String> {
    if !final_stage.host_wrapper_required {
        return Vec::new();
    }

    if host_assisted_writer_execution_enabled() {
        return Vec::new();
    }
    if host_finalizer_policy_allows_invoke() {
        return vec!["final-executable-writer:host-assisted:explicit-allow-missing".to_owned()];
    }

    vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
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
    let mut args = Vec::with_capacity(if plan.cpu_target.clang_target.is_empty() {
        4
    } else {
        6
    });
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

pub(crate) fn host_assisted_writer_execution_enabled() -> bool {
    host_finalizer_policy_allows_invoke() && host_finalizer_explicit_allow_present()
}

fn host_finalizer_policy_allows_invoke() -> bool {
    env::var(HOST_FINALIZER_POLICY_ENV)
        .map(|value| {
            let value = value.trim();
            value == "allow-host-invoke" || value.eq_ignore_ascii_case("allow")
        })
        .unwrap_or(false)
}

fn host_finalizer_explicit_allow_present() -> bool {
    env::var(HOST_FINALIZER_ALLOW_ENV)
        .map(|value| {
            let value = value.trim();
            value == "1"
                || value.eq_ignore_ascii_case("true")
                || value.eq_ignore_ascii_case("yes")
                || value.eq_ignore_ascii_case("allow")
        })
        .unwrap_or(false)
}

pub(crate) fn render_final_executable_writer_input(
    report: &NsldFinalExecutableWriterPlanReport,
    plan: &nuisc::linker::LinkPlan,
) -> String {
    let command_args = final_executable_writer_command_args(report, plan);
    let mut out = String::with_capacity(1024 + report.inputs.len() * 192);
    out.push_str("schema = \"nuis-nsld-final-executable-writer-input-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    writeln!(
        out,
        "manifest = \"{}\"",
        toml::escape_toml_string(&report.manifest)
    )
    .unwrap();
    writeln!(
        out,
        "output_path = \"{}\"",
        toml::escape_toml_string(&report.output_path)
    )
    .unwrap();
    writeln!(
        out,
        "writer_kind = \"{}\"",
        toml::escape_toml_string(&report.writer_kind)
    )
    .unwrap();
    writeln!(
        out,
        "writer_status = \"{}\"",
        toml::escape_toml_string(&report.writer_status)
    )
    .unwrap();
    writeln!(
        out,
        "final_stage_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.final_stage_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "final_stage_driver = \"{}\"",
        toml::escape_toml_string(&report.final_stage_driver)
    )
    .unwrap();
    writeln!(
        out,
        "final_stage_link_mode = \"{}\"",
        toml::escape_toml_string(&report.final_stage_link_mode)
    )
    .unwrap();
    writeln!(
        out,
        "host_wrapper_required = {}",
        report.host_wrapper_required
    )
    .unwrap();
    writeln!(out, "command_arg_count = {}", command_args.len()).unwrap();
    writeln!(
        out,
        "command_args = [{}]",
        toml::toml_string_array_literal(&command_args)
    )
    .unwrap();
    writeln!(
        out,
        "writer_steps = [{}]",
        toml::toml_string_array_literal(&report.writer_steps)
    )
    .unwrap();
    writeln!(
        out,
        "writer_blockers = [{}]",
        toml::toml_string_array_literal(&report.writer_blockers)
    )
    .unwrap();
    writeln!(
        out,
        "notes = [{}]",
        toml::toml_string_array_literal(&report.notes)
    )
    .unwrap();
    for input in &report.inputs {
        out.push_str("\n[[final_stage_input]]\n");
        writeln!(out, "order_index = {}", input.order_index).unwrap();
        writeln!(
            out,
            "input_id = \"{}\"",
            toml::escape_toml_string(&input.input_id)
        )
        .unwrap();
        writeln!(
            out,
            "input_kind = \"{}\"",
            toml::escape_toml_string(&input.input_kind)
        )
        .unwrap();
        writeln!(out, "path = \"{}\"", toml::escape_toml_string(&input.path)).unwrap();
        writeln!(
            out,
            "content_hash = \"{}\"",
            toml::escape_toml_string(&input.content_hash)
        )
        .unwrap();
        writeln!(out, "required = {}", input.required).unwrap();
        writeln!(out, "present = {}", input.present).unwrap();
    }
    out
}

pub(crate) fn render_final_executable_host_invoke_plan(
    report: &NsldFinalExecutableHostInvokePlanReport,
) -> String {
    let mut out = String::with_capacity(1024 + report.command_args.len() * 64);
    out.push_str("schema = \"nuis-nsld-final-executable-host-invoke-plan-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("producer = \"nsld\"\n");
    out.push_str("producer_phase = \"alpha-0.10.0\"\n");
    writeln!(
        out,
        "manifest = \"{}\"",
        toml::escape_toml_string(&report.manifest)
    )
    .unwrap();
    writeln!(
        out,
        "output_path = \"{}\"",
        toml::escape_toml_string(&report.output_path)
    )
    .unwrap();
    writeln!(
        out,
        "writer_input_path = \"{}\"",
        toml::escape_toml_string(&report.writer_input_path)
    )
    .unwrap();
    writeln!(
        out,
        "invocation_kind = \"{}\"",
        toml::escape_toml_string(&report.invocation_kind)
    )
    .unwrap();
    writeln!(
        out,
        "invocation_policy = \"{}\"",
        toml::escape_toml_string(&report.invocation_policy)
    )
    .unwrap();
    writeln!(
        out,
        "invocation_policy_reason = \"{}\"",
        toml::escape_toml_string(&report.invocation_policy_reason)
    )
    .unwrap();
    writeln!(
        out,
        "requires_explicit_allow = {}",
        report.requires_explicit_allow
    )
    .unwrap();
    writeln!(
        out,
        "explicit_allow_present = {}",
        report.explicit_allow_present
    )
    .unwrap();
    writeln!(out, "environment_ready = {}", report.environment_ready).unwrap();
    writeln!(out, "driver_available = {}", report.driver_available).unwrap();
    writeln!(
        out,
        "driver_resolved_path = \"{}\"",
        toml::escape_toml_string(report.driver_resolved_path.as_deref().unwrap_or(""))
    )
    .unwrap();
    writeln!(
        out,
        "can_invoke_host_finalizer = {}",
        report.can_invoke_host_finalizer
    )
    .unwrap();
    writeln!(out, "would_invoke = {}", report.would_invoke).unwrap();
    writeln!(out, "command_arg_count = {}", report.command_arg_count).unwrap();
    writeln!(
        out,
        "command_args = [{}]",
        toml::toml_string_array_literal(&report.command_args)
    )
    .unwrap();
    writeln!(out, "blocker_count = {}", report.blockers.len()).unwrap();
    writeln!(
        out,
        "blockers = [{}]",
        toml::toml_string_array_literal(&report.blockers)
    )
    .unwrap();
    writeln!(
        out,
        "notes = [{}]",
        toml::toml_string_array_literal(&report.notes)
    )
    .unwrap();
    out
}
