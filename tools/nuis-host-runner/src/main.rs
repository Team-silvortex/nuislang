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
    let prefix = format!("{key} =");
    source
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&prefix))
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
        "{{\"kind\":\"nuis_host_runner\",\"protocol\":\"{}\",\"ready\":{},\"would_enter_lifecycle_hook\":{},\"manifest_path\":\"{}\",\"nsb_path\":{},\"nsb_readable\":{},\"nsb_hash_expected\":{},\"nsb_hash_actual\":{},\"nsb_hash_matches\":{},\"nsb_payload_offset\":{},\"nsb_payload_span\":{},\"nsb_layout_hash\":{},\"nsb_byte_map_hash\":{},\"scheduler_entry\":\"{}\",\"lifecycle_hook\":\"{}\",\"launch_steps\":[{}],\"blockers\":[{}]}}",
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

    fn nsb_bytes() -> Vec<u8> {
        let mut bytes = vec![0u8; IMAGE_HEADER_SIZE + 8];
        bytes[0..8].copy_from_slice(IMAGE_MAGIC);
        bytes[8..12].copy_from_slice(&IMAGE_VERSION.to_le_bytes());
        bytes[12..16].copy_from_slice(&(IMAGE_HEADER_SIZE as u32).to_le_bytes());
        bytes[24..32].copy_from_slice(&(8u64).to_le_bytes());
        bytes[32..40].copy_from_slice(&(IMAGE_HEADER_SIZE as u64).to_le_bytes());
        bytes[40..48].copy_from_slice(&0x1234u64.to_le_bytes());
        bytes[48..56].copy_from_slice(&0x5678u64.to_le_bytes());
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
        assert_eq!(report.nsb_payload_span, Some(8));
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
}
