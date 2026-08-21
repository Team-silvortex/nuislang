use crate::{
    final_executable_macho_shell_image::MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT,
    final_executable_macho_shell_signature_validation::{
        MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT, MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT,
    },
    reports::{NsldMachOArm64LoaderProbeReport, NsldMachOArm64ShellImageSerializationReport},
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

pub(crate) const MACHO_ARM64_LOADER_PROBE_CONTRACT: &str =
    "nuis-nsld-macho-arm64-os-loader-probe-v1";
pub(crate) const MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND: &str =
    "create-new-owner-executable-temporary-file";
pub(crate) const MACHO_ARM64_LOADER_PROBE_TIMEOUT_MILLIS: u64 = 5_000;
const PROBE_TIMEOUT: Duration = Duration::from_millis(MACHO_ARM64_LOADER_PROBE_TIMEOUT_MILLIS);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const CAPTURE_LIMIT_BYTES: u64 = 1024 * 1024;
static PROBE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct MachOArm64LoaderProbeInput<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) serialization: &'a NsldMachOArm64ShellImageSerializationReport,
    pub(crate) unresolved_external_symbol_count: usize,
    pub(crate) bind_count: usize,
}

#[derive(Debug)]
struct ProbeObservation {
    status: String,
    attempted: bool,
    materialized: bool,
    materialized_hash_matches: bool,
    kernel_accepted: bool,
    process_completed: bool,
    timed_out: bool,
    exit_code: Option<i32>,
    termination_signal: Option<i32>,
    stdout: CapturedOutput,
    stderr: CapturedOutput,
    failure_kind: Option<String>,
    cleanup_attempted: bool,
    cleanup_succeeded: bool,
    blockers: Vec<String>,
}

impl ProbeObservation {
    fn blocked(status: &str, blocker: &str) -> Self {
        Self {
            status: status.to_owned(),
            attempted: false,
            materialized: false,
            materialized_hash_matches: false,
            kernel_accepted: false,
            process_completed: false,
            timed_out: false,
            exit_code: None,
            termination_signal: None,
            stdout: CapturedOutput::empty(),
            stderr: CapturedOutput::empty(),
            failure_kind: None,
            cleanup_attempted: false,
            cleanup_succeeded: true,
            blockers: vec![blocker.to_owned()],
        }
    }
}

#[derive(Debug)]
struct CapturedOutput {
    bytes: usize,
    truncated: bool,
    hash: String,
}

impl CapturedOutput {
    fn empty() -> Self {
        Self {
            bytes: 0,
            truncated: false,
            hash: crate::fnv1a64_hex(&[]),
        }
    }
}

struct ProbePaths {
    executable: PathBuf,
    stdout: PathBuf,
    stderr: PathBuf,
}

#[derive(Default)]
struct ProbeCreatedPaths {
    executable: bool,
    stdout: bool,
    stderr: bool,
}

pub(crate) fn probe_macho_arm64_signed_shell_image(
    input: MachOArm64LoaderProbeInput<'_>,
    probe_root: &Path,
    execute: bool,
) -> Result<NsldMachOArm64LoaderProbeReport, String> {
    validate_input(&input)?;
    let host_supported = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let input_eligible = input.unresolved_external_symbol_count == 0 && input.bind_count == 0;
    let observation = if !host_supported {
        ProbeObservation::blocked("blocked-unsupported-probe-host", "unsupported-probe-host")
    } else if !input_eligible {
        ProbeObservation::blocked(
            "blocked-external-compatibility-input",
            "private-image-has-external-compatibility-bindings",
        )
    } else if !execute {
        ProbeObservation::blocked(
            "ready-explicit-apply-required",
            "explicit-loader-probe-apply-required",
        )
    } else {
        execute_probe(&input, probe_root)?
    };
    Ok(build_report(
        &input,
        execute,
        host_supported,
        input_eligible,
        observation,
    ))
}

