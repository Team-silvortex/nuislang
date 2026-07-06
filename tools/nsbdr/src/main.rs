use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Status,
    Plan {
        final_output: PathBuf,
        package_output_dir: PathBuf,
        target: Option<String>,
        json: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsbdrPlanReport {
    final_output_path: String,
    final_output_present: bool,
    final_output_size_bytes: Option<usize>,
    final_output_hash: Option<String>,
    package_output_dir: String,
    target_id: String,
    target_os: String,
    package_kind: String,
    staging_dir: String,
    primary_bundle_path: String,
    primary_package_path: String,
    host_os: String,
    host_matches_target: bool,
    package_candidates: Vec<NsbdrPackageCandidate>,
    can_emit_primary_bundle: bool,
    can_emit_primary_package: bool,
    blockers: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsbdrPackageCandidate {
    target_id: String,
    target_os: String,
    package_kind: String,
    bundle_kind: String,
    package_extension: String,
    native_distribution_standard: String,
    host_tool_names: Vec<String>,
    host_tools_available: Vec<String>,
    host_tools_missing: Vec<String>,
    planned_bundle_path: String,
    planned_package_path: String,
    can_emit: bool,
    blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackageTargetSpec {
    target_id: &'static str,
    target_os: &'static str,
    package_kind: &'static str,
    bundle_kind: &'static str,
    package_extension: &'static str,
    native_distribution_standard: &'static str,
    host_tool_names: &'static [&'static str],
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match parse_args(env::args().skip(1))? {
        Command::Status => print_status(),
        Command::Plan {
            final_output,
            package_output_dir,
            target,
            json,
        } => {
            let report = nsbdr_plan_report(&final_output, &package_output_dir, target.as_deref())?;
            if json {
                println!("{}", report_json(&report));
            } else {
                print_report(&report);
            }
        }
    }
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Err(usage().to_owned());
    };
    match command.as_str() {
        "status" => Ok(Command::Status),
        "plan" => {
            let final_output = args.next().ok_or_else(|| usage().to_owned())?;
            let package_output_dir = args.next().ok_or_else(|| usage().to_owned())?;
            let mut json = false;
            let mut target = None;
            for arg in args {
                if arg == "--json" {
                    json = true;
                } else if let Some(value) = arg.strip_prefix("--target=") {
                    target = Some(value.to_owned());
                } else {
                    return Err(format!("unexpected argument `{arg}`\n{}", usage()));
                }
            }
            Ok(Command::Plan {
                final_output: PathBuf::from(final_output),
                package_output_dir: PathBuf::from(package_output_dir),
                target,
                json,
            })
        }
        "--help" | "-h" | "help" => Err(usage().to_owned()),
        other => Err(format!("unknown nsbdr command `{other}`\n{}", usage())),
    }
}

fn usage() -> &'static str {
    concat!(
        "usage:\n",
        "  nsbdr status\n",
        "  nsbdr plan <nsld-final-output> <package-output-dir> [--target=<target-id>] [--json]\n"
    )
}

fn print_status() {
    println!("Nsbdr OS bundle/distribution front-door");
    println!("  tool: nsbdr");
    println!("  phase: alpha-0.8.x packaging boundary");
    println!("  owns: cross-platform OS bundle / package distribution plans");
    println!("  does_not_own: nsld linker graph or core binary assembly");
    println!("  mutating_packagers: disabled");
}

