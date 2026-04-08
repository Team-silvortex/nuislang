use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use nuis_semantics::model::{
    AstExpr, AstExternFunction, AstModule, AstStmt, AstTypeRef, NirExpr, NirModule, NirStmt,
};
use yir_core::{EdgeKind, Node, Operation, Resource, ResourceKind, YirModule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NuisProjectManifest {
    pub name: String,
    pub entry: String,
    pub modules: Vec<String>,
    pub links: Vec<ProjectLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLink {
    pub from: String,
    pub to: String,
    pub via: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectModule {
    pub path: PathBuf,
    pub ast: AstModule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProject {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: NuisProjectManifest,
    pub entry_path: PathBuf,
    pub entry_source: String,
    pub modules: Vec<ProjectModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBuildMetadata {
    pub manifest_copy_path: String,
    pub modules_index_path: String,
    pub links_index_path: String,
    pub host_ffi_index_path: String,
}

pub fn is_project_input(path: &Path) -> bool {
    path.is_dir() || path.file_name().and_then(|name| name.to_str()) == Some("nuis.toml")
}

pub fn load_project(input: &Path) -> Result<LoadedProject, String> {
    let manifest_path = if input.is_dir() {
        input.join("nuis.toml")
    } else {
        input.to_path_buf()
    };
    let root = manifest_path
        .parent()
        .ok_or_else(|| format!("project manifest `{}` has no parent directory", manifest_path.display()))?
        .to_path_buf();
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest = parse_project_manifest(&source, &manifest_path)?;

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
        modules.push(ProjectModule { path, ast });
    }

    let entry_path = root.join(&manifest.entry);
    let entry_source = fs::read_to_string(&entry_path)
        .map_err(|error| format!("failed to read `{}`: {error}", entry_path.display()))?;

    validate_project_modules(&modules)?;
    validate_project_unit_bindings(&modules)?;
    validate_project_uses(&modules)?;
    validate_project_links(&manifest, &modules)?;

    Ok(LoadedProject {
        root,
        manifest_path,
        manifest,
        entry_path,
        entry_source,
        modules,
    })
}

pub fn describe_project(project: &LoadedProject) -> String {
    let modules = project
        .modules
        .iter()
        .map(|module| {
            let relative = module
                .path
                .strip_prefix(&project.root)
                .unwrap_or(module.path.as_path());
            format!(
                "{} (mod {} {})",
                relative.display(),
                module.ast.domain,
                module.ast.unit
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let links = if project.manifest.links.is_empty() {
        "<none>".to_owned()
    } else {
        project
            .manifest
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
    format!(
        "project={} entry={} modules={} links={}",
        project.manifest.name, project.manifest.entry, modules, links
    )
}

pub fn write_project_metadata(
    output_dir: &Path,
    project: &LoadedProject,
) -> Result<ProjectBuildMetadata, String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
    let manifest_copy_path = output_dir.join("nuis.project.toml");
    let modules_index_path = output_dir.join("nuis.project.modules.txt");
    let links_index_path = output_dir.join("nuis.project.links.txt");
    let host_ffi_index_path = output_dir.join("nuis.project.host_ffi.txt");
    fs::copy(&project.manifest_path, &manifest_copy_path).map_err(|error| {
        format!(
            "failed to copy project manifest `{}` -> `{}`: {error}",
            project.manifest_path.display(),
            manifest_copy_path.display()
        )
    })?;
    let modules_index = project
        .modules
        .iter()
        .map(|module| {
            let relative = module
                .path
                .strip_prefix(&project.root)
                .unwrap_or(module.path.as_path());
            format!(
                "{}\tmod {} {}\n",
                relative.display(),
                module.ast.domain,
                module.ast.unit
            )
        })
        .collect::<String>();
    fs::write(&modules_index_path, modules_index).map_err(|error| {
        format!(
            "failed to write project modules index `{}`: {error}",
            modules_index_path.display()
        )
    })?;
    let links_index = if project.manifest.links.is_empty() {
        String::new()
    } else {
        project
            .manifest
            .links
            .iter()
            .map(|link| {
                if let Some(via) = &link.via {
                    format!("{}\t{}\t{}\n", link.from, link.to, via)
                } else {
                    format!("{}\t{}\t<direct>\n", link.from, link.to)
                }
            })
            .collect::<String>()
    };
    fs::write(&links_index_path, links_index).map_err(|error| {
        format!(
            "failed to write project links index `{}`: {error}",
            links_index_path.display()
        )
    })?;
    let host_ffi_index = render_project_host_ffi_index(project);
    fs::write(&host_ffi_index_path, host_ffi_index).map_err(|error| {
        format!(
            "failed to write project host ffi index `{}`: {error}",
            host_ffi_index_path.display()
        )
    })?;
    Ok(ProjectBuildMetadata {
        manifest_copy_path: manifest_copy_path.display().to_string(),
        modules_index_path: modules_index_path.display().to_string(),
        links_index_path: links_index_path.display().to_string(),
        host_ffi_index_path: host_ffi_index_path.display().to_string(),
    })
}

fn render_project_host_ffi_index(project: &LoadedProject) -> String {
    let mut lines = Vec::new();
    for module in &project.modules {
        let relative = module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(module.path.as_path())
            .display()
            .to_string();

        for function in &module.ast.externs {
            lines.push(format!(
                "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}\tsignature={}",
                relative,
                module.ast.domain,
                module.ast.unit,
                function.abi,
                function.interface.as_deref().unwrap_or("-"),
                function.name,
                render_host_ffi_signature(function),
            ));
        }

        for interface in &module.ast.extern_interfaces {
            for method in &interface.methods {
                lines.push(format!(
                    "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}__{}\tsignature={}",
                    relative,
                    module.ast.domain,
                    module.ast.unit,
                    interface.abi,
                    interface.name,
                    interface.name,
                    method.name,
                    render_host_ffi_signature(method),
                ));
            }
        }
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn render_host_ffi_signature(function: &AstExternFunction) -> String {
    let params = function
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type_ref(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "fn {}({}) -> {}",
        function.name,
        params,
        render_ast_type_ref(&function.return_type)
    )
}

fn render_ast_type_ref(ty: &AstTypeRef) -> String {
    let mut rendered = ty.name.clone();
    if !ty.generic_args.is_empty() {
        rendered.push('<');
        rendered.push_str(
            &ty.generic_args
                .iter()
                .map(render_ast_type_ref)
                .collect::<Vec<_>>()
                .join(", "),
        );
        rendered.push('>');
    }
    if ty.is_optional {
        rendered.push('?');
    }
    if ty.is_ref {
        format!("ref {rendered}")
    } else {
        rendered
    }
}

pub fn apply_project_links_to_yir(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    let mut required = BTreeSet::new();
    for link in &project.manifest.links {
        if !link.from.starts_with("cpu.") {
            continue;
        }
        let (target_domain, target_unit) = split_domain_unit(&link.to)?;
        required.insert((target_domain, target_unit));
        if let Some(via) = &link.via {
            let (via_domain, via_unit) = split_domain_unit(via)?;
            required.insert((via_domain, via_unit));
        }
    }

    for (domain, unit) in required {
        let exists = module.nodes.iter().any(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "instantiate_unit"
                && node.op.args.first().map(String::as_str) == Some(domain.as_str())
                && node.op.args.get(1).map(String::as_str) == Some(unit.as_str())
        });
        if exists {
            continue;
        }
        let name = format!(
            "project_link_instantiate_{}_{}",
            sanitize_ident(&domain),
            sanitize_ident(&unit)
        );
        module.nodes.push(Node {
            name,
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "instantiate_unit".to_owned(),
                args: vec![domain, unit],
            },
        });
    }

    Ok(())
}

pub fn apply_project_support_modules_to_yir(
    project: &LoadedProject,
    module: &mut YirModule,
) -> Result<(), String> {
    for project_module in &project.modules {
        if project_module.path == project.entry_path {
            continue;
        }
        apply_support_module_profile(&project_module.ast, module)?;
    }
    resolve_project_profile_refs(module)?;
    stitch_shader_profile_edges(module);
    stitch_data_profile_edges(module);
    Ok(())
}

pub fn validate_project_links_against_yir(
    project: &LoadedProject,
    module: &YirModule,
) -> Result<(), String> {
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_families = module
        .nodes
        .iter()
        .map(|node| {
            let family = resource_families
                .get(node.resource.as_str())
                .cloned()
                .unwrap_or_else(|| node.op.module.clone());
            (node.name.as_str(), family)
        })
        .collect::<BTreeMap<_, _>>();

    for link in &project.manifest.links {
        if let Some(via) = &link.via {
            let (via_domain, _via_unit) = split_domain_unit(via)?;
            if via_domain == "data" {
                let has_fabric = module
                    .resources
                    .iter()
                    .any(|resource| resource.kind.raw == "data.fabric");
                if !has_fabric {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires a `data.fabric` resource in YIR",
                        link.from, link.to, via
                    ));
                }
                let has_data_plane = module.nodes.iter().any(|node| node.op.module == "data");
                if !has_data_plane {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires at least one `data.*` node in YIR",
                        link.from, link.to, via
                    ));
                }
                let has_cross_domain_xfer = module
                    .edges
                    .iter()
                    .any(|edge| edge.kind == EdgeKind::CrossDomainExchange);
                if !has_cross_domain_xfer {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires at least one `xfer` edge in YIR",
                        link.from, link.to, via
                    ));
                }
                let (from_domain, _from_unit) = split_domain_unit(&link.from)?;
                let (to_domain, _to_unit) = split_domain_unit(&link.to)?;
                let has_uplink = has_xfer_segment(module, &node_families, &from_domain, "data");
                if !has_uplink {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires a `{}` -> `data` xfer segment in YIR",
                        link.from, link.to, via, from_domain
                    ));
                }
                let has_downlink = has_xfer_segment(module, &node_families, "data", &to_domain);
                if !has_downlink {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires a `data` -> `{}` xfer segment in YIR",
                        link.from, link.to, via, to_domain
                    ));
                }
                validate_data_profile_for_link(module, via)?;
            }
        }

        validate_shader_profile_for_link(module, &link.from)?;
        validate_shader_profile_for_link(module, &link.to)?;
        validate_kernel_profile_for_link(module, &link.from)?;
        validate_kernel_profile_for_link(module, &link.to)?;
    }
    Ok(())
}

