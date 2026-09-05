use crate::{
    provider_execution_adapter::ProviderRequestExecution, provider_request::ProviderRequest,
    provider_sample_artifact::fnv1a64_hex,
};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub const PROVIDER_RUNTIME_RESULT_STREAM_CONTRACT: &str = "nuis-provider-runtime-result-stream-v2";
pub const PROVIDER_RUNTIME_RESULT_STREAM_FILE_NAME: &str =
    "nuis.runtime.provider-result-stream.toml";
const PAYLOAD_PREFIX: &str = "nuis.runtime.provider-result.";
const PAYLOAD_SUFFIX: &str = ".bin";
const MAX_RUNTIME_RESULTS: usize = 256;

pub fn provider_runtime_result_stream_path(output_dir: &Path) -> PathBuf {
    output_dir.join(PROVIDER_RUNTIME_RESULT_STREAM_FILE_NAME)
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProviderRuntimeResultTarget {
    pub source_yir_fnv1a64: String,
    pub provider_family: String,
    pub module: String,
    pub instruction: String,
    pub node: String,
    pub resource: String,
}

#[allow(dead_code)] // Public through the library; the standalone CLI has not exposed this query yet.
pub fn provider_runtime_result_targets(
    output_dir: &Path,
    provider_family_filter: Option<&str>,
) -> Result<Vec<ProviderRuntimeResultTarget>, String> {
    let manifest = crate::provider_sample::read_device_provider_sample_manifest_info(output_dir);
    if !manifest.available {
        return Ok(Vec::new());
    }
    if manifest.protocol != crate::provider_sample::DEVICE_PROVIDER_SAMPLE_PROTOCOL
        || manifest.schema != crate::provider_sample::DEVICE_PROVIDER_SAMPLE_SCHEMA
    {
        return Err(format!(
            "provider runtime result discovery rejected manifest protocol `{}` schema `{}`",
            manifest.protocol, manifest.schema
        ));
    }
    if manifest.invalid_record_count != 0 {
        return Err(
            "provider runtime result discovery rejected invalid manifest records".to_owned(),
        );
    }
    let mut targets = BTreeSet::new();
    for record in manifest.records.iter().filter(|record| {
        provider_family_filter.is_none_or(|family| record.provider_family == family)
    }) {
        let collection = crate::provider_request::provider_request_collection_from_evidence(
            &record.input_evidence,
        );
        if collection.is_none()
            && record
                .input_evidence
                .contains("runtime_result_binding_contract=")
        {
            return Err(format!(
                "provider runtime result binding in `{}` is malformed",
                record.trace_id
            ));
        }
        let Some(collection) = collection else {
            continue;
        };
        for request in collection.requests {
            let Some(binding) = request.runtime_result_binding else {
                continue;
            };
            let target = ProviderRuntimeResultTarget {
                source_yir_fnv1a64: binding.source_yir_fnv1a64,
                provider_family: record.provider_family.clone(),
                module: binding.module,
                instruction: binding.instruction,
                node: binding.node,
                resource: binding.resource,
            };
            if !targets.insert(target) {
                return Err("provider runtime result binding target is duplicated".to_owned());
            }
        }
    }
    if targets.len() > MAX_RUNTIME_RESULTS {
        return Err("provider runtime result target count exceeds protocol limit".to_owned());
    }
    Ok(targets.into_iter().collect())
}

pub(crate) struct ProviderRuntimeResult {
    pub(crate) arguments: yir_core::provider_runtime_ipc::DispatchArguments,
    pub(crate) source_yir_fnv1a64: String,
    pub(crate) provider_family: String,
    pub(crate) request_id: String,
    pub(crate) module: String,
    pub(crate) instruction: String,
    pub(crate) node: String,
    pub(crate) resource: String,
    pub(crate) element_type: String,
    pub(crate) layout: String,
    pub(crate) shape: Vec<usize>,
    pub(crate) row_stride_bytes: usize,
    pub(crate) payload: Vec<u8>,
    pub(crate) completion_wire: String,
}

impl ProviderRuntimeResult {
    pub(crate) fn from_execution(
        provider_family: &str,
        request: &ProviderRequest,
        execution: &ProviderRequestExecution,
        arguments: Option<&yir_core::provider_runtime_ipc::DispatchArguments>,
    ) -> Result<Option<Self>, String> {
        let Some(binding) = request.runtime_result_binding.as_ref() else {
            return Ok(None);
        };
        let output = request.output_bindings.first().ok_or_else(|| {
            format!(
                "provider request `{}` has no runtime output",
                request.kernel.id
            )
        })?;
        let payload = execution.output_payload.as_bytes();
        if payload.len() != output.byte_length {
            return Err(format!(
                "provider request `{}` runtime output byte length drifted",
                request.kernel.id
            ));
        }
        if execution.summary.completion_evidence_contract
            != yir_core::PROVIDER_PHYSICAL_COMPLETION_CONTRACT
        {
            return Err(format!(
                "provider request `{}` runtime result lacks physical completion evidence",
                request.kernel.id
            ));
        }
        yir_core::ProviderPhysicalCompletion::parse(&execution.summary.completion_clock_evidence)?;
        Ok(Some(Self {
            arguments: arguments
                .ok_or("runtime result lacks validated dispatch arguments")?
                .descriptor(),
            source_yir_fnv1a64: binding.source_yir_fnv1a64.clone(),
            provider_family: provider_family.to_owned(),
            request_id: request.kernel.id.clone(),
            module: binding.module.clone(),
            instruction: binding.instruction.clone(),
            node: binding.node.clone(),
            resource: binding.resource.clone(),
            element_type: output.element_type.clone(),
            layout: output.layout.clone(),
            shape: output.shape.clone(),
            row_stride_bytes: output.row_stride_bytes,
            payload: payload.to_vec(),
            completion_wire: execution.summary.completion_clock_evidence.clone(),
        }))
    }
}

pub(crate) fn persist_provider_runtime_results(
    output_dir: &Path,
    results: &[ProviderRuntimeResult],
) -> Result<PathBuf, String> {
    if results.is_empty() || results.len() > MAX_RUNTIME_RESULTS {
        return Err("provider runtime result stream count is invalid".to_owned());
    }
    let source_yir_fnv1a64 = &results[0].source_yir_fnv1a64;
    if results
        .iter()
        .any(|result| result.source_yir_fnv1a64 != *source_yir_fnv1a64)
    {
        return Err("provider runtime result stream mixes source YIR identities".to_owned());
    }
    clear_previous_stream(output_dir)?;

    let mut records = Vec::with_capacity(results.len());
    for (index, result) in results.iter().enumerate() {
        validate_result(result)?;
        let payload_path = format!("{PAYLOAD_PREFIX}{index:04}{PAYLOAD_SUFFIX}");
        let payload_hash = fnv1a64_hex(&result.payload);
        fs::write(output_dir.join(&payload_path), &result.payload).map_err(|error| {
            format!("failed to write provider runtime result `{payload_path}`: {error}")
        })?;
        records.push(RuntimeResultRecord {
            index,
            result,
            payload_path,
            payload_hash,
        });
    }

    let stream_hash = stream_hash(source_yir_fnv1a64, &records);
    let mut manifest = format!(
        "schema = \"{PROVIDER_RUNTIME_RESULT_STREAM_CONTRACT}\"\nsource_yir_fnv1a64 = \"{source_yir_fnv1a64}\"\nframe_count = {}\nstream_hash = \"{stream_hash}\"\n",
        records.len()
    );
    for record in &records {
        let result = record.result;
        manifest.push_str("\n[[frame]]\n");
        push_string(
            &mut manifest,
            "dispatch_arguments",
            &result.arguments.to_wire()?,
        );
        for (name, value) in [
            ("request_id", result.request_id.as_str()),
            ("provider_family", result.provider_family.as_str()),
            ("module", result.module.as_str()),
            ("instruction", result.instruction.as_str()),
            ("node", result.node.as_str()),
            ("resource", result.resource.as_str()),
            ("element_type", result.element_type.as_str()),
            ("layout", result.layout.as_str()),
        ] {
            push_string(&mut manifest, name, value);
        }
        manifest.push_str(&format!(
            "index = {}\nshape = \"{}\"\nrow_stride_bytes = {}\npayload_path = \"{}\"\npayload_byte_length = {}\npayload_hash = \"{}\"\ncompletion_wire = \"{}\"\n",
            record.index,
            render_shape(&result.shape),
            result.row_stride_bytes,
            record.payload_path,
            result.payload.len(),
            record.payload_hash,
            escape_toml(&result.completion_wire),
        ));
    }
    let path = provider_runtime_result_stream_path(output_dir);
    fs::write(&path, manifest)
        .map_err(|error| format!("failed to write provider runtime result stream: {error}"))?;
    Ok(path)
}

struct RuntimeResultRecord<'a> {
    index: usize,
    result: &'a ProviderRuntimeResult,
    payload_path: String,
    payload_hash: String,
}

