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
const FILE_BACKEND: &str = "file-v1";
const LOCK_PROTOCOL: &str = "nuis-provider-completion-trust-anchor-lock-v1";
const LOCK_STALE_AFTER_MS: u128 = 30_000;
static LOCK_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
    let backend_supported = env::var_os(ANCHOR_BACKEND_ENV).map_or(true, |backend| {
        backend.to_str().is_some_and(is_supported_backend)
    });
    if !backend_supported {
        return AnchorCheck::Invalid;
    }
    let path = env::var_os(ANCHOR_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{}.anchor", registry_path.to_string_lossy())));
    enforce_at_path(&path, registry_protocol, generation, registry_hash)
}

fn is_supported_backend(backend: &str) -> bool {
    backend == FILE_BACKEND
}

fn enforce_at_path(
    path: &Path,
    registry_protocol: &str,
    generation: usize,
    registry_hash: &str,
) -> AnchorCheck {
    let lock_path = PathBuf::from(format!("{}.lock", path.to_string_lossy()));
    let Ok(_guard) = AnchorLock::acquire(lock_path) else {
        return AnchorCheck::Invalid;
    };
    match read_anchor(&path) {
        Ok(None) => persist_anchor(&path, registry_protocol, generation, registry_hash)
            .map_or(AnchorCheck::Invalid, |_| AnchorCheck::Accepted),
        Ok(Some(anchor)) if anchor.registry_protocol != registry_protocol => AnchorCheck::Invalid,
        Ok(Some(anchor)) if generation < anchor.highest_generation => AnchorCheck::Rollback,
        Ok(Some(anchor)) if generation == anchor.highest_generation => {
            if registry_hash == anchor.registry_hash {
                AnchorCheck::Accepted
            } else {
                AnchorCheck::Fork
            }
        }
        Ok(Some(_)) => persist_anchor(&path, registry_protocol, generation, registry_hash)
            .map_or(AnchorCheck::Invalid, |_| AnchorCheck::Accepted),
        Err(()) => AnchorCheck::Invalid,
    }
}

struct TrustAnchor {
    registry_protocol: String,
    highest_generation: usize,
    registry_hash: String,
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

fn persist_anchor(
    path: &Path,
    registry_protocol: &str,
    generation: usize,
    hash: &str,
) -> Result<(), ()> {
    let temporary = PathBuf::from(format!("{}.tmp", path.to_string_lossy()));
    let content = format!(
        "protocol = \"{ANCHOR_PROTOCOL}\"\nregistry_protocol = \"{registry_protocol}\"\nhighest_generation = {generation}\nregistry_hash = \"{hash}\"\n"
    );
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| ())?;
    file.write_all(content.as_bytes()).map_err(|_| ())?;
    file.sync_all().map_err(|_| ())?;
    fs::rename(&temporary, path).map_err(|_| ())?;
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
            match OpenOptions::new().write(true).create_new(true).open(&path) {
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
        assert!(!is_supported_backend("keychain-v1"));
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
}