pub fn validate_project_links_against_nir(
    project: &LoadedProject,
    module: &NirModule,
) -> Result<(), String> {
    let mut support_surface_cache = BTreeMap::<String, BTreeSet<String>>::new();
    for link in &project.manifest.links {
        let (from_domain, _from_unit) = split_domain_unit(&link.from)?;
        let (to_domain, to_unit) = split_domain_unit(&link.to)?;
        if from_domain == "cpu" && to_domain == "shader" {
            let shader_support = support_surface_for_domain(&mut support_surface_cache, "shader")?;
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.packet.v1",
            )?;
            if !nir_uses_shader_profile_packet(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_packet(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.render.v1",
            )?;
            if !nir_uses_shader_profile_render(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_render(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.seed.color.v1",
            )?;
            if !nir_uses_shader_profile_color_seed(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_color_seed(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.seed.speed.v1",
            )?;
            if !nir_uses_shader_profile_speed_seed(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_speed_seed(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &shader_support,
                "shader",
                &to_unit,
                "shader.profile.seed.radius.v1",
            )?;
            if !nir_uses_shader_profile_radius_seed(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use shader_profile_radius_seed(\"{}\", ...) at NIR level",
                    link.from, link.to, to_unit
                ));
            }
        }
        if from_domain == "cpu" && to_domain == "kernel" {
            let kernel_support = support_surface_for_domain(&mut support_surface_cache, "kernel")?;
            require_declared_support_surface(
                &kernel_support,
                "kernel",
                &to_unit,
                "kernel.profile.bind-core.v1",
            )?;
            if !nir_uses_kernel_profile_bind_core(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use kernel_profile_bind_core(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &kernel_support,
                "kernel",
                &to_unit,
                "kernel.profile.queue-depth.v1",
            )?;
            if !nir_uses_kernel_profile_queue_depth(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use kernel_profile_queue_depth(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
            require_declared_support_surface(
                &kernel_support,
                "kernel",
                &to_unit,
                "kernel.profile.batch-lanes.v1",
            )?;
            if !nir_uses_kernel_profile_batch_lanes(module, &to_unit) {
                return Err(format!(
                    "project link `{}` -> `{}` requires CPU entry to use kernel_profile_batch_lanes(\"{}\") at NIR level",
                    link.from, link.to, to_unit
                ));
            }
        }
        if let Some(via) = &link.via {
            let (via_domain, via_unit) = split_domain_unit(via)?;
            if via_domain == "data" {
                let data_support = support_surface_for_domain(&mut support_surface_cache, "data")?;
                require_declared_support_surface(
                    &data_support,
                    "data",
                    &via_unit,
                    "data.profile.bind-core.v1",
                )?;
                if !nir_uses_data_profile_bind_core(module, &via_unit) {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires CPU entry to use data_profile_bind_core(\"{}\") at NIR level",
                        link.from, link.to, via, via_unit
                    ));
                }
                if !nir_uses_data_profile_handle_table(module, &via_unit) {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires CPU entry to use data_profile_handle_table(\"{}\") at NIR level",
                        link.from, link.to, via, via_unit
                    ));
                }
                require_declared_support_surface(
                    &data_support,
                    "data",
                    &via_unit,
                    "data.profile.send.uplink.v1",
                )?;
                if !nir_uses_data_profile_send_uplink(module, &via_unit) {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires CPU entry to use data_profile_send_uplink(\"{}\") at NIR level",
                        link.from, link.to, via, via_unit
                    ));
                }
                require_declared_support_surface(
                    &data_support,
                    "data",
                    &via_unit,
                    "data.profile.send.downlink.v1",
                )?;
                if !nir_uses_data_profile_send_downlink(module, &via_unit) {
                    return Err(format!(
                        "project link `{}` -> `{}` via `{}` requires CPU entry to use data_profile_send_downlink(\"{}\") at NIR level",
                        link.from, link.to, via, via_unit
                    ));
                }
            }
        }
    }
    Ok(())
}

fn support_surface_for_domain(
    cache: &mut BTreeMap<String, BTreeSet<String>>,
    domain: &str,
) -> Result<BTreeSet<String>, String> {
    if let Some(surface) = cache.get(domain) {
        return Ok(surface.clone());
    }
    let manifest =
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), domain)?;
    let surface = manifest.support_surface.into_iter().collect::<BTreeSet<_>>();
    cache.insert(domain.to_owned(), surface.clone());
    Ok(surface)
}

fn require_declared_support_surface(
    declared_surface: &BTreeSet<String>,
    domain: &str,
    unit: &str,
    required_surface: &str,
) -> Result<(), String> {
    if declared_surface.contains(required_surface) {
        return Ok(());
    }
    Err(format!(
        "project {} unit `{}.{}` requires nustar to declare support surface `{}`",
        domain, domain, unit, required_surface
    ))
}

fn support_profile_slots_for_domain(domain: &str) -> Result<BTreeSet<String>, String> {
    let manifest =
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), domain)?;
    Ok(manifest
        .support_profile_slots
        .into_iter()
        .collect::<BTreeSet<_>>())
}

fn require_declared_profile_slot(
    declared_slots: &BTreeSet<String>,
    domain: &str,
    unit: &str,
    required_slot: &str,
) -> Result<(), String> {
    if declared_slots.contains(required_slot) {
        return Ok(());
    }
    Err(format!(
        "project {} unit `{}.{}` requires nustar to declare profile slot `{}`",
        domain, domain, unit, required_slot
    ))
}

fn validate_shader_profile_for_link(module: &YirModule, endpoint: &str) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "shader" {
        return Ok(());
    }
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "shader")?;
    let declared_slots = support_profile_slots_for_domain("shader")?;
    for required_surface in shader_support_surface_contract() {
        require_declared_support_surface(&declared_support, "shader", &unit, required_surface)?;
    }

    for (slot, node_name) in shader_profile_slot_targets(&unit) {
        require_declared_profile_slot(&declared_slots, "shader", &unit, slot)?;
        let exists = module.nodes.iter().any(|node| node.name == node_name);
        if !exists {
            return Err(format!(
                "project shader unit `shader.{}` requires support profile slot `{}` in YIR",
                unit, slot
            ));
        }
    }

    validate_shader_profile_flow(module, &unit)?;

    Ok(())
}

