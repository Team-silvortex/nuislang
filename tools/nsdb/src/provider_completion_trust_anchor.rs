use std::{
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

const ANCHOR_PROTOCOL: &str = "nuis-provider-completion-trust-anchor-v1";
const ANCHOR_PATH_ENV: &str = "NUIS_PROVIDER_COMPLETION_TRUST_ANCHOR";

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
    let path = env::var_os(ANCHOR_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{}.anchor", registry_path.to_string_lossy())));
    enforce_at_path(&path, registry_protocol, generation, registry_hash)
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

struct AnchorLock(PathBuf);

impl AnchorLock {
    fn acquire(path: PathBuf) -> Result<Self, ()> {
        for _ in 0..200 {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    writeln!(file, "pid = {}", std::process::id()).map_err(|_| ())?;
                    file.sync_all().map_err(|_| ())?;
                    return Ok(Self(path));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
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
        let _ = fs::remove_file(&self.0);
    }
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
}
