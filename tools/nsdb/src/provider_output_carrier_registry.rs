use crate::provider_carrier_channel_registry::PreparedProviderCarrierChannel;
#[cfg(unix)]
use crate::provider_carrier_channel_unix::MappedInheritedFdFrame;
#[cfg(unix)]
use crate::provider_output_carrier_unix::InheritedFdOutputCarrier;
use std::process::Command;

pub(crate) const PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT: &str =
    "nuis-provider-output-carrier-registry-v1";
pub(crate) const PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE: &str =
    "builtin-provider-output-carrier-registry";
pub(crate) const PROVIDER_OUTPUT_RESIDENCY_CONTRACT: &str = "nuis-provider-output-residency-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderOutputCarrierAdapter {
    pub(crate) registry_contract: &'static str,
    pub(crate) registry_source: &'static str,
    pub(crate) adapter_id: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) capability_status: &'static str,
    pub(crate) residency_kind: &'static str,
    pub(crate) transfer_scope: &'static str,
    pub(crate) observation_mode: &'static str,
    pub(crate) device_retention_status: &'static str,
    priority: u16,
    kind: &'static str,
}

const INHERITED_FD_OUTPUT: ProviderOutputCarrierAdapter = ProviderOutputCarrierAdapter {
    registry_contract: PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT,
    registry_source: PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE,
    adapter_id: "inherited.fd.output.v1",
    mode: "inherited-fd-output",
    capability_status: if cfg!(unix) {
        "registered-available"
    } else {
        "registered-unavailable"
    },
    residency_kind: "host-visible-file",
    transfer_scope: "cross-process-static",
    observation_mode: "mapped-on-demand",
    device_retention_status: "unsupported",
    priority: 10,
    kind: "inherited-fd-output",
};

const HEX_STDOUT_OUTPUT: ProviderOutputCarrierAdapter = ProviderOutputCarrierAdapter {
    registry_contract: PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT,
    registry_source: PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE,
    adapter_id: "hex.stdout.output.v1",
    mode: "hex-stdout-output",
    capability_status: "registered-available",
    residency_kind: "host-owned-bytes",
    transfer_scope: "observation-only",
    observation_mode: "stdout-eager",
    device_retention_status: "unsupported",
    priority: 100,
    kind: "hex-stdout-output",
};

const ADAPTERS: &[ProviderOutputCarrierAdapter] = &[INHERITED_FD_OUTPUT, HEX_STDOUT_OUTPUT];

pub(crate) enum PreparedProviderOutputCarrier {
    #[cfg(unix)]
    InheritedFd(InheritedFdOutputCarrier),
    HexStdout,
}

pub(crate) struct ProviderOutputCarrierConsumption {
    pub(crate) payload: Option<ProviderOutputPayload>,
    pub(crate) transferable: Option<PreparedProviderCarrierChannel>,
}

pub(crate) enum ProviderOutputPayload {
    #[cfg(unix)]
    InheritedFd(MappedInheritedFdFrame),
    Owned(Vec<u8>),
}

impl ProviderOutputPayload {
    pub(crate) fn owned(bytes: Vec<u8>) -> Self {
        Self::Owned(bytes)
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            #[cfg(unix)]
            Self::InheritedFd(frame) => frame.as_bytes(),
            Self::Owned(bytes) => bytes,
        }
    }
}

impl PreparedProviderOutputCarrier {
    pub(crate) fn configure_command(&self, command: &mut Command) -> Result<(), String> {
        #[cfg(unix)]
        if let Self::InheritedFd(carrier) = self {
            return carrier.configure_command(command);
        }
        Ok(())
    }

    pub(crate) fn consume(self, output: &str) -> Result<ProviderOutputCarrierConsumption, String> {
        match self {
            #[cfg(unix)]
            Self::InheritedFd(carrier) => {
                if output_field(output, "output_channel") != Some("inherited-fd") {
                    return Err("provider output did not use inherited fd".to_owned());
                }
                let hash = output_field(output, "output_hash")
                    .ok_or_else(|| "provider output omitted carrier hash".to_owned())?
                    .parse::<u64>()
                    .map_err(|error| format!("provider output hash is invalid: {error}"))?;
                let (frame, carrier) = carrier.consume(hash)?;
                Ok(ProviderOutputCarrierConsumption {
                    payload: Some(ProviderOutputPayload::InheritedFd(frame)),
                    transferable: Some(PreparedProviderCarrierChannel::InheritedFd(carrier)),
                })
            }
            Self::HexStdout => Ok(ProviderOutputCarrierConsumption {
                payload: None,
                transferable: None,
            }),
        }
    }
}

pub(crate) fn select_provider_output_carrier_adapter(
    requested_mode: &str,
) -> Option<ProviderOutputCarrierAdapter> {
    ADAPTERS
        .iter()
        .filter(|adapter| {
            adapter.capability_status == "registered-available"
                && (requested_mode == "auto" || requested_mode == adapter.mode)
        })
        .min_by_key(|adapter| (adapter.priority, adapter.adapter_id))
        .copied()
}

pub(crate) fn prepare_provider_output_carrier(
    adapter: ProviderOutputCarrierAdapter,
    byte_len: usize,
) -> Result<PreparedProviderOutputCarrier, String> {
    if adapter.registry_contract != PROVIDER_OUTPUT_CARRIER_REGISTRY_CONTRACT
        || adapter.registry_source != PROVIDER_OUTPUT_CARRIER_REGISTRY_SOURCE
    {
        return Err("provider output carrier registry identity mismatch".to_owned());
    }
    match adapter.kind {
        #[cfg(unix)]
        "inherited-fd-output" => {
            InheritedFdOutputCarrier::new(byte_len).map(PreparedProviderOutputCarrier::InheritedFd)
        }
        "hex-stdout-output" => Ok(PreparedProviderOutputCarrier::HexStdout),
        _ => Err(format!(
            "provider output carrier adapter `{}` is unavailable",
            adapter.adapter_id
        )),
    }
}

fn output_field<'a>(output: &'a str, name: &str) -> Option<&'a str> {
    output
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{name}=")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_selects_native_output_or_portable_hex_fallback() {
        let adapter = select_provider_output_carrier_adapter("auto").expect("adapter");
        assert_eq!(
            adapter.adapter_id,
            if cfg!(unix) {
                "inherited.fd.output.v1"
            } else {
                "hex.stdout.output.v1"
            }
        );
        assert!(select_provider_output_carrier_adapter("hex-stdout-output").is_some());
        assert_eq!(adapter.residency_kind, "host-visible-file");
        assert_eq!(adapter.observation_mode, "mapped-on-demand");
        assert_eq!(adapter.device_retention_status, "unsupported");
    }
}
