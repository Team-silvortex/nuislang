use super::provider_result_stream::{
    execute_with_provider_source, ProviderResultFrame, ProviderResultSource,
};
use crate::application_failure::ProviderFailure;
use std::{os::unix::net::UnixStream, path::Path, time::Duration};
use yir_core::{
    provider_runtime_ipc::{
        hash_bytes, DispatchArguments, DispatchTarget, Message, Rejection, RejectionCode,
        RejectionPhase, SessionDrain, MAX_DISPATCHES,
    },
    ApplicationFailureKind, Node, YirModule,
};

pub fn execute_module_source_with_provider_ipc(
    module_source: &str,
    socket_path: impl AsRef<Path>,
) -> Result<yir_exec::ExecutionTrace, String> {
    let module = yir_syntax::parse_module(module_source)?;
    let client = connect_provider_runtime(module_source, &module, socket_path.as_ref())?;
    execute_with_provider_source(module_source, ProviderResultSource::Live(client))
}

pub(super) fn connect_provider_runtime(
    module_source: &str,
    module: &YirModule,
    socket_path: &Path,
) -> Result<ProviderRuntimeClient, String> {
    let mut stream = UnixStream::connect(socket_path)
        .map_err(|error| format!("runtime IPC connection failed: {error}"))?;
    let timeout = Some(Duration::from_secs(120));
    stream
        .set_read_timeout(timeout)
        .and_then(|_| stream.set_write_timeout(timeout))
        .map_err(|error| format!("runtime IPC timeout setup failed: {error}"))?;
    let Message::Hello(target) = Message::read_from(&mut stream)? else {
        return Err("runtime IPC expected target admission".to_owned());
    };
    if target.source_yir_fnv1a64 != hash_bytes(module_source.as_bytes()) {
        return Err("runtime IPC target belongs to a different YIR module".to_owned());
    }
    if !module.nodes.iter().any(|node| target.matches(node)) {
        return Err("runtime IPC target is absent from the admitted YIR".to_owned());
    }
    if target.module != "shader" || target.instruction != "draw_instanced" {
        return Err("no registered runtime result adapter accepts this IPC target".to_owned());
    }
    Ok(ProviderRuntimeClient {
        stream,
        target,
        sequence: 0,
        frontier: true,
    })
}

pub(super) struct ProviderRuntimeClient {
    stream: UnixStream,
    target: DispatchTarget,
    sequence: usize,
    // False from the first attempted write until the whole reply is validated.
    // Failed or terminal exchanges never regain authority for another request.
    frontier: bool,
}

impl ProviderRuntimeClient {
    pub(super) fn targets(&self, node: &Node) -> bool {
        self.target.matches(node)
    }

    pub(super) fn take(
        &mut self,
        node: &Node,
        arguments: &DispatchArguments,
    ) -> Result<ProviderResultFrame, ProviderFailure> {
        if !self.targets(node) {
            return Err("runtime IPC target rejected".to_owned().into());
        }
        if self.sequence >= MAX_DISPATCHES {
            return Err(ProviderFailure::new(
                ApplicationFailureKind::DispatchLimit,
                "runtime IPC invocation limit rejected",
            ));
        }
        self.begin_exchange()?;
        Message::Dispatch {
            sequence: self.sequence,
            target: self.target.clone(),
            arguments: arguments.clone(),
        }
        .write_to(&mut self.stream)
        .map_err(ProviderFailure::exchange)?;
        let frame = match Message::read_from(&mut self.stream).map_err(ProviderFailure::exchange)? {
            Message::Frame(frame) if frame.sequence == self.sequence => frame,
            Message::Rejected(error) => return Err(self.rejected(error, RejectionPhase::Dispatch)),
            _ => {
                return Err("runtime IPC reply sequence or message mismatch"
                    .to_owned()
                    .into())
            }
        };
        if !frame.arguments.matches_identity(arguments)? {
            return Err("runtime IPC reply dispatch arguments mismatch"
                .to_owned()
                .into());
        }
        if frame.element_type != "u8"
            || frame.layout != "image-2d-row-major:pixel-format=rgba8"
            || frame.shape.len() != 2
            || frame.row_stride_bytes != frame.shape[0].saturating_mul(4)
            || frame.row_stride_bytes.checked_mul(frame.shape[1]) != Some(frame.payload.len())
        {
            return Err("runtime IPC frame layout or byte length mismatch"
                .to_owned()
                .into());
        }
        let result = ProviderResultFrame::from_ipc(&self.target, frame)?;
        self.sequence += 1;
        self.frontier = true;
        Ok(result)
    }

    pub(super) fn finish(&mut self) -> Result<(), ProviderFailure> {
        self.begin_exchange()?;
        Message::Finish(self.sequence)
            .write_to(&mut self.stream)
            .map_err(ProviderFailure::exchange)?;
        match Message::read_from(&mut self.stream).map_err(ProviderFailure::exchange)? {
            Message::Closed(count) if count == self.sequence => Ok(()),
            Message::Rejected(error) => Err(self.rejected(error, RejectionPhase::Finish)),
            _ => Err("runtime IPC close acknowledgement mismatch"
                .to_owned()
                .into()),
        }
    }

    /// None means the connection is not at a validated dispatch frontier. Never
    /// retry, reconnect, send Finish or reset timeouts to manufacture a receipt.
    pub(super) fn drain(&mut self) -> Result<Option<usize>, ProviderFailure> {
        if !self.frontier {
            return Ok(None);
        }
        self.begin_exchange()?;
        Message::Drain(SessionDrain {
            sequence: self.sequence,
            target: self.target.clone(),
        })
        .write_to(&mut self.stream)
        .map_err(ProviderFailure::exchange)?;
        match Message::read_from(&mut self.stream).map_err(ProviderFailure::exchange)? {
            Message::Drained(receipt) => {
                receipt.admit(&self.target, self.sequence)?;
                Ok(Some(self.sequence))
            }
            Message::Rejected(error) => Err(self.rejected(error, RejectionPhase::Drain)),
            _ => Err("runtime IPC drain acknowledgement mismatch"
                .to_owned()
                .into()),
        }
    }

    fn begin_exchange(&mut self) -> Result<(), ProviderFailure> {
        if !std::mem::replace(&mut self.frontier, false) {
            return Err("runtime IPC has no validated dispatch frontier"
                .to_owned()
                .into());
        }
        Ok(())
    }

    fn rejected(&self, error: Rejection, phase: RejectionPhase) -> ProviderFailure {
        if let Err(detail) = error.admit(phase, self.sequence) {
            return ProviderFailure::new(ApplicationFailureKind::ProviderContract, detail);
        }
        let kind = match error.code {
            RejectionCode::Request => ApplicationFailureKind::ProviderRejected,
            RejectionCode::Budget => ApplicationFailureKind::ProviderBudget,
            RejectionCode::Execution => ApplicationFailureKind::ProviderExecution,
            RejectionCode::Result => ApplicationFailureKind::ProviderContract,
            RejectionCode::Finalization => ApplicationFailureKind::ProviderFinalization,
            RejectionCode::Exchange => ApplicationFailureKind::ProviderExchange,
        };
        ProviderFailure::new(
            kind,
            format!("runtime provider rejected {phase:?}: {error}"),
        )
    }
}

#[cfg(test)]
#[path = "provider_runtime_ipc_tests.rs"]
mod tests;
