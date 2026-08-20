use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;

use ed25519_dalek::{Signature, VerifyingKey};

use super::{
    GalaxyResolutionCandidateSetReport, GalaxyResolutionProviderDescriptor, StdlibIndexModule,
};

pub const GALAXY_CANDIDATE_SET_CONTRACT: &str = "nuis-galaxy-candidate-set-v1";
pub const GALAXY_CANDIDATE_SET_FILE: &str = "candidate-set.toml";
const MAX_CANDIDATE_SET_BYTES: u64 = 1024 * 1024;
const MAX_CANDIDATE_SET_SIGNATURES: usize = 32;

#[derive(Debug)]
struct CandidateSetClaim {
    contract: String,
    provider_id: String,
    provider_kind: String,
    generation: u64,
    index_sha256: String,
    candidate_count: usize,
    candidate_sha256: String,
    signatures: Vec<CandidateSetSignature>,
}

#[derive(Debug)]
struct CandidateSetSignature {
    signer_id: String,
    public_key_hex: String,
    signature_hex: String,
}

pub(super) fn verify_candidate_set(
    provider: &GalaxyResolutionProviderDescriptor,
    index_bytes: &[u8],
    candidates: &BTreeMap<(String, String), StdlibIndexModule>,
) -> Result<GalaxyResolutionCandidateSetReport, String> {
    let index_sha256 = sha256(index_bytes);
    let candidate_sha256 = canonical_candidate_sha256(candidates);
    let sidecar_path = provider.root.join(GALAXY_CANDIDATE_SET_FILE);
    if !sidecar_path.exists() {
        let payload = candidate_set_signing_payload(
            provider,
            0,
            &index_sha256,
            candidates.len(),
            &candidate_sha256,
        );
        return Ok(GalaxyResolutionCandidateSetReport {
            contract: GALAXY_CANDIDATE_SET_CONTRACT.to_owned(),
            status: "unsigned-exact-only".to_owned(),
            generation: 0,
            index_sha256,
            candidate_sha256,
            response_sha256: sha256(&payload),
            signature_count: 0,
            signer_ids: Vec::new(),
        });
    }

    let sidecar_bytes = fs::metadata(&sidecar_path)
        .map_err(|error| {
            format!(
                "failed to inspect Galaxy candidate-set sidecar `{}`: {error}",
                sidecar_path.display()
            )
        })?
        .len();
    if sidecar_bytes > MAX_CANDIDATE_SET_BYTES {
        return Err(format!(
            "Galaxy candidate-set sidecar exceeds the {MAX_CANDIDATE_SET_BYTES}-byte resource limit"
        ));
    }
    let source = fs::read_to_string(&sidecar_path).map_err(|error| {
        format!(
            "failed to read Galaxy candidate-set sidecar `{}`: {error}",
            sidecar_path.display()
        )
    })?;
    let claim = parse_claim(&source)?;
    validate_claim(
        &claim,
        provider,
        &index_sha256,
        candidates.len(),
        &candidate_sha256,
    )?;
    let payload = candidate_set_signing_payload(
        provider,
        claim.generation,
        &index_sha256,
        candidates.len(),
        &candidate_sha256,
    );
    let mut signer_ids = verify_signatures(&claim.signatures, &payload)?;
    signer_ids.sort();
    let mut response = String::from_utf8(payload.clone())
        .map_err(|_| "canonical Galaxy candidate-set response is not UTF-8".to_owned())?;
    for signer_id in &signer_ids {
        append_text(&mut response, signer_id);
    }
    Ok(GalaxyResolutionCandidateSetReport {
        contract: GALAXY_CANDIDATE_SET_CONTRACT.to_owned(),
        status: "verified-signed-candidate-set".to_owned(),
        generation: claim.generation,
        index_sha256,
        candidate_sha256,
        response_sha256: sha256(response.as_bytes()),
        signature_count: signer_ids.len(),
        signer_ids,
    })
}