fn validate_input(input: &MachOArm64LoaderProbeInput<'_>) -> Result<(), String> {
    let report = input.serialization;
    if report.contract != MACHO_ARM64_SHELL_IMAGE_SERIALIZATION_CONTRACT
        || report.status != "signed-private-image-validated"
        || report.code_signature.validation_contract != MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT
        || report.code_signature.validation_status != "signed-private-image-structurally-valid"
        || report.code_signature.publication_eligibility_contract
            != MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT
    {
        return Err("Mach-O loader probe rejects the serialization contract".to_owned());
    }
    if input.bytes.len() != report.shell_image_span_bytes
        || crate::fnv1a64_hex(input.bytes) != report.shell_image_hash
        || report.code_signature.signature_file_offset
            + report.code_signature.signature_payload_bytes
            != input.bytes.len()
        || report.code_signature.verified_code_slot_count != report.code_signature.code_slot_count
    {
        return Err("Mach-O loader probe rejects private image drift".to_owned());
    }
    Ok(())
}

fn execute_probe(
    input: &MachOArm64LoaderProbeInput<'_>,
    probe_root: &Path,
) -> Result<ProbeObservation, String> {
    let probe_root = fs::canonicalize(probe_root)
        .map_err(|error| format!("failed to resolve Mach-O loader probe root: {error}"))?;
    if !probe_root.is_dir() {
        return Err("Mach-O loader probe root is not a directory".to_owned());
    }
    let paths = unique_probe_paths(&probe_root);
    let mut created = ProbeCreatedPaths::default();
    let result = execute_probe_inner(input, &probe_root, &paths, &mut created);
    let cleanup_succeeded = cleanup_probe_paths(&paths, &created);
    match result {
        Ok(mut observation) => {
            observation.cleanup_attempted = true;
            observation.cleanup_succeeded = cleanup_succeeded;
            if !cleanup_succeeded {
                observation.blockers.push("probe-cleanup-failed".to_owned());
                observation.status = "probe-completed-cleanup-failed".to_owned();
            }
            Ok(observation)
        }
        Err(error) => {
            if cleanup_succeeded {
                Err(error)
            } else {
                Err(format!("{error}; Mach-O loader probe cleanup failed"))
            }
        }
    }
}

fn execute_probe_inner(
    input: &MachOArm64LoaderProbeInput<'_>,
    probe_root: &Path,
    paths: &ProbePaths,
    created: &mut ProbeCreatedPaths,
) -> Result<ProbeObservation, String> {
    let mut executable = create_new_file(&paths.executable, "executable", 0o700)?;
    created.executable = true;
    executable
        .write_all(input.bytes)
        .map_err(|error| format!("failed to write Mach-O loader probe image: {error}"))?;
    executable
        .sync_all()
        .map_err(|error| format!("failed to sync Mach-O loader probe image: {error}"))?;
    set_owner_executable(&executable)?;
    drop(executable);
    let materialized_bytes = fs::read(&paths.executable)
        .map_err(|error| format!("failed to verify Mach-O loader probe image: {error}"))?;
    let materialized_hash_matches = materialized_bytes == input.bytes;
    if !materialized_hash_matches {
        return Ok(failed_materialization_observation());
    }

    let stdout = create_new_file(&paths.stdout, "stdout capture", 0o600)?;
    created.stdout = true;
    let stderr = create_new_file(&paths.stderr, "stderr capture", 0o600)?;
    created.stderr = true;
    let spawn = Command::new(&paths.executable)
        .current_dir(probe_root)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn();
    let mut child = match spawn {
        Ok(child) => child,
        Err(error) => {
            return Ok(ProbeObservation {
                status: "os-loader-rejected-and-cleaned".to_owned(),
                attempted: true,
                materialized: true,
                materialized_hash_matches: true,
                kernel_accepted: false,
                process_completed: false,
                timed_out: false,
                exit_code: None,
                termination_signal: None,
                stdout: read_capture(&paths.stdout)?,
                stderr: read_capture(&paths.stderr)?,
                failure_kind: Some(format!("spawn-{:?}", error.kind()).to_ascii_lowercase()),
                cleanup_attempted: false,
                cleanup_succeeded: false,
                blockers: vec!["os-loader-rejected-private-image".to_owned()],
            });
        }
    };
    let (status, timed_out, wait_failure) = wait_bounded(&mut child, paths);
    let stdout = read_capture(&paths.stdout)?;
    let stderr = read_capture(&paths.stderr)?;
    let exit_code = status.as_ref().and_then(ExitStatus::code);
    let termination_signal = status.as_ref().and_then(exit_status_signal);
    let process_completed = status.is_some() && !timed_out && wait_failure.is_none();
    let capture_limit_exceeded = stdout.truncated
        || stderr.truncated
        || wait_failure.as_deref() == Some("capture-limit-exceeded");
    let successful = process_completed && exit_code == Some(0) && !capture_limit_exceeded;
    let mut blockers = Vec::new();
    if timed_out {
        blockers.push("loader-probe-timeout".to_owned());
    } else if capture_limit_exceeded {
        blockers.push("loader-probe-output-limit-exceeded".to_owned());
    } else if wait_failure.is_some() {
        blockers.push("loader-probe-wait-failed".to_owned());
    } else if !successful {
        blockers.push("loader-probe-process-unsuccessful".to_owned());
    }
    Ok(ProbeObservation {
        status: if successful {
            "os-loader-accepted-process-succeeded".to_owned()
        } else if timed_out {
            "os-loader-accepted-process-timed-out".to_owned()
        } else if capture_limit_exceeded {
            "os-loader-accepted-output-limit-exceeded".to_owned()
        } else {
            "os-loader-accepted-process-unsuccessful".to_owned()
        },
        attempted: true,
        materialized: true,
        materialized_hash_matches: true,
        kernel_accepted: true,
        process_completed,
        timed_out,
        exit_code,
        termination_signal,
        stdout,
        stderr,
        failure_kind: wait_failure,
        cleanup_attempted: false,
        cleanup_succeeded: false,
        blockers,
    })
}

