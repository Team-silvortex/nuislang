use super::*;

pub(crate) fn artifact_doctor_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis artifact-doctor {}", output_dir.display())
}

pub(crate) fn run_artifact_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis run-artifact {}", output_dir.display())
}

pub(crate) fn load_link_plan_for_output_dir(output_dir: &Path) -> Option<nuisc::linker::LinkPlan> {
    let manifest = output_dir.join("nuis.build.manifest.toml");
    if !manifest.exists() {
        return None;
    }
    nuisc::linker::build_link_plan_from_manifest(&manifest).ok()
}

fn workflow_link_plan_domain_unit_record(unit: &nuisc::linker::LinkPlanDomainUnit) -> String {
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", &unit.kind),
            json_field("package_id", &unit.package_id),
            json_field("domain_family", &unit.domain_family),
            json_field("contract_family", &unit.contract_family),
            json_field("packaging_role", &unit.packaging_role),
        ],
    );
    if let Some(value) = unit.abi.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("abi", value)]);
    }
    if let Some(value) = unit.backend_family.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("backend_family", value)]);
    }
    if let Some(value) = unit.selected_lowering_target.as_deref() {
        append_json_field_strings(
            &mut out,
            vec![json_field("selected_lowering_target", value)],
        );
    }
    if let Some(value) = unit.machine_arch.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_arch", value)]);
    }
    if let Some(value) = unit.machine_os.as_deref() {
        append_json_field_strings(&mut out, vec![json_field("machine_os", value)]);
    }
    out.push('}');
    out
}

fn workflow_link_plan_json_fields(link_plan: Option<&nuisc::linker::LinkPlan>) -> Vec<String> {
    let domain_unit_records = link_plan
        .map(|plan| {
            plan.domain_units
                .iter()
                .map(workflow_link_plan_domain_unit_record)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    vec![
        json_bool_field("link_plan_available", link_plan.is_some()),
        json_optional_string_field(
            "link_plan_final_stage",
            link_plan.map(|plan| plan.final_stage.kind.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_driver",
            link_plan.map(|plan| plan.final_stage.driver.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_link_mode",
            link_plan.map(|plan| plan.final_stage.link_mode.as_str()),
        ),
        json_optional_string_field(
            "link_plan_final_output",
            link_plan.map(|plan| plan.final_stage.output_path.as_str()),
        ),
        json_optional_string_field(
            "link_plan_lowering_plan_index_path",
            link_plan.and_then(|plan| plan.lowering_plan_index_path.as_deref()),
        ),
        json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
        json_object_array_field("link_plan_domain_unit_records", &domain_unit_records),
    ]
}

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

pub(super) fn workflow_compile_pipeline_json_fields(input: &Path) -> Vec<String> {
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

pub(crate) fn append_workflow_link_plan_json_fields(
    out: &mut String,
    link_plan: Option<&nuisc::linker::LinkPlan>,
) {
    append_json_field_strings(out, workflow_link_plan_json_fields(link_plan));
}
