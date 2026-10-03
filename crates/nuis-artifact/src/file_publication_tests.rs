use super::*;
use std::io::Read;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("nuis-publication-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn assert_no_staging_files(&self) {
        for entry in fs::read_dir(&self.0).unwrap() {
            assert!(!entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".nuis-publish-"));
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn publication_write_replaces_identity_without_mutating_retained_bytes() {
    let fixture = Fixture::new();
    let target = fixture.0.join("image");
    fs::write(&target, b"old").unwrap();
    let retained = fixture.0.join("retained");
    fs::hard_link(&target, &retained).unwrap();
    atomic_write_artifact_file(&target, b"new").unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new");
    assert_eq!(fs::read(retained).unwrap(), b"old");
    fixture.assert_no_staging_files();
}

#[test]
fn publication_copy_streams_bytes_and_preserves_source_permissions() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source");
    let target = fixture.0.join("target");
    let bytes = vec![0x5a; 256 * 1024];
    fs::write(&source, &bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&source, Permissions::from_mode(0o751)).unwrap();
    }
    atomic_copy_artifact_file(&source, &target).unwrap();
    assert_eq!(fs::read(&target).unwrap(), bytes);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o751
        );
    }
    fixture.assert_no_staging_files();
}

#[test]
fn publication_writer_failure_preserves_old_file_and_cleans_staging() {
    let fixture = Fixture::new();
    let target = fixture.0.join("image");
    fs::write(&target, b"old").unwrap();
    let error = publish(&target, None, |file| {
        file.write_all(b"partial new image")?;
        Err(io::Error::other("injected write failure"))
    })
    .unwrap_err();
    assert!(error.0.contains("injected write failure"));
    assert_eq!(fs::read(target).unwrap(), b"old");
    fixture.assert_no_staging_files();
}

#[test]
fn publication_missing_source_preserves_existing_destination() {
    let fixture = Fixture::new();
    let target = fixture.0.join("image");
    fs::write(&target, b"old").unwrap();
    assert!(atomic_copy_artifact_file(&fixture.0.join("missing"), &target).is_err());
    assert_eq!(fs::read(target).unwrap(), b"old");
    fixture.assert_no_staging_files();
}

#[test]
fn publication_refuses_directory_destinations_and_sources() {
    let fixture = Fixture::new();
    let directory = fixture.0.join("directory");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("marker"), b"keep").unwrap();
    assert!(atomic_write_artifact_file(&directory, b"replace").is_err());
    assert!(atomic_copy_artifact_file(&directory, &fixture.0.join("target")).is_err());
    assert_eq!(fs::read(directory.join("marker")).unwrap(), b"keep");
    fixture.assert_no_staging_files();
}

#[test]
fn publication_readers_observe_complete_old_or_complete_new_file() {
    let fixture = Fixture::new();
    let target = fixture.0.join("image");
    fs::write(&target, b"old").unwrap();
    let mut retained = File::open(&target).unwrap();
    publish(&target, None, |file| {
        file.write_all(b"new")?;
        assert_eq!(fs::read(&target).unwrap(), b"old");
        Ok(())
    })
    .unwrap();
    assert_eq!(fs::read(target).unwrap(), b"new");
    let mut previous = Vec::new();
    retained.read_to_end(&mut previous).unwrap();
    assert_eq!(previous, b"old");
    fixture.assert_no_staging_files();
}

#[cfg(unix)]
#[test]
fn publication_write_does_not_inherit_execute_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let target = fixture.0.join("image");
    fs::write(&target, b"old").unwrap();
    fs::set_permissions(&target, Permissions::from_mode(0o755)).unwrap();
    atomic_write_artifact_file(&target, b"unadmitted").unwrap();
    assert_eq!(
        fs::metadata(target).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[cfg(unix)]
#[test]
fn publication_refuses_symlinks_without_modifying_their_referent() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let outside = fixture.0.join("outside");
    let target = fixture.0.join("link");
    fs::write(&outside, b"keep").unwrap();
    symlink(&outside, &target).unwrap();
    assert!(atomic_write_artifact_file(&target, b"replace").is_err());
    assert!(atomic_copy_artifact_file(&outside, &target).is_err());
    assert_eq!(fs::read(outside).unwrap(), b"keep");
    assert!(fs::symlink_metadata(target)
        .unwrap()
        .file_type()
        .is_symlink());
    fixture.assert_no_staging_files();
}

#[cfg(target_os = "linux")]
#[test]
fn publication_supports_non_utf8_file_names() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let fixture = Fixture::new();
    let target = fixture.0.join(OsString::from_vec(vec![b'i', 0xff]));
    atomic_write_artifact_file(&target, b"first").unwrap();
    atomic_write_artifact_file(&target, b"second").unwrap();
    assert_eq!(fs::read(target).unwrap(), b"second");
    fixture.assert_no_staging_files();
}

#[test]
fn publication_supports_utf8_file_names() {
    let fixture = Fixture::new();
    let target = fixture.0.join("image-\u{56fe}\u{50cf}");
    atomic_write_artifact_file(&target, b"first").unwrap();
    atomic_write_artifact_file(&target, b"second").unwrap();
    assert_eq!(fs::read(target).unwrap(), b"second");
    fixture.assert_no_staging_files();
}

#[cfg(unix)]
#[test]
fn publication_does_not_extend_near_limit_destination_names() {
    let fixture = Fixture::new();
    let target = fixture.0.join("a".repeat(240));
    atomic_write_artifact_file(&target, b"complete").unwrap();
    assert_eq!(fs::read(target).unwrap(), b"complete");
    fixture.assert_no_staging_files();
}
