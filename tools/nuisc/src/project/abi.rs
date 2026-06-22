use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use super::support_contracts::support_surface_for_domain;
use super::validation_core::collect_project_domains;
use super::{
    split_domain_unit, LoadedProject, ProjectAbiIssue, ProjectAbiIssueKind, ProjectAbiRequirement,
    ProjectAbiResolution, ProjectAbiSelectionCheck,
};

pub(crate) fn backend_family_for_registered_abi_target(
    domain: &str,
    target: &crate::registry::RegisteredAbiTarget,
) -> Option<String> {
    match domain {
        "shader" | "kernel" => target.backend_family.clone(),
        "network" => Some(match target.machine_os.as_str() {
            "darwin" => "urlsession".to_owned(),
            "windows" => "winsock".to_owned(),
            _ => "socket".to_owned(),
        }),
        _ => None,
    }
}

pub(crate) fn selected_lowering_target_for_registered_abi_target(
    domain: &str,
    target: &crate::registry::RegisteredAbiTarget,
    registered_lowering_targets: &[String],
) -> Option<String> {
    let base = match domain {
        "shader" | "kernel" => target.backend_family.clone()?,
        "network" => Some(match target.machine_os.as_str() {
            "darwin" => "urlsession".to_owned(),
            "windows" => "winsock".to_owned(),
            _ => "socket-abi".to_owned(),
        })?,
        _ => return None,
    };
    let mut candidates = Vec::new();
    if let Some(device_class) = target.device_class.as_deref() {
        candidates.push(format!("{base}.{device_class}"));
    }
    if let Some(vendor) = target.vendor.as_deref() {
        candidates.push(format!("{base}.{vendor}"));
    }
    candidates.push(base.clone());
    for candidate in &candidates {
        if registered_lowering_targets.iter().any(|item| item == candidate) {
            return Some(candidate.clone());
        }
    }
    candidates.into_iter().next()
}

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

pub fn validate_project_abi_selections(
    project: &LoadedProject,
    resolution: &ProjectAbiResolution,
) -> Result<Vec<ProjectAbiSelectionCheck>, String> {
    let project_domains = collect_project_domains(&project.manifest, &project.modules)?;
    let mut views = resolution
        .requirements
        .iter()
        .map(|item| {
            let mut issues = Vec::new();
            let mut abi_registered = false;
            if resolution.explicit && !project_domains.contains(&item.domain) {
                issues.push(ProjectAbiIssue {
                    kind: ProjectAbiIssueKind::UnusedExplicitDomainAbi,
                    message: format!(
                        "project manifest ABI requirement `{}` targets domain `{}` which is not used by this project",
                        item.abi, item.domain
                    ),
                });
            }
            match crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &item.domain) {
                Ok(manifest) => {
                    abi_registered = manifest
                        .abi_profiles
                        .iter()
                        .any(|profile| profile == &item.abi);
                    if !abi_registered {
                        issues.push(ProjectAbiIssue {
                            kind: ProjectAbiIssueKind::AbiNotRegistered,
                            message: format!(
                                "project requires ABI `{}` for domain `{}`, but nustar package `{}` declares [{}]",
                                item.abi,
                                item.domain,
                                manifest.package_id,
                                if manifest.abi_profiles.is_empty() {
                                    "<none>".to_owned()
                                } else {
                                    manifest.abi_profiles.join(", ")
                                }
                            ),
                        });
                    }
                }
                Err(error) => issues.push(ProjectAbiIssue {
                    kind: ProjectAbiIssueKind::DomainNotRegistered,
                    message: error,
                }),
            }
            ProjectAbiSelectionCheck {
                domain: item.domain.clone(),
                abi: Some(item.abi.clone()),
                source: if resolution.explicit {
                    "explicit".to_owned()
                } else {
                    "recommended".to_owned()
                },
                abi_registered,
                ok: issues.is_empty(),
                issues,
            }
        })
        .collect::<Vec<_>>();
    if resolution.explicit {
        let required_domains = resolution
            .requirements
            .iter()
            .map(|item| item.domain.clone())
            .collect::<BTreeSet<_>>();
        for domain in project_domains.difference(&required_domains) {
            views.push(ProjectAbiSelectionCheck {
                domain: domain.clone(),
                abi: None,
                source: "explicit".to_owned(),
                abi_registered: false,
                ok: false,
                issues: vec![ProjectAbiIssue {
                    kind: ProjectAbiIssueKind::MissingExplicitDomainAbi,
                    message: format!(
                        "project manifest declares ABI locking but is missing domain ABI entry for `{}`",
                        domain
                    ),
                }],
            });
        }
    }
    views.sort_by(|lhs, rhs| lhs.domain.cmp(&rhs.domain));
    Ok(views)
}

