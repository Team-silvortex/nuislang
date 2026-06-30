use super::*;

pub fn project_docs_summary(project: &LoadedProject) -> ProjectDocsSummary {
    let mut documented_modules = 0usize;
    let mut documented_items = 0usize;
    for module in &project.modules {
        let index = crate::frontend::extract_ast_doc_index(&module.ast);
        if !index.items.is_empty() {
            documented_modules += 1;
            documented_items += index.items.len();
        }
    }
    ProjectDocsSummary {
        modules: project.modules.len(),
        documented_modules,
        documented_items,
    }
}

pub fn project_galaxy_summary(project: &LoadedProject) -> ProjectGalaxySummary {
    let mut documented_galaxies = 0usize;
    let mut documented_library_modules = 0usize;
    let mut documented_items = 0usize;
    for dependency in &project.resolved_galaxies {
        let summary = crate::stdlib_registry::summarize_resolved_galaxy_docs(dependency);
        if summary.documented_items > 0 {
            documented_galaxies += 1;
            documented_library_modules += summary.documented_library_modules;
            documented_items += summary.documented_items;
        }
    }
    ProjectGalaxySummary {
        galaxies: project.resolved_galaxies.len(),
        documented_galaxies,
        documented_library_modules,
        documented_items,
    }
}

pub(in crate::project) fn write_project_docs_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> fmt::Result {
    let summary = project_docs_summary(project);
    writeln!(
        out,
        "summary\tmodules={}\tdocumented_modules={}\tdocumented_items={}",
        summary.modules, summary.documented_modules, summary.documented_items
    )?;
    for module in &project.modules {
        let index = crate::frontend::extract_ast_doc_index(&module.ast);
        writeln!(
            out,
            "module\t{}\titems={}\tsource_kind={}\t{}",
            index.module_path,
            index.items.len(),
            module.origin.source_kind(),
            module.origin.source_detail()
        )?;
        for item in &index.items {
            writeln!(
                out,
                "item\t{}\t{}\tdoc_lines={}\tsignature={}",
                item.kind,
                item.path,
                item.docs.len(),
                item.signature.as_deref().unwrap_or("<none>")
            )?;
        }
    }
    Ok(())
}
