use std::{collections::BTreeMap, fs, path::Path};

const CODEGEN_TABLE_FILE_NAME: &str = "nuis.domain.kernel.codegen-table.toml";
const CODEGEN_TABLE_CONTRACT: &str = "nuis-kernel-yir-codegen-table-v1";
const REQUEST_PROJECTION_CONTRACT: &str = "nuis-kernel-source-request-projection-v1";

pub(crate) struct ProjectKernelRequestProjection {
    source_function: String,
    source_node: String,
    entry: String,
    operation: String,
    input_shape: Vec<usize>,
    output_shape: Vec<usize>,
    input_values: Vec<i64>,
    scalar: Option<i64>,
    input_source_node: Option<String>,
    expected_values: Vec<i64>,
}

pub(crate) fn augment_evidence(
    output_dir: &Path,
    evidence: &str,
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
    asset_bytes: &[u8],
) -> Result<String, String> {
    let projections = load_projections(output_dir)?;
    if projections.is_empty() {
        return Ok(evidence.to_owned());
    }
    let old_count = "provider_request_count=2";
    if evidence.matches(old_count).count() != 1 {
        return Err(
            "Kernel provider request collection has an invalid compatibility count".to_owned(),
        );
    }
    let mut resolved = evidence.replace(
        old_count,
        &format!("provider_request_count={}", 2 + projections.len()),
    );
    let projection_indices = projections
        .iter()
        .enumerate()
        .map(|(offset, projection)| (projection.source_node.as_str(), 2 + offset))
        .collect::<BTreeMap<_, _>>();
    let projection_by_node = projections
        .iter()
        .map(|projection| (projection.source_node.as_str(), projection))
        .collect::<BTreeMap<_, _>>();
    for (offset, projection) in projections.iter().enumerate() {
        resolved.push(';');
        resolved.push_str(&render_request(
            2 + offset,
            projection,
            &projection_indices,
            &projection_by_node,
            asset,
            asset_bytes,
        )?);
    }
    Ok(resolved)
}

pub(crate) fn persist_payloads(output_dir: &Path) -> Result<(), String> {
    for projection in load_projections(output_dir)? {
        if projection.input_source_node.is_none() {
            fs::write(
                output_dir.join(projection.input_file_name()),
                encode_i64_values(&projection.input_values),
            )
            .map_err(|error| format!("failed to persist project Kernel input payload: {error}"))?;
        }
        fs::write(
            output_dir.join(projection.expected_file_name()),
            encode_i64_values(&projection.expected_values),
        )
        .map_err(|error| format!("failed to persist project Kernel expected payload: {error}"))?;
    }
    Ok(())
}

fn load_projections(output_dir: &Path) -> Result<Vec<ProjectKernelRequestProjection>, String> {
    let path = output_dir.join(CODEGEN_TABLE_FILE_NAME);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(format!(
                "failed to read project Kernel codegen table `{}`: {error}",
                path.display()
            ));
        }
    };
    if string_field(&source, "schema").as_deref() != Some(CODEGEN_TABLE_CONTRACT) {
        return Err("project Kernel codegen table contract is invalid".to_owned());
    }
    source
        .split("[[source_adaptation]]")
        .skip(1)
        .filter(|record| {
            string_field(record, "request_projection_contract").as_deref()
                == Some(REQUEST_PROJECTION_CONTRACT)
        })
        .map(parse_projection)
        .collect()
}

fn parse_projection(record: &str) -> Result<ProjectKernelRequestProjection, String> {
    if string_field(record, "status").as_deref() != Some("adapted")
        || string_field(record, "request_element_type").as_deref() != Some("i64")
    {
        return Err("project Kernel request projection status/type is invalid".to_owned());
    }
    let projection = ProjectKernelRequestProjection {
        source_function: required_string(record, "source_function")?,
        source_node: required_string(record, "source_node")?,
        entry: required_string(record, "generated_entry")?,
        operation: required_string(record, "request_operation")?,
        input_shape: usize_array_field(record, "request_input_shape")?,
        output_shape: usize_array_field(record, "request_output_shape")?,
        input_values: i64_array_field(record, "request_input_values")?,
        scalar: optional_integer_field(record, "request_scalar")?,
        input_source_node: string_field(record, "request_input_source_node"),
        expected_values: i64_array_field(record, "request_expected_values")?,
    };
    let input_element_count = projection
        .input_shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .ok_or_else(|| "project Kernel request projection shape overflows".to_owned())?;
    let output_element_count = projection
        .output_shape
        .iter()
        .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
        .ok_or_else(|| "project Kernel request projection output shape overflows".to_owned())?;
    let expected = expected_projection_values(&projection)?;
    if projection.input_shape.is_empty()
        || projection.output_shape.is_empty()
        || projection
            .input_shape
            .iter()
            .chain(&projection.output_shape)
            .any(|dimension| *dimension == 0)
        || projection.input_values.len() != input_element_count
        || projection.expected_values.len() != output_element_count
        || projection.expected_values != expected
        || !valid_identifier(&projection.entry)
    {
        return Err("project Kernel request projection evidence is inconsistent".to_owned());
    }
    Ok(projection)
}

