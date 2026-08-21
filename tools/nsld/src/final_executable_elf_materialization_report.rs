use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64MaterializationPreviewReport {
    pub(crate) contract: &'static str,
    pub(crate) status: String,
    pub(crate) plan_hash: String,
    pub(crate) placement_plan_hash: String,
    pub(crate) relocation_plan_hash: String,
    pub(crate) source_object_set_hash: String,
    pub(crate) file_image_hash: String,
    pub(crate) memory_image_hash: String,
    pub(crate) file_span_bytes: usize,
    pub(crate) memory_span_bytes: usize,
    pub(crate) copied_bytes: usize,
    pub(crate) zero_fill_bytes: usize,
    pub(crate) input_object_count: usize,
    pub(crate) section_audit_count: usize,
    pub(crate) merged_section_audit_count: usize,
    pub(crate) zero_fill_section_count: usize,
    pub(crate) planned_direct_count: usize,
    pub(crate) previewed_patch_count: usize,
    pub(crate) deferred_patch_count: usize,
    pub(crate) no_op_count: usize,
    pub(crate) object_audits: Vec<ElfAmd64ObjectInputAudit>,
    pub(crate) section_audits: Vec<ElfAmd64SectionMaterializationAudit>,
    pub(crate) merged_section_audits: Vec<ElfAmd64MergedSectionAudit>,
    pub(crate) patches: Vec<ElfAmd64PatchSpanPreview>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64ObjectInputAudit {
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) planned_size_bytes: usize,
    pub(crate) size_bytes: usize,
    pub(crate) planned_source_hash: String,
    pub(crate) source_hash: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64SectionMaterializationAudit {
    pub(crate) object_id: String,
    pub(crate) object_role: String,
    pub(crate) input_section_index: usize,
    pub(crate) input_section_name: String,
    pub(crate) output_section_id: String,
    pub(crate) source_payload_offset: Option<usize>,
    pub(crate) output_file_offset: Option<usize>,
    pub(crate) output_image_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) zero_fill: bool,
    pub(crate) source_bytes_hash: Option<String>,
    pub(crate) materialized_bytes_hash: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64MergedSectionAudit {
    pub(crate) section_id: String,
    pub(crate) class: String,
    pub(crate) file_offset: Option<usize>,
    pub(crate) image_offset: usize,
    pub(crate) size_bytes: usize,
    pub(crate) contribution_count: usize,
    pub(crate) zero_fill: bool,
    pub(crate) materialized_bytes_hash: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ElfAmd64PatchSpanPreview {
    pub(crate) relocation_id: String,
    pub(crate) relocation_kind: String,
    pub(crate) source_file_offset: usize,
    pub(crate) source_image_offset: usize,
    pub(crate) width_bytes: usize,
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) encoded_bytes: Vec<u8>,
    pub(crate) source_bytes_hash: String,
    pub(crate) encoded_bytes_hash: String,
    pub(crate) audit_hash: String,
    pub(crate) status: String,
}

impl ElfAmd64MaterializationPreviewReport {
    pub(crate) fn canonical_plan(&self) -> String {
        let mut out = String::new();
        append_text(&mut out, self.contract);
        append_text(&mut out, &self.status);
        append_text(&mut out, &self.placement_plan_hash);
        append_text(&mut out, &self.relocation_plan_hash);
        append_text(&mut out, &self.source_object_set_hash);
        append_text(&mut out, &self.file_image_hash);
        append_text(&mut out, &self.memory_image_hash);
        writeln!(
            out,
            "spans={}|{}|{}|{}",
            self.file_span_bytes, self.memory_span_bytes, self.copied_bytes, self.zero_fill_bytes
        )
        .unwrap();
        writeln!(
            out,
            "counts={}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.input_object_count,
            self.section_audit_count,
            self.merged_section_audit_count,
            self.zero_fill_section_count,
            self.planned_direct_count,
            self.previewed_patch_count,
            self.deferred_patch_count,
            self.no_op_count,
            self.patches.len()
        )
        .unwrap();
        for audit in &self.object_audits {
            append_text(&mut out, "object");
            append_text(&mut out, &audit.object_id);
            append_text(&mut out, &audit.object_role);
            append_text(&mut out, &audit.planned_source_hash);
            append_text(&mut out, &audit.source_hash);
            append_text(&mut out, &audit.status);
            writeln!(
                out,
                "sizes={}|{}",
                audit.planned_size_bytes, audit.size_bytes
            )
            .unwrap();
        }
        for audit in &self.section_audits {
            append_section_audit(&mut out, audit);
        }
        for audit in &self.merged_section_audits {
            append_merged_section_audit(&mut out, audit);
        }
        for patch in &self.patches {
            append_patch(&mut out, patch);
        }
        out
    }
}

fn append_section_audit(out: &mut String, audit: &ElfAmd64SectionMaterializationAudit) {
    append_text(out, "section");
    append_text(out, &audit.object_id);
    append_text(out, &audit.object_role);
    append_text(out, &audit.input_section_name);
    append_text(out, &audit.output_section_id);
    append_text(out, audit.source_bytes_hash.as_deref().unwrap_or("none"));
    append_text(out, &audit.materialized_bytes_hash);
    append_text(out, &audit.status);
    writeln!(
        out,
        "facts={}|{}|{}|{}|{}|{}|{}",
        audit.input_section_index,
        optional_usize(audit.source_payload_offset),
        optional_usize(audit.output_file_offset),
        audit.output_image_offset,
        audit.size_bytes,
        audit.zero_fill,
        audit.materialized_bytes_hash.len()
    )
    .unwrap();
}

fn append_merged_section_audit(out: &mut String, audit: &ElfAmd64MergedSectionAudit) {
    append_text(out, "merged-section");
    append_text(out, &audit.section_id);
    append_text(out, &audit.class);
    append_text(out, &audit.materialized_bytes_hash);
    append_text(out, &audit.status);
    writeln!(
        out,
        "facts={}|{}|{}|{}|{}|{}",
        optional_usize(audit.file_offset),
        audit.image_offset,
        audit.size_bytes,
        audit.contribution_count,
        audit.zero_fill,
        audit.materialized_bytes_hash.len()
    )
    .unwrap();
}

fn append_patch(out: &mut String, patch: &ElfAmd64PatchSpanPreview) {
    append_text(out, "patch");
    append_text(out, &patch.relocation_id);
    append_text(out, &patch.relocation_kind);
    append_text(out, &patch.source_bytes_hash);
    append_text(out, &patch.encoded_bytes_hash);
    append_text(out, &patch.audit_hash);
    append_text(out, &patch.status);
    writeln!(
        out,
        "facts={}|{}|{}|{}|{}|{}",
        patch.source_file_offset,
        patch.source_image_offset,
        patch.width_bytes,
        hex_bytes(&patch.source_bytes),
        hex_bytes(&patch.encoded_bytes),
        patch.encoded_bytes.len()
    )
    .unwrap();
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn optional_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
