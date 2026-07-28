use crate::provider_request::ProviderRequest;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

const FILE_NAME: &str = "nuis.domain.code-asset-contributions.toml";
const TABLE_CONTRACT: &str = "nuis-domain-code-asset-contribution-table-v1";
const CONTRIBUTION_CONTRACT: &str = "nuis-nustar-code-asset-identity-contribution-v1";
const SELECTION_CONTRACT: &str = "nuis-provider-code-asset-contribution-selection-v1";
const DESCRIPTOR_IDENTITY_CONTRACT: &str = "nuis-provider-code-asset-descriptor-identity-v1";
const IDENTITY_SET_CONTRACT: &str = "nuis-provider-code-asset-identity-set-v1";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContributionRow {
    index: usize,
    owner_package_id: String,
    domain_family: String,
    asset_id: String,
    format: String,
    lowering_target: String,
    target: String,
    path: String,
    entries: Vec<String>,
    byte_length: usize,
    content_hash: String,
    identity_hash: String,
}

pub(crate) fn validate_compiled_contribution_selection(
    output_dir: &Path,
    evidence: &str,
    requests: &[ProviderRequest],
) -> Result<(), String> {
    let evidence = parse_evidence_fields(evidence)?;
    let selection_fields = evidence
        .keys()
        .any(|key| key.starts_with("provider_code_asset_contribution_"));
    let Some(contract) = evidence.get("provider_code_asset_contribution_selection_contract") else {
        return (!selection_fields)
            .then_some(())
            .ok_or_else(|| "provider code asset contribution selection is partial".to_owned());
    };
    if contract != SELECTION_CONTRACT
        || evidence.get("provider_code_asset_contribution_table_contract")
            != Some(&TABLE_CONTRACT.to_owned())
        || evidence.get("provider_code_asset_contribution_identity_contract")
            != Some(&DESCRIPTOR_IDENTITY_CONTRACT.to_owned())
        || evidence.get("compiled_code_asset_contribution_table_contract")
            != Some(&TABLE_CONTRACT.to_owned())
        || evidence.get("compiled_code_asset_contribution_table_status")
            != Some(&"verified".to_owned())
    {
        return Err("provider code asset contribution selection contract is invalid".to_owned());
    }
    let source = fs::read_to_string(output_dir.join(FILE_NAME)).map_err(|error| {
        format!("failed to read compiled code asset contribution table: {error}")
    })?;
    let (header, rows) = parse_table(&source)?;
    validate_table(output_dir, &header, &rows)?;
    let index = required_usize(
        &evidence,
        "provider_code_asset_contribution_index",
        "selection",
    )?;
    let row = rows
        .get(index)
        .ok_or_else(|| "provider code asset contribution selection index is invalid".to_owned())?;
    for (key, expected) in [
        (
            "provider_code_asset_contribution_table_hash",
            string_field(&header, "table_hash")?,
        ),
        (
            "provider_code_asset_contribution_identity_set_root_hash",
            string_field(&header, "identity_set_root_hash")?,
        ),
        (
            "provider_code_asset_contribution_owner_package_id",
            row.owner_package_id.clone(),
        ),
        (
            "provider_code_asset_contribution_domain_family",
            row.domain_family.clone(),
        ),
        (
            "provider_code_asset_contribution_lowering_target",
            row.lowering_target.clone(),
        ),
        (
            "provider_code_asset_contribution_asset_id",
            row.asset_id.clone(),
        ),
        (
            "provider_code_asset_contribution_identity_hash",
            row.identity_hash.clone(),
        ),
        (
            "compiled_code_asset_identity_set_root_hash",
            string_field(&header, "identity_set_root_hash")?,
        ),
        (
            "compiled_code_asset_contribution_table_hash",
            string_field(&header, "table_hash")?,
        ),
    ] {
        if evidence.get(key) != Some(&expected) {
            return Err(format!(
                "provider code asset contribution selection field `{key}` drifted"
            ));
        }
    }
    if required_usize(
        &evidence,
        "compiled_code_asset_contribution_count",
        "compiled table",
    )? != rows.len()
    {
        return Err("compiled code asset contribution count drifted".to_owned());
    }
    validate_requests(row, requests)
}

fn parse_table(source: &str) -> Result<(BTreeMap<String, String>, Vec<ContributionRow>), String> {
    let mut sections = source.split("\n[[contribution]]\n");
    let header = parse_table_fields(
        sections
            .next()
            .ok_or_else(|| "compiled contribution table header is missing".to_owned())?,
    )?;
    let rows = sections.map(parse_row).collect::<Result<Vec<_>, _>>()?;
    Ok((header, rows))
}