pub(in crate::stdlib_registry) fn canonical_candidate_sha256(
    candidates: &BTreeMap<(String, String), StdlibIndexModule>,
) -> String {
    let mut canonical = String::new();
    append_text(&mut canonical, GALAXY_CANDIDATE_SET_CONTRACT);
    for candidate in candidates.values() {
        append_text(&mut canonical, &candidate.name);
        append_text(&mut canonical, &candidate.version);
        append_text(&mut canonical, &candidate.kind);
        append_text(&mut canonical, &candidate.path);
        append_text(&mut canonical, &candidate.package_id);
        let mut dependencies = candidate.depends_on.clone();
        dependencies.sort();
        for dependency in dependencies {
            append_text(&mut canonical, &dependency);
        }
        append_text(&mut canonical, &candidate.summary);
    }
    sha256(canonical.as_bytes())
}

pub(in crate::stdlib_registry) fn candidate_set_signing_payload(
    provider: &GalaxyResolutionProviderDescriptor,
    generation: u64,
    index_sha256: &str,
    candidate_count: usize,
    candidate_sha256: &str,
) -> Vec<u8> {
    let mut canonical = String::new();
    append_text(&mut canonical, GALAXY_CANDIDATE_SET_CONTRACT);
    append_text(&mut canonical, &provider.provider_id);
    append_text(&mut canonical, &provider.provider_kind);
    writeln!(canonical, "generation={generation}").unwrap();
    append_text(&mut canonical, index_sha256);
    writeln!(canonical, "candidate_count={candidate_count}").unwrap();
    append_text(&mut canonical, candidate_sha256);
    canonical.into_bytes()
}

fn validate_claim(
    claim: &CandidateSetClaim,
    provider: &GalaxyResolutionProviderDescriptor,
    index_sha256: &str,
    candidate_count: usize,
    candidate_sha256: &str,
) -> Result<(), String> {
    if claim.contract != GALAXY_CANDIDATE_SET_CONTRACT {
        return Err(format!(
            "Galaxy candidate set has unsupported contract `{}`",
            claim.contract
        ));
    }
    if claim.provider_id != provider.provider_id || claim.provider_kind != provider.provider_kind {
        return Err(format!(
            "Galaxy candidate set provider identity drift: expected `{}/{}`, found `{}/{}`",
            provider.provider_id, provider.provider_kind, claim.provider_id, claim.provider_kind
        ));
    }
    if claim.generation == 0 {
        return Err("signed Galaxy candidate set generation must be greater than zero".to_owned());
    }
    if claim.index_sha256 != index_sha256 {
        return Err("Galaxy candidate set index_sha256 does not match index.toml bytes".to_owned());
    }
    if claim.candidate_count != candidate_count {
        return Err(format!(
            "Galaxy candidate set candidate_count drift: expected {candidate_count}, found {}",
            claim.candidate_count
        ));
    }
    if claim.candidate_sha256 != candidate_sha256 {
        return Err(
            "Galaxy candidate set candidate_sha256 does not match canonical candidates".to_owned(),
        );
    }
    if claim.signatures.is_empty() {
        return Err("signed Galaxy candidate set must contain at least one signature".to_owned());
    }
    Ok(())
}

fn verify_signatures(
    signatures: &[CandidateSetSignature],
    payload: &[u8],
) -> Result<Vec<String>, String> {
    let mut signer_ids = BTreeSet::new();
    for claim in signatures {
        let public_key = decode_array::<32>(&claim.public_key_hex, "public key")?;
        let verifying_key = VerifyingKey::from_bytes(&public_key)
            .map_err(|_| "Galaxy candidate-set public key is malformed".to_owned())?;
        let expected_signer_id = format!("ed25519:{}", sha256(&public_key));
        if claim.signer_id != expected_signer_id {
            return Err(format!(
                "Galaxy candidate-set signer id `{}` does not match its public key identity",
                claim.signer_id
            ));
        }
        if !signer_ids.insert(claim.signer_id.clone()) {
            return Err(format!(
                "Galaxy candidate set repeats signer `{}`",
                claim.signer_id
            ));
        }
        let signature =
            Signature::from_bytes(&decode_array::<64>(&claim.signature_hex, "signature")?);
        verifying_key
            .verify_strict(payload, &signature)
            .map_err(|_| {
                format!(
                    "Galaxy candidate-set signature from `{}` does not match the canonical response",
                    claim.signer_id
                )
            })?;
    }
    Ok(signer_ids.into_iter().collect())
}

