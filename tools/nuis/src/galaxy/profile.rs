use super::*;

pub fn inspect_ns_nova_profile(input: &Path) -> Result<Option<NsNovaProfileSummary>, String> {
    let project = nuisc::project::load_project(input)?;
    let path = project.root.join("ns-nova.toml");
    if !path.exists() {
        return Ok(None);
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let profile = parse_ns_nova_manifest(&source, &path)?;
    Ok(Some(NsNovaProfileSummary {
        path,
        framework_schema: profile.framework_schema,
        framework: profile.framework,
        stdlib_schema: profile.stdlib_schema,
        stdlib_manifest: profile.stdlib_manifest,
        stdlib_sources: profile.stdlib_sources,
        family_schema: profile.family_schema,
        family_layers: profile.family_layers,
        render_schema: profile.render_schema,
        render_owner_unit: profile.render_owner_unit,
        render_bridge_unit: profile.render_bridge_unit,
        render_surface_unit: profile.render_surface_unit,
        selection_schema: profile.selection_schema,
        selection_owner_unit: profile.selection_owner_unit,
        selection_bridge_unit: profile.selection_bridge_unit,
        selection_render_unit: profile.selection_render_unit,
        selection_controls: profile.selection_controls,
    }))
}

pub fn inspect_ns_nova_stdlib(root: &Path) -> Result<Option<NsNovaStdlibSummary>, String> {
    let path = root.join("stdlib/ns-nova/module.toml");
    if !path.exists() {
        return Ok(None);
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let modules = parse_optional_string_array(&source, "source_modules").unwrap_or_default();
    let mut source_modules = Vec::new();
    let mut missing_modules = Vec::new();
    for item in modules {
        let module_path = root.join("stdlib/ns-nova").join(item);
        if !module_path.exists() {
            missing_modules.push(module_path.clone());
        }
        source_modules.push(module_path);
    }
    Ok(Some(NsNovaStdlibSummary {
        path,
        source_modules,
        missing_modules,
    }))
}

pub(super) fn default_ns_nova_manifest(project: &nuisc::project::LoadedProject) -> NsNovaManifest {
    let mut cpu_units = Vec::new();
    let mut data_units = Vec::new();
    let mut shader_units = Vec::new();
    let mut kernel_units = Vec::new();
    let mut entry_cpu_unit = None;
    for module in &project.modules {
        let entry = module.ast.unit.clone();
        match module.ast.domain.as_str() {
            "cpu" => {
                if module.path == project.entry_path {
                    entry_cpu_unit = Some(format!("cpu.{}", module.ast.unit));
                }
                cpu_units.push(entry)
            }
            "data" => data_units.push(entry),
            "shader" => shader_units.push(entry),
            "kernel" => kernel_units.push(entry),
            _ => {}
        }
    }
    cpu_units.sort();
    data_units.sort();
    shader_units.sort();
    kernel_units.sort();
    let primary_data_unit = project
        .manifest
        .links
        .iter()
        .find_map(|link| link.via.as_ref().map(|via| via.trim().to_owned()));
    let primary_shader_unit = project
        .manifest
        .links
        .iter()
        .find_map(|link| link.from.starts_with("shader.").then(|| link.from.clone()))
        .or_else(|| {
            project
                .manifest
                .links
                .iter()
                .find_map(|link| link.to.starts_with("shader.").then(|| link.to.clone()))
        });
    let primary_kernel_unit = project
        .manifest
        .links
        .iter()
        .find_map(|link| link.from.starts_with("kernel.").then(|| link.from.clone()))
        .or_else(|| {
            project
                .manifest
                .links
                .iter()
                .find_map(|link| link.to.starts_with("kernel.").then(|| link.to.clone()))
        });
    let render_links = project
        .manifest
        .links
        .iter()
        .map(|link| match &link.via {
            Some(via) => format!("{} -> {} via {}", link.from, link.to, via),
            None => format!("{} -> {}", link.from, link.to),
        })
        .collect::<Vec<_>>();
    let selection_controls = vec![
        "list".to_owned(),
        "table".to_owned(),
        "tree".to_owned(),
        "inspector".to_owned(),
        "outline".to_owned(),
    ];
    let stdlib_sources = inspect_ns_nova_stdlib(std::path::Path::new("."))
        .ok()
        .flatten()
        .map(|summary| {
            let stdlib_root = std::path::Path::new(".").join("stdlib/ns-nova");
            summary
                .source_modules
                .into_iter()
                .filter_map(|path| {
                    path.strip_prefix(&stdlib_root)
                        .ok()
                        .map(|relative| relative.display().to_string())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    NsNovaManifest {
        framework_schema: "ns-nova-manifest-v1".to_owned(),
        framework: "ns-nova".to_owned(),
        project: "nuis.toml".to_owned(),
        stdlib_schema: Some("ns-nova-stdlib-v1".to_owned()),
        stdlib_manifest: Some("stdlib/ns-nova/module.toml".to_owned()),
        stdlib_sources,
        family_schema: Some("ns-nova-family-v1".to_owned()),
        family_layers: if primary_shader_unit.is_some() {
            vec!["core".to_owned(), "ui".to_owned()]
        } else {
            vec!["core".to_owned()]
        },
        entry_cpu_unit,
        primary_data_unit: primary_data_unit.clone(),
        primary_shader_unit: primary_shader_unit.clone(),
        primary_kernel_unit,
        render_links,
        render_schema: Some("ns-nova-render-v1".to_owned()),
        render_owner_unit: project.entry_path.file_name().and_then(|_| {
            project
                .modules
                .iter()
                .find(|module| module.path == project.entry_path)
                .map(|module| format!("cpu.{}", module.ast.unit))
        }),
        render_bridge_unit: primary_data_unit.clone(),
        render_surface_unit: primary_shader_unit.clone(),
        selection_schema: Some("ns-nova-selection-v1".to_owned()),
        selection_owner_unit: project.entry_path.file_name().and_then(|_| {
            project
                .modules
                .iter()
                .find(|module| module.path == project.entry_path)
                .map(|module| format!("cpu.{}", module.ast.unit))
        }),
        selection_bridge_unit: primary_data_unit.clone(),
        selection_render_unit: primary_shader_unit.clone(),
        selection_controls,
        cpu_units,
        data_units,
        shader_units,
        kernel_units,
    }
}
