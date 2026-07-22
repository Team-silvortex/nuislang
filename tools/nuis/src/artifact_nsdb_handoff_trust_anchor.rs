#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const ANCHOR_PROTOCOL: &str = "nuis-provider-completion-trust-anchor-v1";
const ANCHOR_PATH_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR";
const ANCHOR_BACKEND_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_BACKEND";
const ANCHOR_MARKER_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_MARKER";
const ANCHOR_ROOT_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_ROOT";
const ANCHOR_MARKER_ROOT_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR_MARKER_ROOT";
const FILE_BACKEND: &str = "file-v1";
const PROTECTED_FILE_BACKEND: &str = "protected-file-v1";
const MARKER_PROTOCOL: &str = "nuis-provider-completion-trust-anchor-marker-v1";
const LOCK_PROTOCOL: &str = "nuis-provider-completion-trust-anchor-lock-v1";
const LOCK_STALE_AFTER_MS: u128 = 30_000;
static LOCK_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Clone, Copy)]
enum TrustAnchorBackend {
    File,
    ProtectedFile,
}

pub(crate) enum AnchorCheck {
    Accepted,
    Rollback,
    Fork,
    Invalid,
}

pub(crate) fn enforce(
    registry_path: &Path,
    registry_protocol: &str,
    generation: usize,
    registry_hash: &str,
) -> AnchorCheck {
    let backend = match env::var_os(ANCHOR_BACKEND_ENV) {
        Some(raw) => match parse_anchor_backend(&raw) {
            Some(value) => value,
            None => return AnchorCheck::Invalid,
        },
        None => TrustAnchorBackend::File,
    };
    let Some((path, marker_path)) = resolve_anchor_paths(registry_path, &backend) else {
        return AnchorCheck::Invalid;
    };
    enforce_at_paths(
        &path,
        &marker_path,
        backend,
        registry_protocol,
        generation,
        registry_hash,
    )
}

fn parse_anchor_backend(raw: &std::ffi::OsString) -> Option<TrustAnchorBackend> {
    let backend = raw.to_str()?;
    (backend == FILE_BACKEND)
        .then_some(TrustAnchorBackend::File)
        .or_else(|| {
            (backend == PROTECTED_FILE_BACKEND).then_some(TrustAnchorBackend::ProtectedFile)
        })
}

fn resolve_anchor_paths(
    registry_path: &Path,
    backend: &TrustAnchorBackend,
) -> Option<(PathBuf, PathBuf)> {
    let anchor_path = match backend {
        TrustAnchorBackend::File => env::var_os(ANCHOR_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(format!("{}.anchor", registry_path.to_string_lossy()))
            }),
        TrustAnchorBackend::ProtectedFile => env::var_os(ANCHOR_PATH_ENV).map(PathBuf::from)?,
    };
    if anchor_path.as_os_str().is_empty() {
        return None;
    }
    let marker_path = match backend {
        TrustAnchorBackend::File => env::var_os(ANCHOR_MARKER_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| default_marker_path(&anchor_path)),
        TrustAnchorBackend::ProtectedFile => env::var_os(ANCHOR_MARKER_ENV).map(PathBuf::from)?,
    };
    if marker_path.as_os_str().is_empty() || anchor_path == marker_path {
        return None;
    }
    if !is_anchor_path_valid(&anchor_path, *backend) {
        return None;
    }
    if !is_anchor_path_valid(&marker_path, *backend) {
        return None;
    }
    if let TrustAnchorBackend::ProtectedFile = backend {
        let anchor_root = env::var_os(ANCHOR_ROOT_ENV).map(PathBuf::from)?;
        let marker_root = env::var_os(ANCHOR_MARKER_ROOT_ENV).map(PathBuf::from)?;
        if !protected_path_is_rooted(&anchor_path, &anchor_root)
            || !protected_path_is_rooted(&marker_path, &marker_root)
            || anchor_root == marker_root
        {
            return None;
        }
    }
    Some((anchor_path, marker_path))
}

fn is_anchor_path_valid(path: &Path, backend: TrustAnchorBackend) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    if is_symlink_path(path) {
        return false;
    }
    if !is_existing_directory(parent) {
        return false;
    }
    if let TrustAnchorBackend::ProtectedFile = backend {
        return is_protected_directory(parent);
    }
    true
}