fn validate_kernel_profile_for_link(module: &YirModule, endpoint: &str) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "kernel" {
        return Ok(());
    }
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "kernel")?;
    let declared_slots = support_profile_slots_for_domain("kernel")?;
    for required_surface in kernel_support_surface_contract() {
        require_declared_support_surface(&declared_support, "kernel", &unit, required_surface)?;
    }

    for (slot, node_name) in kernel_profile_slot_targets(&unit) {
        require_declared_profile_slot(&declared_slots, "kernel", &unit, slot)?;
        let exists = module.nodes.iter().any(|node| node.name == node_name);
        if !exists {
            return Err(format!(
                "project kernel unit `kernel.{}` requires support profile slot `{}` in YIR",
                unit, slot
            ));
        }
    }

    let has_kernel_work = module.nodes.iter().any(|node| node.op.module == "kernel");
    if !has_kernel_work {
        return Err(format!(
            "project kernel unit `kernel.{}` requires at least one kernel.* node in YIR",
            unit
        ));
    }

    Ok(())
}

fn nir_uses_shader_profile_render(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_shader_profile_render(stmt, unit)))
}

fn nir_uses_shader_profile_packet(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_shader_profile_packet(stmt, unit)))
}

fn nir_uses_shader_profile_color_seed(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_shader_profile_color_seed(stmt, unit)))
}

fn nir_uses_shader_profile_speed_seed(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_shader_profile_speed_seed(stmt, unit)))
}

fn nir_uses_shader_profile_radius_seed(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_shader_profile_radius_seed(stmt, unit)))
}

fn nir_uses_data_profile_bind_core(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_data_profile_bind_core(stmt, unit)))
}

fn nir_uses_data_profile_handle_table(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_data_profile_handle_table(stmt, unit)))
}

fn nir_uses_data_profile_send_uplink(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_data_profile_send_uplink(stmt, unit)))
}

fn nir_uses_data_profile_send_downlink(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_data_profile_send_downlink(stmt, unit)))
}

fn nir_uses_kernel_profile_bind_core(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_kernel_profile_bind_core(stmt, unit)))
}

fn nir_uses_kernel_profile_queue_depth(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_kernel_profile_queue_depth(stmt, unit)))
}

fn nir_uses_kernel_profile_batch_lanes(module: &NirModule, unit: &str) -> bool {
    module
        .functions
        .iter()
        .any(|function| function.body.iter().any(|stmt| stmt_uses_kernel_profile_batch_lanes(stmt, unit)))
}

fn stmt_uses_shader_profile_render(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_shader_profile_render(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_shader_profile_render(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_render(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_render(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_shader_profile_render(value, unit)),
    }
}

fn stmt_uses_shader_profile_packet(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_shader_profile_packet(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_shader_profile_packet(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_packet(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_packet(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_shader_profile_packet(value, unit)),
    }
}

fn stmt_uses_shader_profile_color_seed(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_shader_profile_color_seed(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_shader_profile_color_seed(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_color_seed(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_color_seed(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_shader_profile_color_seed(value, unit)),
    }
}

fn stmt_uses_shader_profile_speed_seed(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_shader_profile_speed_seed(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_shader_profile_speed_seed(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_speed_seed(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_speed_seed(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_shader_profile_speed_seed(value, unit)),
    }
}

fn stmt_uses_shader_profile_radius_seed(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_shader_profile_radius_seed(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_shader_profile_radius_seed(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_radius_seed(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_shader_profile_radius_seed(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_shader_profile_radius_seed(value, unit)),
    }
}

fn stmt_uses_data_profile_bind_core(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_data_profile_bind_core(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_data_profile_bind_core(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_bind_core(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_bind_core(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_data_profile_bind_core(value, unit)),
    }
}

fn stmt_uses_data_profile_handle_table(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_data_profile_handle_table(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_data_profile_handle_table(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_handle_table(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_handle_table(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_data_profile_handle_table(value, unit)),
    }
}

fn stmt_uses_data_profile_send_uplink(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_data_profile_send_uplink(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_data_profile_send_uplink(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_send_uplink(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_send_uplink(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_data_profile_send_uplink(value, unit)),
    }
}

fn stmt_uses_data_profile_send_downlink(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_data_profile_send_downlink(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_data_profile_send_downlink(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_send_downlink(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_data_profile_send_downlink(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_data_profile_send_downlink(value, unit)),
    }
}

fn stmt_uses_kernel_profile_bind_core(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_kernel_profile_bind_core(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_kernel_profile_bind_core(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_kernel_profile_bind_core(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_kernel_profile_bind_core(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_kernel_profile_bind_core(value, unit)),
    }
}

fn stmt_uses_kernel_profile_queue_depth(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_kernel_profile_queue_depth(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_kernel_profile_queue_depth(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_kernel_profile_queue_depth(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_kernel_profile_queue_depth(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_kernel_profile_queue_depth(value, unit)),
    }
}

fn stmt_uses_kernel_profile_batch_lanes(stmt: &NirStmt, unit: &str) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value) => expr_uses_kernel_profile_batch_lanes(value, unit),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_uses_kernel_profile_batch_lanes(condition, unit)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_kernel_profile_batch_lanes(stmt, unit))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_kernel_profile_batch_lanes(stmt, unit))
        }
        NirStmt::Return(value) => value
            .as_ref()
            .is_some_and(|value| expr_uses_kernel_profile_batch_lanes(value, unit)),
    }
}

fn expr_uses_shader_profile_render(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileRender {
            unit: shader_unit,
            packet,
        } => shader_unit == unit || expr_uses_shader_profile_render(packet, unit),
        NirExpr::Borrow(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => expr_uses_shader_profile_render(inner, unit),
        NirExpr::AllocNode { value, next } => {
            expr_uses_shader_profile_render(value, unit)
                || expr_uses_shader_profile_render(next, unit)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_uses_shader_profile_render(len, unit)
                || expr_uses_shader_profile_render(fill, unit)
        }
        NirExpr::LoadAt { buffer, index } => {
            expr_uses_shader_profile_render(buffer, unit)
                || expr_uses_shader_profile_render(index, unit)
        }
        NirExpr::StoreValue { target, value } => {
            expr_uses_shader_profile_render(target, unit)
                || expr_uses_shader_profile_render(value, unit)
        }
        NirExpr::StoreNext { target, next } => {
            expr_uses_shader_profile_render(target, unit)
                || expr_uses_shader_profile_render(next, unit)
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            expr_uses_shader_profile_render(buffer, unit)
                || expr_uses_shader_profile_render(index, unit)
                || expr_uses_shader_profile_render(value, unit)
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            expr_uses_shader_profile_render(input, unit)
                || expr_uses_shader_profile_render(offset, unit)
                || expr_uses_shader_profile_render(len, unit)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileColorSeed { base: input, .. }
        | NirExpr::ShaderProfileRadiusSeed { base: input, .. } => {
            expr_uses_shader_profile_render(input, unit)
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta,
            scale,
            base,
            ..
        } => {
            expr_uses_shader_profile_render(delta, unit)
                || expr_uses_shader_profile_render(scale, unit)
                || expr_uses_shader_profile_render(base, unit)
        }
        NirExpr::CpuExternCall { args, .. } | NirExpr::Call { args, .. } => args
            .iter()
            .any(|arg| expr_uses_shader_profile_render(arg, unit)),
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_uses_shader_profile_render(receiver, unit)
                || args
                    .iter()
                    .any(|arg| expr_uses_shader_profile_render(arg, unit))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_uses_shader_profile_render(value, unit)),
        NirExpr::FieldAccess { base, .. } => expr_uses_shader_profile_render(base, unit),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_uses_shader_profile_render(lhs, unit)
                || expr_uses_shader_profile_render(rhs, unit)
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            expr_uses_shader_profile_render(target, unit)
                || expr_uses_shader_profile_render(pipeline, unit)
                || expr_uses_shader_profile_render(viewport, unit)
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            expr_uses_shader_profile_render(pass, unit)
                || expr_uses_shader_profile_render(packet, unit)
                || expr_uses_shader_profile_render(vertex_count, unit)
                || expr_uses_shader_profile_render(instance_count, unit)
        }
        _ => false,
    }
}

fn expr_uses_shader_profile_packet(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfilePacket { unit: shader_unit, .. } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_shader_profile_packet(inner, unit)),
    }
}

fn expr_uses_shader_profile_color_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileColorSeed { unit: shader_unit, .. } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_shader_profile_color_seed(inner, unit)),
    }
}

fn expr_uses_shader_profile_speed_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileSpeedSeed { unit: shader_unit, .. } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_shader_profile_speed_seed(inner, unit)),
    }
}

fn expr_uses_shader_profile_radius_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileRadiusSeed { unit: shader_unit, .. } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_shader_profile_radius_seed(inner, unit)),
    }
}

fn expr_uses_data_profile_bind_core(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileBindCoreRef { unit: data_unit } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_data_profile_bind_core(inner, unit)),
    }
}

fn expr_uses_data_profile_handle_table(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileHandleTableRef { unit: data_unit } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_data_profile_handle_table(inner, unit)),
    }
}

