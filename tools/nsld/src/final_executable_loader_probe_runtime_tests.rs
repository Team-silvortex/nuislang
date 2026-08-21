use super::*;
use std::{fs, time::SystemTime};

#[test]
fn cleanup_preserves_paths_not_created_by_the_probe() {
    let root = unique_temp_dir("nsld-loader-probe-unowned");
    fs::create_dir_all(&root).unwrap();
    let paths = unique_probe_paths(&root, "shared");
    fs::write(&paths.executable, b"preexisting").unwrap();

    assert!(cleanup_probe_paths(&paths, &ProbeCreatedPaths::default()));
    assert_eq!(fs::read(&paths.executable).unwrap(), b"preexisting");

    fs::remove_file(paths.executable).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn capture_limit_detects_oversized_probe_output() {
    let root = unique_temp_dir("nsld-loader-probe-capture-limit");
    fs::create_dir_all(&root).unwrap();
    let paths = unique_probe_paths(&root, "shared");
    File::create(&paths.stdout)
        .unwrap()
        .set_len(CAPTURE_LIMIT_BYTES + 1)
        .unwrap();
    File::create(&paths.stderr).unwrap();

    assert!(capture_limit_exceeded(&paths));

    fs::remove_file(paths.stdout).unwrap();
    fs::remove_file(paths.stderr).unwrap();
    fs::remove_dir(root).unwrap();
}

#[test]
fn path_namespace_cannot_escape_probe_root() {
    let root = unique_temp_dir("nsld-loader-probe-namespace");
    fs::create_dir_all(&root).unwrap();

    let error = execute_isolated_loader_probe(LoaderProbeRuntimeRequest {
        bytes: b"not-an-executable",
        probe_root: &root,
        path_namespace: "../escape",
    })
    .unwrap_err();

    assert!(error.contains("path namespace"));
    assert!(fs::read_dir(&root).unwrap().next().is_none());
    fs::remove_dir(root).unwrap();
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}
