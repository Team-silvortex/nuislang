use crate::model::{NsdbLoweringUnitDebugInfo, NsdbSidecarDebugInfo};
use std::fs;

pub(crate) fn read_sidecar_debug_info(
    unit: &NsdbLoweringUnitDebugInfo,
) -> Option<NsdbSidecarDebugInfo> {
    let source = fs::read_to_string(&unit.artifact_ir_sidecar_path).ok()?;
    Some(NsdbSidecarDebugInfo {
        domain_family: unit.domain_family.clone(),
        package_id: unit.package_id.clone(),
        path: unit.artifact_ir_sidecar_path.clone(),
        schema: toml_string_value(&source, "schema").unwrap_or_else(|| "unknown".to_owned()),
        capability_owner: toml_string_value(&source, "capability_owner")
            .unwrap_or_else(|| "unknown".to_owned()),
        frontend_ir: toml_string_value(&source, "frontend_ir")
            .unwrap_or_else(|| "unknown".to_owned()),
        native_ir: toml_string_value(&source, "native_ir").unwrap_or_else(|| "unknown".to_owned()),
        pipeline_lowering: toml_string_value(&source, "pipeline_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        resource_lowering: toml_string_value(&source, "resource_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        dispatch_lowering: toml_string_value(&source, "dispatch_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        texture_lowering: toml_string_value(&source, "texture_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        transport_lowering: toml_string_value(&source, "transport_lowering")
            .or_else(|| toml_string_value(&source, "transport_binding_model"))
            .unwrap_or_else(|| "none".to_owned()),
        tensor_lowering: toml_string_value(&source, "tensor_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        memory_lowering: toml_string_value(&source, "memory_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        result_lowering: toml_string_value(&source, "result_lowering")
            .unwrap_or_else(|| "none".to_owned()),
        validation_contracts: toml_string_array_value(&source, "validation_contracts"),
        entry_symbol: sidecar_entry_symbol(&source),
        stage_kind: toml_string_value(&source, "stage_kind").unwrap_or_else(|| "none".to_owned()),
    })
}

fn sidecar_entry_symbol(source: &str) -> String {
    [
        "entry_symbol",
        "fragment",
        "vertex",
        "compute",
        "graph",
        "grid",
        "range",
        "connect",
    ]
    .into_iter()
    .find_map(|key| toml_string_value(source, key))
    .unwrap_or_else(|| "none".to_owned())
}

fn toml_string_value(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        if found_key.trim() != key {
            return None;
        }
        let value = value.trim();
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(|value| {
                value
                    .replace("\\n", "\n")
                    .replace("\\\"", "\"")
                    .replace("\\\\", "\\")
            })
    })
}

fn toml_string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(value) = source.lines().find_map(|raw| {
        let line = raw.trim();
        let (found_key, value) = line.split_once('=')?;
        (found_key.trim() == key).then(|| value.trim().to_owned())
    }) else {
        return Vec::new();
    };
    let Some(body) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            entry
                .strip_prefix('"')
                .and_then(|entry| entry.strip_suffix('"'))
                .map(str::to_owned)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{read_sidecar_debug_info, sidecar_entry_symbol};
    use crate::model::NsdbLoweringUnitDebugInfo;
    use std::{env, fs};

    #[test]
    fn normalizes_non_shader_entry_symbols() {
        let kernel_source = r#"
schema = "nuis-kernel-ir-sidecar-v1"
[entry_points]
graph = "infer_main"
batch = "infer_batch"
"#;
        let network_source = r#"
schema = "nuis-network-ir-sidecar-v1"
[entry_points]
connect = "open_fd_session"
send = "submit_send_recv"
"#;

        assert_eq!(sidecar_entry_symbol(kernel_source), "infer_main");
        assert_eq!(sidecar_entry_symbol(network_source), "open_fd_session");
    }

    #[test]
    fn reads_network_sidecar_capability_metadata() {
        let path =
            env::temp_dir().join(format!("nsdb-network-sidecar-{}.toml", std::process::id()));
        fs::write(
            &path,
            r#"
schema = "nuis-network-ir-sidecar-v1"
[lowering_capabilities]
capability_owner = "network-nustar"
frontend_ir = "nuis-yir.network"
native_ir = "posix-socket"
transport_lowering = "packet-poll-reactor"
dispatch_lowering = "poll-send-recv-submit"
validation_contracts = ["glm.fd-handle-lifetime", "time.poll-ready-order"]
[entry_points]
connect = "open_fd_session"
"#,
        )
        .unwrap();
        let unit = NsdbLoweringUnitDebugInfo {
            index: 0,
            package_id: "official.network".to_owned(),
            domain_family: "network".to_owned(),
            backend_family: "socket-abi".to_owned(),
            selected_lowering_target: "socket-abi.socket-io".to_owned(),
            artifact_ir_sidecar_path: path.display().to_string(),
            contract_family: "nustar.network".to_owned(),
            packaging_role: "hetero-contract".to_owned(),
        };

        let sidecar = read_sidecar_debug_info(&unit).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(sidecar.capability_owner, "network-nustar");
        assert_eq!(sidecar.frontend_ir, "nuis-yir.network");
        assert_eq!(sidecar.native_ir, "posix-socket");
        assert_eq!(sidecar.transport_lowering, "packet-poll-reactor");
        assert_eq!(sidecar.dispatch_lowering, "poll-send-recv-submit");
        assert_eq!(sidecar.entry_symbol, "open_fd_session");
        assert_eq!(
            sidecar.validation_contracts,
            vec![
                "glm.fd-handle-lifetime".to_owned(),
                "time.poll-ready-order".to_owned()
            ]
        );
    }
}
