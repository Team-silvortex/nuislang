use std::{fs, path::PathBuf};

use nuis_semantics::bootstrap_subset::{
    validate_bootstrap_subset, BootstrapSubsetContext, BootstrapSubsetDiagnostic,
    BootstrapSubsetReport, BOOTSTRAP_SUBSET_PROTOCOL,
};
use nuis_semantics::model::AstModule;

use crate::command_helpers::resolve_compile_input;
use crate::{
    frontend, json_bool_field, json_escape, json_string_field, json_usize_field, pipeline,
};

#[cfg(test)]
#[path = "command_bootstrap_tests.rs"]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrapCheckReport {
    input_kind: &'static str,
    modules: Vec<BootstrapSubsetReport>,
    semantic_pipeline: &'static str,
    semantic_error: Option<String>,
}

impl BootstrapCheckReport {
    fn accepted(&self) -> bool {
        self.semantic_error.is_none() && self.modules.iter().all(BootstrapSubsetReport::accepted)
    }

    fn checked_nodes(&self) -> usize {
        self.modules.iter().map(|report| report.checked_nodes).sum()
    }

    fn diagnostic_count(&self) -> usize {
        self.modules
            .iter()
            .map(|report| report.diagnostics.len())
            .sum()
    }
}

pub(crate) fn run_bootstrap_check(input: PathBuf, json: bool) -> Result<(), String> {
    let report = inspect_bootstrap_input(&input)?;
    let rendered = if json {
        render_bootstrap_check_json(&report)
    } else {
        render_bootstrap_check_text(&report)
    };
    if report.accepted() {
        println!("{rendered}");
        Ok(())
    } else {
        Err(rendered)
    }
}

pub(crate) fn ensure_bootstrap_subset_resolved(
    resolved: &pipeline::ResolvedCompileInput,
) -> Result<(), String> {
    let (input_kind, modules) = bootstrap_modules_for_resolved(resolved)?;
    let report = inspect_bootstrap_subset_modules(input_kind, modules)?;
    ensure_bootstrap_report(&report)
}

fn ensure_bootstrap_report(report: &BootstrapCheckReport) -> Result<(), String> {
    if report.accepted() {
        Ok(())
    } else {
        Err(render_bootstrap_check_text(report))
    }
}

fn inspect_bootstrap_input(input: &std::path::Path) -> Result<BootstrapCheckReport, String> {
    let resolved = resolve_compile_input(input)?;
    inspect_bootstrap_resolved(&resolved)
}

fn inspect_bootstrap_resolved(
    resolved: &pipeline::ResolvedCompileInput,
) -> Result<BootstrapCheckReport, String> {
    let (input_kind, modules) = bootstrap_modules_for_resolved(resolved)?;
    inspect_bootstrap_modules(input_kind, modules, || resolved.compile().map(|_| ()))
}

fn bootstrap_modules_for_resolved(
    resolved: &pipeline::ResolvedCompileInput,
) -> Result<(&'static str, Vec<AstModule>), String> {
    let input_kind = if resolved.project.is_some() {
        "project"
    } else {
        "source"
    };
    let modules = if let Some(project) = &resolved.project {
        let mut modules = project
            .modules
            .iter()
            .filter(|module| module.origin.source_kind() == "project-local")
            .map(|module| module.ast.clone())
            .collect::<Vec<_>>();
        if modules.is_empty() {
            modules.push(frontend::parse_nuis_ast(&project.entry_source)?);
        }
        modules
    } else {
        let source = fs::read_to_string(&resolved.effective_input_path).map_err(|error| {
            format!(
                "failed to read `{}`: {error}",
                resolved.effective_input_path.display()
            )
        })?;
        vec![frontend::parse_nuis_ast(&source)?]
    };
    Ok((input_kind, modules))
}

fn inspect_bootstrap_modules<F>(
    input_kind: &'static str,
    modules: Vec<AstModule>,
    semantic_check: F,
) -> Result<BootstrapCheckReport, String>
where
    F: FnOnce() -> Result<(), String>,
{
    let mut report = inspect_bootstrap_subset_modules(input_kind, modules)?;
    if !report.accepted() {
        return Ok(report);
    }
    report.semantic_error = semantic_check().err();
    report.semantic_pipeline = if report.semantic_error.is_some() {
        "failed"
    } else {
        "checked"
    };
    Ok(report)
}

