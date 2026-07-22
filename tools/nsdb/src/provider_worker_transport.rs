use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

pub(crate) const PROVIDER_WORKER_TRANSPORT_CONTRACT: &str = "nuis-provider-worker-transport-v1";
pub(crate) const PROVIDER_WORKER_TRANSPORT_REGISTRY_CONTRACT: &str =
    "nuis-provider-worker-transport-registry-v1";
pub(crate) const PROVIDER_WORKER_TRANSPORT_REGISTRY_SOURCE: &str =
    "builtin-provider-worker-transport-registry";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderWorkerTransportAdapter {
    pub(crate) registry_contract: &'static str,
    pub(crate) registry_source: &'static str,
    pub(crate) transport_contract: &'static str,
    pub(crate) adapter_id: &'static str,
    pub(crate) mode: &'static str,
    pub(crate) capability_status: &'static str,
    pub(crate) descriptor_transfer_status: &'static str,
}

const FRAMED_STDIO: ProviderWorkerTransportAdapter = ProviderWorkerTransportAdapter {
    registry_contract: PROVIDER_WORKER_TRANSPORT_REGISTRY_CONTRACT,
    registry_source: PROVIDER_WORKER_TRANSPORT_REGISTRY_SOURCE,
    transport_contract: PROVIDER_WORKER_TRANSPORT_CONTRACT,
    adapter_id: "framed.stdio.worker.v1",
    mode: "persistent-child-stdio",
    capability_status: "registered-protocol-ready",
    descriptor_transfer_status: "unsupported",
};

#[cfg(unix)]
const UNIX_RIGHTS: ProviderWorkerTransportAdapter = ProviderWorkerTransportAdapter {
    registry_contract: PROVIDER_WORKER_TRANSPORT_REGISTRY_CONTRACT,
    registry_source: PROVIDER_WORKER_TRANSPORT_REGISTRY_SOURCE,
    transport_contract: PROVIDER_WORKER_TRANSPORT_CONTRACT,
    adapter_id: "unix.scm-rights.worker.v1",
    mode: "persistent-unix-socket",
    capability_status: "registered-protocol-ready",
    descriptor_transfer_status: "supported",
};

pub(crate) fn select_provider_worker_transport_adapter(
    requested_mode: &str,
) -> Option<ProviderWorkerTransportAdapter> {
    #[cfg(unix)]
    if matches!(requested_mode, "auto" | "persistent-unix-socket") {
        return Some(UNIX_RIGHTS);
    }
    (requested_mode == "auto" || requested_mode == "persistent-child-stdio").then_some(FRAMED_STDIO)
}

pub(crate) struct ProviderWorkerTransport {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    lease_id: String,
    provider_family: String,
    next_sequence: usize,
    closed: bool,
}

pub(crate) struct ProviderWorkerReply {
    pub(crate) sequence: usize,
    pub(crate) request_id: String,
    pub(crate) worker_pid: u32,
}

