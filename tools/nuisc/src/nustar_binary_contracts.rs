use super::*;

pub struct ImplementationContract {
    pub kind: String,
    pub loader_abi: String,
    pub entry_symbol: String,
    pub entry_signature: String,
    pub host_abi_struct: String,
    pub result_struct: String,
    pub status_convention: String,
    pub artifact_container: String,
    pub implementation_section: String,
    pub required_exports: Vec<String>,
    pub required_metadata: Vec<String>,
    pub link_mode: String,
    pub machine_abi_policy: String,
    pub notes: String,
}
fn render_abi_target_contracts(manifest: &NustarPackageManifest) -> Vec<String> {
    manifest
        .abi_targets
        .iter()
        .map(|entry| format!("abi_target={entry}"))
        .collect::<Vec<_>>()
}

fn render_runtime_symbol_contracts(manifest: &NustarPackageManifest) -> Vec<String> {
    match manifest.domain_family.as_str() {
        "network" => vec![
            "host_symbol=network.connect:host_network_connect_probe".to_owned(),
            "host_symbol=network.open_tcp:host_network_open_tcp_stream".to_owned(),
            "host_symbol=network.open_tcp_listener:host_network_open_tcp_listener".to_owned(),
            "host_symbol=network.open_udp:host_network_open_udp_datagram".to_owned(),
            "host_symbol=network.bind_udp:host_network_bind_udp_datagram".to_owned(),
            "host_symbol=network.accept:host_network_accept_probe".to_owned(),
            "host_symbol=network.accept_owned:host_network_accept_owned".to_owned(),
            "host_symbol=network.close:host_network_close".to_owned(),
            "host_symbol=network.close_owned:host_network_close_owned".to_owned(),
            "host_symbol=network.send_owned:host_network_send_owned".to_owned(),
            "host_symbol=network.recv_owned:host_network_recv_owned".to_owned(),
            "host_symbol=network.recv_http_status_owned:host_network_recv_http_status_owned"
                .to_owned(),
            "host_symbol=network.send:host_network_send_probe".to_owned(),
            "host_symbol=network.recv:host_network_recv_probe".to_owned(),
        ],
        _ => Vec::new(),
    }
}

fn append_runtime_symbol_notes(base: &str, manifest: &NustarPackageManifest) -> String {
    match manifest.domain_family.as_str() {
        "network" => format!(
            "{base}; runtime symbol contract currently reserves host_network_connect_probe, host_network_open_tcp_stream, host_network_open_tcp_listener, host_network_open_udp_datagram, host_network_bind_udp_datagram, host_network_accept_probe, host_network_accept_owned, host_network_close, host_network_close_owned, host_network_send_owned, host_network_recv_owned, host_network_recv_http_status_owned, host_network_send_probe, and host_network_recv_probe as the minimal control/transport syscall bridge surface"
        ),
        _ => base.to_owned(),
    }
}
pub fn implementation_contracts(binary: &NustarBinary) -> Vec<ImplementationContract> {
    binary
        .manifest
        .implementation_kinds
        .iter()
        .map(|kind| implementation_contract(binary, kind))
        .collect()
}

fn canonical_entry_signature(binary: &NustarBinary, kind: &str) -> String {
    match kind {
        "native-dylib" => format!(
            "{CANONICAL_ENTRY_SIGNATURE} // machine={} / {} / {} / {}",
            binary.machine_arch, binary.machine_os, binary.object_format, binary.calling_abi
        ),
        "llvm-bc" => format!(
            "{CANONICAL_ENTRY_SIGNATURE} // lowered under {} to {} / {} / {} / {}",
            binary.manifest.machine_abi_policy,
            binary.machine_arch,
            binary.machine_os,
            binary.object_format,
            binary.calling_abi
        ),
        _ => CANONICAL_ENTRY_SIGNATURE.to_owned(),
    }
}

