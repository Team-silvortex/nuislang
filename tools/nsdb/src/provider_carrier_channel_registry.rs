use crate::provider_carrier_channel::encode_provider_carrier_frames;
#[cfg(unix)]
use crate::provider_carrier_channel_unix::InheritedFdCarrier;
use std::{
    fs::File,
    io::Write,
    process::{Child, Command, Stdio},
};

pub(crate) const PROVIDER_CARRIER_CHANNEL_REGISTRY_CONTRACT: &str =
    "nuis-provider-carrier-channel-registry-v1";
pub(crate) const PROVIDER_CARRIER_CHANNEL_REGISTRY_SOURCE: &str =
    "builtin-provider-carrier-channel-registry";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderCarrierChannelAdapter {
    pub(crate) adapter_id: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) capability_status: &'static str,
    pub(crate) priority: u16,
    kind: &'static str,
}

const INHERITED_FD_ADAPTER: ProviderCarrierChannelAdapter = ProviderCarrierChannelAdapter {
    adapter_id: "inherited.fd.v1",
    mode: "inherited-fd",
    capability_status: if cfg!(unix) {
        "registered-available"
    } else {
        "registered-unavailable"
    },
    priority: 10,
    kind: "inherited-fd",
};

const FRAMED_STDIN_ADAPTER: ProviderCarrierChannelAdapter = ProviderCarrierChannelAdapter {
    adapter_id: "framed.stdin.v1",
    mode: "framed-stdin",
    capability_status: "registered-available",
    priority: 100,
    kind: "framed-stdin",
};

const REGISTERED_ADAPTERS: &[ProviderCarrierChannelAdapter] =
    &[INHERITED_FD_ADAPTER, FRAMED_STDIN_ADAPTER];

pub(crate) enum PreparedProviderCarrierChannel {
    #[cfg(unix)]
    InheritedFd(InheritedFdCarrier),
    FramedStdin(Vec<u8>),
}

impl PreparedProviderCarrierChannel {
    pub(crate) fn frame_argument(&self, frame_index: usize) -> String {
        match self {
            #[cfg(unix)]
            Self::InheritedFd(carrier) => carrier.frame_argument(frame_index),
            Self::FramedStdin(_) => format!("frame:{frame_index}"),
        }
    }

    #[cfg(unix)]
    pub(crate) fn worker_frame_argument(&self, frame_index: usize) -> Option<String> {
        match self {
            Self::InheritedFd(carrier) => Some(carrier.worker_frame_argument(frame_index)),
            Self::FramedStdin(_) => None,
        }
    }

    pub(crate) fn configure_command(&self, command: &mut Command) {
        match self {
            #[cfg(unix)]
            Self::InheritedFd(carrier) => {
                command.stdin(Stdio::null());
                carrier.configure_command(command);
            }
            Self::FramedStdin(_) => {
                command.stdin(Stdio::piped());
            }
        }
    }

    pub(crate) fn complete_spawn(&self, child: &mut Child) -> Result<(), String> {
        match self {
            #[cfg(unix)]
            Self::InheritedFd(_) => Ok(()),
            Self::FramedStdin(packet) => child
                .stdin
                .take()
                .ok_or_else(|| "provider carrier channel stdin is unavailable".to_owned())?
                .write_all(packet)
                .map_err(|error| format!("failed to write provider carrier packet: {error}")),
        }
    }

    pub(crate) fn try_clone_transferable(&self) -> Result<Option<Self>, String> {
        match self {
            #[cfg(unix)]
            Self::InheritedFd(carrier) => carrier.try_clone().map(Self::InheritedFd).map(Some),
            Self::FramedStdin(_) => Ok(None),
        }
    }

    #[cfg(unix)]
    pub(crate) fn try_clone_worker_descriptor(&self) -> Result<Option<File>, String> {
        match self {
            Self::InheritedFd(carrier) => carrier.try_clone_descriptor().map(Some),
            Self::FramedStdin(_) => Ok(None),
        }
    }
}

pub(crate) fn select_provider_carrier_channel_adapter(
    requested_mode: &str,
) -> Option<ProviderCarrierChannelAdapter> {
    REGISTERED_ADAPTERS
        .iter()
        .filter(|adapter| {
            adapter.capability_status == "registered-available"
                && (requested_mode == "auto" || adapter.mode == requested_mode)
        })
        .min_by_key(|adapter| (adapter.priority, adapter.adapter_id))
        .and_then(|adapter| provider_carrier_channel_capability(adapter.adapter_id))
}

pub(crate) fn provider_carrier_channel_capability(
    adapter_id: &str,
) -> Option<ProviderCarrierChannelAdapter> {
    REGISTERED_ADAPTERS
        .iter()
        .find(|adapter| adapter.adapter_id == adapter_id)
        .copied()
}

pub(crate) fn encode_provider_carrier_channel(
    adapter: ProviderCarrierChannelAdapter,
    frames: &[&[u8]],
) -> Result<Vec<u8>, String> {
    if adapter.kind != "framed-stdin" || adapter.capability_status != "registered-available" {
        return Err(format!(
            "provider carrier channel adapter `{}` cannot encode frames",
            adapter.adapter_id
        ));
    }
    encode_provider_carrier_frames(frames)
}

pub(crate) fn prepare_provider_carrier_channel(
    adapter: ProviderCarrierChannelAdapter,
    frames: &[&[u8]],
) -> Result<PreparedProviderCarrierChannel, String> {
    if adapter.capability_status != "registered-available" {
        return Err(format!(
            "provider carrier channel adapter `{}` is unavailable",
            adapter.adapter_id
        ));
    }
    match adapter.kind {
        #[cfg(unix)]
        "inherited-fd" => {
            InheritedFdCarrier::new(frames).map(PreparedProviderCarrierChannel::InheritedFd)
        }
        "framed-stdin" => encode_provider_carrier_channel(adapter, frames)
            .map(PreparedProviderCarrierChannel::FramedStdin),
        _ => Err(format!(
            "provider carrier channel adapter `{}` cannot prepare frames",
            adapter.adapter_id
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_selects_highest_priority_available_adapter() {
        let adapter = select_provider_carrier_channel_adapter("auto").expect("adapter");
        if cfg!(unix) {
            assert_eq!(adapter.adapter_id, "inherited.fd.v1");
            assert_eq!(adapter.mode, "inherited-fd");
        } else {
            assert_eq!(adapter.adapter_id, "framed.stdin.v1");
            assert_eq!(adapter.mode, "framed-stdin");
        }
    }

    #[test]
    fn inherited_fd_capability_tracks_host_support() {
        let adapter = provider_carrier_channel_capability("inherited.fd.v1").expect("registered");
        if cfg!(unix) {
            assert_eq!(adapter.capability_status, "registered-available");
            assert!(select_provider_carrier_channel_adapter("inherited-fd").is_some());
        } else {
            assert_eq!(adapter.capability_status, "registered-unavailable");
            assert!(select_provider_carrier_channel_adapter("inherited-fd").is_none());
        }
    }

    #[test]
    fn framed_stdin_remains_an_explicit_fallback() {
        let adapter = select_provider_carrier_channel_adapter("framed-stdin").expect("fallback");
        assert_eq!(adapter.adapter_id, "framed.stdin.v1");
        let channel = prepare_provider_carrier_channel(adapter, &[b"nuis"]).expect("channel");
        assert_eq!(channel.frame_argument(0), "frame:0");
    }
}
