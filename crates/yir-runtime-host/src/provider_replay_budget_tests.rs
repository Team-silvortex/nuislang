use super::*;
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};
use yir_core::provider_runtime_ipc::{MAX_PAYLOAD_BYTES, MAX_REPLAY_MANIFEST_BYTES};

struct Files(std::path::PathBuf);
impl Files {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "nuis-replay-budget-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn manifest(&self, lengths: &[usize]) -> std::path::PathBuf {
        let mut source = format!(
            "schema = \"{PROVIDER_RESULT_STREAM_CONTRACT}\"\nsource_yir_fnv1a64 = \"0x0000000000000000\"\nframe_count = {}\nstream_hash = \"0x0000000000000000\"\n", lengths.len()
        );
        for (index, length) in lengths.iter().enumerate() {
            source.push_str(&format!(
                "[[frame]]\nindex = {index}\npayload_path = \"payload.bin\"\npayload_byte_length = {length}\n"
            ));
        }
        let path = self.0.join("stream.toml");
        fs::write(&path, source).unwrap();
        path
    }
}
impl Drop for Files {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn declared_replay_budget_is_checked_before_any_payload_is_opened() {
    let files = Files::new();
    for lengths in [
        vec![0],
        vec![MAX_PAYLOAD_BYTES + 1],
        vec![MAX_PAYLOAD_BYTES; 5],
    ] {
        let error = ProviderResultStream::load(&files.manifest(&lengths))
            .err()
            .unwrap();
        assert!(error.contains("budget"), "{error}");
        assert!(!error.contains("failed to open"), "{error}");
    }
}

#[test]
fn oversized_sparse_files_are_rejected_without_loading_their_contents() {
    let files = Files::new();
    let manifest = files.manifest(&[4]);
    let payload = fs::File::create(files.0.join("payload.bin")).unwrap();
    payload.set_len(MAX_PAYLOAD_BYTES as u64 + 1).unwrap();
    let error = ProviderResultStream::load(&manifest).err().unwrap();
    assert!(error.contains("byte budget"), "{error}");
    fs::File::create(&manifest)
        .unwrap()
        .set_len(MAX_REPLAY_MANIFEST_BYTES as u64 + 1)
        .unwrap();
    let error = ProviderResultStream::load(&manifest).err().unwrap();
    assert!(error.contains("byte budget"), "{error}");
}

#[test]
fn payload_reads_are_limited_and_short_files_still_fail_identity_checks() {
    let files = Files::new();
    assert!(replay_io::read_bounded(&files.0, 4).is_err());
    let path = files.0.join("bytes.bin");
    fs::write(&path, [1, 2, 3, 4]).unwrap();
    assert_eq!(replay_io::read_bounded(&path, 4).unwrap(), [1, 2, 3, 4]);
    assert!(replay_io::read_bounded(&path, 3).is_err());
    let manifest = files.manifest(&[4]);
    let mut source = fs::read_to_string(&manifest).unwrap();
    source.push_str("payload_hash = \"0x0000000000000000\"\n");
    fs::write(&manifest, source).unwrap();
    fs::write(files.0.join("payload.bin"), [1, 2, 3]).unwrap();
    let error = ProviderResultStream::load(&manifest).err().unwrap();
    assert!(error.contains("identity mismatch"), "{error}");
}
