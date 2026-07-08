use super::*;

pub(super) fn decode_bundle(bytes: &[u8], path: &Path) -> Result<InspectedGalaxyBundle, String> {
    if bytes.len() < 18 {
        return Err(format!(
            "`{}` is too short to be a galaxy bundle",
            path.display()
        ));
    }
    if &bytes[..8] != GALAXY_MAGIC {
        return Err(format!(
            "`{}` does not start with the galaxy bundle magic",
            path.display()
        ));
    }
    let version = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
    if version != GALAXY_BUNDLE_VERSION {
        return Err(format!(
            "`{}` has unsupported galaxy bundle version {}; expected {}",
            path.display(),
            version,
            GALAXY_BUNDLE_VERSION
        ));
    }
    let manifest_len = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
    let entry_count = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let mut offset = 18usize;
    if bytes.len() < offset + manifest_len {
        return Err(format!(
            "`{}` is truncated before manifest payload",
            path.display()
        ));
    }
    let manifest_source =
        std::str::from_utf8(&bytes[offset..offset + manifest_len]).map_err(|e| {
            format!(
                "`{}` has invalid utf-8 in galaxy manifest payload: {e}",
                path.display()
            )
        })?;
    offset += manifest_len;
    let manifest = parse_manifest(manifest_source, path)?;

    let mut entries = Vec::new();
    for _ in 0..entry_count {
        if bytes.len() < offset + 12 {
            return Err(format!(
                "`{}` is truncated while decoding bundle entries",
                path.display()
            ));
        }
        let path_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let content_len =
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        if bytes.len() < offset + path_len + content_len {
            return Err(format!(
                "`{}` is truncated while decoding entry payload",
                path.display()
            ));
        }
        let entry_path = std::str::from_utf8(&bytes[offset..offset + path_len]).map_err(|e| {
            format!(
                "`{}` has invalid utf-8 in bundle entry path: {e}",
                path.display()
            )
        })?;
        validate_relative_bundle_path("entry path", entry_path, path)?;
        offset += path_len + content_len;
        entries.push(GalaxyBundleEntry {
            path: entry_path.to_owned(),
            bytes: content_len,
        });
    }

    if offset != bytes.len() {
        return Err(format!(
            "`{}` has trailing bytes after decoding galaxy bundle",
            path.display()
        ));
    }

    Ok(InspectedGalaxyBundle { manifest, entries })
}

pub(super) fn extract_bundle(bytes: &[u8], path: &Path, output: &Path) -> Result<(), String> {
    if bytes.len() < 18 || &bytes[..8] != GALAXY_MAGIC {
        return Err(format!("`{}` is not a valid galaxy bundle", path.display()));
    }
    let manifest_len = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
    let entry_count = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let mut offset = 18usize + manifest_len;
    if bytes.len() < offset {
        return Err(format!(
            "`{}` is truncated before install entries",
            path.display()
        ));
    }
    for _ in 0..entry_count {
        if bytes.len() < offset + 12 {
            return Err(format!(
                "`{}` is truncated while decoding install entries",
                path.display()
            ));
        }
        let path_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let content_len =
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        if bytes.len() < offset + path_len + content_len {
            return Err(format!(
                "`{}` is truncated while decoding install entry payload",
                path.display()
            ));
        }
        let relative = std::str::from_utf8(&bytes[offset..offset + path_len]).map_err(|e| {
            format!(
                "`{}` has invalid utf-8 in install entry path: {e}",
                path.display()
            )
        })?;
        offset += path_len;
        let content = &bytes[offset..offset + content_len];
        offset += content_len;
        let relative = validate_relative_bundle_path("entry path", relative, path)?;
        let target = output.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
        }
        fs::write(&target, content)
            .map_err(|error| format!("failed to write `{}`: {error}", target.display()))?;
    }
    Ok(())
}