fn implementation_contract(binary: &NustarBinary, kind: &str) -> ImplementationContract {
    let canonical_export = binary.manifest.loader_entry.clone();
    match kind {
        "native-dylib" => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: canonical_export.clone(),
            entry_signature: canonical_entry_signature(binary, kind),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: format!("native shared library ({})", binary.object_format),
            implementation_section: ".nustar.impl.native-dylib".to_owned(),
            required_exports: vec![
                canonical_export,
                "nustar.manifest.v1".to_owned(),
                "nustar.loader_abi.v1".to_owned(),
            ],
            required_metadata: vec![
                format!("machine_arch={}", binary.machine_arch),
                format!("machine_os={}", binary.machine_os),
                format!("object_format={}", binary.object_format),
                format!("calling_abi={}", binary.calling_abi),
            ]
            .into_iter()
            .chain(render_abi_target_contracts(&binary.manifest))
            .chain(render_runtime_symbol_contracts(&binary.manifest))
            .collect(),
            link_mode: "host-dynamic-load".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: append_runtime_symbol_notes(
                "expects a host-loadable shared library exporting the canonical loader entry with the canonical host/result structs",
                &binary.manifest,
            ),
        },
        "llvm-bc" => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: canonical_export.clone(),
            entry_signature: canonical_entry_signature(binary, kind),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: "llvm-bitcode-module".to_owned(),
            implementation_section: ".nustar.impl.llvm-bc".to_owned(),
            required_exports: vec![
                canonical_export,
                "nustar.manifest.v1".to_owned(),
                "nustar.loader_abi.v1".to_owned(),
            ],
            required_metadata: vec![
                "llvm_bitcode_version=opaque-pointer-compatible".to_owned(),
                format!("lowering_target_machine={}", binary.machine_arch),
                format!("lowering_object_format={}", binary.object_format),
                format!("lowering_calling_abi={}", binary.calling_abi),
            ]
            .into_iter()
            .chain(render_abi_target_contracts(&binary.manifest))
            .chain(render_runtime_symbol_contracts(&binary.manifest))
            .collect(),
            link_mode: "nuisc-link-or-lower".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: append_runtime_symbol_notes(
                "expects LLVM bitcode carrying the canonical loader entry symbol and the same bootstrap signature for later lowering/link integration",
                &binary.manifest,
            ),
        },
        "native-stub" => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: canonical_export,
            entry_signature: canonical_entry_signature(binary, kind),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: "opaque stub payload".to_owned(),
            implementation_section: ".nustar.impl.stub".to_owned(),
            required_exports: vec!["nustar.manifest.v1".to_owned()],
            required_metadata: vec!["prototype_only=true".to_owned()]
                .into_iter()
                .chain(render_abi_target_contracts(&binary.manifest))
                .chain(render_runtime_symbol_contracts(&binary.manifest))
                .collect(),
            link_mode: "non-loadable".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: append_runtime_symbol_notes(
                "prototype-only placeholder implementation; may be inspected and packaged but does not provide executable domain code",
                &binary.manifest,
            ),
        },
        other => ImplementationContract {
            kind: kind.to_owned(),
            loader_abi: binary.manifest.loader_abi.clone(),
            entry_symbol: binary.manifest.loader_entry.clone(),
            entry_signature: canonical_entry_signature(binary, other),
            host_abi_struct: CANONICAL_HOST_ABI_STRUCT.to_owned(),
            result_struct: CANONICAL_RESULT_STRUCT.to_owned(),
            status_convention: CANONICAL_LOADER_STATUS_CONVENTION.to_owned(),
            artifact_container: "custom-container".to_owned(),
            implementation_section: format!(".nustar.impl.{other}"),
            required_exports: vec![
                binary.manifest.loader_entry.clone(),
                "nustar.manifest.v1".to_owned(),
                "nustar.loader_abi.v1".to_owned(),
            ],
            required_metadata: vec!["custom_kind_requires_explicit_loader_adapter=true".to_owned()]
                .into_iter()
                .chain(render_abi_target_contracts(&binary.manifest))
                .chain(render_runtime_symbol_contracts(&binary.manifest))
                .collect(),
            link_mode: "custom".to_owned(),
            machine_abi_policy: binary.manifest.machine_abi_policy.clone(),
            notes: append_runtime_symbol_notes(
                &format!(
                    "custom implementation kind `{other}` must still satisfy the canonical loader ABI and entry contract"
                ),
                &binary.manifest,
            ),
        },
    }
}