fn render_request(
    index: usize,
    projection: &ProjectKernelRequestProjection,
    projection_indices: &BTreeMap<&str, usize>,
    projection_by_node: &BTreeMap<&str, &ProjectKernelRequestProjection>,
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
    asset_bytes: &[u8],
) -> Result<String, String> {
    let prefix = format!("provider_request_{index}_");
    let input_shape = render_shape(&projection.input_shape);
    let output_shape = render_shape(&projection.output_shape);
    let input_element_count = projection.input_values.len();
    let output_element_count = projection.expected_values.len();
    let input_byte_length = input_element_count
        .checked_mul(std::mem::size_of::<i64>())
        .ok_or_else(|| "project Kernel request payload length overflows".to_owned())?;
    let output_byte_length = output_element_count
        .checked_mul(std::mem::size_of::<i64>())
        .ok_or_else(|| "project Kernel request output length overflows".to_owned())?;
    let row_stride = projection
        .input_shape
        .last()
        .and_then(|cols| cols.checked_mul(std::mem::size_of::<i64>()))
        .ok_or_else(|| "project Kernel request row stride overflows".to_owned())?;
    let input = encode_i64_values(&projection.input_values);
    let expected = encode_i64_values(&projection.expected_values);
    let identity = projection.identity();
    let scalar_bindings = match projection.operation.as_str() {
        "add-scalar-i64" => format!(
            "element_count:u32:{input_element_count},scalar:i64:{},device_selection_policy:u32:1,minimum_compute_capability:u32:{}",
            projection
                .scalar
                .ok_or_else(|| "project scalar-map request has no scalar".to_owned())?,
            asset.minimum_compute_capability
        ),
        "reduce-sum-i64" => format!(
            "element_count:u32:{input_element_count},device_selection_policy:u32:1,minimum_compute_capability:u32:{}",
            asset.minimum_compute_capability
        ),
        operation => {
            return Err(format!(
                "project Kernel request operation `{operation}` is unsupported"
            ));
        }
    };
    let dispatch = if projection.operation == "reduce-sum-i64" {
        "1x1x1".to_owned()
    } else {
        format!("{output_element_count}x1x1")
    };
    let (input_path, dependency, input_binding) = match &projection.input_source_node {
        None => (
            projection.input_file_name(),
            format!("{prefix}dependency_count=0"),
            format!(
                "{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_payload_path={};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none",
                projection.input_file_name()
            ),
        ),
        Some(source_node) => {
            let producer = projection_by_node.get(source_node.as_str()).ok_or_else(|| {
                format!(
                    "project Kernel request `{}` has unknown input source `{source_node}`",
                    projection.source_node
                )
            })?;
            let producer_index = projection_indices[source_node.as_str()];
            if producer_index >= index || producer.expected_values != projection.input_values {
                return Err(format!(
                    "project Kernel request `{}` has an invalid upstream projection",
                    projection.source_node
                ));
            }
            let producer_request_id = producer.request_id();
            let producer_output_buffer = producer.output_buffer();
            let consumer_input_buffer = format!("input.{identity}");
            (
                producer.expected_file_name(),
                format!(
                    "{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id={producer_request_id};{prefix}dependency_0_producer_output_buffer={producer_output_buffer};{prefix}dependency_0_consumer_input_buffer={consumer_input_buffer};{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:{producer_request_id}:{producer_output_buffer}->{}:{consumer_input_buffer};{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-{producer_index}:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-{index}:dispatch-ready",
                    projection.request_id()
                ),
                format!(
                    "{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id={producer_request_id};{prefix}input_binding_0_producer_output_buffer={producer_output_buffer}"
                ),
            )
        }
    };
    Ok(format!(
        "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id=input.{identity};{prefix}buffer_element_type=i64;{prefix}buffer_layout=tensor-contiguous;{prefix}buffer_shape={input_shape};{prefix}buffer_row_stride_bytes={row_stride};{prefix}buffer_byte_length={input_byte_length};{prefix}buffer_payload_path={input_path};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={};{prefix}kernel_operation={};{prefix}kernel_input_buffer=input.{identity};{prefix}kernel_output_buffer={};{prefix}kernel_dispatch={dispatch};{prefix}kernel_scalar_bindings={scalar_bindings};{prefix}code_asset_descriptor_contract=nuis-provider-code-asset-descriptor-v1;{prefix}code_asset_id={};{prefix}code_asset_format={};{prefix}code_asset_target={};{prefix}code_asset_entry={};{prefix}code_asset_path={};{prefix}code_asset_byte_length={};{prefix}code_asset_digest_contract={};{prefix}code_asset_content_hash={};{prefix}output_binding_contract=nuis-provider-output-binding-v1;{prefix}output_binding_count=1;{prefix}output_binding_0_role=output.source;{prefix}output_binding_0_buffer={};{prefix}output_binding_0_element_type=i64;{prefix}output_binding_0_shape={output_shape};{prefix}output_binding_0_byte_length={output_byte_length};{prefix}output_binding_0_comparison_id=comparison.output.{identity};{prefix}output_comparison_id=comparison.output.{identity};{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer={};{prefix}output_comparison_element_type=i64;{prefix}output_comparison_shape={output_shape};{prefix}output_comparison_expected_path={};{prefix}output_comparison_expected_byte_length={output_byte_length};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject;{prefix}dependency_contract=nuis-provider-request-dependency-v1;{dependency};{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name=input.{identity};{prefix}input_binding_0_element_type=i64;{prefix}input_binding_0_shape={input_shape};{prefix}input_binding_0_byte_length={input_byte_length};{prefix}input_binding_0_content_hash={};{input_binding};{prefix}adapter_binding_contract=nuis-provider-request-adapter-binding-v1;{prefix}adapter_binding_provider_family=cuda:nvidia-gpu;{prefix}adapter_binding_execution_requirement=real-device",
        fnv1a64_hex(&input),
        projection.request_id(),
        projection.operation,
        projection.output_buffer(),
        asset.id,
        asset.format,
        asset.target,
        projection.entry,
        asset.file_name,
        asset_bytes.len(),
        asset.digest_contract,
        fnv1a64_hex(asset_bytes),
        projection.output_buffer(),
        projection.output_buffer(),
        projection.expected_file_name(),
        fnv1a64_hex(&expected),
        fnv1a64_hex(&input),
    ))
}

