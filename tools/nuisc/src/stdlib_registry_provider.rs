use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Component, Path};

use super::{
    detect_auto_injectability, load_stdlib_layout, load_stdlib_module_manifest_with_identity,
    read_content_identities, GalaxyResolutionProviderDescriptor, GalaxyResolutionProviderReport,
    GalaxyResolutionProviderRequest, GalaxyResolutionProviderRequirement,
    GalaxyResolutionProviderResolution, GalaxyResolutionProviderSelection,
    ResolvedGalaxyDependency, StdlibIndexModule,
};

pub const GALAXY_RESOLUTION_PROVIDER_CONTRACT: &str = "nuis-galaxy-resolution-provider-v1";
pub const GALAXY_RESOLUTION_PROVIDER_KINDS: &[&str] = &[
    "workspace-layout",
    "locked-resolution-cache",
    "offline-layout",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingRequirement {
    name: String,
    exact_version: Option<String>,
    direct: bool,
    requested_by: String,
}

pub fn resolve_galaxy_dependencies_with_provider(
    provider: &GalaxyResolutionProviderDescriptor,
    requested: &[crate::project::ProjectGalaxyDependency],
) -> Result<GalaxyResolutionProviderResolution, String> {
    validate_provider(provider)?;
    let requirements = normalize_requirements(requested)?;
    let layout = load_stdlib_layout(&provider.root)?;
    if layout.schema != "nuis-stdlib-layout-v1" {
        return Err(format!(
            "Galaxy resolution provider `{}` has unsupported layout schema `{}`",
            provider.provider_id, layout.schema
        ));
    }
    let candidates = collect_candidates(provider, layout.modules)?;
    let mut pending = requirements
        .iter()
        .map(|requirement| PendingRequirement {
            name: requirement.name.clone(),
            exact_version: Some(requirement.exact_version.clone()),
            direct: true,
            requested_by: requirement.name.clone(),
        })
        .collect::<Vec<_>>();
    let mut dependencies = BTreeMap::<String, ResolvedGalaxyDependency>::new();
    let mut selected_paths = BTreeMap::<String, String>::new();

    while !pending.is_empty() {
        pending.sort_by(|lhs, rhs| {
            lhs.name
                .cmp(&rhs.name)
                .then(lhs.exact_version.cmp(&rhs.exact_version))
                .then(rhs.direct.cmp(&lhs.direct))
                .then(lhs.requested_by.cmp(&rhs.requested_by))
        });
        let requirement = pending.remove(0);
        if let Some(existing) = dependencies.get_mut(&requirement.name) {
            if requirement
                .exact_version
                .as_ref()
                .is_some_and(|version| version != &existing.version)
            {
                return Err(format!(
                    "Galaxy provider `{}` selected `{}` at version `{}`, but `{}` requires exact version `{}`",
                    provider.provider_id,
                    requirement.name,
                    existing.version,
                    requirement.requested_by,
                    requirement.exact_version.as_deref().unwrap_or("<none>")
                ));
            }
            existing.direct |= requirement.direct;
            insert_sorted_unique(&mut existing.requested_by, requirement.requested_by);
            continue;
        }

        let candidate = select_candidate(provider, &candidates, &requirement)?;
        let module_dir = provider.root.join(&candidate.path);
        let manifest_path = module_dir.join("module.toml");
        verify_provider_paths(
            &provider.root,
            [module_dir.as_path(), manifest_path.as_path()].into_iter(),
        )?;
        let (manifest, manifest_content_identity) =
            load_stdlib_module_manifest_with_identity(&provider.root, &candidate.path)?;
        validate_candidate_manifest(candidate, &manifest)?;
        let dependency_requirements =
            validate_dependency_contract(candidate, &manifest.depends_on)?;
        for logical_path in manifest
            .source_modules
            .iter()
            .chain(&manifest.library_modules)
        {
            validate_relative_path(logical_path)?;
        }
        let resolved_source_paths = manifest
            .source_modules
            .iter()
            .map(|item| module_dir.join(item))
            .collect::<Vec<_>>();
        let resolved_library_paths = manifest
            .library_modules
            .iter()
            .map(|item| module_dir.join(item))
            .collect::<Vec<_>>();
        let provider_paths = resolved_source_paths
            .iter()
            .chain(&resolved_library_paths)
            .map(|path| path.as_path());
        verify_provider_paths(&provider.root, provider_paths)?;
        let source_content_identities =
            read_content_identities(&manifest.source_modules, &resolved_source_paths)?;
        let library_content_identities =
            read_content_identities(&manifest.library_modules, &resolved_library_paths)?;
        let (auto_injectable, auto_inject_blockers) = detect_auto_injectability(
            &resolved_library_paths,
            &library_content_identities,
            &manifest.library_import_policy,
        )?;
        let mut requested_by = vec![requirement.requested_by];
        requested_by.sort();
        requested_by.dedup();
        let dependency = ResolvedGalaxyDependency {
            name: candidate.name.clone(),
            version: candidate.version.clone(),
            package_id: manifest.package_id.clone(),
            direct: requirement.direct,
            requested_by,
            module_dir: module_dir.clone(),
            manifest_path,
            manifest_content_identity,
            depends_on: manifest.depends_on,
            surfaces: manifest.surfaces,
            code_assets: manifest.code_assets,
            source_modules: manifest.source_modules,
            resolved_source_paths,
            source_content_identities,
            library_modules: manifest.library_modules,
            resolved_library_paths,
            library_content_identities,
            library_import_policy: manifest.library_import_policy,
            auto_injectable,
            auto_inject_blockers,
        };
        selected_paths.insert(candidate.name.clone(), candidate.path.clone());
        dependencies.insert(candidate.name.clone(), dependency);
        pending.extend(
            dependency_requirements
                .into_iter()
                .map(|(name, exact_version)| PendingRequirement {
                    name,
                    exact_version,
                    direct: false,
                    requested_by: candidate.name.clone(),
                }),
        );
    }

    let dependencies = dependencies.into_values().collect::<Vec<_>>();
    let selections = dependencies
        .iter()
        .map(|dependency| GalaxyResolutionProviderSelection {
            name: dependency.name.clone(),
            version: dependency.version.clone(),
            package_id: dependency.package_id.clone(),
            relative_path: selected_paths[&dependency.name].clone(),
            direct: dependency.direct,
            requested_by: dependency.requested_by.clone(),
        })
        .collect::<Vec<_>>();
    let request_sha256 = hash_request(provider, &requirements);
    let selection_sha256 = hash_selection(&dependencies, &selected_paths);
    let request = GalaxyResolutionProviderRequest {
        contract: GALAXY_RESOLUTION_PROVIDER_CONTRACT.to_owned(),
        provider_id: provider.provider_id.clone(),
        provider_kind: provider.provider_kind.clone(),
        request_sha256: request_sha256.clone(),
        requirements: requirements.clone(),
    };
    Ok(GalaxyResolutionProviderResolution {
        request,
        report: GalaxyResolutionProviderReport {
            contract: GALAXY_RESOLUTION_PROVIDER_CONTRACT.to_owned(),
            status: "resolved-pinned-provider-closure".to_owned(),
            provider_id: provider.provider_id.clone(),
            provider_kind: provider.provider_kind.clone(),
            request_sha256,
            selection_sha256,
            candidate_count: candidates.len(),
            selected_count: selections.len(),
            requirements,
            selections,
        },
        dependencies,
    })
}

fn validate_provider(provider: &GalaxyResolutionProviderDescriptor) -> Result<(), String> {
    validate_token("provider id", &provider.provider_id)?;
    if !GALAXY_RESOLUTION_PROVIDER_KINDS.contains(&provider.provider_kind.as_str()) {
        return Err(format!(
            "Galaxy resolution provider `{}` has unregistered kind `{}`; registered kinds=[{}]",
            provider.provider_id,
            provider.provider_kind,
            GALAXY_RESOLUTION_PROVIDER_KINDS.join(", ")
        ));
    }
    if !provider.root.join("index.toml").is_file() {
        return Err(format!(
            "Galaxy resolution provider `{}` is missing `{}`",
            provider.provider_id,
            provider.root.join("index.toml").display()
        ));
    }
    Ok(())
}

fn normalize_requirements(
    requested: &[crate::project::ProjectGalaxyDependency],
) -> Result<Vec<GalaxyResolutionProviderRequirement>, String> {
    let mut normalized = BTreeMap::new();
    for item in requested {
        validate_token("Galaxy dependency name", &item.name)?;
        validate_exact_version(&item.version)?;
        if let Some(previous) = normalized.insert(item.name.clone(), item.version.clone()) {
            return Err(format!(
                "duplicate Galaxy dependency `{}` requests `{previous}` and `{}`",
                item.name, item.version
            ));
        }
    }
    Ok(normalized
        .into_iter()
        .map(
            |(name, exact_version)| GalaxyResolutionProviderRequirement {
                name,
                exact_version,
            },
        )
        .collect())
}

fn collect_candidates(
    provider: &GalaxyResolutionProviderDescriptor,
    modules: Vec<StdlibIndexModule>,
) -> Result<BTreeMap<(String, String), StdlibIndexModule>, String> {
    let mut candidates = BTreeMap::new();
    for candidate in modules {
        validate_token("Galaxy candidate name", &candidate.name)?;
        validate_exact_version(&candidate.version)?;
        if let Err(error) = validate_relative_path(&candidate.path) {
            if provider.provider_kind == "locked-resolution-cache" {
                return Err(format!(
                    "locked Galaxy candidate path `{}` escapes resolution cache `{}`: {error}",
                    candidate.path,
                    provider.root.display()
                ));
            }
            return Err(error);
        }
        let candidate_path = provider.root.join(&candidate.path);
        verify_provider_paths(&provider.root, std::iter::once(candidate_path.as_path()))?;
        let key = (candidate.name.clone(), candidate.version.clone());
        if candidates.insert(key.clone(), candidate).is_some() {
            return Err(format!(
                "Galaxy provider index declares duplicate candidate `{}={}`",
                key.0, key.1
            ));
        }
    }
    Ok(candidates)
}

fn select_candidate<'a>(
    provider: &GalaxyResolutionProviderDescriptor,
    candidates: &'a BTreeMap<(String, String), StdlibIndexModule>,
    requirement: &PendingRequirement,
) -> Result<&'a StdlibIndexModule, String> {
    if let Some(version) = &requirement.exact_version {
        return candidates
            .get(&(requirement.name.clone(), version.clone()))
            .ok_or_else(|| {
                format!(
                    "Galaxy provider `{}` has no exact candidate `{}={}`",
                    provider.provider_id, requirement.name, version
                )
            });
    }
    let available = candidates
        .iter()
        .filter(|((name, _), _)| name == &requirement.name)
        .map(|((_, version), candidate)| (version, candidate))
        .collect::<Vec<_>>();
    match available.as_slice() {
        [] => Err(format!(
            "Galaxy provider `{}` has no candidate for transitive dependency `{}` requested by `{}`",
            provider.provider_id, requirement.name, requirement.requested_by
        )),
        [(_, candidate)] => Ok(candidate),
        _ => Err(format!(
            "Galaxy provider `{}` has ambiguous unpinned transitive dependency `{}` requested by `{}`; candidates=[{}]",
            provider.provider_id,
            requirement.name,
            requirement.requested_by,
            available
                .iter()
                .map(|(version, _)| version.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn validate_candidate_manifest(
    candidate: &StdlibIndexModule,
    manifest: &super::StdlibModuleManifest,
) -> Result<(), String> {
    if candidate.name != manifest.name || candidate.package_id != manifest.package_id {
        return Err(format!(
            "Galaxy candidate `{}={}` identity drift: index name/package={}/{}, manifest name/package={}/{}",
            candidate.name,
            candidate.version,
            candidate.name,
            candidate.package_id,
            manifest.name,
            manifest.package_id
        ));
    }
    Ok(())
}

fn validate_dependency_contract(
    candidate: &StdlibIndexModule,
    manifest_dependencies: &[String],
) -> Result<Vec<(String, Option<String>)>, String> {
    let mut requirements = Vec::new();
    for raw in &candidate.depends_on {
        requirements.push(parse_dependency_requirement(raw)?);
    }
    let indexed_names = requirements
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    let manifest_names = manifest_dependencies
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if indexed_names != manifest_names
        || indexed_names.len() != requirements.len()
        || manifest_names.len() != manifest_dependencies.len()
    {
        return Err(format!(
            "Galaxy candidate `{}={}` dependency contract differs between provider index and module manifest",
            candidate.name, candidate.version
        ));
    }
    requirements.sort();
    Ok(requirements)
}

fn parse_dependency_requirement(raw: &str) -> Result<(String, Option<String>), String> {
    let (name, version) = raw
        .split_once('=')
        .map(|(name, version)| (name, Some(version)))
        .unwrap_or((raw, None));
    validate_token("transitive Galaxy dependency name", name)?;
    let version = match version {
        Some(version) => {
            validate_exact_version(version)?;
            Some(version.to_owned())
        }
        None => None,
    };
    Ok((name.to_owned(), version))
}

fn verify_provider_paths<'a>(
    root: &Path,
    paths: impl Iterator<Item = &'a Path>,
) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|error| {
        format!(
            "failed to canonicalize Galaxy provider root `{}`: {error}",
            root.display()
        )
    })?;
    for path in paths {
        let canonical = path.canonicalize().map_err(|error| {
            format!(
                "failed to canonicalize Galaxy provider member `{}`: {error}",
                path.display()
            )
        })?;
        if !canonical.starts_with(&canonical_root) {
            return Err(format!(
                "Galaxy provider member `{}` escapes provider root `{}`",
                path.display(),
                root.display()
            ));
        }
    }
    Ok(())
}

fn validate_relative_path(raw: &str) -> Result<(), String> {
    if raw.is_empty()
        || raw.contains('\\')
        || Path::new(raw)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "Galaxy provider candidate path `{raw}` must be a normalized relative path"
        ));
    }
    Ok(())
}

