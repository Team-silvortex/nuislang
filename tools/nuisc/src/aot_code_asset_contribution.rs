use std::{
    collections::BTreeSet,
    path::{Component, Path},
};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::{
    aot_encoding::fnv1a64_hex,
    aot_toml::{escape_toml_string, render_string_array},
    kernel_codegen_table::KernelYirCodegenTable,
    registry::NustarCodeAssetRegistration,
    shader_render_codegen_table::ShaderRenderCodeAsset,
};

pub(crate) const DOMAIN_CODE_ASSET_CONTRIBUTION_TABLE_CONTRACT: &str =
    "nuis-domain-code-asset-contribution-table-v1";
const CONTRIBUTION_CONTRACT: &str = "nuis-nustar-code-asset-identity-contribution-v1";
const DESCRIPTOR_IDENTITY_CONTRACT: &str = "nuis-provider-code-asset-descriptor-identity-v1";
const IDENTITY_SET_CONTRACT: &str = "nuis-provider-code-asset-identity-set-v1";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainCodeAssetContribution {
    pub(crate) owner_package_id: String,
    pub(crate) domain_family: String,
    pub(crate) asset_id: String,
    pub(crate) format: String,
    pub(crate) lowering_target: String,
    pub(crate) target: String,
    pub(crate) path: String,
    pub(crate) entries: Vec<String>,
    pub(crate) byte_length: usize,
    pub(crate) content_hash: String,
    pub(crate) identity_hash: String,
}

pub(crate) fn shader_sidecar_contribution(
    unit: &BuildManifestDomainBuildUnit,
    path: &Path,
    bytes: &[u8],
) -> Result<Option<DomainCodeAssetContribution>, String> {
    if unit.domain_family != "shader" {
        return Ok(None);
    }
    let target = unit
        .selected_lowering_target
        .as_deref()
        .ok_or_else(|| "Shader code asset contribution is missing a lowering target".to_owned())?;
    let entry = shader_primary_entry(target);
    contribution(
        unit,
        format!("shader.{target}.ir.{}", &fnv1a64_hex(bytes)[2..]),
        "nuis-shader-ir-sidecar",
        target,
        path,
        vec![entry.to_owned()],
        bytes,
    )
    .map(Some)
}

pub(crate) fn shader_render_asset_contribution(
    unit: &BuildManifestDomainBuildUnit,
    asset: &ShaderRenderCodeAsset,
    path: &Path,
) -> Result<DomainCodeAssetContribution, String> {
    if unit.domain_family != "shader"
        || unit.selected_lowering_target.as_deref() != Some(asset.target.as_str())
    {
        return Err(format!(
            "generated Shader render asset `{}` does not belong to AOT unit `{}`",
            asset.asset_id, unit.package_id
        ));
    }
    contribution(
        unit,
        asset.asset_id.clone(),
        asset.format,
        &asset.target,
        path,
        asset.entries.clone(),
        asset.source.as_bytes(),
    )
}

pub(crate) fn kernel_asset_contribution(
    unit: &BuildManifestDomainBuildUnit,
    path: &Path,
    bytes: &[u8],
    table: Option<&KernelYirCodegenTable>,
) -> Result<Option<DomainCodeAssetContribution>, String> {
    if unit.domain_family != "kernel" {
        return Ok(None);
    }
    let target = unit
        .selected_lowering_target
        .as_deref()
        .ok_or_else(|| "Kernel code asset contribution is missing a lowering target".to_owned())?;
    let Some(asset) = crate::kernel_code_asset::select_kernel_code_asset(target) else {
        return Ok(None);
    };
    let entries = table
        .map(|table| {
            table
                .functions
                .iter()
                .map(|function| function.entry.clone())
                .collect::<Vec<_>>()
        })
        .filter(|entries| !entries.is_empty())
        .unwrap_or_else(|| {
            asset
                .visible_entries
                .iter()
                .map(|entry| (*entry).to_owned())
                .collect()
        });
    let asset_id = table
        .and_then(KernelYirCodegenTable::compiled_project_code_asset_id)
        .unwrap_or_else(|| asset.id.to_owned());
    contribution(
        unit,
        asset_id,
        asset.format,
        asset.target,
        path,
        entries,
        bytes,
    )
    .map(Some)
}

