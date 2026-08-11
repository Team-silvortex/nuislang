use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use super::{
    LoadedProject, ProjectGalaxyResolutionLockSummary, ProjectModuleOrigin,
    RenderedProjectGalaxyResolutionLock,
};

pub const PROJECT_GALAXY_RESOLUTION_LOCK_SCHEMA: &str = "nuis-galaxy-resolution-lock-v1";
pub const PROJECT_GALAXY_RESOLUTION_LOCK_DIGEST: &str = "sha256";

pub fn render_project_galaxy_resolution_lock(
    project: &LoadedProject,
) -> Result<RenderedProjectGalaxyResolutionLock, String> {
    let selections = selected_library_modules(project);
    let mut dependencies = project.resolved_galaxies.iter().collect::<Vec<_>>();
    dependencies.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then(lhs.version.cmp(&rhs.version))
            .then(lhs.package_id.cmp(&rhs.package_id))
    });

    let mut payload = String::new();
    let source_module_count = dependencies
        .iter()
        .map(|dependency| dependency.source_modules.len())
        .sum::<usize>();
    let library_module_count = dependencies
        .iter()
        .map(|dependency| dependency.library_modules.len())
        .sum::<usize>();
    let selected_library_module_count = selections.len();
    writeln!(payload, "dependency_count = {}", dependencies.len()).unwrap();
    writeln!(payload, "source_module_count = {source_module_count}").unwrap();
    writeln!(payload, "library_module_count = {library_module_count}").unwrap();
    writeln!(
        payload,
        "selected_library_module_count = {selected_library_module_count}"
    )
    .unwrap();

    for dependency in dependencies {
        validate_lock_token("galaxy name", &dependency.name)?;
        validate_lock_token("galaxy version", &dependency.version)?;
        validate_lock_token("galaxy package id", &dependency.package_id)?;
        if dependency.source_modules.len() != dependency.source_content_identities.len()
            || dependency.library_modules.len() != dependency.library_content_identities.len()
        {
            return Err(format!(
                "Galaxy `{}` has mismatched declared and content-identity tables",
                dependency.name
            ));
        }
        let mut requested_by = dependency.requested_by.clone();
        requested_by.sort();
        requested_by.dedup();
        let mut depends_on = dependency.depends_on.clone();
        depends_on.sort();
        depends_on.dedup();
        let mut blockers = dependency.auto_inject_blockers.clone();
        blockers.sort();
        blockers.dedup();

        let manifest_record = content_record(&dependency.manifest_content_identity, None)?;
        let source_records = content_records(&dependency.source_content_identities, None)?;
        let library_selections = dependency
            .library_modules
            .iter()
            .map(|library_module| {
                selections
                    .get(&(dependency.name.clone(), library_module.clone()))
                    .copied()
                    .unwrap_or("hidden")
            })
            .collect::<Vec<_>>();
        let library_records = content_records(
            &dependency.library_content_identities,
            Some(&library_selections),
        )?;

        payload.push_str("\n[[dependency]]\n");
        writeln!(
            payload,
            "name = \"{}\"",
            crate::aot_toml::escape_toml_string(&dependency.name)
        )
        .unwrap();
        writeln!(
            payload,
            "version = \"{}\"",
            crate::aot_toml::escape_toml_string(&dependency.version)
        )
        .unwrap();
        writeln!(
            payload,
            "package_id = \"{}\"",
            crate::aot_toml::escape_toml_string(&dependency.package_id)
        )
        .unwrap();
        writeln!(payload, "direct = {}", dependency.direct).unwrap();
        writeln!(
            payload,
            "requested_by = {}",
            crate::aot_toml::render_string_array(&requested_by)
        )
        .unwrap();
        writeln!(
            payload,
            "depends_on = {}",
            crate::aot_toml::render_string_array(&depends_on)
        )
        .unwrap();
        writeln!(
            payload,
            "library_import_policy = \"{}\"",
            dependency.library_import_policy.as_str()
        )
        .unwrap();
        writeln!(payload, "auto_injectable = {}", dependency.auto_injectable).unwrap();
        writeln!(
            payload,
            "auto_inject_blockers = {}",
            crate::aot_toml::render_string_array(&blockers)
        )
        .unwrap();
        writeln!(
            payload,
            "manifest_record = \"{}\"",
            crate::aot_toml::escape_toml_string(&manifest_record)
        )
        .unwrap();
        writeln!(
            payload,
            "source_module_records = {}",
            crate::aot_toml::render_string_array(&source_records)
        )
        .unwrap();
        writeln!(
            payload,
            "library_module_records = {}",
            crate::aot_toml::render_string_array(&library_records)
        )
        .unwrap();
    }

    let resolution_sha256 = format!(
        "sha256:{}",
        crate::digest_sha256::sha256_hex(payload.as_bytes())
    );
    let mut source = String::new();
    writeln!(
        source,
        "lock_schema = \"{PROJECT_GALAXY_RESOLUTION_LOCK_SCHEMA}\""
    )
    .unwrap();
    writeln!(
        source,
        "digest_contract = \"{PROJECT_GALAXY_RESOLUTION_LOCK_DIGEST}\""
    )
    .unwrap();
    writeln!(source, "resolution_sha256 = \"{resolution_sha256}\"").unwrap();
    writeln!(source, "payload_begin = true").unwrap();
    source.push_str(&payload);

    Ok(RenderedProjectGalaxyResolutionLock {
        source,
        summary: ProjectGalaxyResolutionLockSummary {
            schema: PROJECT_GALAXY_RESOLUTION_LOCK_SCHEMA.to_owned(),
            digest_contract: PROJECT_GALAXY_RESOLUTION_LOCK_DIGEST.to_owned(),
            resolution_sha256,
            dependencies: project.resolved_galaxies.len(),
            source_modules: source_module_count,
            library_modules: library_module_count,
            selected_library_modules: selected_library_module_count,
        },
    })
}