pub fn render_project_abi_selection_check_lines(check: &ProjectAbiSelectionCheck) -> Vec<String> {
    let mut out = String::new();
    write_project_abi_selection_check_lines(&mut out, check)
        .expect("writing project abi selection check lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_abi_selection_check_lines<W: fmt::Write>(
    out: &mut W,
    check: &ProjectAbiSelectionCheck,
) -> fmt::Result {
    writeln!(
        out,
        "abi_check: {} source={} abi={} ok={} abi_registered={} issues={}",
        check.domain,
        check.source,
        check.abi.as_deref().unwrap_or("<none>"),
        if check.ok { "yes" } else { "no" },
        if check.abi_registered { "yes" } else { "no" },
        check.issue_count()
    )?;
    for issue in &check.issues {
        writeln!(out, "abi_issue: {}", issue.summary().replace(": ", " "))?;
    }
    Ok(())
}

pub fn project_abi_selection_check_json(check: &ProjectAbiSelectionCheck) -> String {
    let issues = check
        .issues
        .iter()
        .map(|issue| {
            format!(
                "{{\"code\":\"{}\",\"kind\":\"{}\",\"message\":\"{}\"}}",
                issue.kind.code(),
                issue.kind.as_str(),
                json_escape(&issue.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"domain\":\"{}\",\"abi\":{},\"source\":\"{}\",\"abi_registered\":{},\"ok\":{},\"issues\":[{}]}}",
        json_escape(&check.domain),
        check.abi
            .as_deref()
            .map(|value| format!("\"{}\"", json_escape(value)))
            .unwrap_or_else(|| "null".to_owned()),
        json_escape(&check.source),
        if check.abi_registered { "true" } else { "false" },
        if check.ok { "true" } else { "false" },
        issues
    )
}

pub fn ensure_project_abi_selections_valid(
    project: &LoadedProject,
    resolution: &ProjectAbiResolution,
) -> Result<(), String> {
    let failures = validate_project_abi_selections(project, resolution)?
        .into_iter()
        .filter(|check| !check.ok)
        .map(|check| check.summary_line())
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "project ABI selection validation failed:\n{}",
            failures.join("\n")
        ))
    }
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
        if let Some(preferred_vendor) =
            preferred_host_vendor_hint(&manifest.domain_family, host_arch, host_os)
        {
            if target.vendor.as_deref() == Some(preferred_vendor) {
                score += 25;
            }
        }
        if let Some(preferred_device) =
            preferred_host_device_hint(&manifest.domain_family, host_arch, host_os)
        {
            if target.device_class.as_deref() == Some(preferred_device) {
                score += 25;
            }
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

fn preferred_host_vendor_hint(
    domain_family: &str,
    host_arch: &str,
    host_os: &str,
) -> Option<&'static str> {
    match (domain_family, host_os, host_arch) {
        ("shader", "darwin", _) => Some("apple"),
        ("shader", "linux", _) => Some("cross-vendor"),
        ("shader", "windows", _) => Some("microsoft-runtime"),
        ("kernel", "darwin", _) => Some("apple"),
        ("network", "darwin", _) => Some("apple"),
        ("network", "linux", _) => Some("posix"),
        ("network", "windows", _) => Some("microsoft"),
        _ => None,
    }
}

fn preferred_host_device_hint(
    domain_family: &str,
    host_arch: &str,
    host_os: &str,
) -> Option<&'static str> {
    match (domain_family, host_os, host_arch) {
        ("shader", "darwin", "arm64") => Some("apple-silicon-gpu"),
        ("shader", "darwin", "x86_64") => Some("mac-discrete-or-integrated-gpu"),
        ("shader", "linux", _) => Some("discrete-or-integrated-gpu"),
        ("shader", "windows", _) => Some("discrete-or-integrated-gpu"),
        ("kernel", "darwin", "arm64") => Some("apple-ane"),
        ("network", _, _) => Some("socket-io"),
        _ => None,
    }
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
    let mut support_surface_cache = std::collections::BTreeMap::new();
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
        for surface in support_surface_for_domain(&mut support_surface_cache, domain)? {
            surfaces.insert(surface);
        }
    }
    Ok(surfaces.into_iter().collect())
}
