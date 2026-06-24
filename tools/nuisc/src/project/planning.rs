use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use super::{
    organize_project, organize_project_exchanges, packet, parse_project_manifest,
    render_project_abi_graph_line, resolve_project_abi, validate_project_abi_requirements,
    validate_project_links, validate_project_modules, validate_project_unit_bindings, validate_project_uses,
    LoadedProject, ProjectBuildMetadata, ProjectCompilationDependency, ProjectCompilationPlan,
    ProjectDocsSummary, ProjectGalaxySummary, ProjectModule,
    ProjectModuleOrigin, ProjectOutputIntent, ProjectSyntheticInput,
};
use super::{
    write_project_abi_index, write_project_exchange_index, write_project_host_ffi_index,
    write_project_import_index, write_project_organization_index,
};

fn write_project_modules_index<W: fmt::Write>(
    out: &mut W,
    organization: &super::ProjectOrganization,
) -> fmt::Result {
    for module in &organization.modules {
        writeln!(
            out,
            "{}\tmod {} {}\tentry={}\tsource_kind={}\t{}",
            module.path,
            module.domain,
            module.unit,
            module.is_entry,
            module.source_kind,
            module.source_detail
        )?;
    }
    Ok(())
}

fn write_project_links_index<W: fmt::Write>(
    out: &mut W,
    organization: &super::ProjectOrganization,
) -> fmt::Result {
    for link in &organization.links {
        writeln!(
            out,
            "{}\t{}\t{}",
            link.from,
            link.to,
            link.via.as_deref().unwrap_or("<direct>")
        )?;
    }
    Ok(())
}

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

