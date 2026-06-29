use std::path::{Path, PathBuf};

use crate::{
    aot, json_string_field, json_usize_field, load_nuis_compiled_artifact, project,
    reconstruct_manifest_report_from_artifact,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectMetadataSummary {
    pub(crate) source_kind: String,
    pub(crate) project_name: Option<String>,
    pub(crate) project_root: Option<String>,
    pub(crate) manifest_path: Option<String>,
    pub(crate) build_manifest_path: Option<String>,
    pub(crate) artifact_path: Option<String>,
    pub(crate) docs_index_path: Option<String>,
    pub(crate) docs_module_count: usize,
    pub(crate) docs_documented_module_count: usize,
    pub(crate) docs_documented_item_count: usize,
    pub(crate) imports_index_path: Option<String>,
    pub(crate) imports_library_count: usize,
    pub(crate) imports_visible_library_count: usize,
    pub(crate) imports_visible_module_count: usize,
    pub(crate) imports_documented_visible_module_count: usize,
    pub(crate) imports_documented_visible_item_count: usize,
    pub(crate) galaxy_index_path: Option<String>,
    pub(crate) galaxy_count: usize,
    pub(crate) documented_galaxy_count: usize,
    pub(crate) documented_galaxy_library_module_count: usize,
    pub(crate) documented_galaxy_item_count: usize,
}

pub(crate) fn project_metadata_summary_from_manifest_report(
    source_kind: &str,
    manifest_path: Option<&Path>,
    artifact_path: Option<&Path>,
    report: &aot::BuildManifestVerifyReport,
) -> ProjectMetadataSummary {
    ProjectMetadataSummary {
        source_kind: source_kind.to_owned(),
        project_name: None,
        project_root: Path::new(&report.input)
            .parent()
            .map(|path| path.display().to_string()),
        manifest_path: manifest_path.map(|path| path.display().to_string()),
        build_manifest_path: manifest_path.map(|path| path.display().to_string()),
        artifact_path: artifact_path
            .map(|path| path.display().to_string())
            .or_else(|| {
                if report.artifact_path.is_empty() {
                    None
                } else {
                    Some(report.artifact_path.clone())
                }
            }),
        docs_index_path: report.project_docs_index.clone(),
        docs_module_count: report.project_docs_module_count,
        docs_documented_module_count: report.project_docs_documented_module_count,
        docs_documented_item_count: report.project_docs_documented_item_count,
        imports_index_path: report.project_imports_index.clone(),
        imports_library_count: report.project_imports_library_count,
        imports_visible_library_count: report.project_imports_visible_library_count,
        imports_visible_module_count: report.project_imports_visible_module_count,
        imports_documented_visible_module_count: report
            .project_imports_documented_visible_module_count,
        imports_documented_visible_item_count: report.project_imports_documented_visible_item_count,
        galaxy_index_path: report.project_galaxy_index.clone(),
        galaxy_count: report.project_galaxy_count,
        documented_galaxy_count: report.project_documented_galaxy_count,
        documented_galaxy_library_module_count: report
            .project_documented_galaxy_library_module_count,
        documented_galaxy_item_count: report.project_documented_galaxy_item_count,
    }
}

pub(crate) fn inspect_project_metadata_from_source(
    input: &Path,
) -> Result<ProjectMetadataSummary, String> {
    let loaded_project = project::load_project(input)?;
    let docs_summary = project::project_docs_summary(&loaded_project);
    let imports_summary = project::project_imports_summary(&loaded_project);
    let galaxy_summary = project::project_galaxy_summary(&loaded_project);
    Ok(ProjectMetadataSummary {
        source_kind: "project-source".to_owned(),
        project_name: Some(loaded_project.manifest.name.clone()),
        project_root: Some(loaded_project.root.display().to_string()),
        manifest_path: Some(loaded_project.manifest_path.display().to_string()),
        build_manifest_path: None,
        artifact_path: None,
        docs_index_path: None,
        docs_module_count: docs_summary.modules,
        docs_documented_module_count: docs_summary.documented_modules,
        docs_documented_item_count: docs_summary.documented_items,
        imports_index_path: None,
        imports_library_count: imports_summary.libraries,
        imports_visible_library_count: imports_summary.visible_libraries,
        imports_visible_module_count: imports_summary.visible_modules,
        imports_documented_visible_module_count: imports_summary.documented_visible_modules,
        imports_documented_visible_item_count: imports_summary.documented_visible_items,
        galaxy_index_path: None,
        galaxy_count: galaxy_summary.galaxies,
        documented_galaxy_count: galaxy_summary.documented_galaxies,
        documented_galaxy_library_module_count: galaxy_summary.documented_library_modules,
        documented_galaxy_item_count: galaxy_summary.documented_items,
    })
}

pub(crate) fn inspect_project_metadata(input: &Path) -> Result<ProjectMetadataSummary, String> {
    if input.is_dir() {
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            let report = aot::verify_build_manifest(&manifest_path)?;
            let artifact_path = input.join("nuis.compiled.artifact");
            return Ok(project_metadata_summary_from_manifest_report(
                "build-output-dir",
                Some(&manifest_path),
                artifact_path.is_file().then_some(artifact_path.as_path()),
                &report,
            ));
        }
    }
    let is_manifest = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let is_artifact = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.compiled.artifact")
        .unwrap_or(false);
    if is_manifest {
        let report = aot::verify_build_manifest(input)?;
        return Ok(project_metadata_summary_from_manifest_report(
            "build-manifest",
            Some(input),
            None,
            &report,
        ));
    }
    if is_artifact {
        let artifact = load_nuis_compiled_artifact(input)?;
        let (manifest_path, report) = reconstruct_manifest_report_from_artifact(input, &artifact)?;
        return Ok(project_metadata_summary_from_manifest_report(
            "compiled-artifact",
            Some(&manifest_path),
            Some(input),
            &report,
        ));
    }
    inspect_project_metadata_from_source(input)
}