fn expr_uses_data_profile_send_uplink(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileSendUplink { unit: data_unit, .. } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_data_profile_send_uplink(inner, unit)),
    }
}

fn expr_uses_data_profile_send_downlink(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileSendDownlink { unit: data_unit, .. } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_data_profile_send_downlink(inner, unit)),
    }
}

fn expr_uses_kernel_profile_bind_core(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::KernelProfileBindCoreRef { unit: kernel_unit } => kernel_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_kernel_profile_bind_core(inner, unit)),
    }
}

fn expr_uses_kernel_profile_queue_depth(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::KernelProfileQueueDepthRef { unit: kernel_unit } => kernel_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_kernel_profile_queue_depth(inner, unit)),
    }
}

fn expr_uses_kernel_profile_batch_lanes(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::KernelProfileBatchLanesRef { unit: kernel_unit } => kernel_unit == unit,
        _ => expr_walk_any(expr, &|inner| expr_uses_kernel_profile_batch_lanes(inner, unit)),
    }
}

fn expr_walk_any(expr: &NirExpr, predicate: &dyn Fn(&NirExpr) -> bool) -> bool {
    match expr {
        NirExpr::Borrow(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::FieldAccess { base: inner, .. } => predicate(inner),
        NirExpr::AllocNode { value, next } => predicate(value) || predicate(next),
        NirExpr::AllocBuffer { len, fill } => predicate(len) || predicate(fill),
        NirExpr::LoadAt { buffer, index } => predicate(buffer) || predicate(index),
        NirExpr::StoreValue { target, value } => predicate(target) || predicate(value),
        NirExpr::StoreNext { target, next } => predicate(target) || predicate(next),
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => predicate(buffer) || predicate(index) || predicate(value),
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            predicate(input) || predicate(offset) || predicate(len)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => predicate(input),
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            predicate(base) || predicate(delta)
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta,
            scale,
            base,
            ..
        } => predicate(delta) || predicate(scale) || predicate(base),
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => predicate(color) || predicate(speed) || predicate(radius),
        NirExpr::CpuExternCall { args, .. } | NirExpr::Call { args, .. } => {
            args.iter().any(predicate)
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            predicate(receiver) || args.iter().any(predicate)
        }
        NirExpr::StructLiteral { fields, .. } => fields.iter().any(|(_, value)| predicate(value)),
        NirExpr::Binary { lhs, rhs, .. } => predicate(lhs) || predicate(rhs),
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => predicate(target) || predicate(pipeline) || predicate(viewport),
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            predicate(pass)
                || predicate(packet)
                || predicate(vertex_count)
                || predicate(instance_count)
        }
        NirExpr::ShaderProfileRender { packet, .. } => predicate(packet),
        _ => false,
    }
}

fn validate_shader_profile_flow(module: &YirModule, unit: &str) -> Result<(), String> {
    let target = resolve_project_profile_target_name("shader", unit, "target");
    let viewport = resolve_project_profile_target_name("shader", unit, "viewport");
    let pipeline = resolve_project_profile_target_name("shader", unit, "pipeline");
    let vertex_count = resolve_project_profile_target_name("shader", unit, "vertex_count");
    let instance_count = resolve_project_profile_target_name("shader", unit, "instance_count");
    let pass_kind = resolve_project_profile_target_name("shader", unit, "pass_kind");
    let packet_field_count =
        resolve_project_profile_target_name("shader", unit, "packet_field_count");

    let begin_passes = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "shader" && node.op.instruction == "begin_pass")
        .map(|node| node.name.as_str())
        .collect::<Vec<_>>();
    let begin_pass_wired = begin_passes.iter().any(|pass| {
        has_edge_to(module, &target, pass)
            && has_edge_to(module, &viewport, pass)
            && has_edge_to(module, &pipeline, pass)
            && has_edge_to(module, &pass_kind, pass)
    });
    if !begin_pass_wired {
        return Err(format!(
            "project shader unit `shader.{}` requires target/viewport/pipeline/pass_kind profile nodes to feed a shader.begin_pass node",
            unit
        ));
    }

    let draws = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "shader" && node.op.instruction == "draw_instanced")
        .map(|node| node.name.as_str())
        .collect::<Vec<_>>();
    let draw_wired = draws.iter().any(|draw| {
        has_edge_to(module, &vertex_count, draw)
            && has_edge_to(module, &instance_count, draw)
            && has_edge_to(module, &packet_field_count, draw)
    });
    if !draw_wired {
        return Err(format!(
            "project shader unit `shader.{}` requires vertex_count/instance_count/packet_field_count profile nodes to feed a shader.draw_instanced node",
            unit
        ));
    }

    Ok(())
}

fn validate_data_profile_for_link(module: &YirModule, endpoint: &str) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(endpoint)?;
    if domain != "data" {
        return Ok(());
    }
    let declared_support = support_surface_for_domain(&mut BTreeMap::new(), "data")?;
    let declared_slots = support_profile_slots_for_domain("data")?;
    for required_surface in data_support_surface_contract() {
        require_declared_support_surface(&declared_support, "data", &unit, required_surface)?;
    }

    for (slot, node_name) in data_profile_slot_targets(&unit) {
        require_declared_profile_slot(&declared_slots, "data", &unit, slot)?;
        let exists = module.nodes.iter().any(|node| node.name == node_name);
        if !exists {
            return Err(format!(
                "project data unit `data.{}` requires support profile slot `{}` in YIR",
                unit, slot
            ));
        }
    }

    let uplink_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "data" && matches!(node.op.instruction.as_str(), "output_pipe" | "input_pipe"))
        .take(2)
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "data" && matches!(node.op.instruction.as_str(), "output_pipe" | "input_pipe"))
        .skip(2)
        .take(2)
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_payload = resolve_project_profile_target_name("data", &unit, "marker:uplink_payload_class");
    let downlink_payload = resolve_project_profile_target_name("data", &unit, "marker:downlink_payload_class");
    let uplink_shape = resolve_project_profile_target_name("data", &unit, "marker:uplink_payload_shape");
    let downlink_shape = resolve_project_profile_target_name("data", &unit, "marker:downlink_payload_shape");
    let uplink_windows = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && matches!(node.op.instruction.as_str(), "copy_window" | "immutable_window")
                && node.name.contains("_uplink_window")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_windows = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && matches!(node.op.instruction.as_str(), "copy_window" | "immutable_window")
                && node.name.contains("_downlink_window")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    if !uplink_nodes.iter().all(|pipe| has_edge_to(module, &uplink_payload, pipe)) {
        return Err(format!(
            "project data unit `data.{}` requires uplink payload class to feed all uplink pipe nodes",
            unit
        ));
    }
    if !uplink_nodes.iter().all(|pipe| has_edge_to(module, &uplink_shape, pipe)) {
        return Err(format!(
            "project data unit `data.{}` requires uplink payload shape to feed all uplink pipe nodes",
            unit
        ));
    }
    if !uplink_windows.iter().all(|window| has_edge_to(module, &uplink_shape, window)) {
        return Err(format!(
            "project data unit `data.{}` requires uplink payload shape to feed all uplink window nodes",
            unit
        ));
    }
    if !downlink_nodes.iter().all(|pipe| has_edge_to(module, &downlink_payload, pipe)) {
        return Err(format!(
            "project data unit `data.{}` requires downlink payload class to feed all downlink pipe nodes",
            unit
        ));
    }
    if !downlink_nodes.iter().all(|pipe| has_edge_to(module, &downlink_shape, pipe)) {
        return Err(format!(
            "project data unit `data.{}` requires downlink payload shape to feed all downlink pipe nodes",
            unit
        ));
    }
    if !downlink_windows.iter().all(|window| has_edge_to(module, &downlink_shape, window)) {
        return Err(format!(
            "project data unit `data.{}` requires downlink payload shape to feed all downlink window nodes",
            unit
        ));
    }

    Ok(())
}

