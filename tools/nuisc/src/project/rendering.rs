use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use nuis_semantics::model::{AstExternFunction, AstTypeRef};

use super::{
    profile_apply::resolve_registered_abi_target, resolve_project_abi, LoadedProject,
    ProjectAbiResolution, ProjectAbiSelectionView, ProjectExchangeOrganization,
    ProjectExchangeRoute, ProjectLoweringIssue, ProjectLoweringIssueKind,
    ProjectLoweringSelectionView, ProjectOrganization, ProjectOrganizationLink,
    ProjectOrganizationModule,
};

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn selected_lowering_target_for_domain(domain: &str, abi: &str) -> Result<Option<String>, String> {
    match domain {
        "cpu" => {
            crate::aot::resolve_cpu_build_target_from_abi(Path::new("nustar-packages"), abi)?;
            Ok(Some("llvm".to_owned()))
        }
        "shader" | "kernel" => Ok(resolve_registered_abi_target(
            domain,
            Some(&ProjectAbiResolution {
                requirements: vec![super::ProjectAbiRequirement {
                    domain: domain.to_owned(),
                    abi: abi.to_owned(),
                }],
                explicit: true,
            }),
        )?
        .and_then(|target| target.backend_family)),
        "network" => Ok(resolve_registered_abi_target(
            domain,
            Some(&ProjectAbiResolution {
                requirements: vec![super::ProjectAbiRequirement {
                    domain: domain.to_owned(),
                    abi: abi.to_owned(),
                }],
                explicit: true,
            }),
        )?
        .map(|target| match target.machine_os.as_str() {
            "darwin" => "urlsession".to_owned(),
            "windows" => "winsock".to_owned(),
            _ => "socket-abi".to_owned(),
        })),
        _ => Ok(None),
    }
}

pub fn project_abi_selection_views(
    resolution: &ProjectAbiResolution,
) -> Vec<ProjectAbiSelectionView> {
    let mut views = resolution
        .requirements
        .iter()
        .map(|item| {
            let target = resolve_registered_abi_target(&item.domain, Some(resolution))
                .ok()
                .flatten();
            ProjectAbiSelectionView {
                domain: item.domain.clone(),
                abi: item.abi.clone(),
                machine_arch: target.as_ref().map(|target| target.machine_arch.clone()),
                machine_os: target.as_ref().map(|target| target.machine_os.clone()),
                object_format: target.as_ref().map(|target| target.object_format.clone()),
                calling_abi: target.as_ref().map(|target| target.calling_abi.clone()),
                clang_target: target.as_ref().map(|target| target.clang_target.clone()),
                backend_family: target
                    .as_ref()
                    .and_then(|target| target.backend_family.clone()),
                host_adaptive: target.as_ref().map(|target| target.host_adaptive),
            }
        })
        .collect::<Vec<_>>();
    views.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
    views
}

pub fn project_abi_selection_view_json(item: &ProjectAbiSelectionView) -> String {
    format!(
        "{{\"domain\":\"{}\",\"abi\":\"{}\",\"abi_target_machine\":{},\"abi_target_object\":{},\"abi_target_calling\":{},\"abi_target_clang\":{},\"abi_target_backend\":{},\"abi_target_host_adaptive\":{}}}",
        json_escape(&item.domain),
        json_escape(&item.abi),
        item.machine_arch
            .as_deref()
            .zip(item.machine_os.as_deref())
            .map(|(arch, os)| format!("\"{}-{}\"", json_escape(arch), json_escape(os)))
            .unwrap_or_else(|| "null".to_owned()),
        item.object_format
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.calling_abi
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.clang_target
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.backend_family
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        item.host_adaptive
            .map(|value| if value { "true" } else { "false" }.to_owned())
            .unwrap_or_else(|| "null".to_owned())
    )
}