pub(crate) fn resolve_build_manifest_path(input: &Path) -> Result<PathBuf, String> {
    if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        return Ok(input.to_path_buf());
    }
    if input.is_dir() {
        let manifest_path = input.join("nuis.build.manifest.toml");
        if manifest_path.is_file() {
            return Ok(manifest_path);
        }
        return Err(format!(
            "`{}` does not contain `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    Err(format!(
        "expected a build manifest path or output directory, got `{}`",
        input.display()
    ))
}

pub(crate) fn resolve_artifact_report_inputs(
    input: &Path,
) -> Result<
    (
        PathBuf,
        aot::NuisCompiledArtifact,
        PathBuf,
        aot::BuildManifestVerifyReport,
        bool,
    ),
    String,
> {
    let is_manifest_input = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let is_output_dir_input = input.is_dir() && input.join("nuis.build.manifest.toml").is_file();
    let artifact = load_nuis_compiled_artifact(input)?;
    if is_manifest_input || is_output_dir_input {
        let manifest_input = if is_manifest_input {
            input.to_path_buf()
        } else {
            input.join("nuis.build.manifest.toml")
        };
        let report = aot::verify_build_manifest(&manifest_input)?;
        return Ok((
            manifest_input,
            artifact,
            PathBuf::from(&report.artifact_path),
            report,
            false,
        ));
    }
    let (manifest_input, manifest_verify) =
        reconstruct_manifest_report_from_artifact(input, &artifact)?;
    Ok((
        manifest_input,
        artifact,
        input.to_path_buf(),
        manifest_verify,
        true,
    ))
}