fn has_edge_to(module: &YirModule, from: &str, to: &str) -> bool {
    module.edges.iter().any(|edge| edge.from == from && edge.to == to)
}

fn shader_support_surface_contract() -> &'static [&'static str] {
    &[
        "shader.profile.packet.v1",
        "shader.profile.target.v1",
        "shader.profile.viewport.v1",
        "shader.profile.pipeline.v1",
        "shader.profile.draw-budget.v1",
        "shader.profile.packet-slots.v1",
        "shader.profile.packet-tag.v1",
        "shader.profile.material-mode.v1",
        "shader.profile.pass-kind.v1",
        "shader.profile.packet-field-count.v1",
    ]
}

fn kernel_support_surface_contract() -> &'static [&'static str] {
    &[
        "kernel.profile.bind-core.v1",
        "kernel.profile.queue-depth.v1",
        "kernel.profile.batch-lanes.v1",
        "kernel.profile.entry.v1",
    ]
}

fn data_support_surface_contract() -> &'static [&'static str] {
    &[
        "data.profile.bind-core.v1",
        "data.profile.handle-table.v1",
        "data.profile.window-layout.v1",
        "data.profile.sync-markers.v1",
        "data.profile.pipe-markers.v1",
        "data.profile.pipe-class.v1",
        "data.profile.payload-class.v1",
        "data.profile.payload-shape.v1",
        "data.profile.window-policy.v1",
    ]
}

fn shader_profile_slot_targets(unit: &str) -> Vec<(&'static str, String)> {
    vec![
        ("target", resolve_project_profile_target_name("shader", unit, "target")),
        (
            "viewport",
            resolve_project_profile_target_name("shader", unit, "viewport"),
        ),
        (
            "pipeline",
            resolve_project_profile_target_name("shader", unit, "pipeline"),
        ),
        (
            "vertex_count",
            resolve_project_profile_target_name("shader", unit, "vertex_count"),
        ),
        (
            "instance_count",
            resolve_project_profile_target_name("shader", unit, "instance_count"),
        ),
        (
            "packet_color_slot",
            resolve_project_profile_target_name("shader", unit, "packet_color_slot"),
        ),
        (
            "packet_speed_slot",
            resolve_project_profile_target_name("shader", unit, "packet_speed_slot"),
        ),
        (
            "packet_radius_slot",
            resolve_project_profile_target_name("shader", unit, "packet_radius_slot"),
        ),
        (
            "packet_tag",
            resolve_project_profile_target_name("shader", unit, "packet_tag"),
        ),
        (
            "material_mode",
            resolve_project_profile_target_name("shader", unit, "material_mode"),
        ),
        (
            "pass_kind",
            resolve_project_profile_target_name("shader", unit, "pass_kind"),
        ),
        (
            "packet_field_count",
            resolve_project_profile_target_name("shader", unit, "packet_field_count"),
        ),
    ]
}

fn kernel_profile_slot_targets(unit: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "bind_core",
            resolve_project_profile_target_name("kernel", unit, "bind_core"),
        ),
        (
            "queue_depth",
            resolve_project_profile_target_name("kernel", unit, "queue_depth"),
        ),
        (
            "batch_lanes",
            resolve_project_profile_target_name("kernel", unit, "batch_lanes"),
        ),
    ]
}

fn data_profile_slot_targets(unit: &str) -> Vec<(&'static str, String)> {
    vec![
        ("bind_core", resolve_project_profile_target_name("data", unit, "bind_core")),
        (
            "window_offset",
            resolve_project_profile_target_name("data", unit, "window_offset"),
        ),
        ("uplink_len", resolve_project_profile_target_name("data", unit, "uplink_len")),
        (
            "downlink_len",
            resolve_project_profile_target_name("data", unit, "downlink_len"),
        ),
        (
            "handle_table",
            resolve_project_profile_target_name("data", unit, "handle_table"),
        ),
        (
            "marker:cpu_to_shader",
            resolve_project_profile_target_name("data", unit, "marker:cpu_to_shader"),
        ),
        (
            "marker:shader_to_cpu",
            resolve_project_profile_target_name("data", unit, "marker:shader_to_cpu"),
        ),
        (
            "marker:uplink_pipe",
            resolve_project_profile_target_name("data", unit, "marker:uplink_pipe"),
        ),
        (
            "marker:downlink_pipe",
            resolve_project_profile_target_name("data", unit, "marker:downlink_pipe"),
        ),
        (
            "marker:uplink_pipe_class",
            resolve_project_profile_target_name("data", unit, "marker:uplink_pipe_class"),
        ),
        (
            "marker:downlink_pipe_class",
            resolve_project_profile_target_name("data", unit, "marker:downlink_pipe_class"),
        ),
        (
            "marker:uplink_payload_class",
            resolve_project_profile_target_name("data", unit, "marker:uplink_payload_class"),
        ),
        (
            "marker:downlink_payload_class",
            resolve_project_profile_target_name("data", unit, "marker:downlink_payload_class"),
        ),
        (
            "marker:uplink_payload_shape",
            resolve_project_profile_target_name("data", unit, "marker:uplink_payload_shape"),
        ),
        (
            "marker:downlink_payload_shape",
            resolve_project_profile_target_name("data", unit, "marker:downlink_payload_shape"),
        ),
        (
            "marker:uplink_window_policy",
            resolve_project_profile_target_name("data", unit, "marker:uplink_window_policy"),
        ),
        (
            "marker:downlink_window_policy",
            resolve_project_profile_target_name("data", unit, "marker:downlink_window_policy"),
        ),
    ]
}

fn stitch_shader_profile_edges(module: &mut YirModule) {
    let pass_kind_nodes = module
        .nodes
        .iter()
        .filter(|node| node.name.contains("_pass_kind"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let packet_field_count_nodes = module
        .nodes
        .iter()
        .filter(|node| node.name.contains("_packet_field_count"))
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let begin_pass_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "shader" && node.op.instruction == "begin_pass")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let draw_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "shader" && node.op.instruction == "draw_instanced")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    for pass_kind in &pass_kind_nodes {
        for begin_pass in &begin_pass_nodes {
            push_edge_if_missing(module, EdgeKind::CrossDomainExchange, pass_kind, begin_pass);
        }
    }
    for packet_field_count in &packet_field_count_nodes {
        for draw in &draw_nodes {
            push_edge_if_missing(module, EdgeKind::CrossDomainExchange, packet_field_count, draw);
        }
    }
}

fn has_xfer_segment(
    module: &YirModule,
    node_families: &BTreeMap<&str, String>,
    from_family: &str,
    to_family: &str,
) -> bool {
    module.edges.iter().any(|edge| {
        edge.kind == EdgeKind::CrossDomainExchange
            && node_family(node_families, &edge.from) == Some(from_family)
            && node_family(node_families, &edge.to) == Some(to_family)
    })
}

fn node_family<'a>(node_families: &'a BTreeMap<&str, String>, node_name: &str) -> Option<&'a str> {
    node_families.get(node_name).map(String::as_str)
}

