use std::collections::BTreeSet;
use std::path::Path;

use super::{
    packet, required_project_link_stage_contract, validate_project_link_stage_contract,
    NuisProjectManifest, ProjectModule,
};

pub(super) fn validate_project_modules(modules: &[ProjectModule]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for module in modules {
        let key = (module.ast.domain.clone(), module.ast.unit.clone());
        if !seen.insert(key.clone()) {
            return Err(format!(
                "duplicate project mod definition for `mod {} {}`",
                key.0, key.1
            ));
        }
        packet::validate_project_packet_contracts(module)?;
    }
    Ok(())
}

pub(super) fn validate_project_unit_bindings(modules: &[ProjectModule]) -> Result<(), String> {
    for module in modules {
        crate::registry::load_manifest_for_domain(
            Path::new("nustar-packages"),
            &module.ast.domain,
        )?;
    }
    Ok(())
}

pub(super) fn validate_project_uses(modules: &[ProjectModule]) -> Result<(), String> {
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

pub(super) fn validate_project_links(
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
            format!(
                "project link references unknown source unit `{}`",
                link.from
            )
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
            let contract = required_project_link_stage_contract(&link.from, &link.to, via)
                .map_err(|error| {
                    format!(
                        "project link `{}` -> `{}` via `{}` is not yet supported: {error}",
                        link.from, link.to, via
                    )
                })?;
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
            validate_project_link_stage_contract(&link.from, &link.to, via, contract)?;
        }
    }
    Ok(())
}

pub(super) fn validate_project_abi_requirements(
    manifest: &NuisProjectManifest,
    modules: &[ProjectModule],
) -> Result<(), String> {
    if manifest.abi_requirements.is_empty() {
        return Ok(());
    }

    let project_domains = collect_project_domains(manifest, modules)?;
    let mut required_domains = BTreeSet::new();
    for requirement in &manifest.abi_requirements {
        if !project_domains.contains(&requirement.domain) {
            return Err(format!(
                "project manifest ABI requirement `{}` targets domain `{}` which is not used by this project",
                requirement.abi, requirement.domain
            ));
        }
        let domain_manifest = crate::registry::load_manifest_for_domain(
            Path::new("nustar-packages"),
            &requirement.domain,
        )?;
        if !domain_manifest
            .abi_profiles
            .iter()
            .any(|profile| profile == &requirement.abi)
        {
            return Err(format!(
                "project requires ABI `{}` for domain `{}`, but nustar package `{}` declares [{}]",
                requirement.abi,
                requirement.domain,
                domain_manifest.package_id,
                if domain_manifest.abi_profiles.is_empty() {
                    "<none>".to_owned()
                } else {
                    domain_manifest.abi_profiles.join(", ")
                }
            ));
        }
        required_domains.insert(requirement.domain.clone());
    }

    let missing_domains = project_domains
        .difference(&required_domains)
        .cloned()
        .collect::<Vec<_>>();
    if !missing_domains.is_empty() {
        return Err(format!(
            "project manifest declares ABI locking but is missing domain ABI entries for: {}",
            missing_domains.join(", ")
        ));
    }
    Ok(())
}

pub(super) fn collect_project_domains(
    manifest: &NuisProjectManifest,
    modules: &[ProjectModule],
) -> Result<BTreeSet<String>, String> {
    let mut project_domains = modules
        .iter()
        .map(|module| module.ast.domain.clone())
        .collect::<BTreeSet<_>>();
    for link in &manifest.links {
        let (from_domain, _) = split_domain_unit(&link.from)?;
        let (to_domain, _) = split_domain_unit(&link.to)?;
        project_domains.insert(from_domain);
        project_domains.insert(to_domain);
        if let Some(via) = &link.via {
            let (via_domain, _) = split_domain_unit(via)?;
            project_domains.insert(via_domain);
        }
    }
    Ok(project_domains)
}

fn validate_external_unit_ref(reference: &str) -> Result<(), String> {
    let (domain, unit) = split_domain_unit(reference)?;
    let manifest =
        crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &domain)?;
    crate::registry::validate_unit_binding(&[manifest], &domain, &unit)
}

pub(super) fn split_domain_unit(reference: &str) -> Result<(String, String), String> {
    let Some((domain, unit)) = reference.split_once('.') else {
        return Err(format!(
            "project link reference `{reference}` must use `domain.Unit` form"
        ));
    };
    Ok((domain.trim().to_owned(), unit.trim().to_owned()))
}
