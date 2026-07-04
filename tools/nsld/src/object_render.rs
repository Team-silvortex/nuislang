use super::{
    reports::{NsldObjectByteLayoutReport, NsldObjectFileLayoutReport},
    reports::{NsldObjectPlanReport, NsldObjectWriterDryRunReport},
    toml::{escape_toml_string, toml_string_array_literal},
    NSLD_LINK_INPUT_TABLE_PRODUCER, NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE, NSLD_OBJECT_PLAN_KIND,
    NSLD_OBJECT_PLAN_SCHEMA, NSLD_OBJECT_PLAN_SCHEMA_VERSION,
};

pub(crate) fn render_object_plan(report: &NsldObjectPlanReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "schema = \"{}\"\n",
        escape_toml_string(NSLD_OBJECT_PLAN_SCHEMA)
    ));
    out.push_str(&format!(
        "schema_version = {NSLD_OBJECT_PLAN_SCHEMA_VERSION}\n"
    ));
    out.push_str(&format!(
        "plan_kind = \"{}\"\n",
        escape_toml_string(NSLD_OBJECT_PLAN_KIND)
    ));
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!("ready = {}\n", report.ready));
    out.push_str(&format!(
        "target_arch = \"{}\"\n",
        escape_toml_string(&report.target_arch)
    ));
    out.push_str(&format!(
        "target_os = \"{}\"\n",
        escape_toml_string(&report.target_os)
    ));
    out.push_str(&format!(
        "object_format = \"{}\"\n",
        escape_toml_string(&report.object_format)
    ));
    out.push_str(&format!(
        "calling_abi = \"{}\"\n",
        escape_toml_string(&report.calling_abi)
    ));
    out.push_str(&format!(
        "clang_target = \"{}\"\n",
        escape_toml_string(&report.clang_target)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "source_container_path = \"{}\"\n",
        escape_toml_string(&report.source_container_path)
    ));
    out.push_str(&format!(
        "source_payload_path = \"{}\"\n",
        escape_toml_string(&report.source_payload_path)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "section_table_hash = \"{}\"\n",
        escape_toml_string(&report.section_table_hash)
    ));
    out.push_str(&format!(
        "object_plan_hash = \"{}\"\n",
        escape_toml_string(&report.object_plan_hash)
    ));
    out.push_str(&format!(
        "object_layout_hash = \"{}\"\n",
        escape_toml_string(&report.object_layout_hash)
    ));
    out.push_str(&format!(
        "relocation_seed_count = {}\n",
        report.relocation_seed_count
    ));
    out.push_str(&format!(
        "relocation_seed_table_hash = \"{}\"\n",
        escape_toml_string(&report.relocation_seed_table_hash)
    ));
    out.push_str(&format!(
        "writer_target_id = \"{}\"\n",
        escape_toml_string(&report.writer_target_id)
    ));
    out.push_str(&format!(
        "writer_status = \"{}\"\n",
        escape_toml_string(&report.writer_status)
    ));
    out.push_str(&format!(
        "unsupported_features = [{}]\n",
        toml_string_array_literal(&report.unsupported_features)
    ));
    out.push_str(&format!(
        "emission_status = \"{}\"\n",
        escape_toml_string(&report.emission_status)
    ));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.object_sections {
        out.push_str("\n[[object_section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            escape_toml_string(&section.source_section_id)
        ));
        out.push_str(&format!(
            "source_section_kind = \"{}\"\n",
            escape_toml_string(&section.source_section_kind)
        ));
        out.push_str(&format!(
            "object_section_name = \"{}\"\n",
            escape_toml_string(&section.object_section_name)
        ));
        out.push_str(&format!(
            "object_section_role = \"{}\"\n",
            escape_toml_string(&section.object_section_role)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!(
            "source_size_bytes = {}\n",
            section.source_size_bytes
        ));
        out.push_str(&format!(
            "payload_offset_seed = {}\n",
            section.payload_offset_seed
        ));
        out.push_str(&format!(
            "file_offset_seed = {}\n",
            section.file_offset_seed
        ));
        out.push_str(&format!("file_size_seed = {}\n", section.file_size_seed));
        out.push_str(&format!("alignment = {}\n", section.alignment));
        out.push_str(&format!("required = {}\n", section.required));
    }
    for seed in &report.relocation_seeds {
        out.push_str("\n[[object_relocation_seed]]\n");
        out.push_str(&format!("order_index = {}\n", seed.order_index));
        out.push_str(&format!(
            "relocation_seed_id = \"{}\"\n",
            escape_toml_string(&seed.relocation_seed_id)
        ));
        out.push_str(&format!(
            "relocation_seed_kind = \"{}\"\n",
            escape_toml_string(&seed.relocation_seed_kind)
        ));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            escape_toml_string(&seed.source_section_id)
        ));
        out.push_str(&format!(
            "source_offset_seed = {}\n",
            seed.source_offset_seed
        ));
        out.push_str(&format!(
            "target_symbol = \"{}\"\n",
            escape_toml_string(&seed.target_symbol)
        ));
        out.push_str(&format!("addend = {}\n", seed.addend));
        out.push_str(&format!(
            "native_relocation_ready = {}\n",
            seed.native_relocation_ready
        ));
    }
    out
}