pub fn render_project_abi_selection_lines(resolution: &ProjectAbiResolution) -> Vec<String> {
    let mut out = String::new();
    write_project_abi_selection_lines(&mut out, resolution)
        .expect("writing project abi selection lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn render_project_abi_selection_view_lines(item: &ProjectAbiSelectionView) -> Vec<String> {
    let mut out = String::new();
    write_project_abi_selection_view_lines(&mut out, item)
        .expect("writing project abi selection view lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_abi_selection_lines<W: fmt::Write>(
    out: &mut W,
    resolution: &ProjectAbiResolution,
) -> fmt::Result {
    for item in project_abi_selection_views(resolution) {
        write_project_abi_selection_view_lines(out, &item)?;
    }
    Ok(())
}

pub fn write_project_abi_selection_view_lines<W: fmt::Write>(
    out: &mut W,
    item: &ProjectAbiSelectionView,
) -> fmt::Result {
    writeln!(out, "abi: {}={}", item.domain, item.abi)?;
    if let (Some(machine_arch), Some(machine_os)) =
        (item.machine_arch.as_deref(), item.machine_os.as_deref())
    {
        writeln!(out, "  abi_target_machine: {}-{}", machine_arch, machine_os)?;
    }
    if let Some(object_format) = item.object_format.as_deref() {
        writeln!(out, "  abi_target_object: {}", object_format)?;
    }
    if let Some(calling_abi) = item.calling_abi.as_deref() {
        writeln!(out, "  abi_target_calling: {}", calling_abi)?;
    }
    if let Some(clang_target) = item.clang_target.as_deref() {
        writeln!(out, "  abi_target_clang: {}", clang_target)?;
    }
    if let Some(backend_family) = item.backend_family.as_deref() {
        writeln!(out, "  abi_target_backend: {}", backend_family)?;
    }
    if let Some(host_adaptive) = item.host_adaptive {
        writeln!(
            out,
            "  abi_target_host_adaptive: {}",
            if host_adaptive { "true" } else { "false" }
        )?;
    }
    Ok(())
}

pub fn validate_project_lowering_selections(
    resolution: &ProjectAbiResolution,
) -> Vec<ProjectLoweringSelectionView> {
    let mut views = resolution
        .requirements
        .iter()
        .map(|item| {
            let mut issues = Vec::new();
            let mut registered_lowering_targets = Vec::new();
            let mut selected_lowering_target = None;
            match crate::registry::load_domain_registration_for_domain(
                Path::new("nustar-packages"),
                &item.domain,
            ) {
                Ok(registration) => {
                    registered_lowering_targets = registration.lowering_targets.clone();
                    if registered_lowering_targets.is_empty() {
                        issues.push(ProjectLoweringIssue {
                            kind: ProjectLoweringIssueKind::NoRegisteredLoweringTargets,
                            message: format!(
                                "registered domain `{}` does not declare any lowering targets",
                                item.domain
                            ),
                        });
                    }
                    match selected_lowering_target_for_domain(&item.domain, &item.abi) {
                        Ok(selected) => {
                            selected_lowering_target = selected;
                        }
                        Err(error) => issues.push(ProjectLoweringIssue {
                            kind: ProjectLoweringIssueKind::AbiTargetResolutionFailed,
                            message: error,
                        }),
                    }
                    if let Some(selected) = selected_lowering_target.as_deref() {
                        if !registered_lowering_targets
                            .iter()
                            .any(|target| target == selected)
                        {
                            issues.push(ProjectLoweringIssue {
                                kind: ProjectLoweringIssueKind::SelectedLoweringTargetNotRegistered,
                                message: format!(
                                    "selected lowering target `{selected}` is not declared by registered lowering targets: {}",
                                    if registered_lowering_targets.is_empty() {
                                        "<none>".to_owned()
                                    } else {
                                        registered_lowering_targets.join(", ")
                                    }
                                ),
                            });
                        }
                    }
                }
                Err(error) => issues.push(ProjectLoweringIssue {
                    kind: ProjectLoweringIssueKind::DomainNotRegistered,
                    message: error,
                }),
            }
            ProjectLoweringSelectionView {
                domain: item.domain.clone(),
                abi: Some(item.abi.clone()),
                registered_lowering_targets,
                selected_lowering_target,
                ok: issues.is_empty(),
                issues,
            }
        })
        .collect::<Vec<_>>();
    views.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
    views
}

pub fn render_project_lowering_selection_lines(view: &ProjectLoweringSelectionView) -> Vec<String> {
    let mut out = String::new();
    write_project_lowering_selection_lines(&mut out, view)
        .expect("writing project lowering selection lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_lowering_selection_lines<W: fmt::Write>(
    out: &mut W,
    view: &ProjectLoweringSelectionView,
) -> fmt::Result {
    write!(
        out,
        "lowering: {} abi={} ok={} selected={} registered=",
        view.domain,
        view.abi.as_deref().unwrap_or("<none>"),
        if view.ok { "yes" } else { "no" },
        view.selected_lowering_target.as_deref().unwrap_or("<none>"),
    )?;
    if view.registered_lowering_targets.is_empty() {
        out.write_str("<none>")?;
    } else {
        write_joined(
            out,
            &view.registered_lowering_targets,
            ", ",
            |out, target| write!(out, "{target}"),
        )?;
    }
    writeln!(out, "\tissues={}", view.issue_count())?;
    for issue in &view.issues {
        writeln!(
            out,
            "lowering_issue: {}",
            issue.summary().replace(": ", " ")
        )?;
    }
    Ok(())
}

pub fn project_lowering_issue_json(issue: &ProjectLoweringIssue) -> String {
    format!(
        "{{\"code\":\"{}\",\"kind\":\"{}\",\"message\":\"{}\"}}",
        issue.kind.code(),
        issue.kind.as_str(),
        json_escape(&issue.message)
    )
}

pub fn project_lowering_selection_json(view: &ProjectLoweringSelectionView) -> String {
    let issues = view
        .issues
        .iter()
        .map(project_lowering_issue_json)
        .collect::<Vec<_>>()
        .join(",");
    let registered = view
        .registered_lowering_targets
        .iter()
        .map(|target| format!("\"{}\"", json_escape(target)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"domain\":\"{}\",\"abi\":{},\"registered_lowering_targets\":[{}],\"selected_lowering_target\":{},\"ok\":{},\"issues\":[{}]}}",
        json_escape(&view.domain),
        view.abi
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        registered,
        view.selected_lowering_target
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        if view.ok { "true" } else { "false" },
        issues
    )
}

pub fn ensure_project_lowering_selections_valid(
    resolution: &ProjectAbiResolution,
) -> Result<(), String> {
    let failures = validate_project_lowering_selections(resolution)
        .into_iter()
        .filter(|view| !view.ok)
        .map(|view| view.summary_line())
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "project lowering selection validation failed:\n{}",
            failures.join("\n")
        ))
    }
}

pub fn organize_project(project: &LoadedProject) -> ProjectOrganization {
    let mut domains = project
        .modules
        .iter()
        .map(|module| module.ast.domain.clone())
        .collect::<std::collections::BTreeSet<_>>();
    for link in &project.manifest.links {
        if let Some((domain, _)) = link.from.split_once('.') {
            domains.insert(domain.to_owned());
        }
        if let Some((domain, _)) = link.to.split_once('.') {
            domains.insert(domain.to_owned());
        }
        if let Some(via) = &link.via {
            if let Some((domain, _)) = via.split_once('.') {
                domains.insert(domain.to_owned());
            }
        }
    }
    let entry_relative = project
        .entry_path
        .strip_prefix(&project.root)
        .unwrap_or(project.entry_path.as_path())
        .display()
        .to_string();
    let modules = project
        .modules
        .iter()
        .map(|module| {
            let relative = module
                .path
                .strip_prefix(&project.root)
                .unwrap_or(module.path.as_path())
                .display()
                .to_string();
            ProjectOrganizationModule {
                is_entry: relative == entry_relative,
                path: relative,
                domain: module.ast.domain.clone(),
                unit: module.ast.unit.clone(),
                source_kind: module.origin.source_kind().to_owned(),
                source_detail: module.origin.source_detail(),
            }
        })
        .collect::<Vec<_>>();
    let links = project
        .manifest
        .links
        .iter()
        .map(|link| ProjectOrganizationLink {
            from: link.from.clone(),
            to: link.to.clone(),
            via: link.via.clone(),
        })
        .collect::<Vec<_>>();
    ProjectOrganization {
        entry: project.manifest.entry.clone(),
        domains: domains.into_iter().collect(),
        modules,
        links,
    }
}

pub fn organize_project_exchanges(project: &LoadedProject) -> ProjectExchangeOrganization {
    let routes = project
        .manifest
        .links
        .iter()
        .map(|link| {
            let mut domains = Vec::new();
            if let Some((domain, _)) = link.from.split_once('.') {
                domains.push(domain.to_owned());
            }
            if let Some((domain, _)) = link.to.split_once('.') {
                if !domains.iter().any(|item| item == domain) {
                    domains.push(domain.to_owned());
                }
            }
            if let Some(via) = &link.via {
                if let Some((domain, _)) = via.split_once('.') {
                    if !domains.iter().any(|item| item == domain) {
                        domains.push(domain.to_owned());
                    }
                }
            }
            ProjectExchangeRoute {
                from: link.from.clone(),
                to: link.to.clone(),
                via: link.via.clone(),
                mode: if link.via.is_some() {
                    "bridged".to_owned()
                } else {
                    "direct".to_owned()
                },
                class: if link.via.is_some() {
                    "bridged".to_owned()
                } else {
                    "local".to_owned()
                },
                domains,
            }
        })
        .collect();
    ProjectExchangeOrganization { routes }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn render_project_organization_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_organization_index(&mut out, project)
        .expect("writing project organization index to String should not fail");
    out
}

pub(super) fn write_project_organization_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> fmt::Result {
    let organization = organize_project(project);
    writeln!(out, "entry\t{}", organization.entry)?;
    write!(out, "domains\t")?;
    write_joined(out, &organization.domains, ", ", |out, domain| {
        write!(out, "{domain}")
    })?;
    writeln!(out)?;
    for module in organization.modules {
        writeln!(
            out,
            "module\t{}\t{}\t{}\tentry={}\tsource_kind={}\t{}",
            module.path,
            module.domain,
            module.unit,
            module.is_entry,
            module.source_kind,
            module.source_detail
        )?;
    }
    for link in organization.links {
        writeln!(
            out,
            "link\t{}\t{}\t{}",
            link.from,
            link.to,
            link.via.unwrap_or_else(|| "<direct>".to_owned())
        )?;
    }
    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn render_project_exchange_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_exchange_index(&mut out, project)
        .expect("writing project exchange index to String should not fail");
    out
}

pub(super) fn write_project_exchange_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> fmt::Result {
    let exchanges = organize_project_exchanges(project);
    if exchanges.routes.is_empty() {
        return Ok(());
    }
    for route in exchanges.routes {
        write!(
            out,
            "route\t{}\t{}\t{}\tmode={}\tclass={}\tdomains=",
            route.from,
            route.to,
            route.via.unwrap_or_else(|| "<direct>".to_owned()),
            route.mode,
            route.class,
        )?;
        write_joined(out, &route.domains, ",", |out, domain| {
            write!(out, "{domain}")
        })?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn render_project_import_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_import_index(&mut out, project)
        .expect("writing project import index to String should not fail");
    out
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
    let visible_library_paths = project
        .modules
        .iter()
        .filter_map(|module| match &module.origin {
            super::ProjectModuleOrigin::AutoInjectedGalaxy { .. }
            | super::ProjectModuleOrigin::ExplicitGalaxyImport { .. } => Some(module.path.clone()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();

    for dependency in &project.resolved_galaxies {
        for (library_module, library_path) in dependency
            .library_modules
            .iter()
            .zip(dependency.resolved_library_paths.iter())
        {
            writeln!(
                out,
                "library\t{}\t{}\timport_policy={}\tauto_injectable={}\tvisible={}",
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
                }
            )?;
        }
    }

    for module in &project.modules {
        writeln!(
            out,
            "visible\t{}\t{}\tsource_kind={}\t{}",
            module.ast.domain,
            module.ast.unit,
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

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn render_project_abi_index(project: &LoadedProject) -> Result<String, String> {
    let mut out = String::new();
    write_project_abi_index(&mut out, project)
        .expect("writing project abi index to String should not fail");
    Ok(out)
}

pub(super) fn write_project_abi_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> Result<(), String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok(());
    }
    let mode = if resolution.explicit {
        "# mode=explicit"
    } else {
        "# mode=auto-recommended"
    };
    writeln!(out, "{mode}").map_err(|error| error.to_string())?;
    write_project_abi_graph_line(out, &resolution).map_err(|error| error.to_string())?;
    writeln!(out).map_err(|error| error.to_string())?;
    for item in project_abi_selection_views(&resolution) {
        let arch = item.machine_arch.as_deref().unwrap_or("unknown");
        let os = item.machine_os.as_deref().unwrap_or("unknown");
        let object = item.object_format.as_deref().unwrap_or("unknown");
        let calling = item.calling_abi.as_deref().unwrap_or("unknown");
        let backend = item.backend_family.as_deref().unwrap_or("none");
        writeln!(
            out,
            "domain\t{}\tabi={}\tarch={}\tos={}\tobject={}\tcalling={}\tbackend={}",
            item.domain, item.abi, arch, os, object, calling, backend
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn render_project_abi_graph_line(resolution: &ProjectAbiResolution) -> String {
    let mut out = String::new();
    write_project_abi_graph_line(&mut out, resolution)
        .expect("writing project abi graph line to String should not fail");
    out
}

pub fn describe_project_abi_graph(project: &LoadedProject) -> Result<String, String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok("graph\tmode=none\tdomains=<none>".to_owned());
    }
    Ok(render_project_abi_graph_line(&resolution))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn render_project_host_ffi_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_host_ffi_index(&mut out, project)
        .expect("writing project host ffi index to String should not fail");
    out
}

pub(super) fn write_project_host_ffi_index<W: fmt::Write>(
    out: &mut W,
    project: &LoadedProject,
) -> fmt::Result {
    for module in &project.modules {
        let relative = module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(module.path.as_path())
            .display()
            .to_string();

        for function in &module.ast.externs {
            write!(
                out,
                "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}\tsignature=",
                relative,
                module.ast.domain,
                module.ast.unit,
                function.abi,
                function.interface.as_deref().unwrap_or("-"),
                function.name,
            )?;
            write_host_ffi_signature(out, function)?;
            writeln!(out)?;
        }

        for interface in &module.ast.extern_interfaces {
            for method in &interface.methods {
                write!(
                    out,
                    "{}\tmod {} {}\tabi={}\tinterface={}\tsymbol={}__{}\tsignature=",
                    relative,
                    module.ast.domain,
                    module.ast.unit,
                    interface.abi,
                    interface.name,
                    interface.name,
                    method.name,
                )?;
                write_host_ffi_signature(out, method)?;
                writeln!(out)?;
            }
        }
    }
    Ok(())
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

fn write_project_abi_graph_line<W: fmt::Write>(
    out: &mut W,
    resolution: &ProjectAbiResolution,
) -> fmt::Result {
    let mut has_cpu = false;
    let mut has_data = false;
    let mut has_kernel = false;
    let mut has_shader = false;
    let mut has_network = false;

    write!(
        out,
        "graph\tmode={}\tdomains=",
        if resolution.explicit {
            "explicit"
        } else {
            "auto"
        }
    )?;
    write_joined(out, &resolution.requirements, ",", |out, item| {
        match item.domain.as_str() {
            "cpu" => has_cpu = true,
            "data" => has_data = true,
            "kernel" => has_kernel = true,
            "shader" => has_shader = true,
            "network" => has_network = true,
            _ => {}
        }
        write!(out, "{}", item.domain)
    })?;
    write!(
        out,
        "\tcpu_summary={}\tdata_summary={}\tkernel_target={}\tshader_target={}\tnetwork_target={}",
        if has_cpu { "present" } else { "absent" },
        if has_data { "present" } else { "absent" },
        if has_kernel { "present" } else { "absent" },
        if has_shader { "present" } else { "absent" },
        if has_network { "present" } else { "absent" },
    )
}

pub(super) fn render_ast_type_ref(ty: &AstTypeRef) -> String {
    let mut out = String::new();
    write_ast_type_ref(&mut out, ty).expect("writing ast type ref to String should not fail");
    out
}

fn write_host_ffi_signature<W: fmt::Write>(
    out: &mut W,
    function: &AstExternFunction,
) -> fmt::Result {
    write!(out, "fn {}(", function.name)?;
    write_joined(out, &function.params, ", ", |out, param| {
        write!(out, "{}: ", param.name)?;
        write_ast_type_ref(out, &param.ty)
    })?;
    write!(out, ") -> ")?;
    write_ast_type_ref(out, &function.return_type)
}

fn write_ast_type_ref<W: fmt::Write>(out: &mut W, ty: &AstTypeRef) -> fmt::Result {
    if ty.is_ref {
        write!(out, "ref ")?;
    }
    write!(out, "{}", ty.name)?;
    if !ty.generic_args.is_empty() {
        write!(out, "<")?;
        write_joined(out, &ty.generic_args, ", ", |out, arg| {
            write_ast_type_ref(out, arg)
        })?;
        write!(out, ">")?;
    }
    if ty.is_optional {
        write!(out, "?")?;
    }
    Ok(())
}
