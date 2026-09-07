use crate::{
    final_image_provider_dispatch::{
        final_image_provider_dispatch_authority, validate_provider_families_against_final_image,
    },
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_request::provider_request_collection_from_evidence,
    provider_runner_registry::{select_provider_runner_adapter, ProviderRunnerAdapter},
    provider_runtime_dispatch_session::{
        runtime_dispatch_observations, NativeProviderOutputs, ProviderRuntimeDispatchSession,
        ProviderRuntimeDispatchSessionEvidence,
    },
    provider_runtime_result_stream::{
        persist_provider_runtime_results, provider_runtime_result_targets,
    },
    provider_sample::read_device_provider_sample_manifest_info,
    provider_sample_payload::{
        provider_output_payload_file_name, render_real_device_provider_output_payload,
    },
};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use yir_core::provider_runtime_ipc::{
    DispatchArguments, DispatchFrame, DispatchTarget, Message, Rejection, RejectionCode,
    RejectionPhase, ReplayBudget, MAX_DISPATCHES,
};

/// Serve one bounded lifecycle. Device work starts only after a validated Dispatch message.
pub fn serve_runtime_provider_session(
    output_dir: &Path,
    stream: &mut (impl Read + Write),
) -> Result<usize, String> {
    serve_runtime_provider_session_with_request_reader(output_dir, stream, Message::read_from)
}

/// Supply transport-specific request waiting without changing dispatch admission,
/// session accounting or close handling. A reader error terminates the lifecycle;
/// the reader must not retry a partially consumed message or reconnect the peer.
pub fn serve_runtime_provider_session_with_request_reader<S: Read + Write>(
    output_dir: &Path,
    stream: &mut S,
    read_request: impl FnMut(&mut S) -> Result<Message, String>,
) -> Result<usize, String> {
    let (record, target, adapter) = admit_target(output_dir)?;
    let payload_bytes = admitted_payload_bytes(&record, &target)?;
    Message::Hello(target.clone()).write_to(stream)?;
    let mut session = ProviderRuntimeDispatchSession::open(output_dir);
    let execution = dispatch_loop(stream, &target, payload_bytes, read_request, |arguments| {
        session.execute_graph(output_dir, &record, &adapter, Some((&target, arguments)))
    });
    let close = session.close();
    let result = complete_session(execution, close).and_then(|(count, outputs)| {
        finalize_session(stream, count, || {
            if count > 0 {
                persist_outputs(output_dir, &record, &adapter, &outputs)?;
            }
            Ok(())
        })
    });
    if let Err(error) = &result {
        let _ = Message::Rejected(error.clone()).write_to(stream);
    }
    result.map_err(|error| error.to_string())
}

fn complete_session(
    execution: Result<(usize, NativeProviderOutputs), Rejection>,
    close: Result<(), String>,
) -> Result<(usize, NativeProviderOutputs), Rejection> {
    match execution {
        Err(mut error) => {
            if let Err(close) = close {
                error
                    .detail
                    .push_str(&format!("; provider cleanup failed: {close}"));
            }
            Err(error)
        }
        Ok((count, outputs)) => {
            close.map_err(|error| {
                Rejection::new(
                    RejectionPhase::Finish,
                    count,
                    RejectionCode::Finalization,
                    error,
                )
            })?;
            Ok((count, outputs))
        }
    }
}

fn finalize_session(
    stream: &mut impl Write,
    count: usize,
    persist: impl FnOnce() -> Result<(), String>,
) -> Result<usize, Rejection> {
    persist().map_err(|error| {
        Rejection::new(
            RejectionPhase::Finish,
            count,
            RejectionCode::Finalization,
            error,
        )
    })?;
    Message::Closed(count).write_to(stream).map_err(|error| {
        Rejection::new(
            RejectionPhase::Finish,
            count,
            RejectionCode::Exchange,
            error,
        )
    })?;
    Ok(count)
}

fn admit_target(
    output_dir: &Path,
) -> Result<
    (
        NsdbDeviceProviderSampleRecordInfo,
        DispatchTarget,
        ProviderRunnerAdapter,
    ),
    String,