fn is_existing_directory(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|metadata| metadata.is_dir())
}

fn is_protected_directory(path: &Path) -> bool {
    #[cfg(unix)]
    if !fs::metadata(path).is_ok_and(|metadata| {
        metadata.is_dir() && (metadata.permissions().mode() & 0o077) == 0 && !is_symlink_path(path)
    }) {
        return false;
    }
    true
}

#[cfg(not(unix))]
fn is_protected_directory(path: &Path) -> bool {
    let _ = path;
    false
}

fn protected_path_is_rooted(path: &Path, root: &Path) -> bool {
    path.is_absolute()
        && root.is_absolute()
        && path.parent() == Some(root)
        && is_protected_directory(root)
        && protected_file_is_secure_or_missing(path)
}

fn protected_file_is_secure_or_missing(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        #[cfg(unix)]
        Ok(metadata) => {
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && (metadata.permissions().mode() & 0o077) == 0
        }
        #[cfg(not(unix))]
        Ok(_) => false,
        Err(_) => false,
    }
}

fn is_symlink_path(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

#[allow(dead_code)]
fn is_supported_backend(backend: &str) -> bool {
    backend == FILE_BACKEND || backend == PROTECTED_FILE_BACKEND
}

#[cfg(test)]
fn enforce_at_path(
    path: &Path,
    registry_protocol: &str,
    generation: usize,
    registry_hash: &str,
) -> AnchorCheck {
    enforce_at_paths(
        path,
        &default_marker_path(path),
        TrustAnchorBackend::File,
        registry_protocol,
        generation,
        registry_hash,
    )
}

fn enforce_at_paths(
    path: &Path,
    marker_path: &Path,
    backend: TrustAnchorBackend,
    registry_protocol: &str,
    generation: usize,
    registry_hash: &str,
) -> AnchorCheck {
    let lock_path = PathBuf::from(format!("{}.lock", path.to_string_lossy()));
    let Ok(_guard) = AnchorLock::acquire(lock_path) else {
        return AnchorCheck::Invalid;
    };
    let marker = match read_marker(marker_path) {
        Ok(marker) => marker,
        Err(()) => return AnchorCheck::Invalid,
    };
    let anchor = match read_anchor(path) {
        Ok(anchor) => anchor,
        Err(()) => return AnchorCheck::Invalid,
    };
    let (anchor, marker_missing) = match (anchor, marker) {
        (None, Some(_)) => return AnchorCheck::Invalid,
        (None, None) => {
            let anchor = TrustAnchor {
                registry_protocol: registry_protocol.to_owned(),
                highest_generation: generation,
                registry_hash: registry_hash.to_owned(),
            };
            if persist_anchor(path, backend, registry_protocol, generation, registry_hash).is_err()
                || persist_marker(marker_path, backend, &anchor).is_err()
            {
                return AnchorCheck::Invalid;
            }
            return AnchorCheck::Accepted;
        }
        (Some(anchor), None) => (anchor, true),
        (Some(anchor), Some(marker)) => {
            if !marker.matches(&anchor) {
                return AnchorCheck::Invalid;
            }
            (anchor, false)
        }
    };
    if anchor.registry_protocol != registry_protocol {
        AnchorCheck::Invalid
    } else if generation < anchor.highest_generation {
        AnchorCheck::Rollback
    } else if generation == anchor.highest_generation {
        if registry_hash == anchor.registry_hash {
            if marker_missing && persist_marker(marker_path, backend, &anchor).is_err() {
                AnchorCheck::Invalid
            } else {
                AnchorCheck::Accepted
            }
        } else {
            AnchorCheck::Fork
        }
    } else {
        if marker_missing && persist_marker(marker_path, backend, &anchor).is_err() {
            AnchorCheck::Invalid
        } else {
            persist_anchor(path, backend, registry_protocol, generation, registry_hash)
                .map_or(AnchorCheck::Invalid, |_| AnchorCheck::Accepted)
        }
    }
}

fn default_marker_path(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.initialized", path.to_string_lossy()))
}

struct TrustAnchor {
    registry_protocol: String,
    highest_generation: usize,
    registry_hash: String,
}

struct TrustAnchorMarker {
    registry_protocol: String,
    initialized_generation: usize,
    initialized_registry_hash: String,
}

impl TrustAnchorMarker {
    fn matches(&self, anchor: &TrustAnchor) -> bool {
        self.registry_protocol == anchor.registry_protocol
            && self.initialized_generation <= anchor.highest_generation
            && (self.initialized_generation < anchor.highest_generation
                || self.initialized_registry_hash == anchor.registry_hash)
    }
}

fn read_anchor(path: &Path) -> Result<Option<TrustAnchor>, ()> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
    };
    if string_field(&source, "protocol").as_deref() != Some(ANCHOR_PROTOCOL) {
        return Err(());
    }
    Ok(Some(TrustAnchor {
        registry_protocol: string_field(&source, "registry_protocol").ok_or(())?,
        highest_generation: usize_field(&source, "highest_generation")
            .filter(|v| *v > 0)
            .ok_or(())?,
        registry_hash: string_field(&source, "registry_hash")
            .filter(|v| v.len() == 64 && v.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .ok_or(())?,
    }))
}

