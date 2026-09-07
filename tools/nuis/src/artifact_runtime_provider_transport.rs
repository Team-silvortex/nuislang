use std::{
    io::{self, Read},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};
use yir_core::provider_runtime_ipc::Message;

pub(super) const IO_TIMEOUT: Duration = Duration::from_secs(120);

pub(crate) fn configure_connection(stream: &UnixStream, timeout: Duration) -> Result<(), String> {
    validate_timeout(timeout)?;
    // BSD may inherit the listener's nonblocking flag on accept.
    stream
        .set_nonblocking(false)
        .and_then(|_| stream.set_read_timeout(Some(timeout)))
        .and_then(|_| stream.set_write_timeout(Some(timeout)))
        .map_err(|error| format!("runtime IPC connection setup failed: {error}"))
}

pub(crate) fn read_request(stream: &mut UnixStream, timeout: Duration) -> Result<Message, String> {
    validate_timeout(timeout)?;
    stream
        .set_read_timeout(None)
        .map_err(|error| format!("runtime IPC idle setup failed: {error}"))?;
    let mut first = [0];
    // Only the wait for a new message is unbounded. Socket shutdown/EOF still
    // wakes this read, including when the supervising child exits or is killed.
    stream
        .read_exact(&mut first)
        .map_err(|error| format!("runtime IPC idle read failed: {error}"))?;
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or("runtime IPC request deadline overflow")?;
    let reader = RequestReader { stream, deadline };
    Message::read_from(&mut first.as_slice().chain(reader))
}

fn validate_timeout(timeout: Duration) -> Result<(), String> {
    if timeout.is_zero() || Instant::now().checked_add(timeout).is_none() {
        return Err("runtime IPC request timeout must be positive and bounded".to_owned());
    }
    Ok(())
}

struct RequestReader<'a> {
    stream: &'a mut UnixStream,
    deadline: Instant,
}

impl Read for RequestReader<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(request_timeout)?;
        // A trickling prefix/header/upload shares one deadline, not a fresh
        // socket timeout for each successful partial read.
        self.stream.set_read_timeout(Some(remaining))?;
        let result = self.stream.read(bytes);
        if Instant::now() >= self.deadline
            || result.as_ref().is_err_and(|error| {
                matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                )
            })
        {
            return Err(request_timeout());
        }
        result
    }
}

fn request_timeout() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "runtime IPC request deadline exceeded",
    )
}

#[cfg(test)]
#[path = "artifact_runtime_provider_transport_tests.rs"]
mod tests;