fn parse_claim(source: &str) -> Result<CandidateSetClaim, String> {
    let mut root = BTreeMap::new();
    let mut signatures = Vec::<BTreeMap<String, String>>::new();
    for (index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[signature]]" {
            if signatures.len() >= MAX_CANDIDATE_SET_SIGNATURES {
                return Err(format!(
                    "Galaxy candidate set exceeds the limit of {MAX_CANDIDATE_SET_SIGNATURES} signatures"
                ));
            }
            signatures.push(BTreeMap::new());
            continue;
        }
        let (key, raw_value) = line.split_once('=').ok_or_else(|| {
            format!(
                "Galaxy candidate-set sidecar line {} is not a key/value field",
                index + 1
            )
        })?;
        let key = key.trim().to_owned();
        let value = parse_value(raw_value.trim()).ok_or_else(|| {
            format!("Galaxy candidate-set sidecar field `{key}` has an invalid scalar value")
        })?;
        let fields = signatures.last_mut().unwrap_or(&mut root);
        if fields.insert(key.clone(), value).is_some() {
            return Err(format!(
                "Galaxy candidate-set sidecar repeats field `{key}`"
            ));
        }
    }

    let signatures = signatures
        .into_iter()
        .map(|mut fields| {
            let signature = CandidateSetSignature {
                signer_id: take_required(&mut fields, "signer_id")?,
                public_key_hex: take_required(&mut fields, "public_key_hex")?,
                signature_hex: take_required(&mut fields, "signature_hex")?,
            };
            reject_unknown(fields)?;
            Ok(signature)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let claim = CandidateSetClaim {
        contract: take_required(&mut root, "candidate_set_contract")?,
        provider_id: take_required(&mut root, "provider_id")?,
        provider_kind: take_required(&mut root, "provider_kind")?,
        generation: parse_integer(&mut root, "generation")?,
        index_sha256: take_required(&mut root, "index_sha256")?,
        candidate_count: parse_integer(&mut root, "candidate_count")?,
        candidate_sha256: take_required(&mut root, "candidate_sha256")?,
        signatures,
    };
    reject_unknown(root)?;
    Ok(claim)
}

fn take_required(fields: &mut BTreeMap<String, String>, key: &str) -> Result<String, String> {
    fields
        .remove(key)
        .ok_or_else(|| format!("Galaxy candidate-set sidecar is missing required field `{key}`"))
}

fn parse_integer<T: std::str::FromStr>(
    fields: &mut BTreeMap<String, String>,
    key: &str,
) -> Result<T, String> {
    let raw = take_required(fields, key)?;
    raw.parse().map_err(|_| {
        format!("Galaxy candidate-set sidecar field `{key}` must be an unsigned integer")
    })
}

fn reject_unknown(fields: BTreeMap<String, String>) -> Result<(), String> {
    if let Some(key) = fields.keys().next() {
        return Err(format!(
            "Galaxy candidate-set sidecar contains unknown field `{key}`"
        ));
    }
    Ok(())
}

fn parse_value(raw: &str) -> Option<String> {
    if let Some(value) = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return Some(value.to_owned());
    }
    (!raw.is_empty() && raw.bytes().all(|byte| byte.is_ascii_digit())).then(|| raw.to_owned())
}

fn decode_array<const N: usize>(encoded: &str, label: &str) -> Result<[u8; N], String> {
    if encoded.len() != N * 2
        || encoded != encoded.to_ascii_lowercase()
        || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "Galaxy candidate-set {label} must contain exactly {} lowercase hexadecimal characters",
            N * 2
        ));
    }
    let mut out = [0u8; N];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16)
            .map_err(|_| format!("Galaxy candidate-set {label} is malformed"))?;
    }
    Ok(out)
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", crate::digest_sha256::sha256_hex(bytes))
}

fn append_text(out: &mut String, value: &str) {
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}
