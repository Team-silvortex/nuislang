use super::*;

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
    let imports_summary = crate::project::project_imports_summary(project);
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