fn apply_support_module_profile(ast: &AstModule, module: &mut YirModule) -> Result<(), String> {
    let Some(profile) = ast.functions.iter().find(|function| function.name == "profile") else {
        return Ok(());
    };
    let int_bindings = collect_profile_int_bindings(&profile.body);

    match ast.domain.as_str() {
        "shader" => {
            ensure_project_resource(module, "shader0", "shader.render");
            for stmt in &profile.body {
                apply_shader_profile_stmt(ast, stmt, module, &int_bindings)?;
            }
        }
        "kernel" => {
            ensure_project_resource(module, "kernel0", "kernel.apple");
            for stmt in &profile.body {
                apply_kernel_profile_stmt(ast, stmt, module, &int_bindings)?;
            }
        }
        "data" => {
            ensure_project_resource(module, "fabric0", "data.fabric");
            for stmt in &profile.body {
                apply_data_profile_stmt(ast, stmt, module, &int_bindings)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn apply_shader_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "cpu0",
            Operation {
                module: "cpu".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
        return Ok(());
    }

    let Some((node_name, callee, args)) = extract_profile_call(stmt) else {
        return Ok(());
    };
    let name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit),
        sanitize_ident(node_name)
    );
    let op = match callee {
        "shader_target" => Operation {
            module: "shader".to_owned(),
            instruction: "target".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "shader_target")?,
                expect_profile_int_arg(args, 1, "shader_target", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 2, "shader_target", int_bindings)?.to_string(),
            ],
        },
        "shader_viewport" => Operation {
            module: "shader".to_owned(),
            instruction: "viewport".to_owned(),
            args: vec![
                expect_profile_int_arg(args, 0, "shader_viewport", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 1, "shader_viewport", int_bindings)?.to_string(),
            ],
        },
        "shader_pipeline" => Operation {
            module: "shader".to_owned(),
            instruction: "pipeline".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "shader_pipeline")?,
                expect_text_arg(args, 1, "shader_pipeline")?,
            ],
        },
        _ => return Ok(()),
    };
    push_profile_node(module, name, "shader0", op);
    Ok(())
}

fn apply_data_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "cpu0",
            Operation {
                module: "cpu".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
        return Ok(());
    }

    let Some((node_name, callee, args)) = extract_profile_call(stmt) else {
        return Ok(());
    };
    let name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit),
        sanitize_ident(node_name)
    );
    let op = match callee {
        "data_bind_core" => Operation {
            module: "data".to_owned(),
            instruction: "bind_core".to_owned(),
            args: vec![expect_profile_int_arg(args, 0, "data_bind_core", int_bindings)?.to_string()],
        },
        "data_handle_table" => Operation {
            module: "data".to_owned(),
            instruction: "handle_table".to_owned(),
            args: args
                .iter()
                .enumerate()
                .map(|(index, _)| expect_text_arg(args, index, "data_handle_table"))
                .collect::<Result<Vec<_>, _>>()?,
        },
        "data_marker" => Operation {
            module: "data".to_owned(),
            instruction: "marker".to_owned(),
            args: vec![expect_text_arg(args, 0, "data_marker")?],
        },
        "data_copy_window" => Operation {
            module: "data".to_owned(),
            instruction: "copy_window".to_owned(),
            args: vec![
                expect_profile_value_input_name(ast, args, 0, "data_copy_window")?,
                expect_profile_int_arg(args, 1, "data_copy_window", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 2, "data_copy_window", int_bindings)?.to_string(),
            ],
        },
        "data_immutable_window" => Operation {
            module: "data".to_owned(),
            instruction: "immutable_window".to_owned(),
            args: vec![
                expect_profile_value_input_name(ast, args, 0, "data_immutable_window")?,
                expect_profile_int_arg(args, 1, "data_immutable_window", int_bindings)?.to_string(),
                expect_profile_int_arg(args, 2, "data_immutable_window", int_bindings)?.to_string(),
            ],
        },
        _ => return Ok(()),
    };
    push_profile_node(module, name, "fabric0", op);
    Ok(())
}

fn apply_kernel_profile_stmt(
    ast: &AstModule,
    stmt: &AstStmt,
    module: &mut YirModule,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<(), String> {
    if let Some((node_name, value)) = extract_profile_int_binding(stmt) {
        let name = format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(node_name)
        );
        push_profile_node(
            module,
            name,
            "cpu0",
            Operation {
                module: "cpu".to_owned(),
                instruction: "const_i64".to_owned(),
                args: vec![value.to_string()],
            },
        );
        return Ok(());
    }

    let Some((node_name, callee, args)) = extract_profile_call(stmt) else {
        return Ok(());
    };
    let name = format!(
        "project_profile_{}_{}_{}",
        sanitize_ident(&ast.domain),
        sanitize_ident(&ast.unit),
        sanitize_ident(node_name)
    );
    let op = match callee {
        "kernel_target_config" => Operation {
            module: "kernel".to_owned(),
            instruction: "target_config".to_owned(),
            args: vec![
                expect_text_arg(args, 0, "kernel_target_config")?,
                expect_text_arg(args, 1, "kernel_target_config")?,
                expect_profile_int_arg(args, 2, "kernel_target_config", int_bindings)?.to_string(),
            ],
        },
        _ => return Ok(()),
    };
    push_profile_node(module, name, "kernel0", op);
    Ok(())
}

fn collect_profile_int_bindings(body: &[AstStmt]) -> BTreeMap<String, i64> {
    let mut bindings = BTreeMap::new();
    for stmt in body {
        if let Some((name, value)) = extract_profile_int_binding(stmt) {
            bindings.insert(name.to_owned(), value);
        }
    }
    bindings
}

fn extract_profile_call(stmt: &AstStmt) -> Option<(&str, &str, &[AstExpr])> {
    match stmt {
        AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
            if let AstExpr::Call { callee, args } = value {
                Some((name.as_str(), callee.as_str(), args.as_slice()))
            } else {
                None
            }
        }
        AstStmt::Expr(AstExpr::Call { callee, args }) => {
            Some((callee.as_str(), callee.as_str(), args.as_slice()))
        }
        _ => None,
    }
}

fn extract_profile_int_binding(stmt: &AstStmt) -> Option<(&str, i64)> {
    match stmt {
        AstStmt::Let { name, value, .. } | AstStmt::Const { name, value, .. } => {
            if let AstExpr::Int(value) = value {
                Some((name.as_str(), *value))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn expect_text_arg(args: &[AstExpr], index: usize, callee: &str) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Text(value)) => Ok(value.clone()),
        _ => Err(format!("{callee}(...) expects string literal arg {}", index + 1)),
    }
}

fn expect_profile_int_arg(
    args: &[AstExpr],
    index: usize,
    callee: &str,
    int_bindings: &BTreeMap<String, i64>,
) -> Result<i64, String> {
    match args.get(index) {
        Some(AstExpr::Int(value)) => Ok(*value),
        Some(AstExpr::Var(name)) => int_bindings.get(name).copied().ok_or_else(|| {
            format!(
                "{callee}(...) expects integer literal or profile const arg {}, unknown `{}`",
                index + 1,
                name
            )
        }),
        _ => Err(format!(
            "{callee}(...) expects integer literal or profile const arg {}",
            index + 1
        )),
    }
}

fn expect_profile_value_input_name(
    ast: &AstModule,
    args: &[AstExpr],
    index: usize,
    callee: &str,
) -> Result<String, String> {
    match args.get(index) {
        Some(AstExpr::Var(name)) => Ok(format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(&ast.domain),
            sanitize_ident(&ast.unit),
            sanitize_ident(name)
        )),
        _ => Err(format!(
            "{callee}(...) expects profile value reference arg {}",
            index + 1
        )),
    }
}

fn ensure_project_resource(module: &mut YirModule, name: &str, kind: &str) {
    if module.resources.iter().any(|resource| resource.name == name) {
        return;
    }
    module.resources.push(Resource {
        name: name.to_owned(),
        kind: ResourceKind::parse(kind),
    });
}

fn push_profile_node(module: &mut YirModule, name: String, resource: &str, op: Operation) {
    if module.nodes.iter().any(|node| node.name == name) {
        return;
    }
    module.nodes.push(Node {
        name,
        resource: resource.to_owned(),
        op,
    });
}

