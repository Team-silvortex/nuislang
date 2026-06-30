use super::*;

pub fn render_project_import_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_import_index(&mut out, project)
        .expect("writing project import index to String should not fail");
    out
}

pub fn project_imports_summary(project: &LoadedProject) -> ProjectImportsSummary {
    let doc_item_counts = project
        .modules
        .iter()
        .map(|module| {
            (
                module.path.clone(),
                crate::frontend::extract_ast_doc_index(&module.ast)
                    .items
                    .len(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let visible_library_paths = project
        .modules
        .iter()
        .filter_map(|module| match &module.origin {
            super::ProjectModuleOrigin::AutoInjectedGalaxy { .. }
            | super::ProjectModuleOrigin::ExplicitGalaxyImport { .. } => Some(module.path.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    ProjectImportsSummary {
        libraries: project
            .resolved_galaxies
            .iter()
            .map(|dependency| dependency.library_modules.len())
            .sum(),
        visible_libraries: visible_library_paths.len(),
        visible_modules: project.modules.len(),
        documented_visible_modules: project
            .modules
            .iter()
            .filter(|module| doc_item_counts.get(&module.path).copied().unwrap_or(0) > 0)
            .count(),
        documented_visible_items: project
            .modules
            .iter()
            .map(|module| doc_item_counts.get(&module.path).copied().unwrap_or(0))
            .sum(),
    }
}

pub fn write_project_import_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> fmt::Result {
    let local_units = project
        .modules
        .iter()
        .map(|module| ((module.ast.domain.clone(), module.ast.unit.clone()), module))
        .collect::<BTreeMap<_, _>>();
    let doc_item_counts = project
        .modules
        .iter()
        .map(|module| {
            (
                module.path.clone(),
                crate::frontend::extract_ast_doc_index(&module.ast)
                    .items
                    .len(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let visible_library_paths = project
        .modules
        .iter()
        .filter_map(|module| match &module.origin {
            super::ProjectModuleOrigin::AutoInjectedGalaxy { .. }
            | super::ProjectModuleOrigin::ExplicitGalaxyImport { .. } => Some(module.path.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let summary = project_imports_summary(project);

    writeln!(
        out,
        "summary\tlibraries={}\tvisible_libraries={}\tvisible_modules={}\tdocumented_visible_modules={}\tdocumented_visible_items={}",
        summary.libraries,
        summary.visible_libraries,
        summary.visible_modules,
        summary.documented_visible_modules,
        summary.documented_visible_items
    )?;

    for dependency in &project.resolved_galaxies {
        for (library_module, library_path) in dependency
            .library_modules
            .iter()
            .zip(dependency.resolved_library_paths.iter())
        {
            writeln!(
                out,
                "library\t{}\t{}\timport_policy={}\tauto_injectable={}\tvisible={}\tdoc_items={}",
                dependency.name,
                library_module,
                dependency.library_import_policy.as_str(),
                if dependency.auto_injectable {
                    "true"
                } else {
                    "false"
                },
                if visible_library_paths.contains(library_path) {
                    "true"
                } else {
                    "false"
                },
                doc_item_counts.get(library_path).copied().unwrap_or(0)
            )?;
        }
    }

    for module in &project.modules {
        writeln!(
            out,
            "visible\t{}\t{}\tdoc_items={}\tsource_kind={}\t{}",
            module.ast.domain,
            module.ast.unit,
            doc_item_counts.get(&module.path).copied().unwrap_or(0),
            module.origin.source_kind(),
            module.origin.source_detail()
        )?;
    }

    for module in &project.modules {
        for item in &module.ast.uses {
            write!(
                out,
                "use\t{}.{}\t{}.{}\tresolution=",
                module.ast.domain, module.ast.unit, item.domain, item.unit
            )?;
            if let Some(local) = local_units.get(&(item.domain.clone(), item.unit.clone())) {
                write!(
                    out,
                    "local-visible:{}:{}",
                    local.origin.source_kind(),
                    local.origin.source_detail()
                )?;
            } else {
                write!(out, "registered-domain-unit")?;
            }
            writeln!(out)?;
        }
    }

    Ok(())
}
