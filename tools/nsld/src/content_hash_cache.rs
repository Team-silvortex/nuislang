use super::fnv1a64_hex;
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::SystemTime,
};

const MAX_CACHED_FILE_HASHES: usize = 64;
static FILE_HASH_CACHE: OnceLock<Mutex<VecDeque<FileHashCacheEntry>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileFingerprint {
    size: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    #[cfg(unix)]
    changed_seconds: i64,
    #[cfg(unix)]
    changed_nanoseconds: i64,
}

#[derive(Debug, Clone)]
struct FileHashCacheEntry {
    path: PathBuf,
    fingerprint: FileFingerprint,
    hash: String,
}

pub(crate) fn cached_file_content_hash(path: &Path) -> Result<String, String> {
    let fingerprint_before = file_fingerprint(path)?;
    let cache = FILE_HASH_CACHE.get_or_init(|| Mutex::new(VecDeque::new()));
    {
        let mut entries = cache
            .lock()
            .map_err(|_| "nsld file hash cache lock is poisoned".to_owned())?;
        if let Some(index) = entries
            .iter()
            .position(|entry| entry.path == path && entry.fingerprint == fingerprint_before)
        {
            let entry = entries
                .remove(index)
                .expect("file hash cache index came from the same deque");
            let hash = entry.hash.clone();
            entries.push_back(entry);
            return Ok(hash);
        }
    }

    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read `{}` for hashing: {error}", path.display()))?;
    let fingerprint_after = file_fingerprint(path)?;
    if fingerprint_before != fingerprint_after {
        return Err(format!(
            "file `{}` changed while Nsld was hashing it",
            path.display()
        ));
    }
    let hash = fnv1a64_hex(&bytes);
    let mut entries = cache
        .lock()
        .map_err(|_| "nsld file hash cache lock is poisoned".to_owned())?;
    entries.retain(|entry| entry.path != path);
    entries.push_back(FileHashCacheEntry {
        path: path.to_path_buf(),
        fingerprint: fingerprint_after,
        hash: hash.clone(),
    });
    while entries.len() > MAX_CACHED_FILE_HASHES {
        entries.pop_front();
    }
    Ok(hash)
}

pub(crate) fn file_fingerprint(path: &Path) -> Result<FileFingerprint, String> {
    let metadata = fs::metadata(path).map_err(|error| {
        format!(
            "failed to inspect `{}` for hashing: {error}",
            path.display()
        )
    })?;
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;

    Ok(FileFingerprint {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        #[cfg(unix)]
        device: metadata.dev(),
        #[cfg(unix)]
        inode: metadata.ino(),
        #[cfg(unix)]
        changed_seconds: metadata.ctime(),
        #[cfg(unix)]
        changed_nanoseconds: metadata.ctime_nsec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn caches_stable_content_and_invalidates_content_changes() {
        let path = unique_temp_path();
        fs::write(&path, b"first").unwrap();
        let first = cached_file_content_hash(&path).unwrap();
        assert_eq!(cached_file_content_hash(&path).unwrap(), first);

        fs::write(&path, b"other").unwrap();
        let same_size = cached_file_content_hash(&path).unwrap();
        assert_ne!(same_size, first);

        fs::write(&path, b"second-value").unwrap();
        let second = cached_file_content_hash(&path).unwrap();
        fs::remove_file(path).unwrap();

        assert_ne!(second, same_size);
    }

    fn unique_temp_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!(
            "nsld-content-hash-cache-{}-{nanos}",
            std::process::id()
        ))
    }
}
