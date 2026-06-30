use super::*;

pub(crate) fn reconstruct_manifest_report_from_artifact(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
) -> Result<(PathBuf, aot::BuildManifestVerifyReport), String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nuis_artifact_report_{nonce}"));
    std::fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create `{}`: {error}", temp_root.display()))?;

    let manifest_path = temp_root.join("nuis.build.manifest.toml");
    let envelope_path = temp_root.join("nuis.executable.envelope.toml");
    let artifact_path = temp_root.join("nuis.compiled.artifact");
    let binary_path = temp_root.join(&artifact.binary_name);

    let result = (|| {
        std::fs::write(&binary_path, &artifact.binary_blob)
            .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
        aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
        let relocated_manifest = aot::render_relocated_unpacked_build_manifest(
            artifact,
            &temp_root,
            &envelope_path,
            &artifact_path,
            &binary_path,
        )?;
        let mut relocated_artifact = artifact.clone();
        relocated_artifact.build_manifest_source = relocated_manifest.clone();
        relocated_artifact.build_manifest_bytes = relocated_manifest.len();
        aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
        std::fs::write(&manifest_path, relocated_manifest)
            .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
        let report = aot::verify_build_manifest(&manifest_path)?;
        Ok((manifest_path.clone(), report))
    })();

    let _ = std::fs::remove_dir_all(&temp_root);
    result.map_err(|error: String| {
        format!(
            "failed to reconstruct build manifest context for `{}`: {error}",
            input.display()
        )
    })
}
