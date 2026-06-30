use super::*;

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
pub(in crate::project) fn render_project_organization_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_organization_index(&mut out, project)
        .expect("writing project organization index to String should not fail");
    out
}

pub(in crate::project) fn write_project_organization_index<W: fmt::Write>(
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
pub(in crate::project) fn render_project_exchange_index(project: &LoadedProject) -> String {
    let mut out = String::new();
    write_project_exchange_index(&mut out, project)
        .expect("writing project exchange index to String should not fail");
    out
}

pub(in crate::project) fn write_project_exchange_index<W: fmt::Write>(
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