fn nsbdr_plan_report(
    final_output: &Path,
    package_output_dir: &Path,
    target: Option<&str>,
) -> Result<NsbdrPlanReport, String> {
    let final_bytes = fs::read(final_output);
    let final_output_present = final_bytes.is_ok();
    let final_output_size_bytes = final_bytes.as_ref().ok().map(Vec::len);
    let final_output_hash = final_bytes.as_ref().ok().map(|bytes| fnv1a64_hex(bytes));
    let package_name = final_output
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("NuisApp");
    let staging_dir = package_output_dir.join("staging");
    let host_os = current_host_os();
    let target_id = target
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_target_for_host(&host_os).to_owned());
    let target_spec = package_target_specs()
        .into_iter()
        .find(|spec| spec.target_id == target_id)
        .ok_or_else(|| format!("unknown nsbdr target `{target_id}`"))?;
    let package_candidates = package_target_specs()
        .into_iter()
        .map(|spec| package_candidate(&spec, package_name, package_output_dir, &staging_dir))
        .collect::<Vec<_>>();
    let primary = package_candidates
        .iter()
        .find(|candidate| candidate.target_id == target_id)
        .expect("target spec must have a matching candidate");
    let host_matches_target = host_os == target_spec.target_os;
    let mut blockers = Vec::new();
    if !final_output_present {
        blockers.push("final-output:missing".to_owned());
    }
    if !host_matches_target {
        blockers.push(format!(
            "host-target-mismatch:{}:{}",
            host_os, target_spec.target_os
        ));
    }
    blockers.extend(primary.blockers.iter().cloned());
    let notes = vec![
        "nsbdr-consumes-nsld-final-output".to_owned(),
        "nsbdr-does-not-link-or-rewrite-core-binary".to_owned(),
        "os-package-emission-is-planned-but-non-mutating".to_owned(),
        "signing-notarization-and-store-upload-are-future-policy-gates".to_owned(),
        "nuis-os-package-is-a-first-class-future-target".to_owned(),
    ];

    Ok(NsbdrPlanReport {
        final_output_path: final_output.display().to_string(),
        final_output_present,
        final_output_size_bytes,
        final_output_hash,
        package_output_dir: package_output_dir.display().to_string(),
        target_id,
        target_os: target_spec.target_os.to_owned(),
        package_kind: target_spec.package_kind.to_owned(),
        staging_dir: staging_dir.display().to_string(),
        primary_bundle_path: primary.planned_bundle_path.clone(),
        primary_package_path: primary.planned_package_path.clone(),
        host_os,
        host_matches_target,
        package_candidates,
        can_emit_primary_bundle: false,
        can_emit_primary_package: false,
        blockers,
        notes,
    })
}

fn resolve_host_tool(tool: &str) -> Option<String> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths).find_map(|dir| {
        let candidate = dir.join(tool);
        candidate.is_file().then(|| candidate.display().to_string())
    })
}

fn current_host_os() -> String {
    match env::consts::OS {
        "macos" => "macos".to_owned(),
        "windows" => "windows".to_owned(),
        "linux" => "linux".to_owned(),
        other => other.to_owned(),
    }
}

fn default_target_for_host(host_os: &str) -> &'static str {
    match host_os {
        "macos" => "macos.dmg",
        "windows" => "windows.msix",
        "linux" => "linux.appimage",
        _ => "nuisos.nspkg",
    }
}

fn package_target_specs() -> Vec<PackageTargetSpec> {
    vec![
        PackageTargetSpec {
            target_id: "macos.dmg",
            target_os: "macos",
            package_kind: "macos-dmg",
            bundle_kind: "macos-app-bundle",
            package_extension: "dmg",
            native_distribution_standard: "macos-app-dmg",
            host_tool_names: &["hdiutil", "codesign", "productbuild"],
        },
        PackageTargetSpec {
            target_id: "windows.msix",
            target_os: "windows",
            package_kind: "windows-msix",
            bundle_kind: "windows-app-layout",
            package_extension: "msix",
            native_distribution_standard: "windows-msix",
            host_tool_names: &["makeappx", "signtool"],
        },
        PackageTargetSpec {
            target_id: "linux.appimage",
            target_os: "linux",
            package_kind: "linux-appimage",
            bundle_kind: "linux-appdir",
            package_extension: "AppImage",
            native_distribution_standard: "linux-appimage",
            host_tool_names: &["appimagetool"],
        },
        PackageTargetSpec {
            target_id: "nuisos.nspkg",
            target_os: "nuisos",
            package_kind: "nuisos-nspkg",
            bundle_kind: "nuisos-package-root",
            package_extension: "nspkg",
            native_distribution_standard: "nuis-os-native-package",
            host_tool_names: &["nsbdr"],
        },
    ]
}

fn package_candidate(
    spec: &PackageTargetSpec,
    package_name: &str,
    package_output_dir: &Path,
    staging_dir: &Path,
) -> NsbdrPackageCandidate {
    let bundle_suffix = match spec.bundle_kind {
        "macos-app-bundle" => ".app",
        "windows-app-layout" => ".appxroot",
        "linux-appdir" => ".AppDir",
        "nuisos-package-root" => ".nspkgroot",
        _ => ".bundle",
    };
    let planned_bundle_path = staging_dir.join(format!("{package_name}{bundle_suffix}"));
    let planned_package_path =
        package_output_dir.join(format!("{package_name}.{}", spec.package_extension));
    let host_tools_available = spec
        .host_tool_names
        .iter()
        .filter(|tool| resolve_host_tool(tool).is_some())
        .map(|tool| (*tool).to_owned())
        .collect::<Vec<_>>();
    let host_tools_missing = spec
        .host_tool_names
        .iter()
        .filter(|tool| resolve_host_tool(tool).is_none())
        .map(|tool| (*tool).to_owned())
        .collect::<Vec<_>>();
    let mut blockers = vec![
        format!("bundle-writer:{}:not-implemented", spec.bundle_kind),
        format!("package-writer:{}:not-implemented", spec.package_kind),
    ];
    blockers.extend(
        host_tools_missing
            .iter()
            .map(|tool| format!("host-tool:{tool}:unavailable")),
    );

    NsbdrPackageCandidate {
        target_id: spec.target_id.to_owned(),
        target_os: spec.target_os.to_owned(),
        package_kind: spec.package_kind.to_owned(),
        bundle_kind: spec.bundle_kind.to_owned(),
        package_extension: spec.package_extension.to_owned(),
        native_distribution_standard: spec.native_distribution_standard.to_owned(),
        host_tool_names: spec
            .host_tool_names
            .iter()
            .map(|tool| (*tool).to_owned())
            .collect(),
        host_tools_available,
        host_tools_missing,
        planned_bundle_path: planned_bundle_path.display().to_string(),
        planned_package_path: planned_package_path.display().to_string(),
        can_emit: false,
        blockers,
    }
}