impl ProviderWorkerTransport {
    pub(crate) fn spawn(
        command: &mut Command,
        lease_id: &str,
        provider_family: &str,
    ) -> Result<Self, String> {
        validate_token(lease_id, "lease id")?;
        validate_token(provider_family, "provider family")?;
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("failed to spawn provider worker: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "provider worker stdin is unavailable".to_owned())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "provider worker stdout is unavailable".to_owned())?;
        Ok(Self {
            child,
            stdin: Some(stdin),
            stdout: BufReader::new(stdout),
            lease_id: lease_id.to_owned(),
            provider_family: provider_family.to_owned(),
            next_sequence: 0,
            closed: false,
        })
    }

    pub(crate) fn request(&mut self, request_id: &str) -> Result<ProviderWorkerReply, String> {
        validate_token(request_id, "request id")?;
        if self.closed {
            return Err("provider worker transport is closed".to_owned());
        }
        let sequence = self.next_sequence;
        self.write_frame(&format!(
            "NUISPW1\trequest\t{sequence}\t{}\t{}\t{request_id}",
            self.lease_id, self.provider_family
        ))?;
        let fields = self.read_frame()?;
        if fields.len() != 7
            || fields[0] != "NUISPW1"
            || fields[1] != "request"
            || fields[2] != sequence.to_string()
            || fields[3] != self.lease_id
            || fields[4] != self.provider_family
            || fields[5] != request_id
        {
            return Err("provider worker returned a mismatched request receipt".to_owned());
        }
        let worker_pid = fields[6]
            .parse::<u32>()
            .map_err(|error| format!("provider worker pid is invalid: {error}"))?;
        self.next_sequence += 1;
        Ok(ProviderWorkerReply {
            sequence,
            request_id: request_id.to_owned(),
            worker_pid,
        })
    }

    pub(crate) fn close(mut self) -> Result<u32, String> {
        self.write_frame(&format!("NUISPW1\tclose\t{}", self.lease_id))?;
        let fields = self.read_frame()?;
        if fields.len() != 4
            || fields[0] != "NUISPW1"
            || fields[1] != "close"
            || fields[2] != self.lease_id
        {
            return Err("provider worker returned a mismatched close receipt".to_owned());
        }
        let worker_pid = fields[3]
            .parse::<u32>()
            .map_err(|error| format!("provider worker close pid is invalid: {error}"))?;
        self.stdin.take();
        let status = self
            .child
            .wait()
            .map_err(|error| format!("failed to wait for provider worker: {error}"))?;
        if !status.success() {
            return Err(format!("provider worker exited with status {status}"));
        }
        self.closed = true;
        Ok(worker_pid)
    }

    fn write_frame(&mut self, frame: &str) -> Result<(), String> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| "provider worker stdin is closed".to_owned())?;
        writeln!(stdin, "{frame}")
            .and_then(|_| stdin.flush())
            .map_err(|error| format!("failed to write provider worker frame: {error}"))
    }

    fn read_frame(&mut self) -> Result<Vec<String>, String> {
        let mut line = String::new();
        let count = self
            .stdout
            .read_line(&mut line)
            .map_err(|error| format!("failed to read provider worker frame: {error}"))?;
        if count == 0 {
            return Err("provider worker closed before returning a receipt".to_owned());
        }
        Ok(line
            .trim_end_matches(['\r', '\n'])
            .split('\t')
            .map(str::to_owned)
            .collect())
    }
}

impl Drop for ProviderWorkerTransport {
    fn drop(&mut self) {
        if !self.closed {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn validate_token(value: &str, name: &str) -> Result<(), String> {
    if value.is_empty() || value.contains(['\t', '\r', '\n']) {
        return Err(format!("provider worker {name} is not frame-safe"));
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn two_requests_share_one_persistent_worker_process() {
        let adapter =
            select_provider_worker_transport_adapter("persistent-child-stdio").expect("adapter");
        assert_eq!(
            adapter.transport_contract,
            PROVIDER_WORKER_TRANSPORT_CONTRACT
        );
        assert_eq!(adapter.descriptor_transfer_status, "unsupported");
        let script = r#"while IFS= read -r line; do
case "$line" in
  NUISPW1$'\t'close$'\t'*) printf '%s\t%s\n' "$line" "$$"; break ;;
  *) printf '%s\t%s\n' "$line" "$$" ;;
esac
done"#;
        let mut command = Command::new("bash");
        command.args(["-c", script]);
        let mut worker = ProviderWorkerTransport::spawn(
            &mut command,
            "provider-session:test",
            "coreml:apple-ane",
        )
        .expect("worker");
        let first = worker.request("affine").expect("first");
        let second = worker.request("add").expect("second");
        assert_eq!((first.sequence, second.sequence), (0, 1));
        assert_eq!(
            (first.request_id.as_str(), second.request_id.as_str()),
            ("affine", "add")
        );
        assert_eq!(first.worker_pid, second.worker_pid);
        assert_eq!(worker.close().expect("close"), first.worker_pid);
    }
}