pub(crate) fn repair_project_metadata_target(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if input.is_dir() {
        let manifest_path = resolve_build_manifest_path(input)?;
        let report = aot::verify_build_manifest(&manifest_path)?;
        let project_input = PathBuf::from(&report.input);
        if !project_input.exists() {
            return Err(format!(
                "cannot repair project metadata from `{}` because the original compile input `{}` no longer exists; try `nuisc inspect-project-metadata \"{}\"` or `nuisc verify-build-manifest \"{}\"` instead",
                input.display(),
                project_input.display(),
                input.display(),
                input.display()
            ));
        }
        return Ok((project_input, PathBuf::from(report.output_dir)));
    }
    let is_manifest = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let is_artifact = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.compiled.artifact")
        .unwrap_or(false);
    if is_manifest {
        let report = aot::verify_build_manifest(input)?;
        let project_input = PathBuf::from(&report.input);
        if !project_input.exists() {
            return Err(format!(
                "cannot repair project metadata from `{}` because the original compile input `{}` no longer exists; try `nuisc inspect-project-metadata \"{}\"` or `nuisc verify-build-manifest \"{}\"` instead",
                input.display(),
                project_input.display(),
                input.display(),
                input.display()
            ));
        }
        return Ok((project_input, PathBuf::from(report.output_dir)));
    }
    if is_artifact {
        let artifact = load_nuis_compiled_artifact(input)?;
        let (_manifest_path, report) = reconstruct_manifest_report_from_artifact(input, &artifact)?;
        let project_input = PathBuf::from(&report.input);
        if !project_input.exists() {
            return Err(format!(
                "cannot repair project metadata from `{}` because the original compile input `{}` no longer exists; try `nuisc inspect-project-metadata \"{}\"` instead",
                input.display(),
                project_input.display(),
                input.display()
            ));
        }
        return Ok((project_input, PathBuf::from(report.output_dir)));
    }
    Err(
        "usage: nuisc repair-project-metadata [--dry-run] <output-dir|nuis.build.manifest.toml|nuis.compiled.artifact>"
            .to_owned(),
    )
}

