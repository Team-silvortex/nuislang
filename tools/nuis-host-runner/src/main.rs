use std::{
    env, fs,
    path::{Path, PathBuf},
};

mod container;
mod report;

use container::scan_container_loader;
#[cfg(test)]
use container::{
    CONTAINER_KIND, CONTAINER_MAGIC, CONTAINER_PRODUCER, CONTAINER_SCHEMA,
    CONTAINER_SCHEMA_VERSION, CONTAINER_VERSION,
};
use report::{print_text_report, render_json_report};

const RUNNER_PROTOCOL: &str = "nuis-host-runner-v1";
const MANIFEST_SCHEMA: &str = "nuis-host-launcher-manifest-v1";
const HANDOFF_CONTRACT: &str = "nsld-final-output-handoff-v1";
const IMAGE_MAGIC: &[u8; 8] = b"NUIFIMG\0";
const IMAGE_VERSION: u32 = 1;
const IMAGE_HEADER_SIZE: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunnerArgs {
    manifest: PathBuf,
    nsb: Option<PathBuf>,
    output_dir: Option<PathBuf>,
    scheduler_entry: Option<String>,
    lifecycle_hook: Option<String>,
    json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LauncherManifest {
    ready: bool,
    execution_handoff_contract: String,
    execution_handoff_ready: bool,
    nsb_path: String,
    nsb_hash: Option<String>,
    nsb_size_bytes: Option<usize>,
    image_header_required: bool,
    image_header_valid: bool,
    scheduler_entry: String,
    entry_lifecycle_hook: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsbImageHeader {
    version: u32,
    header_size: usize,
    payload_offset: usize,
    payload_span: usize,
    layout_hash: String,
    byte_map_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PayloadRegionScan {
    status: String,
    kind: String,
    prefix_hex: Option<String>,
    prefix_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunnerReport {
    ready: bool,
    would_enter_lifecycle_hook: bool,
    manifest_path: String,
    nsb_path: Option<String>,
    nsb_readable: bool,
    nsb_hash_expected: Option<String>,
    nsb_hash_actual: Option<String>,
    nsb_hash_matches: bool,
    nsb_payload_offset: Option<usize>,
    nsb_payload_span: Option<usize>,
    nsb_payload_region_mapped: bool,
    nsb_payload_region_bytes: Option<usize>,
    nsb_payload_region_hash: Option<String>,
    nsb_payload_scan_status: String,
    nsb_payload_scan_kind: String,
    nsb_payload_prefix_hex: Option<String>,
    nsb_payload_prefix_text: Option<String>,
    container_loader_status: String,
    container_schema: Option<String>,
    container_schema_version: Option<usize>,
    container_kind: Option<String>,
    container_producer: Option<String>,
    container_producer_phase: Option<String>,
    container_ready: Option<bool>,
    container_blockers: Vec<String>,
    container_magic: Option<String>,
    container_version: Option<usize>,
    container_metadata_table_hash: Option<String>,
    container_section_table_hash: Option<String>,
    container_hash: Option<String>,
    container_section_count: Option<usize>,
    container_section_parsed_count: usize,
    container_first_section_id: Option<String>,
    container_first_section_kind: Option<String>,
    container_entry_section_found: bool,
    container_payload_size_bytes: Option<usize>,
    container_payload_hash: Option<String>,
    container_loader_readiness: Option<String>,
    container_loader_blockers: Vec<String>,
    container_loader_entry_kind: Option<String>,
    container_loader_entry_symbol: Option<String>,
    container_loader_entry_section_id: Option<String>,
    container_loader_symbol_count: Option<usize>,
    container_loader_symbol_status: String,
    container_loader_symbol_id: Option<String>,
    container_loader_symbol_kind: Option<String>,
    container_loader_symbol_name: Option<String>,
    container_loader_symbol_lifecycle_hook: Option<String>,
    container_loader_symbol_section_id: Option<String>,
    container_relocation_count: Option<usize>,
    container_relocation_parsed_count: usize,
    container_first_relocation_kind: Option<String>,
    container_first_relocation_source_section_id: Option<String>,
    container_first_relocation_target_symbol_id: Option<String>,
    container_first_relocation_targets_loader_symbol: bool,
    container_first_relocation_source_matches_loader_symbol: bool,
    compatibility_domain_count: Option<usize>,
    compatibility_domain_parsed_count: usize,
    compatibility_domain_first_kind: Option<String>,
    compatibility_domain_required_count: usize,
    loader_symbol_table_hash: Option<String>,
    relocation_table_hash: Option<String>,
    compatibility_domain_table_hash: Option<String>,
    external_import_table_hash: Option<String>,
    external_import_count: Option<usize>,
    external_import_parsed_count: usize,
    external_import_first_kind: Option<String>,
    external_import_first_name: Option<String>,
    external_import_required_imports: Vec<String>,
    container_payload_path: Option<String>,
    container_loader_handoff_status: String,
    container_loader_handoff_ready: bool,
    container_loader_handoff_blockers: Vec<String>,
    nsb_layout_hash: Option<String>,
    nsb_byte_map_hash: Option<String>,
    scheduler_entry: String,
    lifecycle_hook: String,
    launch_steps: Vec<String>,
    blockers: Vec<String>,
}

fn main() {
    match run(env::args().skip(1).collect()) {
        Ok(report) => {
            if env::args().any(|arg| arg == "--json") {
                println!("{}", render_json_report(&report));
            } else {
                print_text_report(&report);
            }
            if !report.ready {
                std::process::exit(2);
            }
        }
        Err(error) => {
            eprintln!("nuis-host-runner: {error}");
            std::process::exit(1);
        }
    }
}

fn run(args: Vec<String>) -> Result<RunnerReport, String> {
    let parsed = parse_args(args)?;
    let manifest_path = resolve_path(Path::new("."), &parsed.manifest);
    let manifest_source = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "failed to read launcher manifest `{}`: {error}",
            manifest_path.display()
        )
    })?;
    let manifest = parse_launcher_manifest(&manifest_source)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let nsb_path = parsed
        .nsb
        .as_ref()
        .map(|path| resolve_path(manifest_dir, path))
        .unwrap_or_else(|| resolve_path(manifest_dir, Path::new(&manifest.nsb_path)));
    let scheduler_entry = parsed
        .scheduler_entry
        .clone()
        .unwrap_or_else(|| manifest.scheduler_entry.clone());
    let lifecycle_hook = parsed
        .lifecycle_hook
        .clone()
        .unwrap_or_else(|| manifest.entry_lifecycle_hook.clone());
    Ok(validate_handoff(
        &manifest_path,
        &nsb_path,
        parsed.output_dir.as_deref(),
        &scheduler_entry,
        &lifecycle_hook,
        &manifest,
    ))
}

fn parse_args(args: Vec<String>) -> Result<RunnerArgs, String> {
    let mut manifest = None;
    let mut nsb = None;
    let mut output_dir = None;
    let mut scheduler_entry = None;
    let mut lifecycle_hook = None;
    let mut json = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--manifest" => manifest = Some(required_path_arg(&mut iter, "--manifest")?),
            "--nsb" => nsb = Some(required_path_arg(&mut iter, "--nsb")?),
            "--output-dir" => output_dir = Some(required_path_arg(&mut iter, "--output-dir")?),
            "--scheduler-entry" => {
                scheduler_entry = Some(required_string_arg(&mut iter, "--scheduler-entry")?)
            }
            "--lifecycle-hook" => {
                lifecycle_hook = Some(required_string_arg(&mut iter, "--lifecycle-hook")?)
            }
            "--json" => json = true,
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown argument `{other}`\n{}", usage())),
        }
    }
    Ok(RunnerArgs {
        manifest: manifest.ok_or_else(usage)?,
        nsb,
        output_dir,
        scheduler_entry,
        lifecycle_hook,
        json,
    })
}