fn inspect_bootstrap_subset_modules(
    input_kind: &'static str,
    mut modules: Vec<AstModule>,
) -> Result<BootstrapCheckReport, String> {
    if modules.is_empty() {
        return Err("bootstrap check requires at least one local source module".to_owned());
    }
    modules.sort_by(|lhs, rhs| (&lhs.domain, &lhs.unit).cmp(&(&rhs.domain, &rhs.unit)));
    let context = BootstrapSubsetContext::from_modules(&modules);
    let reports = modules
        .iter()
        .map(|module| validate_bootstrap_subset(module, &context))
        .collect::<Vec<_>>();
    let subset_accepted = reports.iter().all(BootstrapSubsetReport::accepted);
    let semantic_pipeline = if subset_accepted {
        "deferred"
    } else {
        "skipped"
    };
    Ok(BootstrapCheckReport {
        input_kind,
        modules: reports,
        semantic_pipeline,
        semantic_error: None,
    })
}

#[cfg(test)]
fn inspect_bootstrap_source(source: &str) -> Result<BootstrapCheckReport, String> {
    let module = frontend::parse_nuis_ast(source)?;
    inspect_bootstrap_modules("source", vec![module], || {
        crate::pipeline::compile_source(source).map(|_| ())
    })
}

fn render_bootstrap_check_text(report: &BootstrapCheckReport) -> String {
    let mut lines = vec![
        format!(
            "bootstrap subset check: {}",
            if report.accepted() {
                "accepted"
            } else {
                "rejected"
            }
        ),
        format!("  protocol: {BOOTSTRAP_SUBSET_PROTOCOL}"),
        format!("  input_kind: {}", report.input_kind),
        format!("  modules: {}", report.modules.len()),
        format!("  checked_nodes: {}", report.checked_nodes()),
        format!("  semantic_pipeline: {}", report.semantic_pipeline),
        format!("  diagnostics: {}", report.diagnostic_count()),
    ];
    for module in &report.modules {
        for diagnostic in &module.diagnostics {
            lines.push(render_diagnostic_text(module, diagnostic));
        }
    }
    if let Some(error) = &report.semantic_error {
        lines.push(format!("  semantic_error: {error}"));
    }
    lines.join("\n")
}

fn render_diagnostic_text(
    module: &BootstrapSubsetReport,
    diagnostic: &BootstrapSubsetDiagnostic,
) -> String {
    format!(
        "  - [{}] {}: {} ({})",
        diagnostic.code,
        module.module_identity(),
        diagnostic.message,
        diagnostic.path
    )
}

fn render_bootstrap_check_json(report: &BootstrapCheckReport) -> String {
    let diagnostics = report
        .modules
        .iter()
        .flat_map(|module| {
            module
                .diagnostics
                .iter()
                .map(move |diagnostic| render_diagnostic_json(module, diagnostic))
        })
        .collect::<Vec<_>>()
        .join(",");
    let semantic_error = report
        .semantic_error
        .as_ref()
        .map(|error| format!("\"{}\"", json_escape(error)))
        .unwrap_or_else(|| "null".to_owned());
    format!(
        "{{{},{},{},{},{},{},{},\"diagnostics\":[{}],\"semantic_error\":{}}}",
        json_string_field("protocol", BOOTSTRAP_SUBSET_PROTOCOL),
        json_bool_field("accepted", report.accepted()),
        json_string_field("input_kind", report.input_kind),
        json_usize_field("module_count", report.modules.len()),
        json_usize_field("checked_nodes", report.checked_nodes()),
        json_string_field("semantic_pipeline", report.semantic_pipeline),
        json_usize_field("diagnostic_count", report.diagnostic_count()),
        diagnostics,
        semantic_error,
    )
}

fn render_diagnostic_json(
    module: &BootstrapSubsetReport,
    diagnostic: &BootstrapSubsetDiagnostic,
) -> String {
    format!(
        "{{{},{},{},{}}}",
        json_string_field("module", &module.module_identity()),
        json_string_field("code", diagnostic.code),
        json_string_field("path", &diagnostic.path),
        json_string_field("message", &diagnostic.message),
    )
}