fn print_report(report: &NsbdrPlanReport) {
    println!("Nsbdr package plan");
    println!("  target_id: {}", report.target_id);
    println!("  target_os: {}", report.target_os);
    println!("  package_kind: {}", report.package_kind);
    println!("  final_output_path: {}", report.final_output_path);
    println!("  final_output_present: {}", report.final_output_present);
    println!(
        "  final_output_size_bytes: {}",
        optional_usize_text(report.final_output_size_bytes)
    );
    println!(
        "  final_output_hash: {}",
        report.final_output_hash.as_deref().unwrap_or("missing")
    );
    println!("  package_output_dir: {}", report.package_output_dir);
    println!("  staging_dir: {}", report.staging_dir);
    println!("  primary_bundle_path: {}", report.primary_bundle_path);
    println!("  primary_package_path: {}", report.primary_package_path);
    println!("  host_os: {}", report.host_os);
    println!("  host_matches_target: {}", report.host_matches_target);
    println!(
        "  can_emit_primary_bundle: {}",
        report.can_emit_primary_bundle
    );
    println!(
        "  can_emit_primary_package: {}",
        report.can_emit_primary_package
    );
    for candidate in &report.package_candidates {
        println!(
            "  package_candidate: target={} os={} package={} bundle={} standard={} can_emit={} package_path={} tools_available={} tools_missing={} blockers={}",
            candidate.target_id,
            candidate.target_os,
            candidate.package_kind,
            candidate.bundle_kind,
            candidate.native_distribution_standard,
            candidate.can_emit,
            candidate.planned_package_path,
            candidate.host_tools_available.join("|"),
            candidate.host_tools_missing.join("|"),
            candidate.blockers.len()
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

fn report_json(report: &NsbdrPlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsbdr"),
        json_string_field("kind", "nsbdr_package_plan"),
        json_string_field("target_id", &report.target_id),
        json_string_field("target_os", &report.target_os),
        json_string_field("package_kind", &report.package_kind),
        json_string_field("final_output_path", &report.final_output_path),
        json_bool_field("final_output_present", report.final_output_present),
        json_optional_usize_field("final_output_size_bytes", report.final_output_size_bytes),
        json_optional_string_field("final_output_hash", report.final_output_hash.as_deref()),
        json_string_field("package_output_dir", &report.package_output_dir),
        json_string_field("staging_dir", &report.staging_dir),
        json_string_field("primary_bundle_path", &report.primary_bundle_path),
        json_string_field("primary_package_path", &report.primary_package_path),
        json_string_field("host_os", &report.host_os),
        json_bool_field("host_matches_target", report.host_matches_target),
        json_bool_field("can_emit_primary_bundle", report.can_emit_primary_bundle),
        json_bool_field("can_emit_primary_package", report.can_emit_primary_package),
        format!(
            "\"package_candidates\":[{}]",
            package_candidates_json(&report.package_candidates)
        ),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

fn package_candidates_json(candidates: &[NsbdrPackageCandidate]) -> String {
    candidates
        .iter()
        .map(|candidate| {
            let fields = vec![
                json_string_field("target_id", &candidate.target_id),
                json_string_field("target_os", &candidate.target_os),
                json_string_field("package_kind", &candidate.package_kind),
                json_string_field("bundle_kind", &candidate.bundle_kind),
                json_string_field("package_extension", &candidate.package_extension),
                json_string_field(
                    "native_distribution_standard",
                    &candidate.native_distribution_standard,
                ),
                json_string_array_field("host_tool_names", &candidate.host_tool_names),
                json_string_array_field("host_tools_available", &candidate.host_tools_available),
                json_string_array_field("host_tools_missing", &candidate.host_tools_missing),
                json_string_field("planned_bundle_path", &candidate.planned_bundle_path),
                json_string_field("planned_package_path", &candidate.planned_package_path),
                json_bool_field("can_emit", candidate.can_emit),
                json_string_array_field("blockers", &candidate.blockers),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn json_string_field(key: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", escape_json(key), escape_json(value))
}

fn json_bool_field(key: &str, value: bool) -> String {
    format!("\"{}\":{}", escape_json(key), value)
}

fn json_optional_usize_field(key: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => format!("\"{}\":{}", escape_json(key), value),
        None => format!("\"{}\":null", escape_json(key)),
    }
}

fn json_optional_string_field(key: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => json_string_field(key, value),
        None => format!("\"{}\":null", escape_json(key)),
    }
}

fn json_string_array_field(key: &str, values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", escape_json(key), values)
}

fn escape_json(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{fnv1a64_hex, nsbdr_plan_report, parse_args, Command};
    use std::{fs, path::PathBuf};

    #[test]
    fn parses_plan_command_with_json_flag() {
        let command = parse_args([
            "plan".to_owned(),
            "build/app.nsb".to_owned(),
            "dist".to_owned(),
            "--json".to_owned(),
        ])
        .unwrap();

        assert_eq!(
            command,
            Command::Plan {
                final_output: PathBuf::from("build/app.nsb"),
                package_output_dir: PathBuf::from("dist"),
                target: None,
                json: true,
            }
        );
    }

    #[test]
    fn parses_plan_command_with_explicit_target() {
        let command = parse_args([
            "plan".to_owned(),
            "build/app.nsb".to_owned(),
            "dist".to_owned(),
            "--target=nuisos.nspkg".to_owned(),
        ])
        .unwrap();

        assert_eq!(
            command,
            Command::Plan {
                final_output: PathBuf::from("build/app.nsb"),
                package_output_dir: PathBuf::from("dist"),
                target: Some("nuisos.nspkg".to_owned()),
                json: false,
            }
        );
    }

    #[test]
    fn package_plan_reports_final_output_hash_and_non_mutating_blockers() {
        let dir = std::env::temp_dir().join(format!("nsbdr-plan-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let final_output = dir.join("demo.nsb");
        fs::write(&final_output, b"nuis-final-output").unwrap();
        let package_dir = dir.join("dist");

        let report = nsbdr_plan_report(&final_output, &package_dir, Some("macos.dmg")).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert!(report.final_output_present);
        assert_eq!(report.final_output_size_bytes, Some(17));
        assert_eq!(
            report.final_output_hash,
            Some(fnv1a64_hex(b"nuis-final-output"))
        );
        assert_eq!(report.target_id, "macos.dmg");
        assert_eq!(report.target_os, "macos");
        assert_eq!(report.package_kind, "macos-dmg");
        assert!(report.primary_bundle_path.ends_with("demo.app"));
        assert!(report.primary_package_path.ends_with("demo.dmg"));
        assert_eq!(report.package_candidates.len(), 4);
        assert!(report
            .package_candidates
            .iter()
            .any(|candidate| candidate.target_id == "windows.msix"));
        assert!(report
            .package_candidates
            .iter()
            .any(|candidate| candidate.target_id == "linux.appimage"));
        assert!(report
            .package_candidates
            .iter()
            .any(|candidate| candidate.target_id == "nuisos.nspkg"));
        assert!(!report.can_emit_primary_bundle);
        assert!(!report.can_emit_primary_package);
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker == "bundle-writer:macos-app-bundle:not-implemented"));
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker == "package-writer:macos-dmg:not-implemented"));
    }

    #[test]
    fn package_plan_can_target_future_nuis_os_package() {
        let dir = std::env::temp_dir().join(format!("nsbdr-nuisos-plan-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let final_output = dir.join("demo.nsb");
        fs::write(&final_output, b"nuis-final-output").unwrap();
        let package_dir = dir.join("dist");

        let report = nsbdr_plan_report(&final_output, &package_dir, Some("nuisos.nspkg")).unwrap();
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(report.target_id, "nuisos.nspkg");
        assert_eq!(report.target_os, "nuisos");
        assert_eq!(report.package_kind, "nuisos-nspkg");
        assert!(report.primary_bundle_path.ends_with("demo.nspkgroot"));
        assert!(report.primary_package_path.ends_with("demo.nspkg"));
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker == "bundle-writer:nuisos-package-root:not-implemented"));
        assert!(report
            .blockers
            .iter()
            .any(|blocker| blocker == "package-writer:nuisos-nspkg:not-implemented"));
    }
}
