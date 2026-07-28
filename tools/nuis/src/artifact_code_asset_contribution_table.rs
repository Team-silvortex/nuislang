use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

const FILE_NAME: &str = "nuis.domain.code-asset-contributions.toml";
const TABLE_CONTRACT: &str = "nuis-domain-code-asset-contribution-table-v1";
const CONTRIBUTION_CONTRACT: &str = "nuis-nustar-code-asset-identity-contribution-v1";
const DESCRIPTOR_IDENTITY_CONTRACT: &str = "nuis-provider-code-asset-descriptor-identity-v1";
const IDENTITY_SET_CONTRACT: &str = "nuis-provider-code-asset-identity-set-v1";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";
const SELECTION_CONTRACT: &str = "nuis-provider-code-asset-contribution-selection-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifiedCodeAssetContributionTable {
    pub count: usize,
    pub identity_set_root_hash: String,
    pub table_hash: String,
    contributions: Vec<ContributionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SelectedCodeAssetContribution {
    pub index: usize,
    pub owner_package_id: String,
    pub domain_family: String,
    pub asset_id: String,
    pub format: String,
    pub lowering_target: String,
    pub target: String,
    pub path: String,
    pub entries: Vec<String>,
    pub byte_length: usize,
    pub content_hash: String,
    pub identity_hash: String,
    pub identity_set_root_hash: String,
    pub table_hash: String,
}

