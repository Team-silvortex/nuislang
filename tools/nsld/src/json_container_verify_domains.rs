use super::{container::NsldContainerVerifyReport, json_fields::*};

pub(crate) fn expected_container_domain_fields_json(report: &NsldContainerVerifyReport) -> String {
    [
        json_bool_field(
            "expected_native_object_section_present",
            report.expected_native_object_section_present,
        ),
        json_string_field(
            "expected_native_object_section_id",
            &report.expected_native_object_section_id,
        ),
        json_bool_field(
            "expected_native_object_loader_symbol_present",
            report.expected_native_object_loader_symbol_present,
        ),
        json_string_field(
            "expected_native_object_loader_symbol_id",
            &report.expected_native_object_loader_symbol_id,
        ),
        json_bool_field(
            "expected_native_object_relocation_present",
            report.expected_native_object_relocation_present,
        ),
        json_string_field(
            "expected_native_object_relocation_id",
            &report.expected_native_object_relocation_id,
        ),
        expected_domain_fields_json(report, "shader"),
        expected_domain_fields_json(report, "kernel"),
    ]
    .join(",")
}

pub(crate) fn actual_container_domain_fields_json(report: &NsldContainerVerifyReport) -> String {
    [
        json_bool_field(
            "actual_native_object_section_present",
            report.actual_native_object_section_present,
        ),
        json_optional_string_field(
            "actual_native_object_section_id",
            report.actual_native_object_section_id.as_deref(),
        ),
        json_bool_field(
            "actual_native_object_loader_symbol_present",
            report.actual_native_object_loader_symbol_present,
        ),
        json_optional_string_field(
            "actual_native_object_loader_symbol_id",
            report.actual_native_object_loader_symbol_id.as_deref(),
        ),
        json_bool_field(
            "actual_native_object_relocation_present",
            report.actual_native_object_relocation_present,
        ),
        json_optional_string_field(
            "actual_native_object_relocation_id",
            report.actual_native_object_relocation_id.as_deref(),
        ),
        actual_domain_fields_json(report, "shader"),
        actual_domain_fields_json(report, "kernel"),
    ]
    .join(",")
}

fn expected_domain_fields_json(report: &NsldContainerVerifyReport, domain: &str) -> String {
    let (section_present, section_id, symbol_present, symbol_id, relocation_present, relocation_id) =
        match domain {
            "shader" => (
                report.expected_shader_section_present,
                report.expected_shader_section_id.as_str(),
                report.expected_shader_loader_symbol_present,
                report.expected_shader_loader_symbol_id.as_str(),
                report.expected_shader_relocation_present,
                report.expected_shader_relocation_id.as_str(),
            ),
            "kernel" => (
                report.expected_kernel_section_present,
                report.expected_kernel_section_id.as_str(),
                report.expected_kernel_loader_symbol_present,
                report.expected_kernel_loader_symbol_id.as_str(),
                report.expected_kernel_relocation_present,
                report.expected_kernel_relocation_id.as_str(),
            ),
            _ => return String::new(),
        };
    [
        json_bool_field(
            &format!("expected_{domain}_section_present"),
            section_present,
        ),
        json_string_field(&format!("expected_{domain}_section_id"), section_id),
        json_bool_field(
            &format!("expected_{domain}_loader_symbol_present"),
            symbol_present,
        ),
        json_string_field(&format!("expected_{domain}_loader_symbol_id"), symbol_id),
        json_bool_field(
            &format!("expected_{domain}_relocation_present"),
            relocation_present,
        ),
        json_string_field(&format!("expected_{domain}_relocation_id"), relocation_id),
    ]
    .join(",")
}

fn actual_domain_fields_json(report: &NsldContainerVerifyReport, domain: &str) -> String {
    let (section_present, section_id, symbol_present, symbol_id, relocation_present, relocation_id) =
        match domain {
            "shader" => (
                report.actual_shader_section_present,
                report.actual_shader_section_id.as_deref(),
                report.actual_shader_loader_symbol_present,
                report.actual_shader_loader_symbol_id.as_deref(),
                report.actual_shader_relocation_present,
                report.actual_shader_relocation_id.as_deref(),
            ),
            "kernel" => (
                report.actual_kernel_section_present,
                report.actual_kernel_section_id.as_deref(),
                report.actual_kernel_loader_symbol_present,
                report.actual_kernel_loader_symbol_id.as_deref(),
                report.actual_kernel_relocation_present,
                report.actual_kernel_relocation_id.as_deref(),
            ),
            _ => return String::new(),
        };
    [
        json_bool_field(&format!("actual_{domain}_section_present"), section_present),
        json_optional_string_field(&format!("actual_{domain}_section_id"), section_id),
        json_bool_field(
            &format!("actual_{domain}_loader_symbol_present"),
            symbol_present,
        ),
        json_optional_string_field(&format!("actual_{domain}_loader_symbol_id"), symbol_id),
        json_bool_field(
            &format!("actual_{domain}_relocation_present"),
            relocation_present,
        ),
        json_optional_string_field(&format!("actual_{domain}_relocation_id"), relocation_id),
    ]
    .join(",")
}
