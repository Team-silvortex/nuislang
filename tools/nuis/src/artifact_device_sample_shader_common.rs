pub(crate) const DIGEST_CONTRACT: &str = "nuis-code-asset-digest-fnv1a64-v1";
pub(crate) const U32_ELEMENT_COUNT: usize = 4;
pub(crate) const U32_INPUT: &[u8] = &[1, 0, 0, 0, 8, 0, 0, 0, 13, 0, 0, 0, 21, 0, 0, 0];
pub(crate) const U32_PAIR_RIGHT_INPUT: &[u8] = &[2, 0, 0, 0, 3, 0, 0, 0, 5, 0, 0, 0, 8, 0, 0, 0];
pub(crate) const U32_ADD_EXPECTED: &[u8] = &[2, 0, 0, 0, 16, 0, 0, 0, 26, 0, 0, 0, 42, 0, 0, 0];
pub(crate) const U32_PAIR_ADD_EXPECTED: &[u8] =
    &[3, 0, 0, 0, 11, 0, 0, 0, 18, 0, 0, 0, 29, 0, 0, 0];
pub(crate) const U32_ZERO_EXPECTED: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub(crate) const U32_MUL_EXPECTED: &[u8] = &[1, 0, 0, 0, 64, 0, 0, 0, 169, 0, 0, 0, 185, 1, 0, 0];

pub(crate) struct U32RequestEvidence<'a> {
    pub(crate) prefix: &'a str,
    pub(crate) provider_family: &'a str,
    pub(crate) kernel_id: &'a str,
    pub(crate) operation: &'a str,
    pub(crate) kernel_input_buffers: &'a str,
    pub(crate) buffer_layout: &'a str,
    pub(crate) buffer_shape: &'a str,
    pub(crate) row_stride_bytes: usize,
    pub(crate) dispatch: &'a str,
    pub(crate) input_file_name: &'a str,
    pub(crate) input_hash: String,
    pub(crate) input_byte_length: usize,
    pub(crate) expected_file_name: &'a str,
    pub(crate) expected: &'a [u8],
    pub(crate) asset: &'a nuisc::registry::NustarCodeAssetRegistration,
    pub(crate) bytes: &'a [u8],
    pub(crate) input_binding: String,
    pub(crate) dependency: String,
}

pub(crate) fn render_u32_sample_request_evidence(
    provider_family: &str,
    kernel_id: &str,
    operation: &str,
    input_file_name: &str,
    input: &[u8],
    expected_file_name: &str,
    expected: &[u8],
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
) -> String {
    render_u32_prefixed_request_evidence(U32RequestEvidence {
        prefix: "provider_",
        provider_family,
        kernel_id,
        operation,
        kernel_input_buffers: "input.values",
        buffer_layout: "tensor-contiguous",
        buffer_shape: "4",
        row_stride_bytes: input.len(),
        dispatch: "4x1x1",
        input_file_name,
        input_hash: fnv1a64_hex(input),
        input_byte_length: input.len(),
        expected_file_name,
        expected,
        asset,
        bytes,
        input_binding: render_u32_artifact_binding("provider_", input_file_name, input),
        dependency: render_dependency_count_zero("provider_"),
    })
}

