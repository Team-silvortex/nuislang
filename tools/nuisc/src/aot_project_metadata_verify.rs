use std::{fs, path::Path};

pub(crate) struct ProjectMetadataVerifyReport {
    pub doc_index_checked: usize,
    pub project_metadata_checked: usize,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn verify_project_metadata_artifacts(
    manifest_path: &Path,
    input: &str,
    output_dir: &str,
    doc_index_path: Option<&str>,
    doc_index_module_count: usize,
    doc_index_documented_item_count: usize,
    project_plan_index: Option<&str>,
    project_plan_summary: Option<&str>,
    project_docs_index: Option<&str>,
    project_docs_module_count: usize,
    project_docs_documented_module_count: usize,
    project_docs_documented_item_count: usize,
    project_imports_index: Option<&str>,
    project_imports_library_count: usize,
    project_imports_visible_library_count: usize,
    project_imports_visible_module_count: usize,
    project_imports_documented_visible_module_count: usize,
    project_imports_documented_visible_item_count: usize,
    project_galaxy_index: Option<&str>,
    project_galaxy_count: usize,
    project_documented_galaxy_count: usize,
    project_documented_galaxy_library_module_count: usize,
    project_documented_galaxy_item_count: usize,
    project_packet_index: Option<&str>,
) -> Result<ProjectMetadataVerifyReport, String> {
    let doc_index_checked = verify_doc_index(
        manifest_path,
        doc_index_path,
        doc_index_module_count,
        doc_index_documented_item_count,
    )?;
    let mut project_metadata_checked = 0usize;
    if let Some(plan_index) = project_plan_index {
        let plan_source = read_project_index("plan", plan_index, manifest_path)?;
        if let Some(summary) = project_plan_summary {
            let expected = expected_project_plan_summary(summary);
            if !plan_source.lines().any(|line| line.trim() == expected) {
                return Err(project_metadata_summary_mismatch_error(
                    "plan",
                    plan_index,
                    &expected,
                    &plan_source,
                    input,
                    output_dir,
                ));
            }
        }
        project_metadata_checked += 1;
    }
    if let Some(docs_index) = project_docs_index {
        let docs_source = read_project_index("docs", docs_index, manifest_path)?;
        let expected = expected_project_docs_summary(
            project_docs_module_count,
            project_docs_documented_module_count,
            project_docs_documented_item_count,
        );
        if !docs_source.lines().any(|line| line.trim() == expected) {
            return Err(project_metadata_summary_mismatch_error(
                "docs",
                docs_index,
                &expected,
                &docs_source,
                input,
                output_dir,
            ));
        }
        project_metadata_checked += 1;
    }
    if let Some(imports_index) = project_imports_index {
        let imports_source = read_project_index("imports", imports_index, manifest_path)?;
        let expected = expected_project_imports_summary(
            project_imports_library_count,
            project_imports_visible_library_count,
            project_imports_visible_module_count,
            project_imports_documented_visible_module_count,
            project_imports_documented_visible_item_count,
        );
        if !imports_source.lines().any(|line| line.trim() == expected) {
            return Err(project_metadata_summary_mismatch_error(
                "imports",
                imports_index,
                &expected,
                &imports_source,
                input,
                output_dir,
            ));
        }
        project_metadata_checked += 1;
    }
    if let Some(galaxy_index) = project_galaxy_index {
        let galaxy_source = read_project_index("galaxy", galaxy_index, manifest_path)?;
        let expected = expected_project_galaxy_summary(
            project_galaxy_count,
            project_documented_galaxy_count,
            project_documented_galaxy_library_module_count,
            project_documented_galaxy_item_count,
        );
        if !galaxy_source.lines().any(|line| line.trim() == expected) {
            return Err(project_metadata_summary_mismatch_error(
                "galaxy",
                galaxy_index,
                &expected,
                &galaxy_source,
                input,
                output_dir,
            ));
        }
        project_metadata_checked += 1;
    }
    if let Some(packet_index) = project_packet_index {
        read_project_index("packet", packet_index, manifest_path)?;
        project_metadata_checked += 1;
    }
    Ok(ProjectMetadataVerifyReport {
        doc_index_checked,
        project_metadata_checked,
    })
}

fn verify_doc_index(
    manifest_path: &Path,
    doc_index_path: Option<&str>,
    doc_index_module_count: usize,
    doc_index_documented_item_count: usize,
) -> Result<usize, String> {
    let Some(doc_index_path) = doc_index_path else {
        return Ok(0);
    };
    let doc_index_source = fs::read_to_string(doc_index_path).map_err(|error| {
        format!(
            "failed to read doc index `{}` referenced by `{}`: {error}",
            doc_index_path,
            manifest_path.display()
        )
    })?;
    if !doc_index_source.contains("\"kind\":\"nuis_doc_index\"") {
        return Err(format!(
            "doc index `{}` has unexpected kind; expected `nuis_doc_index`",
            doc_index_path
        ));
    }
    if !doc_index_source.contains(&format!("\"module_count\":{}", doc_index_module_count)) {
        return Err(format!(
            "doc index `{}` module_count mismatch: manifest={}, index payload differs",
            doc_index_path, doc_index_module_count
        ));
    }
    if !doc_index_source.contains(&format!(
        "\"documented_item_count\":{}",
        doc_index_documented_item_count
    )) {
        return Err(format!(
            "doc index `{}` documented_item_count mismatch: manifest={}, index payload differs",
            doc_index_path, doc_index_documented_item_count
        ));
    }
    Ok(1)
}

fn read_project_index(
    index_kind: &str,
    index_path: &str,
    manifest_path: &Path,
) -> Result<String, String> {
    fs::read_to_string(index_path).map_err(|error| {
        format!(
            "failed to read project {index_kind} index `{}` referenced by `{}`: {error}",
            index_path,
            manifest_path.display()
        )
    })
}

pub(crate) fn project_metadata_summary_mismatch_error(
    index_kind: &str,
    index_path: &str,
    expected: &str,
    source: &str,
    project_input: &str,
    output_dir: &str,
) -> String {
    let actual = source
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("summary\t"))
        .unwrap_or("<missing summary line>");
    let source_exists = Path::new(project_input).exists();
    let build_manifest_path = Path::new(output_dir)
        .join("nuis.build.manifest.toml")
        .display()
        .to_string();
    let suggestions = if source_exists {
        vec![
            format!("nuisc compile \"{}\" \"{}\"", project_input, output_dir),
            format!("nuisc inspect-project-metadata \"{}\"", project_input),
        ]
    } else {
        vec![
            format!("nuisc inspect-project-metadata \"{}\"", build_manifest_path),
            format!("nuisc verify-build-manifest \"{}\"", build_manifest_path),
        ]
    };
    format!(
        "project {index_kind} index `{index_path}` summary mismatch: expected `{expected}`, found `{actual}`; this usually means the build artifact was produced by an older nuisc metadata format or the index file drifted after compilation. Rebuild the project with the current nuisc, or regenerate the build output before inspecting/verifying it. Suggested commands: {}.",
        suggestions
            .iter()
            .map(|command| format!("`{command}`"))
            .collect::<Vec<_>>()
            .join(" or ")
    )
}

