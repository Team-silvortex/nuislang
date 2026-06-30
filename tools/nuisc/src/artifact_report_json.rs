use super::*;

pub(crate) fn artifact_report_json(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
    artifact_verify_input: &Path,
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    manifest_input: &Path,
    manifest_verify: &aot::BuildManifestVerifyReport,
    manifest_verify_reconstructed: bool,
) -> String {
    let verdicts = collect_domain_build_unit_verdicts(manifest_verify);
    let summary = summarize_domain_build_verification(&verdicts);
    let link_plan = linker::build_link_plan(manifest_verify, artifact);
    let doc_indexes =
        collect_doc_indexes_from_manifest_input(manifest_verify).unwrap_or_else(|_| Vec::new());
    let execution_inspect = inspect_execution_json(manifest_input).unwrap_or_else(|error| {
        format!(
            "{{{},{},{}}}",
            json_string_field("kind", "nuis_execution_inspect_error"),
            json_string_field("input", &manifest_input.display().to_string()),
            json_string_field("error", &error)
        )
    });
    let project_metadata =
        inspect_project_metadata_json(&project_metadata_summary_from_manifest_report(
            "build-manifest",
            Some(manifest_input),
            Some(artifact_verify_input),
            manifest_verify,
        ));
    let artifact_container =
        aot::inspect_nuis_compiled_artifact_container(artifact_verify_input).ok();
    let fields = vec![
        json_string_field("kind", "nuis_artifact_report"),
        json_string_field("input", &input.display().to_string()),
        json_bool_field(
            "manifest_verify_reconstructed",
            manifest_verify_reconstructed,
        ),
        format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ),
        format!(
            "\"artifact_inspect\":{}",
            inspect_artifact_json(
                input,
                artifact,
                artifact_container.as_ref(),
                Some(manifest_verify),
            )
        ),
        format!(
            "\"artifact_verify\":{}",
            verify_artifact_json(artifact_verify_input, artifact_verify)
        ),
        format!(
            "\"manifest_verify\":{}",
            verify_build_manifest_json(manifest_input, manifest_verify)
        ),
        format!("\"project_metadata\":{}", project_metadata),
        format!(
            "\"doc_index\":{}",
            inspect_docs_json(Path::new(&manifest_verify.input), &doc_indexes)
        ),
        format!("\"execution_inspect\":{}", execution_inspect),
        format!("\"link_plan\":{}", link_plan_json(&link_plan)),
    ];
    format!("{{{}}}", fields.join(","))
}