fn failed_materialization_observation() -> ProbeObservation {
    let mut observation = ProbeObservation::blocked(
        "blocked-materialized-image-drift",
        "materialized-private-image-hash-mismatch",
    );
    observation.attempted = true;
    observation.materialized = true;
    observation
}

fn wait_bounded(
    child: &mut std::process::Child,
    paths: &ProbePaths,
) -> (Option<ExitStatus>, bool, Option<String>) {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (Some(status), false, None),
            Ok(None) if capture_limit_exceeded(paths) => {
                let _ = child.kill();
                return (
                    child.wait().ok(),
                    false,
                    Some("capture-limit-exceeded".to_owned()),
                );
            }
            Ok(None) if start.elapsed() < PROBE_TIMEOUT => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                let kill_failure = child
                    .kill()
                    .err()
                    .map(|error| format!("kill-{:?}", error.kind()).to_ascii_lowercase());
                let status = child.wait().ok();
                return (status, true, kill_failure);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return (
                    None,
                    false,
                    Some(format!("wait-{:?}", error.kind()).to_ascii_lowercase()),
                );
            }
        }
    }
}

fn capture_limit_exceeded(paths: &ProbePaths) -> bool {
    [&paths.stdout, &paths.stderr].into_iter().any(|path| {
        fs::metadata(path)
            .map(|metadata| metadata.len() > CAPTURE_LIMIT_BYTES)
            .unwrap_or(true)
    })
}

fn read_capture(path: &Path) -> Result<CapturedOutput, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("failed to open Mach-O loader probe capture: {error}"))?;
    let total = file
        .metadata()
        .map_err(|error| format!("failed to inspect Mach-O loader probe capture: {error}"))?
        .len();
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(CAPTURE_LIMIT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read Mach-O loader probe capture: {error}"))?;
    Ok(CapturedOutput {
        bytes: bytes.len(),
        truncated: total > CAPTURE_LIMIT_BYTES,
        hash: crate::fnv1a64_hex(&bytes),
    })
}