fn read_marker(path: &Path) -> Result<Option<TrustAnchorMarker>, ()> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
    };
    if string_field(&source, "protocol").as_deref() != Some(MARKER_PROTOCOL)
        || string_field(&source, "anchor_protocol").as_deref() != Some(ANCHOR_PROTOCOL)
    {
        return Err(());
    }
    Ok(Some(TrustAnchorMarker {
        registry_protocol: string_field(&source, "registry_protocol").ok_or(())?,
        initialized_generation: usize_field(&source, "initialized_generation")
            .filter(|value| *value > 0)
            .ok_or(())?,
        initialized_registry_hash: string_field(&source, "initialized_registry_hash")
            .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .ok_or(())?,
    }))
}

fn persist_anchor(
    path: &Path,
    backend: TrustAnchorBackend,
    registry_protocol: &str,
    generation: usize,
    hash: &str,
) -> Result<(), ()> {
    let temporary = PathBuf::from(format!("{}.tmp", path.to_string_lossy()));
    let content = format!(
        "protocol = \"{ANCHOR_PROTOCOL}\"\nregistry_protocol = \"{registry_protocol}\"\nhighest_generation = {generation}\nregistry_hash = \"{hash}\"\n"
    );
    persist_atomic(path, &temporary, content.as_bytes(), backend)
}

fn persist_marker(
    path: &Path,
    backend: TrustAnchorBackend,
    anchor: &TrustAnchor,
) -> Result<(), ()> {
    let temporary = PathBuf::from(format!("{}.tmp", path.to_string_lossy()));
    let content = format!(
        "protocol = \"{MARKER_PROTOCOL}\"\nanchor_protocol = \"{ANCHOR_PROTOCOL}\"\nregistry_protocol = \"{}\"\ninitialized_generation = {}\ninitialized_registry_hash = \"{}\"\n",
        anchor.registry_protocol, anchor.highest_generation, anchor.registry_hash
    );
    persist_atomic(path, &temporary, content.as_bytes(), backend)
}

fn persist_atomic(
    path: &Path,
    temporary: &Path,
    content: &[u8],
    backend: TrustAnchorBackend,
) -> Result<(), ()> {
    let mut options = OpenOptions::new();
    if matches!(backend, TrustAnchorBackend::ProtectedFile) {
        options.create_new(true).write(true);
    } else {
        options.create(true).truncate(true).write(true);
    }
    #[cfg(unix)]
    if matches!(backend, TrustAnchorBackend::ProtectedFile) {
        options.mode(0o600);
    }
    let mut file = options.open(temporary).map_err(|_| ())?;
    file.write_all(content).map_err(|_| ())?;
    file.sync_all().map_err(|_| ())?;
    fs::rename(temporary, path).map_err(|_| ())?;
    if matches!(backend, TrustAnchorBackend::ProtectedFile)
        && !protected_file_is_secure_or_missing(path)
    {
        return Err(());
    }
    if let Some(parent) = path.parent() {
        OpenOptions::new()
            .read(true)
            .open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| ())?;
    }
    Ok(())
}

struct AnchorLock {
    path: PathBuf,
    owner_token: String,
}

