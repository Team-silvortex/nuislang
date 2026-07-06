use crate::aot;

pub(crate) fn json_escape(value: &str) -> String {
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

pub(crate) fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

pub(crate) fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape(value))
}

pub(crate) fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", name, value)
}

pub(crate) fn json_i64_field(name: &str, value: i64) -> String {
    format!("\"{}\":{}", name, value)
}

pub(crate) fn json_string_array_field(name: &str, values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

pub(crate) fn json_optional_i64_field(name: &str, value: Option<i64>) -> String {
    match value {
        Some(value) => json_i64_field(name, value),
        None => format!("\"{}\":null", name),
    }
}

pub(crate) fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(name, value),
        None => format!("\"{}\":null", name),
    }
}

fn artifact_lowering_unit_json(unit: &aot::NuisCompiledArtifactLoweringUnitInspect) -> String {
    let fields = vec![
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_optional_string_field("backend_family", unit.backend_family.as_deref()),
        json_optional_string_field("target_device", unit.target_device.as_deref()),
        json_optional_string_field("ir_format", unit.ir_format.as_deref()),
        json_optional_string_field("dispatch_abi", unit.dispatch_abi.as_deref()),
        match unit.backend_priority {
            Some(value) => json_usize_field("backend_priority", value),
            None => "\"backend_priority\":null".to_owned(),
        },
        json_optional_string_field("verification", unit.verification.as_deref()),
        json_optional_string_field(
            "selected_lowering_target",
            unit.selected_lowering_target.as_deref(),
        ),
        json_optional_string_field(
            "artifact_ir_sidecar_path",
            unit.artifact_ir_sidecar_path.as_deref(),
        ),
        json_string_field("contract_family", &unit.contract_family),
        json_string_field("packaging_role", &unit.packaging_role),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn artifact_lowering_units_json(
    units: &[aot::NuisCompiledArtifactLoweringUnitInspect],
) -> String {
    let entries = units
        .iter()
        .map(artifact_lowering_unit_json)
        .collect::<Vec<_>>()
        .join(",");
    format!("\"lowering_units\":[{}]", entries)
}