pub(super) fn verify_compiled_code_asset_contribution_table(
    output_dir: &Path,
) -> Result<Option<VerifiedCodeAssetContributionTable>, String> {
    let path = output_dir.join(FILE_NAME);
    if !path.exists() {
        return Ok(None);
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    verify_source(output_dir, &source).map(Some)
}

pub(super) fn render_verified_table_evidence(
    table: Option<&VerifiedCodeAssetContributionTable>,
) -> String {
    let Some(table) = table else {
        return String::new();
    };
    format!(
        ";compiled_code_asset_contribution_table_contract={TABLE_CONTRACT};compiled_code_asset_contribution_table_status=verified;compiled_code_asset_contribution_count={};compiled_code_asset_identity_set_root_hash={};compiled_code_asset_contribution_table_hash={}",
        table.count, table.identity_set_root_hash, table.table_hash
    )
}

pub(super) fn select_compiled_code_asset_contribution(
    output_dir: &Path,
    owner_package_id: &str,
    domain_family: &str,
    lowering_target: &str,
    format: &str,
    target: &str,
    entries: &[String],
) -> Result<Option<SelectedCodeAssetContribution>, String> {
    let Some(table) = verify_compiled_code_asset_contribution_table(output_dir)? else {
        return Ok(None);
    };
    let matches = table
        .contributions
        .iter()
        .filter(|row| {
            row.owner_package_id == owner_package_id
                && row.domain_family == domain_family
                && row.lowering_target == lowering_target
                && row.format == format
                && row.target == target
                && row.entries == entries
        })
        .collect::<Vec<_>>();
    let [row] = matches.as_slice() else {
        return Err(format!(
            "compiled code asset contribution selection expected one `{owner_package_id}` `{domain_family}` `{lowering_target}` row, found {}",
            matches.len()
        ));
    };
    Ok(Some(SelectedCodeAssetContribution {
        index: row.index,
        owner_package_id: row.owner_package_id.clone(),
        domain_family: row.domain_family.clone(),
        asset_id: row.asset_id.clone(),
        format: row.format.clone(),
        lowering_target: row.lowering_target.clone(),
        target: row.target.clone(),
        path: row.path.clone(),
        entries: row.entries.clone(),
        byte_length: row.byte_length,
        content_hash: row.content_hash.clone(),
        identity_hash: row.identity_hash.clone(),
        identity_set_root_hash: table.identity_set_root_hash.clone(),
        table_hash: table.table_hash.clone(),
    }))
}

pub(super) fn render_selected_contribution_evidence(
    selection: &SelectedCodeAssetContribution,
) -> String {
    format!(
        "provider_code_asset_contribution_selection_contract={SELECTION_CONTRACT};provider_code_asset_contribution_table_contract={TABLE_CONTRACT};provider_code_asset_contribution_table_hash={};provider_code_asset_contribution_identity_set_root_hash={};provider_code_asset_contribution_index={};provider_code_asset_contribution_owner_package_id={};provider_code_asset_contribution_domain_family={};provider_code_asset_contribution_lowering_target={};provider_code_asset_contribution_asset_id={};provider_code_asset_contribution_identity_contract={DESCRIPTOR_IDENTITY_CONTRACT};provider_code_asset_contribution_identity_hash={}",
        selection.table_hash,
        selection.identity_set_root_hash,
        selection.index,
        selection.owner_package_id,
        selection.domain_family,
        selection.lowering_target,
        selection.asset_id,
        selection.identity_hash,
    )
}

fn verify_source(
    output_dir: &Path,
    source: &str,
) -> Result<VerifiedCodeAssetContributionTable, String> {
    let mut sections = source.split("\n[[contribution]]\n");
    let header = parse_fields(
        sections
            .next()
            .ok_or_else(|| "code asset contribution table header is missing".to_owned())?,
    )?;
    require(&header, "protocol", TABLE_CONTRACT)?;
    require(&header, "contribution_contract", CONTRIBUTION_CONTRACT)?;
    require(&header, "identity_set_contract", IDENTITY_SET_CONTRACT)?;
    let count = usize_field(&header, "contribution_count")?;
    if !(1..=64).contains(&count) {
        return Err("code asset contribution count must be within 1..=64".to_owned());
    }
    let claimed_root = string_field(&header, "identity_set_root_hash")?;
    let claimed_table_hash = string_field(&header, "table_hash")?;
    let rows = sections.map(parse_row).collect::<Result<Vec<_>, _>>()?;
    if rows.len() != count {
        return Err("code asset contribution count does not match rows".to_owned());
    }
    validate_rows(output_dir, &rows)?;
    let expected_root = identity_set_root_hash(&rows);
    let expected_table_hash = contribution_table_hash(&rows);
    if claimed_root != expected_root || claimed_table_hash != expected_table_hash {
        return Err("code asset contribution table hash evidence mismatch".to_owned());
    }
    Ok(VerifiedCodeAssetContributionTable {
        count,
        identity_set_root_hash: expected_root,
        table_hash: expected_table_hash,
        contributions: rows,
    })
}

fn parse_row(source: &str) -> Result<ContributionRow, String> {
    let fields = parse_fields(source)?;
    require(&fields, "digest_contract", DIGEST_CONTRACT)?;
    require(&fields, "identity_contract", DESCRIPTOR_IDENTITY_CONTRACT)?;
    let entries = string_array_field(&fields, "entries")?;
    let row = ContributionRow {
        index: usize_field(&fields, "index")?,
        owner_package_id: string_field(&fields, "owner_package_id")?,
        domain_family: string_field(&fields, "domain_family")?,
        asset_id: string_field(&fields, "asset_id")?,
        format: string_field(&fields, "format")?,
        lowering_target: string_field(&fields, "lowering_target")?,
        target: string_field(&fields, "target")?,
        path: string_field(&fields, "path")?,
        entries,
        byte_length: usize_field(&fields, "byte_length")?,
        content_hash: string_field(&fields, "content_hash")?,
        identity_hash: string_field(&fields, "identity_hash")?,
    };
    if usize_field(&fields, "entry_count")? != row.entries.len() {
        return Err(format!(
            "code asset contribution `{}` entry count mismatch",
            row.asset_id
        ));
    }
    Ok(row)
}

fn validate_rows(output_dir: &Path, rows: &[ContributionRow]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut ordered = rows.to_vec();
    ordered.sort_by(|lhs, rhs| {
        lhs.domain_family
            .cmp(&rhs.domain_family)
            .then_with(|| lhs.owner_package_id.cmp(&rhs.owner_package_id))
            .then_with(|| lhs.asset_id.cmp(&rhs.asset_id))
    });
    if ordered != rows {
        return Err("code asset contribution rows are not in canonical order".to_owned());
    }
    for (expected_index, row) in rows.iter().enumerate() {
        if row.index != expected_index
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
            || !valid_hash(&row.content_hash)
            || !valid_hash(&row.identity_hash)
            || !relative_path_is_valid(&row.path)
        {
            return Err(format!(
                "code asset contribution row `{}` is invalid",
                row.asset_id
            ));
        }
        let bytes = fs::read(output_dir.join(&row.path)).map_err(|error| {
            format!(
                "failed to read code asset contribution `{}`: {error}",
                row.path
            )
        })?;
        let expected_identity = descriptor_identity_hash(row);
        if bytes.len() != row.byte_length
            || fnv1a64_hex(&bytes) != row.content_hash
            || expected_identity != row.identity_hash
        {
            return Err(format!(
                "code asset contribution `{}` byte or identity evidence mismatch",
                row.asset_id
            ));
        }
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

fn parse_fields(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut fields = BTreeMap::new();
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("malformed code asset contribution line `{line}`"))?;
        let key = key.trim().to_owned();
        if fields
            .insert(key.clone(), value.trim().to_owned())
            .is_some()
        {
            return Err(format!(
                "code asset contribution field `{key}` is duplicated"
            ));
        }
    }
    Ok(fields)
}

fn require(fields: &BTreeMap<String, String>, key: &str, expected: &str) -> Result<(), String> {
    (string_field(fields, key)? == expected)
        .then_some(())
        .ok_or_else(|| format!("code asset contribution field `{key}` is unsupported"))
}

fn string_field(fields: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    fields
        .get(key)
        .and_then(|value| value.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("code asset contribution string field `{key}` is missing"))
}

fn usize_field(fields: &BTreeMap<String, String>, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("code asset contribution usize field `{key}` is missing"))
}