pub(crate) fn registered_asset_contribution(
    unit: &BuildManifestDomainBuildUnit,
    asset: &NustarCodeAssetRegistration,
    path: &Path,
) -> Result<DomainCodeAssetContribution, String> {
    if unit.package_id != asset.package_id
        || unit.domain_family != asset.domain_family
        || unit.selected_lowering_target.as_deref() != Some(asset.lowering_target.as_str())
    {
        return Err(format!(
            "registered code asset `{}` does not belong to AOT unit `{}`",
            asset.asset_id, unit.package_id
        ));
    }
    contribution(
        unit,
        asset.asset_id.clone(),
        &asset.format,
        &asset.target,
        path,
        vec![asset.entry.clone()],
        &asset.bytes,
    )
}

pub(crate) fn required_registered_asset_contribution(
    asset: &NustarCodeAssetRegistration,
    path: &Path,
) -> Result<DomainCodeAssetContribution, String> {
    let path = relative_file_name(path)?;
    let content_hash = fnv1a64_hex(&asset.bytes);
    let entries = vec![asset.entry.clone()];
    let identity_hash = descriptor_identity_hash(
        &asset.asset_id,
        &asset.format,
        &asset.target,
        &path,
        asset.bytes.len(),
        &content_hash,
        &entries,
    );
    Ok(DomainCodeAssetContribution {
        owner_package_id: asset.package_id.clone(),
        domain_family: asset.domain_family.clone(),
        asset_id: asset.asset_id.clone(),
        format: asset.format.clone(),
        lowering_target: asset.lowering_target.clone(),
        target: asset.target.clone(),
        path,
        entries,
        byte_length: asset.bytes.len(),
        content_hash,
        identity_hash,
    })
}

pub(crate) fn render_code_asset_contribution_table(
    contributions: &[DomainCodeAssetContribution],
) -> Result<String, String> {
    if contributions.is_empty() || contributions.len() > 64 {
        return Err("domain code asset contribution count must be within 1..=64".to_owned());
    }
    let mut ordered = contributions.to_vec();
    ordered.sort_by(|lhs, rhs| {
        lhs.domain_family
            .cmp(&rhs.domain_family)
            .then_with(|| lhs.owner_package_id.cmp(&rhs.owner_package_id))
            .then_with(|| lhs.asset_id.cmp(&rhs.asset_id))
    });
    validate_contributions(&ordered)?;
    let set_root_hash = identity_set_root_hash(&ordered);
    let table_hash = contribution_table_hash(&ordered);
    let mut out = format!(
        "protocol = \"{DOMAIN_CODE_ASSET_CONTRIBUTION_TABLE_CONTRACT}\"\ncontribution_contract = \"{CONTRIBUTION_CONTRACT}\"\nidentity_set_contract = \"{IDENTITY_SET_CONTRACT}\"\ncontribution_count = {}\nidentity_set_root_hash = \"{set_root_hash}\"\ntable_hash = \"{table_hash}\"\n",
        ordered.len()
    );
    for (index, row) in ordered.iter().enumerate() {
        out.push_str("\n[[contribution]]\n");
        out.push_str(&format!(
            "index = {index}\nowner_package_id = \"{}\"\ndomain_family = \"{}\"\nasset_id = \"{}\"\nformat = \"{}\"\nlowering_target = \"{}\"\ntarget = \"{}\"\npath = \"{}\"\nentry_count = {}\nentries = {}\nbyte_length = {}\ndigest_contract = \"{DIGEST_CONTRACT}\"\ncontent_hash = \"{}\"\nidentity_contract = \"{DESCRIPTOR_IDENTITY_CONTRACT}\"\nidentity_hash = \"{}\"\n",
            escape_toml_string(&row.owner_package_id),
            escape_toml_string(&row.domain_family),
            escape_toml_string(&row.asset_id),
            escape_toml_string(&row.format),
            escape_toml_string(&row.lowering_target),
            escape_toml_string(&row.target),
            escape_toml_string(&row.path),
            row.entries.len(),
            render_string_array(&row.entries),
            row.byte_length,
            row.content_hash,
            row.identity_hash,
        ));
    }
    Ok(out)
}

fn contribution(
    unit: &BuildManifestDomainBuildUnit,
    asset_id: String,
    format: &str,
    target: &str,
    path: &Path,
    entries: Vec<String>,
    bytes: &[u8],
) -> Result<DomainCodeAssetContribution, String> {
    let path = relative_file_name(path)?;
    let content_hash = fnv1a64_hex(bytes);
    let identity_hash = descriptor_identity_hash(
        &asset_id,
        format,
        target,
        &path,
        bytes.len(),
        &content_hash,
        &entries,
    );
    Ok(DomainCodeAssetContribution {
        owner_package_id: unit.package_id.clone(),
        domain_family: unit.domain_family.clone(),
        asset_id,
        format: format.to_owned(),
        lowering_target: unit
            .selected_lowering_target
            .clone()
            .ok_or_else(|| "code asset contribution has no lowering target".to_owned())?,
        target: target.to_owned(),
        path,
        entries,
        byte_length: bytes.len(),
        content_hash,
        identity_hash,
    })
}