fn resolve_project_profile_refs(module: &mut YirModule) -> Result<(), String> {
    let replacements = module
        .nodes
        .iter()
        .filter_map(|node| {
            (node.op.module == "cpu" && node.op.instruction == "project_profile_ref").then_some(node)
        })
        .map(|node| {
            let [domain, unit, slot] = node.op.args.as_slice() else {
                return Err(format!(
                    "project profile ref node `{}` expects `<domain> <unit> <slot>` args",
                    node.name
                ));
            };
            let target = resolve_project_profile_target_name(domain, unit, slot);
            if !module.nodes.iter().any(|candidate| candidate.name == target) {
                return Err(format!(
                    "project profile ref `{}` could not resolve `{}` `{}` slot `{}` into a support-module profile node",
                    node.name, domain, unit, slot
                ));
            }
            Ok((node.name.clone(), target))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    if replacements.is_empty() {
        return Ok(());
    }

    for node in &mut module.nodes {
        if node.op.module == "cpu" && node.op.instruction == "project_profile_ref" {
            continue;
        }
        for arg in &mut node.op.args {
            if let Some(target) = replacements.get(arg) {
                *arg = target.clone();
            }
        }
    }
    for edge in &mut module.edges {
        if let Some(target) = replacements.get(&edge.from) {
            edge.from = target.clone();
        }
        if let Some(target) = replacements.get(&edge.to) {
            edge.to = target.clone();
        }
    }
    let replacement_targets = replacements.values().cloned().collect::<BTreeSet<_>>();
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = module
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut extra_dep_edges = Vec::new();
    for node in &module.nodes {
        if node.op.module == "cpu" && node.op.instruction == "project_profile_ref" {
            continue;
        }
        for arg in &node.op.args {
            if !replacement_targets.contains(arg) {
                continue;
            }
            let edge_kind =
                inferred_project_dependency_edge_kind(&resource_families, &node_resources, arg, &node.name);
            let exists = module.edges.iter().any(|edge| {
                edge.kind == edge_kind && edge.from == *arg && edge.to == node.name
            });
            if !exists {
                extra_dep_edges.push(yir_core::Edge {
                    kind: edge_kind,
                    from: arg.clone(),
                    to: node.name.clone(),
                });
            }
        }
    }
    module
        .nodes
        .retain(|node| !(node.op.module == "cpu" && node.op.instruction == "project_profile_ref"));
    module.edges.extend(extra_dep_edges);
    Ok(())
}

fn inferred_project_dependency_edge_kind(
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    from_node: &str,
    to_node: &str,
) -> EdgeKind {
    let from_family = node_resources
        .get(from_node)
        .and_then(|resource| resource_families.get(resource))
        .map(String::as_str);
    let to_family = node_resources
        .get(to_node)
        .and_then(|resource| resource_families.get(resource))
        .map(String::as_str);
    if from_family.is_some() && from_family == to_family {
        EdgeKind::Dep
    } else {
        EdgeKind::CrossDomainExchange
    }
}

fn resolve_project_profile_target_name(domain: &str, unit: &str, slot: &str) -> String {
    match (domain, slot) {
        ("shader", "target") => format!(
            "project_profile_shader_{}_profile_target",
            sanitize_ident(unit)
        ),
        ("shader", "viewport") => format!(
            "project_profile_shader_{}_profile_view",
            sanitize_ident(unit)
        ),
        ("shader", "pipeline") => format!(
            "project_profile_shader_{}_profile_pipe",
            sanitize_ident(unit)
        ),
        ("shader", "vertex_count") => format!(
            "project_profile_shader_{}_vertex_count",
            sanitize_ident(unit)
        ),
        ("shader", "instance_count") => format!(
            "project_profile_shader_{}_instance_count",
            sanitize_ident(unit)
        ),
        ("shader", "packet_color_slot") => format!(
            "project_profile_shader_{}_packet_color_slot",
            sanitize_ident(unit)
        ),
        ("shader", "packet_speed_slot") => format!(
            "project_profile_shader_{}_packet_speed_slot",
            sanitize_ident(unit)
        ),
        ("shader", "packet_radius_slot") => format!(
            "project_profile_shader_{}_packet_radius_slot",
            sanitize_ident(unit)
        ),
        ("shader", "packet_tag") => format!(
            "project_profile_shader_{}_packet_tag",
            sanitize_ident(unit)
        ),
        ("shader", "material_mode") => format!(
            "project_profile_shader_{}_material_mode",
            sanitize_ident(unit)
        ),
        ("shader", "pass_kind") => format!(
            "project_profile_shader_{}_pass_kind",
            sanitize_ident(unit)
        ),
        ("shader", "packet_field_count") => format!(
            "project_profile_shader_{}_packet_field_count",
            sanitize_ident(unit)
        ),
        ("kernel", "bind_core") => format!(
            "project_profile_kernel_{}_bind_core",
            sanitize_ident(unit)
        ),
        ("kernel", "queue_depth") => format!(
            "project_profile_kernel_{}_queue_depth",
            sanitize_ident(unit)
        ),
        ("kernel", "batch_lanes") => format!(
            "project_profile_kernel_{}_batch_lanes",
            sanitize_ident(unit)
        ),
        ("data", "bind_core") => format!(
            "project_profile_data_{}_data_bind_core",
            sanitize_ident(unit)
        ),
        ("data", "window_offset") => format!(
            "project_profile_data_{}_window_offset",
            sanitize_ident(unit)
        ),
        ("data", "uplink_len") => format!(
            "project_profile_data_{}_uplink_len",
            sanitize_ident(unit)
        ),
        ("data", "downlink_len") => format!(
            "project_profile_data_{}_downlink_len",
            sanitize_ident(unit)
        ),
        ("data", "handle_table") => format!(
            "project_profile_data_{}_profile_handles",
            sanitize_ident(unit)
        ),
        ("data", marker) if marker.starts_with("marker:") => format!(
            "project_profile_data_{}_{}",
            sanitize_ident(unit),
            sanitize_ident(marker.trim_start_matches("marker:"))
        ),
        _ => format!(
            "project_profile_{}_{}_{}",
            sanitize_ident(domain),
            sanitize_ident(unit),
            sanitize_ident(slot)
        ),
    }
}

fn stitch_data_profile_edges(module: &mut YirModule) {
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = module
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    let handle_tables = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "data" && node.op.instruction == "handle_table")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let cpu_to_shader_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("cpu_to_shader")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let cpu_to_kernel_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("cpu_to_kernel")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_pipe_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("uplink_pipe")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_pipe_class_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("uplink_pipe_class")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_payload_class_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("uplink_payload_class")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_payload_shape_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("uplink_payload_shape")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_pipe_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("downlink_pipe")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_payload_class_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("downlink_payload_class")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_payload_shape_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("downlink_payload_shape")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_pipe_class_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("downlink_pipe_class")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_window_policy_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("uplink_window_policy")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_window_policy_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("downlink_window_policy")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let shader_to_cpu_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("shader_to_cpu")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let kernel_to_cpu_markers = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && node.op.instruction == "marker"
                && node.op.args.first().map(String::as_str) == Some("kernel_to_cpu")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let kernel_nodes = module
        .nodes
        .iter()
        .filter(|node| node.op.module == "kernel")
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let data_pipe_nodes = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && matches!(node.op.instruction.as_str(), "output_pipe" | "input_pipe")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let uplink_windows = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && matches!(node.op.instruction.as_str(), "copy_window" | "immutable_window")
                && node.name.contains("_uplink_window")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let downlink_windows = module
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "data"
                && matches!(node.op.instruction.as_str(), "copy_window" | "immutable_window")
                && node.name.contains("_downlink_window")
        })
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let window_offset = module
        .nodes
        .iter()
        .find(|node| node.name.contains("_window_offset"))
        .map(|node| node.name.clone());
    let uplink_len = module
        .nodes
        .iter()
        .find(|node| node.name.contains("_uplink_len"))
        .map(|node| node.name.clone());
    let downlink_len = module
        .nodes
        .iter()
        .find(|node| node.name.contains("_downlink_len"))
        .map(|node| node.name.clone());

    for handle in &handle_tables {
        for pipe in &data_pipe_nodes {
            push_edge_if_missing(module, EdgeKind::Dep, handle, pipe);
        }
    }
    if let Some(marker) = cpu_to_shader_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = shader_to_cpu_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = cpu_to_kernel_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = kernel_to_cpu_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_pipe_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_pipe_class_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_payload_class_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = uplink_payload_shape_markers.first() {
        for pipe in data_pipe_nodes.iter().take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
        for window in &uplink_windows {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
    }
    if let Some(marker) = downlink_pipe_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = downlink_pipe_class_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = downlink_payload_class_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
    }
    if let Some(marker) = downlink_payload_shape_markers.first() {
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_edge_if_missing(module, EdgeKind::Effect, marker, pipe);
        }
        for window in &downlink_windows {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
    }
    for window in &uplink_windows {
        if let Some(marker) = uplink_window_policy_markers.first() {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
        for pipe in data_pipe_nodes.iter().take(2) {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                window,
                pipe,
            );
        }
        if let Some(offset) = &window_offset {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                offset,
                window,
            );
        }
        if let Some(len) = &uplink_len {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                len,
                window,
            );
        }
    }
    for window in &downlink_windows {
        if let Some(marker) = downlink_window_policy_markers.first() {
            push_edge_if_missing(module, EdgeKind::Effect, marker, window);
        }
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                window,
                pipe,
            );
        }
        if let Some(offset) = &window_offset {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                offset,
                window,
            );
        }
        if let Some(len) = &downlink_len {
            push_project_dependency_edge_if_missing(
                module,
                &resource_families,
                &node_resources,
                len,
                window,
            );
        }
    }
    if uplink_windows.is_empty() {
        if let Some(offset) = &window_offset {
            for pipe in data_pipe_nodes.iter().take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    offset,
                    pipe,
                );
            }
        }
        if let Some(len) = &uplink_len {
            for pipe in data_pipe_nodes.iter().take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    len,
                    pipe,
                );
            }
        }
    }
    if downlink_windows.is_empty() {
        if let Some(offset) = &window_offset {
            for pipe in data_pipe_nodes.iter().skip(2).take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    offset,
                    pipe,
                );
            }
        }
        if let Some(len) = &downlink_len {
            for pipe in data_pipe_nodes.iter().skip(2).take(2) {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    len,
                    pipe,
                );
            }
        }
    }
    if !kernel_nodes.is_empty() {
        for pipe in data_pipe_nodes.iter().take(2) {
            for kernel_node in &kernel_nodes {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    pipe,
                    kernel_node,
                );
            }
        }
        for pipe in data_pipe_nodes.iter().skip(2).take(2) {
            for kernel_node in &kernel_nodes {
                push_project_dependency_edge_if_missing(
                    module,
                    &resource_families,
                    &node_resources,
                    kernel_node,
                    pipe,
                );
            }
        }
    }
}

