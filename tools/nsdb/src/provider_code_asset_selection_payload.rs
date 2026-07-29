use crate::model::{CompiledCodeAssetSelectionEvidence, CompiledCodeAssetSelectionItem};

const SINGLE_CONTRACT: &str = "nuis-provider-code-asset-contribution-selection-v1";
const SET_CONTRACT: &str = "nuis-provider-code-asset-contribution-selection-set-v1";
const TABLE_CONTRACT: &str = "nuis-domain-code-asset-contribution-table-v1";

pub(crate) fn append_provider_output_selection(
    out: &mut String,
    selection: Option<&CompiledCodeAssetSelectionEvidence>,
) {
    let default = CompiledCodeAssetSelectionEvidence::default();
    render_fields(out, selection.unwrap_or(&default));
}

pub(crate) fn validate_provider_output_selection(
    source: &str,
) -> Result<CompiledCodeAssetSelectionEvidence, String> {
    parse_serialized_selection(source)
}

pub(crate) fn render_completion_event_fields(
    out: &mut String,
    selection: &CompiledCodeAssetSelectionEvidence,
) {
    render_fields(out, selection);
}

pub(crate) fn parse_completion_event_fields(source: &str) -> CompiledCodeAssetSelectionEvidence {
    parse_serialized_selection(source).unwrap_or_else(|_| {
        let mut invalid = CompiledCodeAssetSelectionEvidence::default();
        invalid.status = "invalid".to_owned();
        invalid
    })
}

pub(crate) fn append_selection_hash_material(
    material: &mut String,
    selection: &CompiledCodeAssetSelectionEvidence,
) {
    if selection.status == "verified" {
        let mut rendered = String::new();
        render_fields(&mut rendered, selection);
        for line in rendered.lines() {
            material.push('\0');
            material.push_str(line);
        }
    }
}

fn render_fields(out: &mut String, selection: &CompiledCodeAssetSelectionEvidence) {
    for (key, value) in [
        (
            "compiled_code_asset_selection_contract",
            selection.contract.as_str(),
        ),
        (
            "compiled_code_asset_selection_status",
            selection.status.as_str(),
        ),
        (
            "compiled_code_asset_table_contract",
            selection.table_contract.as_str(),
        ),
        (
            "compiled_code_asset_table_hash",
            selection.table_hash.as_str(),
        ),
        (
            "compiled_code_asset_contribution_count",
            &selection.contribution_count.to_string(),
        ),
        (
            "compiled_code_asset_identity_set_root_hash",
            selection.identity_set_root_hash.as_str(),
        ),
        (
            "compiled_code_asset_contribution_index",
            &selection.contribution_index.to_string(),
        ),
        ("compiled_code_asset_asset_id", selection.asset_id.as_str()),
        (
            "compiled_code_asset_identity_hash",
            selection.identity_hash.as_str(),
        ),
        (
            "compiled_code_asset_selection_count",
            &selection.selections.len().to_string(),
        ),
    ] {
        push(out, key, value);
    }
    for (index, item) in selection.selections.iter().enumerate() {
        push(
            out,
            &format!("compiled_code_asset_selection_{index}_contribution_index"),
            &item.contribution_index.to_string(),
        );
        push(
            out,
            &format!("compiled_code_asset_selection_{index}_asset_id"),
            &item.asset_id,
        );
        push(
            out,
            &format!("compiled_code_asset_selection_{index}_identity_hash"),
            &item.identity_hash,
        );
    }
}