fn string_array_field(fields: &BTreeMap<String, String>, key: &str) -> Result<Vec<String>, String> {
    let value = fields
        .get(key)
        .and_then(|value| value.strip_prefix('[')?.strip_suffix(']'))
        .ok_or_else(|| format!("code asset contribution array field `{key}` is missing"))?;
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|item| {
            item.trim()
                .strip_prefix('"')
                .and_then(|item| item.strip_suffix('"'))
                .map(str::to_owned)
                .ok_or_else(|| format!("code asset contribution array `{key}` is malformed"))
        })
        .collect()
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

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
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

    fn fixture(output_dir: &Path) -> String {
        let bytes = b"kernel void main0() {}";
        fs::write(output_dir.join("shader.ir"), bytes).unwrap();
        let mut row = ContributionRow {
            index: 0,
            owner_package_id: "official.shader".to_owned(),
            domain_family: "shader".to_owned(),
            asset_id: "shader.asset".to_owned(),
            format: "nuis-shader-ir-sidecar".to_owned(),
            lowering_target: "metal.apple-silicon-gpu".to_owned(),
            target: "metal.apple-silicon-gpu".to_owned(),
            path: "shader.ir".to_owned(),
            entries: vec!["main0".to_owned()],
            byte_length: bytes.len(),
            content_hash: fnv1a64_hex(bytes),
            identity_hash: String::new(),
        };
        row.identity_hash = descriptor_identity_hash(&row);
        format!(
            "protocol = \"{TABLE_CONTRACT}\"\ncontribution_contract = \"{CONTRIBUTION_CONTRACT}\"\nidentity_set_contract = \"{IDENTITY_SET_CONTRACT}\"\ncontribution_count = 1\nidentity_set_root_hash = \"{}\"\ntable_hash = \"{}\"\n\n[[contribution]]\nindex = 0\nowner_package_id = \"{}\"\ndomain_family = \"{}\"\nasset_id = \"{}\"\nformat = \"{}\"\nlowering_target = \"{}\"\ntarget = \"{}\"\npath = \"{}\"\nentry_count = 1\nentries = [\"main0\"]\nbyte_length = {}\ndigest_contract = \"{DIGEST_CONTRACT}\"\ncontent_hash = \"{}\"\nidentity_contract = \"{DESCRIPTOR_IDENTITY_CONTRACT}\"\nidentity_hash = \"{}\"\n",
            identity_set_root_hash(&[row.clone()]),
            contribution_table_hash(&[row.clone()]),
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
        )
    }

    #[test]
    fn independently_verifies_table_and_asset_bytes() {
        let dir = std::env::temp_dir().join(format!(
            "nuis-code-asset-table-verifier-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let source = fixture(&dir);
        let verified = verify_source(&dir, &source).unwrap();
        assert_eq!(verified.count, 1);
        assert!(verified.identity_set_root_hash.starts_with("0x"));
        let evidence = render_verified_table_evidence(Some(&verified));
        assert!(evidence.contains(
            "compiled_code_asset_contribution_table_contract=nuis-domain-code-asset-contribution-table-v1"
        ));
        assert!(evidence.contains("compiled_code_asset_contribution_table_status=verified"));
        assert!(evidence.contains("compiled_code_asset_contribution_count=1"));
        assert!(evidence.contains(&verified.identity_set_root_hash));
        assert!(evidence.contains(&verified.table_hash));
        fs::write(dir.join(FILE_NAME), &source).unwrap();
        let selection = select_compiled_code_asset_contribution(
            &dir,
            "official.shader",
            "shader",
            "metal.apple-silicon-gpu",
            "nuis-shader-ir-sidecar",
            "metal.apple-silicon-gpu",
            &["main0".to_owned()],
        )
        .unwrap()
        .unwrap();
        assert_eq!(selection.asset_id, "shader.asset");
        let selection_evidence = render_selected_contribution_evidence(&selection);
        assert!(selection_evidence.contains(
            "provider_code_asset_contribution_selection_contract=nuis-provider-code-asset-contribution-selection-v1"
        ));
        assert!(
            selection_evidence.contains("provider_code_asset_contribution_asset_id=shader.asset")
        );
        assert!(select_compiled_code_asset_contribution(
            &dir,
            "official.shader",
            "shader",
            "metal.apple-silicon-gpu",
            "nuis-shader-ir-sidecar",
            "metal.apple-silicon-gpu",
            &["missing".to_owned()],
        )
        .is_err());
        fs::write(dir.join("shader.ir"), b"tampered").unwrap();
        assert!(verify_source(&dir, &source).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