fn push_edge_if_missing(module: &mut YirModule, kind: EdgeKind, from: &str, to: &str) {
    if module
        .edges
        .iter()
        .any(|edge| edge.kind == kind && edge.from == from && edge.to == to)
    {
        return;
    }
    module.edges.push(yir_core::Edge {
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

fn push_project_dependency_edge_if_missing(
    module: &mut YirModule,
    resource_families: &BTreeMap<String, String>,
    node_resources: &BTreeMap<String, String>,
    from: &str,
    to: &str,
) {
    let kind = inferred_project_dependency_edge_kind(resource_families, node_resources, from, to);
    push_edge_if_missing(module, kind, from, to);
}

fn validate_project_modules(modules: &[ProjectModule]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for module in modules {
        let key = (module.ast.domain.clone(), module.ast.unit.clone());
        if !seen.insert(key.clone()) {
            return Err(format!(
                "duplicate project mod definition for `mod {} {}`",
                key.0, key.1
            ));
        }
    }
    Ok(())
}

fn validate_project_unit_bindings(modules: &[ProjectModule]) -> Result<(), String> {
    for module in modules {
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new("nustar-packages"),
            &module.ast.domain,
        )?;
        crate::registry::validate_unit_binding(&[manifest], &module.ast.domain, &module.ast.unit)?;
    }
    Ok(())
}

fn validate_project_uses(modules: &[ProjectModule]) -> Result<(), String> {
    let local_units = modules
        .iter()
        .map(|module| (module.ast.domain.clone(), module.ast.unit.clone()))
        .collect::<BTreeSet<_>>();
    for module in modules {
        for item in &module.ast.uses {
            if local_units.contains(&(item.domain.clone(), item.unit.clone())) {
                continue;
            }
            let manifest = crate::registry::load_manifest_for_domain(
                Path::new("nustar-packages"),
                &item.domain,
            )?;
            crate::registry::validate_unit_binding(&[manifest], &item.domain, &item.unit)?;
        }
    }
    Ok(())
}

fn validate_project_links(
    manifest: &NuisProjectManifest,
    modules: &[ProjectModule],
) -> Result<(), String> {
    let local_units = modules
        .iter()
        .map(|module| {
            (
                format!("{}.{}", module.ast.domain, module.ast.unit),
                module.ast.clone(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    for link in &manifest.links {
        let from_module = local_units.get(&link.from).ok_or_else(|| {
            format!("project link references unknown source unit `{}`", link.from)
        })?;
        if !local_units.contains_key(&link.to) {
            validate_external_unit_ref(&link.to)?;
        }
        if let Some(via) = &link.via {
            validate_external_unit_ref(via)?;
            let (via_domain, via_unit) = split_domain_unit(via)?;
            if via_domain != "data" {
                return Err(format!(
                    "project link `{}` -> `{}` uses unsupported mediator `{}`; current project links must use a `data.*` unit",
                    link.from, link.to, via
                ));
            }
            if !from_module
                .uses
                .iter()
                .any(|item| item.domain == via_domain && item.unit == via_unit)
            {
                return Err(format!(
                    "project link source `{}` must `use {} {}` because link is mediated via `{}`",
                    link.from, via_domain, via_unit, via
                ));
            }
            if let Some(target_module) = local_units.get(&link.to) {
                if !target_module
                    .uses
                    .iter()
                    .any(|item| item.domain == via_domain && item.unit == via_unit)
                {
                    return Err(format!(
                        "project link target `{}` must `use {} {}` because link is mediated via `{}`",
                        link.to, via_domain, via_unit, via
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_external_unit_ref(reference: &str) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(reference)?;
    let manifest =
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &domain)?;
    crate::registry::validate_unit_binding(&[manifest], &domain, &unit)
}

fn split_domain_unit(reference: &str) -> Result<(String, String), String> {
    let Some((domain, unit)) = reference.split_once('.') else {
        return Err(format!(
            "project link reference `{reference}` must use `domain.Unit` form"
        ));
    };
    Ok((domain.trim().to_owned(), unit.trim().to_owned()))
}

fn parse_project_manifest(source: &str, path: &Path) -> Result<NuisProjectManifest, String> {
    let name = parse_required_string(source, "name", path)?;
    let entry = parse_required_string(source, "entry", path)?;
    let modules = parse_optional_string_array(source, "modules").unwrap_or_default();
    let links = parse_optional_link_array(source, "links").unwrap_or_default();
    Ok(NuisProjectManifest {
        name,
        entry,
        modules,
        links,
    })
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    parse_optional_string(source, key).ok_or_else(|| {
        format!(
            "project manifest `{}` is missing required field `{key}`",
            path.display()
        )
    })
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    let mut lines = source.lines();
    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let mut collected = rest.trim().to_owned();
            if !collected.contains(']') {
                for next_line in lines.by_ref() {
                    collected.push(' ');
                    collected.push_str(next_line.trim());
                    if next_line.contains(']') {
                        break;
                    }
                }
            }
            let body = collected.trim();
            let body = body.strip_prefix('[')?.strip_suffix(']')?;
            let mut values = Vec::new();
            for item in body.split(',') {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                values.push(
                    parse_quoted(item)
                        .ok_or_else(|| format!("invalid string array value `{item}`"))
                        .ok()?,
                );
            }
            return Some(values);
        }
    }
    None
}

fn parse_optional_link_array(source: &str, key: &str) -> Option<Vec<ProjectLink>> {
    let values = parse_optional_string_array(source, key)?;
    let mut links = Vec::new();
    for value in values {
        let parts = value.split("->").map(str::trim).collect::<Vec<_>>();
        if parts.len() < 2 {
            return None;
        }
        let from = parts[0].to_owned();
        let rhs = parts[1];
        let (to, via) = if let Some((to, via)) = rhs.split_once(" via ") {
            (to.trim().to_owned(), Some(via.trim().to_owned()))
        } else {
            (rhs.to_owned(), None)
        };
        links.push(ProjectLink { from, to, via });
    }
    Some(links)
}

fn parse_quoted(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

fn sanitize_ident(raw: &str) -> String {
    raw.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}
