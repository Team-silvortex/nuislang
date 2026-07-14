use std::{
    env, fs,
    path::{Path, PathBuf},
};

const RUNNER_PROTOCOL: &str = "nuis-host-runner-v1";
const MANIFEST_SCHEMA: &str = "nuis-host-launcher-manifest-v1";
const HANDOFF_CONTRACT: &str = "nsld-final-output-handoff-v1";
const IMAGE_MAGIC: &[u8; 8] = b"NUIFIMG\0";
const IMAGE_VERSION: u32 = 1;
const IMAGE_HEADER_SIZE: usize = 64;
const CONTAINER_MAGIC: &str = "NUISNSLD";
const CONTAINER_VERSION: usize = 1;

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
struct ContainerLoaderSymbolSummary {
    status: String,
    symbol_id: Option<String>,
    symbol_kind: Option<String>,
    symbol_name: Option<String>,
    lifecycle_hook: Option<String>,
    section_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContainerLoaderSummary {
    status: String,
    container_ready: Option<bool>,
    container_blockers: Vec<String>,
    container_magic: Option<String>,
    container_version: Option<usize>,
    container_payload_size_bytes: Option<usize>,
    container_payload_hash: Option<String>,
    loader_readiness: Option<String>,
    loader_blockers: Vec<String>,
    loader_entry_kind: Option<String>,
    loader_entry_symbol: Option<String>,
    loader_entry_section_id: Option<String>,
    loader_symbol_count: Option<usize>,
    loader_symbol: ContainerLoaderSymbolSummary,
    handoff_status: String,
    handoff_ready: bool,
    handoff_blockers: Vec<String>,
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
    container_ready: Option<bool>,
    container_blockers: Vec<String>,
    container_magic: Option<String>,
    container_version: Option<usize>,
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
        container_ready: container_loader.container_ready,
        container_blockers: container_loader.container_blockers,
        container_magic: container_loader.container_magic,
        container_version: container_loader.container_version,
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

fn scan_container_loader(region: Option<&[u8]>, payload_kind: &str) -> ContainerLoaderSummary {
    if payload_kind != "nsld-container-toml" {
        return ContainerLoaderSummary {
            status: "not-container-toml".to_owned(),
            container_ready: None,
            container_blockers: Vec::new(),
            container_magic: None,
            container_version: None,
            container_payload_size_bytes: None,
            container_payload_hash: None,
            loader_readiness: None,
            loader_blockers: Vec::new(),
            loader_entry_kind: None,
            loader_entry_symbol: None,
            loader_entry_section_id: None,
            loader_symbol_count: None,
            loader_symbol: ContainerLoaderSymbolSummary::empty("not-container-toml"),
            handoff_status: "not-container-toml".to_owned(),
            handoff_ready: false,
            handoff_blockers: Vec::new(),
        };
    }
    let Some(region) = region else {
        return ContainerLoaderSummary {
            status: "not-mapped".to_owned(),
            container_ready: None,
            container_blockers: Vec::new(),
            container_magic: None,
            container_version: None,
            container_payload_size_bytes: None,
            container_payload_hash: None,
            loader_readiness: None,
            loader_blockers: Vec::new(),
            loader_entry_kind: None,
            loader_entry_symbol: None,
            loader_entry_section_id: None,
            loader_symbol_count: None,
            loader_symbol: ContainerLoaderSymbolSummary::empty("not-mapped"),
            handoff_status: "not-mapped".to_owned(),
            handoff_ready: false,
            handoff_blockers: Vec::new(),
        };
    };
    let Ok(source) = std::str::from_utf8(region) else {
        return ContainerLoaderSummary {
            status: "invalid-utf8".to_owned(),
            container_ready: None,
            container_blockers: Vec::new(),
            container_magic: None,
            container_version: None,
            container_payload_size_bytes: None,
            container_payload_hash: None,
            loader_readiness: None,
            loader_blockers: Vec::new(),
            loader_entry_kind: None,
            loader_entry_symbol: None,
            loader_entry_section_id: None,
            loader_symbol_count: None,
            loader_symbol: ContainerLoaderSymbolSummary::empty("invalid-utf8"),
            handoff_status: "invalid-utf8".to_owned(),
            handoff_ready: false,
            handoff_blockers: vec!["container-loader:invalid-utf8".to_owned()],
        };
    };
    let container_ready = bool_value(source, "ready");
    let container_blockers = string_array_value(source, "blockers");
    let container_magic = string_value(source, "container_magic");
    let container_version = usize_value(source, "container_version");
    let container_payload_size_bytes = usize_value(source, "payload_size_bytes");
    let container_payload_hash = string_value(source, "payload_hash");
    let loader_readiness = string_value(source, "loader_readiness");
    let loader_blockers = string_array_value(source, "loader_blockers");
    let loader_entry_kind = string_value(source, "loader_entry_kind");
    let loader_entry_symbol = string_value(source, "loader_entry_symbol");
    let loader_entry_section_id = string_value(source, "loader_entry_section_id");
    let loader_symbol_count = usize_value(source, "loader_symbol_count");
    let loader_symbol = scan_first_loader_symbol(source);
    let handoff_blockers = container_loader_handoff_blockers(
        container_ready,
        &container_blockers,
        container_magic.as_deref(),
        container_version,
        container_payload_size_bytes,
        container_payload_hash.as_deref(),
        loader_readiness.as_deref(),
        &loader_blockers,
        loader_entry_kind.as_deref(),
        loader_entry_symbol.as_deref(),
        loader_entry_section_id.as_deref(),
        loader_symbol_count,
        &loader_symbol,
    );
    let handoff_ready = handoff_blockers.is_empty();
    ContainerLoaderSummary {
        status: "parsed".to_owned(),
        container_ready,
        container_blockers,
        container_magic,
        container_version,
        container_payload_size_bytes,
        container_payload_hash,
        loader_readiness,
        loader_blockers,
        loader_entry_kind,
        loader_entry_symbol,
        loader_entry_section_id,
        loader_symbol_count,
        loader_symbol,
        handoff_status: if handoff_ready { "ready" } else { "blocked" }.to_owned(),
        handoff_ready,
        handoff_blockers,
    }
}

impl ContainerLoaderSymbolSummary {
    fn empty(status: &str) -> Self {
        Self {
            status: status.to_owned(),
            symbol_id: None,
            symbol_kind: None,
            symbol_name: None,
            lifecycle_hook: None,
            section_id: None,
        }
    }
}

fn scan_first_loader_symbol(source: &str) -> ContainerLoaderSymbolSummary {
    let Some(block) = first_array_table_block(source, "loader_symbol") else {
        return ContainerLoaderSymbolSummary::empty("missing");
    };
    ContainerLoaderSymbolSummary {
        status: "parsed".to_owned(),
        symbol_id: string_value_from_lines(&block, "symbol_id"),
        symbol_kind: string_value_from_lines(&block, "symbol_kind"),
        symbol_name: string_value_from_lines(&block, "symbol_name"),
        lifecycle_hook: string_value_from_lines(&block, "lifecycle_hook"),
        section_id: string_value_from_lines(&block, "section_id"),
    }
}

fn container_loader_handoff_blockers(
    container_ready: Option<bool>,
    container_blockers: &[String],
    container_magic: Option<&str>,
    container_version: Option<usize>,
    container_payload_size_bytes: Option<usize>,
    container_payload_hash: Option<&str>,
    loader_readiness: Option<&str>,
    loader_blockers: &[String],
    loader_entry_kind: Option<&str>,
    loader_entry_symbol: Option<&str>,
    loader_entry_section_id: Option<&str>,
    loader_symbol_count: Option<usize>,
    loader_symbol: &ContainerLoaderSymbolSummary,
) -> Vec<String> {
    let mut blockers = Vec::new();
    match container_ready {
        Some(true) => {}
        Some(false) => blockers.push("container:not-ready".to_owned()),
        None => blockers.push("container:ready-missing".to_owned()),
    }
    blockers.extend(
        container_blockers
            .iter()
            .map(|blocker| format!("container:blocker:{blocker}")),
    );
    match container_magic {
        Some(CONTAINER_MAGIC) => {}
        Some(_) => blockers.push("container:magic-unsupported".to_owned()),
        None => blockers.push("container:magic-missing".to_owned()),
    }
    match container_version {
        Some(CONTAINER_VERSION) => {}
        Some(_) => blockers.push("container:version-unsupported".to_owned()),
        None => blockers.push("container:version-missing".to_owned()),
    }
    if container_payload_size_bytes.is_none() {
        blockers.push("container:payload-size-missing".to_owned());
    }
    if container_payload_hash.is_none_or(str::is_empty) {
        blockers.push("container:payload-hash-missing".to_owned());
    }
    match loader_readiness {
        Some("self-contained" | "host-assisted") => {}
        Some("blocked") => blockers.push("container-loader:readiness-blocked".to_owned()),
        Some(_) => blockers.push("container-loader:readiness-unsupported".to_owned()),
        None => blockers.push("container-loader:readiness-missing".to_owned()),
    }
    blockers.extend(
        loader_blockers
            .iter()
            .map(|blocker| format!("container-loader:blocker:{blocker}")),
    );
    if loader_entry_symbol.is_none_or(str::is_empty) {
        blockers.push("container-loader:entry-symbol-missing".to_owned());
    }
    if loader_entry_kind.is_none_or(str::is_empty) {
        blockers.push("container-loader:entry-kind-missing".to_owned());
    }
    if loader_entry_section_id.is_none_or(str::is_empty) {
        blockers.push("container-loader:entry-section-missing".to_owned());
    }
    if loader_symbol_count.unwrap_or(0) == 0 {
        blockers.push("container-loader:symbols-missing".to_owned());
    }
    if loader_symbol_count.unwrap_or(0) > 0 {
        if loader_symbol.status != "parsed" {
            blockers.push("container-loader:symbol-table-missing".to_owned());
        } else {
            match (loader_entry_kind, loader_symbol.symbol_kind.as_deref()) {
                (Some(entry_kind), Some(symbol_kind)) if entry_kind == symbol_kind => {}
                (Some(_), Some(_)) => {
                    blockers.push("container-loader:entry-kind-mismatch".to_owned())
                }
                (_, None) => blockers.push("container-loader:symbol-kind-missing".to_owned()),
                (None, Some(_)) => {}
            }
            match loader_symbol.lifecycle_hook.as_deref() {
                Some("on_lifecycle_bootstrap") => {}
                Some(_) => blockers.push("container-loader:lifecycle-hook-unsupported".to_owned()),
                None => blockers.push("container-loader:lifecycle-hook-missing".to_owned()),
            }
            if loader_entry_symbol.is_some_and(|entry| {
                loader_symbol
                    .symbol_name
                    .as_deref()
                    .is_some_and(|symbol| symbol != entry)
            }) {
                blockers.push("container-loader:entry-symbol-mismatch".to_owned());
            }
            if loader_entry_section_id.is_some_and(|entry| {
                loader_symbol
                    .section_id
                    .as_deref()
                    .is_some_and(|section| section != entry)
            }) {
                blockers.push("container-loader:entry-section-mismatch".to_owned());
            }
            if loader_symbol
                .symbol_name
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("container-loader:symbol-name-missing".to_owned());
            }
            if loader_symbol
                .section_id
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("container-loader:symbol-section-missing".to_owned());
            }
        }
    }
    blockers
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

fn usize_value(source: &str, key: &str) -> Option<usize> {
    raw_value(source, key)?.trim().parse().ok()
}

fn string_array_value(source: &str, key: &str) -> Vec<String> {
    let Some(raw) = raw_value(source, key) else {
        return Vec::new();
    };
    let Some(body) = raw
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Vec::new();
    };
    body.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            let quoted = entry.strip_prefix('"')?.strip_suffix('"')?;
            Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
        })
        .collect()
}

