use std::{collections::BTreeSet, fs, path::Path};

use crate::registry::{
    domain_contract, manifest_path, NustarDomainRegistration, NustarPackageIndexEntry,
    NustarPackageManifest, NustarRegistryIssue, NustarRegistryIssueKind,
    ProjectDomainRegistryCheck, ProjectDomainRegistryIssue, ProjectDomainRegistryIssueKind,
    NUSTAR_DOMAIN_CONTRACT_SCHEMA,
};
use crate::registry_domain_contract_validate::{
    validate_build_contract_fields, validate_domain_specific_contracts,
};
use crate::registry_load::{load_index, resolve_registry_root, INDEX_FILE};
use crate::registry_manifest_parse::parse_manifest;

pub fn domain_registration(
    root: &Path,
    entry: &NustarPackageIndexEntry,
) -> Result<NustarDomainRegistration, String> {
    let root = resolve_registry_root(root);
    let path = manifest_path(&root, entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let manifest = parse_manifest(&source, &path)?;
    Ok(NustarDomainRegistration {
        manifest_path: path.display().to_string(),
        package_id: manifest.package_id.clone(),
        domain_family: manifest.domain_family.clone(),
        frontend: manifest.frontend.clone(),
        entry_crate: manifest.entry_crate.clone(),
        ast_entry: manifest.ast_entry.clone(),
        nir_entry: manifest.nir_entry.clone(),
        yir_lowering_entry: manifest.yir_lowering_entry.clone(),
        part_verify_entry: manifest.part_verify_entry.clone(),
        ast_surface: manifest.ast_surface.clone(),
        nir_surface: manifest.nir_surface.clone(),
        yir_lowering: manifest.yir_lowering.clone(),
        part_verify: manifest.part_verify.clone(),
        resource_families: manifest.resource_families.clone(),
        unit_types: manifest.unit_types.clone(),
        lowering_targets: manifest.lowering_targets.clone(),
        ops: manifest.ops.clone(),
        contract: domain_contract(&manifest),
    })
}

fn load_registered_domains_unvalidated(
    root: &Path,
) -> Result<Vec<NustarDomainRegistration>, String> {
    let root = resolve_registry_root(root);
    let mut registrations = load_index(&root)?
        .into_iter()
        .map(|entry| domain_registration(&root, &entry))
        .collect::<Result<Vec<_>, _>>()?;
    registrations.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(registrations)
}

fn lane_target_from_entry(entry: &str) -> Option<&str> {
    let (target, _) = entry.split_once('=')?;
    let target = target.trim();
    if target.is_empty() {
        None
    } else {
        Some(target)
    }
}

fn lane_target_is_declared(manifest: &NustarPackageManifest, target: &str) -> bool {
    if manifest.ops.iter().any(|op| op == target) {
        return true;
    }
    let prefix = format!("{}.", manifest.domain_family);
    let Some(slot) = target.strip_prefix(&prefix) else {
        return false;
    };
    manifest
        .support_profile_slots
        .iter()
        .any(|candidate| candidate == slot)
}

pub fn validate_registered_domains(root: &Path) -> Result<Vec<NustarRegistryIssue>, String> {
    let root = resolve_registry_root(root);
    let index = load_index(&root)?;
    if index.is_empty() {
        return Ok(vec![NustarRegistryIssue {
            kind: NustarRegistryIssueKind::IndexEmpty,
            package: None,
            domain: None,
            manifest_path: Some(root.join(INDEX_FILE).display().to_string()),
            message: format!(
                "no nustar packages are indexed in `{}`",
                root.join(INDEX_FILE).display()
            ),
        }]);
    }

    let mut issues = Vec::new();
    let mut seen_packages = BTreeSet::new();
    for entry in &index {
        let manifest_path = manifest_path(&root, entry);
        if !seen_packages.insert(entry.package_id.clone()) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DuplicatePackageId,
                package: Some(entry.package_id.clone()),
                domain: Some(entry.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "package `{}` appears more than once in `{}`",
                    entry.package_id,
                    root.join(INDEX_FILE).display()
                ),
            });
        }

        let source = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
        let manifest = parse_manifest(&source, &manifest_path)?;

        if manifest.package_id != entry.package_id {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::PackageIdentityMismatch,
                package: Some(entry.package_id.clone()),
                domain: Some(entry.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "index package `{}` does not match manifest package `{}`",
                    entry.package_id, manifest.package_id
                ),
            });
        }
        if manifest.domain_family != entry.domain_family {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainFamilyMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(entry.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "index domain `{}` does not match manifest domain `{}`",
                    entry.domain_family, manifest.domain_family
                ),
            });
        }
        if manifest.manifest_schema != "nustar-manifest-v1" {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::ManifestSchemaMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "manifest schema `{}` is not supported; expected `nustar-manifest-v1`",
                    manifest.manifest_schema
                ),
            });
        }
        if manifest.loader_abi != "nustar-loader-v1"
            || manifest.loader_entry != "nustar.bootstrap.v1"
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::LoaderContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "loader contract must be `nustar-loader-v1` + `nustar.bootstrap.v1`, got abi=`{}` entry=`{}`",
                    manifest.loader_abi, manifest.loader_entry
                ),
            });
        }
        if !manifest
            .resource_families
            .iter()
            .any(|family| family == &manifest.domain_family)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::ResourceFamilyContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "resource_families must include the owning domain `{}`",
                    manifest.domain_family
                ),
            });
        }
        let op_prefix = format!("{}.", manifest.domain_family);
        let invalid_ops = manifest
            .ops
            .iter()
            .filter(|op| !op.starts_with(&op_prefix))
            .cloned()
            .collect::<Vec<_>>();
        if !invalid_ops.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::OpContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "ops must stay inside domain prefix `{}`; invalid ops: {}",
                    op_prefix,
                    invalid_ops.join(", ")
                ),
            });
        }
        let invalid_lane_targets = manifest
            .default_lanes
            .iter()
            .filter_map(|entry| {
                let target = lane_target_from_entry(entry)?;
                if lane_target_is_declared(&manifest, target) {
                    None
                } else {
                    Some(target.to_owned())
                }
            })
            .collect::<Vec<_>>();
        if !invalid_lane_targets.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::LaneContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "default_lanes reference undeclared ops: {}",
                    invalid_lane_targets.join(", ")
                ),
            });
        }
        if let Err(error) = crate::nustar_binary::validate_manifest_for_packaging(&manifest) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::PackagingContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: error,
            });
        }
        let missing_contract_groups =
            crate::registry_contract::missing_domain_contract_groups(&manifest);
        if !missing_contract_groups.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "domain contract is incomplete; missing groups: {}",
                    missing_contract_groups.join(", ")
                ),
            });
        }
        issues.extend(validate_build_contract_fields(&manifest, &manifest_path));
        issues.extend(validate_domain_specific_contracts(
            &manifest,
            &manifest_path,
        ));
    }

    Ok(issues)
}