fn validate_contributions(contributions: &[DomainCodeAssetContribution]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for row in contributions {
        if !ids.insert(row.asset_id.as_str()) || !paths.insert(row.path.as_str()) {
            return Err("domain code asset contributions have duplicate asset or path".to_owned());
        }
        if row.owner_package_id.is_empty()
            || row.domain_family.is_empty()
            || row.lowering_target.is_empty()
            || row.entries.is_empty()
            || row.entries.iter().any(|entry| entry.is_empty())
            || row.byte_length == 0
            || !valid_hash(&row.content_hash)
            || !valid_hash(&row.identity_hash)
            || !relative_path_is_valid(&row.path)
        {
            return Err(format!(
                "domain code asset contribution `{}` is invalid",
                row.asset_id
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn descriptor_identity_hash(
    asset_id: &str,
    format: &str,
    target: &str,
    path: &str,
    byte_length: usize,
    content_hash: &str,
    entries: &[String],
) -> String {
    fnv1a64_hex(
        format!(
            "{DESCRIPTOR_IDENTITY_CONTRACT}\n{asset_id}\n{format}\n{target}\n{path}\n{byte_length}\n{DIGEST_CONTRACT}\n{content_hash}\n{}\n{}",
            entries.len(),
            entries.join("\n")
        )
        .as_bytes(),
    )
}

fn identity_set_root_hash(rows: &[DomainCodeAssetContribution]) -> String {
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

fn contribution_table_hash(rows: &[DomainCodeAssetContribution]) -> String {
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
    fnv1a64_hex(
        format!(
            "{DOMAIN_CODE_ASSET_CONTRIBUTION_TABLE_CONTRACT}\n{}\n{material}",
            rows.len()
        )
        .as_bytes(),
    )
}

fn shader_primary_entry(target: &str) -> &'static str {
    match target {
        "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu" => "main0",
        "cpu-fallback.cpu-host" => "shade_stub",
        _ => "main",
    }
}

fn relative_file_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "code asset path `{}` has no UTF-8 file name",
                path.display()
            )
        })
}

fn relative_path_is_valid(value: &str) -> bool {
    let path = Path::new(value);
    !value.contains(['\\', ':'])
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(domain: &str, id: &str, path: &str) -> DomainCodeAssetContribution {
        let entries = vec![format!("{domain}_main")];
        let content_hash = fnv1a64_hex(id.as_bytes());
        DomainCodeAssetContribution {
            owner_package_id: format!("official.{domain}"),
            domain_family: domain.to_owned(),
            asset_id: id.to_owned(),
            format: format!("{domain}-ir"),
            lowering_target: format!("{domain}.lowering"),
            target: format!("{domain}.target"),
            path: path.to_owned(),
            entries: entries.clone(),
            byte_length: id.len(),
            identity_hash: descriptor_identity_hash(
                id,
                &format!("{domain}-ir"),
                &format!("{domain}.target"),
                path,
                id.len(),
                &content_hash,
                &entries,
            ),
            content_hash,
        }
    }

    #[test]
    fn table_is_deterministic_and_provider_neutral() {
        let shader = row("shader", "shader.asset", "shader.ir");
        let kernel = row("kernel", "kernel.asset", "kernel.ptx");
        let first =
            render_code_asset_contribution_table(&[shader.clone(), kernel.clone()]).unwrap();
        let second = render_code_asset_contribution_table(&[kernel, shader]).unwrap();
        assert_eq!(first, second);
        assert!(first.contains("contribution_count = 2"));
        assert!(first.contains("owner_package_id = \"official.kernel\""));
        assert!(first.contains("owner_package_id = \"official.shader\""));
        assert!(first.contains("identity_set_root_hash = \"0x"));
        assert!(first.contains("table_hash = \"0x"));
    }

    #[test]
    fn table_rejects_duplicate_paths() {
        assert!(render_code_asset_contribution_table(&[
            row("shader", "shader.first", "same.ir"),
            row("kernel", "kernel.second", "same.ir"),
        ])
        .is_err());
    }
}
