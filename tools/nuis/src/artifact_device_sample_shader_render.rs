use crate::{
    artifact_code_asset_contribution_table::{
        render_selected_contribution_evidence, render_selected_contribution_set_evidence,
        select_compiled_code_asset_contribution, SelectedCodeAssetContribution,
    },
    artifact_device_sample_registration::DeviceSampleInputRegistration,
    artifact_device_sample_shader_common::fnv1a64_hex,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

const PACKAGE_ID: &str = "official.shader";
const PROVIDER_FAMILY: &str = "metal:apple-silicon-gpu";
const REGISTRATION_ID: &str = "official.shader.project-render";
const METADATA_SELECTOR: &str = "official.shader:provider-sample=project-render";
const PROJECTION_CONTRACT: &str = "nuis-shader-render-provider-projection-v1";
const TABLE_FILE_NAME: &str = "nuis.domain.shader.render-codegen-table.toml";
const TABLE_CONTRACT: &str = "nuis-shader-render-codegen-table-v1";
const ASSET_CONTRACT: &str = "nuis-shader-render-code-asset-v1";
const PASS_CONTRACT: &str = "nuis-shader-render-pass-projection-v1";
const DESCRIPTOR_CONTRACT: &str = "nuis-provider-code-asset-descriptor-v2";
const DESCRIPTOR_IDENTITY_CONTRACT: &str = "nuis-provider-code-asset-descriptor-identity-v1";
const IDENTITY_SET_CONTRACT: &str = "nuis-provider-code-asset-identity-set-v1";
const CONTRIBUTION_CONTRACT: &str = "nuis-nustar-code-asset-identity-contribution-v1";
const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderAsset {
    asset_id: String,
    file_name: String,
    format: String,
    target: String,
    entries: Vec<String>,
    byte_length: usize,
    content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderPass {
    pass_node: String,
    module_node: String,
    asset_id: String,
    target_format: String,
    width: usize,
    height: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct RenderTable {
    source_yir_version: String,
    source_fnv1a64: String,
    lowering_target: String,
    assets: Vec<RenderAsset>,
    passes: Vec<RenderPass>,
}

pub(crate) fn registration() -> DeviceSampleInputRegistration {
    DeviceSampleInputRegistration {
        package_id: PACKAGE_ID,
        registration_id: REGISTRATION_ID,
        provider_family: PROVIDER_FAMILY,
        supports: supports_metal,
        metadata_selector: Some(selects_project_render),
        enrich_evidence: projection_marker,
        resolve_evidence: Some(resolve_project_render_evidence),
        persist_payloads: persist_project_render_payloads,
    }
}

fn supports_metal(backend_family: &str, target_device: &str) -> bool {
    backend_family == "metal" && target_device == "apple-silicon-gpu"
}

fn selects_project_render(base: &str) -> bool {
    base.split(';')
        .filter_map(|field| field.split_once('='))
        .any(|(key, value)| {
            key.starts_with("artifact_provider_metadata_") && value == METADATA_SELECTOR
        })
}

fn projection_marker(_: &str) -> String {
    format!("provider_shader_render_projection_contract={PROJECTION_CONTRACT}")
}

fn persist_project_render_payloads(output_dir: &Path, evidence: &[&str]) -> Result<(), String> {
    if evidence.iter().any(|item| registration_selected(item)) {
        load_verified_render_table(output_dir)?;
    }
    Ok(())
}

pub(crate) fn resolve_project_render_evidence(
    output_dir: &Path,
    evidence: &str,
) -> Result<String, String> {
    if !registration_selected(evidence) {
        return Ok(evidence.to_owned());
    }
    if evidence.split(';').any(|field| {
        field == "provider_request_collection_contract=nuis-provider-request-collection-v1"
    }) {
        return Ok(evidence.to_owned());
    }
    let table = load_verified_render_table(output_dir)?;
    let selections = select_render_assets(output_dir, &table)?;
    let mut requests = Vec::with_capacity(table.passes.len());
    for (index, pass) in table.passes.iter().enumerate() {
        let asset = table
            .assets
            .iter()
            .find(|asset| asset.asset_id == pass.asset_id)
            .ok_or_else(|| format!("render pass `{}` lost its code asset", pass.pass_node))?;
        let selection = selections
            .get(&asset.asset_id)
            .ok_or_else(|| format!("render asset `{}` lost its contribution", asset.asset_id))?;
        requests.push(render_request(index, table.passes.len(), pass, selection)?);
    }
    let ordered_selections = ordered_request_selections(&table, &selections)?;
    let selection_evidence = if ordered_selections.len() == 1 {
        render_selected_contribution_evidence(ordered_selections[0])
    } else {
        render_selected_contribution_set_evidence(
            &ordered_selections
                .iter()
                .map(|selection| (*selection).clone())
                .collect::<Vec<_>>(),
        )?
    };
    Ok(format!(
        "{evidence};provider_shader_render_projection_status=verified;provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count={};{};{};{}",
        requests.len(),
        requests.join(";"),
        render_identity_evidence(&ordered_selections),
        selection_evidence,
    ))
}

fn registration_selected(evidence: &str) -> bool {
    evidence
        .split(';')
        .any(|field| field == format!("provider_sample_registration_id={REGISTRATION_ID}"))
}

fn select_render_assets(
    output_dir: &Path,
    table: &RenderTable,
) -> Result<BTreeMap<String, SelectedCodeAssetContribution>, String> {
    let mut selected = BTreeMap::new();
    for asset in &table.assets {
        let selection = select_compiled_code_asset_contribution(
            output_dir,
            PACKAGE_ID,
            "shader",
            &table.lowering_target,
            &asset.format,
            &asset.target,
            &asset.entries,
        )?
        .ok_or_else(|| "compiled code asset contribution table is missing".to_owned())?;
        if selection.asset_id != asset.asset_id
            || selection.path != asset.file_name
            || selection.byte_length != asset.byte_length
            || selection.content_hash != asset.content_hash
        {
            return Err(format!(
                "render code asset `{}` drifted from its compiled contribution",
                asset.asset_id
            ));
        }
        selected.insert(asset.asset_id.clone(), selection);
    }
    Ok(selected)
}

fn ordered_request_selections<'a>(
    table: &'a RenderTable,
    selections: &'a BTreeMap<String, SelectedCodeAssetContribution>,
) -> Result<Vec<&'a SelectedCodeAssetContribution>, String> {
    let mut seen = BTreeSet::new();
    table
        .passes
        .iter()
        .filter(|pass| seen.insert(pass.asset_id.as_str()))
        .map(|pass| {
            selections
                .get(&pass.asset_id)
                .ok_or_else(|| format!("render pass `{}` has no contribution", pass.pass_node))
        })
        .collect()
}

fn render_request(
    index: usize,
    request_count: usize,
    pass: &RenderPass,
    asset: &SelectedCodeAssetContribution,
) -> Result<String, String> {
    let prefix = format!("provider_request_{index}_");
    let [vertex_entry, fragment_entry] = asset.entries.as_slice() else {
        return Err(format!(
            "render asset `{}` must expose one vertex and one fragment entry",
            asset.asset_id
        ));
    };
    let row_stride = pass
        .width
        .checked_mul(4)
        .ok_or_else(|| format!("render pass `{}` row stride overflow", pass.pass_node))?;
    let output_length = row_stride
        .checked_mul(pass.height)
        .ok_or_else(|| format!("render pass `{}` output length overflow", pass.pass_node))?;
    let output = if request_count == 1 {
        "output.frame".to_owned()
    } else {
        format!("output.frame.{index}")
    };
    Ok(format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.shader.module;{prefix}buffer_element_type=u8;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape={asset_bytes};{prefix}buffer_row_stride_bytes={asset_bytes};{prefix}buffer_byte_length={asset_bytes};{prefix}buffer_payload_path={asset_path};{prefix}buffer_content_hash={asset_hash};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id=shader.render.rgba8.{index};{prefix}kernel_operation=render-rgba8;{prefix}kernel_input_buffer=input.shader.module;{prefix}kernel_output_buffer={output};{prefix}kernel_dispatch={width}x{height}x1;{prefix}kernel_scalar_bindings=fragment_entry:symbol:{fragment_entry};{prefix}code_asset_descriptor_contract={DESCRIPTOR_CONTRACT};{prefix}code_asset_id={asset_id};{prefix}code_asset_format={asset_format};{prefix}code_asset_target={asset_target};{prefix}code_asset_entry={vertex_entry};{prefix}code_asset_entry_count=2;{prefix}code_asset_entries={vertex_entry},{fragment_entry};{prefix}code_asset_path={asset_path};{prefix}code_asset_byte_length={asset_bytes};{prefix}code_asset_digest_contract={DIGEST_CONTRACT};{prefix}code_asset_content_hash={asset_hash};{prefix}output_binding_contract=nuis-provider-output-binding-v2;{prefix}output_binding_count=1;{prefix}output_binding_0_role={output};{prefix}output_binding_0_buffer={output};{prefix}output_binding_0_element_type=u8;{prefix}output_binding_0_layout=image-2d-row-major:pixel-format=rgba8;{prefix}output_binding_0_shape={width}x{height};{prefix}output_binding_0_row_stride_bytes={row_stride};{prefix}output_binding_0_byte_length={output_length};{prefix}output_binding_0_comparison_id=none;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0;{prefix}input_binding_contract=nuis-provider-input-binding-v2;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.shader.module;{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_element_type=u8;{prefix}input_binding_0_layout=tensor-contiguous;{prefix}input_binding_0_shape={asset_bytes};{prefix}input_binding_0_row_stride_bytes={asset_bytes};{prefix}input_binding_0_byte_length={asset_bytes};{prefix}input_binding_0_content_hash={asset_hash};{prefix}input_binding_0_payload_path={asset_path};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none;{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family={PROVIDER_FAMILY};{prefix}adapter_binding_execution_requirement=real-device",
        asset_bytes = asset.byte_length,
        asset_path = asset.path,
        asset_hash = asset.content_hash,
        width = pass.width,
        height = pass.height,
        asset_id = asset.asset_id,
        asset_format = asset.format,
        asset_target = asset.target,
    ))
}

fn render_identity_evidence(selections: &[&SelectedCodeAssetContribution]) -> String {
    let material = selections
        .iter()
        .map(|selection| {
            format!(
                "{}\n{DESCRIPTOR_IDENTITY_CONTRACT}\n{}",
                selection.asset_id, selection.identity_hash
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let root_hash = fnv1a64_hex(
        format!("{IDENTITY_SET_CONTRACT}\n{}\n{material}", selections.len()).as_bytes(),
    );
    let mut out = format!(
        "provider_code_asset_identity_set_contract={IDENTITY_SET_CONTRACT};provider_code_asset_identity_set_count={};provider_code_asset_identity_set_root_hash={root_hash}",
        selections.len()
    );
    for (index, selection) in selections.iter().enumerate() {
        out.push_str(&format!(
            ";provider_code_asset_identity_item_{index}_asset_id={};provider_code_asset_identity_item_{index}_contract={DESCRIPTOR_IDENTITY_CONTRACT};provider_code_asset_identity_item_{index}_hash={};provider_code_asset_identity_item_{index}_contribution_contract={CONTRIBUTION_CONTRACT};provider_code_asset_identity_item_{index}_owner_package_id={PACKAGE_ID};provider_code_asset_identity_item_{index}_provider_family={PROVIDER_FAMILY}",
            selection.asset_id, selection.identity_hash
        ));
    }
    out
}

fn load_verified_render_table(output_dir: &Path) -> Result<RenderTable, String> {
    let path = output_dir.join(TABLE_FILE_NAME);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let table = parse_render_table(&source)?;
    validate_render_table(output_dir, &table)?;
    Ok(table)
}

fn parse_render_table(source: &str) -> Result<RenderTable, String> {
    let (header, assets, passes) = parse_sections(source)?;
    require(&header, "schema", TABLE_CONTRACT)?;
    let asset_count = usize_field(&header, "asset_count")?;
    let pass_count = usize_field(&header, "pass_count")?;
    if assets.len() != asset_count || passes.len() != pass_count {
        return Err("Shader render codegen table count mismatch".to_owned());
    }
    let assets = assets
        .iter()
        .map(parse_asset)
        .collect::<Result<Vec<_>, _>>()?;
    let passes = passes
        .iter()
        .map(parse_pass)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RenderTable {
        source_yir_version: string_field(&header, "source_yir_version")?,
        source_fnv1a64: string_field(&header, "source_fnv1a64")?,
        lowering_target: string_field(&header, "lowering_target")?,
        assets,
        passes,
    })
}

type Fields = BTreeMap<String, String>;

fn parse_sections(source: &str) -> Result<(Fields, Vec<Fields>, Vec<Fields>), String> {
    let mut header = Fields::new();
    let mut assets = Vec::<Fields>::new();
    let mut passes = Vec::<Fields>::new();
    enum Section {
        Header,
        Asset(usize),
        Pass(usize),
    }
    let mut section = Section::Header;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[asset]]" {
            assets.push(Fields::new());
            section = Section::Asset(assets.len() - 1);
            continue;
        }
        if line == "[[pass]]" {
            passes.push(Fields::new());
            section = Section::Pass(passes.len() - 1);
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .map(|(key, value)| (key.trim(), value.trim()))
            .ok_or_else(|| format!("malformed Shader render table line `{line}`"))?;
        let target = match section {
            Section::Header => &mut header,
            Section::Asset(index) => &mut assets[index],
            Section::Pass(index) => &mut passes[index],
        };
        if key.is_empty() || target.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate Shader render table field `{key}`"));
        }
    }
    Ok((header, assets, passes))
}

fn parse_asset(fields: &Fields) -> Result<RenderAsset, String> {
    require(fields, "contract", ASSET_CONTRACT)?;
    Ok(RenderAsset {
        asset_id: string_field(fields, "asset_id")?,
        file_name: string_field(fields, "file_name")?,
        format: string_field(fields, "format")?,
        target: string_field(fields, "target")?,
        entries: string_array_field(fields, "entries")?,
        byte_length: usize_field(fields, "byte_length")?,
        content_hash: string_field(fields, "content_hash")?,
    })
}

fn parse_pass(fields: &Fields) -> Result<RenderPass, String> {
    require(fields, "contract", PASS_CONTRACT)?;
    Ok(RenderPass {
        pass_node: string_field(fields, "pass_node")?,
        module_node: string_field(fields, "module_node")?,
        asset_id: string_field(fields, "asset_id")?,
        target_format: string_field(fields, "target_format")?,
        width: usize_field(fields, "width")?,
        height: usize_field(fields, "height")?,
    })
}

fn validate_render_table(output_dir: &Path, table: &RenderTable) -> Result<(), String> {
    if !(1..=64).contains(&table.assets.len())
        || !(1..=64).contains(&table.passes.len())
        || !token_is_valid(&table.source_yir_version)
        || !valid_hash(&table.source_fnv1a64)
        || !matches!(
            table.lowering_target.as_str(),
            "metal.apple-silicon-gpu" | "metal.mac-discrete-or-integrated-gpu"
        )
    {
        return Err("Shader render codegen table header is invalid".to_owned());
    }
    verify_source_yir(output_dir, &table.source_yir_version, &table.source_fnv1a64)?;
    let mut asset_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for asset in &table.assets {
        if !asset_ids.insert(asset.asset_id.as_str())
            || !paths.insert(asset.file_name.as_str())
            || !token_is_valid(&asset.asset_id)
            || asset.format != "metal-source"
            || asset.target != table.lowering_target
            || asset.entries.len() != 2
            || asset.entries.iter().any(|entry| !symbol_is_valid(entry))
            || asset.entries[0] == asset.entries[1]
            || !relative_path_is_valid(&asset.file_name)
            || asset.byte_length == 0
            || !valid_hash(&asset.content_hash)
        {
            return Err(format!(
                "Shader render asset `{}` is invalid",
                asset.asset_id
            ));
        }
        let bytes = fs::read(output_dir.join(&asset.file_name)).map_err(|error| {
            format!(
                "failed to read Shader render asset `{}`: {error}",
                asset.file_name
            )
        })?;
        if bytes.len() != asset.byte_length || fnv1a64_hex(&bytes) != asset.content_hash {
            return Err(format!(
                "Shader render asset `{}` byte identity mismatch",
                asset.asset_id
            ));
        }
    }
    let mut pass_nodes = BTreeSet::new();
    let mut referenced_assets = BTreeSet::new();
    for pass in &table.passes {
        if !pass_nodes.insert(pass.pass_node.as_str())
            || pass.module_node.is_empty()
            || !asset_ids.contains(pass.asset_id.as_str())
            || pass.target_format != "rgba8_unorm"
            || pass.width == 0
            || pass.height == 0
            || pass
                .width
                .checked_mul(4)
                .and_then(|row| row.checked_mul(pass.height))
                .is_none()
        {
            return Err(format!(
                "Shader render pass `{}` is invalid",
                pass.pass_node
            ));
        }
        referenced_assets.insert(pass.asset_id.as_str());
    }
    if referenced_assets.len() != asset_ids.len() {
        return Err("Shader render codegen table contains an unreferenced asset".to_owned());
    }
    Ok(())
}

fn verify_source_yir(
    output_dir: &Path,
    expected_version: &str,
    expected_hash: &str,
) -> Result<(), String> {
    let expected_header = format!("yir {expected_version}");
    let matched = fs::read_dir(output_dir)
        .map_err(|error| format!("failed to enumerate AOT output: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("yir"))
        .any(|path| {
            fs::read(path).is_ok_and(|bytes| {
                fnv1a64_hex(&bytes) == expected_hash
                    && std::str::from_utf8(&bytes)
                        .ok()
                        .and_then(|source| source.lines().next())
                        == Some(expected_header.as_str())
            })
        });
    matched.then_some(()).ok_or_else(|| {
        "Shader render table source YIR identity has no matching AOT artifact".to_owned()
    })
}

fn require(fields: &Fields, key: &str, expected: &str) -> Result<(), String> {
    (string_field(fields, key)?.as_str() == expected)
        .then_some(())
        .ok_or_else(|| format!("Shader render table field `{key}` is incompatible"))
}

fn string_field(fields: &Fields, key: &str) -> Result<String, String> {
    let value = fields
        .get(key)
        .ok_or_else(|| format!("Shader render table field `{key}` is missing"))?;
    parse_quoted_string(value)
}

fn usize_field(fields: &Fields, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("Shader render table field `{key}` is missing"))?
        .parse::<usize>()
        .map_err(|_| format!("Shader render table field `{key}` is not an integer"))
}

fn string_array_field(fields: &Fields, key: &str) -> Result<Vec<String>, String> {
    let source = fields
        .get(key)
        .ok_or_else(|| format!("Shader render table field `{key}` is missing"))?
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| format!("Shader render table field `{key}` is not an array"))?;
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }
    source
        .split(',')
        .map(|value| parse_quoted_string(value.trim()))
        .collect()
}

fn parse_quoted_string(value: &str) -> Result<String, String> {
    let source = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| format!("invalid quoted Shader render table value `{value}`"))?;
    let mut out = String::new();
    let mut chars = source.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        out.push(match chars.next() {
            Some('"') => '"',
            Some('\\') => '\\',
            Some('n') => '\n',
            Some('r') => '\r',
            Some('t') => '\t',
            _ => return Err("unsupported Shader render table string escape".to_owned()),
        });
    }
    Ok(out)
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

#[cfg(test)]
#[path = "artifact_device_sample_shader_render_tests.rs"]
mod tests;
