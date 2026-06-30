use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::aot_artifact::{write_nuis_compiled_artifact, write_nuis_executable_envelope};
use crate::aot_compiled_artifact_builder::build_nuis_compiled_artifact;
use crate::aot_lifecycle::build_nuis_lifecycle_contract;
use crate::aot_manifest_artifacts::prepare_build_manifest_artifacts;
use crate::aot_manifest_domain_model::{
    build_manifest_domain_units, build_nuis_envelope, resolve_execution_contracts,
};
use crate::aot_manifest_render::{render_build_manifest_source, BuildManifestRenderInput};
use crate::aot_manifest_types::{BuildManifestContext, CompileArtifacts};
use crate::aot_vcs_info::detect_vcs_info;

pub fn write_build_manifest(
    output_dir: &Path,
    written: &CompileArtifacts,
    context: &BuildManifestContext,
) -> Result<String, String> {
    let path = output_dir.join("nuis.build.manifest.toml");
    let envelope_path = output_dir.join("nuis.executable.envelope.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    let generated_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_secs();
    let engine = crate::engine::default_engine();
    let vcs = detect_vcs_info(&context.input_path, &context.output_dir);

    let mut loaded_nustar = context.loaded_nustar.clone();
    loaded_nustar.sort();
    loaded_nustar.dedup();
    let execution_contracts = resolve_execution_contracts(&loaded_nustar)?;
    let mut domain_build_units = build_manifest_domain_units(context, &execution_contracts)?;
    let envelope = build_nuis_envelope(&execution_contracts, &written.packaging_mode);
    let lifecycle = build_nuis_lifecycle_contract(&envelope, &written.packaging_mode);
    let compiled_binary_name = Path::new(&written.binary_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-binary")
        .to_owned();
    let compiled_binary_bytes = fs::metadata(&written.binary_path)
        .map_err(|error| format!("failed to stat `{}`: {error}", written.binary_path))?
        .len() as usize;

    let artifact_set =
        prepare_build_manifest_artifacts(output_dir, written, &lifecycle, &mut domain_build_units)?;

    write_nuis_executable_envelope(&envelope_path, &envelope)?;
    let out = render_build_manifest_source(
        context,
        BuildManifestRenderInput {
            generated_at_unix,
            packaging_mode: &written.packaging_mode,
            engine_version: engine.version,
            engine_profile: engine.profile,
            vcs: &vcs,
            loaded_nustar: &loaded_nustar,
            envelope_path: &envelope_path,
            envelope: &envelope,
            artifact_path: &artifact_path,
            artifact_binary_name: &compiled_binary_name,
            artifact_binary_bytes: compiled_binary_bytes,
            lifecycle: &lifecycle,
            artifact_set: &artifact_set,
            execution_contracts: &execution_contracts,
            domain_build_units: &domain_build_units,
            project: context.project.as_ref(),
        },
    )?;

    let compiled_artifact =
        build_nuis_compiled_artifact(written, context, &envelope, &lifecycle, &out)?;
    write_nuis_compiled_artifact(&artifact_path, &compiled_artifact)?;
    fs::write(&path, out)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(path.display().to_string())
}
