use crate::{
    json_escape_local, json_field, json_usize_field, resolve_frontdoor_build_manifest_path,
    success_logs_enabled,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn load_frontdoor_compiled_artifact(
    input: &Path,
) -> Result<nuisc::aot::NuisCompiledArtifact, String> {
    if input.is_dir() {
        let artifact_path = input.join("nuis.compiled.artifact");
        if artifact_path.is_file() {
            return nuisc::aot::parse_nuis_compiled_artifact(&artifact_path);
        }
        let manifest_path = resolve_frontdoor_build_manifest_path(input)?;
        let report = nuisc::aot::verify_build_manifest(&manifest_path)?;
        return nuisc::aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path));
    }
    let file_name = input.file_name().and_then(|value| value.to_str());
    if file_name == Some("nuis.compiled.artifact") {
        return nuisc::aot::parse_nuis_compiled_artifact(input);
    }
    if file_name == Some("nuis.build.manifest.toml") {
        let report = nuisc::aot::verify_build_manifest(input)?;
        return nuisc::aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path));
    }
    Err(format!(
        "artifact materialization expected an output directory, `nuis.compiled.artifact`, or `nuis.build.manifest.toml`; got `{}`",
        input.display()
    ))
}

fn render_artifact_materialization_json(
    kind: &str,
    input: &Path,
    output_dir: &Path,
    written_files: &[PathBuf],
) -> String {
    let files = written_files
        .iter()
        .map(|path| format!("\"{}\"", json_escape_local(&path.display().to_string())))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{}}}",
        [
            json_field("kind", kind),
            json_field("input", &input.display().to_string()),
            json_field("output_dir", &output_dir.display().to_string()),
            json_usize_field("written_files_count", written_files.len()),
            format!("\"written_files\":[{}]", files),
        ]
        .join(",")
    )
}

fn materialize_artifact_bundle(input: &Path, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let artifact = load_frontdoor_compiled_artifact(input)?;
    nuisc::aot::validate_nuis_compiled_artifact_layout(input, &artifact)?;
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
    let envelope_path = output_dir.join("nuis.executable.envelope.toml");
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    let binary_path = output_dir.join(&artifact.binary_name);
    nuisc::aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
    fs::write(&binary_path, &artifact.binary_blob)
        .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
    let relocated_manifest = nuisc::aot::render_relocated_unpacked_build_manifest(
        &artifact,
        output_dir,
        &envelope_path,
        &artifact_path,
        &binary_path,
    )?;
    let mut relocated_artifact = artifact.clone();
    relocated_artifact.build_manifest_source = relocated_manifest.clone();
    relocated_artifact.build_manifest_bytes = relocated_manifest.len();
    nuisc::aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
    fs::write(&manifest_path, relocated_manifest)
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
    let mut written = vec![envelope_path, manifest_path, artifact_path, binary_path];
    written.extend(
        nuis_artifact::materialize_embedded_artifact_support(&relocated_artifact, output_dir)
            .map_err(|error| error.to_string())?,
    );
    written.sort();
    written.dedup();
    Ok(written)
}

pub(crate) fn handle_unpack_artifact_support(
    input: PathBuf,
    output_dir: PathBuf,
    json: bool,
) -> Result<(), String> {
    let artifact = load_frontdoor_compiled_artifact(&input)?;
    nuisc::aot::validate_nuis_compiled_artifact_layout(&input, &artifact)?;
    let mut written = nuis_artifact::materialize_embedded_artifact_support(&artifact, &output_dir)
        .map_err(|error| error.to_string())?;
    written.sort();
    written.dedup();
    if json {
        println!(
            "{}",
            render_artifact_materialization_json(
                "unpack_artifact_support",
                &input,
                &output_dir,
                &written,
            )
        );
        return Ok(());
    }
    if success_logs_enabled() {
        println!("unpacked artifact support: {}", output_dir.display());
        println!("  source: {}", input.display());
        println!("  written_files: {}", written.len());
        for path in &written {
            println!("  file: {}", path.display());
        }
    }
    Ok(())
}

pub(crate) fn handle_materialize_artifact(
    input: PathBuf,
    output_dir: PathBuf,
    json: bool,
) -> Result<(), String> {
    let written = materialize_artifact_bundle(&input, &output_dir)?;
    if json {
        println!(
            "{}",
            render_artifact_materialization_json(
                "materialize_artifact",
                &input,
                &output_dir,
                &written,
            )
        );
        return Ok(());
    }
    if success_logs_enabled() {
        println!("materialized artifact: {}", output_dir.display());
        println!("  source: {}", input.display());
        println!("  written_files: {}", written.len());
        for path in &written {
            println!("  file: {}", path.display());
        }
    }
    Ok(())
}
