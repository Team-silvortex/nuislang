use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformWriteAudit {
    pub(crate) write_id: String,
    pub(crate) structure_id: String,
    pub(crate) write_kind: String,
    pub(crate) target_symbol: String,
    pub(crate) image_offset: usize,
    pub(crate) virtual_address: u64,
    pub(crate) width_bytes: usize,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) post_write_bytes_hash: String,
    pub(crate) encoded_bytes_hex: String,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformPatchAudit {
    pub(crate) relocation_id: String,
    pub(crate) structure_id: String,
    pub(crate) rule_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) source_file_offset: usize,
    pub(crate) source_image_offset: usize,
    pub(crate) source_virtual_address: u64,
    pub(crate) width_bytes: usize,
    pub(crate) patch_target_image_offset: usize,
    pub(crate) patch_target_virtual_address: u64,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) post_write_bytes_hash: String,
    pub(crate) binding_audit_hash: String,
    pub(crate) write_audit_hash: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformDynamicBindRecord {
    pub(crate) bind_id: String,
    pub(crate) structure_id: String,
    pub(crate) target_key: String,
    pub(crate) target_symbol: String,
    pub(crate) dynamic_symbol_index: usize,
    pub(crate) got_image_offset: usize,
    pub(crate) got_virtual_address: u64,
    pub(crate) relocation_image_offset: usize,
    pub(crate) relocation_kind: String,
    pub(crate) relocation_type: u32,
    pub(crate) relocation_offset: u64,
    pub(crate) relocation_info: u64,
    pub(crate) relocation_addend: i64,
    pub(crate) status: String,
    pub(crate) audit_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PlatformPatchApplicationReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) application_ledger_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) base_patch_application_ledger_hash: String,
    pub(crate) platform_structure_plan_hash: String,
    pub(crate) base_applied_file_image_hash: String,
    pub(crate) base_applied_memory_image_hash: String,
    pub(crate) applied_file_image_hash: String,
    pub(crate) applied_memory_image_hash: String,
    pub(crate) base_file_span_bytes: usize,
    pub(crate) base_memory_span_bytes: usize,
    pub(crate) applied_file_span_bytes: usize,
    pub(crate) applied_memory_span_bytes: usize,
    pub(crate) expected_structure_write_count: usize,
    pub(crate) applied_structure_write_count: usize,
    pub(crate) expected_deferred_patch_count: usize,
    pub(crate) applied_deferred_patch_count: usize,
    pub(crate) plt_write_count: usize,
    pub(crate) got_write_count: usize,
    pub(crate) dynamic_symbol_write_count: usize,
    pub(crate) dynamic_string_write_count: usize,
    pub(crate) dynamic_relocation_write_count: usize,
    pub(crate) unresolved_dynamic_bind_count: usize,
    pub(crate) write_once_span_count: usize,
    pub(crate) structure_writes: Vec<ElfAmd64PlatformWriteAudit>,
    pub(crate) patches: Vec<ElfAmd64PlatformPatchAudit>,
    pub(crate) dynamic_bind_records: Vec<ElfAmd64PlatformDynamicBindRecord>,
}

impl ElfAmd64PlatformPatchApplicationReport {
    pub(crate) fn canonical_ledger(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, self.contract);
        append_text(&mut out, &self.status);
        append_text(&mut out, &self.placement_plan_hash);
        append_text(&mut out, &self.relocation_plan_hash);
        append_text(&mut out, &self.base_patch_application_ledger_hash);
        append_text(&mut out, &self.platform_structure_plan_hash);
        append_text(&mut out, &self.base_applied_file_image_hash);
        append_text(&mut out, &self.base_applied_memory_image_hash);
        append_text(&mut out, &self.applied_file_image_hash);
        append_text(&mut out, &self.applied_memory_image_hash);
        writeln!(
            out,
            "spans={}|{}|{}|{}",
            self.base_file_span_bytes,
            self.base_memory_span_bytes,
            self.applied_file_span_bytes,
            self.applied_memory_span_bytes
        )
        .unwrap();
        writeln!(
            out,
            "counts={}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.expected_structure_write_count,
            self.applied_structure_write_count,
            self.expected_deferred_patch_count,
            self.applied_deferred_patch_count,
            self.plt_write_count,
            self.got_write_count,
            self.dynamic_symbol_write_count,
            self.dynamic_string_write_count,
            self.dynamic_relocation_write_count,
            self.unresolved_dynamic_bind_count,
            self.write_once_span_count,
            self.dynamic_bind_records.len()
        )
        .unwrap();
        for write in &self.structure_writes {
            append_write(&mut out, write);
        }
        for patch in &self.patches {
            append_patch(&mut out, patch);
        }
        for bind in &self.dynamic_bind_records {
            append_bind(&mut out, bind);
        }
        out
    }
}

