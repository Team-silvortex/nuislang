use super::{
    final_executable_finalizer_registry::{
        executable_finalizer_registry_validation, registered_finalizer_command_args,
        select_executable_finalizer,
    },
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
    plan: &nuisc::linker::LinkPlan,
) -> Vec<String> {
    if !final_stage.host_wrapper_required {
        return Vec::new();
    }

    let registry = executable_finalizer_registry_validation();
    if !registry.valid {
        return vec!["final-executable-finalizer-registry:invalid".to_owned()];
    }
    let selection = match select_executable_finalizer(plan) {
        Ok(selection) => selection,
        Err(_) => {
            return vec!["final-executable-finalizer-provider:not-registered".to_owned()];
        }
    };
    if !selection.ready() {
        return vec![format!(
            "final-executable-finalizer-provider:{}:{}",
            selection.provider_id(),
            selection.provider_status()
        )];
    }

    let input_issues = selection.input_validation_issues(plan);
    if !input_issues.is_empty() {
        return input_issues
            .into_iter()
            .map(|issue| format!("final-executable-finalizer-input:{issue}"))
            .collect();
    }

    if !selection.requires_host_driver() {
        return Vec::new();
    }

    if host_assisted_writer_execution_enabled(plan) {
        return Vec::new();
    }
    if host_finalizer_policy_allows_invoke() {
        return vec!["final-executable-writer:host-assisted:explicit-allow-missing".to_owned()];
    }

    vec!["final-executable-writer:host-assisted:not-implemented".to_owned()]
}

