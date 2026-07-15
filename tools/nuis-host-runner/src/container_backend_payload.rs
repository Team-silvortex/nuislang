use crate::container_toml::{array_table_blocks, string_value_from_lines};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BackendArtifactPayloadSummary {
    pub(super) status: String,
    pub(super) declared_count: Option<usize>,
    pub(super) parsed_count: usize,
    pub(super) first_payload_id: Option<String>,
    pub(super) first_payload_kind: Option<String>,
    pub(super) first_role_status: Option<String>,
    pub(super) ready_count: usize,
}

impl BackendArtifactPayloadSummary {
    pub(super) fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            declared_count: None,
            parsed_count: 0,
            first_payload_id: None,
            first_payload_kind: None,
            first_role_status: None,
            ready_count: 0,
        }
    }
}

pub(super) fn scan_backend_artifact_payloads(
    source: &str,
    declared_count: Option<usize>,
) -> BackendArtifactPayloadSummary {
    let blocks = array_table_blocks(source, "backend_artifact_payload");
    let first = blocks.first();
    let ready_count = blocks
        .iter()
        .filter(|block| string_value_from_lines(block, "role_status").as_deref() == Some("ready"))
        .count();
    let first_payload_kind = first.map(|block| {
        let domain =
            string_value_from_lines(block, "domain_family").unwrap_or_else(|| "unknown".to_owned());
        let backend = string_value_from_lines(block, "backend_family")
            .unwrap_or_else(|| "unknown".to_owned());
        let target =
            string_value_from_lines(block, "target_device").unwrap_or_else(|| "unknown".to_owned());
        format!("nustar-backend-artifact:{domain}:{backend}:{target}")
    });
    BackendArtifactPayloadSummary {
        status: if blocks.is_empty() {
            "missing".to_owned()
        } else {
            "parsed".to_owned()
        },
        declared_count,
        parsed_count: blocks.len(),
        first_payload_id: first.and_then(|block| string_value_from_lines(block, "payload_id")),
        first_payload_kind,
        first_role_status: first.and_then(|block| string_value_from_lines(block, "role_status")),
        ready_count,
    }
}