fn write_project_docs_index<W: fmt::Write>(
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

pub fn load_project(input: &Path) -> Result<LoadedProject, String> {
    let manifest_path = if input.is_dir() {
        input.join("nuis.toml")
    } else {
        input.to_path_buf()
    };
    let root = manifest_path
        .parent()
        .ok_or_else(|| {
            format!(
                "project manifest `{}` has no parent directory",
                manifest_path.display()
            )
        })?
        .to_path_buf();
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest = parse_project_manifest(&source, &manifest_path)?;
    let stdlib_root = crate::stdlib_registry::resolve_stdlib_root()?;
    let resolved_galaxies = crate::stdlib_registry::resolve_galaxy_dependencies(
        &stdlib_root,
        &manifest.galaxy_dependencies,
    )?;

    let module_specs = if manifest.modules.is_empty() {
        vec![manifest.entry.clone()]
    } else {
        manifest.modules.clone()
    };
    let mut seen_paths = BTreeSet::new();
    let mut modules = Vec::new();
    for spec in module_specs {
        let path = root.join(&spec);
        if !seen_paths.insert(path.clone()) {
            continue;
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let ast = crate::frontend::parse_nuis_ast(&source)?;
        modules.push(ProjectModule {
            path,
            ast,
            origin: ProjectModuleOrigin::LocalProject {
                manifest_spec: spec,
            },
        });
    }
    for dependency in &resolved_galaxies {
        if !dependency.auto_injectable {
            continue;
        }
        for (library_module, path) in dependency
            .library_modules
            .iter()
            .zip(dependency.resolved_library_paths.iter())
        {
            if !seen_paths.insert(path.clone()) {
                continue;
            }
            let source = fs::read_to_string(path)
                .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
            let ast = crate::frontend::parse_nuis_ast(&source)?;
            modules.push(ProjectModule {
                path: path.clone(),
                ast,
                origin: ProjectModuleOrigin::AutoInjectedGalaxy {
                    galaxy: dependency.name.clone(),
                    package_id: dependency.package_id.clone(),
                    library_module: library_module.clone(),
                    import_policy: dependency.library_import_policy.as_str().to_owned(),
                },
            });
        }
    }
    for import in &manifest.galaxy_imports {
        let dependency = resolved_galaxies
            .iter()
            .find(|item| item.name == import.galaxy)
            .ok_or_else(|| {
                format!(
                    "project galaxy import `{}:{}` references unknown resolved galaxy `{}`",
                    import.galaxy, import.library_module, import.galaxy
                )
            })?;
        let Some((_, path)) = dependency
            .library_modules
            .iter()
            .zip(dependency.resolved_library_paths.iter())
            .find(|(library_module, _)| *library_module == &import.library_module)
        else {
            return Err(format!(
                "project galaxy import `{}:{}` is not declared by galaxy `{}`; declared library_modules=[{}]",
                import.galaxy,
                import.library_module,
                dependency.name,
                if dependency.library_modules.is_empty() {
                    "<none>".to_owned()
                } else {
                    dependency.library_modules.join(", ")
                }
            ));
        };
        if !seen_paths.insert(path.clone()) {
            continue;
        }
        let source = fs::read_to_string(path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        let ast = crate::frontend::parse_nuis_ast(&source)?;
        modules.push(ProjectModule {
            path: path.clone(),
            ast,
            origin: ProjectModuleOrigin::ExplicitGalaxyImport {
                galaxy: dependency.name.clone(),
                package_id: dependency.package_id.clone(),
                library_module: import.library_module.clone(),
                import_policy: dependency.library_import_policy.as_str().to_owned(),
            },
        });
    }

    let entry_path = root.join(&manifest.entry);
    let entry_source = fs::read_to_string(&entry_path)
        .map_err(|error| format!("failed to read `{}`: {error}", entry_path.display()))?;

    validate_project_modules(&modules)?;
    validate_project_unit_bindings(&modules)?;
    validate_project_uses(&modules, &resolved_galaxies)?;
    validate_project_links(&manifest, &modules)?;
    validate_project_abi_requirements(&manifest, &modules)?;

    Ok(LoadedProject {
        root,
        manifest_path,
        manifest,
        entry_path,
        entry_source,
        modules,
        resolved_galaxies,
    })
}

pub fn describe_project(project: &LoadedProject) -> String {
    let organization = organize_project(project);
    let modules = organization
        .modules
        .iter()
        .map(|module| format!("{} (mod {} {})", module.path, module.domain, module.unit))
        .collect::<Vec<_>>()
        .join(", ");
    let links = if organization.links.is_empty() {
        "<none>".to_owned()
    } else {
        organization
            .links
            .iter()
            .map(|link| {
                if let Some(via) = &link.via {
                    format!("{} -> {} via {}", link.from, link.to, via)
                } else {
                    format!("{} -> {}", link.from, link.to)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let abi_summary = match resolve_project_abi(project) {
        Ok(resolution) if resolution.requirements.is_empty() => "abi=<none>".to_owned(),
        Ok(resolution) => {
            let mode = if resolution.explicit {
                "abi=locked"
            } else {
                "abi=auto"
            };
            let entries = resolution
                .requirements
                .iter()
                .map(|item| format!("{}={}", item.domain, item.abi))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{mode}({entries})")
        }
        Err(_) => "abi=<unresolved>".to_owned(),
    };
    let galaxy_summary = if project.manifest.galaxy_dependencies.is_empty() {
        "galaxy=<none>".to_owned()
    } else {
        let deps = project
            .manifest
            .galaxy_dependencies
            .iter()
            .map(|item| format!("{}={}", item.name, item.version))
            .collect::<Vec<_>>()
            .join(", ");
        format!("galaxy=[{deps}]")
    };
    let galaxy_import_summary = if project.manifest.galaxy_imports.is_empty() {
        "galaxy_imports=<none>".to_owned()
    } else {
        format!(
            "galaxy_imports=[{}]",
            project
                .manifest
                .galaxy_imports
                .iter()
                .map(|item| format!("{}:{}", item.galaxy, item.library_module))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "project={} entry={} modules={} links={} {} {} {}",
        project.manifest.name,
        project.manifest.entry,
        modules,
        links,
        abi_summary,
        galaxy_summary,
        galaxy_import_summary
    )
}

pub fn build_project_compilation_plan(
    project: &LoadedProject,
) -> Result<ProjectCompilationPlan, String> {
    let organization = organize_project(project);
    let exchanges = organize_project_exchanges(project);
    let abi_resolution = resolve_project_abi(project)?;
    let effective_input_path = project.root.join(format!("{}.ns", project.manifest.name));
    let dependencies = project
        .resolved_galaxies
        .iter()
        .map(|item| ProjectCompilationDependency {
            category: if item.direct {
                "stdlib-galaxy-direct".to_owned()
            } else {
                "stdlib-galaxy-transitive".to_owned()
            },
            name: item.name.clone(),
            version: item.version.clone(),
            source: if item.direct {
                "project-galaxy-manifest".to_owned()
            } else {
                format!("transitive via {}", item.requested_by.join(","))
            },
        })
        .collect::<Vec<_>>();
    let synthetic_input = ProjectSyntheticInput {
        kind: "project-name-entry".to_owned(),
        path: effective_input_path.clone(),
    };
    let output_intents = vec![
        ProjectOutputIntent {
            category: "core-artifacts".to_owned(),
            kind: "build-manifest".to_owned(),
            path_hint: "nuis.build.manifest.toml".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-manifest-copy".to_owned(),
            path_hint: "nuis.project.toml".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-plan-index".to_owned(),
            path_hint: "nuis.project.plan.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-organization-index".to_owned(),
            path_hint: "nuis.project.organization.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-exchange-index".to_owned(),
            path_hint: "nuis.project.exchange.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-modules-index".to_owned(),
            path_hint: "nuis.project.modules.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-imports-index".to_owned(),
            path_hint: "nuis.project.imports.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-galaxy-index".to_owned(),
            path_hint: "nuis.project.galaxy.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-links-index".to_owned(),
            path_hint: "nuis.project.links.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-packet-index".to_owned(),
            path_hint: "nuis.project.packet.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "project-metadata".to_owned(),
            kind: "project-host-ffi-index".to_owned(),
            path_hint: "nuis.project.host_ffi.txt".to_owned(),
        },
        ProjectOutputIntent {
            category: "verification-inputs".to_owned(),
            kind: "project-abi-index".to_owned(),
            path_hint: "nuis.project.abi.txt".to_owned(),
        },
    ];
    Ok(ProjectCompilationPlan {
        project_name: project.manifest.name.clone(),
        entry: project.manifest.entry.clone(),
        organization,
        exchanges,
        abi_resolution,
        dependencies,
        synthetic_input,
        output_intents,
        effective_input_path,
    })
}

pub fn describe_project_compilation_plan(plan: &ProjectCompilationPlan) -> String {
    format!(
        "entry={} domains={} exchanges={} abi_mode={}",
        plan.entry,
        plan.organization.domains.join(", "),
        plan.exchanges.routes.len(),
        if plan.abi_resolution.explicit {
            "explicit"
        } else {
            "auto-recommended"
        }
    )
}

pub fn describe_project_output_intent_categories(plan: &ProjectCompilationPlan) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in &plan.output_intents {
        *counts.entry(item.category.clone()).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return "<none>".to_owned();
    }
    counts
        .into_iter()
        .map(|(category, count)| format!("{category}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn describe_project_dependency_categories(plan: &ProjectCompilationPlan) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in &plan.dependencies {
        *counts.entry(item.category.clone()).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return "<none>".to_owned();
    }
    counts
        .into_iter()
        .map(|(category, count)| format!("{category}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn describe_project_exchange_route_classes(plan: &ProjectCompilationPlan) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for route in &plan.exchanges.routes {
        *counts.entry(route.class.clone()).or_insert(0) += 1;
    }
    if counts.is_empty() {
        return "<none>".to_owned();
    }
    counts
        .into_iter()
        .map(|(class, count)| format!("{class}={count}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn render_project_compilation_plan_index(plan: &ProjectCompilationPlan) -> String {
    let mut out = String::new();
    write_project_compilation_plan_index(&mut out, plan)
        .expect("writing project compilation plan index to String should not fail");
    out
}

pub fn write_project_compilation_plan_index<W: fmt::Write>(
    out: &mut W,
    plan: &ProjectCompilationPlan,
) -> fmt::Result {
    let abi_mode = if plan.abi_resolution.explicit {
        "explicit"
    } else {
        "auto-recommended"
    };
    writeln!(out, "project {}", plan.project_name)?;
    writeln!(out, "entry {}", plan.entry)?;
    write!(out, "domains ")?;
    write_joined(out, &plan.organization.domains, ", ", |out, domain| {
        write!(out, "{domain}")
    })?;
    writeln!(out)?;
    writeln!(out, "exchanges {}", plan.exchanges.routes.len())?;
    writeln!(out, "abi_mode {}", abi_mode)?;
    writeln!(
        out,
        "abi_graph {}",
        render_project_abi_graph_line(&plan.abi_resolution)
    )?;
    write!(out, "abi ")?;
    if plan.abi_resolution.requirements.is_empty() {
        writeln!(out, "<none>")?;
    } else {
        write_joined(out, &plan.abi_resolution.requirements, ", ", |out, item| {
            write!(out, "{}={}", item.domain, item.abi)
        })?;
        writeln!(out)?;
    }
    write!(out, "dependencies ")?;
    if plan.dependencies.is_empty() {
        writeln!(out, "<none>")?;
    } else {
        write_joined(out, &plan.dependencies, ", ", |out, item| {
            write!(
                out,
                "{}:{}={} ({})",
                item.category, item.name, item.version, item.source
            )
        })?;
        writeln!(out)?;
    }
    writeln!(out, "synthetic_input_kind {}", plan.synthetic_input.kind)?;
    writeln!(
        out,
        "synthetic_input {}",
        plan.synthetic_input.path.display()
    )?;
    write!(out, "output_intents ")?;
    if plan.output_intents.is_empty() {
        writeln!(out, "<none>")?;
    } else {
        write_joined(out, &plan.output_intents, ", ", |out, item| {
            write!(out, "{}:{}={}", item.category, item.kind, item.path_hint)
        })?;
        writeln!(out)?;
    }
    writeln!(
        out,
        "effective_input {}",
        plan.effective_input_path.display()
    )?;
    writeln!(out, "summary {}", describe_project_compilation_plan(plan))
}

fn write_joined<W, T, F>(out: &mut W, items: &[T], sep: &str, mut write_item: F) -> fmt::Result
where
    W: fmt::Write,
    F: FnMut(&mut W, &T) -> fmt::Result,
{
    let mut first = true;
    for item in items {
        if !first {
            out.write_str(sep)?;
        }
        first = false;
        write_item(out, item)?;
    }
    Ok(())
}

pub fn write_project_metadata(
    output_dir: &Path,
    project: &LoadedProject,
    plan: &ProjectCompilationPlan,
) -> Result<ProjectBuildMetadata, String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
    let manifest_copy_path = output_dir.join("nuis.project.toml");
    let plan_index_path = output_dir.join("nuis.project.plan.txt");
    let organization_index_path = output_dir.join("nuis.project.organization.txt");
    let exchange_index_path = output_dir.join("nuis.project.exchange.txt");
    let modules_index_path = output_dir.join("nuis.project.modules.txt");
    let docs_index_path = output_dir.join("nuis.project.docs.txt");
    let imports_index_path = output_dir.join("nuis.project.imports.txt");
    let galaxy_index_path = output_dir.join("nuis.project.galaxy.txt");
    let links_index_path = output_dir.join("nuis.project.links.txt");
    let packet_index_path = output_dir.join("nuis.project.packet.txt");
    let host_ffi_index_path = output_dir.join("nuis.project.host_ffi.txt");
    let abi_index_path = output_dir.join("nuis.project.abi.txt");
    fs::copy(&project.manifest_path, &manifest_copy_path).map_err(|error| {
        format!(
            "failed to copy project manifest `{}` -> `{}`: {error}",
            project.manifest_path.display(),
            manifest_copy_path.display()
        )
    })?;
    let plan_index = render_project_compilation_plan_index(plan);
    fs::write(&plan_index_path, plan_index).map_err(|error| {
        format!(
            "failed to write project plan index `{}`: {error}",
            plan_index_path.display()
        )
    })?;
    let mut organization_index = String::new();
    write_project_organization_index(&mut organization_index, project)
        .expect("writing project organization index to String should not fail");
    fs::write(&organization_index_path, organization_index).map_err(|error| {
        format!(
            "failed to write project organization index `{}`: {error}",
            organization_index_path.display()
        )
    })?;
    let mut exchange_index = String::new();
    write_project_exchange_index(&mut exchange_index, project)
        .expect("writing project exchange index to String should not fail");
    fs::write(&exchange_index_path, exchange_index).map_err(|error| {
        format!(
            "failed to write project exchange index `{}`: {error}",
            exchange_index_path.display()
        )
    })?;
    let organization = organize_project(project);
    let mut modules_index = String::new();
    write_project_modules_index(&mut modules_index, &organization)
        .expect("writing project modules index to String should not fail");
    fs::write(&modules_index_path, modules_index).map_err(|error| {
        format!(
            "failed to write project modules index `{}`: {error}",
            modules_index_path.display()
        )
    })?;
    let mut docs_index = String::new();
    write_project_docs_index(&mut docs_index, project)
        .expect("writing project docs index to String should not fail");
    fs::write(&docs_index_path, docs_index).map_err(|error| {
        format!(
            "failed to write project docs index `{}`: {error}",
            docs_index_path.display()
        )
    })?;
    let mut imports_index = String::new();
    write_project_import_index(&mut imports_index, project)
        .expect("writing project imports index to String should not fail");
    fs::write(&imports_index_path, imports_index).map_err(|error| {
        format!(
            "failed to write project imports index `{}`: {error}",
            imports_index_path.display()
        )
    })?;
    let mut galaxy_index = String::new();
    let docs_summary = project_docs_summary(project);
    let imports_summary = super::rendering::project_imports_summary(project);
    let galaxy_summary = project_galaxy_summary(project);
    writeln!(
        &mut galaxy_index,
        "summary\tgalaxies={}\tdocumented_galaxies={}\tdocumented_library_modules={}\tdocumented_items={}",
        galaxy_summary.galaxies,
        galaxy_summary.documented_galaxies,
        galaxy_summary.documented_library_modules,
        galaxy_summary.documented_items
    )
    .expect("writing resolved galaxy index summary to String should not fail");
    crate::stdlib_registry::write_resolved_galaxy_index(
        &mut galaxy_index,
        &project.resolved_galaxies,
    )
    .expect("writing resolved galaxy index to String should not fail");
    fs::write(&galaxy_index_path, galaxy_index).map_err(|error| {
        format!(
            "failed to write project galaxy index `{}`: {error}",
            galaxy_index_path.display()
        )
    })?;
    let mut links_index = String::new();
    write_project_links_index(&mut links_index, &organization)
        .expect("writing project links index to String should not fail");
    fs::write(&links_index_path, links_index).map_err(|error| {
        format!(
            "failed to write project links index `{}`: {error}",
            links_index_path.display()
        )
    })?;
    let mut packet_index = String::new();
    packet::write_project_packet_index(&mut packet_index, project)
        .expect("writing project packet index to String should not fail");
    fs::write(&packet_index_path, packet_index).map_err(|error| {
        format!(
            "failed to write project packet index `{}`: {error}",
            packet_index_path.display()
        )
    })?;
    let mut host_ffi_index = String::new();
    write_project_host_ffi_index(&mut host_ffi_index, project)
        .expect("writing project host ffi index to String should not fail");
    fs::write(&host_ffi_index_path, host_ffi_index).map_err(|error| {
        format!(
            "failed to write project host ffi index `{}`: {error}",
            host_ffi_index_path.display()
        )
    })?;
    let mut abi_index = String::new();
    write_project_abi_index(&mut abi_index, project)?;
    fs::write(&abi_index_path, abi_index).map_err(|error| {
        format!(
            "failed to write project abi index `{}`: {error}",
            abi_index_path.display()
        )
    })?;
    Ok(ProjectBuildMetadata {
        manifest_copy_path: manifest_copy_path.display().to_string(),
        plan_index_path: plan_index_path.display().to_string(),
        organization_index_path: organization_index_path.display().to_string(),
        exchange_index_path: exchange_index_path.display().to_string(),
        modules_index_path: modules_index_path.display().to_string(),
        docs_index_path: docs_index_path.display().to_string(),
        docs_summary,
        imports_index_path: imports_index_path.display().to_string(),
        imports_summary,
        galaxy_index_path: galaxy_index_path.display().to_string(),
        galaxy_summary,
        links_index_path: links_index_path.display().to_string(),
        packet_index_path: packet_index_path.display().to_string(),
        host_ffi_index_path: host_ffi_index_path.display().to_string(),
        abi_index_path: abi_index_path.display().to_string(),
    })
}
