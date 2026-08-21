use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformTargetPlan {
    pub(crate) structure_id: String,
    pub(crate) rule_ids: Vec<String>,
    pub(crate) target_class: String,
    pub(crate) target_key: String,
    pub(crate) target_symbol: String,
    pub(crate) resolver_status: String,
    pub(crate) dynamic_symbol_index: usize,
    pub(crate) dynamic_string_offset: usize,
    pub(crate) dynamic_symbol_image_offset: usize,
    pub(crate) dynamic_string_image_offset: usize,
    pub(crate) plt_slot_index: Option<usize>,
    pub(crate) plt_image_offset: Option<usize>,
    pub(crate) plt_virtual_address: Option<u64>,
    pub(crate) plt_got_displacement: Option<i64>,
    pub(crate) got_slot_index: Option<usize>,
    pub(crate) got_image_offset: Option<usize>,
    pub(crate) got_virtual_address: Option<u64>,
    pub(crate) dynamic_relocation_index: Option<usize>,
    pub(crate) dynamic_relocation_image_offset: Option<usize>,
    pub(crate) dynamic_relocation_kind: Option<String>,
    pub(crate) dynamic_relocation_type: Option<u32>,
    pub(crate) dynamic_relocation_offset: Option<u64>,
    pub(crate) dynamic_relocation_info: Option<u64>,
    pub(crate) dynamic_relocation_addend: Option<i64>,
    pub(crate) relocation_ids: Vec<String>,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformRelocationBinding {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) action_kind: String,
    pub(crate) rule_id: String,
    pub(crate) structure_id: String,
    pub(crate) source_file_offset: usize,
    pub(crate) source_image_offset: usize,
    pub(crate) source_virtual_address: u64,
    pub(crate) width_bytes: usize,
    pub(crate) patch_target_kind: String,
    pub(crate) patch_target_image_offset: usize,
    pub(crate) patch_target_virtual_address: u64,
    pub(crate) computed_value: i64,
    pub(crate) encoded_value: u64,
    pub(crate) encoded_bytes: Vec<u8>,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformStructurePlanReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) registry_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) patch_application_ledger_hash: String,
    pub(crate) applied_memory_image_hash: String,
    pub(crate) image_base: u64,
    pub(crate) base_file_span_bytes: usize,
    pub(crate) base_memory_span_bytes: usize,
    pub(crate) planned_file_span_bytes: usize,
    pub(crate) planned_memory_span_bytes: usize,
    pub(crate) registered_rule_count: usize,
    pub(crate) deferred_relocation_count: usize,
    pub(crate) target_count: usize,
    pub(crate) plt_region_image_offset: usize,
    pub(crate) plt_region_bytes: usize,
    pub(crate) plt_entry_size: usize,
    pub(crate) plt_alignment: usize,
    pub(crate) plt_entry_count: usize,
    pub(crate) got_region_image_offset: usize,
    pub(crate) got_region_bytes: usize,
    pub(crate) got_entry_size: usize,
    pub(crate) got_alignment: usize,
    pub(crate) got_entry_count: usize,
    pub(crate) metadata_region_image_offset: usize,
    pub(crate) metadata_region_bytes: usize,
    pub(crate) dynamic_symbol_region_image_offset: usize,
    pub(crate) dynamic_symbol_region_bytes: usize,
    pub(crate) dynamic_symbol_entry_size: usize,
    pub(crate) dynamic_symbol_entry_count: usize,
    pub(crate) dynamic_string_region_image_offset: usize,
    pub(crate) dynamic_string_region_bytes: usize,
    pub(crate) dynamic_relocation_region_image_offset: usize,
    pub(crate) dynamic_relocation_region_bytes: usize,
    pub(crate) dynamic_relocation_entry_size: usize,
    pub(crate) dynamic_relocation_alignment: usize,
    pub(crate) dynamic_relocation_entry_count: usize,
    pub(crate) targets: Vec<ElfAmd64PlatformTargetPlan>,
    pub(crate) relocation_bindings: Vec<ElfAmd64PlatformRelocationBinding>,
}

impl ElfAmd64PlatformStructurePlanReport {
    pub(crate) fn canonical_plan(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, self.contract);
        append_text(&mut out, &self.status);
        append_text(&mut out, &self.registry_hash);
        append_text(&mut out, &self.placement_plan_hash);
        append_text(&mut out, &self.relocation_plan_hash);
        append_text(&mut out, &self.patch_application_ledger_hash);
        append_text(&mut out, &self.applied_memory_image_hash);
        writeln!(
            out,
            "envelope={}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.image_base,
            self.base_file_span_bytes,
            self.base_memory_span_bytes,
            self.planned_file_span_bytes,
            self.planned_memory_span_bytes,
            self.registered_rule_count,
            self.deferred_relocation_count,
            self.target_count,
            self.targets.len(),
            self.relocation_bindings.len()
        )
        .unwrap();
        append_layout(&mut out, self);
        for target in &self.targets {
            append_target(&mut out, target, true);
        }
        for binding in &self.relocation_bindings {
            append_binding(&mut out, binding, true);
        }
        out
    }
}