pub(crate) fn final_executable_writer_steps(
    final_stage: &NsldFinalStagePlanReport,
    plan: &nuisc::linker::LinkPlan,
) -> Vec<String> {
    if final_stage.host_wrapper_required {
        if select_executable_finalizer(plan)
            .is_ok_and(|selection| !selection.requires_host_driver())
        {
            return vec![
                "consume-compiled-artifact-native-handoff".to_owned(),
                "validate-relocatable-host-objects".to_owned(),
                "validate-os-native-compatibility-image".to_owned(),
                "atomically-materialize-os-native-executable".to_owned(),
                "verify-final-executable-boundary".to_owned(),
            ];
        }
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
    registered_finalizer_command_args(report, plan)
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

pub(crate) fn host_assisted_writer_execution_enabled(plan: &nuisc::linker::LinkPlan) -> bool {
    select_executable_finalizer(plan).is_ok_and(|selection| {
        selection.ready()
            && selection.input_validation_issues(plan).is_empty()
            && (!selection.requires_host_driver()
                || (host_finalizer_policy_allows_invoke()
                    && host_finalizer_explicit_allow_present()))
    })
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
        "finalizer_contract = \"{}\"",
        toml::escape_toml_string(&report.finalizer_contract)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_registry_hash = \"{}\"",
        toml::escape_toml_string(&report.finalizer_registry_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_registry_valid = {}",
        report.finalizer_registry_valid
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_target_key = \"{}\"",
        toml::escape_toml_string(&report.finalizer_target_key)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_provider_id = \"{}\"",
        toml::escape_toml_string(report.finalizer_provider_id.as_deref().unwrap_or(""))
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_provider_status = \"{}\"",
        toml::escape_toml_string(report.finalizer_provider_status.as_deref().unwrap_or(""))
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_execution_kind = \"{}\"",
        toml::escape_toml_string(report.finalizer_execution_kind.as_deref().unwrap_or(""))
    )
    .unwrap();
    render_finalizer_input_summary(&mut out, report.finalizer_input_summary.as_ref());
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

fn render_finalizer_input_summary(
    out: &mut String,
    summary: Option<&crate::reports::NsldExecutableFinalizerInputSummary>,
) {
    let Some(summary) = summary else {
        out.push_str("finalizer_input_summary_present = false\n");
        return;
    };
    out.push_str("finalizer_input_summary_present = true\n");
    writeln!(
        out,
        "finalizer_input_contract = \"{}\"",
        toml::escape_toml_string(&summary.contract)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_status = \"{}\"",
        toml::escape_toml_string(&summary.status)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_object_count = {}",
        summary.object_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_section_count = {}",
        summary.section_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_symbol_count = {}",
        summary.symbol_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_count = {}",
        summary.relocation_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_defined_symbol_count = {}",
        summary.defined_symbol_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_undefined_symbol_count = {}",
        summary.undefined_symbol_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_internally_resolved_symbol_count = {}",
        summary.internally_resolved_symbol_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_unresolved_external_symbol_count = {}",
        summary.unresolved_external_symbol_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_unresolved_external_symbols = [{}]",
        toml::toml_string_array_literal(&summary.unresolved_external_symbols)
    )
    .unwrap();
    let placement = &summary.placement_binding;
    writeln!(
        out,
        "finalizer_input_placement_contract = \"{}\"",
        toml::escape_toml_string(&placement.contract)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_placement_status = \"{}\"",
        toml::escape_toml_string(&placement.status)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_placement_plan_hash = \"{}\"",
        toml::escape_toml_string(&placement.plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_placement_image_span_bytes = {}",
        placement.image_span_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_merged_section_count = {}",
        placement.merged_sections.len()
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_section_placement_count = {}",
        placement.section_placements.len()
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_common_allocation_count = {}",
        placement.common_allocations.len()
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_symbol_binding_count = {}",
        placement.symbol_bindings.len()
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_internally_bound_symbol_count = {}",
        placement.internally_bound_symbol_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_external_compatibility_symbol_count = {}",
        placement.external_compatibility_symbol_count
    )
    .unwrap();
    let merged_sections = placement
        .merged_sections
        .iter()
        .map(|section| {
            format!(
                "{}|{}|{}|0x{:08x}|{}|{}|{}|{}|{}",
                section.section_id,
                section.segment_name,
                section.section_name,
                section.flags,
                section.alignment,
                section.output_offset,
                section.size_bytes,
                section.contribution_count,
                section.zero_fill
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_merged_sections = [{}]",
        toml::toml_string_array_literal(&merged_sections)
    )
    .unwrap();
    let section_placements = placement
        .section_placements
        .iter()
        .map(|section| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                section.object_id,
                section.object_role,
                section.input_section_ordinal,
                section.input_segment_name,
                section.input_section_name,
                section.output_section_id,
                section.output_offset,
                section.output_section_offset,
                section.size_bytes,
                section.alignment,
                section.zero_fill
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_section_placements = [{}]",
        toml::toml_string_array_literal(&section_placements)
    )
    .unwrap();
    let common_allocations = placement
        .common_allocations
        .iter()
        .map(|allocation| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                allocation.allocation_id,
                allocation.symbol,
                allocation.owner_object_id,
                allocation.owner_object_role,
                allocation.owner_symbol_index,
                allocation.declaration_count,
                allocation.size_bytes,
                allocation.alignment,
                allocation.output_section_id,
                allocation.output_offset,
                allocation.output_section_offset
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_common_allocations = [{}]",
        toml::toml_string_array_literal(&common_allocations)
    )
    .unwrap();
    let symbol_bindings = placement
        .symbol_bindings
        .iter()
        .map(|binding| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                binding.symbol,
                binding.reference_object_id,
                binding.reference_symbol_index,
                binding.status,
                binding.target_object_id.as_deref().unwrap_or("none"),
                binding
                    .target_symbol_index
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                binding.target_kind.as_deref().unwrap_or("none"),
                binding.target_section_id.as_deref().unwrap_or("none"),
                binding
                    .target_output_offset
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned())
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_symbol_bindings = [{}]",
        toml::toml_string_array_literal(&symbol_bindings)
    )
    .unwrap();
    let relocation = &summary.relocation_application;
    writeln!(
        out,
        "finalizer_input_relocation_application_contract = \"{}\"",
        toml::escape_toml_string(&relocation.contract)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_status = \"{}\"",
        toml::escape_toml_string(&relocation.status)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_plan_hash = \"{}\"",
        toml::escape_toml_string(&relocation.plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_placement_plan_hash = \"{}\"",
        toml::escape_toml_string(&relocation.placement_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_count = {}",
        relocation.relocation_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_registered_kind_count = {}",
        relocation.registered_kind_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_ready_count = {}",
        relocation.ready_application_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_platform_structure_count = {}",
        relocation.platform_structure_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_external_compatibility_count = {}",
        relocation.external_compatibility_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_relocation_application_metadata_record_count = {}",
        relocation.metadata_record_count
    )
    .unwrap();
    let relocation_applications = relocation
        .applications
        .iter()
        .map(|item| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                item.relocation_id,
                item.object_id,
                item.object_role,
                item.input_section_ordinal,
                item.source_section_id,
                item.source_offset,
                item.source_output_offset,
                item.width_bytes,
                item.pc_relative,
                item.external,
                item.relocation_type,
                item.relocation_kind,
                item.action_kind,
                item.target_symbol.as_deref().unwrap_or("none"),
                item.target_symbol_index
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                item.target_object_id.as_deref().unwrap_or("none"),
                item.target_section_id.as_deref().unwrap_or("none"),
                item.target_output_offset
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                item.explicit_addend
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                item.pair_relocation_id.as_deref().unwrap_or("none"),
                item.resolver_status,
                item.application_status
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_relocation_applications = [{}]",
        toml::toml_string_array_literal(&relocation_applications)
    )
    .unwrap();
    crate::final_executable_writer_macho::render_macho_materialization_preview(
        out,
        &summary.materialization_preview,
    );
    crate::final_executable_writer_macho::render_macho_patch_application(
        out,
        &summary.patch_application,
    );
    crate::final_executable_writer_macho::render_macho_platform_structure_plan(
        out,
        &summary.platform_structure_plan,
    );
    crate::final_executable_writer_macho::render_macho_platform_patch_application(
        out,
        &summary.platform_patch_application,
    );
    crate::final_executable_writer_macho_shell::render_macho_shell_layout_plan(
        out,
        &summary.shell_layout_plan,
    );
    crate::final_executable_writer_macho_shell::render_macho_shell_image_serialization(
        out,
        &summary.shell_image_serialization,
    );
}
