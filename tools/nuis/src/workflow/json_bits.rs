use super::*;

fn artifact_lowering_unit_json(
    unit: &nuisc::aot::NuisCompiledArtifactLoweringUnitInspect,
) -> String {
    let fields = [
        json_field("package_id", &unit.package_id),
        json_field("domain_family", &unit.domain_family),
        json_optional_string_field("backend_family", unit.backend_family.as_deref()),
        json_optional_string_field(
            "selected_lowering_target",
            unit.selected_lowering_target.as_deref(),
        ),
        json_optional_string_field(
            "artifact_ir_sidecar_path",
            unit.artifact_ir_sidecar_path.as_deref(),
        ),
        json_field("contract_family", &unit.contract_family),
        json_field("packaging_role", &unit.packaging_role),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn artifact_lowering_units_json(
    units: &[nuisc::aot::NuisCompiledArtifactLoweringUnitInspect],
) -> String {
    let entries = units
        .iter()
        .map(artifact_lowering_unit_json)
        .collect::<Vec<_>>()
        .join(",");
    format!("\"lowering_units\":[{}]", entries)
}

#[allow(dead_code)]
pub(crate) fn json_object_field(name: &str, fields: &[String]) -> String {
    let mut out = String::new();
    out.push('"');
    out.push_str(name);
    out.push_str("\":{");
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(field);
    }
    out.push('}');
    out
}

pub(crate) fn append_json_object_fields(base_json: &str, fields: &[String]) -> String {
    let mut out = base_json.to_owned();
    if out.ends_with('}') {
        out.pop();
        if !fields.is_empty() {
            out.push(',');
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(field);
            }
        }
        out.push('}');
    }
    out
}

pub(crate) fn json_object_array_field(name: &str, values: &[String]) -> String {
    let mut out = String::new();
    out.push('"');
    out.push_str(name);
    out.push_str("\":[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

pub(crate) fn project_domain_registry_checks_json(
    checks: &[nuisc::registry::ProjectDomainRegistryCheck],
) -> Vec<String> {
    checks
        .iter()
        .map(nuisc::registry::project_domain_registry_check_json)
        .collect()
}

pub(crate) fn project_lowering_checks_json(
    checks: &[nuisc::project::ProjectLoweringSelectionView],
) -> Vec<String> {
    checks
        .iter()
        .map(nuisc::project::project_lowering_selection_json)
        .collect()
}

pub(crate) fn project_abi_checks_json(
    checks: &[nuisc::project::ProjectAbiSelectionCheck],
) -> Vec<String> {
    checks
        .iter()
        .map(nuisc::project::project_abi_selection_check_json)
        .collect()
}
