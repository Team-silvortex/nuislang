use std::{fs, path::Path};

use nuis_artifact::{NuisCompiledArtifact, NuisExecutableEnvelope, NuisLifecycleContract};

use crate::aot_manifest_types::{BuildManifestContext, CompileArtifacts};

pub(crate) fn build_nuis_compiled_artifact(
    written: &CompileArtifacts,
    context: &BuildManifestContext,
    envelope: &NuisExecutableEnvelope,
    lifecycle: &NuisLifecycleContract,
    build_manifest_source: &str,
) -> Result<NuisCompiledArtifact, String> {
    let binary_blob = fs::read(&written.binary_path).map_err(|error| {
        format!(
            "failed to read compiled binary `{}`: {error}",
            written.binary_path
        )
    })?;
    let binary_name = Path::new(&written.binary_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-binary")
        .to_owned();
    Ok(NuisCompiledArtifact {
        schema: "nuis-compiled-artifact-v1".to_owned(),
        packaging_mode: written.packaging_mode.clone(),
        cpu_target_abi: context.cpu_target.abi.clone(),
        cpu_target_machine_arch: context.cpu_target.machine_arch.clone(),
        cpu_target_machine_os: context.cpu_target.machine_os.clone(),
        cpu_target_object_format: context.cpu_target.object_format.clone(),
        cpu_target_calling_abi: context.cpu_target.calling_abi.clone(),
        binary_name,
        binary_bytes: binary_blob.len(),
        build_manifest_bytes: build_manifest_source.len(),
        envelope: envelope.clone(),
        lifecycle: lifecycle.clone(),
        build_manifest_source: build_manifest_source.to_owned(),
        binary_blob,
    })
}