fn required_path_arg(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<PathBuf, String> {
    iter.next()
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} expects a path\n{}", usage()))
}

fn required_string_arg(
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, String> {
    iter.next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{flag} expects a non-empty value\n{}", usage()))
}

fn usage() -> String {
    "usage: nuis-host-runner --manifest <nuis.nsld.final-executable-launcher.toml> [--nsb <path>] [--output-dir <path>] [--scheduler-entry <id>] [--lifecycle-hook <hook>] [--json]".to_owned()
}

fn parse_launcher_manifest(source: &str) -> Result<LauncherManifest, String> {
    let schema = string_value(source, "schema").ok_or("launcher manifest missing `schema`")?;
    if schema != MANIFEST_SCHEMA {
        return Err(format!(
            "unsupported launcher manifest schema `{schema}`; expected `{MANIFEST_SCHEMA}`"
        ));
    }
    Ok(LauncherManifest {
        ready: bool_value(source, "ready").unwrap_or(false),
        execution_handoff_contract: string_value(source, "execution_handoff_contract")
            .unwrap_or_default(),
        execution_handoff_ready: bool_value(source, "execution_handoff_ready").unwrap_or(false),
        nsb_path: string_value(source, "nsb_path").unwrap_or_default(),
        nsb_hash: non_empty_string_value(source, "nsb_hash"),
        nsb_size_bytes: non_zero_usize_value(source, "nsb_size_bytes"),
        image_header_required: bool_value(source, "image_header_required").unwrap_or(true),
        image_header_valid: bool_value(source, "image_header_valid").unwrap_or(false),
        scheduler_entry: string_value(source, "scheduler_entry").unwrap_or_default(),
        entry_lifecycle_hook: string_value(source, "entry_lifecycle_hook").unwrap_or_default(),
    })
}

fn validate_handoff(
    manifest_path: &Path,
    nsb_path: &Path,
    output_dir: Option<&Path>,
    scheduler_entry: &str,
    lifecycle_hook: &str,
    manifest: &LauncherManifest,
) -> RunnerReport {
    let nsb_bytes = fs::read(nsb_path).ok();
    let nsb_readable = nsb_bytes.is_some();
    let nsb_hash_actual = nsb_bytes.as_ref().map(|bytes| fnv1a64_hex(bytes));
    let nsb_hash_matches = manifest.nsb_hash.is_some() && nsb_hash_actual == manifest.nsb_hash;
    let nsb_header = nsb_bytes
        .as_deref()
        .and_then(parse_nsb_image_header)
        .filter(|header| header.payload_region_in_bounds(nsb_bytes.as_deref().unwrap_or(&[])));
    let nsb_payload_region = nsb_bytes
        .as_deref()
        .zip(nsb_header.as_ref())
        .and_then(|(bytes, header)| header.payload_region(bytes));
    let payload_scan = scan_payload_region(nsb_payload_region);
    let container_loader = scan_container_loader(nsb_payload_region, &payload_scan.kind);
    let mut blockers = Vec::new();
    if !manifest.ready {
        blockers.push("launcher-manifest:not-ready".to_owned());
    }
    if manifest.execution_handoff_contract != HANDOFF_CONTRACT {
        blockers.push("execution-handoff-contract:unsupported".to_owned());
    }
    if !manifest.execution_handoff_ready {
        blockers.push("execution-handoff:not-ready".to_owned());
    }
    if manifest.nsb_path.is_empty() {
        blockers.push("nsb-path:missing".to_owned());
    }
    if !nsb_readable {
        blockers.push("nsb:unreadable".to_owned());
    }
    if !nsb_hash_matches {
        blockers.push("nsb:hash-mismatch".to_owned());
    }
    if let (Some(expected), Some(actual)) = (manifest.nsb_size_bytes, nsb_bytes.as_ref()) {
        if actual.len() != expected {
            blockers.push("nsb:size-mismatch".to_owned());
        }
    }
    if manifest.image_header_required && nsb_header.is_none() {
        blockers.push("nsb:image-header-invalid".to_owned());
    }
    if manifest.image_header_required && !manifest.image_header_valid {
        blockers.push("launcher-manifest:image-header-not-certified".to_owned());
    }
    if scheduler_entry.is_empty() || scheduler_entry != manifest.scheduler_entry {
        blockers.push("scheduler-entry:mismatch".to_owned());
    }
    if lifecycle_hook.is_empty() || lifecycle_hook != manifest.entry_lifecycle_hook {
        blockers.push("lifecycle-hook:mismatch".to_owned());
    }
    blockers.extend(container_loader.handoff_blockers.iter().cloned());
    if let Some(output_dir) = output_dir {
        if output_dir.as_os_str().is_empty() {
            blockers.push("output-dir:empty".to_owned());
        }
    }
    let ready = blockers.is_empty();
    RunnerReport {
        ready,
        would_enter_lifecycle_hook: ready,
        manifest_path: manifest_path.display().to_string(),
        nsb_path: Some(nsb_path.display().to_string()),
        nsb_readable,
        nsb_hash_expected: manifest.nsb_hash.clone(),
        nsb_hash_actual,
        nsb_hash_matches,
        nsb_payload_offset: nsb_header.as_ref().map(|header| header.payload_offset),
        nsb_payload_span: nsb_header.as_ref().map(|header| header.payload_span),
        nsb_payload_region_mapped: nsb_payload_region.is_some(),
        nsb_payload_region_bytes: nsb_payload_region.map(|region| region.len()),
        nsb_payload_region_hash: nsb_payload_region.map(fnv1a64_hex),
        nsb_payload_scan_status: payload_scan.status,
        nsb_payload_scan_kind: payload_scan.kind,
        nsb_payload_prefix_hex: payload_scan.prefix_hex,
        nsb_payload_prefix_text: payload_scan.prefix_text,
        container_loader_status: container_loader.status,
        container_schema: container_loader.container_schema,
        container_schema_version: container_loader.container_schema_version,
        container_kind: container_loader.container_kind,
        container_producer: container_loader.container_producer,
        container_producer_phase: container_loader.container_producer_phase,
        container_ready: container_loader.container_ready,
        container_blockers: container_loader.container_blockers,
        container_magic: container_loader.container_magic,
        container_version: container_loader.container_version,
        container_metadata_table_hash: container_loader.container_metadata_table_hash,
        container_section_table_hash: container_loader.container_section_table_hash,
        container_hash: container_loader.container_hash,
        container_section_count: container_loader.container_section.declared_count,
        container_section_parsed_count: container_loader.container_section.parsed_count,
        container_first_section_id: container_loader.container_section.first_section_id,
        container_first_section_kind: container_loader.container_section.first_section_kind,
        container_entry_section_found: container_loader.container_section.entry_section_found,
        container_payload_size_bytes: container_loader.container_payload_size_bytes,
        container_payload_hash: container_loader.container_payload_hash,
        container_loader_readiness: container_loader.loader_readiness,
        container_loader_blockers: container_loader.loader_blockers,
        container_loader_entry_kind: container_loader.loader_entry_kind,
        container_loader_entry_symbol: container_loader.loader_entry_symbol,
        container_loader_entry_section_id: container_loader.loader_entry_section_id,
        container_loader_symbol_count: container_loader.loader_symbol_count,
        container_loader_symbol_status: container_loader.loader_symbol.status,
        container_loader_symbol_id: container_loader.loader_symbol.symbol_id,
        container_loader_symbol_kind: container_loader.loader_symbol.symbol_kind,
        container_loader_symbol_name: container_loader.loader_symbol.symbol_name,
        container_loader_symbol_lifecycle_hook: container_loader.loader_symbol.lifecycle_hook,
        container_loader_symbol_section_id: container_loader.loader_symbol.section_id,
        container_relocation_count: container_loader.relocation.declared_count,
        container_relocation_parsed_count: container_loader.relocation.parsed_count,
        container_first_relocation_kind: container_loader.relocation.first_relocation_kind,
        container_first_relocation_source_section_id: container_loader
            .relocation
            .first_source_section_id,
        container_first_relocation_target_symbol_id: container_loader
            .relocation
            .first_target_symbol_id,
        container_first_relocation_targets_loader_symbol: container_loader
            .relocation
            .first_targets_loader_symbol,
        container_first_relocation_source_matches_loader_symbol: container_loader
            .relocation
            .first_source_matches_loader_symbol,
        compatibility_domain_count: container_loader.compatibility_domain.declared_count,
        compatibility_domain_parsed_count: container_loader.compatibility_domain.parsed_count,
        compatibility_domain_first_kind: container_loader.compatibility_domain.first_domain_kind,
        compatibility_domain_required_count: container_loader.compatibility_domain.required_count,
        loader_symbol_table_hash: container_loader.loader_symbol_table_hash,
        relocation_table_hash: container_loader.relocation_table_hash,
        compatibility_domain_table_hash: container_loader.compatibility_domain_table_hash,
        external_import_table_hash: container_loader.external_import_table_hash,
        external_import_count: container_loader.external_import.declared_count,
        external_import_parsed_count: container_loader.external_import.parsed_count,
        external_import_first_kind: container_loader.external_import.first_import_kind,
        external_import_first_name: container_loader.external_import.first_import_name,
        external_import_required_imports: container_loader.external_import.required_imports,
        container_payload_path: container_loader.container_payload_path,
        container_loader_handoff_status: container_loader.handoff_status,
        container_loader_handoff_ready: container_loader.handoff_ready,
        container_loader_handoff_blockers: container_loader.handoff_blockers,
        nsb_layout_hash: nsb_header.as_ref().map(|header| header.layout_hash.clone()),
        nsb_byte_map_hash: nsb_header
            .as_ref()
            .map(|header| header.byte_map_hash.clone()),
        scheduler_entry: scheduler_entry.to_owned(),
        lifecycle_hook: lifecycle_hook.to_owned(),
        launch_steps: if ready {
            vec![
                "read-launcher-manifest".to_owned(),
                "verify-nsb-header".to_owned(),
                "verify-nsb-hash".to_owned(),
                "map-payload-region".to_owned(),
                format!("enter-lifecycle-hook:{lifecycle_hook}"),
            ]
        } else {
            Vec::new()
        },
        blockers,
    }
}

fn parse_nsb_image_header(bytes: &[u8]) -> Option<NsbImageHeader> {
    if bytes.len() < IMAGE_HEADER_SIZE {
        return None;
    }
    if bytes.get(0..8) != Some(IMAGE_MAGIC) {
        return None;
    }
    let version = read_u32_le(bytes, 8)?;
    let header_size = read_u32_le(bytes, 12)? as usize;
    let payload_span = read_u64_le(bytes, 24)? as usize;
    let payload_offset = read_u64_le(bytes, 32)? as usize;
    if version != IMAGE_VERSION
        || header_size != IMAGE_HEADER_SIZE
        || payload_offset != IMAGE_HEADER_SIZE
    {
        return None;
    }
    Some(NsbImageHeader {
        version,
        header_size,
        payload_offset,
        payload_span,
        layout_hash: format!("0x{:016x}", read_u64_le(bytes, 40)?),
        byte_map_hash: format!("0x{:016x}", read_u64_le(bytes, 48)?),
    })
}

impl NsbImageHeader {
    fn payload_region_in_bounds(&self, bytes: &[u8]) -> bool {
        self.payload_offset
            .checked_add(self.payload_span)
            .is_some_and(|end| end <= bytes.len())
    }

    fn payload_region<'a>(&self, bytes: &'a [u8]) -> Option<&'a [u8]> {
        let end = self.payload_offset.checked_add(self.payload_span)?;
        bytes.get(self.payload_offset..end)
    }
}