pub fn verify_project_galaxy_resolution_lock_source(
    source: &str,
    path: &Path,
) -> Result<ProjectGalaxyResolutionLockSummary, String> {
    if source.contains('\r') {
        return Err(format!(
            "Galaxy resolution lock `{}` must use canonical LF line endings",
            path.display()
        ));
    }
    if !source.ends_with('\n') {
        return Err(format!(
            "Galaxy resolution lock `{}` must end with a newline",
            path.display()
        ));
    }
    let schema = crate::aot_toml::parse_required_toml_string(source, "lock_schema", path)?;
    if schema != PROJECT_GALAXY_RESOLUTION_LOCK_SCHEMA {
        return Err(format!(
            "Galaxy resolution lock `{}` has unsupported schema `{schema}`",
            path.display()
        ));
    }
    let digest_contract =
        crate::aot_toml::parse_required_toml_string(source, "digest_contract", path)?;
    if digest_contract != PROJECT_GALAXY_RESOLUTION_LOCK_DIGEST {
        return Err(format!(
            "Galaxy resolution lock `{}` has unsupported digest contract `{digest_contract}`",
            path.display()
        ));
    }
    let expected = crate::aot_toml::parse_required_toml_string(source, "resolution_sha256", path)?;
    validate_sha256(&expected, path)?;
    if !crate::aot_toml::parse_required_toml_bool(source, "payload_begin", path)? {
        return Err(format!(
            "Galaxy resolution lock `{}` has disabled payload boundary",
            path.display()
        ));
    }
    let marker = "payload_begin = true\n";
    let payload = source
        .split_once(marker)
        .map(|(_, payload)| payload)
        .ok_or_else(|| {
            format!(
                "Galaxy resolution lock `{}` is missing canonical payload boundary",
                path.display()
            )
        })?;
    let actual = format!(
        "sha256:{}",
        crate::digest_sha256::sha256_hex(payload.as_bytes())
    );
    if actual != expected {
        return Err(format!(
            "Galaxy resolution lock `{}` payload hash mismatch: expected `{expected}`, actual `{actual}`",
            path.display()
        ));
    }

    Ok(ProjectGalaxyResolutionLockSummary {
        schema,
        digest_contract,
        resolution_sha256: actual,
        dependencies: crate::aot_toml::parse_required_toml_usize(
            payload,
            "dependency_count",
            path,
        )?,
        source_modules: crate::aot_toml::parse_required_toml_usize(
            payload,
            "source_module_count",
            path,
        )?,
        library_modules: crate::aot_toml::parse_required_toml_usize(
            payload,
            "library_module_count",
            path,
        )?,
        selected_library_modules: crate::aot_toml::parse_required_toml_usize(
            payload,
            "selected_library_module_count",
            path,
        )?,
    })
}

