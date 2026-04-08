use std::{fs, path::Path, path::PathBuf};

pub struct FormatReport {
    pub total_files: usize,
    pub changed_files: Vec<String>,
}

pub fn format_input(input: &Path) -> Result<FormatReport, String> {
    let files = collect_input_files(input)?;
    let mut changed_files = Vec::new();
    for path in &files {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let ast = crate::frontend::parse_nuis_ast(&source)?;
        let formatted = format_source_by_domain(&ast.domain, &source);
        // Ensure formatter never produces syntactically invalid nuis source.
        crate::frontend::parse_nuis_ast(&formatted)?;
        if formatted != source {
            fs::write(path, formatted)
                .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
            changed_files.push(path.display().to_string());
        }
    }
    Ok(FormatReport {
        total_files: files.len(),
        changed_files,
    })
}

fn collect_input_files(input: &Path) -> Result<Vec<PathBuf>, String> {
    if crate::project::is_project_input(input) {
        let project = crate::project::load_project(input)?;
        let mut files = project
            .modules
            .iter()
            .map(|module| module.path.clone())
            .collect::<Vec<_>>();
        files.sort();
        files.dedup();
        return Ok(files);
    }
    Ok(vec![input.to_path_buf()])
}

fn format_source_by_domain(domain: &str, source: &str) -> String {
    // Dispatch point: future domain-specific formatter hooks should be provided
    // by each domain nustar implementation. Current phase uses shared baseline.
    match domain {
        "cpu" | "shader" | "kernel" | "data" => format_with_baseline_rules(source),
        _ => format_with_baseline_rules(source),
    }
}

fn format_with_baseline_rules(source: &str) -> String {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = normalized
        .split('\n')
        .map(|line| line.trim_end().to_owned())
        .collect::<Vec<_>>();
    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}