fn build_report(
    input: &MachOArm64LoaderProbeInput<'_>,
    execute: bool,
    host_supported: bool,
    input_eligible: bool,
    observation: ProbeObservation,
) -> NsldMachOArm64LoaderProbeReport {
    let publication_eligible = observation.attempted
        && observation.materialized_hash_matches
        && observation.kernel_accepted
        && observation.process_completed
        && observation.exit_code == Some(0)
        && observation.cleanup_succeeded
        && observation.blockers.is_empty();
    let publication_eligibility_status = if publication_eligible {
        "eligible-isolated-os-loader-probe-passed"
    } else {
        "blocked-isolated-os-loader-probe-incomplete"
    };
    let mut report = NsldMachOArm64LoaderProbeReport {
        contract: MACHO_ARM64_LOADER_PROBE_CONTRACT.to_owned(),
        status: observation.status,
        probe_mode: if execute { "execute" } else { "plan-only" }.to_owned(),
        materialization_kind: MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND.to_owned(),
        target_arch: "aarch64".to_owned(),
        target_os: "macos".to_owned(),
        host_supported,
        input_eligible,
        attempted: observation.attempted,
        image_span_bytes: input.bytes.len(),
        shell_image_hash: input.serialization.shell_image_hash.clone(),
        signature_validation_ledger_hash: input
            .serialization
            .code_signature
            .validation_ledger_hash
            .clone(),
        unresolved_external_symbol_count: input.unresolved_external_symbol_count,
        bind_count: input.bind_count,
        probe_timeout_millis: PROBE_TIMEOUT.as_millis() as u64,
        materialized: observation.materialized,
        materialized_hash_matches: observation.materialized_hash_matches,
        kernel_accepted: observation.kernel_accepted,
        process_completed: observation.process_completed,
        timed_out: observation.timed_out,
        exit_code: observation.exit_code,
        termination_signal: observation.termination_signal,
        stdout_captured_bytes: observation.stdout.bytes,
        stdout_truncated: observation.stdout.truncated,
        stdout_hash: observation.stdout.hash,
        stderr_captured_bytes: observation.stderr.bytes,
        stderr_truncated: observation.stderr.truncated,
        stderr_hash: observation.stderr.hash,
        failure_kind: observation.failure_kind,
        cleanup_attempted: observation.cleanup_attempted,
        cleanup_succeeded: observation.cleanup_succeeded,
        publication_eligibility_contract: MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT.to_owned(),
        publication_eligibility_status: publication_eligibility_status.to_owned(),
        publication_eligible,
        publication_blockers: observation.blockers,
        probe_ledger_hash: String::new(),
        admission_receipt_file: None,
        admission_receipt_persisted: false,
        admission_receipt_hash_sha256: None,
        admission_receipt_validation_status: "not-requested".to_owned(),
    };
    report.probe_ledger_hash = probe_ledger_hash(&report);
    report
}

pub(crate) fn validate_successful_macho_arm64_loader_probe(
    report: &NsldMachOArm64LoaderProbeReport,
) -> Result<(), String> {
    if report.contract != MACHO_ARM64_LOADER_PROBE_CONTRACT
        || report.status != "os-loader-accepted-process-succeeded"
        || report.probe_mode != "execute"
        || report.materialization_kind != MACHO_ARM64_LOADER_PROBE_MATERIALIZATION_KIND
        || report.target_arch != "aarch64"
        || report.target_os != "macos"
    {
        return Err("Mach-O admission rejects the loader-probe contract identity".to_owned());
    }
    if !report.host_supported
        || !report.input_eligible
        || !report.attempted
        || report.unresolved_external_symbol_count != 0
        || report.bind_count != 0
    {
        return Err("Mach-O admission rejects loader-probe input eligibility".to_owned());
    }
    if !report.materialized
        || !report.materialized_hash_matches
        || !report.kernel_accepted
        || !report.process_completed
        || report.timed_out
        || report.exit_code != Some(0)
        || report.termination_signal.is_some()
        || report.stdout_truncated
        || report.stderr_truncated
        || report.failure_kind.is_some()
    {
        return Err("Mach-O admission rejects unsuccessful loader-probe execution".to_owned());
    }
    if !report.cleanup_attempted || !report.cleanup_succeeded {
        return Err("Mach-O admission rejects incomplete loader-probe cleanup".to_owned());
    }
    if report.publication_eligibility_contract != MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT
        || report.publication_eligibility_status != "eligible-isolated-os-loader-probe-passed"
        || !report.publication_eligible
        || !report.publication_blockers.is_empty()
    {
        return Err("Mach-O admission rejects loader-probe publication eligibility".to_owned());
    }
    if report.probe_ledger_hash != probe_ledger_hash(report) {
        return Err("Mach-O admission rejects loader-probe ledger drift".to_owned());
    }
    Ok(())
}

