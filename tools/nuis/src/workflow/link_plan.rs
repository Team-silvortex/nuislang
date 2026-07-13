use super::*;

pub(crate) fn artifact_doctor_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis artifact-doctor {}", output_dir.display())
}

pub(crate) fn run_artifact_command_for_output_dir(output_dir: &Path) -> String {
    format!("nuis run-artifact {}", output_dir.display())
}

const NSLD_PREPARED_ARTIFACT_STAGES: &[(&str, &str)] = &[
    ("link-inputs", "nuis.nsld.link-inputs.toml"),
    ("link-units", "nuis.nsld.link-units.toml"),
    ("link-bundle", "nuis.nsld.link-bundle.toml"),
    ("assemble-plan", "nuis.nsld.assemble-plan.toml"),
    ("section-manifest", "nuis.nsld.section-manifest.toml"),
    ("object-plan", "nuis.nsld.object-plan.toml"),
    ("object-writer-input", "nuis.nsld.object-writer-input.toml"),
    ("object-byte-layout", "nuis.nsld.object-byte-layout.toml"),
    ("object-file-layout", "nuis.nsld.object-file-layout.toml"),
    (
        "object-image-dry-run",
        "nuis.nsld.object-image-dry-run.toml",
    ),
    ("object-emit-blocked", "nuis.nsld.object.blocked.toml"),
    (
        "object-writer-dry-run",
        "nuis.nsld.object-writer-dry-run.toml",
    ),
    ("container-plan", "nuis.nsld.container-plan.toml"),
    ("container", "nuis.nsld.container"),
    ("closure", "nuis.nsld.closure.toml"),
    ("final-stage-plan", "nuis.nsld.final-stage-plan.toml"),
];

const NSLD_FINAL_EXECUTABLE_TAIL_STAGES: &[(&str, &str)] = &[
    (
        "final-executable-writer-input",
        "nuis.nsld.final-executable-writer-input.toml",
    ),
    (
        "final-executable-host-invoke-plan",
        "nuis.nsld.final-executable-host-invoke-plan.toml",
    ),
    (
        "final-executable-layout",
        "nuis.nsld.final-executable-layout.toml",
    ),
    (
        "final-executable-image-dry-run",
        "nuis.nsld.final-executable-image-dry-run.toml",
    ),
    (
        "final-executable-image-dry-run-bytes",
        "nuis.nsld.final-executable-image-dry-run.bin",
    ),
    (
        "final-executable-blocked",
        "nuis.nsld.final-executable.blocked.toml",
    ),
    (
        "final-executable-launcher",
        "nuis.nsld.final-executable-launcher.toml",
    ),
    (
        "final-executable-launcher-dry-run",
        "nuis.nsld.final-executable-launcher-dry-run.toml",
    ),
    (
        "final-executable-pipeline",
        "nuis.nsld.final-executable-pipeline.toml",
    ),
];

pub(crate) struct NsldPreparedArtifactChainSummary {
    pub(crate) ready: bool,
    pub(crate) stage_count: usize,
    pub(crate) present_count: usize,
    pub(crate) next_missing_stage: Option<String>,
    pub(crate) prepare_command: String,
}

pub(crate) struct NsldFinalExecutableTailSummary {
    pub(crate) ready: bool,
    pub(crate) stage_count: usize,
    pub(crate) present_count: usize,
    pub(crate) next_missing_stage: Option<String>,
    pub(crate) pipeline_command: String,
    pub(crate) pipeline_valid: Option<bool>,
    pub(crate) final_executable_emitted: Option<bool>,
    pub(crate) launcher_manifest_ready: Option<bool>,
    pub(crate) launcher_dry_run_ready: Option<bool>,
    pub(crate) would_enter_lifecycle_hook: Option<bool>,
    pub(crate) blocker_count: Option<usize>,
    pub(crate) first_blocker: Option<String>,
    pub(crate) scheduler_metadata_payload_id: Option<String>,
    pub(crate) scheduler_metadata_present: Option<bool>,
    pub(crate) scheduler_metadata_hash: Option<String>,
    pub(crate) required_stage_path_count: Option<usize>,
    pub(crate) required_stage_path_present_count: Option<usize>,
    pub(crate) first_missing_required_stage_path: Option<String>,
}

pub(crate) fn nsld_prepare_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld prepare {}",
        output_dir.join("nuis.build.manifest.toml").display()
    )
}

