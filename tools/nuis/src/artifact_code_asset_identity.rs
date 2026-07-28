use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};

const CONTRIBUTION_CONTRACT: &str = "nuis-nustar-code-asset-identity-contribution-v1";
const DESCRIPTOR_CONTRACT: &str = "nuis-provider-code-asset-descriptor-v1";
const DESCRIPTOR_IDENTITY_CONTRACT: &str = "nuis-provider-code-asset-descriptor-identity-v1";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";
const IDENTITY_SET_CONTRACT: &str = "nuis-provider-code-asset-identity-set-v1";

pub(super) struct NustarCodeAssetContribution<'a> {
    pub request_index: usize,
    pub owner_package_id: &'a str,
    pub provider_family: &'a str,
    pub asset_id: &'a str,
    pub format: &'a str,
    pub target: &'a str,
    pub entry: &'a str,
    pub path: &'a str,
    pub bytes: &'a [u8],
}

pub(super) struct AssembledCodeAssetIdentity {
    descriptors: BTreeMap<usize, String>,
    pub identity_evidence: String,
}

impl AssembledCodeAssetIdentity {
    pub fn descriptor_for(&self, request_index: usize) -> Result<&str, String> {
        self.descriptors
            .get(&request_index)
            .map(String::as_str)
            .ok_or_else(|| {
                format!("no Nustar code asset contribution exists for request {request_index}")
            })
    }
}