fn parse_row(source: &str) -> Result<ContributionRow, String> {
    let fields = parse_table_fields(source)?;
    require_table_field(&fields, "digest_contract", DIGEST_CONTRACT)?;
    require_table_field(&fields, "identity_contract", DESCRIPTOR_IDENTITY_CONTRACT)?;
    let entries = string_array_field(&fields, "entries")?;
    if required_usize(&fields, "entry_count", "row")? != entries.len() {
        return Err("compiled contribution row entry count drifted".to_owned());
    }
    Ok(ContributionRow {
        index: required_usize(&fields, "index", "row")?,
        owner_package_id: string_field(&fields, "owner_package_id")?,
        domain_family: string_field(&fields, "domain_family")?,
        asset_id: string_field(&fields, "asset_id")?,
        format: string_field(&fields, "format")?,
        lowering_target: string_field(&fields, "lowering_target")?,
        target: string_field(&fields, "target")?,
        path: string_field(&fields, "path")?,
        entries,
        byte_length: required_usize(&fields, "byte_length", "row")?,
        content_hash: string_field(&fields, "content_hash")?,
        identity_hash: string_field(&fields, "identity_hash")?,
    })
}

fn validate_table(
    output_dir: &Path,
    header: &BTreeMap<String, String>,
    rows: &[ContributionRow],
) -> Result<(), String> {
    require_table_field(header, "protocol", TABLE_CONTRACT)?;
    require_table_field(header, "contribution_contract", CONTRIBUTION_CONTRACT)?;
    require_table_field(header, "identity_set_contract", IDENTITY_SET_CONTRACT)?;
    if rows.is_empty()
        || rows.len() > 64
        || required_usize(header, "contribution_count", "table")? != rows.len()
    {
        return Err("compiled contribution table count is invalid".to_owned());
    }
    let mut ordered = rows.to_vec();
    ordered.sort_by(|lhs, rhs| {
        lhs.domain_family
            .cmp(&rhs.domain_family)
            .then_with(|| lhs.owner_package_id.cmp(&rhs.owner_package_id))
            .then_with(|| lhs.asset_id.cmp(&rhs.asset_id))
    });
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    if ordered != rows {
        return Err("compiled contribution rows are not canonical".to_owned());
    }
    for (index, row) in rows.iter().enumerate() {
        if row.index != index
            || !ids.insert(row.asset_id.as_str())
            || !paths.insert(row.path.as_str())
            || !token_is_valid(&row.owner_package_id)
            || !token_is_valid(&row.domain_family)
            || !token_is_valid(&row.asset_id)
            || !token_is_valid(&row.format)
            || !token_is_valid(&row.lowering_target)
            || !token_is_valid(&row.target)
            || row.entries.is_empty()
            || row.entries.iter().any(|entry| !symbol_is_valid(entry))
            || row.byte_length == 0
            || !relative_path_is_valid(&row.path)
        {
            return Err(format!(
                "compiled contribution row `{}` is invalid",
                row.asset_id
            ));
        }
        let bytes = fs::read(output_dir.join(&row.path))
            .map_err(|error| format!("failed to read selected compiled code asset: {error}"))?;
        if bytes.len() != row.byte_length
            || fnv1a64_hex(&bytes) != row.content_hash
            || descriptor_identity_hash(row) != row.identity_hash
        {
            return Err(format!(
                "compiled contribution row `{}` identity drifted",
                row.asset_id
            ));
        }
    }
    if string_field(header, "identity_set_root_hash")? != identity_set_root_hash(rows)
        || string_field(header, "table_hash")? != contribution_table_hash(rows)
    {
        return Err("compiled contribution table hash evidence drifted".to_owned());
    }
    Ok(())
}

fn validate_requests(row: &ContributionRow, requests: &[ProviderRequest]) -> Result<(), String> {
    let selected = requests
        .iter()
        .filter_map(|request| request.code_asset.as_ref())
        .filter(|asset| asset.id == row.asset_id)
        .collect::<Vec<_>>();
    if selected.len() != requests.len()
        || selected
            .iter()
            .map(|asset| asset.entry.as_str())
            .collect::<Vec<_>>()
            != row.entries
        || selected.iter().any(|asset| {
            asset.format != row.format
                || asset.target != row.target
                || asset.path != row.path
                || asset.byte_length != row.byte_length
                || asset.digest_contract != DIGEST_CONTRACT
                || asset.content_hash != row.content_hash
        })
    {
        return Err("provider requests do not match selected compiled contribution".to_owned());
    }
    Ok(())
}