fn string_value_from_lines(lines: &[&str], key: &str) -> Option<String> {
    let raw = raw_value_from_lines(lines, key)?.trim();
    let quoted = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(quoted.replace("\\\"", "\"").replace("\\\\", "\\"))
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

fn first_array_table_block<'a>(source: &'a str, table: &str) -> Option<Vec<&'a str>> {
    let header = format!("[[{table}]]");
    let mut in_table = false;
    let mut block = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            if in_table {
                break;
            }
            in_table = true;
            continue;
        }
        if in_table && trimmed.starts_with("[[") {
            break;
        }
        if in_table {
            block.push(line);
        }
    }
    (!block.is_empty()).then_some(block)
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn print_text_report(report: &RunnerReport) {
    println!("nuis-host-runner: {}", RUNNER_PROTOCOL);
    println!("  ready: {}", report.ready);
    println!(
        "  would_enter_lifecycle_hook: {}",
        report.would_enter_lifecycle_hook
    );
    println!("  manifest_path: {}", report.manifest_path);
    println!(
        "  nsb_path: {}",
        report.nsb_path.as_deref().unwrap_or("<none>")
    );
    println!("  nsb_readable: {}", report.nsb_readable);
    println!("  nsb_hash_matches: {}", report.nsb_hash_matches);
    println!(
        "  nsb_payload_offset: {}",
        optional_usize_text(report.nsb_payload_offset)
    );
    println!(
        "  nsb_payload_span: {}",
        optional_usize_text(report.nsb_payload_span)
    );
    println!(
        "  nsb_payload_region_mapped: {}",
        report.nsb_payload_region_mapped
    );
    println!(
        "  nsb_payload_region_bytes: {}",
        optional_usize_text(report.nsb_payload_region_bytes)
    );
    println!(
        "  nsb_payload_region_hash: {}",
        report
            .nsb_payload_region_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsb_payload_scan_status: {}",
        report.nsb_payload_scan_status
    );
    println!("  nsb_payload_scan_kind: {}", report.nsb_payload_scan_kind);
    println!(
        "  nsb_payload_prefix_hex: {}",
        report.nsb_payload_prefix_hex.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsb_payload_prefix_text: {}",
        report
            .nsb_payload_prefix_text
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_status: {}",
        report.container_loader_status
    );
    println!(
        "  container_ready: {}",
        report
            .container_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<none>".to_owned())
    );
    println!(
        "  container_blockers: {}",
        if report.container_blockers.is_empty() {
            "<none>".to_owned()
        } else {
            report.container_blockers.join(", ")
        }
    );
    println!(
        "  container_magic: {}",
        report.container_magic.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_version: {}",
        optional_usize_text(report.container_version)
    );
    println!(
        "  container_payload_size_bytes: {}",
        optional_usize_text(report.container_payload_size_bytes)
    );
    println!(
        "  container_payload_hash: {}",
        report.container_payload_hash.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_loader_readiness: {}",
        report
            .container_loader_readiness
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_blockers: {}",
        if report.container_loader_blockers.is_empty() {
            "<none>".to_owned()
        } else {
            report.container_loader_blockers.join(", ")
        }
    );
    println!(
        "  container_loader_entry_kind: {}",
        report
            .container_loader_entry_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_entry_symbol: {}",
        report
            .container_loader_entry_symbol
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_entry_section_id: {}",
        report
            .container_loader_entry_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_count: {}",
        optional_usize_text(report.container_loader_symbol_count)
    );
    println!(
        "  container_loader_symbol_status: {}",
        report.container_loader_symbol_status
    );
    println!(
        "  container_loader_symbol_id: {}",
        report
            .container_loader_symbol_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_kind: {}",
        report
            .container_loader_symbol_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_name: {}",
        report
            .container_loader_symbol_name
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_lifecycle_hook: {}",
        report
            .container_loader_symbol_lifecycle_hook
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_section_id: {}",
        report
            .container_loader_symbol_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_handoff_status: {}",
        report.container_loader_handoff_status
    );
    println!(
        "  container_loader_handoff_ready: {}",
        report.container_loader_handoff_ready
    );
    println!(
        "  container_loader_handoff_blockers: {}",
        if report.container_loader_handoff_blockers.is_empty() {
            "<none>".to_owned()
        } else {
            report.container_loader_handoff_blockers.join(", ")
        }
    );
    println!(
        "  nsb_layout_hash: {}",
        report.nsb_layout_hash.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsb_byte_map_hash: {}",
        report.nsb_byte_map_hash.as_deref().unwrap_or("<none>")
    );
    println!("  scheduler_entry: {}", report.scheduler_entry);
    println!("  lifecycle_hook: {}", report.lifecycle_hook);
    println!(
        "  launch_steps: {}",
        if report.launch_steps.is_empty() {
            "<none>".to_owned()
        } else {
            report.launch_steps.join(", ")
        }
    );
    println!(
        "  blockers: {}",
        if report.blockers.is_empty() {
            "<none>".to_owned()
        } else {
            report.blockers.join(", ")
        }
    );
}