pub(crate) fn expected_project_plan_summary(summary: &str) -> String {
    format!("summary {summary}")
}

pub(crate) fn expected_project_docs_summary(
    module_count: usize,
    documented_module_count: usize,
    documented_item_count: usize,
) -> String {
    format!(
        "summary\tmodules={module_count}\tdocumented_modules={documented_module_count}\tdocumented_items={documented_item_count}"
    )
}

pub(crate) fn expected_project_imports_summary(
    library_count: usize,
    visible_library_count: usize,
    visible_module_count: usize,
    documented_visible_module_count: usize,
    documented_visible_item_count: usize,
) -> String {
    format!(
        "summary\tlibraries={library_count}\tvisible_libraries={visible_library_count}\tvisible_modules={visible_module_count}\tdocumented_visible_modules={documented_visible_module_count}\tdocumented_visible_items={documented_visible_item_count}"
    )
}

pub(crate) fn expected_project_galaxy_summary(
    galaxy_count: usize,
    documented_galaxy_count: usize,
    documented_library_module_count: usize,
    documented_item_count: usize,
) -> String {
    format!(
        "summary\tgalaxies={galaxy_count}\tdocumented_galaxies={documented_galaxy_count}\tdocumented_library_modules={documented_library_module_count}\tdocumented_items={documented_item_count}"
    )
}
