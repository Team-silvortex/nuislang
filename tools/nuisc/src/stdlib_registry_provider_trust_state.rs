use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::{
    GalaxyResolutionProviderDescriptor, GalaxyResolutionProviderTrustPolicy,
    GalaxyResolutionProviderTrustReport,
};

pub const GALAXY_PROVIDER_TRUST_REGISTRY_CONTRACT: &str = "nuis-galaxy-provider-trust-registry-v1";
pub const GALAXY_PROVIDER_TRUST_STATE_CONTRACT: &str = "nuis-galaxy-provider-trust-state-v1";

const MAX_TRUST_DOCUMENT_BYTES: u64 = 1024 * 1024;
const MAX_TRUST_SIGNERS: usize = 64;
type ParsedTrustDocument = (BTreeMap<String, String>, Vec<BTreeMap<String, String>>);

#[path = "stdlib_registry_provider_trust_state_io.rs"]
mod trust_state_io;

use trust_state_io::{persist_state, validate_state_target, TrustStateLock};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustSigner {
    signer_id: String,
    status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustRegistry {
    provider_id: String,
    provider_kind: String,
    generation: u64,
    signers: Vec<TrustSigner>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustState {
    provider_id: String,
    provider_kind: String,
    registry_generation: u64,
    registry_sha256: String,
    highest_candidate_generation: u64,
    candidate_response_sha256: String,
    state_sha256: String,
    signers: Vec<TrustSigner>,
}

pub(super) fn enforce_candidate_set_trust(
    provider: &GalaxyResolutionProviderDescriptor,
    candidate_generation: u64,
    candidate_response_sha256: &str,
    candidate_signer_ids: &[String],
) -> Result<Option<GalaxyResolutionProviderTrustReport>, String> {
    let Some(policy) = provider.trust_policy.as_ref() else {
        return Ok(None);
    };
    validate_policy_paths(&provider.root, policy)?;
    validate_sha256(candidate_response_sha256, "candidate response")?;
    if candidate_generation == 0 || candidate_signer_ids.is_empty() {
        return Err(
            "trusted Galaxy candidate set requires a nonzero generation and signer set".to_owned(),
        );
    }

    let (registry, registry_source) = read_registry(&policy.registry_path)?;
    validate_registry_for_provider(&registry, provider)?;
    authorize_candidate_signers(&registry, candidate_signer_ids)?;
    let registry_sha256 = sha256(registry_source.as_bytes());

    let _lock = TrustStateLock::acquire(&policy.state_path)?;
    validate_state_target(&policy.state_path)?;
    let previous = read_state(&policy.state_path)?;
    transition_status(
        previous.as_ref(),
        &registry,
        &registry_sha256,
        candidate_generation,
        candidate_response_sha256,
        provider,
    )?;
    let mut next = TrustState {
        provider_id: provider.provider_id.clone(),
        provider_kind: provider.provider_kind.clone(),
        registry_generation: registry.generation,
        registry_sha256,
        highest_candidate_generation: candidate_generation,
        candidate_response_sha256: candidate_response_sha256.to_owned(),
        state_sha256: String::new(),
        signers: registry.signers.clone(),
    };
    next.state_sha256 = state_identity(&next);
    let next_source = render_state(&next);
    let unchanged = previous
        .as_ref()
        .is_some_and(|state| render_state(state) == next_source);
    if !unchanged {
        persist_state(&policy.state_path, next_source.as_bytes())?;
        let reread = read_state(&policy.state_path)?.ok_or_else(|| {
            "Galaxy provider trust state disappeared after atomic persistence".to_owned()
        })?;
        if reread != next {
            return Err("Galaxy provider trust state changed after atomic persistence".to_owned());
        }
    }
    Ok(Some(report(&next)))
}

fn validate_policy_paths(
    provider_root: &Path,
    policy: &GalaxyResolutionProviderTrustPolicy,
) -> Result<(), String> {
    if policy.registry_path == policy.state_path {
        return Err("Galaxy provider trust registry and state paths must be distinct".to_owned());
    }
    let provider_root = fs::canonicalize(provider_root).map_err(|error| {
        format!(
            "failed to canonicalize Galaxy provider root `{}`: {error}",
            provider_root.display()
        )
    })?;
    let registry = canonical_regular_file(&policy.registry_path, "trust registry")?;
    canonical_parent(&policy.registry_path, "trust registry")?;
    let state_parent = canonical_parent(&policy.state_path, "trust state")?;
    let state_target = state_parent.join(
        policy
            .state_path
            .file_name()
            .ok_or_else(|| "Galaxy provider trust state path must name a file".to_owned())?,
    );
    if registry == state_target {
        return Err(
            "Galaxy provider trust registry and state paths resolve to one file".to_owned(),
        );
    }
    if registry.starts_with(&provider_root) || state_parent.starts_with(&provider_root) {
        return Err(
            "Galaxy provider trust registry and state must live outside the provider root"
                .to_owned(),
        );
    }
    validate_state_target(&policy.state_path)
}

fn canonical_regular_file(path: &Path, label: &str) -> Result<PathBuf, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "failed to inspect Galaxy provider {label} `{}`: {error}",
            path.display()
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "Galaxy provider {label} `{}` must be a regular non-symlink file",
            path.display()
        ));
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(format!(
            "Galaxy provider {label} `{}` must not be group or other writable",
            path.display()
        ));
    }
    fs::canonicalize(path).map_err(|error| {
        format!(
            "failed to canonicalize Galaxy provider {label} `{}`: {error}",
            path.display()
        )
    })
}