fn stream_hash(source_yir_fnv1a64: &str, records: &[RuntimeResultRecord<'_>]) -> String {
    let mut material = format!(
        "{PROVIDER_RUNTIME_RESULT_STREAM_CONTRACT}\n{source_yir_fnv1a64}\n{}",
        records.len()
    );
    for record in records {
        let result = record.result;
        material.push_str(&format!(
            "\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            record.index,
            result.request_id,
            result.provider_family,
            result.module,
            result.instruction,
            result.node,
            result.resource,
            result.element_type,
            result.layout,
            render_shape(&result.shape),
            result.row_stride_bytes,
            record.payload_path,
            result.payload.len(),
            record.payload_hash,
            result.completion_wire,
        ));
        material.push('\n');
        material.push_str(
            &result
                .arguments
                .to_wire()
                .expect("validated runtime arguments"),
        );
    }
    fnv1a64_hex(material.as_bytes())
}

fn validate_result(result: &ProviderRuntimeResult) -> Result<(), String> {
    result.arguments.to_wire()?;
    if !valid_hash(&result.source_yir_fnv1a64)
        || result.provider_family.is_empty()
        || result.request_id.is_empty()
        || result.module.is_empty()
        || result.instruction.is_empty()
        || result.node.is_empty()
        || result.resource.is_empty()
        || result.element_type.is_empty()
        || result.layout.is_empty()
        || result.shape.is_empty()
        || result.shape.contains(&0)
        || result.row_stride_bytes == 0
        || result.payload.is_empty()
    {
        return Err("provider runtime result is invalid".to_owned());
    }
    yir_core::ProviderPhysicalCompletion::parse(&result.completion_wire)?;
    Ok(())
}

fn clear_previous_stream(output_dir: &Path) -> Result<(), String> {
    let _ = fs::remove_file(provider_runtime_result_stream_path(output_dir));
    let entries = fs::read_dir(output_dir)
        .map_err(|error| format!("failed to enumerate provider runtime results: {error}"))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(PAYLOAD_PREFIX) && name.ends_with(PAYLOAD_SUFFIX) {
            fs::remove_file(entry.path()).map_err(|error| {
                format!("failed to remove stale provider runtime result `{name}`: {error}")
            })?;
        }
    }
    Ok(())
}

fn render_shape(shape: &[usize]) -> String {
    shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join("x")
}

fn push_string(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!("{name} = \"{}\"\n", escape_toml(value)));
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
