use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    json_bool_field, json_field, json_object_array_field, json_optional_string_field,
    json_string_array_field, json_usize_field, yes_no,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectImportRecord {
    galaxy: String,
    library_module: String,
    import_policy: String,
    auto_injectable: bool,
    visible: bool,
    explicit: bool,
    source_kind: Option<String>,
    source_detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectImportsReport {
    project_name: String,
    root: PathBuf,
    manifest_path: PathBuf,
    galaxy_dependencies: Vec<String>,
    explicit_galaxy_imports: Vec<String>,
    suggested_galaxy_imports: Vec<String>,
    visible_library_modules: Vec<String>,
    hidden_manual_only_library_modules: Vec<String>,
    records: Vec<ProjectImportRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectImportsApplyResult {
    pub(crate) manifest_path: PathBuf,
    pub(crate) applied: Vec<String>,
    pub(crate) total_explicit_galaxy_imports: usize,
    pub(crate) manifest_updated: bool,
}

pub(crate) fn hidden_manual_only_library_modules_for_project(
    project: &nuisc::project::LoadedProject,
) -> Vec<String> {
    let explicit_galaxy_imports = project
        .manifest
        .galaxy_imports
        .iter()
        .map(|item| format!("{}:{}", item.galaxy, item.library_module))
        .collect::<BTreeSet<_>>();
    project
        .resolved_galaxies
        .iter()
        .filter(|dependency| dependency.library_import_policy.as_str() == "manual-only")
        .flat_map(|dependency| {
            dependency
                .library_modules
                .iter()
                .filter_map(|library_module| {
                    let key = format!("{}:{}", dependency.name, library_module);
                    if explicit_galaxy_imports.contains(&key) {
                        None
                    } else {
                        Some(key)
                    }
                })
        })
        .collect::<Vec<_>>()
}

fn collect_project_imports_report(input: &Path) -> Result<ProjectImportsReport, String> {
    let project = nuisc::project::load_project(input)?;
    let explicit_galaxy_imports = project
        .manifest
        .galaxy_imports
        .iter()
        .map(|item| format!("{}:{}", item.galaxy, item.library_module))
        .collect::<BTreeSet<_>>();
    let mut records = Vec::new();
    let mut visible_library_modules = Vec::new();
    let hidden_manual_only_library_modules =
        hidden_manual_only_library_modules_for_project(&project);
    let suggested_galaxy_imports = hidden_manual_only_library_modules.clone();

    for dependency in &project.resolved_galaxies {
        for (library_module, library_path) in dependency
            .library_modules
            .iter()
            .zip(dependency.resolved_library_paths.iter())
        {
            let key = format!("{}:{}", dependency.name, library_module);
            let visible_module = project
                .modules
                .iter()
                .find(|module| module.path == *library_path);
            let visible = visible_module.is_some();
            if visible {
                visible_library_modules.push(key.clone());
            }
            records.push(ProjectImportRecord {
                galaxy: dependency.name.clone(),
                library_module: library_module.clone(),
                import_policy: dependency.library_import_policy.as_str().to_owned(),
                auto_injectable: dependency.auto_injectable,
                visible,
                explicit: explicit_galaxy_imports.contains(&key),
                source_kind: visible_module.map(|module| module.origin.source_kind().to_owned()),
                source_detail: visible_module.map(|module| module.origin.source_detail()),
            });
        }
    }

    Ok(ProjectImportsReport {
        project_name: project.manifest.name.clone(),
        root: project.root.clone(),
        manifest_path: project.manifest_path.clone(),
        galaxy_dependencies: project
            .manifest
            .galaxy_dependencies
            .iter()
            .map(|item| format!("{}={}", item.name, item.version))
            .collect::<Vec<_>>(),
        explicit_galaxy_imports: explicit_galaxy_imports.into_iter().collect::<Vec<_>>(),
        suggested_galaxy_imports,
        visible_library_modules,
        hidden_manual_only_library_modules,
        records,
    })
}

pub(crate) fn handle_project_imports(
    input: std::path::PathBuf,
    json: bool,
    apply_suggested: bool,
) -> Result<(), String> {
    if apply_suggested {
        let applied = apply_suggested_project_imports(&input)?;
        if json {
            println!("{}", render_project_imports_apply_json(&input, &applied)?);
            return Ok(());
        }
        println!(
            "applied project imports: {}",
            applied.manifest_path.display()
        );
        println!("  applied_galaxy_imports: {}", applied.applied.len());
        println!(
            "  total_explicit_galaxy_imports: {}",
            applied.total_explicit_galaxy_imports
        );
        println!("  manifest_updated: {}", yes_no(applied.manifest_updated));
        if applied.applied.is_empty() {
            println!("  result: no suggested galaxy imports needed to be written");
        } else {
            for item in &applied.applied {
                println!("  applied_galaxy_import: {}", item);
            }
        }
        for line in render_project_imports_text_summary(&input)? {
            println!("{line}");
        }
        return Ok(());
    }
    if json {
        println!("{}", render_project_imports_json(&input)?);
        return Ok(());
    }
    for line in render_project_imports_text_summary(&input)? {
        println!("{line}");
    }
    Ok(())
}

pub(crate) fn apply_suggested_project_imports(
    input: &Path,
) -> Result<ProjectImportsApplyResult, String> {
    let report = collect_project_imports_report(input)?;
    let manifest_source = fs::read_to_string(&report.manifest_path).map_err(|error| {
        format!(
            "failed to read project manifest `{}`: {error}",
            report.manifest_path.display()
        )
    })?;
    let updated_source = write_manifest_galaxy_imports(
        &manifest_source,
        &report.explicit_galaxy_imports,
        &report.suggested_galaxy_imports,
    )?;
    let manifest_updated = updated_source != manifest_source;
    if manifest_updated {
        fs::write(&report.manifest_path, updated_source).map_err(|error| {
            format!(
                "failed to update project manifest `{}`: {error}",
                report.manifest_path.display()
            )
        })?;
    }
    Ok(ProjectImportsApplyResult {
        manifest_path: report.manifest_path,
        total_explicit_galaxy_imports: report.explicit_galaxy_imports.len()
            + report.suggested_galaxy_imports.len(),
        applied: report.suggested_galaxy_imports,
        manifest_updated,
    })
}

fn write_manifest_galaxy_imports(
    source: &str,
    explicit: &[String],
    suggested: &[String],
) -> Result<String, String> {
    if suggested.is_empty() {
        return Ok(source.to_owned());
    }
    let merged = merge_manifest_galaxy_imports(explicit, suggested);
    let replacement = render_manifest_galaxy_imports_block(&merged);
    if let Some((start, end)) = find_manifest_field_span(source, "galaxy_imports") {
        let mut updated = String::new();
        updated.push_str(&source[..start]);
        updated.push_str(&replacement);
        updated.push_str(&source[end..]);
        Ok(updated)
    } else {
        let mut updated = source.to_owned();
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(&replacement);
        Ok(updated)
    }
}

fn merge_manifest_galaxy_imports(explicit: &[String], suggested: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();
    for item in explicit.iter().chain(suggested.iter()) {
        if seen.insert(item.clone()) {
            merged.push(item.clone());
        }
    }
    merged
}

fn render_manifest_galaxy_imports_block(values: &[String]) -> String {
    let mut rendered = String::from("galaxy_imports = [\n");
    for value in values {
        rendered.push_str("  \"");
        rendered.push_str(value);
        rendered.push_str("\",\n");
    }
    rendered.push_str("]\n");
    rendered
}

fn find_manifest_field_span(source: &str, key: &str) -> Option<(usize, usize)> {
    let prefix = format!("{key} = ");
    let mut cursor = 0usize;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let offset = line.len() - trimmed.len();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            let start = cursor + offset;
            let mut end = cursor + line.len();
            if !rest.contains(']') {
                let mut scan = end;
                while scan < source.len() {
                    let remaining = &source[scan..];
                    let next_len = remaining
                        .find('\n')
                        .map(|idx| idx + 1)
                        .unwrap_or(remaining.len());
                    let next = &remaining[..next_len];
                    end += next.len();
                    if next.contains(']') {
                        break;
                    }
                    scan += next_len;
                }
            }
            return Some((start, end));
        }
        cursor += line.len();
    }
    None
}