> {
    let targets = provider_runtime_result_targets(output_dir, None)?;
    let [target] = targets.as_slice() else {
        return Err("runtime IPC currently requires exactly one target".to_owned());
    };
    let manifest = read_device_provider_sample_manifest_info(output_dir);
    let mut matching = manifest.records.into_iter().filter(|record| {
        record.provider_family == target.provider_family
            && provider_request_collection_from_evidence(&record.input_evidence).is_some_and(
                |collection| {
                    collection.requests.iter().any(|request| {
                        request
                            .runtime_result_binding
                            .as_ref()
                            .is_some_and(|binding| {
                                binding.source_yir_fnv1a64 == target.source_yir_fnv1a64
                                    && binding.module == target.module
                                    && binding.instruction == target.instruction
                                    && binding.node == target.node
                                    && binding.resource == target.resource
                            })
                    })
                },
            )
    });
    let record = matching
        .next()
        .ok_or("runtime IPC target has no admitted record")?;
    if matching.next().is_some() {
        return Err("runtime IPC target is ambiguous".to_owned());
    }
    let authority = final_image_provider_dispatch_authority(output_dir);
    if !authority.blockers.is_empty() {
        return Err(authority.blockers.join(", "));
    }
    let families = crate::provider_bundle_registry::provider_families_for_records(
        std::slice::from_ref(&record),
    )?;
    validate_provider_families_against_final_image(&authority, &families)?;
    let adapter = select_provider_runner_adapter(&record.provider_family);
    if (authority.available && record.provider_runner_adapter_id != adapter.adapter_id)
        || !adapter.real_device_capable
    {
        return Err("runtime IPC registered adapter identity or capability drift".to_owned());
    }
    Ok((
        record,
        DispatchTarget {
            source_yir_fnv1a64: target.source_yir_fnv1a64.clone(),
            module: target.module.clone(),
            instruction: target.instruction.clone(),
            node: target.node.clone(),
            resource: target.resource.clone(),
        },
        adapter,
    ))
}

fn admitted_payload_bytes(
    record: &NsdbDeviceProviderSampleRecordInfo,
    target: &DispatchTarget,
) -> Result<usize, String> {
    let collection = provider_request_collection_from_evidence(&record.input_evidence)
        .ok_or("runtime IPC request collection is invalid")?;
    let mut payload_bytes = None;
    for request in collection.requests {
        if request
            .runtime_result_binding
            .as_ref()
            .is_some_and(|binding| {
                binding.source_yir_fnv1a64 == target.source_yir_fnv1a64
                    && binding.module == target.module
                    && binding.instruction == target.instruction
                    && binding.node == target.node
                    && binding.resource == target.resource
            })
        {
            // from_execution binds the first declared output, not an extent
            // chosen by application arguments or a returned device payload.
            for output in &request.output_bindings {
                ReplayBudget::default().reserve(output.byte_length)?;
            }
            let bytes = request
                .output_bindings
                .first()
                .ok_or("runtime IPC registered result has no output")?
                .byte_length;
            if payload_bytes.replace(bytes).is_some() {
                return Err("runtime IPC registered result extent is ambiguous".to_owned());
            }
        }
    }
    payload_bytes.ok_or_else(|| "runtime IPC registered result extent is missing".to_owned())
}