fn validate_exact_version(version: &str) -> Result<(), String> {
    if version.starts_with(['^', '~', '>', '<', '=']) || version.contains('*') {
        return Err(format!(
            "Galaxy version requirement `{version}` is not exact; range solving is not registered yet"
        ));
    }
    validate_token("exact Galaxy version", version)?;
    Ok(())
}

fn validate_token(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err(format!(
            "{label} `{value}` must contain only ASCII letters, digits, `.`, `-`, or `_`"
        ));
    }
    Ok(())
}

fn hash_request(
    provider: &GalaxyResolutionProviderDescriptor,
    requirements: &[GalaxyResolutionProviderRequirement],
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, GALAXY_RESOLUTION_PROVIDER_CONTRACT);
    append_text(&mut canonical, &provider.provider_id);
    append_text(&mut canonical, &provider.provider_kind);
    for requirement in requirements {
        append_text(&mut canonical, &requirement.name);
        append_text(&mut canonical, &requirement.exact_version);
    }
    format!(
        "sha256:{}",
        crate::digest_sha256::sha256_hex(canonical.as_bytes())
    )
}

fn hash_selection(
    dependencies: &[ResolvedGalaxyDependency],
    selected_paths: &BTreeMap<String, String>,
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, GALAXY_RESOLUTION_PROVIDER_CONTRACT);
    for dependency in dependencies {
        append_text(&mut canonical, &dependency.name);
        append_text(&mut canonical, &dependency.version);
        append_text(&mut canonical, &dependency.package_id);
        append_text(&mut canonical, &selected_paths[&dependency.name]);
        writeln!(canonical, "direct={}", dependency.direct).unwrap();
        for requested_by in &dependency.requested_by {
            append_text(&mut canonical, requested_by);
        }
        for depends_on in &dependency.depends_on {
            append_text(&mut canonical, depends_on);
        }
        append_text(&mut canonical, &dependency.manifest_content_identity.sha256);
        for identity in dependency
            .source_content_identities
            .iter()
            .chain(&dependency.library_content_identities)
        {
            append_text(&mut canonical, &identity.logical_path);
            append_text(&mut canonical, &identity.sha256);
            writeln!(canonical, "bytes={}", identity.bytes).unwrap();
        }
    }
    format!(
        "sha256:{}",
        crate::digest_sha256::sha256_hex(canonical.as_bytes())
    )
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn insert_sorted_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
        values.sort();
    }
}

#[cfg(test)]
#[path = "stdlib_registry_provider_tests.rs"]
mod tests;
