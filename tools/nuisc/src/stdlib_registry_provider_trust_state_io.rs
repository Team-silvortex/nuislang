#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const LOCK_PROTOCOL: &str = "nuis-galaxy-provider-trust-state-lock-v1";
const LOCK_STALE_AFTER_MS: u128 = 30_000;
const LOCK_ATTEMPTS: usize = 200;
const MAX_LOCK_BYTES: u64 = 4096;
static LOCK_SEQUENCE: AtomicU64 = AtomicU64::new(0);
type LockSnapshot = (Vec<u8>, Option<u128>);

pub(super) fn validate_state_target(path: &Path) -> Result<(), String> {
    if path.file_name().is_none() {
        return Err("Galaxy provider trust state path must name a file".to_owned());
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            #[cfg(unix)]
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(format!(
                    "Galaxy provider trust state `{}` must not grant group or other permissions",
                    path.display()
                ));
            }
            Ok(())
        }
        Ok(_) => Err(format!(
            "Galaxy provider trust state `{}` must be a regular non-symlink file",
            path.display()
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to inspect Galaxy provider trust state `{}`: {error}",
            path.display()
        )),
    }
}

pub(super) fn persist_state(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let token = format!(
        "{}-{}-{}",
        std::process::id(),
        now_unix_ms()?,
        LOCK_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let temporary = sibling_with_suffix(path, &format!(".tmp-{token}"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary).map_err(|error| {
        format!(
            "failed to create Galaxy provider trust state temporary `{}`: {error}",
            temporary.display()
        )
    })?;
    let result = file
        .write_all(bytes)
        .and_then(|_| file.sync_all())
        .and_then(|_| fs::rename(&temporary, path));
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(format!(
            "failed to atomically persist Galaxy provider trust state `{}`: {error}",
            path.display()
        ));
    }
    sync_parent(path)?;
    Ok(())
}

fn sync_parent(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    OpenOptions::new()
        .read(true)
        .open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "failed to sync Galaxy provider trust state parent `{}`: {error}",
                parent.display()
            )
        })
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .ok_or_else(|| "Galaxy provider trust state path must name a file".to_owned())?;
    let mut name = file_name.to_os_string();
    name.push(suffix);
    Ok(path.with_file_name(name))
}

pub(super) struct TrustStateLock {
    path: PathBuf,
    owner_token: String,
}

impl TrustStateLock {
    pub(super) fn acquire(state_path: &Path) -> Result<Self, String> {
        let path = sibling_with_suffix(state_path, ".lock")?;
        for _ in 0..LOCK_ATTEMPTS {
            let created_unix_ms = now_unix_ms()?;
            let owner_token = format!(
                "{}-{created_unix_ms}-{}",
                std::process::id(),
                LOCK_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            );
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            match options.open(&path) {
                Ok(mut file) => {
                    let source = format!(
                        "protocol = \"{LOCK_PROTOCOL}\"\nowner_pid = {}\ncreated_unix_ms = {created_unix_ms}\nowner_token = \"{owner_token}\"\n",
                        std::process::id()
                    );
                    if let Err(error) = file
                        .write_all(source.as_bytes())
                        .and_then(|_| file.sync_all())
                    {
                        let _ = fs::remove_file(&path);
                        return Err(format!(
                            "failed to initialize Galaxy provider trust state lock: {error}"
                        ));
                    }
                    return Ok(Self { path, owner_token });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if let Some(snapshot) = stale_lock_snapshot(&path, created_unix_ms)? {
                        let unchanged = read_lock_snapshot(&path)?
                            .is_some_and(|(current, _)| current == snapshot);
                        if !unchanged {
                            thread::sleep(Duration::from_millis(5));
                            continue;
                        }
                        match fs::remove_file(&path) {
                            Ok(()) => continue,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                            Err(error) => {
                                return Err(format!(
                                    "failed to recover stale Galaxy provider trust state lock: {error}"
                                ));
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => {
                    return Err(format!(
                        "failed to acquire Galaxy provider trust state lock `{}`: {error}",
                        path.display()
                    ));
                }
            }
        }
        Err("timed out acquiring Galaxy provider trust state lock".to_owned())
    }
}

impl Drop for TrustStateLock {
    fn drop(&mut self) {
        let still_owned = read_lock_snapshot(&self.path)
            .ok()
            .flatten()
            .and_then(|(source, _)| String::from_utf8(source).ok())
            .and_then(|source| string_field(&source, "owner_token"))
            .is_some_and(|owner| owner == self.owner_token);
        if still_owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn stale_lock_snapshot(path: &Path, now_unix_ms: u128) -> Result<Option<Vec<u8>>, String> {
    let Some((snapshot, modified_unix_ms)) = read_lock_snapshot(path)? else {
        return Ok(None);
    };
    let created_unix_ms = std::str::from_utf8(&snapshot)
        .ok()
        .and_then(valid_lock_created_unix_ms)
        .unwrap_or_else(|| modified_unix_ms.unwrap_or(now_unix_ms));
    Ok((now_unix_ms.saturating_sub(created_unix_ms) > LOCK_STALE_AFTER_MS).then_some(snapshot))
}

fn read_lock_snapshot(path: &Path) -> Result<Option<LockSnapshot>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to inspect Galaxy provider trust state lock `{}`: {error}",
                path.display()
            ));
        }
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "Galaxy provider trust state lock `{}` must be a regular non-symlink file",
            path.display()
        ));
    }
    if metadata.len() > MAX_LOCK_BYTES {
        return Err(format!(
            "Galaxy provider trust state lock exceeds the {MAX_LOCK_BYTES}-byte limit"
        ));
    }
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to read Galaxy provider trust state lock `{}`: {error}",
                path.display()
            ));
        }
    };
    let mut snapshot = Vec::with_capacity(metadata.len() as usize + 1);
    (&mut file)
        .take(MAX_LOCK_BYTES + 1)
        .read_to_end(&mut snapshot)
        .map_err(|error| {
            format!(
                "failed to read Galaxy provider trust state lock `{}`: {error}",
                path.display()
            )
        })?;
    if snapshot.len() as u64 > MAX_LOCK_BYTES {
        return Err(format!(
            "Galaxy provider trust state lock exceeds the {MAX_LOCK_BYTES}-byte limit"
        ));
    }
    match fs::symlink_metadata(path) {
        Ok(current) if current.is_file() && !current.file_type().is_symlink() => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        _ => {
            return Err(format!(
                "Galaxy provider trust state lock `{}` changed type while being read",
                path.display()
            ));
        }
    }
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis());
    Ok(Some((snapshot, modified_unix_ms)))
}