fn canonical_parent(path: &Path, label: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let metadata = fs::symlink_metadata(parent).map_err(|error| {
        format!(
            "failed to inspect Galaxy provider {label} parent `{}`: {error}",
            parent.display()
        )
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!(
            "Galaxy provider {label} parent `{}` must be a regular non-symlink directory",
            parent.display()
        ));
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o022 != 0 {
        return Err(format!(
            "Galaxy provider {label} parent `{}` must not be group or other writable",
            parent.display()
        ));
    }
    fs::canonicalize(parent).map_err(|error| {
        format!(
            "failed to canonicalize Galaxy provider {label} parent `{}`: {error}",
            parent.display()
        )
    })
}

fn read_registry(path: &Path) -> Result<(TrustRegistry, String), String> {
    let source = read_bounded_text(path, "trust registry")?;
    let (mut root, signer_fields) = parse_document(&source, "trust registry")?;
    let mut signers = parse_signers(signer_fields, "trust registry")?;
    signers.sort_by(|left, right| left.signer_id.cmp(&right.signer_id));
    let registry = TrustRegistry {
        provider_id: take_required(&mut root, "provider_id", "trust registry")?,
        provider_kind: take_required(&mut root, "provider_kind", "trust registry")?,
        generation: parse_integer(&mut root, "generation", "trust registry")?,
        signers,
    };
    if take_required(&mut root, "trust_registry_contract", "trust registry")?
        != GALAXY_PROVIDER_TRUST_REGISTRY_CONTRACT
    {
        return Err("Galaxy provider trust registry contract is unsupported".to_owned());
    }
    reject_unknown(root, "trust registry")?;
    validate_registry(&registry)?;
    if render_registry(&registry) != source {
        return Err("Galaxy provider trust registry is not canonically encoded".to_owned());
    }
    Ok((registry, source))
}

fn validate_registry(registry: &TrustRegistry) -> Result<(), String> {
    validate_token("provider id", &registry.provider_id)?;
    validate_token("provider kind", &registry.provider_kind)?;
    if registry.generation == 0 {
        return Err("Galaxy provider trust registry generation must be nonzero".to_owned());
    }
    if registry.signers.is_empty() {
        return Err("Galaxy provider trust registry must contain at least one signer".to_owned());
    }
    let mut seen = BTreeSet::new();
    let mut active = 0;
    for signer in &registry.signers {
        validate_signer_id(&signer.signer_id)?;
        if !seen.insert(&signer.signer_id) {
            return Err(format!(
                "Galaxy provider trust registry repeats signer `{}`",
                signer.signer_id
            ));
        }
        match signer.status.as_str() {
            "active" => active += 1,
            "revoked" => {}
            other => {
                return Err(format!(
                    "Galaxy provider trust registry signer `{}` has unsupported status `{other}`",
                    signer.signer_id
                ));
            }
        }
    }
    if active == 0 {
        return Err(
            "Galaxy provider trust registry must retain at least one active signer".to_owned(),
        );
    }
    Ok(())
}

fn validate_registry_for_provider(
    registry: &TrustRegistry,
    provider: &GalaxyResolutionProviderDescriptor,
) -> Result<(), String> {
    if registry.provider_id != provider.provider_id
        || registry.provider_kind != provider.provider_kind
    {
        return Err(format!(
            "Galaxy provider trust registry identity drift: expected `{}/{}`, found `{}/{}`",
            provider.provider_id,
            provider.provider_kind,
            registry.provider_id,
            registry.provider_kind
        ));
    }
    Ok(())
}

fn authorize_candidate_signers(
    registry: &TrustRegistry,
    candidate_signer_ids: &[String],
) -> Result<(), String> {
    let status_by_id = registry
        .signers
        .iter()
        .map(|signer| (signer.signer_id.as_str(), signer.status.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for signer_id in candidate_signer_ids {
        if !seen.insert(signer_id) {
            return Err(format!(
                "Galaxy candidate trust check repeats signer `{signer_id}`"
            ));
        }
        match status_by_id.get(signer_id.as_str()).copied() {
            Some("active") => {}
            Some("revoked") => {
                return Err(format!(
                    "Galaxy candidate-set signer `{signer_id}` is revoked by the trust registry"
                ));
            }
            _ => {
                return Err(format!(
                    "Galaxy candidate-set signer `{signer_id}` is not authorized by the trust registry"
                ));
            }
        }
    }
    Ok(())
}

fn transition_status(
    previous: Option<&TrustState>,
    registry: &TrustRegistry,
    registry_sha256: &str,
    candidate_generation: u64,
    candidate_response_sha256: &str,
    provider: &GalaxyResolutionProviderDescriptor,
) -> Result<&'static str, String> {
    let Some(previous) = previous else {
        return Ok("initialized");
    };
    if previous.provider_id != provider.provider_id
        || previous.provider_kind != provider.provider_kind
    {
        return Err("Galaxy provider trust state belongs to a different provider".to_owned());
    }
    if registry.generation < previous.registry_generation {
        return Err(format!(
            "Galaxy provider trust registry rollback: highest generation is {}, received {}",
            previous.registry_generation, registry.generation
        ));
    }
    if registry.generation == previous.registry_generation
        && registry_sha256 != previous.registry_sha256
    {
        return Err("Galaxy provider trust registry same-generation fork detected".to_owned());
    }
    if candidate_generation < previous.highest_candidate_generation {
        return Err(format!(
            "Galaxy candidate-set rollback: highest generation is {}, received {candidate_generation}",
            previous.highest_candidate_generation
        ));
    }
    if candidate_generation == previous.highest_candidate_generation
        && candidate_response_sha256 != previous.candidate_response_sha256
    {
        return Err("Galaxy candidate-set same-generation fork detected".to_owned());
    }
    if candidate_generation > previous.highest_candidate_generation {
        Ok("candidate-generation-advanced")
    } else if registry.generation > previous.registry_generation {
        Ok("registry-generation-advanced")
    } else {
        Ok("unchanged")
    }
}

fn read_state(path: &Path) -> Result<Option<TrustState>, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to inspect Galaxy provider trust state `{}`: {error}",
                path.display()
            ));
        }
    }
    let source = read_bounded_text(path, "trust state")?;
    let (mut root, signer_fields) = parse_document(&source, "trust state")?;
    let mut signers = parse_signers(signer_fields, "trust state")?;
    signers.sort_by(|left, right| left.signer_id.cmp(&right.signer_id));
    let contract = take_required(&mut root, "trust_state_contract", "trust state")?;
    if contract != GALAXY_PROVIDER_TRUST_STATE_CONTRACT {
        return Err("Galaxy provider trust state contract is unsupported".to_owned());
    }
    let active_signer_count = parse_integer(&mut root, "active_signer_count", "trust state")?;
    let revoked_signer_count = parse_integer(&mut root, "revoked_signer_count", "trust state")?;
    let state = TrustState {
        provider_id: take_required(&mut root, "provider_id", "trust state")?,
        provider_kind: take_required(&mut root, "provider_kind", "trust state")?,
        registry_generation: parse_integer(&mut root, "registry_generation", "trust state")?,
        registry_sha256: take_required(&mut root, "registry_sha256", "trust state")?,
        highest_candidate_generation: parse_integer(
            &mut root,
            "highest_candidate_generation",
            "trust state",
        )?,
        candidate_response_sha256: take_required(
            &mut root,
            "candidate_response_sha256",
            "trust state",
        )?,
        state_sha256: take_required(&mut root, "state_sha256", "trust state")?,
        signers,
    };
    reject_unknown(root, "trust state")?;
    validate_state(&state, active_signer_count, revoked_signer_count)?;
    if render_state(&state) != source {
        return Err("Galaxy provider trust state is not canonically encoded".to_owned());
    }
    Ok(Some(state))
}

