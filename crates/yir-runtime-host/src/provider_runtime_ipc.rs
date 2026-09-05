use super::provider_result_stream::{
    execute_with_provider_source, ProviderResultFrame, ProviderResultSource,
};
use std::{os::unix::net::UnixStream, path::Path, time::Duration};
use yir_core::{
    provider_runtime_ipc::{
        hash_bytes, DispatchArguments, DispatchTarget, Message, MAX_DISPATCHES,
    },
    Node,
};

pub fn execute_module_source_with_provider_ipc(
    module_source: &str,
    socket_path: impl AsRef<Path>,
) -> Result<yir_exec::ExecutionTrace, String> {
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
    let module = yir_syntax::parse_module(module_source)?;
    if !module.nodes.iter().any(|node| target.matches(node)) {
        return Err("runtime IPC target is absent from the admitted YIR".to_owned());
    }
    if target.module != "shader" || target.instruction != "draw_instanced" {
        return Err("no registered runtime result adapter accepts this IPC target".to_owned());
    }
    execute_with_provider_source(
        module_source,
        ProviderResultSource::Live(ProviderRuntimeClient {
            stream,
            target,
            sequence: 0,
        }),
    )
}

pub(super) struct ProviderRuntimeClient {
    stream: UnixStream,
    target: DispatchTarget,
    sequence: usize,
}

impl ProviderRuntimeClient {
    pub(super) fn targets(&self, node: &Node) -> bool {
        self.target.matches(node)
    }

    pub(super) fn take(
        &mut self,
        node: &Node,
        arguments: &DispatchArguments,
    ) -> Result<ProviderResultFrame, String> {
        if !self.targets(node) || self.sequence >= MAX_DISPATCHES {
            return Err("runtime IPC target or invocation limit rejected".to_owned());
        }
        Message::Dispatch {
            sequence: self.sequence,
            target: self.target.clone(),
            arguments: arguments.clone(),
        }
        .write_to(&mut self.stream)?;
        let frame = match Message::read_from(&mut self.stream)? {
            Message::Frame(frame) if frame.sequence == self.sequence => frame,
            Message::Rejected(error) => {
                return Err(format!("runtime provider rejected dispatch: {error}"))
            }
            _ => return Err("runtime IPC reply sequence or message mismatch".to_owned()),
        };
        if !frame.arguments.matches_identity(arguments)? {
            return Err("runtime IPC reply dispatch arguments mismatch".to_owned());
        }
        if frame.element_type != "u8"
            || frame.layout != "image-2d-row-major:pixel-format=rgba8"
            || frame.shape.len() != 2
            || frame.row_stride_bytes != frame.shape[0].saturating_mul(4)
            || frame.row_stride_bytes.checked_mul(frame.shape[1]) != Some(frame.payload.len())
        {
            return Err("runtime IPC frame layout or byte length mismatch".to_owned());
        }
        let result = ProviderResultFrame::from_ipc(&self.target, frame)?;
        self.sequence += 1;
        Ok(result)
    }

    pub(super) fn finish(&mut self) -> Result<(), String> {
        Message::Finish(self.sequence).write_to(&mut self.stream)?;
        match Message::read_from(&mut self.stream)? {
            Message::Closed(count) if count == self.sequence => Ok(()),
            Message::Rejected(error) => Err(format!("runtime provider rejected close: {error}")),
            _ => Err("runtime IPC close acknowledgement mismatch".to_owned()),
        }
    }
}

#[cfg(test)]
#[path = "provider_runtime_ipc_tests.rs"]
mod tests;
