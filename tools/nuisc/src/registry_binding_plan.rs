use std::{collections::BTreeSet, path::Path};

use crate::registry::{
    execution_summary, load_manifest_for_domain, load_required_manifests, validate_unit_binding,
    NustarBinding, NustarBindingPlan,
};
use crate::registry_support_usage::{
    collect_resource_usage_hints, covered_profile_slots, detect_matched_support_usage,
};
use nuis_semantics::model::NirModule;
use yir_core::YirModule;

pub fn plan_bindings(
    root: &Path,
    nir: &NirModule,
    module: &YirModule,
    domain: &str,
    unit: &str,
    declared_used_units: &[(String, String)],
    declared_externs: &[(String, String)],
) -> Result<NustarBindingPlan, String> {
    let mut manifests = load_required_manifests(root, module)?;
    let mut loaded_domains = manifests
        .iter()
        .map(|manifest| manifest.domain_family.clone())
        .collect::<BTreeSet<_>>();
    if loaded_domains.insert(domain.to_owned()) {
        manifests.push(load_manifest_for_domain(root, domain)?);
    }
    for (used_domain, _) in declared_used_units {
        if loaded_domains.insert(used_domain.clone()) {
            manifests.push(load_manifest_for_domain(root, used_domain)?);
        }
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    validate_unit_binding(&manifests, domain, unit)?;
    let mut bindings = Vec::new();

    for manifest in manifests {
        let execution = execution_summary(&manifest);
        let registered_units = manifest
            .unit_types
            .iter()
            .filter(|unit| !unit.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        let bound_unit = if manifest.domain_family == domain {
            Some(unit.to_owned())
        } else {
            None
        };
        let used_units = declared_used_units
            .iter()
            .filter(|(used_domain, _)| used_domain == &manifest.domain_family)
            .map(|(_, used_unit)| used_unit.clone())
            .collect::<Vec<_>>();
        let instantiated_units = module
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "instantiate_unit"
                    && node.op.args.first().map(String::as_str)
                        == Some(manifest.domain_family.as_str())
            })
            .filter_map(|node| node.op.args.get(1).cloned())
            .collect::<Vec<_>>();
        let used_host_ffi_abis = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(abi, _)| abi.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let used_host_ffi_symbols = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(_, symbol)| symbol.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let (matched_support_surface, matched_support_profile_slots) =
            detect_matched_support_usage(nir, &manifest.domain_family);
        let covered_support_profile_slots = covered_profile_slots(
            &manifest.domain_family,
            &matched_support_surface,
            &matched_support_profile_slots,
        );
        let uncovered_support_profile_slots = manifest
            .support_profile_slots
            .iter()
            .filter(|slot| {
                !covered_support_profile_slots
                    .iter()
                    .any(|covered| covered == *slot)
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut matched_resources = module
            .resources
            .iter()
            .filter(|resource| {
                manifest
                    .resource_families
                    .iter()
                    .any(|family| family == resource.kind.family())
            })
            .map(|resource| resource.name.clone())
            .collect::<BTreeSet<_>>();
        collect_resource_usage_hints(nir, &manifest.domain_family, &mut matched_resources);
        let matched_resources = matched_resources.into_iter().collect::<Vec<_>>();

        let matched_ops = module
            .nodes
            .iter()
            .filter(|node| node.op.module == manifest.domain_family)
            .map(|node| node.op.full_name())
            .collect::<Vec<_>>();

        if matched_ops.is_empty() && instantiated_units.is_empty() && used_units.is_empty() {
            return Err(format!(
                "nustar package `{}` was selected but no matching ops were bound",
                manifest.package_id
            ));
        }

        let undeclared_ops = matched_ops
            .iter()
            .filter(|op| !manifest.ops.iter().any(|candidate| candidate == *op))
            .cloned()
            .collect::<Vec<_>>();

        bindings.push(NustarBinding {
            package_id: manifest.package_id,
            domain_family: manifest.domain_family,
            ast_entry: manifest.ast_entry,
            nir_entry: manifest.nir_entry,
            yir_lowering_entry: manifest.yir_lowering_entry,
            part_verify_entry: manifest.part_verify_entry,
            machine_abi_policy: manifest.machine_abi_policy,
            abi_profiles: manifest.abi_profiles,
            abi_capabilities: manifest.abi_capabilities,
            ast_surface: manifest.ast_surface,
            nir_surface: manifest.nir_surface,
            yir_lowering: manifest.yir_lowering,
            part_verify: manifest.part_verify,
            support_surface: manifest.support_surface,
            support_profile_slots: manifest.support_profile_slots,
            capability_tags: manifest.capability_tags,
            default_lanes: manifest.default_lanes,
            execution,
            matched_support_surface,
            matched_support_profile_slots,
            covered_support_profile_slots,
            uncovered_support_profile_slots,
            registered_units,
            bound_unit,
            used_units,
            instantiated_units,
            used_host_ffi_abis,
            used_host_ffi_symbols,
            matched_resources,
            matched_ops,
            undeclared_ops,
            frontend: manifest.frontend,
            entry_crate: manifest.entry_crate,
        });
    }

    bindings.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(NustarBindingPlan { bindings })
}
