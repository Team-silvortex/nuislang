use super::bundle::decode_bundle;
use super::local::{ensure_local_layout, local_index_root, local_packages_root};
use super::*;

pub fn pack(input: &Path, output: &Path) -> Result<PathBuf, String> {
    let checked = check(input)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
    }

    let manifest_source = fs::read_to_string(&checked.manifest_path).map_err(|error| {
        format!(
            "failed to read `{}` for pack: {error}",
            checked.manifest_path.display()
        )
    })?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(GALAXY_MAGIC);
    bytes.extend_from_slice(&GALAXY_BUNDLE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(manifest_source.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(checked.include_files.len() as u32).to_le_bytes());
    bytes.extend_from_slice(manifest_source.as_bytes());

    for path in &checked.include_files {
        let relative = path
            .strip_prefix(&checked.root)
            .unwrap_or(path)
            .display()
            .to_string();
        let content = fs::read(path)
            .map_err(|error| format!("failed to read `{}` for pack: {error}", path.display()))?;
        bytes.extend_from_slice(&(relative.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
        bytes.extend_from_slice(relative.as_bytes());
        bytes.extend_from_slice(&content);
    }

    fs::write(output, bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", output.display()))?;

    write_local_index_entry(&checked, output)?;
    Ok(output.to_path_buf())
}

pub fn inspect_bundle(input: &Path) -> Result<InspectedGalaxyBundle, String> {
    let bytes = fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    decode_bundle(&bytes, input)
}

pub fn publish_local(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    ensure_local_layout()?;
    let checked = check(input)?;
    let bundle_path = output.map(PathBuf::from).unwrap_or_else(|| {
        local_packages_root()
            .join(&checked.manifest.name)
            .join(&checked.manifest.version)
            .join(format!(
                "{}-{}.galaxy",
                checked.manifest.name, checked.manifest.version
            ))
    });
    let packed = pack(input, &bundle_path)?;
    Ok(packed)
}

fn write_local_index_entry(checked: &CheckedGalaxy, output: &Path) -> Result<(), String> {
    let package_dir = local_index_root().join(&checked.manifest.name);
    fs::create_dir_all(&package_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", package_dir.display()))?;
    let entry_path = package_dir.join(format!("{}.toml", checked.manifest.version));
    let mut abi_entries = checked
        .abi_entries
        .iter()
        .map(|(domain, abi)| format!("{domain}={abi}"))
        .collect::<Vec<_>>();
    abi_entries.sort();
    let bundle_bytes = fs::read(output).map_err(|error| {
        format!(
            "failed to read `{}` for local index: {error}",
            output.display()
        )
    })?;
    let bundle_len = bundle_bytes.len() as u64;
    let bundle_hash = fnv1a64_hex(&bundle_bytes);
    let source = format!(
        "name = \"{}\"\nversion = \"{}\"\npackage = \"{}\"\nproject = \"{}\"\nabi = {}\nbundle_bytes = {}\nbundle_fnv1a64 = \"{}\"\n",
        checked.manifest.name,
        checked.manifest.version,
        output.display(),
        checked.manifest.project,
        render_string_array(&abi_entries),
        bundle_len,
        bundle_hash
    );
    fs::write(&entry_path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", entry_path.display()))?;
    Ok(())
}
