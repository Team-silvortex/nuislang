use std::path::{Path, PathBuf};

pub(crate) const PROVIDER_CARRIER_INPUT_CONTRACT: &str = "nuis-provider-carrier-input-v1";

#[derive(Debug)]
pub(crate) enum ProviderCarrierInput {
    Path(PathBuf),
    OpaqueBytes { handle: String, bytes: Vec<u8> },
}

impl ProviderCarrierInput {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Path(_) => "path",
            Self::OpaqueBytes { .. } => "opaque-bytes",
        }
    }

    pub(crate) fn path(&self) -> Option<&Path> {
        match self {
            Self::Path(path) => Some(path),
            Self::OpaqueBytes { .. } => None,
        }
    }

    pub(crate) fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Path(_) => None,
            Self::OpaqueBytes { bytes, .. } => Some(bytes),
        }
    }

    pub(crate) fn handle(&self) -> Option<&str> {
        match self {
            Self::Path(_) => None,
            Self::OpaqueBytes { handle, .. } => Some(handle),
        }
    }

    pub(crate) fn clear(&mut self) {
        if let Self::OpaqueBytes { bytes, .. } = self {
            bytes.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_carrier_exposes_handle_and_bytes_without_path() {
        let input = ProviderCarrierInput::OpaqueBytes {
            handle: "memory:test".to_owned(),
            bytes: b"nuis".to_vec(),
        };
        assert_eq!(input.kind(), "opaque-bytes");
        assert_eq!(input.handle(), Some("memory:test"));
        assert_eq!(input.bytes(), Some(b"nuis".as_slice()));
        assert!(input.path().is_none());
    }
}
