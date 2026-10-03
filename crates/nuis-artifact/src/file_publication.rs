use std::{
    fs::{self, File, OpenOptions, Permissions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::ArtifactError;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Publish complete bytes on a fresh file identity, without executing or signing them.
/// On Unix, newly written files are private and non-executable until admitted.
pub fn atomic_write_artifact_file(path: &Path, bytes: &[u8]) -> Result<(), ArtifactError> {
    publish(path, None, |file| file.write_all(bytes))
}

/// Copy through a fresh sibling file, preserving source permissions but not inode identity.
pub fn atomic_copy_artifact_file(source: &Path, target: &Path) -> Result<(), ArtifactError> {
    let mut input =
        File::open(source).map_err(|error| failure("open copy source", source, error))?;
    let metadata = input
        .metadata()
        .map_err(|error| failure("inspect copy source", source, error))?;
    if !metadata.is_file() {
        return Err(ArtifactError::new(format!(
            "artifact copy source must be a regular file: `{}`",
            source.display()
        )));
    }
    publish(target, Some(metadata.permissions()), |file| {
        io::copy(&mut input, file).map(|_| ())
    })
}

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn publish(
    target: &Path,
    permissions: Option<Permissions>,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<(), ArtifactError> {
    let parent = target
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    target.file_name().ok_or_else(|| {
        ArtifactError::new(format!(
            "artifact output needs a file name: `{}`",
            target.display()
        ))
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| failure("create publication directory", parent, error))?;
    validate_target(target)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ArtifactError::new(format!("publication clock unavailable: {error}")))?
        .as_nanos();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temporary_path = parent.join(format!(
        ".nuis-publish-{}-{nonce}-{sequence}.tmp",
        std::process::id()
    ));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary_path)
        .map_err(|error| failure("create private publication file", &temporary_path, error))?;
    // Only clean a file whose exclusive creation succeeded; never unlink a collision.
    let temporary = TemporaryFile(temporary_path);
    write(&mut file).map_err(|error| failure("write publication file", &temporary.0, error))?;
    if let Some(permissions) = permissions {
        file.set_permissions(permissions)
            .map_err(|error| failure("set publication permissions", &temporary.0, error))?;
    }
    file.sync_all()
        .map_err(|error| failure("sync publication file", &temporary.0, error))?;
    drop(file);
    validate_target(target)?;
    fs::rename(&temporary.0, target)
        .map_err(|error| failure("atomically publish artifact file", target, error))?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| failure("sync directory after publication", parent, error))?;
    Ok(())
}

fn validate_target(target: &Path) -> Result<(), ArtifactError> {
    match fs::symlink_metadata(target) {
        Ok(metadata) if !metadata.file_type().is_file() => Err(ArtifactError::new(format!(
            "artifact publication refuses a non-regular destination: `{}`",
            target.display()
        ))),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(failure("inspect publication destination", target, error)),
    }
}

fn failure(action: &str, path: &Path, error: io::Error) -> ArtifactError {
    ArtifactError::new(format!("failed to {action} `{}`: {error}", path.display()))
}

#[cfg(test)]
#[path = "file_publication_tests.rs"]
mod tests;
