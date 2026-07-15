use super::final_executable_image::parse_final_executable_image_header;
use super::toml;

const CONTAINER_TOML_SCHEMA_MARKER: &[u8] = b"schema = \"nuis-nsld-container-v1\"";
const CONTAINER_TOML_HANDOFF_END_MARKER: &[u8] = b"\nloader_symbol_count = ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalExecutableContainerLoaderEvidence {
    pub(crate) status: String,
    pub(crate) payload_scan_kind: String,
    pub(crate) parsed: bool,
    pub(crate) readiness: Option<String>,
    pub(crate) ready: Option<bool>,
    pub(crate) handoff_status: String,
    pub(crate) handoff_ready: bool,
    pub(crate) handoff_first_blocker: Option<String>,
    pub(crate) entry_symbol: Option<String>,
    pub(crate) entry_kind: Option<String>,
    pub(crate) entry_section_id: Option<String>,
    pub(crate) symbol_count: Option<usize>,
}

pub(crate) fn final_executable_container_loader_evidence(
    output_bytes: Option<&[u8]>,
    host_native_output: bool,
) -> FinalExecutableContainerLoaderEvidence {
    if host_native_output {
        return empty_evidence("not-required", "host-native-output", None);
    }
    let Some(bytes) = output_bytes else {
        return empty_evidence("missing-output", "none", Some("final-output:missing"));
    };
    let Some(header) = parse_final_executable_image_header(bytes) else {
        return empty_evidence(
            "image-header-missing",
            "none",
            Some("container-loader:image-header-missing"),
        );
    };
    let payload_end = header.payload_offset.saturating_add(header.payload_span);
    if header.payload_offset > bytes.len() || payload_end > bytes.len() {
        return empty_evidence(
            "payload-range-invalid",
            "none",
            Some("container-loader:payload-range-invalid"),
        );
    }
    let payload = &bytes[header.payload_offset..payload_end];
    let prefix = match container_toml_capsule(payload) {
        Ok(Some(prefix)) => prefix,
        Ok(None) => {
            return empty_evidence(
                "not-container-toml",
                "payload-without-container-toml-capsule",
                Some("container-loader:toml-prefix-missing"),
            );
        }
        Err(()) => {
            return empty_evidence(
                "invalid-utf8",
                "nsld-container-toml",
                Some("container-loader:invalid-utf8"),
            );
        }
    };

    let readiness = toml::string_value(prefix, "loader_readiness");
    let ready = toml::bool_value(prefix, "ready");
    let entry_symbol = toml::string_value(prefix, "loader_entry_symbol");
    let entry_kind = toml::string_value(prefix, "loader_entry_kind");
    let entry_section_id = toml::string_value(prefix, "loader_entry_section_id");
    let symbol_count = toml::usize_value(prefix, "loader_symbol_count");
    let mut blockers = string_array_or_empty(prefix, "loader_blockers");
    if readiness.as_deref() == Some("host-assisted") {
        blockers.retain(|blocker| !blocker.starts_with("external-import:"));
    }

    if readiness.as_deref() == Some("blocked") {
        blockers.push("container-loader:readiness-blocked".to_owned());
    } else if readiness.is_none() {
        blockers.push("container-loader:readiness-missing".to_owned());
    }
    if entry_symbol.as_deref().unwrap_or_default().is_empty() {
        blockers.push("container-loader:entry-symbol-missing".to_owned());
    }
    if entry_kind.as_deref().unwrap_or_default().is_empty() {
        blockers.push("container-loader:entry-kind-missing".to_owned());
    }
    if entry_section_id.as_deref().unwrap_or_default().is_empty() {
        blockers.push("container-loader:entry-section-missing".to_owned());
    }
    if symbol_count.unwrap_or_default() == 0 {
        blockers.push("container-loader:symbols-missing".to_owned());
    }

    let handoff_ready = blockers.is_empty();
    FinalExecutableContainerLoaderEvidence {
        status: "parsed".to_owned(),
        payload_scan_kind: "nsld-container-toml".to_owned(),
        parsed: true,
        readiness,
        ready,
        handoff_status: if handoff_ready { "ready" } else { "blocked" }.to_owned(),
        handoff_ready,
        handoff_first_blocker: blockers.first().cloned(),
        entry_symbol,
        entry_kind,
        entry_section_id,
        symbol_count,
    }
}

fn container_toml_capsule(payload: &[u8]) -> Result<Option<&str>, ()> {
    let mut search_offset = 0usize;
    let mut invalid_utf8_candidate_seen = false;
    while search_offset < payload.len() {
        let Some(relative_marker_offset) =
            find_bytes(&payload[search_offset..], CONTAINER_TOML_SCHEMA_MARKER)
        else {
            break;
        };
        let marker_offset = search_offset + relative_marker_offset;
        let capsule = &payload[marker_offset..];
        let Some(capsule_end) = container_toml_capsule_end(capsule) else {
            search_offset = marker_offset.saturating_add(1);
            continue;
        };
        match std::str::from_utf8(&capsule[..capsule_end]) {
            Ok(prefix)
                if toml::string_value(prefix, "schema").as_deref()
                    == Some("nuis-nsld-container-v1") =>
            {
                return Ok(Some(prefix));
            }
            Ok(_) => {}
            Err(_) => invalid_utf8_candidate_seen = true,
        }
        search_offset = marker_offset.saturating_add(1);
    }
    if invalid_utf8_candidate_seen {
        Err(())
    } else {
        Ok(None)
    }
}

fn container_toml_capsule_end(capsule: &[u8]) -> Option<usize> {
    if let Some(line_offset) = find_bytes(capsule, CONTAINER_TOML_HANDOFF_END_MARKER) {
        let line_start = line_offset.saturating_add(1);
        let line_end = capsule[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| line_start + offset + 1)?;
        return Some(line_end);
    }
    capsule.iter().position(|byte| *byte == 0)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn empty_evidence(
    status: &str,
    payload_scan_kind: &str,
    first_blocker: Option<&str>,
) -> FinalExecutableContainerLoaderEvidence {
    FinalExecutableContainerLoaderEvidence {
        status: status.to_owned(),
        payload_scan_kind: payload_scan_kind.to_owned(),
        parsed: false,
        readiness: None,
        ready: None,
        handoff_status: if first_blocker.is_some() {
            "blocked"
        } else {
            "not-required"
        }
        .to_owned(),
        handoff_ready: false,
        handoff_first_blocker: first_blocker.map(str::to_owned),
        entry_symbol: None,
        entry_kind: None,
        entry_section_id: None,
        symbol_count: None,
    }
}

fn string_array_or_empty(source: &str, key: &str) -> Vec<String> {
    toml::string_array_value(source, key)
}
