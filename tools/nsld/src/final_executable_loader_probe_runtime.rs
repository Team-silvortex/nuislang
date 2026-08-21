use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

pub(crate) const LOADER_PROBE_MATERIALIZATION_KIND: &str =
    "create-new-owner-executable-temporary-file";
pub(crate) const LOADER_PROBE_TIMEOUT_MILLIS: u64 = 5_000;
const PROBE_TIMEOUT: Duration = Duration::from_millis(LOADER_PROBE_TIMEOUT_MILLIS);
const POLL_INTERVAL: Duration = Duration::from_millis(10);
const CAPTURE_LIMIT_BYTES: u64 = 1024 * 1024;
static PROBE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct LoaderProbeRuntimeRequest<'a> {
    pub(crate) bytes: &'a [u8],
    pub(crate) probe_root: &'a Path,
    pub(crate) path_namespace: &'static str,
}

#[derive(Debug)]
pub(crate) struct LoaderProbeRuntimeObservation {
    pub(crate) status: String,
    pub(crate) attempted: bool,
    pub(crate) materialized: bool,
    pub(crate) materialized_hash_matches: bool,
    pub(crate) kernel_accepted: bool,
    pub(crate) process_completed: bool,
    pub(crate) timed_out: bool,
    pub(crate) exit_code: Option<i32>,
    pub(crate) termination_signal: Option<i32>,
    pub(crate) stdout: LoaderProbeCapturedOutput,
    pub(crate) stderr: LoaderProbeCapturedOutput,
    pub(crate) failure_kind: Option<String>,
    pub(crate) cleanup_attempted: bool,
    pub(crate) cleanup_succeeded: bool,
    pub(crate) blockers: Vec<String>,
}

impl LoaderProbeRuntimeObservation {
    pub(crate) fn blocked(status: &str, blocker: &str) -> Self {
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
            stdout: LoaderProbeCapturedOutput::empty(),
            stderr: LoaderProbeCapturedOutput::empty(),
            failure_kind: None,
            cleanup_attempted: false,
            cleanup_succeeded: true,
            blockers: vec![blocker.to_owned()],
        }
    }
}

#[derive(Debug)]
pub(crate) struct LoaderProbeCapturedOutput {
    pub(crate) bytes: usize,
    pub(crate) truncated: bool,
    pub(crate) hash: String,
}

impl LoaderProbeCapturedOutput {
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

pub(crate) fn execute_isolated_loader_probe(
    request: LoaderProbeRuntimeRequest<'_>,
) -> Result<LoaderProbeRuntimeObservation, String> {
    validate_path_namespace(request.path_namespace)?;
    let probe_root = fs::canonicalize(request.probe_root)
        .map_err(|error| format!("failed to resolve loader probe root: {error}"))?;
    if !probe_root.is_dir() {
        return Err("loader probe root is not a directory".to_owned());
    }
    let paths = unique_probe_paths(&probe_root, request.path_namespace);
    let mut created = ProbeCreatedPaths::default();
    let result = execute_probe_inner(&request, &probe_root, &paths, &mut created);
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
        Err(error) if cleanup_succeeded => Err(error),
        Err(error) => Err(format!("{error}; loader probe cleanup failed")),
    }
}

