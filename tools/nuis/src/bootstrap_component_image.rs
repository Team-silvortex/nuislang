use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn read_image(path: &Path, label: &str) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read {label} `{}`: {error}", path.display()))?;
    if bytes.is_empty() {
        return Err(format!("{label} cannot be empty"));
    }
    Ok(bytes)
}

pub(crate) struct StagedImage {
    path: PathBuf,
}

impl StagedImage {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for StagedImage {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(crate) fn stage_verified_image(
    bytes: &[u8],
    receipt_path: &Path,
) -> Result<StagedImage, String> {
    let root = receipt_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !root.is_dir() {
        return Err(format!(
            "compiler component dispatch output directory `{}` does not exist",
            root.display()
        ));
    }
    for _ in 0..32 {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut name = format!(".nuis-stage-driver-{}-{sequence}", std::process::id());
        if cfg!(windows) {
            name.push_str(".exe");
        }
        let path = root.join(name);
        let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create private compiler image staging slot: {error}"
                ))
            }
        };
        let staged = StagedImage { path };
        write_executable(&mut file, staged.path(), bytes)?;
        return Ok(staged);
    }
    Err("failed to allocate a unique private compiler image staging slot".to_owned())
}

fn write_executable(file: &mut fs::File, path: &Path, bytes: &[u8]) -> Result<(), String> {
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("failed to persist verified compiler image bytes: {error}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o700))
            .map_err(|error| {
                format!("failed to make verified compiler image executable: {error}")
            })?;
    }
    let persisted = fs::read(path)
        .map_err(|error| format!("failed to reread staged compiler image: {error}"))?;
    if persisted != bytes {
        return Err("staged compiler image bytes changed before execution".to_owned());
    }
    Ok(())
}

pub(crate) fn write_new(path: &Path, bytes: &[u8], label: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            format!(
                "failed to create {label} `{}` without replacement: {error}",
                path.display()
            )
        })?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("failed to persist {label} `{}`: {error}", path.display()))
}