pub(crate) fn inspect_project_metadata_json(summary: &ProjectMetadataSummary) -> String {
    let mut fields = vec![
        json_string_field("kind", "nuis_project_metadata"),
        json_string_field("source_kind", &summary.source_kind),
        json_usize_field("docs_module_count", summary.docs_module_count),
        json_usize_field(
            "docs_documented_module_count",
            summary.docs_documented_module_count,
        ),
        json_usize_field(
            "docs_documented_item_count",
            summary.docs_documented_item_count,
        ),
        json_usize_field("imports_library_count", summary.imports_library_count),
        json_usize_field(
            "imports_visible_library_count",
            summary.imports_visible_library_count,
        ),
        json_usize_field(
            "imports_visible_module_count",
            summary.imports_visible_module_count,
        ),
        json_usize_field(
            "imports_documented_visible_module_count",
            summary.imports_documented_visible_module_count,
        ),
        json_usize_field(
            "imports_documented_visible_item_count",
            summary.imports_documented_visible_item_count,
        ),
        json_usize_field("galaxy_count", summary.galaxy_count),
        json_usize_field("documented_galaxy_count", summary.documented_galaxy_count),
        json_usize_field(
            "documented_galaxy_library_module_count",
            summary.documented_galaxy_library_module_count,
        ),
        json_usize_field(
            "documented_galaxy_item_count",
            summary.documented_galaxy_item_count,
        ),
    ];
    if let Some(value) = &summary.project_name {
        fields.push(json_string_field("project_name", value));
    }
    if let Some(value) = &summary.project_root {
        fields.push(json_string_field("project_root", value));
    }
    if let Some(value) = &summary.manifest_path {
        fields.push(json_string_field("manifest_path", value));
    }
    if let Some(value) = &summary.build_manifest_path {
        fields.push(json_string_field("build_manifest_path", value));
    }
    if let Some(value) = &summary.artifact_path {
        fields.push(json_string_field("artifact_path", value));
    }
    if let Some(value) = &summary.docs_index_path {
        fields.push(json_string_field("docs_index_path", value));
    }
    if let Some(value) = &summary.imports_index_path {
        fields.push(json_string_field("imports_index_path", value));
    }
    if let Some(value) = &summary.galaxy_index_path {
        fields.push(json_string_field("galaxy_index_path", value));
    }
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn render_project_metadata_summary(summary: &ProjectMetadataSummary) -> String {
    let mut lines = vec!["project metadata".to_owned()];
    lines.push(format!("  source_kind: {}", summary.source_kind));
    if let Some(value) = &summary.project_name {
        lines.push(format!("  project_name: {}", value));
    }
    if let Some(value) = &summary.project_root {
        lines.push(format!("  project_root: {}", value));
    }
    if let Some(value) = &summary.manifest_path {
        lines.push(format!("  manifest_path: {}", value));
    }
    if let Some(value) = &summary.build_manifest_path {
        lines.push(format!("  build_manifest_path: {}", value));
    }
    if let Some(value) = &summary.artifact_path {
        lines.push(format!("  artifact_path: {}", value));
    }
    lines.push(format!(
        "  docs: modules={} documented_modules={} documented_items={}",
        summary.docs_module_count,
        summary.docs_documented_module_count,
        summary.docs_documented_item_count
    ));
    if let Some(value) = &summary.docs_index_path {
        lines.push(format!("  docs_index_path: {}", value));
    }
    lines.push(format!(
        "  imports: libraries={} visible_libraries={} visible_modules={} documented_visible_modules={} documented_visible_items={}",
        summary.imports_library_count,
        summary.imports_visible_library_count,
        summary.imports_visible_module_count,
        summary.imports_documented_visible_module_count,
        summary.imports_documented_visible_item_count
    ));
    if let Some(value) = &summary.imports_index_path {
        lines.push(format!("  imports_index_path: {}", value));
    }
    lines.push(format!(
        "  galaxies: total={} documented={} documented_library_modules={} documented_items={}",
        summary.galaxy_count,
        summary.documented_galaxy_count,
        summary.documented_galaxy_library_module_count,
        summary.documented_galaxy_item_count
    ));
    if let Some(value) = &summary.galaxy_index_path {
        lines.push(format!("  galaxy_index_path: {}", value));
    }
    lines.join("\n")
}

pub(crate) fn render_project_metadata_compact_summary(summary: &ProjectMetadataSummary) -> String {
    format!(
        "project metadata summary: source_kind={} project={} docs={}/{}/{} imports={}/{}/{}/{}/{} galaxies={}/{}/{}/{}",
        summary.source_kind,
        summary.project_name.as_deref().unwrap_or("<none>"),
        summary.docs_module_count,
        summary.docs_documented_module_count,
        summary.docs_documented_item_count,
        summary.imports_library_count,
        summary.imports_visible_library_count,
        summary.imports_visible_module_count,
        summary.imports_documented_visible_module_count,
        summary.imports_documented_visible_item_count,
        summary.galaxy_count,
        summary.documented_galaxy_count,
        summary.documented_galaxy_library_module_count,
        summary.documented_galaxy_item_count
    )
}

pub(crate) fn render_project_metadata_paths(summary: &ProjectMetadataSummary) -> String {
    let mut lines = Vec::new();
    if let Some(value) = &summary.project_root {
        lines.push(format!("project_root={}", value));
    }
    if let Some(value) = &summary.manifest_path {
        lines.push(format!("manifest_path={}", value));
    }
    if let Some(value) = &summary.build_manifest_path {
        lines.push(format!("build_manifest_path={}", value));
    }
    if let Some(value) = &summary.artifact_path {
        lines.push(format!("artifact_path={}", value));
    }
    if let Some(value) = &summary.docs_index_path {
        lines.push(format!("docs_index_path={}", value));
    }
    if let Some(value) = &summary.imports_index_path {
        lines.push(format!("imports_index_path={}", value));
    }
    if let Some(value) = &summary.galaxy_index_path {
        lines.push(format!("galaxy_index_path={}", value));
    }
    lines.join("\n")
}