fn render_project_imports_text_summary(input: &Path) -> Result<Vec<String>, String> {
    let report = collect_project_imports_report(input)?;
    let mut lines = vec![
        format!("project imports: {}", report.project_name),
        format!("  root: {}", report.root.display()),
        format!("  manifest: {}", report.manifest_path.display()),
        format!(
            "  galaxy_dependencies: {}",
            report.galaxy_dependencies.len()
        ),
        format!(
            "  explicit_galaxy_imports: {}",
            report.explicit_galaxy_imports.len()
        ),
        format!(
            "  visible_library_modules: {}",
            report.visible_library_modules.len()
        ),
        format!(
            "  hidden_manual_only_library_modules: {}",
            report.hidden_manual_only_library_modules.len()
        ),
        format!(
            "  suggested_galaxy_imports: {}",
            report.suggested_galaxy_imports.len()
        ),
    ];
    for item in &report.galaxy_dependencies {
        lines.push(format!("  galaxy_dependency: {}", item));
    }
    for item in &report.explicit_galaxy_imports {
        lines.push(format!("  explicit_galaxy_import: {}", item));
    }
    for item in &report.suggested_galaxy_imports {
        lines.push(format!("  suggested_galaxy_import: {}", item));
    }
    if !report.suggested_galaxy_imports.is_empty() {
        lines.push(format!(
            "  manifest_snippet: galaxy_imports = [{}]",
            report
                .suggested_galaxy_imports
                .iter()
                .map(|item| format!("\"{}\"", item))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for record in &report.records {
        let mut line = format!(
            "  library: {}:{} import_policy={} auto_injectable={} visible={} explicit={}",
            record.galaxy,
            record.library_module,
            record.import_policy,
            yes_no(record.auto_injectable),
            yes_no(record.visible),
            yes_no(record.explicit),
        );
        if let Some(source_kind) = record.source_kind.as_deref() {
            line.push_str(&format!(" source_kind={source_kind}"));
        }
        lines.push(line);
    }
    Ok(lines)
}

pub(crate) fn render_project_imports_json(input: &Path) -> Result<String, String> {
    let report = collect_project_imports_report(input)?;
    let records = report
        .records
        .iter()
        .map(|record| {
            let fields = [
                json_field("galaxy", &record.galaxy),
                json_field("library_module", &record.library_module),
                json_field("import_policy", &record.import_policy),
                json_bool_field("auto_injectable", record.auto_injectable),
                json_bool_field("visible", record.visible),
                json_bool_field("explicit", record.explicit),
                json_optional_string_field("source_kind", record.source_kind.as_deref()),
                json_optional_string_field("source_detail", record.source_detail.as_deref()),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>();
    let suggested_manifest_snippet = format!(
        "galaxy_imports = [{}]",
        report
            .suggested_galaxy_imports
            .iter()
            .map(|item| format!("\"{}\"", item))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut out = String::from("{");
    for field in [
        json_field("source_kind", "project"),
        json_field("input", &input.display().to_string()),
        json_field("project", &report.project_name),
        json_field("root", &report.root.display().to_string()),
        json_field("manifest", &report.manifest_path.display().to_string()),
        json_usize_field(
            "galaxy_dependencies_count",
            report.galaxy_dependencies.len(),
        ),
        json_string_array_field("galaxy_dependencies", &report.galaxy_dependencies),
        json_usize_field(
            "explicit_galaxy_imports_count",
            report.explicit_galaxy_imports.len(),
        ),
        json_string_array_field("explicit_galaxy_imports", &report.explicit_galaxy_imports),
        json_usize_field(
            "visible_library_modules_count",
            report.visible_library_modules.len(),
        ),
        json_string_array_field("visible_library_modules", &report.visible_library_modules),
        json_usize_field(
            "hidden_manual_only_library_modules_count",
            report.hidden_manual_only_library_modules.len(),
        ),
        json_string_array_field(
            "hidden_manual_only_library_modules",
            &report.hidden_manual_only_library_modules,
        ),
        json_usize_field(
            "suggested_galaxy_imports_count",
            report.suggested_galaxy_imports.len(),
        ),
        json_string_array_field("suggested_galaxy_imports", &report.suggested_galaxy_imports),
        json_field("suggested_manifest_snippet", &suggested_manifest_snippet),
        json_object_array_field("library_records", &records),
    ] {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
    out.push('}');
    Ok(out)
}

pub(crate) fn render_project_imports_apply_json(
    input: &Path,
    applied: &ProjectImportsApplyResult,
) -> Result<String, String> {
    let base = render_project_imports_json(input)?;
    let Some(prefix) = base.strip_suffix('}') else {
        return Err("project imports json renderer returned malformed object".to_owned());
    };
    let mut out = String::from("{");
    for field in [
        json_field("kind", "project_imports_apply"),
        json_field("action", "apply_suggested"),
        json_field(
            "manifest_path",
            &applied.manifest_path.display().to_string(),
        ),
        json_bool_field("manifest_updated", applied.manifest_updated),
        json_usize_field("applied_galaxy_imports_count", applied.applied.len()),
        json_string_array_field("applied_galaxy_imports", &applied.applied),
        json_usize_field(
            "total_explicit_galaxy_imports",
            applied.total_explicit_galaxy_imports,
        ),
    ] {
        if !out.ends_with('{') {
            out.push(',');
        }
        out.push_str(&field);
    }
    if !prefix.trim_start_matches('{').is_empty() {
        out.push(',');
        out.push_str(prefix.trim_start_matches('{'));
    }
    out.push('}');
    Ok(out)
}
