use super::*;
use sha2::{Digest, Sha256};

pub(super) fn record(verb: &str, input: &Path, output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if verb != "run-artifact" || !stderr.contains("SIGKILL") {
        return;
    }
    eprintln!("native workflow abnormal launch at {}", input.display());
    let manifest = input.join("nuis.build.manifest.toml");
    let report = match nuisc::aot::verify_build_manifest(&manifest) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("native workflow manifest recheck: {error}");
            return;
        }
    };
    let binary = input.join(report.artifact_binary_name);
    match fs::read(&binary) {
        Ok(bytes) => eprintln!(
            "native workflow binary: {} bytes={} sha256={:x}",
            binary.display(),
            bytes.len(),
            Sha256::digest(&bytes)
        ),
        Err(error) => eprintln!("native workflow binary read: {error}"),
    }
    #[cfg(unix)]
    if let Ok(metadata) = fs::metadata(&binary) {
        use std::os::unix::fs::MetadataExt;
        eprintln!(
            "native workflow file identity: device={} inode={} mode={:o}",
            metadata.dev(),
            metadata.ino(),
            metadata.mode()
        );
    }
    #[cfg(target_os = "macos")]
    match Command::new("/usr/bin/codesign")
        .args(["--verify", "--strict", "--verbose=2"])
        .arg(&binary)
        .output()
    {
        Ok(check) => eprintln!(
            "native workflow signature: {}\n{}",
            check.status,
            String::from_utf8_lossy(&check.stderr)
        ),
        Err(error) => eprintln!("native workflow signature check unavailable: {error}"),
    }
}
