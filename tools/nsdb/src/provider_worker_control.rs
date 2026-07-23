use crate::provider_sample_artifact::fnv1a64_hex;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) const PROVIDER_WORKER_ADAPTER_CONTROL_CARRIER_CONTRACT: &str =
    "nuis-provider-worker-adapter-control-carrier-v1";
pub(crate) const PROVIDER_WORKER_ADAPTER_CONTROL_ROLE: &str = "control.adapter";
const MAX_CONTROL_CARRIER_BYTES: usize = 60 * 1024;
static NEXT_CONTROL_CARRIER_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) struct ProviderWorkerControlCarrier {
    file: File,
    pub(crate) byte_length: usize,
    pub(crate) payload_hash: String,
}

impl ProviderWorkerControlCarrier {
    pub(crate) fn new(payload: &[u8]) -> Result<Self, String> {
        if payload.is_empty() || payload.len() > MAX_CONTROL_CARRIER_BYTES {
            return Err("provider worker control carrier payload is out of bounds".to_owned());
        }
        let (mut file, path) = create_control_file()?;
        file.write_all(payload)
            .map_err(|error| format!("failed to write provider control carrier: {error}"))?;
        fs::remove_file(&path).map_err(|error| {
            format!(
                "failed to unlink provider control carrier `{}`: {error}",
                path.display()
            )
        })?;
        Ok(Self {
            file,
            byte_length: payload.len(),
            payload_hash: fnv1a64_hex(payload),
        })
    }

    pub(crate) fn file(&self) -> &File {
        &self.file
    }
}

fn create_control_file() -> Result<(File, PathBuf), String> {
    for _ in 0..32 {
        let id = NEXT_CONTROL_CARRIER_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nuis-provider-control-{}-{id}.bin",
            std::process::id()
        ));
        match OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok((file, path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create provider control carrier `{}`: {error}",
                    path.display()
                ));
            }
        }
    }
    Err("failed to allocate a unique provider control carrier".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::FileExt;

    #[test]
    fn control_carrier_is_unlinked_and_hash_bound() {
        let carrier = ProviderWorkerControlCarrier::new(b"adapter-control").expect("carrier");
        let mut bytes = [0u8; 15];
        carrier.file.read_exact_at(&mut bytes, 0).expect("read");
        assert_eq!(&bytes, b"adapter-control");
        assert_eq!(carrier.byte_length, bytes.len());
        assert_eq!(carrier.payload_hash, fnv1a64_hex(&bytes));
    }
}