pub(crate) fn render_object_writer_input(report: &NsldObjectPlanReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-object-writer-input-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-writer-input\"\n");
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!(
        "writer_target_id = \"{}\"\n",
        escape_toml_string(&report.writer_target_id)
    ));
    out.push_str(&format!(
        "writer_status = \"{}\"\n",
        escape_toml_string(&report.writer_status)
    ));
    out.push_str(&format!(
        "target_arch = \"{}\"\n",
        escape_toml_string(&report.target_arch)
    ));
    out.push_str(&format!(
        "target_os = \"{}\"\n",
        escape_toml_string(&report.target_os)
    ));
    out.push_str(&format!(
        "object_format = \"{}\"\n",
        escape_toml_string(&report.object_format)
    ));
    out.push_str(&format!(
        "object_plan_hash = \"{}\"\n",
        escape_toml_string(&report.object_plan_hash)
    ));
    out.push_str(&format!(
        "object_layout_hash = \"{}\"\n",
        escape_toml_string(&report.object_layout_hash)
    ));
    out.push_str(&format!(
        "relocation_seed_table_hash = \"{}\"\n",
        escape_toml_string(&report.relocation_seed_table_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "relocation_seed_count = {}\n",
        report.relocation_seed_count
    ));
    out.push_str(&format!(
        "source_container_path = \"{}\"\n",
        escape_toml_string(&report.source_container_path)
    ));
    out.push_str(&format!(
        "source_payload_path = \"{}\"\n",
        escape_toml_string(&report.source_payload_path)
    ));
    for section in &report.object_sections {
        out.push_str("\n[[writer_section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            escape_toml_string(&section.source_section_id)
        ));
        out.push_str(&format!(
            "object_section_name = \"{}\"\n",
            escape_toml_string(&section.object_section_name)
        ));
        out.push_str(&format!(
            "object_section_role = \"{}\"\n",
            escape_toml_string(&section.object_section_role)
        ));
        out.push_str(&format!(
            "source_path = \"{}\"\n",
            escape_toml_string(&section.source_path)
        ));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
        out.push_str(&format!(
            "source_size_bytes = {}\n",
            section.source_size_bytes
        ));
        out.push_str(&format!(
            "file_offset_seed = {}\n",
            section.file_offset_seed
        ));
        out.push_str(&format!("file_size_seed = {}\n", section.file_size_seed));
        out.push_str(&format!("alignment = {}\n", section.alignment));
        out.push_str(&format!("required = {}\n", section.required));
    }
    for seed in &report.relocation_seeds {
        out.push_str("\n[[writer_relocation_seed]]\n");
        out.push_str(&format!("order_index = {}\n", seed.order_index));
        out.push_str(&format!(
            "relocation_seed_id = \"{}\"\n",
            escape_toml_string(&seed.relocation_seed_id)
        ));
        out.push_str(&format!(
            "relocation_seed_kind = \"{}\"\n",
            escape_toml_string(&seed.relocation_seed_kind)
        ));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            escape_toml_string(&seed.source_section_id)
        ));
        out.push_str(&format!(
            "source_offset_seed = {}\n",
            seed.source_offset_seed
        ));
        out.push_str(&format!(
            "target_symbol = \"{}\"\n",
            escape_toml_string(&seed.target_symbol)
        ));
        out.push_str(&format!("addend = {}\n", seed.addend));
        out.push_str(&format!(
            "native_relocation_ready = {}\n",
            seed.native_relocation_ready
        ));
    }
    out
}