pub fn ensure_registered_domains_valid(root: &Path) -> Result<(), String> {
    let issues = validate_registered_domains(root)?;
    if issues.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "nustar registry validation failed:\n{}",
            issues
                .iter()
                .map(NustarRegistryIssue::summary)
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

pub fn load_registered_domains(root: &Path) -> Result<Vec<NustarDomainRegistration>, String> {
    ensure_registered_domains_valid(root)?;
    load_registered_domains_unvalidated(root)
}

pub fn load_domain_registration_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarDomainRegistration, String> {
    let root = resolve_registry_root(root);
    let entry = load_index(&root)?
        .into_iter()
        .find(|entry| entry.domain_family == domain_family)
        .ok_or_else(|| {
            format!(
                "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    domain_registration(&root, &entry)
}

pub fn validate_project_domain_registry(
    plan: &crate::project::ProjectCompilationPlan,
) -> Vec<ProjectDomainRegistryCheck> {
    plan.abi_resolution
        .requirements
        .iter()
        .map(|item| {
            let mut issues = Vec::new();
            let mut package = None;
            let mut contract_schema = None;
            let mut abi_registered = false;
            match load_domain_registration_for_domain(Path::new("nustar-packages"), &item.domain) {
                Ok(registration) => {
                    package = Some(registration.package_id.clone());
                    contract_schema = Some(registration.contract.contract_schema.clone());
                    if registration.contract.contract_schema != NUSTAR_DOMAIN_CONTRACT_SCHEMA {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ContractSchemaMismatch,
                            message: format!(
                                "unexpected contract schema `{}`",
                                registration.contract.contract_schema
                            ),
                        });
                    }
                    abi_registered = registration
                        .contract
                        .abi_profiles
                        .iter()
                        .any(|candidate| candidate == &item.abi);
                    if !abi_registered {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::AbiNotRegistered,
                            message: format!(
                                "abi `{}` is not declared by registered profiles",
                                item.abi
                            ),
                        });
                    }
                    if registration.contract.execution.execution_domain != item.domain {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ExecutionContractMismatch,
                            message: format!(
                                "execution domain `{}` does not match project domain `{}`",
                                registration.contract.execution.execution_domain, item.domain
                            ),
                        });
                    }
                    if registration.contract.execution.contract_family
                        != format!("nustar.{}", item.domain)
                    {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ExecutionContractMismatch,
                            message: format!(
                                "execution contract family `{}` does not match expected `nustar.{}`",
                                registration.contract.execution.contract_family, item.domain
                            ),
                        });
                    }
                    if registration.contract.execution.lowering_targets.is_empty() {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ExecutionContractMismatch,
                            message: "execution skeleton declares no lowering targets".to_owned(),
                        });
                    }
                }
                Err(error) => issues.push(ProjectDomainRegistryIssue {
                    kind: ProjectDomainRegistryIssueKind::DomainNotRegistered,
                    message: error,
                }),
            }
            ProjectDomainRegistryCheck {
                domain: item.domain.clone(),
                package,
                contract_schema,
                abi: Some(item.abi.clone()),
                abi_registered,
                ok: issues.is_empty(),
                issues,
            }
        })
        .collect()
}

pub fn ensure_project_domain_registry_valid(
    plan: &crate::project::ProjectCompilationPlan,
) -> Result<(), String> {
    let checks = validate_project_domain_registry(plan);
    let failures = checks
        .iter()
        .filter(|check| !check.ok)
        .map(ProjectDomainRegistryCheck::summary_line)
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "project domain registry validation failed:\n{}",
            failures.join("\n")
        ))
    }
}