fn scan_payload_region(region: Option<&[u8]>) -> PayloadRegionScan {
    let Some(region) = region else {
        return PayloadRegionScan {
            status: "not-mapped".to_owned(),
            kind: "none".to_owned(),
            prefix_hex: None,
            prefix_text: None,
        };
    };
    if region.is_empty() {
        return PayloadRegionScan {
            status: "empty".to_owned(),
            kind: "empty".to_owned(),
            prefix_hex: Some(String::new()),
            prefix_text: Some(String::new()),
        };
    }
    let prefix_text = ascii_preview(region, 64);
    let kind = if prefix_text.contains("schema = \"nuis-nsld-container-v1\"") {
        "nsld-container-toml"
    } else if prefix_text.trim_start().starts_with("schema = ") || prefix_text.contains("[[") {
        "toml-like"
    } else {
        "opaque-bytes"
    };
    PayloadRegionScan {
        status: "scanned".to_owned(),
        kind: kind.to_owned(),
        prefix_hex: Some(hex_preview(region, 24)),
        prefix_text: Some(prefix_text),
    }
}

fn hex_preview(bytes: &[u8], limit: usize) -> String {
    bytes
        .iter()
        .take(limit)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn ascii_preview(bytes: &[u8], limit: usize) -> String {
    bytes
        .iter()
        .take(limit)
        .map(|byte| match byte {
            b'\n' => ' ',
            b'\r' | b'\t' => ' ',
            0x20..=0x7e => char::from(*byte),
            _ => '.',
        })
        .collect()
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let chunk: [u8; 4] = bytes.get(offset..offset + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(chunk))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Option<u64> {
    let chunk: [u8; 8] = bytes.get(offset..offset + 8)?.try_into().ok()?;
    Some(u64::from_le_bytes(chunk))
}

fn resolve_path(base: &Path, value: &Path) -> PathBuf {
    if value.is_absolute() {
        value.to_path_buf()
    } else {
        base.join(value)
    }
}

fn string_value(source: &str, key: &str) -> Option<String> {
    let raw = raw_value(source, key)?.trim();
    let quoted = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn non_empty_string_value(source: &str, key: &str) -> Option<String> {
    string_value(source, key).filter(|value| !value.is_empty())
}

fn bool_value(source: &str, key: &str) -> Option<bool> {
    match raw_value(source, key)?.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn non_zero_usize_value(source: &str, key: &str) -> Option<usize> {
    raw_value(source, key)?
        .trim()
        .parse()
        .ok()
        .filter(|value| *value != 0)
}

fn raw_value<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    raw_value_from_lines(&source.lines().collect::<Vec<_>>(), key)
}

fn raw_value_from_lines<'a>(lines: &[&'a str], key: &str) -> Option<&'a str> {
    lines.iter().copied().find_map(|raw| {
        let (found_key, value) = raw.trim().split_once('=')?;
        (found_key.trim() == key).then_some(value.trim())
    })
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
mod tests;