fn render_json_report(report: &RunnerReport) -> String {
    format!(
        "{{\"kind\":\"nuis_host_runner\",\"protocol\":\"{}\",\"ready\":{},\"would_enter_lifecycle_hook\":{},\"manifest_path\":\"{}\",\"nsb_path\":{},\"nsb_readable\":{},\"nsb_hash_expected\":{},\"nsb_hash_actual\":{},\"nsb_hash_matches\":{},\"nsb_payload_offset\":{},\"nsb_payload_span\":{},\"nsb_payload_region_mapped\":{},\"nsb_payload_region_bytes\":{},\"nsb_payload_region_hash\":{},\"nsb_payload_scan_status\":\"{}\",\"nsb_payload_scan_kind\":\"{}\",\"nsb_payload_prefix_hex\":{},\"nsb_payload_prefix_text\":{},\"container_loader_status\":\"{}\",\"container_ready\":{},\"container_blockers\":[{}],\"container_magic\":{},\"container_version\":{},\"container_payload_size_bytes\":{},\"container_payload_hash\":{},\"container_loader_readiness\":{},\"container_loader_blockers\":[{}],\"container_loader_entry_kind\":{},\"container_loader_entry_symbol\":{},\"container_loader_entry_section_id\":{},\"container_loader_symbol_count\":{},\"container_loader_symbol_status\":\"{}\",\"container_loader_symbol_id\":{},\"container_loader_symbol_kind\":{},\"container_loader_symbol_name\":{},\"container_loader_symbol_lifecycle_hook\":{},\"container_loader_symbol_section_id\":{},\"container_loader_handoff_status\":\"{}\",\"container_loader_handoff_ready\":{},\"container_loader_handoff_blockers\":[{}],\"nsb_layout_hash\":{},\"nsb_byte_map_hash\":{},\"scheduler_entry\":\"{}\",\"lifecycle_hook\":\"{}\",\"launch_steps\":[{}],\"blockers\":[{}]}}",
        json_escape(RUNNER_PROTOCOL),
        report.ready,
        report.would_enter_lifecycle_hook,
        json_escape(&report.manifest_path),
        json_optional_string(report.nsb_path.as_deref()),
        report.nsb_readable,
        json_optional_string(report.nsb_hash_expected.as_deref()),
        json_optional_string(report.nsb_hash_actual.as_deref()),
        report.nsb_hash_matches,
        json_optional_usize(report.nsb_payload_offset),
        json_optional_usize(report.nsb_payload_span),
        report.nsb_payload_region_mapped,
        json_optional_usize(report.nsb_payload_region_bytes),
        json_optional_string(report.nsb_payload_region_hash.as_deref()),
        json_escape(&report.nsb_payload_scan_status),
        json_escape(&report.nsb_payload_scan_kind),
        json_optional_string(report.nsb_payload_prefix_hex.as_deref()),
        json_optional_string(report.nsb_payload_prefix_text.as_deref()),
        json_escape(&report.container_loader_status),
        json_optional_bool(report.container_ready),
        json_string_array(&report.container_blockers),
        json_optional_string(report.container_magic.as_deref()),
        json_optional_usize(report.container_version),
        json_optional_usize(report.container_payload_size_bytes),
        json_optional_string(report.container_payload_hash.as_deref()),
        json_optional_string(report.container_loader_readiness.as_deref()),
        json_string_array(&report.container_loader_blockers),
        json_optional_string(report.container_loader_entry_kind.as_deref()),
        json_optional_string(report.container_loader_entry_symbol.as_deref()),
        json_optional_string(report.container_loader_entry_section_id.as_deref()),
        json_optional_usize(report.container_loader_symbol_count),
        json_escape(&report.container_loader_symbol_status),
        json_optional_string(report.container_loader_symbol_id.as_deref()),
        json_optional_string(report.container_loader_symbol_kind.as_deref()),
        json_optional_string(report.container_loader_symbol_name.as_deref()),
        json_optional_string(report.container_loader_symbol_lifecycle_hook.as_deref()),
        json_optional_string(report.container_loader_symbol_section_id.as_deref()),
        json_escape(&report.container_loader_handoff_status),
        report.container_loader_handoff_ready,
        json_string_array(&report.container_loader_handoff_blockers),
        json_optional_string(report.nsb_layout_hash.as_deref()),
        json_optional_string(report.nsb_byte_map_hash.as_deref()),
        json_escape(&report.scheduler_entry),
        json_escape(&report.lifecycle_hook),
        json_string_array(&report.launch_steps),
        json_string_array(&report.blockers)
    )
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_owned())
}

