use std::path::{Path, PathBuf};

use crate::{aot, pipeline, project, registry};

pub(crate) const NUSTAR_REGISTRY_ROOT: &str = "nustar-packages";

pub(crate) struct CompiledCommandInput {
    pub(crate) resolved: pipeline::ResolvedCompileInput,
    pub(crate) artifacts: pipeline::PipelineArtifacts,
}

pub fn project_compile_workflow_brief() -> &'static str {
    "health -> structure -> scheduler -> abi_lock -> check -> test -> build -> project_metadata_inspect -> artifact_doctor -> metadata_repair -> run_artifact -> release_check"
}

pub fn nuisc_compile_pipeline_brief() -> &'static str {
    "resolve_input -> resolve_cpu_target -> compile_plan -> nir_verify -> project_link_validate -> yir_lower -> project_link_apply -> project_abi_validate -> codegen_prune -> llvm_emit -> aot_link -> project_metadata -> build_manifest -> compiled_artifact"
}

pub fn project_compile_samples_brief() -> &'static str {
    "health=nuis project-doctor <project-dir>; structure=nuis project-status <project-dir>; scheduler=nuis scheduler-view <project-dir>; abi_lock=nuis project-lock-abi <project-dir>; compile=nuis check <project-dir> -> nuis test <project-dir> -> nuis build <project-dir> <output-dir> -> nuisc inspect-project-metadata --summary <output-dir> -> nuis artifact-doctor <output-dir> -> nuisc repair-project-metadata --dry-run <output-dir> -> nuis run-artifact <output-dir> -> nuis release-check <project-dir> <output-dir>"
}

pub fn project_test_workflow_brief() -> &'static str {
    "list=nuis test --list <project-dir>; exact=nuis test --exact <project-dir> <test-name>; ignored=nuis test --ignored <project-dir>; include_ignored=nuis test --include-ignored <project-dir>"
}

pub fn project_galaxy_workflow_brief() -> &'static str {
    "galaxy=nuis galaxy init <project-dir> -> nuis galaxy check <project-dir> -> nuis galaxy lock-deps <project-dir> -> nuis galaxy sync-deps <project-dir> -> nuis project-doctor <project-dir> -> nuisc inspect-project-metadata --summary <project-dir>"
}

pub(crate) fn resolve_compile_input(
    input: &Path,
) -> Result<pipeline::ResolvedCompileInput, String> {
    pipeline::resolve_compile_input(input)
}

pub(crate) fn compile_command_input(input: &Path) -> Result<CompiledCommandInput, String> {
    let resolved = resolve_compile_input(input)?;
    let artifacts = resolved.compile()?;
    Ok(CompiledCommandInput {
        resolved,
        artifacts,
    })
}

pub(crate) fn load_nuis_executable_envelope(
    input: &Path,
) -> Result<aot::NuisExecutableEnvelope, String> {
    let bytes = std::fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    if bytes.starts_with(b"NENV") {
        aot::decode_nuis_executable_envelope_binary(&bytes)
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        let report = aot::verify_build_manifest(input)?;
        aot::parse_nuis_executable_envelope(Path::new(&report.envelope_path))
    } else {
        aot::parse_nuis_executable_envelope(input)
    }
}

pub(crate) fn load_nuis_compiled_artifact(
    input: &Path,
) -> Result<aot::NuisCompiledArtifact, String> {
    if input.is_dir() {
        let artifact_path = input.join("nuis.compiled.artifact");
        if artifact_path.is_file() {
            return aot::parse_nuis_compiled_artifact(&artifact_path);
        }
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            let report = aot::verify_build_manifest(&manifest_path)?;
            return aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path));
        }
        return Err(format!(
            "`{}` does not contain `nuis.compiled.artifact` or `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    let bytes = std::fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    if bytes.starts_with(b"NART") {
        aot::decode_nuis_compiled_artifact_binary(&bytes)
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        let report = aot::verify_build_manifest(input)?;
        aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path))
    } else {
        aot::parse_nuis_compiled_artifact(input)
    }
}

pub(crate) fn inspect_artifact_container_for_input(
    input: &Path,
    manifest_verify: Option<&aot::BuildManifestVerifyReport>,
) -> Result<Option<aot::NuisCompiledArtifactContainerInspect>, String> {
    let artifact_path = if input.is_dir() {
        let direct = input.join("nuis.compiled.artifact");
        if direct.is_file() {
            Some(direct)
        } else {
            manifest_verify.map(|report| PathBuf::from(&report.artifact_path))
        }
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        manifest_verify.map(|report| PathBuf::from(&report.artifact_path))
    } else {
        let bytes = std::fs::read(input)
            .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
        if bytes.starts_with(b"NART") {
            Some(input.to_path_buf())
        } else {
            None
        }
    };
    match artifact_path {
        Some(path) => Ok(Some(aot::inspect_nuis_compiled_artifact_container(&path)?)),
        None => Ok(None),
    }
}

pub(crate) fn success_logs_enabled() -> bool {
    std::env::var_os("NUIS_TEST_QUIET_SUCCESS_LOGS").is_none()
}

pub(crate) fn print_project_context(resolved: &pipeline::ResolvedCompileInput) {
    if let Some(project) = &resolved.project {
        eprintln!("nuisc: {}", project::describe_project(project));
    }
}

pub(crate) fn print_required_nustar_context(
    artifacts: &pipeline::PipelineArtifacts,
) -> Result<(), String> {
    let required =
        registry::load_required_manifests(Path::new(NUSTAR_REGISTRY_ROOT), &artifacts.yir)?;
    registry::validate_unit_binding(&required, &artifacts.ast.domain, &artifacts.ast.unit)?;
    eprintln!(
        "nuisc: lazily loaded nustar = {}",
        required
            .iter()
            .map(|manifest| manifest.package_id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}
