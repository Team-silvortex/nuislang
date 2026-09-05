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
    DispatchFrame, DispatchTarget, Message, MAX_DISPATCHES, MAX_PAYLOAD_BYTES,
};

/// Serve one bounded lifecycle. Device work starts only after a validated Dispatch message.
pub fn serve_runtime_provider_session(
    output_dir: &Path,
    stream: &mut (impl Read + Write),
) -> Result<usize, String> {
    let (record, target, adapter) = admit_target(output_dir)?;
    Message::Hello(target.clone()).write_to(stream)?;
    let mut session = ProviderRuntimeDispatchSession::open(output_dir);
    let execution = dispatch_loop(stream, &target, || {
        session.execute_graph(output_dir, &record, &adapter)
    });
    let close = session.close();
    let result = execution.and_then(|(count, outputs)| {
        close?;
        if count > 0 {
            persist_outputs(output_dir, &record, &adapter, &outputs)?;
        }
        Message::Closed(count).write_to(stream)?;
        Ok(count)
    });
    if let Err(error) = &result {
        let detail = error
            .chars()
            .filter(|ch| !matches!(ch, '\n' | '\r' | '\0'))
            .take(60)
            .collect::<String>();
        let _ = Message::Rejected(detail).write_to(stream);
    }
    result
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
    // Bound replay storage before any device request is admitted.
    let collection = provider_request_collection_from_evidence(&record.input_evidence)
        .ok_or("runtime IPC request collection is invalid")?;
    for request in &collection.requests {
        if request.runtime_result_binding.is_some()
            && request
                .output_bindings
                .iter()
                .any(|binding| binding.byte_length == 0 || binding.byte_length > MAX_PAYLOAD_BYTES)
        {
            return Err("runtime IPC bounded replay payload budget exceeded".to_owned());
        }
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

fn dispatch_loop(
    stream: &mut (impl Read + Write),
    target: &DispatchTarget,
    mut execute: impl FnMut() -> Result<NativeProviderOutputs, String>,
) -> Result<(usize, NativeProviderOutputs), String> {
    let mut count = 0;
    let mut retained = NativeProviderOutputs::empty();
    let mut observations = Vec::new();
    let mut replay_bytes = 0usize;
    loop {
        match Message::read_from(stream)? {
            Message::Dispatch {
                sequence,
                target: requested,
            } => {
                if requested != *target || sequence != count || count >= MAX_DISPATCHES {
                    return Err("runtime IPC request target or sequence mismatch".to_owned());
                }
                let mut outputs = execute()?;
                let [result] = outputs.runtime_results.as_slice() else {
                    return Err("runtime IPC graph must return exactly one bound result".to_owned());
                };
                if result.source_yir_fnv1a64 != target.source_yir_fnv1a64
                    || result.module != target.module
                    || result.instruction != target.instruction
                    || result.node != target.node
                    || result.resource != target.resource
                {
                    return Err("runtime IPC result target drift".to_owned());
                }
                observations.extend(runtime_dispatch_observations(
                    count,
                    &outputs.native_outputs,
                )?);
                replay_bytes = replay_bytes
                    .checked_add(result.payload.len())
                    .filter(|bytes| *bytes <= 64 * 1024 * 1024)
                    .ok_or("runtime IPC replay storage budget exceeded")?;
                let reply = Message::Frame(DispatchFrame {
                    sequence,
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
                reply.write_to(stream)?;
                count += 1;
            }
            Message::Finish(sequence) if sequence == count => {
                retained.runtime_session_evidence =
                    ProviderRuntimeDispatchSessionEvidence::from_observations(
                        count,
                        &observations,
                    )?;
                return Ok((count, retained));
            }
            _ => return Err("runtime IPC expected ordered dispatch or matching finish".to_owned()),
        }
    }
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