pub(crate) fn render_object_writer_dry_run(report: &NsldObjectWriterDryRunReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-object-writer-dry-run-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-writer-dry-run\"\n");
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "writer_input_path = \"{}\"\n",
        escape_toml_string(&report.writer_input_path)
    ));
    out.push_str(&format!(
        "planned_output_path = \"{}\"\n",
        escape_toml_string(&report.planned_output_path)
    ));
    out.push_str(&format!(
        "writer_target_id = \"{}\"\n",
        escape_toml_string(&report.writer_target_id)
    ));
    out.push_str(&format!(
        "object_plan_hash = \"{}\"\n",
        escape_toml_string(&report.object_plan_hash)
    ));
    out.push_str(&format!(
        "object_layout_hash = \"{}\"\n",
        escape_toml_string(&report.object_layout_hash)
    ));
    out.push_str(&format!(
        "relocation_seed_table_hash = \"{}\"\n",
        escape_toml_string(&report.relocation_seed_table_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!(
        "relocation_seed_count = {}\n",
        report.relocation_seed_count
    ));
    out.push_str(&format!(
        "writer_input_valid = {}\n",
        report.writer_input_valid
    ));
    out.push_str(&format!("can_emit_object = {}\n", report.can_emit_object));
    out.push_str(&format!("dry_run_ready = {}\n", report.dry_run_ready));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    out
}

pub(crate) fn render_object_byte_layout(report: &NsldObjectByteLayoutReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-object-byte-layout-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-byte-layout\"\n");
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "object_plan_hash = \"{}\"\n",
        escape_toml_string(&report.object_plan_hash)
    ));
    out.push_str(&format!(
        "object_layout_hash = \"{}\"\n",
        escape_toml_string(&report.object_layout_hash)
    ));
    out.push_str(&format!(
        "byte_layout_hash = \"{}\"\n",
        escape_toml_string(&report.byte_layout_hash)
    ));
    out.push_str(&format!("section_count = {}\n", report.section_count));
    out.push_str(&format!("total_size_bytes = {}\n", report.total_size_bytes));
    out.push_str(&format!("layout_ready = {}\n", report.layout_ready));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for section in &report.sections {
        out.push_str("\n[[byte_section]]\n");
        out.push_str(&format!("order_index = {}\n", section.order_index));
        out.push_str(&format!(
            "source_section_id = \"{}\"\n",
            escape_toml_string(&section.source_section_id)
        ));
        out.push_str(&format!(
            "object_section_name = \"{}\"\n",
            escape_toml_string(&section.object_section_name)
        ));
        out.push_str(&format!("file_offset = {}\n", section.file_offset));
        out.push_str(&format!("size_bytes = {}\n", section.size_bytes));
        out.push_str(&format!("alignment = {}\n", section.alignment));
        out.push_str(&format!(
            "source_hash = \"{}\"\n",
            escape_toml_string(&section.source_hash)
        ));
    }
    out
}

pub(crate) fn render_object_file_layout(report: &NsldObjectFileLayoutReport) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-nsld-object-file-layout-v1\"\n");
    out.push_str("schema_version = 1\n");
    out.push_str("kind = \"object-file-layout\"\n");
    out.push_str(&format!(
        "producer = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER)
    ));
    out.push_str(&format!(
        "producer_phase = \"{}\"\n",
        escape_toml_string(NSLD_LINK_INPUT_TABLE_PRODUCER_PHASE)
    ));
    out.push_str(&format!(
        "manifest = \"{}\"\n",
        escape_toml_string(&report.manifest)
    ));
    out.push_str(&format!(
        "output_path = \"{}\"\n",
        escape_toml_string(&report.output_path)
    ));
    out.push_str(&format!(
        "writer_target_id = \"{}\"\n",
        escape_toml_string(&report.writer_target_id)
    ));
    out.push_str(&format!(
        "backend_kind = \"{}\"\n",
        escape_toml_string(&report.backend_kind)
    ));
    out.push_str(&format!(
        "object_format = \"{}\"\n",
        escape_toml_string(&report.object_format)
    ));
    out.push_str(&format!(
        "object_plan_hash = \"{}\"\n",
        escape_toml_string(&report.object_plan_hash)
    ));
    out.push_str(&format!(
        "byte_layout_hash = \"{}\"\n",
        escape_toml_string(&report.byte_layout_hash)
    ));
    out.push_str(&format!(
        "file_layout_hash = \"{}\"\n",
        escape_toml_string(&report.file_layout_hash)
    ));
    out.push_str(&format!("record_count = {}\n", report.record_count));
    out.push_str(&format!(
        "total_file_size_bytes = {}\n",
        report.total_file_size_bytes
    ));
    out.push_str(&format!("layout_ready = {}\n", report.layout_ready));
    out.push_str(&format!(
        "blockers = [{}]\n",
        toml_string_array_literal(&report.blockers)
    ));
    for record in &report.records {
        out.push_str("\n[[file_layout_record]]\n");
        out.push_str(&format!("order_index = {}\n", record.order_index));
        out.push_str(&format!(
            "record_id = \"{}\"\n",
            escape_toml_string(&record.record_id)
        ));
        out.push_str(&format!(
            "record_kind = \"{}\"\n",
            escape_toml_string(&record.record_kind)
        ));
        out.push_str(&format!("file_offset = {}\n", record.file_offset));
        out.push_str(&format!("size_bytes = {}\n", record.size_bytes));
        out.push_str(&format!("alignment = {}\n", record.alignment));
    }
    out
}
