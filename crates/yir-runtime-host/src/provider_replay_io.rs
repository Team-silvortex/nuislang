use std::{
    fs::{self, File, Metadata},
    io::Read,
    path::Path,
};

pub(super) fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    // Reject already-present pipes/devices before opening a potentially blocking file.
    let before_open = fs::metadata(path).map_err(|error| {
        format!(
            "failed to inspect provider replay `{}`: {error}",
            path.display()
        )
    })?;
    validate_metadata(&before_open, limit)?;
    let file = File::open(path).map_err(|error| {
        format!(
            "failed to open provider replay `{}`: {error}",
            path.display()
        )
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("failed to inspect provider replay: {error}"))?;
    validate_metadata(&metadata, limit)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    // Metadata is not a promise that the file cannot grow while being read.
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read provider replay: {error}"))?;
    if bytes.len() > limit {
        return Err("provider replay file grew beyond its byte budget".to_owned());
    }
    Ok(bytes)
}

fn validate_metadata(metadata: &Metadata, limit: usize) -> Result<(), String> {
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err("provider replay file exceeds its regular-file byte budget".to_owned());
    }
    Ok(())
}
