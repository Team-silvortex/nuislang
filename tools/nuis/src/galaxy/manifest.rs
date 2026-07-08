use super::*;

pub(super) fn render_manifest(manifest: &GalaxyManifest) -> String {
    let mut source = format!(
        "manifest_schema = \"{}\"\nname = \"{}\"\nversion = \"{}\"\npackage_kind = \"{}\"\n",
        manifest.manifest_schema,
        escape(&manifest.name),
        escape(&manifest.version),
        escape(&manifest.package_kind),
    );
    if let Some(framework) = &manifest.framework {
        source.push_str(&format!("framework = \"{}\"\n", escape(framework)));
    }
    source.push_str(&format!(
        "project = \"{}\"\nsummary = \"{}\"\nlicense = \"{}\"\nrepository = \"{}\"\nauthors = {}\ninclude = {}\n",
        escape(&manifest.project),
        escape(&manifest.summary),
        escape(&manifest.license),
        escape(&manifest.repository),
        render_string_array(&manifest.authors),
        render_string_array(&manifest.include),
    ));
    source
}

pub(super) fn render_ns_nova_manifest(manifest: &NsNovaManifest) -> String {
    let mut source = format!(
        "framework_schema = \"{}\"\nframework = \"{}\"\nproject = \"{}\"\n",
        escape(&manifest.framework_schema),
        escape(&manifest.framework),
        escape(&manifest.project),
    );
    if let Some(value) = &manifest.stdlib_schema {
        source.push_str(&format!("stdlib_schema = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.stdlib_manifest {
        source.push_str(&format!("stdlib_manifest = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.family_schema {
        source.push_str(&format!("family_schema = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.entry_cpu_unit {
        source.push_str(&format!("entry_cpu_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.primary_data_unit {
        source.push_str(&format!("primary_data_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.primary_shader_unit {
        source.push_str(&format!("primary_shader_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.primary_kernel_unit {
        source.push_str(&format!("primary_kernel_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.render_schema {
        source.push_str(&format!("render_schema = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.render_owner_unit {
        source.push_str(&format!("render_owner_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.render_bridge_unit {
        source.push_str(&format!("render_bridge_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.render_surface_unit {
        source.push_str(&format!("render_surface_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.selection_schema {
        source.push_str(&format!("selection_schema = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.selection_owner_unit {
        source.push_str(&format!("selection_owner_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.selection_bridge_unit {
        source.push_str(&format!("selection_bridge_unit = \"{}\"\n", escape(value)));
    }
    if let Some(value) = &manifest.selection_render_unit {
        source.push_str(&format!("selection_render_unit = \"{}\"\n", escape(value)));
    }
    source.push_str(&format!(
        "stdlib_sources = {}\nfamily_layers = {}\nrender_links = {}\nselection_controls = {}\ncpu_units = {}\ndata_units = {}\nshader_units = {}\nkernel_units = {}\n",
        render_string_array(&manifest.stdlib_sources),
        render_string_array(&manifest.family_layers),
        render_string_array(&manifest.render_links),
        render_string_array(&manifest.selection_controls),
        render_string_array(&manifest.cpu_units),
        render_string_array(&manifest.data_units),
        render_string_array(&manifest.shader_units),
        render_string_array(&manifest.kernel_units),
    ));
    source
}

pub(super) fn parse_manifest(source: &str, path: &Path) -> Result<GalaxyManifest, String> {
    let manifest = GalaxyManifest {
        manifest_schema: parse_required_string(source, "manifest_schema", path)?,
        name: parse_required_string(source, "name", path)?,
        version: parse_required_string(source, "version", path)?,
        package_kind: parse_required_string(source, "package_kind", path)?,
        framework: parse_optional_string(source, "framework"),
        project: parse_required_string(source, "project", path)?,
        summary: parse_optional_string(source, "summary").unwrap_or_default(),
        license: parse_optional_string(source, "license").unwrap_or_default(),
        repository: parse_optional_string(source, "repository").unwrap_or_default(),
        authors: parse_optional_string_array(source, "authors").unwrap_or_default(),
        include: parse_optional_string_array(source, "include").unwrap_or_default(),
    };
    if manifest.manifest_schema != "galaxy-manifest-v1" {
        return Err(format!(
            "galaxy manifest `{}` has unsupported schema `{}`",
            path.display(),
            manifest.manifest_schema
        ));
    }
    validate_galaxy_token("name", &manifest.name, path)?;
    validate_galaxy_token("version", &manifest.version, path)?;
    validate_relative_bundle_path("project", &manifest.project, path)?;
    for item in &manifest.include {
        validate_relative_bundle_path("include", item, path)?;
    }
    Ok(manifest)
}

pub(super) fn parse_ns_nova_manifest(source: &str, path: &Path) -> Result<NsNovaManifest, String> {
    Ok(NsNovaManifest {
        framework_schema: parse_required_string(source, "framework_schema", path)?,
        framework: parse_required_string(source, "framework", path)?,
        project: parse_required_string(source, "project", path)?,
        stdlib_schema: parse_optional_string(source, "stdlib_schema"),
        stdlib_manifest: parse_optional_string(source, "stdlib_manifest"),
        stdlib_sources: parse_optional_string_array(source, "stdlib_sources").unwrap_or_default(),
        family_schema: parse_optional_string(source, "family_schema"),
        family_layers: parse_optional_string_array(source, "family_layers").unwrap_or_default(),
        entry_cpu_unit: parse_optional_string(source, "entry_cpu_unit"),
        primary_data_unit: parse_optional_string(source, "primary_data_unit"),
        primary_shader_unit: parse_optional_string(source, "primary_shader_unit"),
        primary_kernel_unit: parse_optional_string(source, "primary_kernel_unit"),
        render_links: parse_optional_string_array(source, "render_links").unwrap_or_default(),
        render_schema: parse_optional_string(source, "render_schema"),
        render_owner_unit: parse_optional_string(source, "render_owner_unit"),
        render_bridge_unit: parse_optional_string(source, "render_bridge_unit"),
        render_surface_unit: parse_optional_string(source, "render_surface_unit"),
        selection_schema: parse_optional_string(source, "selection_schema"),
        selection_owner_unit: parse_optional_string(source, "selection_owner_unit"),
        selection_bridge_unit: parse_optional_string(source, "selection_bridge_unit"),
        selection_render_unit: parse_optional_string(source, "selection_render_unit"),
        selection_controls: parse_optional_string_array(source, "selection_controls")
            .unwrap_or_default(),
        cpu_units: parse_optional_string_array(source, "cpu_units").unwrap_or_default(),
        data_units: parse_optional_string_array(source, "data_units").unwrap_or_default(),
        shader_units: parse_optional_string_array(source, "shader_units").unwrap_or_default(),
        kernel_units: parse_optional_string_array(source, "kernel_units").unwrap_or_default(),
    })
}

pub(super) fn parse_required_string(
    source: &str,
    key: &str,
    path: &Path,
) -> Result<String, String> {
    parse_optional_string(source, key).ok_or_else(|| {
        format!(
            "galaxy manifest `{}` is missing required key `{key}`",
            path.display()
        )
    })
}

pub(super) fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return Some(value[1..value.len() - 1].to_owned());
            }
            return None;
        }
    }
    None
}

pub(super) fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    let lines = source.lines().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index].trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                if inner.trim().is_empty() {
                    return Some(Vec::new());
                }
                let mut items = Vec::new();
                for item in inner.split(',') {
                    let item = item.trim();
                    if !(item.starts_with('"') && item.ends_with('"') && item.len() >= 2) {
                        return None;
                    }
                    items.push(item[1..item.len() - 1].to_owned());
                }
                return Some(items);
            }
            if !value.starts_with('[') {
                return None;
            }
            let mut items = Vec::new();
            index += 1;
            while index < lines.len() {
                let item = lines[index].trim().trim_end_matches(',').trim();
                if item == "]" {
                    return Some(items);
                }
                if !(item.starts_with('"') && item.ends_with('"') && item.len() >= 2) {
                    return None;
                }
                items.push(item[1..item.len() - 1].to_owned());
                index += 1;
            }
            return None;
        }
        index += 1;
    }
    None
}

pub(super) fn parse_local_index_entry(
    source: &str,
    path: &Path,
) -> Result<LocalGalaxyIndexEntry, String> {
    let entry = LocalGalaxyIndexEntry {
        name: parse_required_string(source, "name", path)?,
        version: parse_required_string(source, "version", path)?,
        package: parse_required_string(source, "package", path)?,
        project: parse_required_string(source, "project", path)?,
        abi: parse_optional_string_array(source, "abi").unwrap_or_default(),
        bundle_bytes: parse_optional_u64(source, "bundle_bytes"),
        bundle_fnv1a64: parse_optional_string(source, "bundle_fnv1a64"),
    };
    validate_galaxy_token("name", &entry.name, path)?;
    validate_galaxy_token("version", &entry.version, path)?;
    validate_relative_bundle_path("project", &entry.project, path)?;
    Ok(entry)
}

pub(super) fn select_local_entry(
    name: &str,
    version: Option<&str>,
) -> Result<LocalGalaxyIndexEntry, String> {
    validate_galaxy_token("name", name, Path::new("<galaxy local lookup>"))?;
    if let Some(version) = version {
        validate_galaxy_token("version", version, Path::new("<galaxy local lookup>"))?;
    }
    let entries = list_local()?;
    let mut matches = entries
        .into_iter()
        .filter(|entry| entry.name == name)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Err(format!("no local galaxy package named `{name}`"));
    }
    matches.sort_by(|lhs, rhs| compare_version(&lhs.version, &rhs.version));
    let chosen = if let Some(version) = version {
        matches
            .into_iter()
            .find(|entry| entry.version == version)
            .ok_or_else(|| format!("no local galaxy package `{name}` with version `{version}`"))?
    } else {
        matches.pop().unwrap()
    };
    Ok(chosen)
}

pub(super) fn compare_version(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    let lhs_parts = parse_version_parts(lhs);
    let rhs_parts = parse_version_parts(rhs);
    if lhs_parts.is_empty() || rhs_parts.is_empty() {
        return lhs.cmp(rhs);
    }
    let width = lhs_parts.len().max(rhs_parts.len());
    for index in 0..width {
        let lhs_part = lhs_parts.get(index).copied().unwrap_or(0);
        let rhs_part = rhs_parts.get(index).copied().unwrap_or(0);
        match lhs_part.cmp(&rhs_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    lhs.cmp(rhs)
}

pub(super) fn parse_version_parts(value: &str) -> Vec<u64> {
    value
        .split('.')
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

pub(super) fn render_string_array(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|value| format!("\"{}\"", escape(value)))
        .collect::<Vec<_>>();
    format!("[{}]", items.join(", "))
}

pub(super) fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(super) fn parse_optional_u64(source: &str, key: &str) -> Option<u64> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().parse::<u64>().ok();
        }
    }
    None
}

pub(super) fn fnv1a64_hex(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}
