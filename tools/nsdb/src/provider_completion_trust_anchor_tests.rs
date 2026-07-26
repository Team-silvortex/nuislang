use super::*;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn anchor_rejects_rollback_and_same_generation_fork() {
    let root = env::temp_dir().join(format!("nsdb-trust-anchor-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let anchor = root.join("anchor.toml");
    assert!(matches!(
        enforce_at_path(&anchor, "registry-v1", 2, &"a".repeat(64)),
        AnchorCheck::Accepted
    ));
    assert!(matches!(
        enforce_at_path(&anchor, "registry-v1", 1, &"b".repeat(64)),
        AnchorCheck::Rollback
    ));
    assert!(matches!(
        enforce_at_path(&anchor, "registry-v1", 2, &"b".repeat(64)),
        AnchorCheck::Fork
    ));
    assert!(matches!(
        enforce_at_path(&anchor, "registry-v1", 3, &"c".repeat(64)),
        AnchorCheck::Accepted
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn anchor_backend_unknown_fails_closed() {
    let _guard = TEST_ENV_LOCK.lock().unwrap();
    let root = env::temp_dir().join(format!(
        "nsdb-trust-anchor-invalid-backend-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let anchor = root.join("anchor.toml");
    env::set_var(ANCHOR_BACKEND_ENV, "unknown");
    env::set_var(ANCHOR_PATH_ENV, &anchor);
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    env::remove_var(ANCHOR_BACKEND_ENV);
    env::remove_var(ANCHOR_PATH_ENV);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stale_protocol_lock_is_recovered_without_deleting_successor_lock() {
    let root = env::temp_dir().join(format!("nsdb-stale-anchor-lock-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let lock = root.join("anchor.lock");
    fs::write(
            &lock,
            format!(
                "protocol = \"{LOCK_PROTOCOL}\"\nowner_pid = 1\ncreated_unix_ms = 0\nowner_token = \"stale\"\n"
            ),
        )
        .unwrap();
    let guard = AnchorLock::acquire(lock.clone()).unwrap();
    assert!(fs::read_to_string(&lock)
        .unwrap()
        .contains(&guard.owner_token));
    drop(guard);
    assert!(!lock.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn anchor_backend_contract_fails_closed() {
    assert!(is_supported_backend(FILE_BACKEND));
    assert!(is_supported_backend(PROTECTED_FILE_BACKEND));
    assert!(!is_supported_backend("keychain-v1"));
}

#[test]
fn protected_file_backend_requires_explicit_paths() {
    let _guard = TEST_ENV_LOCK.lock().unwrap();
    let root = env::temp_dir().join(format!("nsdb-protected-backend-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let anchor = root.join("anchors/trust.anchor");
    let marker = root.join("markers/trust.initialized");
    let anchor_parent = anchor.parent().unwrap();
    let marker_parent = marker.parent().unwrap();
    fs::create_dir_all(anchor.parent().unwrap()).unwrap();
    fs::create_dir_all(marker.parent().unwrap()).unwrap();
    env::set_var(ANCHOR_BACKEND_ENV, PROTECTED_FILE_BACKEND);
    env::set_var(ANCHOR_PATH_ENV, &anchor);
    env::set_var(ANCHOR_ROOT_ENV, anchor_parent);
    env::set_var(ANCHOR_MARKER_ROOT_ENV, marker_parent);
    let _ = env::remove_var(ANCHOR_MARKER_ENV);
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    env::set_var(ANCHOR_MARKER_ENV, &marker);
    env::remove_var(ANCHOR_ROOT_ENV);
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    env::set_var(ANCHOR_ROOT_ENV, anchor_parent);
    #[cfg(unix)]
    {
        fs::set_permissions(anchor_parent, PermissionsExt::from_mode(0o700)).unwrap();
        fs::set_permissions(marker_parent, PermissionsExt::from_mode(0o700)).unwrap();
    }
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Accepted
    ));
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Accepted
    ));
    env::set_var(ANCHOR_PATH_ENV, marker);
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    env::remove_var(ANCHOR_BACKEND_ENV);
    env::remove_var(ANCHOR_PATH_ENV);
    env::remove_var(ANCHOR_MARKER_ENV);
    env::remove_var(ANCHOR_ROOT_ENV);
    env::remove_var(ANCHOR_MARKER_ROOT_ENV);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn protected_file_backend_rejects_world_writable_parent_and_parent_symlink() {
    let _guard = TEST_ENV_LOCK.lock().unwrap();
    let root = env::temp_dir().join(format!(
        "nsdb-protected-backend-perm-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let anchor_parent = root.join("a");
    let marker_parent = root.join("m");
    fs::create_dir_all(&anchor_parent).unwrap();
    fs::create_dir_all(&marker_parent).unwrap();
    let anchor = anchor_parent.join("trust.anchor");
    let marker = marker_parent.join("trust.initialized");
    fs::set_permissions(&anchor_parent, PermissionsExt::from_mode(0o777)).unwrap();
    fs::set_permissions(&marker_parent, PermissionsExt::from_mode(0o777)).unwrap();
    env::set_var(ANCHOR_BACKEND_ENV, PROTECTED_FILE_BACKEND);
    env::set_var(ANCHOR_PATH_ENV, &anchor);
    env::set_var(ANCHOR_MARKER_ENV, &marker);
    env::set_var(ANCHOR_ROOT_ENV, &anchor_parent);
    env::set_var(ANCHOR_MARKER_ROOT_ENV, &marker_parent);
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    fs::set_permissions(&anchor_parent, PermissionsExt::from_mode(0o755)).unwrap();
    fs::set_permissions(&marker_parent, PermissionsExt::from_mode(0o755)).unwrap();
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    fs::set_permissions(&anchor_parent, PermissionsExt::from_mode(0o700)).unwrap();
    fs::set_permissions(&marker_parent, PermissionsExt::from_mode(0o700)).unwrap();
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Accepted
    ));
    let symlink_parent = root.join("symlink-marker");
    let marker_target = root.join("other");
    fs::create_dir_all(&marker_target).unwrap();
    std::os::unix::fs::symlink(&marker_target, &symlink_parent).unwrap();
    env::set_var(ANCHOR_MARKER_ENV, symlink_parent.join("trust.initialized"));
    assert!(matches!(
        enforce(
            &root.join("registry.toml"),
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    env::remove_var(ANCHOR_BACKEND_ENV);
    env::remove_var(ANCHOR_PATH_ENV);
    env::remove_var(ANCHOR_MARKER_ENV);
    env::remove_var(ANCHOR_ROOT_ENV);
    env::remove_var(ANCHOR_MARKER_ROOT_ENV);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn protected_backend_ignores_deleted_ordinary_file_anchor_pair() {
    let _guard = TEST_ENV_LOCK.lock().unwrap();
    let root = env::temp_dir().join(format!(
        "nsdb-protected-backend-ordinary-delete-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let anchor_root = root.join("protected-anchor");
    let marker_root = root.join("protected-marker");
    fs::create_dir_all(&anchor_root).unwrap();
    fs::create_dir_all(&marker_root).unwrap();
    fs::set_permissions(&anchor_root, PermissionsExt::from_mode(0o700)).unwrap();
    fs::set_permissions(&marker_root, PermissionsExt::from_mode(0o700)).unwrap();
    let anchor = anchor_root.join("trust.anchor");
    let marker = marker_root.join("trust.initialized");
    let registry = root.join("registry.toml");
    env::set_var(ANCHOR_BACKEND_ENV, PROTECTED_FILE_BACKEND);
    env::set_var(ANCHOR_PATH_ENV, &anchor);
    env::set_var(ANCHOR_MARKER_ENV, &marker);
    env::set_var(ANCHOR_ROOT_ENV, &anchor_root);
    env::set_var(ANCHOR_MARKER_ROOT_ENV, &marker_root);

    assert!(matches!(
        enforce(&registry, "registry-v1", 4, &"d".repeat(64)),
        AnchorCheck::Accepted
    ));
    assert_eq!(
        fs::metadata(&anchor).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(&marker).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let ordinary_anchor = PathBuf::from(format!("{}.anchor", registry.to_string_lossy()));
    let ordinary_marker = default_marker_path(&ordinary_anchor);
    fs::write(&ordinary_anchor, "ordinary").unwrap();
    fs::write(&ordinary_marker, "ordinary").unwrap();
    fs::remove_file(ordinary_anchor).unwrap();
    fs::remove_file(ordinary_marker).unwrap();
    assert!(matches!(
        enforce(&registry, "registry-v1", 3, &"c".repeat(64)),
        AnchorCheck::Rollback
    ));

    env::remove_var(ANCHOR_BACKEND_ENV);
    env::remove_var(ANCHOR_PATH_ENV);
    env::remove_var(ANCHOR_MARKER_ENV);
    env::remove_var(ANCHOR_ROOT_ENV);
    env::remove_var(ANCHOR_MARKER_ROOT_ENV);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_lock_only_recovers_after_filesystem_lease() {
    let root = env::temp_dir().join(format!("nsdb-malformed-lock-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let lock = root.join("anchor.lock");
    fs::write(&lock, "partial").unwrap();
    assert!(!lock_is_stale(&lock, now_unix_ms().unwrap()));
    assert!(lock_is_stale(&lock, u128::MAX));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn initialization_marker_detects_anchor_deletion() {
    let root = env::temp_dir().join(format!("nsdb-anchor-marker-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let anchor = root.join("anchor.toml");
    let marker = root.join("protected/anchor.initialized");
    fs::create_dir_all(marker.parent().unwrap()).unwrap();
    assert!(matches!(
        enforce_at_paths(
            &anchor,
            &marker,
            TrustAnchorBackend::File,
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Accepted
    ));
    assert!(marker.exists());
    fs::remove_file(&anchor).unwrap();
    assert!(matches!(
        enforce_at_paths(
            &anchor,
            &marker,
            TrustAnchorBackend::File,
            "registry-v1",
            2,
            &"a".repeat(64)
        ),
        AnchorCheck::Invalid
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn existing_anchor_migrates_to_marker_protocol() {
    let root = env::temp_dir().join(format!("nsdb-anchor-migration-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let anchor = root.join("anchor.toml");
    let marker = default_marker_path(&anchor);
    persist_anchor(
        &anchor,
        TrustAnchorBackend::File,
        "registry-v1",
        3,
        &"b".repeat(64),
    )
    .unwrap();
    assert!(matches!(
        enforce_at_paths(
            &anchor,
            &marker,
            TrustAnchorBackend::File,
            "registry-v1",
            3,
            &"c".repeat(64)
        ),
        AnchorCheck::Fork
    ));
    assert!(!marker.exists());
    assert!(matches!(
        enforce_at_paths(
            &anchor,
            &marker,
            TrustAnchorBackend::File,
            "registry-v1",
            3,
            &"b".repeat(64)
        ),
        AnchorCheck::Accepted
    ));
    assert_eq!(
        read_marker(&marker)
            .unwrap()
            .unwrap()
            .initialized_generation,
        3
    );
    fs::remove_dir_all(root).unwrap();
}