fn descriptor_identity_hash(row: &ContributionRow) -> String {
    fnv1a64_hex(
        format!(
            "{DESCRIPTOR_IDENTITY_CONTRACT}\n{}\n{}\n{}\n{}\n{}\n{DIGEST_CONTRACT}\n{}\n{}\n{}",
            row.asset_id,
            row.format,
            row.target,
            row.path,
            row.byte_length,
            row.content_hash,
            row.entries.len(),
            row.entries.join("\n")
        )
        .as_bytes(),
    )
}

fn identity_set_root_hash(rows: &[ContributionRow]) -> String {
    let material = rows
        .iter()
        .map(|row| {
            format!(
                "{}\n{DESCRIPTOR_IDENTITY_CONTRACT}\n{}",
                row.asset_id, row.identity_hash
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fnv1a64_hex(format!("{IDENTITY_SET_CONTRACT}\n{}\n{material}", rows.len()).as_bytes())
}

fn contribution_table_hash(rows: &[ContributionRow]) -> String {
    let material = rows
        .iter()
        .map(|row| {
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                row.owner_package_id,
                row.domain_family,
                row.asset_id,
                row.format,
                row.lowering_target,
                row.target,
                row.path,
                row.entries.len(),
                row.entries.join("\n"),
                row.byte_length,
                row.content_hash,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fnv1a64_hex(format!("{TABLE_CONTRACT}\n{}\n{material}", rows.len()).as_bytes())
}

fn parse_evidence_fields(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut fields = BTreeMap::new();
    for field in source.split(';') {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        if fields.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("provider evidence field `{key}` is duplicated"));
        }
    }
    Ok(fields)
}

fn parse_table_fields(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut fields = BTreeMap::new();
    for line in source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("malformed compiled contribution line `{line}`"))?;
        if fields
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(format!(
                "compiled contribution field `{}` is duplicated",
                key.trim()
            ));
        }
    }
    Ok(fields)
}

fn require_table_field(
    fields: &BTreeMap<String, String>,
    key: &str,
    expected: &str,
) -> Result<(), String> {
    (string_field(fields, key)? == expected)
        .then_some(())
        .ok_or_else(|| format!("compiled contribution field `{key}` is unsupported"))
}

fn string_field(fields: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    fields
        .get(key)
        .and_then(|value| value.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("compiled contribution string field `{key}` is missing"))
}

fn string_array_field(fields: &BTreeMap<String, String>, key: &str) -> Result<Vec<String>, String> {
    let value = fields
        .get(key)
        .and_then(|value| value.strip_prefix('[')?.strip_suffix(']'))
        .ok_or_else(|| format!("compiled contribution array field `{key}` is missing"))?;
    value
        .split(',')
        .filter(|item| !item.trim().is_empty())
        .map(|item| {
            item.trim()
                .strip_prefix('"')
                .and_then(|item| item.strip_suffix('"'))
                .map(str::to_owned)
                .ok_or_else(|| format!("compiled contribution array `{key}` is malformed"))
        })
        .collect()
}

