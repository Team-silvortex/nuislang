use std::{
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn atomic_write_executable(output_path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create final output directory `{}`: {error}",
            parent.display()
        )
    })?;
    let temp_path = temporary_output_path(output_path);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| {
                format!(
                    "failed to create temporary final output `{}`: {error}",
                    temp_path.display()
                )
            })?;
        file.write_all(bytes).map_err(|error| {
            format!(
                "failed to write temporary final output `{}`: {error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync temporary final output `{}`: {error}",
                temp_path.display()
            )
        })?;
        set_executable_permissions(&temp_path)?;
        file.sync_all().map_err(|error| {
            format!(
                "failed to sync executable permissions for `{}`: {error}",
                temp_path.display()
            )
        })?;
        drop(file);
        fs::rename(&temp_path, output_path).map_err(|error| {
            format!(
                "failed to atomically install final output `{}`: {error}",
                output_path.display()
            )
        })?;
        sync_parent_directory(parent)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn temporary_output_path(output_path: &Path) -> PathBuf {
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nuis-output");
    output_path.with_file_name(format!(
        ".{name}.nsld-{}-{sequence}.tmp",
        std::process::id()
    ))
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("failed to inspect `{}`: {error}", path.display()))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("failed to make `{}` executable: {error}", path.display()))
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) -> Result<(), String> {
    fs::File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            format!(
                "failed to sync output directory `{}`: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}
