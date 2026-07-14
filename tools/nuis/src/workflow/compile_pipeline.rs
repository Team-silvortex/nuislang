use super::*;
use std::path::Path;

fn compile_pipeline_stage_json(stage: &nuisc::pipeline::CompilePipelineStage) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("id", stage.id),
            json_field("status", stage.status),
            json_field("detail", &stage.detail),
        ],
    );
    out.push('}');
    out
}

pub(crate) fn workflow_compile_pipeline_json_fields(input: &Path) -> Vec<String> {
    match nuisc::pipeline::resolve_compile_input(input).and_then(|resolved| {
        let artifacts = resolved.compile()?;
        Ok(resolved.compile_report(&artifacts))
    }) {
        Ok(report) => {
            let stage_records = report
                .stages
                .iter()
                .map(compile_pipeline_stage_json)
                .collect::<Vec<_>>();
            vec![
                json_bool_field("compile_pipeline_available", true),
                json_field("compile_pipeline_source_kind", report.source_kind),
                json_field("compile_pipeline_input", &report.input_path),
                json_field(
                    "compile_pipeline_effective_input",
                    &report.effective_input_path,
                ),
                json_optional_string_field(
                    "compile_pipeline_project",
                    report.project_name.as_deref(),
                ),
                json_field("compile_pipeline_domain", &report.domain),
                json_field("compile_pipeline_unit", &report.unit),
                json_usize_field("compile_pipeline_stage_count", report.stage_count()),
                json_usize_field("compile_pipeline_ok_stage_count", report.ok_stage_count()),
                json_usize_field("compile_pipeline_ast_functions", report.ast_functions),
                json_usize_field("compile_pipeline_nir_functions", report.nir_functions),
                json_usize_field("compile_pipeline_yir_nodes", report.yir_nodes),
                json_usize_field("compile_pipeline_yir_resources", report.yir_resources),
                json_usize_field("compile_pipeline_yir_edges", report.yir_edges),
                json_usize_field("compile_pipeline_llvm_ir_bytes", report.llvm_ir_bytes),
                json_usize_field(
                    "compile_pipeline_loaded_nustar_count",
                    report.loaded_nustar.len(),
                ),
                json_string_array_field("compile_pipeline_loaded_nustar", &report.loaded_nustar),
                json_object_array_field("compile_pipeline_stages", &stage_records),
                json_bool_field("compile_pipeline_ready_for_aot", report.ready_for_aot),
                json_field(
                    "compile_pipeline_recommended_next_step",
                    report.recommended_next_step,
                ),
                json_field(
                    "compile_pipeline_recommended_reason",
                    &report.recommended_reason,
                ),
                json_field("compile_pipeline_summary", &report.summary_line()),
            ]
        }
        Err(error) => vec![
            json_bool_field("compile_pipeline_available", false),
            json_field("compile_pipeline_error", &error),
        ],
    }
}
