use std::{
    fs::{self, DirBuilder},
    io::ErrorKind,
    net::Shutdown,
    os::unix::{
        fs::DirBuilderExt,
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[path = "artifact_runtime_provider_transport.rs"]
pub(crate) mod transport;

const FAILURE_CLEANUP_GRACE: Duration = Duration::from_secs(5);

pub(super) fn run_command(
    output_dir: &Path,
    command: &mut Command,
) -> Result<(ExitStatus, usize), String> {
    run_command_with_timeout(output_dir, command, None)
}

pub(super) fn run_command_with_timeout(
    output_dir: &Path,
    command: &mut Command,
    timeout: Option<Duration>,
) -> Result<(ExitStatus, usize), String> {
    let server = RuntimeProviderServer::start(output_dir)?;
    run_supervised_command(server, command, timeout)
}

fn run_supervised_command(
    mut server: RuntimeProviderServer,
    command: &mut Command,
    timeout: Option<Duration>,
) -> Result<(ExitStatus, usize), String> {
    command
        .env_remove(yir_runtime_host::PROVIDER_RESULT_STREAM_ENV)
        .env(
            yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV,
            server.directory.join("dispatch"),
        );
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to launch runtime child: {error}"))?;
    let child_result = supervise_child(&mut child, timeout, FAILURE_CLEANUP_GRACE, || {
        server
            .thread
            .as_ref()
            .is_some_and(|worker| worker.is_finished())
    });
    // A child that cleaned up (or even exited zero) cannot erase provider failure.
    match (child_result, server.finish()) {
        (Ok(status), Ok(count)) => Ok((status, count)),
        (Err(child), Err(provider)) => Err(format!("{provider}; {child}")),
        (Err(error), _) | (_, Err(error)) => Err(error),
    }
}

fn supervise_child(
    child: &mut Child,
    timeout: Option<Duration>,
    failure_grace: Duration,
    mut provider_stopped: impl FnMut() -> bool,
) -> Result<ExitStatus, String> {
    let started = Instant::now();
    let mut failed_at = None;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("failed to wait for runtime child: {error}"));
            }
        }
        if timeout.is_some_and(|limit| started.elapsed() >= limit) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("runtime child exceeded its explicit wall-clock limit".to_owned());
        }
        if provider_stopped() && failed_at.is_none() {
            failed_at = Some(Instant::now());
            eprintln!("runtime provider stopped; allowing bounded application failure cleanup");
        }
        if failed_at.is_some_and(|start| start.elapsed() >= failure_grace) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("runtime child exceeded the provider failure cleanup deadline".to_owned());
        }
        thread::sleep(Duration::from_millis(10));
    }
}

struct RuntimeProviderServer {
    directory: PathBuf,
    stop: Arc<AtomicBool>,
    active: Arc<Mutex<Option<UnixStream>>>,
    thread: Option<JoinHandle<Result<usize, String>>>,
}

impl RuntimeProviderServer {
    fn start(output_dir: &Path) -> Result<Self, String> {
        Self::start_with_reader(output_dir, |stream| {
            transport::read_request(stream, transport::IO_TIMEOUT)
        })
    }

    fn start_with_reader(
        output_dir: &Path,
        read_request: fn(
            &mut UnixStream,
        ) -> Result<yir_core::provider_runtime_ipc::Message, String>,
    ) -> Result<Self, String> {
        let directory = private_socket_directory()?;
        let stop = Arc::new(AtomicBool::new(false));
        let active = Arc::new(Mutex::new(None));
        let mut server = Self {
            directory,
            stop,
            active,
            thread: None,
        };
        let listener = UnixListener::bind(server.directory.join("dispatch"))
            .map_err(|error| format!("failed to bind runtime IPC: {error}"))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let stop = Arc::clone(&server.stop);
        let active = Arc::clone(&server.active);
        let output_dir = output_dir.to_owned();
        server.thread = Some(thread::spawn(move || {
            let mut total = 0usize;
            let mut completed = 0usize;
            while !stop.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        transport::configure_connection(&stream, transport::IO_TIMEOUT)?;
                        *active
                            .lock()
                            .map_err(|_| "runtime IPC stream lock was poisoned")? =
                            Some(stream.try_clone().map_err(|error| error.to_string())?);
                        if stop.load(Ordering::Acquire) {
                            break;
                        }
                        let result = nsdb::serve_runtime_provider_session_with_request_reader(
                            &output_dir,
                            &mut stream,
                            read_request,
                        );
                        active
                            .lock()
                            .map_err(|_| "runtime IPC stream lock was poisoned")?
                            .take();
                        total = total
                            .checked_add(result?.into_finished_count()?)
                            .ok_or("runtime IPC invocation count overflow")?;
                        completed += 1;
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(error) => return Err(format!("runtime IPC accept failed: {error}")),
                }
            }
            if completed == 0 {
                return Err("runtime child did not complete a provider lifecycle".to_owned());
            }
            Ok(total)
        }));
        Ok(server)
    }

    fn finish(&mut self) -> Result<usize, String> {
        self.stop.store(true, Ordering::Release);
        if let Ok(active) = self.active.lock() {
            if let Some(stream) = active.as_ref() {
                let _ = stream.shutdown(Shutdown::Both);
            }
        }
        self.thread
            .take()
            .ok_or_else(|| "runtime IPC server already joined".to_owned())?
            .join()
            .map_err(|_| "runtime IPC server panicked".to_owned())?
    }
}

#[cfg(all(test, target_os = "macos"))]
pub(crate) fn run_command_with_request_reader(
    output_dir: &Path,
    command: &mut Command,
    timeout: Duration,
    read_request: fn(&mut UnixStream) -> Result<yir_core::provider_runtime_ipc::Message, String>,
) -> Result<(ExitStatus, usize), String> {
    run_supervised_command(
        RuntimeProviderServer::start_with_reader(output_dir, read_request)?,
        command,
        Some(timeout),
    )
}

impl Drop for RuntimeProviderServer {
    fn drop(&mut self) {
        if self.thread.is_some() {
            let _ = self.finish();
        }
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn private_socket_directory() -> Result<PathBuf, String> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    for _ in 0..128 {
        let path = std::env::temp_dir().join(format!(
            "nsipc-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "failed to create private runtime IPC directory: {error}"
                ))
            }
        }
    }
    Err("runtime IPC directory allocation exhausted".to_owned())
}

#[cfg(test)]
#[path = "artifact_runtime_provider_ipc_tests.rs"]
mod tests;