pub(super) fn target_audit_hash(target: &ElfAmd64PlatformTargetPlan) -> String {
    let mut out = String::new();
    append_target(&mut out, target, false);
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn binding_audit_hash(binding: &ElfAmd64PlatformRelocationBinding) -> String {
    let mut out = String::new();
    append_binding(&mut out, binding, false);
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_layout(out: &mut String, report: &ElfAmd64PlatformStructurePlanReport) {
    writeln!(
        out,
        "plt={}|{}|{}|{}|{}",
        report.plt_region_image_offset,
        report.plt_region_bytes,
        report.plt_entry_size,
        report.plt_alignment,
        report.plt_entry_count
    )
    .unwrap();
    writeln!(
        out,
        "got={}|{}|{}|{}|{}",
        report.got_region_image_offset,
        report.got_region_bytes,
        report.got_entry_size,
        report.got_alignment,
        report.got_entry_count
    )
    .unwrap();
    writeln!(
        out,
        "metadata={}|{}|{}|{}|{}|{}|{}|{}",
        report.metadata_region_image_offset,
        report.metadata_region_bytes,
        report.dynamic_symbol_region_image_offset,
        report.dynamic_symbol_region_bytes,
        report.dynamic_symbol_entry_size,
        report.dynamic_symbol_entry_count,
        report.dynamic_string_region_image_offset,
        report.dynamic_string_region_bytes
    )
    .unwrap();
    writeln!(
        out,
        "rela={}|{}|{}|{}|{}",
        report.dynamic_relocation_region_image_offset,
        report.dynamic_relocation_region_bytes,
        report.dynamic_relocation_entry_size,
        report.dynamic_relocation_alignment,
        report.dynamic_relocation_entry_count
    )
    .unwrap();
}

fn append_target(out: &mut String, target: &ElfAmd64PlatformTargetPlan, audit: bool) {
    append_text(out, &target.structure_id);
    append_text(out, &target.target_class);
    append_text(out, &target.target_key);
    append_text(out, &target.target_symbol);
    append_text(out, &target.resolver_status);
    append_text(
        out,
        target.dynamic_relocation_kind.as_deref().unwrap_or("none"),
    );
    for rule_id in &target.rule_ids {
        append_text(out, rule_id);
    }
    writeln!(
        out,
        "dynamic={}|{}|{}|{}|{}|{}|{}|{}|{}",
        target.dynamic_symbol_index,
        target.dynamic_string_offset,
        target.dynamic_symbol_image_offset,
        target.dynamic_string_image_offset,
        optional_usize(target.dynamic_relocation_index),
        optional_usize(target.dynamic_relocation_image_offset),
        optional_u32(target.dynamic_relocation_type),
        optional_u64(target.dynamic_relocation_offset),
        optional_u64(target.dynamic_relocation_info)
    )
    .unwrap();
    writeln!(
        out,
        "slots={}|{}|{}|{}|{}|{}|{}|{}",
        optional_usize(target.plt_slot_index),
        optional_usize(target.plt_image_offset),
        optional_u64(target.plt_virtual_address),
        optional_i64(target.plt_got_displacement),
        optional_usize(target.got_slot_index),
        optional_usize(target.got_image_offset),
        optional_u64(target.got_virtual_address),
        optional_i64(target.dynamic_relocation_addend)
    )
    .unwrap();
    for relocation_id in &target.relocation_ids {
        append_text(out, relocation_id);
    }
    if audit {
        append_text(out, &target.audit_hash);
    }
}

fn append_binding(out: &mut String, binding: &ElfAmd64PlatformRelocationBinding, audit: bool) {
    append_text(out, &binding.relocation_id);
    append_text(out, &binding.relocation_kind);
    append_text(out, &binding.action_kind);
    append_text(out, &binding.rule_id);
    append_text(out, &binding.structure_id);
    append_text(out, &binding.patch_target_kind);
    append_text(out, &binding.encoded_bytes_hash);
    writeln!(
        out,
        "binding={}|{}|{}|{}|{}|{}|{}|{}|{}",
        binding.source_file_offset,
        binding.source_image_offset,
        binding.source_virtual_address,
        binding.width_bytes,
        binding.patch_target_image_offset,
        binding.patch_target_virtual_address,
        binding.computed_value,
        binding.encoded_value,
        hex_bytes(&binding.encoded_bytes)
    )
    .unwrap();
    if audit {
        append_text(out, &binding.audit_hash);
    }
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn optional_i64(value: Option<i64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
