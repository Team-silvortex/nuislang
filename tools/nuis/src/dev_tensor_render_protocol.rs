use crate::{
    dev_tensor_drift::DevTensorDriftCheck, dev_tensor_status::DevTensorStatusProtocolEntry,
    json_bool_field, json_field, json_string_array_field, json_usize_field,
};

pub(crate) fn dev_tensor_status_protocol_json(entry: &DevTensorStatusProtocolEntry) -> String {
    format!(
        "{{{}}}",
        [
            json_field("status", entry.status),
            json_usize_field("rank", entry.rank),
            json_field("phase", entry.phase),
            json_bool_field("terminal", entry.terminal),
            json_bool_field("blocks_bootstrap", entry.blocks_bootstrap),
        ]
        .join(",")
    )
}

pub(crate) fn dev_tensor_drift_check_json(check: &DevTensorDriftCheck) -> String {
    format!(
        "{{{}}}",
        [
            json_field("id", check.id),
            json_field("path", check.path),
            json_bool_field("passed", check.passed),
            json_string_array_field("missing_patterns", &check.missing_patterns),
        ]
        .join(",")
    )
}
