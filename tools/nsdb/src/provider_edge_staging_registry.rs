use std::{fs, time::SystemTime};

use crate::provider_carrier_input::ProviderCarrierInput;

pub(crate) const PROVIDER_EDGE_STAGING_REGISTRY_CONTRACT: &str =
    "nuis-provider-edge-staging-registry-v1";
pub(crate) const PROVIDER_EDGE_STAGING_REGISTRY_SOURCE: &str =
    "builtin-provider-edge-staging-registry";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderEdgeStagingAdapter {
    pub(crate) adapter_id: &'static str,
    pub(crate) staging_mode: &'static str,
    pub(crate) capability_status: &'static str,
    pub(crate) priority: u16,
    kind: &'static str,
}

#[derive(Debug)]
pub(crate) struct ProviderEdgeStagingCarrier {
    pub(crate) identity: String,
    pub(crate) input: ProviderCarrierInput,
}

const MEMORY_ADAPTER: ProviderEdgeStagingAdapter = ProviderEdgeStagingAdapter {
    adapter_id: "memory.owned-bytes.v1",
    staging_mode: "memory-backed",
    capability_status: "registered-available",
    priority: 10,
    kind: "memory-owned-bytes",
};

const HOST_FILE_ADAPTER: ProviderEdgeStagingAdapter = ProviderEdgeStagingAdapter {
    adapter_id: "host.visible.owned-file.v1",
    staging_mode: "host-visible-owned-file",
    capability_status: "registered-available",
    priority: 100,
    kind: "host-owned-file",
};

const REGISTERED_ADAPTERS: &[ProviderEdgeStagingAdapter] = &[MEMORY_ADAPTER, HOST_FILE_ADAPTER];

pub(crate) fn select_provider_edge_staging_adapter(
    requested_mode: &str,
) -> Option<ProviderEdgeStagingAdapter> {
    REGISTERED_ADAPTERS
        .iter()
        .filter(|adapter| {
            adapter.capability_status == "registered-available"
                && (requested_mode == "auto" || adapter.staging_mode == requested_mode)
        })
        .min_by_key(|adapter| (adapter.priority, adapter.adapter_id))
        .copied()
}

pub(crate) fn materialize_provider_edge_carrier(
    adapter: ProviderEdgeStagingAdapter,
    owner_hash: &str,
    bytes: &[u8],
) -> Result<ProviderEdgeStagingCarrier, String> {
    if adapter.kind == "memory-owned-bytes" {
        return Ok(ProviderEdgeStagingCarrier {
            identity: format!("memory:{owner_hash}"),
            input: ProviderCarrierInput::OpaqueBytes {
                handle: format!("memory:{owner_hash}"),
                bytes: bytes.to_vec(),
            },
        });
    }
    if adapter.kind != "host-owned-file" {
        return Err(format!(
            "provider edge staging adapter `{}` has no materializer",
            adapter.adapter_id
        ));
    }
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nuis-provider-edge-input-{owner_hash}-{}-{nonce}.bin",
        std::process::id()
    ));
    fs::write(&path, bytes)
        .map_err(|error| format!("failed to materialize provider edge carrier: {error}"))?;
    Ok(ProviderEdgeStagingCarrier {
        identity: format!("owned-file:{owner_hash}"),
        input: ProviderCarrierInput::Path(path),
    })
}

pub(crate) fn consume_provider_edge_carrier(
    adapter: ProviderEdgeStagingAdapter,
    carrier: &ProviderEdgeStagingCarrier,
) -> Result<Vec<u8>, String> {
    if adapter.kind == "memory-owned-bytes" {
        return carrier
            .input
            .bytes()
            .map(ToOwned::to_owned)
            .ok_or_else(|| "memory staging adapter received a non-memory carrier".to_owned());
    }
    if adapter.kind != "host-owned-file" {
        return Err(format!(
            "provider edge staging adapter `{}` has no consumer",
            adapter.adapter_id
        ));
    }
    fs::read(
        carrier
            .input
            .path()
            .ok_or_else(|| "host file staging adapter received a non-path carrier".to_owned())?,
    )
    .map_err(|error| format!("failed to consume provider edge carrier: {error}"))
}

pub(crate) fn release_provider_edge_carrier(
    adapter: ProviderEdgeStagingAdapter,
    carrier: &mut ProviderEdgeStagingCarrier,
) -> Result<(), String> {
    if adapter.kind == "memory-owned-bytes" {
        carrier.input.clear();
        return Ok(());
    }
    if adapter.kind != "host-owned-file" {
        return Err(format!(
            "provider edge staging adapter `{}` has no releaser",
            adapter.adapter_id
        ));
    }
    fs::remove_file(
        carrier
            .input
            .path()
            .ok_or_else(|| "host file staging adapter received a non-path carrier".to_owned())?,
    )
    .map_err(|error| format!("failed to release provider edge carrier: {error}"))
}

pub(crate) fn cleanup_provider_edge_carrier(carrier: &mut ProviderEdgeStagingCarrier) {
    if let Some(path) = carrier.input.path() {
        let _ = fs::remove_file(path);
    }
    carrier.input.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_selection_prefers_memory_backed_adapter() {
        let selected = select_provider_edge_staging_adapter("auto").expect("fallback adapter");
        assert_eq!(selected.adapter_id, "memory.owned-bytes.v1");
        assert_eq!(selected.staging_mode, "memory-backed");
        assert_eq!(selected.priority, 10);
    }

    #[test]
    fn unknown_explicit_mode_fails_closed() {
        assert!(select_provider_edge_staging_adapter("device-direct").is_none());
    }

    #[test]
    fn host_file_adapter_owns_complete_carrier_lifecycle() {
        let adapter = select_provider_edge_staging_adapter("host-visible-owned-file")
            .expect("host file adapter");
        let mut carrier = materialize_provider_edge_carrier(adapter, "test-owner", b"nuis")
            .expect("materialized carrier");
        assert_eq!(carrier.identity, "owned-file:test-owner");
        assert_eq!(
            consume_provider_edge_carrier(adapter, &carrier).expect("consumed carrier"),
            b"nuis"
        );
        let path = carrier.input.path().expect("path carrier").to_owned();
        release_provider_edge_carrier(adapter, &mut carrier).expect("released carrier");
        assert!(!path.exists());
    }

    #[test]
    fn memory_adapter_owns_opaque_carrier_lifecycle() {
        let adapter = select_provider_edge_staging_adapter("auto").expect("memory adapter");
        let mut carrier = materialize_provider_edge_carrier(adapter, "test-owner", b"nuis")
            .expect("materialized carrier");
        assert_eq!(carrier.input.kind(), "opaque-bytes");
        assert_eq!(carrier.input.handle(), Some("memory:test-owner"));
        assert_eq!(
            consume_provider_edge_carrier(adapter, &carrier).unwrap(),
            b"nuis"
        );
        release_provider_edge_carrier(adapter, &mut carrier).unwrap();
        assert_eq!(carrier.input.bytes(), Some([].as_slice()));
    }
}
