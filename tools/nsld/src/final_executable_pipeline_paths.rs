use std::path::Path;

pub(crate) struct NsldFinalExecutablePipelineRequiredPaths<'a> {
    pub(crate) final_executable_emitted: bool,
    pub(crate) final_stage_plan_path: &'a str,
    pub(crate) final_output_path: &'a str,
    pub(crate) writer_input_path: &'a str,
    pub(crate) host_invoke_plan_path: &'a str,
    pub(crate) layout_plan_path: &'a str,
    pub(crate) image_dry_run_path: &'a str,
    pub(crate) final_executable_blocked_path: &'a str,
    pub(crate) launcher_manifest_path: &'a str,
    pub(crate) launcher_dry_run_path: &'a str,
    pub(crate) entrypoint_materialization_path: Option<&'a str>,
}

pub(crate) fn final_executable_pipeline_required_stage_paths(
    input: NsldFinalExecutablePipelineRequiredPaths<'_>,
) -> Vec<String> {
    let mut paths = vec![
        input.final_stage_plan_path.to_owned(),
        input.writer_input_path.to_owned(),
        input.host_invoke_plan_path.to_owned(),
        input.layout_plan_path.to_owned(),
        input.image_dry_run_path.to_owned(),
        input.final_executable_blocked_path.to_owned(),
        input.launcher_manifest_path.to_owned(),
        input.launcher_dry_run_path.to_owned(),
    ];
    if input.final_executable_emitted {
        paths.push(input.final_output_path.to_owned());
    }
    if let Some(path) = input.entrypoint_materialization_path {
        paths.push(path.to_owned());
    }
    paths
}

pub(crate) fn missing_paths(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| !Path::new(path.as_str()).exists())
        .cloned()
        .collect()
}