impl AnchorLock {
    fn acquire(path: PathBuf) -> Result<Self, ()> {
        for _ in 0..200 {
            let created_unix_ms = now_unix_ms()?;
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            match options.open(&path) {
                Ok(mut file) => {
                    let owner_token = format!(
                        "{}-{created_unix_ms}-{}",
                        std::process::id(),
                        LOCK_SEQUENCE.fetch_add(1, Ordering::Relaxed)
                    );
                    let result = writeln!(
                        file,
                        "protocol = \"{LOCK_PROTOCOL}\"\nowner_pid = {}\ncreated_unix_ms = {created_unix_ms}\nowner_token = \"{owner_token}\"",
                        std::process::id()
                    )
                    .and_then(|_| file.sync_all());
                    if result.is_err() {
                        let _ = fs::remove_file(&path);
                        return Err(());
                    }
                    return Ok(Self { path, owner_token });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if lock_is_stale(&path, now_unix_ms()?) {
                        match fs::remove_file(&path) {
                            Ok(()) => continue,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                            Err(_) => return Err(()),
                        }
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(_) => return Err(()),
            }
        }
        Err(())
    }
}

impl Drop for AnchorLock {
    fn drop(&mut self) {
        let still_owned = fs::read_to_string(&self.path)
            .ok()
            .and_then(|source| string_field(&source, "owner_token"))
            .is_some_and(|owner| owner == self.owner_token);
        if still_owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn now_unix_ms() -> Result<u128, ()> {
    system_time_unix_ms(SystemTime::now()).ok_or(())
}

fn system_time_unix_ms(time: SystemTime) -> Option<u128> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis())
}

fn lock_is_stale(path: &Path, now_unix_ms: u128) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    let valid_protocol = string_field(&source, "protocol").as_deref() == Some(LOCK_PROTOCOL)
        && usize_field(&source, "owner_pid").is_some_and(|pid| pid > 0)
        && string_field(&source, "owner_token").is_some_and(|token| !token.is_empty());
    let created_unix_ms = valid_protocol
        .then(|| u128_field(&source, "created_unix_ms"))
        .flatten()
        .or_else(|| {
            fs::metadata(path)
                .ok()?
                .modified()
                .ok()
                .and_then(system_time_unix_ms)
        });
    let Some(created_unix_ms) = created_unix_ms else {
        return false;
    };
    now_unix_ms.saturating_sub(created_unix_ms) > LOCK_STALE_AFTER_MS
}

fn string_field(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key).then(|| {
            value
                .trim()
                .strip_prefix('"')?
                .strip_suffix('"')
                .map(str::to_owned)
        })?
    })
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

