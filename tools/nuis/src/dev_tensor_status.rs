#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DevTensorStatusProtocolEntry {
    pub(crate) status: &'static str,
    pub(crate) rank: usize,
    pub(crate) phase: &'static str,
    pub(crate) terminal: bool,
    pub(crate) blocks_bootstrap: bool,
}

pub(crate) const DEV_TENSOR_STATUS_PROTOCOL_VERSION: &str = "dev-tensor-status-v1";

pub(crate) const DEV_TENSOR_STATUS_PROTOCOL: &[DevTensorStatusProtocolEntry] = &[
    DevTensorStatusProtocolEntry {
        status: "stable",
        rank: 4,
        phase: "validated",
        terminal: true,
        blocks_bootstrap: false,
    },
    DevTensorStatusProtocolEntry {
        status: "usable",
        rank: 3,
        phase: "usable",
        terminal: false,
        blocks_bootstrap: false,
    },
    DevTensorStatusProtocolEntry {
        status: "active",
        rank: 2,
        phase: "in-progress",
        terminal: false,
        blocks_bootstrap: false,
    },
    DevTensorStatusProtocolEntry {
        status: "early",
        rank: 1,
        phase: "exploratory",
        terminal: false,
        blocks_bootstrap: true,
    },
];

pub(crate) fn dev_tensor_status_rank(status: &str) -> usize {
    DEV_TENSOR_STATUS_PROTOCOL
        .iter()
        .find(|entry| entry.status == status)
        .map(|entry| entry.rank)
        .unwrap_or(0)
}