fn json_optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_bool(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nsb_payload() -> &'static [u8] {
        b"schema = \"nuis-nsld-container-v1\"\nschema_version = 1\ncontainer_kind = \"deterministic-hetero-container\"\nproducer = \"nsld\"\nproducer_phase = \"alpha-0.10.0\"\nready = true\ncontainer_magic = \"NUISNSLD\"\ncontainer_version = 1\ncontainer_hash = \"0xaaaaaaaaaaaaaaaa\"\nloader_readiness = \"host-assisted\"\nloader_blockers = []\nloader_entry_kind = \"lifecycle-bootstrap\"\nloader_entry_symbol = \"main\"\nloader_entry_section_id = \"sec0000.compiled-artifact\"\nloader_symbol_count = 3\npayload_size_bytes = 128\npayload_hash = \"0xbbbbbbbbbbbbbbbb\"\nblockers = []\n\n[[loader_symbol]]\nsymbol_id = \"sym0000.loader-entry\"\nsymbol_kind = \"lifecycle-bootstrap\"\nsymbol_name = \"main\"\nlifecycle_hook = \"on_lifecycle_bootstrap\"\nsection_id = \"sec0000.compiled-artifact\"\n"
    }

    fn nsb_bytes() -> Vec<u8> {
        let payload = nsb_payload();
        let mut bytes = vec![0u8; IMAGE_HEADER_SIZE + payload.len()];
        bytes[0..8].copy_from_slice(IMAGE_MAGIC);
        bytes[8..12].copy_from_slice(&IMAGE_VERSION.to_le_bytes());
        bytes[12..16].copy_from_slice(&(IMAGE_HEADER_SIZE as u32).to_le_bytes());
        bytes[24..32].copy_from_slice(&(payload.len() as u64).to_le_bytes());
        bytes[32..40].copy_from_slice(&(IMAGE_HEADER_SIZE as u64).to_le_bytes());
        bytes[40..48].copy_from_slice(&0x1234u64.to_le_bytes());
        bytes[48..56].copy_from_slice(&0x5678u64.to_le_bytes());
        bytes[IMAGE_HEADER_SIZE..].copy_from_slice(payload);
        bytes
    }

    fn manifest_source(nsb_hash: &str, nsb_size: usize) -> String {
        format!(
            "schema = \"{MANIFEST_SCHEMA}\"\nready = true\nexecution_handoff_contract = \"{HANDOFF_CONTRACT}\"\nexecution_handoff_ready = true\nnsb_path = \"nuis-app.nsb\"\nnsb_hash = \"{nsb_hash}\"\nnsb_size_bytes = {nsb_size}\nimage_header_required = true\nimage_header_valid = true\nscheduler_entry = \"nuis.scheduler.loop.v1\"\nentry_lifecycle_hook = \"on_process_start\"\n"
        )
    }

    #[test]
    fn validates_ready_launcher_handoff() {
        let bytes = nsb_bytes();
        let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
            .expect("manifest parses");
        let dir = env::temp_dir().join(format!("nuis-host-runner-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(report.ready);
        assert!(report.would_enter_lifecycle_hook);
        assert!(report
            .launch_steps
            .contains(&"map-payload-region".to_owned()));
        assert!(report
            .launch_steps
            .contains(&"enter-lifecycle-hook:on_process_start".to_owned()));
        assert_eq!(report.nsb_payload_offset, Some(IMAGE_HEADER_SIZE));
        assert_eq!(report.nsb_payload_span, Some(nsb_payload().len()));
        assert!(report.nsb_payload_region_mapped);
        assert_eq!(report.nsb_payload_region_bytes, Some(nsb_payload().len()));
        let expected_payload_region_hash = fnv1a64_hex(nsb_payload());
        assert_eq!(
            report.nsb_payload_region_hash.as_deref(),
            Some(expected_payload_region_hash.as_str())
        );
        assert_eq!(report.nsb_payload_scan_status, "scanned");
        assert_eq!(report.nsb_payload_scan_kind, "nsld-container-toml");
        assert!(report
            .nsb_payload_prefix_text
            .as_deref()
            .is_some_and(|prefix| prefix.contains("nuis-nsld-container-v1")));
        assert!(report
            .nsb_payload_prefix_hex
            .as_deref()
            .is_some_and(|prefix| prefix.starts_with("736368656d6120")));
        assert_eq!(report.container_loader_status, "parsed");
        assert_eq!(report.container_ready, Some(true));
        assert!(report.container_blockers.is_empty());
        assert_eq!(report.container_magic.as_deref(), Some(CONTAINER_MAGIC));
        assert_eq!(report.container_version, Some(CONTAINER_VERSION));
        assert_eq!(report.container_payload_size_bytes, Some(128));
        assert_eq!(
            report.container_payload_hash.as_deref(),
            Some("0xbbbbbbbbbbbbbbbb")
        );
        assert_eq!(
            report.container_loader_readiness.as_deref(),
            Some("host-assisted")
        );
        assert!(report.container_loader_blockers.is_empty());
        assert_eq!(
            report.container_loader_entry_kind.as_deref(),
            Some("lifecycle-bootstrap")
        );
        assert_eq!(
            report.container_loader_entry_symbol.as_deref(),
            Some("main")
        );
        assert_eq!(
            report.container_loader_entry_section_id.as_deref(),
            Some("sec0000.compiled-artifact")
        );
        assert_eq!(report.container_loader_symbol_count, Some(3));
        assert_eq!(report.container_loader_symbol_status, "parsed");
        assert_eq!(
            report.container_loader_symbol_id.as_deref(),
            Some("sym0000.loader-entry")
        );
        assert_eq!(
            report.container_loader_symbol_kind.as_deref(),
            Some("lifecycle-bootstrap")
        );
        assert_eq!(report.container_loader_symbol_name.as_deref(), Some("main"));
        assert_eq!(
            report.container_loader_symbol_lifecycle_hook.as_deref(),
            Some("on_lifecycle_bootstrap")
        );
        assert_eq!(
            report.container_loader_symbol_section_id.as_deref(),
            Some("sec0000.compiled-artifact")
        );
        assert_eq!(report.container_loader_handoff_status, "ready");
        assert!(report.container_loader_handoff_ready);
        assert!(report.container_loader_handoff_blockers.is_empty());
        assert_eq!(
            report.nsb_layout_hash.as_deref(),
            Some("0x0000000000001234")
        );
        assert_eq!(
            report.nsb_byte_map_hash.as_deref(),
            Some("0x0000000000005678")
        );
        assert!(report.blockers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_hash_mismatch() {
        let bytes = nsb_bytes();
        let manifest = parse_launcher_manifest(&manifest_source("0x0000000000000000", bytes.len()))
            .expect("manifest parses");
        let dir =
            env::temp_dir().join(format!("nuis-host-runner-hash-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(!report.ready);
        assert!(report.blockers.contains(&"nsb:hash-mismatch".to_owned()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_container_loader_handoff_when_loader_is_blocked() {
        let mut bytes = nsb_bytes();
        let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
        let tampered = source.replace(
            "loader_readiness = \"host-assisted\"",
            "loader_readiness = \"blocked\"",
        );
        bytes.truncate(IMAGE_HEADER_SIZE);
        bytes.extend_from_slice(tampered.as_bytes());
        bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

        let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
            .expect("manifest parses");
        let dir = env::temp_dir().join(format!(
            "nuis-host-runner-loader-blocked-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(!report.ready);
        assert_eq!(report.container_loader_handoff_status, "blocked");
        assert!(!report.container_loader_handoff_ready);
        assert!(report
            .container_loader_handoff_blockers
            .contains(&"container-loader:readiness-blocked".to_owned()));
        assert!(report
            .blockers
            .contains(&"container-loader:readiness-blocked".to_owned()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_container_loader_handoff_when_symbol_table_mismatches_entry() {
        let mut bytes = nsb_bytes();
        let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
        let tampered = source.replace("symbol_name = \"main\"", "symbol_name = \"boot\"");
        bytes.truncate(IMAGE_HEADER_SIZE);
        bytes.extend_from_slice(tampered.as_bytes());
        bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

        let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
            .expect("manifest parses");
        let dir = env::temp_dir().join(format!(
            "nuis-host-runner-loader-symbol-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(!report.ready);
        assert_eq!(report.container_loader_symbol_status, "parsed");
        assert_eq!(report.container_loader_symbol_name.as_deref(), Some("boot"));
        assert_eq!(report.container_loader_handoff_status, "blocked");
        assert!(!report.container_loader_handoff_ready);
        assert!(report
            .container_loader_handoff_blockers
            .contains(&"container-loader:entry-symbol-mismatch".to_owned()));
        assert!(report
            .blockers
            .contains(&"container-loader:entry-symbol-mismatch".to_owned()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_container_loader_handoff_when_loader_blockers_are_declared() {
        let mut bytes = nsb_bytes();
        let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
        let tampered = source.replace(
            "loader_blockers = []",
            "loader_blockers = [\"external-import:final-stage-driver:cc\"]",
        );
        bytes.truncate(IMAGE_HEADER_SIZE);
        bytes.extend_from_slice(tampered.as_bytes());
        bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

        let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
            .expect("manifest parses");
        let dir = env::temp_dir().join(format!(
            "nuis-host-runner-loader-blocker-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(!report.ready);
        assert_eq!(
            report.container_loader_blockers,
            vec!["external-import:final-stage-driver:cc".to_owned()]
        );
        assert_eq!(report.container_loader_handoff_status, "blocked");
        assert!(!report.container_loader_handoff_ready);
        assert!(report.container_loader_handoff_blockers.contains(
            &"container-loader:blocker:external-import:final-stage-driver:cc".to_owned()
        ));
        assert!(report.blockers.contains(
            &"container-loader:blocker:external-import:final-stage-driver:cc".to_owned()
        ));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_container_handoff_when_container_blockers_are_declared() {
        let mut bytes = nsb_bytes();
        let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
        let tampered = source.replace("\nblockers = []", "\nblockers = [\"payload-not-sealed\"]");
        bytes.truncate(IMAGE_HEADER_SIZE);
        bytes.extend_from_slice(tampered.as_bytes());
        bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

        let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
            .expect("manifest parses");
        let dir = env::temp_dir().join(format!(
            "nuis-host-runner-container-blocker-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(!report.ready);
        assert_eq!(
            report.container_blockers,
            vec!["payload-not-sealed".to_owned()]
        );
        assert_eq!(report.container_loader_handoff_status, "blocked");
        assert!(report
            .container_loader_handoff_blockers
            .contains(&"container:blocker:payload-not-sealed".to_owned()));
        assert!(report
            .blockers
            .contains(&"container:blocker:payload-not-sealed".to_owned()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn blocks_container_loader_handoff_when_entry_kind_mismatches_symbol_kind() {
        let mut bytes = nsb_bytes();
        let source = String::from_utf8(bytes[IMAGE_HEADER_SIZE..].to_vec()).unwrap();
        let tampered = source.replace(
            "loader_entry_kind = \"lifecycle-bootstrap\"",
            "loader_entry_kind = \"host-entry-bootstrap\"",
        );
        bytes.truncate(IMAGE_HEADER_SIZE);
        bytes.extend_from_slice(tampered.as_bytes());
        bytes[24..32].copy_from_slice(&(tampered.len() as u64).to_le_bytes());

        let manifest = parse_launcher_manifest(&manifest_source(&fnv1a64_hex(&bytes), bytes.len()))
            .expect("manifest parses");
        let dir = env::temp_dir().join(format!(
            "nuis-host-runner-entry-kind-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let nsb_path = dir.join("nuis-app.nsb");
        fs::write(&nsb_path, bytes).expect("write nsb");

        let report = validate_handoff(
            &dir.join("nuis.nsld.final-executable-launcher.toml"),
            &nsb_path,
            Some(&dir),
            "nuis.scheduler.loop.v1",
            "on_process_start",
            &manifest,
        );

        assert!(!report.ready);
        assert_eq!(
            report.container_loader_entry_kind.as_deref(),
            Some("host-entry-bootstrap")
        );
        assert_eq!(
            report.container_loader_symbol_kind.as_deref(),
            Some("lifecycle-bootstrap")
        );
        assert_eq!(report.container_loader_handoff_status, "blocked");
        assert!(report
            .container_loader_handoff_blockers
            .contains(&"container-loader:entry-kind-mismatch".to_owned()));
        assert!(report
            .blockers
            .contains(&"container-loader:entry-kind-mismatch".to_owned()));
        let _ = fs::remove_dir_all(&dir);
    }
}