fn validate_state(
    state: &TrustState,
    active_signer_count: usize,
    revoked_signer_count: usize,
) -> Result<(), String> {
    validate_token("provider id", &state.provider_id)?;
    validate_token("provider kind", &state.provider_kind)?;
    validate_sha256(&state.registry_sha256, "trust state registry")?;
    validate_sha256(
        &state.candidate_response_sha256,
        "trust state candidate response",
    )?;
    validate_sha256(&state.state_sha256, "trust state identity")?;
    if state.registry_generation == 0 || state.highest_candidate_generation == 0 {
        return Err("Galaxy provider trust state generations must be nonzero".to_owned());
    }
    let active = state
        .signers
        .iter()
        .filter(|item| item.status == "active")
        .count();
    let revoked = state
        .signers
        .iter()
        .filter(|item| item.status == "revoked")
        .count();
    let registry = TrustRegistry {
        provider_id: state.provider_id.clone(),
        provider_kind: state.provider_kind.clone(),
        generation: state.registry_generation,
        signers: state.signers.clone(),
    };
    validate_registry(&registry)?;
    if active != active_signer_count || revoked != revoked_signer_count {
        return Err("Galaxy provider trust state signer counts drifted".to_owned());
    }
    if state.state_sha256 != state_identity(state) {
        return Err("Galaxy provider trust state identity drifted".to_owned());
    }
    Ok(())
}