fn u128_field(source: &str, key: &str) -> Option<u128> {
    source.lines().find_map(|line| {
        let (candidate, value) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| value.trim().parse().ok())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn independent_anchor_rejects_rollback_and_fork() {
        let root = env::temp_dir().join(format!("nuis-trust-anchor-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let anchor = root.join("anchor.toml");
        assert!(matches!(
            enforce_at_path(&anchor, "registry-v1", 4, &"d".repeat(64)),
            AnchorCheck::Accepted
        ));
        assert!(matches!(
            enforce_at_path(&anchor, "registry-v1", 3, &"e".repeat(64)),
            AnchorCheck::Rollback
        ));
        assert!(matches!(
            enforce_at_path(&anchor, "registry-v1", 4, &"e".repeat(64)),
            AnchorCheck::Fork
        ));
        assert!(matches!(
            enforce_at_path(&anchor, "registry-v1", 5, &"f".repeat(64)),
            AnchorCheck::Accepted
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn independent_anchor_unknown_backend_fails_closed() {
        let _guard = TEST_ENV_LOCK.lock().unwrap();
        let root = env::temp_dir().join(format!(
            "nuis-trust-anchor-invalid-backend-{}",
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
                &"d".repeat(64)
            ),
            AnchorCheck::Invalid
        ));
        env::remove_var(ANCHOR_BACKEND_ENV);
        env::remove_var(ANCHOR_PATH_ENV);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn independently_recovers_stale_protocol_lock() {
        let root = env::temp_dir().join(format!("nuis-stale-anchor-lock-{}", std::process::id()));
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
    fn independent_anchor_backend_contract_fails_closed() {
        assert!(is_supported_backend(FILE_BACKEND));
        assert!(is_supported_backend(PROTECTED_FILE_BACKEND));
        assert!(!is_supported_backend("keychain-v1"));
    }

    #[test]
    fn independent_protected_file_backend_requires_explicit_paths() {
        let _guard = TEST_ENV_LOCK.lock().unwrap();
        let root = env::temp_dir().join(format!("nuis-protected-backend-{}", std::process::id()));
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
                4,
                &"d".repeat(64)
            ),
            AnchorCheck::Invalid
        ));
        env::set_var(ANCHOR_MARKER_ENV, &marker);
        env::remove_var(ANCHOR_ROOT_ENV);
        assert!(matches!(
            enforce(
                &root.join("registry.toml"),
                "registry-v1",
                4,
                &"d".repeat(64)
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
                4,
                &"d".repeat(64)
            ),
            AnchorCheck::Accepted
        ));
        assert!(matches!(
            enforce(
                &root.join("registry.toml"),
                "registry-v1",
                4,
                &"d".repeat(64)
            ),
            AnchorCheck::Accepted
        ));
        env::set_var(ANCHOR_PATH_ENV, marker);
        assert!(matches!(
            enforce(
                &root.join("registry.toml"),
                "registry-v1",
                4,
                &"d".repeat(64)
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
    fn independent_protected_backend_rejects_world_writable_parent_and_parent_symlink() {
        let _guard = TEST_ENV_LOCK.lock().unwrap();
        let root = env::temp_dir().join(format!(
            "nuis-protected-backend-perm-{}",
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
                4,
                &"d".repeat(64)
            ),
            AnchorCheck::Invalid
        ));
        fs::set_permissions(&anchor_parent, PermissionsExt::from_mode(0o755)).unwrap();
        fs::set_permissions(&marker_parent, PermissionsExt::from_mode(0o755)).unwrap();
        assert!(matches!(
            enforce(
                &root.join("registry.toml"),
                "registry-v1",
                4,
                &"d".repeat(64)
            ),
            AnchorCheck::Invalid
        ));
        fs::set_permissions(&anchor_parent, PermissionsExt::from_mode(0o700)).unwrap();
        fs::set_permissions(&marker_parent, PermissionsExt::from_mode(0o700)).unwrap();
        assert!(matches!(
            enforce(
                &root.join("registry.toml"),
                "registry-v1",
                4,
                &"d".repeat(64)
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
                4,
                &"d".repeat(64)
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
    fn independent_protected_backend_ignores_deleted_ordinary_file_anchor_pair() {
        let _guard = TEST_ENV_LOCK.lock().unwrap();
        let root = env::temp_dir().join(format!(
            "nuis-protected-backend-ordinary-delete-{}",
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
            enforce(&registry, "registry-v1", 5, &"e".repeat(64)),
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
            enforce(&registry, "registry-v1", 4, &"d".repeat(64)),
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
    fn independently_uses_filesystem_lease_for_malformed_lock() {
        let root = env::temp_dir().join(format!("nuis-malformed-lock-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let lock = root.join("anchor.lock");
        fs::write(&lock, "partial").unwrap();
        assert!(!lock_is_stale(&lock, now_unix_ms().unwrap()));
        assert!(lock_is_stale(&lock, u128::MAX));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn independently_detects_anchor_deletion_with_marker() {
        let root = env::temp_dir().join(format!("nuis-anchor-marker-{}", std::process::id()));
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
                4,
                &"d".repeat(64)
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
                4,
                &"d".repeat(64)
            ),
            AnchorCheck::Invalid
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn independently_migrates_existing_anchor_to_marker() {
        let root = env::temp_dir().join(format!("nuis-anchor-migration-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let anchor = root.join("anchor.toml");
        let marker = default_marker_path(&anchor);
        persist_anchor(
            &anchor,
            TrustAnchorBackend::File,
            "registry-v1",
            5,
            &"e".repeat(64),
        )
        .unwrap();
        assert!(matches!(
            enforce_at_paths(
                &anchor,
                &marker,
                TrustAnchorBackend::File,
                "registry-v1",
                5,
                &"f".repeat(64)
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
                5,
                &"e".repeat(64)
            ),
            AnchorCheck::Accepted
        ));
        assert_eq!(
            read_marker(&marker)
                .unwrap()
                .unwrap()
                .initialized_generation,
            5
        );
        fs::remove_dir_all(root).unwrap();
    }
}
