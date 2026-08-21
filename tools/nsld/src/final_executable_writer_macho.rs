use crate::{
    reports::{
        NsldMachOArm64MaterializationPreviewReport, NsldMachOArm64PatchApplicationReport,
        NsldMachOArm64PlatformPatchApplicationReport, NsldMachOArm64PlatformStructurePlanReport,
    },
    toml,
};
use std::fmt::Write as _;

pub(crate) fn render_macho_materialization_preview(
    out: &mut String,
    report: &NsldMachOArm64MaterializationPreviewReport,
) {
    writeln!(
        out,
        "finalizer_input_materialization_contract = \"{}\"",
        toml::escape_toml_string(&report.contract)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_status = \"{}\"",
        toml::escape_toml_string(&report.status)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_placement_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.placement_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_relocation_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.relocation_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_image_span_bytes = {}",
        report.image_span_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_copied_bytes = {}",
        report.copied_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_zero_fill_bytes = {}",
        report.zero_fill_bytes
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_image_hash = \"{}\"",
        toml::escape_toml_string(&report.image_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_patch_plan_hash = \"{}\"",
        toml::escape_toml_string(&report.patch_plan_hash)
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_planned_direct_count = {}",
        report.planned_direct_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_previewed_patch_count = {}",
        report.previewed_patch_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_deferred_patch_count = {}",
        report.deferred_patch_count
    )
    .unwrap();
    writeln!(
        out,
        "finalizer_input_materialization_metadata_record_count = {}",
        report.metadata_record_count
    )
    .unwrap();
    let sections = report
        .section_audits
        .iter()
        .map(|section| {
            format!(
                "{}|{}|{}|{}|{}|{}",
                section.section_id,
                section.output_offset,
                section.size_bytes,
                section.copied_bytes,
                section.zero_fill_bytes,
                section.content_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_materialization_sections = [{}]",
        toml::toml_string_array_literal(&sections)
    )
    .unwrap();
    let patches = report
        .patches
        .iter()
        .map(|patch| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                patch.relocation_id,
                patch.relocation_kind,
                patch.source_output_offset,
                patch.width_bytes,
                option_usize(patch.target_output_offset),
                patch
                    .target_absolute_value
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                patch.effective_addend,
                patch.source_bytes_hex,
                patch.encoded_bytes_hex,
                patch.source_bytes_hash,
                patch.encoded_bytes_hash,
                patch.audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_materialization_patches = [{}]",
        toml::toml_string_array_literal(&patches)
    )
    .unwrap();
}

pub(crate) fn render_macho_patch_application(
    out: &mut String,
    report: &NsldMachOArm64PatchApplicationReport,
) {
    let string_fields = [
        ("contract", report.contract.as_str()),
        ("status", report.status.as_str()),
        ("placement_plan_hash", report.placement_plan_hash.as_str()),
        ("relocation_plan_hash", report.relocation_plan_hash.as_str()),
        ("patch_plan_hash", report.patch_plan_hash.as_str()),
        ("original_image_hash", report.original_image_hash.as_str()),
        ("applied_image_hash", report.applied_image_hash.as_str()),
        (
            "application_ledger_hash",
            report.application_ledger_hash.as_str(),
        ),
    ];
    for (name, value) in string_fields {
        writeln!(
            out,
            "finalizer_input_patch_application_{name} = \"{}\"",
            toml::escape_toml_string(value)
        )
        .unwrap();
    }
    let count_fields = [
        ("image_span_bytes", report.image_span_bytes),
        ("expected_patch_count", report.expected_patch_count),
        ("applied_patch_count", report.applied_patch_count),
        ("deferred_patch_count", report.deferred_patch_count),
        ("write_once_span_count", report.write_once_span_count),
    ];
    for (name, value) in count_fields {
        writeln!(out, "finalizer_input_patch_application_{name} = {value}").unwrap();
    }
    let patches = report
        .patches
        .iter()
        .map(|patch| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}",
                patch.relocation_id,
                patch.source_output_offset,
                patch.width_bytes,
                patch.source_bytes_hash,
                patch.encoded_bytes_hash,
                patch.post_write_bytes_hash,
                patch.preview_audit_hash,
                patch.write_audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_patch_application_patches = [{}]",
        toml::toml_string_array_literal(&patches)
    )
    .unwrap();
}

pub(crate) fn render_macho_platform_structure_plan(
    out: &mut String,
    report: &NsldMachOArm64PlatformStructurePlanReport,
) {
    let string_fields = [
        ("contract", report.contract.as_str()),
        ("status", report.status.as_str()),
        ("placement_plan_hash", report.placement_plan_hash.as_str()),
        ("relocation_plan_hash", report.relocation_plan_hash.as_str()),
        (
            "patch_application_ledger_hash",
            report.patch_application_ledger_hash.as_str(),
        ),
        ("applied_image_hash", report.applied_image_hash.as_str()),
        ("plan_hash", report.plan_hash.as_str()),
    ];
    for (name, value) in string_fields {
        writeln!(
            out,
            "finalizer_input_platform_structure_{name} = \"{}\"",
            toml::escape_toml_string(value)
        )
        .unwrap();
    }
    let count_fields = [
        ("base_image_span_bytes", report.base_image_span_bytes),
        ("planned_image_span_bytes", report.planned_image_span_bytes),
        ("registered_rule_count", report.registered_rule_count),
        (
            "deferred_relocation_count",
            report.deferred_relocation_count,
        ),
        ("target_count", report.target_count),
        ("stub_region_offset", report.stub_region_offset),
        ("stub_region_bytes", report.stub_region_bytes),
        ("stub_entry_size", report.stub_entry_size),
        ("stub_alignment", report.stub_alignment),
        ("stub_entry_count", report.stub_entry_count),
        ("got_region_offset", report.got_region_offset),
        ("got_region_bytes", report.got_region_bytes),
        ("got_entry_size", report.got_entry_size),
        ("got_alignment", report.got_alignment),
        ("got_entry_count", report.got_entry_count),
    ];
    for (name, value) in count_fields {
        writeln!(out, "finalizer_input_platform_structure_{name} = {value}").unwrap();
    }
    let targets = report
        .targets
        .iter()
        .map(|target| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                target.structure_id,
                target.target_key,
                target.target_symbol,
                target.resolver_status,
                option_text(target.target_object_id.as_deref()),
                option_text(target.target_section_id.as_deref()),
                option_usize(target.target_output_offset),
                target
                    .target_absolute_value
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                option_usize(target.got_slot_index),
                option_usize(target.got_output_offset),
                option_usize(target.stub_slot_index),
                option_usize(target.stub_output_offset),
                target.relocation_ids.join(","),
                target.audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_platform_structure_targets = [{}]",
        toml::toml_string_array_literal(&targets)
    )
    .unwrap();
    let bindings = report
        .relocation_bindings
        .iter()
        .map(|binding| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                binding.relocation_id,
                binding.relocation_kind,
                binding.action_kind,
                binding.source_output_offset,
                binding.width_bytes,
                binding.structure_id,
                binding.patch_target_kind,
                binding.patch_target_output_offset,
                binding.audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_platform_structure_bindings = [{}]",
        toml::toml_string_array_literal(&bindings)
    )
    .unwrap();
}

pub(crate) fn render_macho_platform_patch_application(
    out: &mut String,
    report: &NsldMachOArm64PlatformPatchApplicationReport,
) {
    let string_fields = [
        ("contract", report.contract.as_str()),
        ("status", report.status.as_str()),
        ("placement_plan_hash", report.placement_plan_hash.as_str()),
        ("relocation_plan_hash", report.relocation_plan_hash.as_str()),
        (
            "direct_patch_application_ledger_hash",
            report.direct_patch_application_ledger_hash.as_str(),
        ),
        (
            "platform_structure_plan_hash",
            report.platform_structure_plan_hash.as_str(),
        ),
        (
            "base_applied_image_hash",
            report.base_applied_image_hash.as_str(),
        ),
        ("platform_image_hash", report.platform_image_hash.as_str()),
        (
            "application_ledger_hash",
            report.application_ledger_hash.as_str(),
        ),
    ];
    for (name, value) in string_fields {
        writeln!(
            out,
            "finalizer_input_platform_patch_application_{name} = \"{}\"",
            toml::escape_toml_string(value)
        )
        .unwrap();
    }
    let count_fields = [
        ("base_image_span_bytes", report.base_image_span_bytes),
        (
            "platform_image_span_bytes",
            report.platform_image_span_bytes,
        ),
        (
            "expected_deferred_patch_count",
            report.expected_deferred_patch_count,
        ),
        (
            "applied_deferred_patch_count",
            report.applied_deferred_patch_count,
        ),
        ("stub_write_count", report.stub_write_count),
        ("got_write_count", report.got_write_count),
        ("unresolved_bind_count", report.unresolved_bind_count),
        ("write_once_span_count", report.write_once_span_count),
    ];
    for (name, value) in count_fields {
        writeln!(
            out,
            "finalizer_input_platform_patch_application_{name} = {value}"
        )
        .unwrap();
    }
    let writes = report
        .structure_writes
        .iter()
        .map(|write| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                write.write_id,
                write.structure_id,
                write.write_kind,
                write.target_symbol,
                write.output_offset,
                write.width_bytes,
                write.encoded_bytes_hex,
                write.encoded_bytes_hash,
                write.write_audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_platform_patch_application_structure_writes = [{}]",
        toml::toml_string_array_literal(&writes)
    )
    .unwrap();
    let patches = report
        .patches
        .iter()
        .map(|patch| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                patch.relocation_id,
                patch.relocation_kind,
                patch.source_output_offset,
                patch.width_bytes,
                patch.patch_target_output_offset,
                patch.effective_addend,
                patch.source_bytes_hex,
                patch.encoded_bytes_hex,
                patch.source_bytes_hash,
                patch.encoded_bytes_hash,
                patch.binding_audit_hash,
                patch.write_audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_platform_patch_application_patches = [{}]",
        toml::toml_string_array_literal(&patches)
    )
    .unwrap();
    let binds = report
        .bind_records
        .iter()
        .map(|bind| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}",
                bind.bind_id,
                bind.structure_id,
                bind.target_key,
                bind.target_symbol,
                bind.got_output_offset,
                bind.width_bytes,
                bind.placeholder_bytes_hash,
                bind.status,
                bind.audit_hash
            )
        })
        .collect::<Vec<_>>();
    writeln!(
        out,
        "finalizer_input_platform_patch_application_bind_records = [{}]",
        toml::toml_string_array_literal(&binds)
    )
    .unwrap();
}

fn option_text(value: Option<&str>) -> &str {
    value.unwrap_or("none")
}

fn option_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned())
}