fn valid_lock_created_unix_ms(source: &str) -> Option<u128> {
    (string_field(source, "protocol").as_deref() == Some(LOCK_PROTOCOL)
        && string_field(source, "owner_token").is_some_and(|token| !token.is_empty()))
    .then(|| integer_field::<u128>(source, "created_unix_ms"))
    .flatten()
}

fn string_field(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (candidate, raw) = line.split_once('=')?;
        (candidate.trim() == key).then(|| {
            raw.trim()
                .strip_prefix('"')?
                .strip_suffix('"')
                .map(str::to_owned)
        })?
    })
}

fn integer_field<T: std::str::FromStr>(source: &str, key: &str) -> Option<T> {
    source.lines().find_map(|line| {
        let (candidate, raw) = line.split_once('=')?;
        (candidate.trim() == key)
            .then(|| raw.trim().parse().ok())
            .flatten()
    })
}

fn now_unix_ms() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_lock_timestamp_requires_complete_owner_claim() {
        let complete = format!(
            "protocol = \"{LOCK_PROTOCOL}\"\ncreated_unix_ms = 7\nowner_token = \"owner\"\n"
        );
        assert_eq!(valid_lock_created_unix_ms(&complete), Some(7));
        assert_eq!(valid_lock_created_unix_ms("protocol = \"partial\"\n"), None);
        assert_eq!(
            valid_lock_created_unix_ms(&format!(
                "protocol = \"{LOCK_PROTOCOL}\"\ncreated_unix_ms = 7\nowner_token = \"\"\n"
            )),
            None
        );
    }

    #[test]
    fn incomplete_lock_becomes_recoverable_after_the_stale_window() {
        let token = LOCK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nuis_galaxy_trust_partial_lock_{}_{token}",
            std::process::id()
        ));
        fs::write(&path, b"protocol = \"").unwrap();
        let now = now_unix_ms().unwrap();
        assert!(stale_lock_snapshot(&path, now).unwrap().is_none());
        let stale = stale_lock_snapshot(&path, now + LOCK_STALE_AFTER_MS + 1)
            .unwrap()
            .unwrap();
        assert_eq!(stale, b"protocol = \"");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn disappearing_lock_is_a_normal_acquisition_race() {
        let token = LOCK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "nuis_galaxy_trust_missing_lock_{}_{token}",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        assert!(read_lock_snapshot(&path).unwrap().is_none());
    }
}
