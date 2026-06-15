use nuis_semantics::model::{AstExternFunction, AstTypeRef};

use super::{
    profile_apply::resolve_registered_abi_target, resolve_project_abi, LoadedProject,
    ProjectAbiResolution, ProjectExchangeOrganization, ProjectExchangeRoute, ProjectOrganization,
    ProjectOrganizationLink, ProjectOrganizationModule,
};

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
    for item in &resolution.requirements {
        let target = resolve_registered_abi_target(&item.domain, Some(&resolution))
            .ok()
            .flatten();
        let arch = target
            .as_ref()
            .map(|target| target.machine_arch.as_str())
            .unwrap_or("unknown");
        let os = target
            .as_ref()
            .map(|target| target.machine_os.as_str())
            .unwrap_or("unknown");
        let object = target
            .as_ref()
            .map(|target| target.object_format.as_str())
            .unwrap_or("unknown");
        let calling = target
            .as_ref()
            .map(|target| target.calling_abi.as_str())
            .unwrap_or("unknown");
        let backend = target
            .as_ref()
            .and_then(|target| target.backend_family.as_deref())
            .unwrap_or("none");
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