impl ProjectKernelRequestProjection {
    fn identity(&self) -> String {
        format!(
            "{}.{}",
            identifier_fragment(&self.source_function),
            identifier_fragment(&self.source_node)
        )
    }

    fn input_file_name(&self) -> String {
        format!("nuis.kernel.cuda.source.{}.input.i64.bin", self.identity())
    }

    fn request_id(&self) -> String {
        format!("kernel.cuda.source.{}.i64", self.identity())
    }

    fn output_buffer(&self) -> String {
        format!("output.{}", self.identity())
    }

    fn expected_file_name(&self) -> String {
        format!(
            "nuis.kernel.cuda.source.{}.expected.i64.bin",
            self.identity()
        )
    }
}

fn encode_i64_values(values: &[i64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn expected_projection_values(
    projection: &ProjectKernelRequestProjection,
) -> Result<Vec<i64>, String> {
    match projection.operation.as_str() {
        "add-scalar-i64" if projection.input_source_node.is_none() => {
            let scalar = projection
                .scalar
                .ok_or_else(|| "project scalar-map request has no scalar".to_owned())?;
            projection
                .input_values
                .iter()
                .map(|value| value.checked_add(scalar))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| "project Kernel request projection arithmetic overflows".to_owned())
        }
        "reduce-sum-i64"
            if projection.scalar.is_none() && projection.input_source_node.is_some() =>
        {
            projection
                .input_values
                .iter()
                .try_fold(0i64, |sum, value| sum.checked_add(*value))
                .map(|sum| vec![sum])
                .ok_or_else(|| "project Kernel reduction projection overflows".to_owned())
        }
        _ => Err("project Kernel request projection operation is inconsistent".to_owned()),
    }
}

fn render_shape(shape: &[usize]) -> String {
    shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join("x")
}

fn required_string(source: &str, key: &str) -> Result<String, String> {
    string_field(source, key)
        .ok_or_else(|| format!("project Kernel request projection is missing `{key}`"))
}

fn string_field(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let (field, value) = line.trim().split_once(" = ")?;
        (field == key).then(|| {
            value
                .strip_prefix('"')?
                .strip_suffix('"')
                .map(str::to_owned)
        })?
    })
}

fn optional_integer_field(source: &str, key: &str) -> Result<Option<i64>, String> {
    let value = source.lines().find_map(|line| {
        let (field, value) = line.trim().split_once(" = ")?;
        (field == key).then_some(value)
    });
    value
        .map(|value| {
            value
                .parse()
                .map(Some)
                .map_err(|_| format!("project Kernel request projection has invalid `{key}`"))
        })
        .unwrap_or(Ok(None))
}

fn usize_array_field(source: &str, key: &str) -> Result<Vec<usize>, String> {
    array_field(source, key)?
        .into_iter()
        .map(|value| value.parse().map_err(|_| format!("invalid `{key}` item")))
        .collect()
}

fn i64_array_field(source: &str, key: &str) -> Result<Vec<i64>, String> {
    array_field(source, key)?
        .into_iter()
        .map(|value| value.parse().map_err(|_| format!("invalid `{key}` item")))
        .collect()
}

fn array_field<'a>(source: &'a str, key: &str) -> Result<Vec<&'a str>, String> {
    let value = source
        .lines()
        .find_map(|line| {
            let (field, value) = line.trim().split_once(" = ")?;
            (field == key).then_some(value)
        })
        .and_then(|value| value.strip_prefix('[')?.strip_suffix(']'))
        .ok_or_else(|| format!("project Kernel request projection has invalid `{key}`"))?;
    if value.is_empty() {
        return Ok(Vec::new());
    }
    Ok(value.split(',').map(str::trim).collect())
}

fn identifier_fragment(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect()
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}