pub(super) fn assemble_nustar_code_asset_identity(
    registry_root: &Path,
    contributions: &[NustarCodeAssetContribution<'_>],
) -> Result<AssembledCodeAssetIdentity, String> {
    if contributions.is_empty() || contributions.len() > 64 {
        return Err("Nustar code asset contribution count must be within 1..=64".to_owned());
    }
    nuisc::registry::ensure_registered_domains_valid(registry_root)?;
    let manifests = nuisc::registry::load_all_manifests(registry_root)?;
    let registrations = manifests
        .iter()
        .map(nuisc::registry::provider_bundle_registrations)
        .collect::<Result<Vec<_>, _>>()?;
    let provider_owners = registrations
        .into_iter()
        .flatten()
        .map(|registration| (registration.provider_family, registration.package_id))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = contributions.iter().collect::<Vec<_>>();
    ordered.sort_by(|lhs, rhs| {
        lhs.request_index
            .cmp(&rhs.request_index)
            .then_with(|| lhs.asset_id.cmp(rhs.asset_id))
    });
    validate_contributions(&ordered, &provider_owners)?;

    let mut descriptors = BTreeMap::new();
    let mut identity_items = Vec::with_capacity(ordered.len());
    for contribution in ordered {
        let content_hash = fnv1a64_hex(contribution.bytes);
        let identity_hash = descriptor_identity_hash(contribution, &content_hash);
        descriptors.insert(
            contribution.request_index,
            render_descriptor(contribution, &content_hash),
        );
        identity_items.push((
            contribution.asset_id,
            identity_hash,
            contribution.owner_package_id,
            contribution.provider_family,
        ));
    }
    let root_hash = identity_set_root_hash(&identity_items);
    Ok(AssembledCodeAssetIdentity {
        descriptors,
        identity_evidence: render_identity_set(&identity_items, &root_hash),
    })
}

fn validate_contributions(
    contributions: &[&NustarCodeAssetContribution<'_>],
    provider_owners: &BTreeMap<String, String>,
) -> Result<(), String> {
    let mut request_indices = BTreeSet::new();
    let mut asset_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for contribution in contributions {
        if !request_indices.insert(contribution.request_index)
            || !asset_ids.insert(contribution.asset_id)
            || !paths.insert(contribution.path)
        {
            return Err("Nustar code asset contributions must have unique request, asset, and path identities".to_owned());
        }
        if !token_is_valid(contribution.asset_id)
            || !token_is_valid(contribution.format)
            || !token_is_valid(contribution.target)
            || !symbol_is_valid(contribution.entry)
            || !relative_path_is_valid(contribution.path)
            || contribution.bytes.is_empty()
        {
            return Err(format!(
                "Nustar code asset contribution `{}` is malformed",
                contribution.asset_id
            ));
        }
        if provider_owners
            .get(contribution.provider_family)
            .map(String::as_str)
            != Some(contribution.owner_package_id)
        {
            return Err(format!(
                "provider family `{}` is not owned by Nustar package `{}`",
                contribution.provider_family, contribution.owner_package_id
            ));
        }
    }
    Ok(())
}

fn render_descriptor(contribution: &NustarCodeAssetContribution<'_>, content_hash: &str) -> String {
    let prefix = format!(
        "provider_request_{}_code_asset_",
        contribution.request_index
    );
    format!(
        "{prefix}descriptor_contract={DESCRIPTOR_CONTRACT};{prefix}id={};{prefix}format={};{prefix}target={};{prefix}entry={};{prefix}path={};{prefix}byte_length={};{prefix}digest_contract={DIGEST_CONTRACT};{prefix}content_hash={content_hash}",
        contribution.asset_id,
        contribution.format,
        contribution.target,
        contribution.entry,
        contribution.path,
        contribution.bytes.len(),
    )
}

fn render_identity_set(items: &[(&str, String, &str, &str)], root_hash: &str) -> String {
    let mut rendered = format!(
        "provider_code_asset_identity_set_contract={IDENTITY_SET_CONTRACT};provider_code_asset_identity_set_count={};provider_code_asset_identity_set_root_hash={root_hash}",
        items.len()
    );
    for (index, (asset_id, identity_hash, package_id, provider_family)) in items.iter().enumerate()
    {
        rendered.push_str(&format!(
            ";provider_code_asset_identity_item_{index}_asset_id={asset_id};provider_code_asset_identity_item_{index}_contract={DESCRIPTOR_IDENTITY_CONTRACT};provider_code_asset_identity_item_{index}_hash={identity_hash};provider_code_asset_identity_item_{index}_contribution_contract={CONTRIBUTION_CONTRACT};provider_code_asset_identity_item_{index}_owner_package_id={package_id};provider_code_asset_identity_item_{index}_provider_family={provider_family}"
        ));
    }
    rendered
}

fn descriptor_identity_hash(
    contribution: &NustarCodeAssetContribution<'_>,
    content_hash: &str,
) -> String {
    fnv1a64_hex(
        format!(
            "{DESCRIPTOR_IDENTITY_CONTRACT}\n{}\n{}\n{}\n{}\n{}\n{DIGEST_CONTRACT}\n{content_hash}\n1\n{}",
            contribution.asset_id,
            contribution.format,
            contribution.target,
            contribution.path,
            contribution.bytes.len(),
            contribution.entry,
        )
        .as_bytes(),
    )
}

fn identity_set_root_hash(items: &[(&str, String, &str, &str)]) -> String {
    let canonical = items
        .iter()
        .map(|(asset_id, identity_hash, _, _)| {
            format!("{asset_id}\n{DESCRIPTOR_IDENTITY_CONTRACT}\n{identity_hash}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fnv1a64_hex(format!("{IDENTITY_SET_CONTRACT}\n{}\n{canonical}", items.len()).as_bytes())
}

fn token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn symbol_is_valid(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'$'))
}

fn relative_path_is_valid(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !value.contains(['\\', ':'])
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &[u8] = b"kernel void first() {}";
    const SECOND: &[u8] = b"kernel void second() {}";

    fn contribution(
        request_index: usize,
        asset_id: &'static str,
        entry: &'static str,
        path: &'static str,
        bytes: &'static [u8],
    ) -> NustarCodeAssetContribution<'static> {
        NustarCodeAssetContribution {
            request_index,
            owner_package_id: "official.shader",
            provider_family: "metal:apple-silicon-gpu",
            asset_id,
            format: "metal-source",
            target: "metal.apple-silicon-gpu",
            entry,
            path,
            bytes,
        }
    }

    #[test]
    fn assembles_registered_nustar_contributions_in_request_order() {
        let assembled = assemble_nustar_code_asset_identity(
            Path::new("nustar-packages"),
            &[
                contribution(6, "shader.second", "second", "second.metal", SECOND),
                contribution(4, "shader.first", "first", "first.metal", FIRST),
            ],
        )
        .unwrap();
        assert!(assembled
            .descriptor_for(4)
            .unwrap()
            .contains("provider_request_4_code_asset_id=shader.first"));
        assert!(assembled
            .identity_evidence
            .contains("provider_code_asset_identity_set_count=2"));
        assert!(assembled
            .identity_evidence
            .contains("provider_code_asset_identity_item_0_asset_id=shader.first"));
        assert!(assembled
            .identity_evidence
            .contains("provider_code_asset_identity_item_1_asset_id=shader.second"));
        assert!(assembled
            .identity_evidence
            .contains("provider_code_asset_identity_item_0_owner_package_id=official.shader"));
    }

    #[test]
    fn rejects_unregistered_or_duplicate_contributions() {
        let mut wrong_owner = contribution(4, "shader.first", "first", "first.metal", FIRST);
        wrong_owner.owner_package_id = "official.kernel";
        assert!(
            assemble_nustar_code_asset_identity(Path::new("nustar-packages"), &[wrong_owner])
                .is_err()
        );
        assert!(assemble_nustar_code_asset_identity(
            Path::new("nustar-packages"),
            &[
                contribution(4, "shader.first", "first", "first.metal", FIRST),
                contribution(4, "shader.second", "second", "second.metal", SECOND),
            ],
        )
        .is_err());
    }
}
