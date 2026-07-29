use crate::{
    model::{CompiledCodeAssetSelectionEvidence, CompiledCodeAssetSelectionItem},
    provider_request::ProviderRequest,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use super::contribution::{load_validated_table, ContributionRow, TABLE_CONTRACT};

const SET_CONTRACT: &str = "nuis-provider-code-asset-contribution-selection-set-v1";
const IDENTITY_CONTRACT: &str = "nuis-provider-code-asset-descriptor-identity-v1";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

pub(super) fn validate_compiled_contribution_set(
    output_dir: &Path,
    evidence: &BTreeMap<String, String>,
    requests: &[ProviderRequest],
) -> Result<CompiledCodeAssetSelectionEvidence, String> {
    require(
        evidence,
        "provider_code_asset_contribution_selection_set_contract",
        SET_CONTRACT,
    )?;
    require(
        evidence,
        "provider_code_asset_contribution_table_contract",
        TABLE_CONTRACT,
    )?;
    require(
        evidence,
        "compiled_code_asset_contribution_table_contract",
        TABLE_CONTRACT,
    )?;
    require(
        evidence,
        "compiled_code_asset_contribution_table_status",
        "verified",
    )?;
    let table = load_validated_table(output_dir)?;
    for (key, expected) in [
        (
            "provider_code_asset_contribution_table_hash",
            table.table_hash.as_str(),
        ),
        (
            "provider_code_asset_contribution_identity_set_root_hash",
            table.identity_set_root_hash.as_str(),
        ),
        (
            "compiled_code_asset_contribution_table_hash",
            table.table_hash.as_str(),
        ),
        (
            "compiled_code_asset_identity_set_root_hash",
            table.identity_set_root_hash.as_str(),
        ),
    ] {
        require(evidence, key, expected)?;
    }
    if usize_field(evidence, "compiled_code_asset_contribution_count")? != table.rows.len() {
        return Err("compiled code asset contribution count drifted".to_owned());
    }
    let count = usize_field(evidence, "provider_code_asset_contribution_selection_count")?;
    if !(2..=64).contains(&count) {
        return Err("compiled contribution selection set count is invalid".to_owned());
    }
    let mut items = Vec::with_capacity(count);
    let mut selected_rows = Vec::with_capacity(count);
    let mut indices = BTreeSet::new();
    for position in 0..count {
        let prefix = format!("provider_code_asset_contribution_selection_{position}_");
        let index = usize_field(evidence, &format!("{prefix}index"))?;
        if !indices.insert(index) {
            return Err("compiled contribution selection set has duplicate rows".to_owned());
        }
        let row = table
            .rows
            .get(index)
            .ok_or_else(|| "compiled contribution selection set index is invalid".to_owned())?;
        for (suffix, expected) in [
            ("owner_package_id", row.owner_package_id.as_str()),
            ("domain_family", row.domain_family.as_str()),
            ("lowering_target", row.lowering_target.as_str()),
            ("asset_id", row.asset_id.as_str()),
            ("identity_contract", IDENTITY_CONTRACT),
            ("identity_hash", row.identity_hash.as_str()),
        ] {
            require(evidence, &format!("{prefix}{suffix}"), expected)?;
        }
        selected_rows.push(row);
        items.push(CompiledCodeAssetSelectionItem {
            contribution_index: index,
            asset_id: row.asset_id.clone(),
            identity_hash: row.identity_hash.clone(),
        });
    }
    validate_requests(&selected_rows, requests)?;
    let first = items
        .first()
        .expect("validated selection set contains at least two items");
    Ok(CompiledCodeAssetSelectionEvidence {
        contract: SET_CONTRACT.to_owned(),
        status: "verified".to_owned(),
        table_contract: TABLE_CONTRACT.to_owned(),
        table_hash: table.table_hash,
        contribution_count: table.rows.len(),
        identity_set_root_hash: table.identity_set_root_hash,
        contribution_index: first.contribution_index,
        asset_id: first.asset_id.clone(),
        identity_hash: first.identity_hash.clone(),
        selections: items,
    })
}

fn validate_requests(
    rows: &[&ContributionRow],
    requests: &[ProviderRequest],
) -> Result<(), String> {
    let request_assets = requests
        .iter()
        .filter_map(|request| request.code_asset.as_ref())
        .collect::<Vec<_>>();
    let mut ordered_ids = Vec::new();
    for asset in &request_assets {
        if !ordered_ids.iter().any(|id| id == &asset.id) {
            ordered_ids.push(asset.id.clone());
        }
    }
    if request_assets.is_empty()
        || ordered_ids
            != rows
                .iter()
                .map(|row| row.asset_id.clone())
                .collect::<Vec<_>>()
    {
        return Err(
            "provider requests do not match compiled contribution selection order".to_owned(),
        );
    }
    for row in rows {
        let assets = request_assets
            .iter()
            .filter(|asset| asset.id == row.asset_id)
            .collect::<Vec<_>>();
        if assets
            .iter()
            .map(|asset| asset.entry.as_str())
            .collect::<Vec<_>>()
            != row.entries
            || assets.iter().any(|asset| {
                asset.format != row.format
                    || asset.target != row.target
                    || asset.path != row.path
                    || asset.byte_length != row.byte_length
                    || asset.digest_contract != DIGEST_CONTRACT
                    || asset.content_hash != row.content_hash
            })
        {
            return Err(format!(
                "provider requests do not match compiled contribution `{}`",
                row.asset_id
            ));
        }
    }
    Ok(())
}

fn require(fields: &BTreeMap<String, String>, key: &str, expected: &str) -> Result<(), String> {
    (fields.get(key).map(String::as_str) == Some(expected))
        .then_some(())
        .ok_or_else(|| format!("compiled contribution selection field `{key}` drifted"))
}

fn usize_field(fields: &BTreeMap<String, String>, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("compiled contribution selection field `{key}` is missing"))
}
