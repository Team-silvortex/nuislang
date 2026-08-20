use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path};

use super::stdlib_registry_provider_semver::{parse_requirement, validate_candidate_version};
use super::stdlib_registry_provider_solver::{parse_dependency_requirement, solve_candidates};
use super::stdlib_registry_provider_trust::{verify_candidate_set, GALAXY_CANDIDATE_SET_FILE};
use super::{
    detect_auto_injectability, load_stdlib_module_manifest_with_identity,
    parse_stdlib_layout_source, read_content_identities, GalaxyResolutionCandidateSetReport,
    GalaxyResolutionProviderDescriptor, GalaxyResolutionProviderReport,
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
const MAX_PROVIDER_INDEX_BYTES: u64 = 16 * 1024 * 1024;

pub fn resolve_galaxy_dependencies_with_provider(
    provider: &GalaxyResolutionProviderDescriptor,
    requested: &[crate::project::ProjectGalaxyDependency],
) -> Result<GalaxyResolutionProviderResolution, String> {
    validate_provider(provider)?;
    let requirements = normalize_requirements(requested)?;
    let index_path = provider.root.join("index.toml");
    let index_bytes = fs::read(&index_path).map_err(|error| {
        format!(
            "failed to read Galaxy provider index `{}`: {error}",
            index_path.display()
        )
    })?;
    let index_source = std::str::from_utf8(&index_bytes).map_err(|error| {
        format!(
            "Galaxy provider index `{}` is not UTF-8: {error}",
            index_path.display()
        )
    })?;
    let layout = parse_stdlib_layout_source(index_source, &index_path)?;
    if layout.schema != "nuis-stdlib-layout-v1" {
        return Err(format!(
            "Galaxy resolution provider `{}` has unsupported layout schema `{}`",
            provider.provider_id, layout.schema
        ));
    }
    let candidates = collect_candidates(provider, layout.modules)?;
    let candidate_set = verify_candidate_set(provider, &index_bytes, &candidates)?;
    let allow_ranges = candidate_set.status == "verified-signed-candidate-set";
    let solved = solve_candidates(
        &provider.provider_id,
        &candidates,
        &requirements,
        allow_ranges,
    )?;
    let mut dependencies = Vec::<ResolvedGalaxyDependency>::new();
    let mut selected_paths = BTreeMap::<String, String>::new();

    for solved_candidate in solved {
        let candidate = &solved_candidate.candidate;
        let module_dir = provider.root.join(&candidate.path);
        let manifest_path = module_dir.join("module.toml");
        verify_provider_paths(
            &provider.root,
            [module_dir.as_path(), manifest_path.as_path()].into_iter(),
        )?;
        let (manifest, manifest_content_identity) =
            load_stdlib_module_manifest_with_identity(&provider.root, &candidate.path)?;
        validate_candidate_manifest(candidate, &manifest)?;
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
        let dependency = ResolvedGalaxyDependency {
            name: candidate.name.clone(),
            version: candidate.version.clone(),
            package_id: manifest.package_id.clone(),
            direct: solved_candidate.direct,
            requested_by: solved_candidate.requested_by,
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
        dependencies.push(dependency);
    }

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
    let request_sha256 = hash_request(provider, &candidate_set, &requirements);
    let selection_sha256 = hash_selection(&dependencies, &selected_paths);
    let request = GalaxyResolutionProviderRequest {
        contract: GALAXY_RESOLUTION_PROVIDER_CONTRACT.to_owned(),
        provider_id: provider.provider_id.clone(),
        provider_kind: provider.provider_kind.clone(),
        request_sha256: request_sha256.clone(),
        candidate_set: candidate_set.clone(),
        requirements: requirements.clone(),
    };
    Ok(GalaxyResolutionProviderResolution {
        request,
        report: GalaxyResolutionProviderReport {
            contract: GALAXY_RESOLUTION_PROVIDER_CONTRACT.to_owned(),
            status: if allow_ranges {
                "resolved-signed-provider-closure".to_owned()
            } else {
                "resolved-pinned-provider-closure".to_owned()
            },
            provider_id: provider.provider_id.clone(),
            provider_kind: provider.provider_kind.clone(),
            request_sha256,
            selection_sha256,
            candidate_count: candidates.len(),
            selected_count: selections.len(),
            candidate_set,
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
    let index_path = provider.root.join("index.toml");
    if !index_path.is_file() {
        return Err(format!(
            "Galaxy resolution provider `{}` is missing `{}`",
            provider.provider_id,
            index_path.display()
        ));
    }
    let index_bytes = fs::metadata(&index_path)
        .map_err(|error| {
            format!(
                "failed to inspect Galaxy provider index `{}`: {error}",
                index_path.display()
            )
        })?
        .len();
    if index_bytes > MAX_PROVIDER_INDEX_BYTES {
        return Err(format!(
            "Galaxy provider index `{}` exceeds the {MAX_PROVIDER_INDEX_BYTES}-byte resource limit",
            index_path.display()
        ));
    }
    let sidecar_path = provider.root.join(GALAXY_CANDIDATE_SET_FILE);
    let mut confined_paths = vec![index_path.as_path()];
    if sidecar_path.exists() {
        confined_paths.push(sidecar_path.as_path());
    }
    verify_provider_paths(&provider.root, confined_paths.into_iter())?;
    Ok(())
}

fn normalize_requirements(
    requested: &[crate::project::ProjectGalaxyDependency],
) -> Result<Vec<GalaxyResolutionProviderRequirement>, String> {
    let mut normalized = BTreeMap::new();
    for item in requested {
        validate_token("Galaxy dependency name", &item.name)?;
        let requirement = parse_requirement(&item.version)?;
        let requirement = requirement.canonical().to_owned();
        if let Some(previous) = normalized.insert(item.name.clone(), requirement.clone()) {
            return Err(format!(
                "duplicate Galaxy dependency `{}` requests `{previous}` and `{requirement}`",
                item.name
            ));
        }
    }
    Ok(normalized
        .into_iter()
        .map(
            |(name, version_requirement)| GalaxyResolutionProviderRequirement {
                name,
                version_requirement,
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
        validate_candidate_version(&candidate.version)?;
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
) -> Result<(), String> {
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
    Ok(())
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
    candidate_set: &GalaxyResolutionCandidateSetReport,
    requirements: &[GalaxyResolutionProviderRequirement],
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, GALAXY_RESOLUTION_PROVIDER_CONTRACT);
    append_text(&mut canonical, &provider.provider_id);
    append_text(&mut canonical, &provider.provider_kind);
    append_text(&mut canonical, &candidate_set.response_sha256);
    writeln!(canonical, "generation={}", candidate_set.generation).unwrap();
    writeln!(
        canonical,
        "signature_count={}",
        candidate_set.signature_count
    )
    .unwrap();
    for signer_id in &candidate_set.signer_ids {
        append_text(&mut canonical, signer_id);
    }
    for requirement in requirements {
        append_text(&mut canonical, &requirement.name);
        append_text(&mut canonical, &requirement.version_requirement);
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

#[cfg(test)]
#[path = "stdlib_registry_provider_tests.rs"]
mod tests;