pub(super) fn write_audit_hash(write: &ElfAmd64PlatformWriteAudit) -> String {
    let mut out = String::new();
    append_write_body(&mut out, write);
    crate::fnv1a64_hex(out.as_bytes())
}

pub(super) fn patch_audit_hash(patch: &ElfAmd64PlatformPatchAudit) -> String {
    let mut out = String::new();
    append_patch_body(&mut out, patch);
    crate::fnv1a64_hex(out.as_bytes())
}

pub(crate) fn bind_audit_hash(bind: &ElfAmd64PlatformDynamicBindRecord) -> String {
    let mut out = String::new();
    append_bind_body(&mut out, bind);
    crate::fnv1a64_hex(out.as_bytes())
}

fn append_write(out: &mut String, write: &ElfAmd64PlatformWriteAudit) {
    append_write_body(out, write);
    append_text(out, &write.audit_hash);
}

fn append_write_body(out: &mut String, write: &ElfAmd64PlatformWriteAudit) {
    append_text(out, &write.write_id);
    append_text(out, &write.structure_id);
    append_text(out, &write.write_kind);
    append_text(out, &write.target_symbol);
    append_text(out, &write.source_bytes_hash);
    append_text(out, &write.encoded_bytes_hash);
    append_text(out, &write.post_write_bytes_hash);
    append_text(out, &write.encoded_bytes_hex);
    append_text(out, &write.status);
    writeln!(
        out,
        "write={}|{}|{}",
        write.image_offset, write.virtual_address, write.width_bytes
    )
    .unwrap();
}

fn append_patch(out: &mut String, patch: &ElfAmd64PlatformPatchAudit) {
    append_patch_body(out, patch);
    append_text(out, &patch.write_audit_hash);
}

fn append_patch_body(out: &mut String, patch: &ElfAmd64PlatformPatchAudit) {
    append_text(out, &patch.relocation_id);
    append_text(out, &patch.structure_id);
    append_text(out, &patch.rule_id);
    append_text(out, &patch.relocation_kind);
    append_text(out, &patch.source_bytes_hash);
    append_text(out, &patch.encoded_bytes_hash);
    append_text(out, &patch.post_write_bytes_hash);
    append_text(out, &patch.binding_audit_hash);
    append_text(out, &patch.status);
    writeln!(
        out,
        "patch={}|{}|{}|{}|{}|{}",
        patch.source_file_offset,
        patch.source_image_offset,
        patch.source_virtual_address,
        patch.width_bytes,
        patch.patch_target_image_offset,
        patch.patch_target_virtual_address
    )
    .unwrap();
}

fn append_bind(out: &mut String, bind: &ElfAmd64PlatformDynamicBindRecord) {
    append_bind_body(out, bind);
    append_text(out, &bind.audit_hash);
}

fn append_bind_body(out: &mut String, bind: &ElfAmd64PlatformDynamicBindRecord) {
    append_text(out, &bind.bind_id);
    append_text(out, &bind.structure_id);
    append_text(out, &bind.target_key);
    append_text(out, &bind.target_symbol);
    append_text(out, &bind.relocation_kind);
    append_text(out, &bind.status);
    writeln!(
        out,
        "bind={}|{}|{}|{}|{}|{}|{}|{}|{}",
        bind.dynamic_symbol_index,
        bind.got_image_offset,
        bind.got_virtual_address,
        bind.relocation_image_offset,
        bind.relocation_type,
        bind.relocation_offset,
        bind.relocation_info,
        bind.relocation_addend,
        bind.target_symbol.len()
    )
    .unwrap();
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