fn dispatch_loop<S: Read + Write>(
    stream: &mut S,
    target: &DispatchTarget,
    payload_bytes: usize,
    mut read_request: impl FnMut(&mut S) -> Result<Message, String>,
    mut execute: impl FnMut(&DispatchArguments) -> Result<NativeProviderOutputs, String>,
) -> Result<(usize, NativeProviderOutputs), Rejection> {
    let mut count = 0;
    let mut retained = NativeProviderOutputs::empty();
    let mut observations = Vec::new();
    let mut replay_budget = ReplayBudget::default();
    loop {
        let request = read_request(stream).map_err(|error| {
            Rejection::new(
                RejectionPhase::Receive,
                count,
                RejectionCode::Exchange,
                error,
            )
        })?;
        let dispatch_error =
            |code, error| Rejection::new(RejectionPhase::Dispatch, count, code, error);
        match request {
            Message::Dispatch {
                sequence,
                target: requested,
                arguments,
            } => {
                if requested != *target || sequence != count || count >= MAX_DISPATCHES {
                    return Err(Rejection::new(
                        RejectionPhase::Receive,
                        count,
                        RejectionCode::Request,
                        "runtime IPC request target or sequence mismatch".to_owned(),
                    ));
                }
                let mut outputs =
                    execute_reserved(&mut replay_budget, payload_bytes, count, || {
                        execute(&arguments)
                    })?;
                let [result] = outputs.runtime_results.as_slice() else {
                    return Err(dispatch_error(
                        RejectionCode::Result,
                        "runtime IPC graph must return exactly one bound result".to_owned(),
                    ));
                };
                if result.source_yir_fnv1a64 != target.source_yir_fnv1a64
                    || result.module != target.module
                    || result.instruction != target.instruction
                    || result.node != target.node
                    || result.resource != target.resource
                    || !result
                        .arguments
                        .matches_identity(&arguments)
                        .map_err(|error| dispatch_error(RejectionCode::Result, error))?
                {
                    return Err(dispatch_error(
                        RejectionCode::Result,
                        "runtime IPC result target drift".to_owned(),
                    ));
                }
                observations.extend(
                    runtime_dispatch_observations(count, &outputs.native_outputs)
                        .map_err(|error| dispatch_error(RejectionCode::Result, error))?,
                );
                let reply = Message::Frame(DispatchFrame {
                    sequence,
                    arguments: result.arguments.clone(),
                    request_id: result.request_id.clone(),
                    provider_family: result.provider_family.clone(),
                    element_type: result.element_type.clone(),
                    layout: result.layout.clone(),
                    shape: result.shape.clone(),
                    row_stride_bytes: result.row_stride_bytes,
                    payload: result.payload.clone(),
                    completion_wire: result.completion_wire.clone(),
                });
                if count == 0 {
                    retained = outputs;
                } else {
                    retained
                        .runtime_results
                        .append(&mut outputs.runtime_results);
                }
                reply
                    .write_to(stream)
                    .map_err(|error| dispatch_error(RejectionCode::Exchange, error))?;
                count += 1;
            }
            Message::Finish(sequence) if sequence == count => {
                retained.runtime_session_evidence =
                    ProviderRuntimeDispatchSessionEvidence::from_observations(count, &observations)
                        .map_err(|error| {
                            Rejection::new(
                                RejectionPhase::Finish,
                                count,
                                RejectionCode::Result,
                                error,
                            )
                        })?;
                return Ok((count, retained));
            }
            _ => {
                return Err(Rejection::new(
                    RejectionPhase::Receive,
                    count,
                    RejectionCode::Request,
                    "runtime IPC expected ordered dispatch or matching finish",
                ))
            }
        }
    }
}

fn execute_reserved(
    budget: &mut ReplayBudget,
    payload_bytes: usize,
    sequence: usize,
    execute: impl FnOnce() -> Result<NativeProviderOutputs, String>,
) -> Result<NativeProviderOutputs, Rejection> {
    let failure = |code, error| Rejection::new(RejectionPhase::Dispatch, sequence, code, error);
    budget
        .reserve(payload_bytes)
        .map_err(|error| failure(RejectionCode::Budget, error))?;
    let outputs = execute().map_err(|error| failure(RejectionCode::Execution, error))?;
    let [result] = outputs.runtime_results.as_slice() else {
        return Err(failure(
            RejectionCode::Result,
            "runtime IPC graph must return exactly one bound result".to_owned(),
        ));
    };
    if result.payload.len() != payload_bytes {
        return Err(failure(
            RejectionCode::Result,
            "runtime IPC result differs from its reserved output extent".to_owned(),
        ));
    }
    Ok(outputs)
}

fn persist_outputs(
    output_dir: &Path,
    record: &NsdbDeviceProviderSampleRecordInfo,
    adapter: &ProviderRunnerAdapter,
    outputs: &NativeProviderOutputs,
) -> Result<(), String> {
    let projections = crate::provider_result_projection::validate_and_render_result_projections(
        &record.input_evidence,
        &outputs.native_outputs,
    )?;
    let mut content =
        render_real_device_provider_output_payload(record, adapter, outputs, &projections);
    crate::provider_sample_payload::push_toml_string(
        &mut content,
        "runtime_dispatch_trigger",
        "child-yir-node-ipc",
    );
    crate::provider_sample_payload::push_toml_string(
        &mut content,
        "runtime_dispatch_ipc_contract",
        yir_core::provider_runtime_ipc::CONTRACT,
    );
    persist_provider_runtime_results(output_dir, &outputs.runtime_results)?;
    fs::write(
        output_dir.join(provider_output_payload_file_name(&record.provider_family)),
        content,
    )
    .map_err(|error| format!("failed to persist live runtime output: {error}"))
}

#[cfg(test)]
#[path = "provider_runtime_ipc_tests.rs"]
mod tests;