pub fn verify_project_galaxy_resolution_lock(
    project: &LoadedProject,
    source: &str,
    path: &Path,
) -> Result<ProjectGalaxyResolutionLockSummary, String> {
    let summary = verify_project_galaxy_resolution_lock_source(source, path)?;
    let expected = render_project_galaxy_resolution_lock(project)?;
    if source != expected.source {
        return Err(format!(
            "Galaxy resolution lock `{}` does not reproduce the current project dependency closure: locked={}, resolved={}",
            path.display(),
            summary.resolution_sha256,
            expected.summary.resolution_sha256
        ));
    }
    Ok(summary)
}

pub fn write_project_galaxy_resolution_lock(
    path: &Path,
    project: &LoadedProject,
) -> Result<ProjectGalaxyResolutionLockSummary, String> {
    let rendered = render_project_galaxy_resolution_lock(project)?;
    fs::write(path, &rendered.source).map_err(|error| {
        format!(
            "failed to write project Galaxy resolution lock `{}`: {error}",
            path.display()
        )
    })?;
    Ok(rendered.summary)
}

fn selected_library_modules(project: &LoadedProject) -> BTreeMap<(String, String), &'static str> {
    let mut selected = BTreeMap::new();
    for module in &project.modules {
        match &module.origin {
            ProjectModuleOrigin::AutoInjectedGalaxy {
                galaxy,
                library_module,
                ..
            } => {
                selected.insert((galaxy.clone(), library_module.clone()), "auto-injected");
            }
            ProjectModuleOrigin::ExplicitGalaxyImport {
                galaxy,
                library_module,
                ..
            } => {
                selected.insert((galaxy.clone(), library_module.clone()), "explicit");
            }
            ProjectModuleOrigin::LocalProject { .. } => {}
        }
    }
    selected
}

fn content_records(
    identities: &[crate::stdlib_registry::ResolvedGalaxyContentIdentity],
    selections: Option<&[&str]>,
) -> Result<Vec<String>, String> {
    if selections.is_some_and(|items| items.len() != identities.len()) {
        return Err("Galaxy resolution produced mismatched content selection tables".to_owned());
    }
    let mut records = identities
        .iter()
        .enumerate()
        .map(|(index, identity)| {
            content_record(
                identity,
                selections.and_then(|items| items.get(index).copied()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    records.sort();
    Ok(records)
}

fn content_record(
    identity: &crate::stdlib_registry::ResolvedGalaxyContentIdentity,
    selection: Option<&str>,
) -> Result<String, String> {
    validate_lock_relative_path(&identity.logical_path)?;
    validate_sha256_identity(&identity.sha256)?;
    let mut record = format!(
        "{}|bytes={}|{}",
        identity.logical_path, identity.bytes, identity.sha256
    );
    if let Some(selection) = selection {
        record.push_str("|selection=");
        record.push_str(selection);
    }
    Ok(record)
}

fn validate_sha256_identity(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(format!("Galaxy content identity `{value}` is malformed"));
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("Galaxy content identity `{value}` is malformed"));
    }
    Ok(())
}

fn validate_lock_relative_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('|')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(format!(
            "Galaxy resolution lock path `{path}` is not a canonical portable relative path"
        ));
    }
    Ok(())
}

fn validate_lock_token(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(format!("{label} `{value}` is not lock-safe"));
    }
    Ok(())
}

fn validate_sha256(value: &str, path: &Path) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(format!(
            "Galaxy resolution lock `{}` has malformed resolution_sha256 `{value}`",
            path.display()
        ));
    };
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "Galaxy resolution lock `{}` has malformed resolution_sha256 `{value}`",
            path.display()
        ));
    }
    Ok(())
}