fn report(state: &TrustState) -> GalaxyResolutionProviderTrustReport {
    GalaxyResolutionProviderTrustReport {
        contract: GALAXY_PROVIDER_TRUST_STATE_CONTRACT.to_owned(),
        status: "verified-persistent-trust".to_owned(),
        registry_generation: state.registry_generation,
        registry_sha256: state.registry_sha256.clone(),
        highest_candidate_generation: state.highest_candidate_generation,
        candidate_response_sha256: state.candidate_response_sha256.clone(),
        state_sha256: state.state_sha256.clone(),
        active_signer_ids: signer_ids(state, "active"),
        revoked_signer_ids: signer_ids(state, "revoked"),
    }
}

fn signer_ids(state: &TrustState, status: &str) -> Vec<String> {
    state
        .signers
        .iter()
        .filter(|signer| signer.status == status)
        .map(|signer| signer.signer_id.clone())
        .collect()
}

fn render_registry(registry: &TrustRegistry) -> String {
    let mut source = format!(
        "trust_registry_contract = \"{GALAXY_PROVIDER_TRUST_REGISTRY_CONTRACT}\"\nprovider_id = \"{}\"\nprovider_kind = \"{}\"\ngeneration = {}\n",
        registry.provider_id, registry.provider_kind, registry.generation
    );
    render_signers(&mut source, &registry.signers);
    source
}

fn render_state(state: &TrustState) -> String {
    let active = state
        .signers
        .iter()
        .filter(|item| item.status == "active")
        .count();
    let revoked = state
        .signers
        .iter()
        .filter(|item| item.status == "revoked")
        .count();
    let mut source = format!(
        "trust_state_contract = \"{GALAXY_PROVIDER_TRUST_STATE_CONTRACT}\"\nprovider_id = \"{}\"\nprovider_kind = \"{}\"\nregistry_generation = {}\nregistry_sha256 = \"{}\"\nhighest_candidate_generation = {}\ncandidate_response_sha256 = \"{}\"\nactive_signer_count = {active}\nrevoked_signer_count = {revoked}\nstate_sha256 = \"{}\"\n",
        state.provider_id,
        state.provider_kind,
        state.registry_generation,
        state.registry_sha256,
        state.highest_candidate_generation,
        state.candidate_response_sha256,
        state.state_sha256
    );
    render_signers(&mut source, &state.signers);
    source
}

fn render_signers(source: &mut String, signers: &[TrustSigner]) {
    for signer in signers {
        write!(
            source,
            "\n[[signer]]\nsigner_id = \"{}\"\nstatus = \"{}\"\n",
            signer.signer_id, signer.status
        )
        .unwrap();
    }
}

fn state_identity(state: &TrustState) -> String {
    let mut unsigned = state.clone();
    unsigned.state_sha256.clear();
    sha256(render_state(&unsigned).as_bytes())
}

fn parse_document(source: &str, label: &str) -> Result<ParsedTrustDocument, String> {
    let mut root = BTreeMap::new();
    let mut signers = Vec::<BTreeMap<String, String>>::new();
    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[signer]]" {
            if signers.len() >= MAX_TRUST_SIGNERS {
                return Err(format!(
                    "Galaxy provider {label} exceeds {MAX_TRUST_SIGNERS} signers"
                ));
            }
            signers.push(BTreeMap::new());
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            format!(
                "Galaxy provider {label} line {} is not a key/value field",
                index + 1
            )
        })?;
        let key = key.trim().to_owned();
        let value = parse_value(raw_value.trim()).ok_or_else(|| {
            format!("Galaxy provider {label} field `{key}` has an invalid scalar value")
        })?;
        let fields = signers.last_mut().unwrap_or(&mut root);
        if fields.insert(key.clone(), value).is_some() {
            return Err(format!("Galaxy provider {label} repeats field `{key}`"));
        }
    }
    Ok((root, signers))
}