fn required_usize(
    fields: &BTreeMap<String, String>,
    key: &str,
    subject: &str,
) -> Result<usize, String> {
    fields
        .get(key)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("{subject} usize field `{key}` is missing"))
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

    const REQUEST: &str = "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id=input.values;provider_buffer_element_type=i64;provider_buffer_layout=tensor-contiguous;provider_buffer_shape=1x1;provider_buffer_row_stride_bytes=8;provider_buffer_byte_length=8;provider_buffer_payload_path=input.bin;provider_buffer_content_hash=0x0123456789abcdef;provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id=kernel.test;provider_kernel_operation=add-scalar-i64;provider_kernel_input_buffer=input.values;provider_kernel_output_buffer=output.values;provider_kernel_dispatch=1x1x1;provider_kernel_scalar_bindings=scalar:i64:1";

    fn fixture(dir: &Path) -> (String, ContributionRow) {
        let bytes = b".version 8.0\n.visible .entry project_main() { ret; }\n";
        fs::write(dir.join("kernel.ptx"), bytes).unwrap();
        let mut row = ContributionRow {
            index: 0,
            owner_package_id: "official.kernel".to_owned(),
            domain_family: "kernel".to_owned(),
            asset_id: "kernel.cuda.project.test".to_owned(),
            format: "ptx".to_owned(),
            lowering_target: "cuda.nvidia-gpu".to_owned(),
            target: "sm_80".to_owned(),
            path: "kernel.ptx".to_owned(),
            entries: vec!["project_main".to_owned()],
            byte_length: bytes.len(),
            content_hash: fnv1a64_hex(bytes),
            identity_hash: String::new(),
        };
        row.identity_hash = descriptor_identity_hash(&row);
        let root = identity_set_root_hash(&[row.clone()]);
        let table_hash = contribution_table_hash(&[row.clone()]);
        let table = format!(
            "protocol = \"{TABLE_CONTRACT}\"\ncontribution_contract = \"{CONTRIBUTION_CONTRACT}\"\nidentity_set_contract = \"{IDENTITY_SET_CONTRACT}\"\ncontribution_count = 1\nidentity_set_root_hash = \"{root}\"\ntable_hash = \"{table_hash}\"\n\n[[contribution]]\nindex = 0\nowner_package_id = \"{}\"\ndomain_family = \"{}\"\nasset_id = \"{}\"\nformat = \"{}\"\nlowering_target = \"{}\"\ntarget = \"{}\"\npath = \"{}\"\nentry_count = 1\nentries = [\"project_main\"]\nbyte_length = {}\ndigest_contract = \"{DIGEST_CONTRACT}\"\ncontent_hash = \"{}\"\nidentity_contract = \"{DESCRIPTOR_IDENTITY_CONTRACT}\"\nidentity_hash = \"{}\"\n",
            row.owner_package_id,
            row.domain_family,
            row.asset_id,
            row.format,
            row.lowering_target,
            row.target,
            row.path,
            row.byte_length,
            row.content_hash,
            row.identity_hash,
        );
        fs::write(dir.join(FILE_NAME), table).unwrap();
        let evidence = format!(
            "{REQUEST};provider_code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;provider_code_asset_id={};provider_code_asset_format={};provider_code_asset_target={};provider_code_asset_entry=project_main;provider_code_asset_path={};provider_code_asset_byte_length={};provider_code_asset_digest_contract={DIGEST_CONTRACT};provider_code_asset_content_hash={};provider_code_asset_contribution_selection_contract={SELECTION_CONTRACT};provider_code_asset_contribution_table_contract={TABLE_CONTRACT};provider_code_asset_contribution_table_hash={table_hash};provider_code_asset_contribution_identity_set_root_hash={root};provider_code_asset_contribution_index=0;provider_code_asset_contribution_owner_package_id={};provider_code_asset_contribution_domain_family={};provider_code_asset_contribution_lowering_target={};provider_code_asset_contribution_asset_id={};provider_code_asset_contribution_identity_contract={DESCRIPTOR_IDENTITY_CONTRACT};provider_code_asset_contribution_identity_hash={};compiled_code_asset_contribution_table_contract={TABLE_CONTRACT};compiled_code_asset_contribution_table_status=verified;compiled_code_asset_contribution_count=1;compiled_code_asset_identity_set_root_hash={root};compiled_code_asset_contribution_table_hash={table_hash}",
            row.asset_id,
            row.format,
            row.target,
            row.path,
            row.byte_length,
            row.content_hash,
            row.owner_package_id,
            row.domain_family,
            row.lowering_target,
            row.asset_id,
            row.identity_hash,
        );
        (evidence, row)
    }

    #[test]
    fn independently_blocks_selection_or_asset_drift_before_launch() {
        let dir = std::env::temp_dir().join(format!(
            "nsdb-compiled-contribution-selection-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let (evidence, row) = fixture(&dir);
        let collection =
            crate::provider_request::provider_request_collection_from_evidence(&evidence).unwrap();
        validate_compiled_contribution_selection(&dir, &evidence, &collection.requests).unwrap();
        fs::write(dir.join(&row.path), b"tampered").unwrap();
        assert!(
            validate_compiled_contribution_selection(&dir, &evidence, &collection.requests)
                .is_err()
        );
        let (_, _) = fixture(&dir);
        let drifted = evidence.replace(
            "provider_code_asset_contribution_table_hash=0x",
            "provider_code_asset_contribution_table_hash=0f",
        );
        assert!(
            validate_compiled_contribution_selection(&dir, &drifted, &collection.requests).is_err()
        );
        fs::remove_dir_all(dir).unwrap();
    }
}