fn parse_serialized_selection(source: &str) -> Result<CompiledCodeAssetSelectionEvidence, String> {
    let has_fields = source
        .lines()
        .any(|line| line.trim().starts_with("compiled_code_asset_"));
    let Some(status) = string_field(source, "compiled_code_asset_selection_status") else {
        return (!has_fields)
            .then(CompiledCodeAssetSelectionEvidence::default)
            .ok_or_else(|| "compiled code asset selection output is partial".to_owned());
    };
    let contract = required(source, "compiled_code_asset_selection_contract")?;
    let selection_count = usize_field(source, "compiled_code_asset_selection_count")
        .unwrap_or_else(|| usize::from(status == "verified"));
    let selections = (0..selection_count)
        .map(|index| {
            let legacy = selection_count == 1
                && string_field(
                    source,
                    &format!("compiled_code_asset_selection_{index}_asset_id"),
                )
                .is_none();
            Ok(CompiledCodeAssetSelectionItem {
                contribution_index: if legacy {
                    required_usize(source, "compiled_code_asset_contribution_index")?
                } else {
                    required_usize(
                        source,
                        &format!("compiled_code_asset_selection_{index}_contribution_index"),
                    )?
                },
                asset_id: if legacy {
                    required(source, "compiled_code_asset_asset_id")?
                } else {
                    required(
                        source,
                        &format!("compiled_code_asset_selection_{index}_asset_id"),
                    )?
                },
                identity_hash: if legacy {
                    required(source, "compiled_code_asset_identity_hash")?
                } else {
                    required(
                        source,
                        &format!("compiled_code_asset_selection_{index}_identity_hash"),
                    )?
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let selection = CompiledCodeAssetSelectionEvidence {
        contract,
        status,
        table_contract: required(source, "compiled_code_asset_table_contract")?,
        table_hash: required(source, "compiled_code_asset_table_hash")?,
        contribution_count: required_usize(source, "compiled_code_asset_contribution_count")?,
        identity_set_root_hash: required(source, "compiled_code_asset_identity_set_root_hash")?,
        contribution_index: required_usize(source, "compiled_code_asset_contribution_index")?,
        asset_id: required(source, "compiled_code_asset_asset_id")?,
        identity_hash: required(source, "compiled_code_asset_identity_hash")?,
        selections,
    };
    validate_selection(&selection)?;
    Ok(selection)
}

fn validate_selection(selection: &CompiledCodeAssetSelectionEvidence) -> Result<(), String> {
    if selection == &CompiledCodeAssetSelectionEvidence::default() {
        return Ok(());
    }
    let count = selection.selections.len();
    let contract_matches = (count == 1 && selection.contract == SINGLE_CONTRACT)
        || (count > 1 && selection.contract == SET_CONTRACT);
    let unique = selection
        .selections
        .iter()
        .map(|item| item.contribution_index)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        == count;
    if !contract_matches
        || selection.status != "verified"
        || selection.table_contract != TABLE_CONTRACT
        || selection.contribution_count == 0
        || selection.contribution_count > 64
        || !(1..=64).contains(&count)
        || !unique
        || selection.selections[0].contribution_index != selection.contribution_index
        || selection.selections[0].asset_id != selection.asset_id
        || selection.selections[0].identity_hash != selection.identity_hash
        || selection.selections.iter().any(|item| {
            item.contribution_index >= selection.contribution_count
                || !token_is_valid(&item.asset_id)
                || !valid_hash(&item.identity_hash)
        })
        || !valid_hash(&selection.table_hash)
        || !valid_hash(&selection.identity_set_root_hash)
    {
        return Err("compiled code asset selection output is invalid".to_owned());
    }
    Ok(())
}

fn required(source: &str, key: &str) -> Result<String, String> {
    string_field(source, key)
        .ok_or_else(|| format!("compiled code asset selection field `{key}` is missing"))
}

fn required_usize(source: &str, key: &str) -> Result<usize, String> {
    usize_field(source, key)
        .ok_or_else(|| format!("compiled code asset selection field `{key}` is missing"))
}

fn string_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)?
            .trim()
            .strip_prefix('"')?
            .strip_suffix('"')
            .map(str::to_owned)
    })
}

fn usize_field(source: &str, key: &str) -> Option<usize> {
    string_field(source, key)?.parse().ok()
}

fn push(out: &mut String, key: &str, value: &str) {
    crate::provider_sample_artifact::push_toml_string(out, key, value);
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn token_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection_set() -> CompiledCodeAssetSelectionEvidence {
        let selections = vec![
            CompiledCodeAssetSelectionItem {
                contribution_index: 1,
                asset_id: "shader.witsage.vector-bias.metal".to_owned(),
                identity_hash: "0x8ffdbef37025887e".to_owned(),
            },
            CompiledCodeAssetSelectionItem {
                contribution_index: 0,
                asset_id: "shader.witsage.argmax.metal".to_owned(),
                identity_hash: "0x2615f55c0ad29c8f".to_owned(),
            },
        ];
        CompiledCodeAssetSelectionEvidence {
            contract: SET_CONTRACT.to_owned(),
            status: "verified".to_owned(),
            table_contract: TABLE_CONTRACT.to_owned(),
            table_hash: "0x94ada66259210849".to_owned(),
            contribution_count: 2,
            identity_set_root_hash: "0x0f4881ae1bda4658".to_owned(),
            contribution_index: selections[0].contribution_index,
            asset_id: selections[0].asset_id.clone(),
            identity_hash: selections[0].identity_hash.clone(),
            selections,
        }
    }

    #[test]
    fn two_item_selection_set_round_trips_in_request_order() {
        let expected = selection_set();
        let mut rendered = String::new();
        render_fields(&mut rendered, &expected);

        let actual = parse_serialized_selection(&rendered).unwrap();

        assert_eq!(actual, expected);
        assert_eq!(actual.selections[0].contribution_index, 1);
        assert_eq!(actual.selections[1].contribution_index, 0);
    }

    #[test]
    fn second_item_tampering_changes_completion_hash_material() {
        let original = selection_set();
        let mut tampered = original.clone();
        tampered.selections[1].identity_hash = "0x3615f55c0ad29c8f".to_owned();
        let mut original_material = String::new();
        let mut tampered_material = String::new();

        append_selection_hash_material(&mut original_material, &original);
        append_selection_hash_material(&mut tampered_material, &tampered);

        assert_ne!(original_material, tampered_material);
        assert!(tampered_material.contains("0x3615f55c0ad29c8f"));
    }

    #[test]
    fn invalid_second_item_is_rejected() {
        let mut rendered = String::new();
        render_fields(&mut rendered, &selection_set());
        let tampered = rendered.replace(
            "compiled_code_asset_selection_1_identity_hash = \"0x2615f55c0ad29c8f\"",
            "compiled_code_asset_selection_1_identity_hash = \"invalid\"",
        );

        assert!(parse_serialized_selection(&tampered).is_err());
    }

    #[test]
    fn legacy_single_item_payload_remains_compatible() {
        let source = r#"
compiled_code_asset_selection_contract = "nuis-provider-code-asset-contribution-selection-v1"
compiled_code_asset_selection_status = "verified"
compiled_code_asset_table_contract = "nuis-domain-code-asset-contribution-table-v1"
compiled_code_asset_table_hash = "0x94ada66259210849"
compiled_code_asset_contribution_count = "2"
compiled_code_asset_identity_set_root_hash = "0x0f4881ae1bda4658"
compiled_code_asset_contribution_index = "1"
compiled_code_asset_asset_id = "shader.witsage.vector-bias.metal"
compiled_code_asset_identity_hash = "0x8ffdbef37025887e"
"#;

        let parsed = parse_serialized_selection(source).unwrap();

        assert_eq!(parsed.selections.len(), 1);
        assert_eq!(parsed.selections[0].contribution_index, 1);
        assert_eq!(
            parsed.selections[0].asset_id,
            "shader.witsage.vector-bias.metal"
        );
    }
}