fn execute_probe_inner(
    request: &LoaderProbeRuntimeRequest<'_>,
    probe_root: &Path,
    paths: &ProbePaths,
    created: &mut ProbeCreatedPaths,
) -> Result<LoaderProbeRuntimeObservation, String> {
    let mut executable = create_new_file(&paths.executable, "executable", 0o700)?;
    created.executable = true;
    executable
        .write_all(request.bytes)
        .map_err(|error| format!("failed to write loader probe image: {error}"))?;
    executable
        .sync_all()
        .map_err(|error| format!("failed to sync loader probe image: {error}"))?;
    set_owner_executable(&executable)?;
    drop(executable);
    let materialized_bytes = fs::read(&paths.executable)
        .map_err(|error| format!("failed to verify loader probe image: {error}"))?;
    if materialized_bytes != request.bytes {
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
        Err(error) => return rejected_observation(error, paths),
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
    let blockers =
        observation_blockers(successful, timed_out, capture_limit_exceeded, &wait_failure);
    Ok(LoaderProbeRuntimeObservation {
        status: observation_status(successful, timed_out, capture_limit_exceeded).to_owned(),
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

fn rejected_observation(
    error: std::io::Error,
    paths: &ProbePaths,
) -> Result<LoaderProbeRuntimeObservation, String> {
    Ok(LoaderProbeRuntimeObservation {
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
    })
}

fn failed_materialization_observation() -> LoaderProbeRuntimeObservation {
    let mut observation = LoaderProbeRuntimeObservation::blocked(
        "blocked-materialized-image-drift",
        "materialized-private-image-hash-mismatch",
    );
    observation.attempted = true;
    observation.materialized = true;
    observation
}

fn observation_blockers(
    successful: bool,
    timed_out: bool,
    capture_limit_exceeded: bool,
    wait_failure: &Option<String>,
) -> Vec<String> {
    if timed_out {
        vec!["loader-probe-timeout".to_owned()]
    } else if capture_limit_exceeded {
        vec!["loader-probe-output-limit-exceeded".to_owned()]
    } else if wait_failure.is_some() {
        vec!["loader-probe-wait-failed".to_owned()]
    } else if !successful {
        vec!["loader-probe-process-unsuccessful".to_owned()]
    } else {
        Vec::new()
    }
}

fn observation_status(
    successful: bool,
    timed_out: bool,
    capture_limit_exceeded: bool,
) -> &'static str {
    if successful {
        "os-loader-accepted-process-succeeded"
    } else if timed_out {
        "os-loader-accepted-process-timed-out"
    } else if capture_limit_exceeded {
        "os-loader-accepted-output-limit-exceeded"
    } else {
        "os-loader-accepted-process-unsuccessful"
    }
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
                return (child.wait().ok(), true, kill_failure);
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

fn read_capture(path: &Path) -> Result<LoaderProbeCapturedOutput, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("failed to open loader probe capture: {error}"))?;
    let total = file
        .metadata()
        .map_err(|error| format!("failed to inspect loader probe capture: {error}"))?
        .len();
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(CAPTURE_LIMIT_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read loader probe capture: {error}"))?;
    Ok(LoaderProbeCapturedOutput {
        bytes: bytes.len(),
        truncated: total > CAPTURE_LIMIT_BYTES,
        hash: crate::fnv1a64_hex(&bytes),
    })
}

fn unique_probe_paths(root: &Path, namespace: &str) -> ProbePaths {
    let sequence = PROBE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let base = format!(
        ".nsld-{namespace}-loader-probe-{}-{sequence}",
        std::process::id()
    );
    ProbePaths {
        executable: root.join(format!("{base}.bin")),
        stdout: root.join(format!("{base}.stdout")),
        stderr: root.join(format!("{base}.stderr")),
    }
}

fn validate_path_namespace(namespace: &str) -> Result<(), String> {
    if namespace.is_empty()
        || !namespace
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err("loader probe path namespace is invalid".to_owned());
    }
    Ok(())
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
        .map_err(|error| format!("failed to create loader probe {label}: {error}"))
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
        clean &= match fs::remove_file(path) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        };
    }
    clean
}

#[cfg(unix)]
fn set_owner_executable(file: &File) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = file
        .metadata()
        .map_err(|error| format!("failed to inspect loader probe image: {error}"))?
        .permissions();
    permissions.set_mode(0o700);
    file.set_permissions(permissions)
        .map_err(|error| format!("failed to mark loader probe image executable: {error}"))
}

#[cfg(not(unix))]
fn set_owner_executable(_file: &File) -> Result<(), String> {
    Err("loader probe executable permissions require Unix".to_owned())
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

#[cfg(test)]
#[path = "final_executable_loader_probe_runtime_tests.rs"]
mod tests;