fn probe_ledger_hash(report: &NsldMachOArm64LoaderProbeReport) -> String {
    let mut material = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}\n",
        report.contract,
        report.status,
        report.probe_mode,
        report.materialization_kind,
        report.target_arch,
        report.target_os,
        report.host_supported,
        report.input_eligible,
        report.attempted,
        report.image_span_bytes,
        report.shell_image_hash,
        report.signature_validation_ledger_hash,
        report.unresolved_external_symbol_count,
        report.bind_count,
        report.probe_timeout_millis,
        report.materialized,
        report.materialized_hash_matches,
        report.kernel_accepted,
        report.process_completed,
        report.timed_out,
        option_i32(report.exit_code),
        option_i32(report.termination_signal),
        report.stdout_captured_bytes,
        report.stdout_truncated,
        report.stdout_hash,
        report.stderr_captured_bytes,
        report.stderr_truncated,
        report.stderr_hash,
        report.failure_kind.as_deref().unwrap_or("none"),
        report.cleanup_attempted,
        report.cleanup_succeeded,
    );
    material.push_str(&format!(
        "{}|{}|{}\n",
        report.publication_eligibility_contract,
        report.publication_eligibility_status,
        report.publication_eligible
    ));
    for blocker in &report.publication_blockers {
        material.push_str("blocker=");
        material.push_str(blocker);
        material.push('\n');
    }
    crate::fnv1a64_hex(material.as_bytes())
}

fn unique_probe_paths(root: &Path) -> ProbePaths {
    let sequence = PROBE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let base = format!(".nsld-macho-loader-probe-{}-{sequence}", std::process::id());
    ProbePaths {
        executable: root.join(format!("{base}.bin")),
        stdout: root.join(format!("{base}.stdout")),
        stderr: root.join(format!("{base}.stderr")),
    }
}

fn create_new_file(path: &Path, label: &str, owner_mode: u32) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(owner_mode);
    }
    #[cfg(not(unix))]
    let _ = owner_mode;
    options
        .open(path)
        .map_err(|error| format!("failed to create Mach-O loader probe {label}: {error}"))
}

fn cleanup_probe_paths(paths: &ProbePaths, created: &ProbeCreatedPaths) -> bool {
    let mut clean = true;
    for (path, owned) in [
        (&paths.executable, created.executable),
        (&paths.stdout, created.stdout),
        (&paths.stderr, created.stderr),
    ] {
        if !owned {
            continue;
        }
        let removed = match fs::remove_file(path) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        };
        clean &= removed;
    }
    clean
}

#[cfg(unix)]
fn set_owner_executable(file: &File) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = file
        .metadata()
        .map_err(|error| format!("failed to inspect Mach-O loader probe image: {error}"))?
        .permissions();
    permissions.set_mode(0o700);
    file.set_permissions(permissions)
        .map_err(|error| format!("failed to mark Mach-O loader probe image executable: {error}"))
}

#[cfg(not(unix))]
fn set_owner_executable(_file: &File) -> Result<(), String> {
    Err("Mach-O loader probe executable permissions require Unix".to_owned())
}

#[cfg(unix)]
fn exit_status_signal(status: &ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn exit_status_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

fn option_i32(value: Option<i32>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

#[cfg(test)]
#[path = "final_executable_macho_loader_probe_tests.rs"]
mod tests;