pub(crate) fn nsld_prepared_artifact_chain_summary(
    output_dir: &Path,
) -> NsldPreparedArtifactChainSummary {
    let mut present_count = 0usize;
    let mut next_missing_stage = None;
    for (stage, file_name) in NSLD_PREPARED_ARTIFACT_STAGES {
        if output_dir.join(file_name).exists() {
            present_count += 1;
        } else if next_missing_stage.is_none() {
            next_missing_stage = Some((*stage).to_owned());
        }
    }
    let stage_count = NSLD_PREPARED_ARTIFACT_STAGES.len();
    NsldPreparedArtifactChainSummary {
        ready: present_count == stage_count,
        stage_count,
        present_count,
        next_missing_stage,
        prepare_command: nsld_prepare_command_for_output_dir(output_dir),
    }
}

pub(crate) fn nsld_final_executable_pipeline_command_for_output_dir(output_dir: &Path) -> String {
    format!(
        "nsld emit-final-executable-pipeline {}",
        output_dir.join("nuis.build.manifest.toml").display()
    )
}

pub(crate) fn nsld_final_executable_tail_summary(
    output_dir: &Path,
) -> NsldFinalExecutableTailSummary {
    let mut present_count = 0usize;
    let mut next_missing_stage = None;
    for (stage, file_name) in NSLD_FINAL_EXECUTABLE_TAIL_STAGES {
        if output_dir.join(file_name).exists() {
            present_count += 1;
        } else if next_missing_stage.is_none() {
            next_missing_stage = Some((*stage).to_owned());
        }
    }
    let pipeline = output_dir.join("nuis.nsld.final-executable-pipeline.toml");
    let (
        pipeline_valid,
        final_executable_emitted,
        launcher_manifest_ready,
        launcher_dry_run_ready,
        would_enter_lifecycle_hook,
        blocker_count,
        first_blocker,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_hash,
        required_stage_path_count,
        required_stage_path_present_count,
        first_missing_required_stage_path,
    ) = fs::read_to_string(&pipeline)
        .ok()
        .map(|source| {
            (
                parse_bool_field(&source, "valid"),
                parse_bool_field(&source, "final_executable_emitted"),
                parse_bool_field(&source, "launcher_manifest_ready"),
                parse_bool_field(&source, "launcher_dry_run_ready"),
                parse_bool_field(&source, "would_enter_lifecycle_hook"),
                parse_usize_field(&source, "blocker_count"),
                parse_first_string_array_item(&source, "blockers"),
                parse_string_field(&source, "scheduler_metadata_payload_id"),
                parse_bool_field(&source, "scheduler_metadata_present"),
                parse_string_field(&source, "scheduler_metadata_hash"),
                parse_usize_field(&source, "required_stage_path_count"),
                parse_usize_field(&source, "required_stage_path_present_count"),
                parse_first_string_array_item(&source, "missing_required_stage_paths"),
            )
        })
        .unwrap_or((
            None, None, None, None, None, None, None, None, None, None, None, None, None,
        ));
    let stage_count = NSLD_FINAL_EXECUTABLE_TAIL_STAGES.len();
    NsldFinalExecutableTailSummary {
        ready: present_count == stage_count && pipeline_valid == Some(true),
        stage_count,
        present_count,
        next_missing_stage,
        pipeline_command: nsld_final_executable_pipeline_command_for_output_dir(output_dir),
        pipeline_valid,
        final_executable_emitted,
        launcher_manifest_ready,
        launcher_dry_run_ready,
        would_enter_lifecycle_hook,
        blocker_count,
        first_blocker,
        scheduler_metadata_payload_id,
        scheduler_metadata_present,
        scheduler_metadata_hash,
        required_stage_path_count,
        required_stage_path_present_count,
        first_missing_required_stage_path,
    }
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
    let nsld_chain =
        link_plan.map(|plan| nsld_prepared_artifact_chain_summary(Path::new(&plan.output_dir)));
    let nsld_tail =
        link_plan.map(|plan| nsld_final_executable_tail_summary(Path::new(&plan.output_dir)));
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
        json_optional_string_field(
            "link_plan_lowering_plan_index_source",
            link_plan.map(|plan| plan.lowering_plan_index_source.as_str()),
        ),
        json_usize_field(
            "link_plan_domain_units",
            link_plan.map(|plan| plan.domain_units.len()).unwrap_or(0),
        ),
        json_object_array_field("link_plan_domain_unit_records", &domain_unit_records),
        json_optional_string_field(
            "nsld_prepare_command",
            nsld_chain
                .as_ref()
                .map(|summary| summary.prepare_command.as_str()),
        ),
        json_bool_field(
            "nsld_prepared_artifact_chain_ready",
            nsld_chain.as_ref().is_some_and(|summary| summary.ready),
        ),
        json_usize_field(
            "nsld_prepared_artifact_stage_count",
            nsld_chain
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_prepared_artifact_present_count",
            nsld_chain
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_prepared_artifact_next_missing_stage",
            nsld_chain
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_command",
            nsld_tail
                .as_ref()
                .map(|summary| summary.pipeline_command.as_str()),
        ),
        json_bool_field(
            "nsld_final_executable_tail_ready",
            nsld_tail.as_ref().is_some_and(|summary| summary.ready),
        ),
        json_usize_field(
            "nsld_final_executable_tail_stage_count",
            nsld_tail
                .as_ref()
                .map(|summary| summary.stage_count)
                .unwrap_or(0),
        ),
        json_usize_field(
            "nsld_final_executable_tail_present_count",
            nsld_tail
                .as_ref()
                .map(|summary| summary.present_count)
                .unwrap_or(0),
        ),
        json_optional_string_field(
            "nsld_final_executable_tail_next_missing_stage",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.next_missing_stage.as_deref()),
        ),
        match nsld_tail
            .as_ref()
            .and_then(|summary| summary.pipeline_valid)
        {
            Some(valid) => json_bool_field("nsld_final_executable_pipeline_valid", valid),
            None => "\"nsld_final_executable_pipeline_valid\":null".to_owned(),
        },
        json_optional_bool_field(
            "nsld_final_executable_pipeline_final_executable_emitted",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.final_executable_emitted),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_manifest_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.launcher_manifest_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_launcher_dry_run_ready",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.launcher_dry_run_ready),
        ),
        json_optional_bool_field(
            "nsld_final_executable_pipeline_would_enter_lifecycle_hook",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.would_enter_lifecycle_hook),
        ),
        match nsld_tail.as_ref().and_then(|summary| summary.blocker_count) {
            Some(count) => json_usize_field("nsld_final_executable_pipeline_blocker_count", count),
            None => "\"nsld_final_executable_pipeline_blocker_count\":null".to_owned(),
        },
        json_optional_string_field(
            "nsld_final_executable_pipeline_first_blocker",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.first_blocker.as_deref()),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_payload_id",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_payload_id.as_deref()),
        ),
        match nsld_tail
            .as_ref()
            .and_then(|summary| summary.scheduler_metadata_present)
        {
            Some(present) => json_bool_field(
                "nsld_final_executable_pipeline_scheduler_metadata_present",
                present,
            ),
            None => "\"nsld_final_executable_pipeline_scheduler_metadata_present\":null".to_owned(),
        },
        json_optional_string_field(
            "nsld_final_executable_pipeline_scheduler_metadata_hash",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.scheduler_metadata_hash.as_deref()),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_count",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.required_stage_path_count),
        ),
        json_optional_usize_field(
            "nsld_final_executable_pipeline_required_stage_path_present_count",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.required_stage_path_present_count),
        ),
        json_optional_string_field(
            "nsld_final_executable_pipeline_first_missing_required_stage_path",
            nsld_tail
                .as_ref()
                .and_then(|summary| summary.first_missing_required_stage_path.as_deref()),
        ),
    ]
}

fn json_optional_bool_field(name: &str, value: Option<bool>) -> String {
    match value {
        Some(value) => json_bool_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn json_optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => json_usize_field(name, value),
        None => format!("\"{name}\":null"),
    }
}

fn parse_bool_field(source: &str, key: &str) -> Option<bool> {
    parse_scalar_field(source, key).and_then(|value| match value.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

fn parse_usize_field(source: &str, key: &str) -> Option<usize> {
    parse_scalar_field(source, key).and_then(|value| value.trim().parse().ok())
}

fn parse_string_field(source: &str, key: &str) -> Option<String> {
    parse_scalar_field(source, key)
        .and_then(|value| value.trim().strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
}

fn parse_first_string_array_item(source: &str, key: &str) -> Option<String> {
    let value = parse_scalar_field(source, key)?;
    let value = value.trim().strip_prefix('[')?.strip_suffix(']')?.trim();
    if value.is_empty() {
        return None;
    }
    value
        .split(',')
        .next()
        .map(str::trim)
        .and_then(|item| item.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
}

fn parse_scalar_field<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    source.lines().find_map(|line| {
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim())
    })
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