fn parse_signers(
    signer_fields: Vec<BTreeMap<String, String>>,
    label: &str,
) -> Result<Vec<TrustSigner>, String> {
    signer_fields
        .into_iter()
        .map(|mut fields| {
            let signer = TrustSigner {
                signer_id: take_required(&mut fields, "signer_id", label)?,
                status: take_required(&mut fields, "status", label)?,
            };
            reject_unknown(fields, label)?;
            Ok(signer)
        })
        .collect()
}

fn take_required(
    fields: &mut BTreeMap<String, String>,
    key: &str,
    label: &str,
) -> Result<String, String> {
    fields
        .remove(key)
        .ok_or_else(|| format!("Galaxy provider {label} is missing required field `{key}`"))
}

fn parse_integer<T: std::str::FromStr>(
    fields: &mut BTreeMap<String, String>,
    key: &str,
    label: &str,
) -> Result<T, String> {
    take_required(fields, key, label)?
        .parse()
        .map_err(|_| format!("Galaxy provider {label} field `{key}` must be an unsigned integer"))
}

fn reject_unknown(fields: BTreeMap<String, String>, label: &str) -> Result<(), String> {
    if let Some(key) = fields.keys().next() {
        return Err(format!(
            "Galaxy provider {label} contains unknown field `{key}`"
        ));
    }
    Ok(())
}

fn parse_value(raw: &str) -> Option<String> {
    if let Some(value) = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return (!value.contains(['"', '\\', '\n', '\r'])).then(|| value.to_owned());
    }
    (!raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit())).then(|| raw.to_owned())
}

fn read_bounded_text(path: &Path, label: &str) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "failed to inspect Galaxy provider {label} `{}`: {error}",
            path.display()
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(format!(
            "Galaxy provider {label} `{}` must be a regular non-symlink file",
            path.display()
        ));
    }
    if metadata.len() > MAX_TRUST_DOCUMENT_BYTES {
        return Err(format!(
            "Galaxy provider {label} exceeds the {MAX_TRUST_DOCUMENT_BYTES}-byte limit"
        ));
    }
    let file = fs::File::open(path).map_err(|error| {
        format!(
            "failed to read Galaxy provider {label} `{}`: {error}",
            path.display()
        )
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize + 1);
    file.take(MAX_TRUST_DOCUMENT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            format!(
                "failed to read Galaxy provider {label} `{}`: {error}",
                path.display()
            )
        })?;
    if bytes.len() as u64 > MAX_TRUST_DOCUMENT_BYTES {
        return Err(format!(
            "Galaxy provider {label} exceeds the {MAX_TRUST_DOCUMENT_BYTES}-byte limit"
        ));
    }
    let source = String::from_utf8(bytes).map_err(|_| {
        format!(
            "Galaxy provider {label} `{}` must use UTF-8/LF text",
            path.display()
        )
    })?;
    if source.contains('\r') {
        return Err(format!("Galaxy provider {label} must use UTF-8/LF text"));
    }
    Ok(source)
}

fn validate_token(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(format!(
            "Galaxy provider {label} `{value}` must use ASCII letters, digits, `.`, `-`, or `_`"
        ));
    }
    Ok(())
}

fn validate_signer_id(signer_id: &str) -> Result<(), String> {
    let Some(hash) = signer_id.strip_prefix("ed25519:sha256:") else {
        return Err(format!(
            "Galaxy provider trust signer `{signer_id}` must use ed25519:sha256 identity"
        ));
    };
    validate_hex64(hash, "trust signer")
}

fn validate_sha256(value: &str, label: &str) -> Result<(), String> {
    let Some(hash) = value.strip_prefix("sha256:") else {
        return Err(format!("Galaxy provider {label} must use sha256 identity"));
    };
    validate_hex64(hash, label)
}

fn validate_hex64(value: &str, label: &str) -> Result<(), String> {
    if value.len() != 64
        || value != value.to_ascii_lowercase()
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "Galaxy provider {label} must contain 64 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", crate::digest_sha256::sha256_hex(bytes))
}

#[cfg(test)]
#[path = "stdlib_registry_provider_trust_state_tests.rs"]
mod tests;
