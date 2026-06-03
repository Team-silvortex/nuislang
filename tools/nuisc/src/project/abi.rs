use std::collections::BTreeSet;
use std::path::Path;

use super::{
    collect_project_domains, data_support_surface_contract, kernel_support_surface_contract,
    network_support_surface_contract, shader_support_surface_contract, split_domain_unit,
    LoadedProject, ProjectAbiRequirement, ProjectAbiResolution,
};

pub fn resolve_project_abi(project: &LoadedProject) -> Result<ProjectAbiResolution, String> {
    if !project.manifest.abi_requirements.is_empty() {
        let mut requirements = project.manifest.abi_requirements.clone();
        requirements.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
        return Ok(ProjectAbiResolution {
            requirements,
            explicit: true,
        });
    }
    let domains = collect_project_domains(&project.manifest, &project.modules)?;
    let mut requirements = Vec::new();
    for domain in domains {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &domain)?;
        let abi = recommend_abi_profile_for_host(&manifest)
            .ok_or_else(|| format!("domain `{domain}` has no ABI profiles to recommend"))?;
        requirements.push(ProjectAbiRequirement { domain, abi });
    }
    requirements.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
    Ok(ProjectAbiResolution {
        requirements,
        explicit: false,
    })
}

pub(super) fn recommend_abi_profile_for_host(
    manifest: &crate::registry::NustarPackageManifest,
) -> Option<String> {
    if manifest.abi_profiles.is_empty() {
        return None;
    }
    if let Some(profile) = recommend_registered_abi_profile_for_host(manifest) {
        return Some(profile);
    }
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let os_tokens: Vec<&str> = match os {
        "darwin" => vec!["darwin", "macos", "apple"],
        "linux" => vec!["linux"],
        "windows" => vec!["windows", "win64", "win32"],
        _ => vec![os],
    };

    let mut best = manifest.abi_profiles[0].clone();
    let mut best_score = i32::MIN;
    for profile in &manifest.abi_profiles {
        let lower = profile.to_ascii_lowercase();
        let mut score = 0i32;
        if lower.contains(&arch.to_ascii_lowercase()) {
            score += 40;
        }
        if os_tokens.iter().any(|token| lower.contains(token)) {
            score += 30;
        }
        if manifest.domain_family == "shader" {
            if os == "darwin" && lower.contains("metal") {
                score += 60;
            }
            if os == "windows" && (lower.contains("dx12") || lower.contains("dxil")) {
                score += 60;
            }
            if os == "linux" && (lower.contains("vulkan") || lower.contains("spv")) {
                score += 60;
            }
            if lower.contains("cpu-fallback") {
                score -= 10;
            }
        } else if manifest.domain_family == "kernel" {
            if os == "darwin" && (lower.contains("apple_ane") || lower.contains("coreml")) {
                score += 60;
            }
            if lower.contains("cpu-fallback") {
                score += 10;
            }
        }
        if score > best_score {
            best_score = score;
            best = profile.clone();
        }
    }
    Some(best)
}

fn recommend_registered_abi_profile_for_host(
    manifest: &crate::registry::NustarPackageManifest,
) -> Option<String> {
    let host_arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };
    let host_os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let mut best = None::<(i32, String)>;
    for profile in &manifest.abi_profiles {
        let Ok(target) = crate::registry::registered_abi_target(manifest, profile) else {
            continue;
        };
        let mut score = 0i32;
        if target.machine_arch == host_arch {
            score += 100;
        }
        if target.machine_os == host_os {
            score += 100;
        }
        if target.object_format == host_object_format() {
            score += 40;
        }
        if target.calling_abi == host_calling_abi(host_arch, host_os) {
            score += 40;
        }
        if !target.host_adaptive {
            score += 15;
        }
        if manifest.domain_family == "shader" {
            let backend = target
                .backend_family
                .as_deref()
                .unwrap_or(profile.as_str())
                .to_ascii_lowercase();
            if host_os == "darwin" && backend.contains("metal") {
                score += 60;
            }
            if host_os == "windows" && (backend.contains("dx12") || backend.contains("dxil")) {
                score += 60;
            }
            if host_os == "linux" && (backend.contains("vulkan") || backend.contains("spv")) {
                score += 60;
            }
            if backend.contains("cpu-fallback") {
                score -= 20;
            }
        } else if manifest.domain_family == "kernel" {
            let backend = target
                .backend_family
                .as_deref()
                .unwrap_or(profile.as_str())
                .to_ascii_lowercase();
            if host_os == "darwin" && (backend.contains("apple_ane") || backend.contains("coreml"))
            {
                score += 60;
            }
            if backend.contains("cpu-fallback") {
                score -= 10;
            }
        }
        match &best {
            Some((best_score, _)) if *best_score >= score => {}
            _ => best = Some((score, profile.clone())),
        }
    }
    best.map(|(_, profile)| profile)
}

pub(super) fn host_object_format() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        _ => "unknown",
    }
}

pub(super) fn host_calling_abi(host_arch: &str, host_os: &str) -> &'static str {
    match (host_arch, host_os) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

pub(super) fn required_abi_surfaces_for_domain(
    project: &LoadedProject,
    domain: &str,
) -> Result<Vec<String>, String> {
    let mut surfaces = BTreeSet::new();
    for link in &project.manifest.links {
        let (from_domain, _) = split_domain_unit(&link.from)?;
        let (to_domain, _) = split_domain_unit(&link.to)?;
        let via_domain = link
            .via
            .as_ref()
            .map(|via| split_domain_unit(via).map(|(d, _)| d))
            .transpose()?;
        let domain_is_in_link =
            from_domain == domain || to_domain == domain || via_domain.as_deref() == Some(domain);
        if !domain_is_in_link {
            continue;
        }
        match domain {
            "shader" => {
                for surface in shader_support_surface_contract() {
                    surfaces.insert((*surface).to_owned());
                }
            }
            "kernel" => {
                for surface in kernel_support_surface_contract() {
                    surfaces.insert((*surface).to_owned());
                }
            }
            "network" => {
                for surface in network_support_surface_contract() {
                    surfaces.insert((*surface).to_owned());
                }
            }
            "data" => {
                for surface in data_support_surface_contract() {
                    surfaces.insert((*surface).to_owned());
                }
                surfaces.insert("data.profile.send.uplink.v1".to_owned());
                surfaces.insert("data.profile.send.downlink.v1".to_owned());
            }
            _ => {}
        }
    }
    Ok(surfaces.into_iter().collect())
}
