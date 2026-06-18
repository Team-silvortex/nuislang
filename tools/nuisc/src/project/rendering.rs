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

fn selected_lowering_target_for_domain(
    domain: &str,
    abi: &str,
) -> Result<Option<String>, String> {
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
    project_abi_selection_views(resolution)
        .into_iter()
        .flat_map(|item| render_project_abi_selection_view_lines(&item))
        .collect()
}

pub fn render_project_abi_selection_view_lines(item: &ProjectAbiSelectionView) -> Vec<String> {
    let mut lines = vec![format!("abi: {}={}", item.domain, item.abi)];
    if let (Some(machine_arch), Some(machine_os)) =
        (item.machine_arch.as_deref(), item.machine_os.as_deref())
    {
        lines.push(format!(
            "  abi_target_machine: {}-{}",
            machine_arch, machine_os
        ));
    }
    if let Some(object_format) = item.object_format.as_deref() {
        lines.push(format!("  abi_target_object: {}", object_format));
    }
    if let Some(calling_abi) = item.calling_abi.as_deref() {
        lines.push(format!("  abi_target_calling: {}", calling_abi));
    }
    if let Some(clang_target) = item.clang_target.as_deref() {
        lines.push(format!("  abi_target_clang: {}", clang_target));
    }
    if let Some(backend_family) = item.backend_family.as_deref() {
        lines.push(format!("  abi_target_backend: {}", backend_family));
    }
    if let Some(host_adaptive) = item.host_adaptive {
        lines.push(format!(
            "  abi_target_host_adaptive: {}",
            if host_adaptive { "true" } else { "false" }
        ));
    }
    lines
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
    let mut lines = vec![format!(
        "lowering: {} abi={} ok={} selected={} registered={} issues={}",
        view.domain,
        view.abi.as_deref().unwrap_or("<none>"),
        if view.ok { "yes" } else { "no" },
        view.selected_lowering_target.as_deref().unwrap_or("<none>"),
        if view.registered_lowering_targets.is_empty() {
            "<none>".to_owned()
        } else {
            view.registered_lowering_targets.join(", ")
        },
        view.issue_count()
    )];
    for issue in &view.issues {
        lines.push(format!(
            "lowering_issue: {}",
            issue.summary().replace(": ", " ")
        ));
    }
    lines
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

pub(super) fn render_project_organization_index(project: &LoadedProject) -> String {
    let organization = organize_project(project);
    let mut lines = Vec::new();
    lines.push(format!("entry\t{}", organization.entry));
    lines.push(format!("domains\t{}", organization.domains.join(", ")));
    for module in organization.modules {
        lines.push(format!(
            "module\t{}\t{}\t{}\tentry={}",
            module.path, module.domain, module.unit, module.is_entry
        ));
    }
    for link in organization.links {
        lines.push(format!(
            "link\t{}\t{}\t{}",
            link.from,
            link.to,
            link.via.unwrap_or_else(|| "<direct>".to_owned())
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(super) fn render_project_exchange_index(project: &LoadedProject) -> String {
    let exchanges = organize_project_exchanges(project);
    if exchanges.routes.is_empty() {
        return String::new();
    }
    let mut lines = Vec::new();
    for route in exchanges.routes {
        lines.push(format!(
            "route\t{}\t{}\t{}\tmode={}\tclass={}\tdomains={}",
            route.from,
            route.to,
            route.via.unwrap_or_else(|| "<direct>".to_owned()),
            route.mode,
            route.class,
            route.domains.join(",")
        ));
    }
    format!("{}\n", lines.join("\n"))
}

pub(super) fn render_project_abi_index(project: &LoadedProject) -> Result<String, String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok(String::new());
    }
    let mode = if resolution.explicit {
        "# mode=explicit"
    } else {
        "# mode=auto-recommended"
    };
    let graph_summary = render_project_abi_graph_line(&resolution);
    let mut lines = vec![graph_summary];
    for item in project_abi_selection_views(&resolution) {
        let arch = item.machine_arch.as_deref().unwrap_or("unknown");
        let os = item.machine_os.as_deref().unwrap_or("unknown");
        let object = item.object_format.as_deref().unwrap_or("unknown");
        let calling = item.calling_abi.as_deref().unwrap_or("unknown");
        let backend = item.backend_family.as_deref().unwrap_or("none");
        lines.push(format!(
            "domain\t{}\tabi={}\tarch={}\tos={}\tobject={}\tcalling={}\tbackend={}",
            item.domain, item.abi, arch, os, object, calling, backend
        ));
    }
    lines.sort_by(|lhs, rhs| {
        if lhs.starts_with("graph\t") {
            std::cmp::Ordering::Less
        } else if rhs.starts_with("graph\t") {
            std::cmp::Ordering::Greater
        } else {
            lhs.cmp(rhs)
        }
    });
    Ok(format!("{mode}\n{}\n", lines.join("\n")))
}

pub fn render_project_abi_graph_line(resolution: &ProjectAbiResolution) -> String {
    let domains = resolution
        .requirements
        .iter()
        .map(|item| item.domain.as_str())
        .collect::<Vec<_>>();
    format!(
        "graph\tmode={}\tdomains={}\tcpu_summary={}\tdata_summary={}\tkernel_target={}\tshader_target={}\tnetwork_target={}",
        if resolution.explicit { "explicit" } else { "auto" },
        domains.join(","),
        if domains.iter().any(|domain| *domain == "cpu") {
            "present"
        } else {
            "absent"
        },
        if domains.iter().any(|domain| *domain == "data") {
            "present"
        } else {
            "absent"
        },
        if domains.iter().any(|domain| *domain == "kernel") {
            "present"
        } else {
            "absent"
        },
        if domains.iter().any(|domain| *domain == "shader") {
            "present"
        } else {
            "absent"
        },
        if domains.iter().any(|domain| *domain == "network") {
            "present"
        } else {
            "absent"
        },
    )
}

pub fn describe_project_abi_graph(project: &LoadedProject) -> Result<String, String> {
    let resolution = resolve_project_abi(project)?;
    if resolution.requirements.is_empty() {
        return Ok("graph\tmode=none\tdomains=<none>".to_owned());
    }
    Ok(render_project_abi_graph_line(&resolution))
}

pub(super) fn render_project_host_ffi_index(project: &LoadedProject) -> String {
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

pub(super) fn render_ast_type_ref(ty: &AstTypeRef) -> String {
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