pub(crate) fn render_u32_prefixed_request_evidence(args: U32RequestEvidence<'_>) -> String {
    format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.values;{prefix}buffer_element_type=u32;{prefix}buffer_layout={};{prefix}buffer_shape={};{prefix}buffer_row_stride_bytes={};{prefix}buffer_byte_length={};{prefix}buffer_payload_path={};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={};{prefix}kernel_operation={};{prefix}kernel_input_buffer=input.values;{prefix}kernel_input_buffers={};{prefix}kernel_output_buffer=output.values;{prefix}kernel_dispatch={};{prefix}kernel_scalar_bindings=element_count:u32:{U32_ELEMENT_COUNT};{prefix}code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;{prefix}code_asset_id={};{prefix}code_asset_format={};{prefix}code_asset_target={};{prefix}code_asset_entry={};{prefix}code_asset_path={};{prefix}code_asset_byte_length={};{prefix}code_asset_digest_contract={DIGEST_CONTRACT};{prefix}code_asset_content_hash={};{prefix}output_binding_contract=nuis-provider-output-binding-v2;{prefix}output_binding_count=1;{prefix}output_binding_0_role=output.result;{prefix}output_binding_0_buffer=output.values;{prefix}output_binding_0_element_type=u32;{prefix}output_binding_0_layout={};{prefix}output_binding_0_shape={};{prefix}output_binding_0_row_stride_bytes={};{prefix}output_binding_0_byte_length={};{prefix}output_binding_0_comparison_id=comparison.output.values;{prefix}output_comparison_id=comparison.output.values;{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer=output.values;{prefix}output_comparison_element_type=u32;{prefix}output_comparison_shape={};{prefix}output_comparison_expected_path={};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{};{};{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family={};{prefix}adapter_binding_execution_requirement=real-device",
        args.buffer_layout,
        args.buffer_shape,
        args.row_stride_bytes,
        args.input_byte_length,
        args.input_file_name,
        args.input_hash,
        args.kernel_id,
        args.operation,
        args.kernel_input_buffers,
        args.dispatch,
        args.asset.asset_id,
        args.asset.format,
        args.asset.target,
        args.asset.entry,
        args.asset.file_name,
        args.bytes.len(),
        fnv1a64_hex(args.bytes),
        args.buffer_layout,
        args.buffer_shape,
        args.row_stride_bytes,
        args.expected.len(),
        args.buffer_shape,
        args.expected_file_name,
        args.expected.len(),
        fnv1a64_hex(args.expected),
        args.dependency,
        args.input_binding,
        args.provider_family,
        prefix = args.prefix,
    )
}

pub(crate) fn render_dependency_count_zero(prefix: &str) -> String {
    format!(
        "{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=0"
    )
}

pub(crate) fn render_u32_dependency_edge(
    prefix: &str,
    producer_request_id: &str,
    producer_output_buffer: &str,
    consumer_input_buffer: &str,
    ownership_token: &str,
) -> String {
    format!(
        "{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id={producer_request_id};{prefix}dependency_0_producer_output_buffer={producer_output_buffer};{prefix}dependency_0_consumer_input_buffer={consumer_input_buffer};{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token={ownership_token};{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-0:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-1:dispatch-ready"
    )
}

pub(crate) fn render_u32_artifact_binding(
    prefix: &str,
    input_file_name: &str,
    input: &[u8],
) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.values;{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_element_type=u32;{prefix}input_binding_0_shape={U32_ELEMENT_COUNT};{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path={input_file_name};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none",
        input.len(),
        fnv1a64_hex(input),
    )
}

pub(crate) fn render_u32_pair_artifact_binding(
    prefix: &str,
    layout: &str,
    shape: &str,
    row_stride_bytes: usize,
    left_file_name: &str,
    left: &[u8],
    right_file_name: &str,
    right: &[u8],
) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v2;{prefix}input_binding_count=2;{prefix}input_binding_0_name=input.values;{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_element_type=u32;{prefix}input_binding_0_layout={layout};{prefix}input_binding_0_shape={shape};{prefix}input_binding_0_row_stride_bytes={row_stride_bytes};{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path={left_file_name};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none;{prefix}input_binding_1_name=input.right;{prefix}input_binding_1_source=artifact;{prefix}input_binding_1_element_type=u32;{prefix}input_binding_1_layout={layout};{prefix}input_binding_1_shape={shape};{prefix}input_binding_1_row_stride_bytes={row_stride_bytes};{prefix}input_binding_1_byte_length={};{prefix}input_binding_1_content_hash={};{prefix}input_binding_1_payload_path={right_file_name};{prefix}input_binding_1_producer_request_id=none;{prefix}input_binding_1_producer_output_buffer=none",
        left.len(),
        fnv1a64_hex(left),
        right.len(),
        fnv1a64_hex(right),
    )
}

pub(crate) fn render_u32_dependency_binding(
    prefix: &str,
    input_hash: &str,
    input_byte_length: usize,
    producer_request_id: &str,
    producer_output_buffer: &str,
) -> String {
    format!(
        "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.values;{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type=u32;{prefix}input_binding_0_shape={U32_ELEMENT_COUNT};{prefix}input_binding_0_byte_length={input_byte_length};{prefix}input_binding_0_content_hash={input_hash};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id={producer_request_id};{prefix}input_binding_0_producer_output_buffer={producer_output_buffer}"
    )
}

pub(crate) fn replace_code_asset_identity_fields(
    evidence: &str,
    replacements: &[(String, usize, String)],
) -> String {
    evidence
        .split(';')
        .map(|field| {
            let Some((key, value)) = field.split_once('=') else {
                return field.to_owned();
            };
            for (prefix, byte_length, content_hash) in replacements {
                if key == format!("{prefix}code_asset_byte_length") {
                    return format!("{key}={byte_length}");
                }
                if key == format!("{prefix}code_asset_content_hash") {
                    return format!("{key}={content_hash}");
                }
            }
            format!("{key}={value}")
        })
        .collect::<Vec<_>>()
        .join(";")
}

pub(crate) fn validate_code_asset_request_evidence(
    provider_label: &str,
    asset: &nuisc::registry::NustarCodeAssetRegistration,
    bytes: &[u8],
    evidence: &str,
    prefix: &str,
) -> Result<(), String> {
    for (field, expected) in [
        ("code_asset_id", asset.asset_id.as_str()),
        ("code_asset_format", asset.format.as_str()),
        ("code_asset_target", asset.target.as_str()),
        ("code_asset_entry", asset.entry.as_str()),
        ("code_asset_path", asset.file_name.as_str()),
        ("code_asset_digest_contract", DIGEST_CONTRACT),
    ] {
        let needle = format!("{prefix}{field}={expected}");
        if !evidence.split(';').any(|item| item == needle) {
            return Err(format!(
                "{provider_label} provider request code asset field `{field}` does not match `{expected}`"
            ));
        }
    }
    let expected_length = format!("{prefix}code_asset_byte_length={}", bytes.len());
    let expected_hash = format!("{prefix}code_asset_content_hash={}", fnv1a64_hex(bytes));
    if !evidence.split(';').any(|item| item == expected_length)
        || !evidence.split(';').any(|item| item == expected_hash)
    {
        return Err(format!(
            "{provider_label} provider request code asset byte identity does not match"
        ));
    }
    Ok(())
}

pub(crate) fn validate_code_asset_contribution_selection(
    provider_label: &str,
    selection: &crate::artifact_code_asset_contribution_table::SelectedCodeAssetContribution,
    evidence: &str,
    prefix: &str,
) -> Result<(), String> {
    for (field, expected) in [
        ("code_asset_id", selection.asset_id.as_str()),
        ("code_asset_format", selection.format.as_str()),
        ("code_asset_target", selection.target.as_str()),
        ("code_asset_path", selection.path.as_str()),
        ("code_asset_content_hash", selection.content_hash.as_str()),
    ] {
        let needle = format!("{prefix}{field}={expected}");
        if !evidence.split(';').any(|item| item == needle) {
            return Err(format!(
                "{provider_label} provider request does not match compiled contribution field `{field}`"
            ));
        }
    }
    Ok(())
}

pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}
