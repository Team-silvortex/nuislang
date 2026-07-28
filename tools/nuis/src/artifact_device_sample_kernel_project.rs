use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

const CODEGEN_TABLE_FILE_NAME: &str = "nuis.domain.kernel.codegen-table.toml";
const CODEGEN_TABLE_CONTRACT: &str = "nuis-kernel-yir-codegen-table-v1";
const REQUEST_PROJECTION_CONTRACT: &str = "nuis-kernel-source-request-projection-v1";
const SOURCE_RESULT_PROJECTION_CONTRACT: &str = "nuis-kernel-source-result-projection-v1";
const PROJECT_CODE_ASSET_IDENTITY_CONTRACT: &str = "nuis-kernel-project-code-asset-identity-v1";
const CODE_ASSET_IDENTITY_SET_CONTRACT: &str = "nuis-provider-code-asset-identity-set-v1";
const PROVIDER_RESULT_PROJECTION_COLLECTION_CONTRACT: &str =
    "nuis-provider-result-projection-collection-v1";
const PROVIDER_RESULT_PROJECTION_CONTRACT: &str = "nuis-provider-result-projection-v1";

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

struct ProjectKernelResultProjection {
    source_function: String,
    source_node: String,
    input_source_node: String,
    expected_i64: i64,
}

struct ProjectKernelCodeAssetIdentity {
    asset_id: String,
    source_fnv1a64: String,
    lowering_target: String,
    entries: Vec<String>,
    identity_hash: String,
    identity_set_root_hash: String,
}

pub(crate) fn augment_evidence(
    output_dir: &Path,
    evidence: &str,
    asset: &nuisc::kernel_code_asset::RegisteredKernelCodeAsset,
    asset_bytes: &[u8],
) -> Result<String, String> {
    let projections = load_projections(output_dir)?;
    let result_projections = load_result_projections(output_dir)?;
    if projections.is_empty() && result_projections.is_empty() {
        return Ok(evidence.to_owned());
    }
    if projections.is_empty() {
        return Err(
            "project Kernel result projections require a project request collection".to_owned(),
        );
    }
    let project_asset_identity = load_project_code_asset_identity(output_dir, &projections)?;
    let mut resolved = replace_compatibility_collection(evidence, projections.len())?;
    resolved.push(';');
    resolved.push_str(&render_project_code_asset_identity(&project_asset_identity));
    let projection_indices = projections
        .iter()
        .enumerate()
        .map(|(index, projection)| (projection.source_node.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let projection_by_node = projections
        .iter()
        .map(|projection| (projection.source_node.as_str(), projection))
        .collect::<BTreeMap<_, _>>();
    for (index, projection) in projections.iter().enumerate() {
        resolved.push(';');
        resolved.push_str(&render_request(
            index,
            projection,
            &projection_indices,
            &projection_by_node,
            &project_asset_identity.asset_id,
            asset,
            asset_bytes,
        )?);
    }
    if !result_projections.is_empty() {
        resolved.push_str(&format!(
            ";provider_result_projection_collection_contract={PROVIDER_RESULT_PROJECTION_COLLECTION_CONTRACT};provider_result_projection_count={}",
            result_projections.len()
        ));
        for (index, result) in result_projections.iter().enumerate() {
            let producer = projection_by_node
                .get(result.input_source_node.as_str())
                .ok_or_else(|| {
                    format!(
                        "project Kernel result `{}` has unknown input source `{}`",
                        result.source_node, result.input_source_node
                    )
                })?;
            if producer.source_function != result.source_function
                || producer.output_shape != [1, 1]
                || producer.expected_values != [result.expected_i64]
            {
                return Err(format!(
                    "project Kernel result `{}` does not match its producer",
                    result.source_node
                ));
            }
            resolved.push(';');
            resolved.push_str(&render_result_projection(index, result, producer));
        }
    }
    Ok(resolved)
}

pub(crate) fn uses_project_request_collection(output_dir: &Path) -> Result<bool, String> {
    Ok(!load_projections(output_dir)?.is_empty())
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

fn replace_compatibility_collection(
    evidence: &str,
    project_count: usize,
) -> Result<String, String> {
    let mut compatibility_count = None;
    for field in evidence.split(';') {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        if key == "provider_request_count" {
            if compatibility_count.is_some() {
                return Err(
                    "Kernel provider request collection has duplicate count evidence".to_owned(),
                );
            }
            compatibility_count = Some(value.parse::<usize>().map_err(|_| {
                "Kernel provider request collection has an invalid compatibility count".to_owned()
            })?);
        }
    }
    let compatibility_count = compatibility_count.ok_or_else(|| {
        "Kernel provider request collection has no compatibility count".to_owned()
    })?;
    if compatibility_count == 0 {
        return Err(
            "Kernel provider request collection has an empty compatibility graph".to_owned(),
        );
    }

    let mut removed_indices = BTreeSet::new();
    let compatibility_suffixes = evidence
        .split(';')
        .filter_map(|field| field.split_once('=').map(|(key, _)| key))
        .filter_map(|key| {
            let suffix = key.strip_prefix("provider_request_")?;
            let (index, suffix) = suffix.split_once('_')?;
            (index.parse::<usize>().ok()? < compatibility_count).then_some(suffix)
        })
        .collect::<BTreeSet<_>>();
    let mut fields = Vec::new();
    for field in evidence.split(';') {
        let Some((key, value)) = field.split_once('=') else {
            fields.push(field.to_owned());
            continue;
        };
        if key == "provider_request_count" {
            fields.push(format!("{key}={project_count}"));
            continue;
        }
        let indexed_request = key
            .strip_prefix("provider_request_")
            .and_then(|suffix| suffix.split_once('_'))
            .and_then(|(index, _)| index.parse::<usize>().ok());
        if indexed_request.is_some_and(|index| index < compatibility_count) {
            removed_indices.insert(indexed_request.expect("checked indexed request"));
            continue;
        }
        if key
            .strip_prefix("provider_")
            .is_some_and(|suffix| compatibility_suffixes.contains(suffix))
        {
            continue;
        }
        fields.push(format!("{key}={value}"));
    }
    if removed_indices.len() != compatibility_count {
        return Err(
            "Kernel provider request collection has incomplete compatibility requests".to_owned(),
        );
    }
    Ok(fields.join(";"))
}

fn load_projections(output_dir: &Path) -> Result<Vec<ProjectKernelRequestProjection>, String> {
    let Some(source) = load_codegen_table_source(output_dir)? else {
        return Ok(Vec::new());
    };
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

fn load_result_projections(
    output_dir: &Path,
) -> Result<Vec<ProjectKernelResultProjection>, String> {
    let Some(source) = load_codegen_table_source(output_dir)? else {
        return Ok(Vec::new());
    };
    source
        .split("[[source_adaptation]]")
        .skip(1)
        .filter(|record| {
            string_field(record, "result_projection_contract").as_deref()
                == Some(SOURCE_RESULT_PROJECTION_CONTRACT)
        })
        .map(|record| {
            if string_field(record, "status").as_deref() != Some("projected")
                || string_field(record, "result_element_type").as_deref() != Some("i64")
                || integer_field(record, "result_row")? != 0
                || integer_field(record, "result_col")? != 0
            {
                return Err("project Kernel result projection is inconsistent".to_owned());
            }
            Ok(ProjectKernelResultProjection {
                source_function: required_string(record, "source_function")?,
                source_node: required_string(record, "source_node")?,
                input_source_node: required_string(record, "result_input_source_node")?,
                expected_i64: integer_field(record, "result_expected_i64")?,
            })
        })
        .collect()
}

fn load_codegen_table_source(output_dir: &Path) -> Result<Option<String>, String> {
    let path = output_dir.join(CODEGEN_TABLE_FILE_NAME);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
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
    Ok(Some(source))
}

fn load_project_code_asset_identity(
    output_dir: &Path,
    projections: &[ProjectKernelRequestProjection],
) -> Result<ProjectKernelCodeAssetIdentity, String> {
    let source = load_codegen_table_source(output_dir)?
        .ok_or_else(|| "project Kernel code asset identity has no codegen table".to_owned())?;
    let source_fnv1a64 = required_string(&source, "source_fnv1a64")?;
    let identity_source_fnv1a64 = required_string(&source, "project_code_asset_source_fnv1a64")?;
    let lowering_target = required_string(&source, "lowering_target")?;
    let identity_target = required_string(&source, "project_code_asset_lowering_target")?;
    let entries = string_array_field(&source, "project_code_asset_entries")?;
    let projected_entries = projections
        .iter()
        .map(|projection| projection.entry.as_str())
        .collect::<Vec<_>>();
    let entry_count = integer_field(&source, "project_code_asset_entry_count")?;
    let function_count = integer_field(&source, "function_count")?;
    let identity_hash = required_string(&source, "project_code_asset_identity_hash")?;
    let expected_hash =
        project_code_asset_identity_hash(&source_fnv1a64, &lowering_target, &projected_entries);
    let asset_id = required_string(&source, "project_code_asset_id")?;
    let expected_id = format!("kernel.cuda.project.{}", &expected_hash[2..]);
    let identity_set_asset_ids =
        string_array_field(&source, "project_code_asset_identity_set_asset_ids")?;
    let identity_set_contracts =
        string_array_field(&source, "project_code_asset_identity_set_contracts")?;
    let identity_set_hashes =
        string_array_field(&source, "project_code_asset_identity_set_hashes")?;
    let identity_set_root_hash =
        required_string(&source, "project_code_asset_identity_set_root_hash")?;
    let expected_set_root_hash = code_asset_identity_set_root_hash(&[(
        &asset_id,
        PROJECT_CODE_ASSET_IDENTITY_CONTRACT,
        &identity_hash,
    )]);
    if string_field(&source, "project_code_asset_identity_contract").as_deref()
        != Some(PROJECT_CODE_ASSET_IDENTITY_CONTRACT)
        || string_field(&source, "project_code_asset_identity_set_contract").as_deref()
            != Some(CODE_ASSET_IDENTITY_SET_CONTRACT)
        || integer_field(&source, "project_code_asset_identity_set_count")? != 1
        || identity_set_asset_ids != [asset_id.as_str()]
        || identity_set_contracts != [PROJECT_CODE_ASSET_IDENTITY_CONTRACT]
        || identity_set_hashes != [identity_hash.as_str()]
        || identity_set_root_hash != expected_set_root_hash
        || !valid_fnv1a64(&source_fnv1a64)
        || identity_source_fnv1a64 != source_fnv1a64
        || lowering_target != "cuda.nvidia-gpu"
        || identity_target != lowering_target
        || entries != projected_entries
        || entry_count != projected_entries.len() as i64
        || function_count != entry_count
        || identity_hash != expected_hash
        || asset_id != expected_id
    {
        return Err("project Kernel code asset identity is inconsistent".to_owned());
    }
    Ok(ProjectKernelCodeAssetIdentity {
        asset_id,
        source_fnv1a64,
        lowering_target,
        entries,
        identity_hash,
        identity_set_root_hash,
    })
}

fn render_project_code_asset_identity(identity: &ProjectKernelCodeAssetIdentity) -> String {
    format!(
        "provider_code_asset_identity_contract={PROJECT_CODE_ASSET_IDENTITY_CONTRACT};provider_code_asset_identity_asset_id={};provider_code_asset_identity_source_fnv1a64={};provider_code_asset_identity_lowering_target={};provider_code_asset_identity_entry_count={};provider_code_asset_identity_entries={};provider_code_asset_identity_hash={};provider_code_asset_identity_set_contract={CODE_ASSET_IDENTITY_SET_CONTRACT};provider_code_asset_identity_set_count=1;provider_code_asset_identity_set_root_hash={};provider_code_asset_identity_item_0_asset_id={};provider_code_asset_identity_item_0_contract={PROJECT_CODE_ASSET_IDENTITY_CONTRACT};provider_code_asset_identity_item_0_hash={};provider_code_asset_identity_item_0_source_fnv1a64={};provider_code_asset_identity_item_0_lowering_target={};provider_code_asset_identity_item_0_entry_count={};provider_code_asset_identity_item_0_entries={}",
        identity.asset_id,
        identity.source_fnv1a64,
        identity.lowering_target,
        identity.entries.len(),
        identity.entries.join(","),
        identity.identity_hash,
        identity.identity_set_root_hash,
        identity.asset_id,
        identity.identity_hash,
        identity.source_fnv1a64,
        identity.lowering_target,
        identity.entries.len(),
        identity.entries.join(","),
    )
}

fn project_code_asset_identity_hash(
    source_fnv1a64: &str,
    lowering_target: &str,
    entries: &[&str],
) -> String {
    fnv1a64_hex(
        format!(
            "{PROJECT_CODE_ASSET_IDENTITY_CONTRACT}\n{source_fnv1a64}\n{lowering_target}\n{}\n{}",
            entries.len(),
            entries.join("\n")
        )
        .as_bytes(),
    )
}

fn code_asset_identity_set_root_hash(items: &[(&str, &str, &str)]) -> String {
    let ordered_items = items
        .iter()
        .map(|(asset_id, contract, identity_hash)| {
            format!("{asset_id}\n{contract}\n{identity_hash}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fnv1a64_hex(
        format!(
            "{CODE_ASSET_IDENTITY_SET_CONTRACT}\n{}\n{ordered_items}",
            items.len()
        )
        .as_bytes(),
    )
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
    project_asset_id: &str,
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
        project_asset_id,
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

fn render_result_projection(
    index: usize,
    result: &ProjectKernelResultProjection,
    producer: &ProjectKernelRequestProjection,
) -> String {
    let prefix = format!("provider_result_projection_{index}_");
    let expected = result.expected_i64.to_le_bytes();
    debug_assert_eq!(producer.output_shape, [1, 1]);
    debug_assert_eq!(producer.expected_values, [result.expected_i64]);
    format!(
        "{prefix}contract={PROVIDER_RESULT_PROJECTION_CONTRACT};{prefix}source_function={};{prefix}source_node={};{prefix}value_type=i64;{prefix}producer_request_id={};{prefix}producer_output_buffer={};{prefix}byte_offset=0;{prefix}byte_length=8;{prefix}expected_i64={};{prefix}expected_content_hash={};{prefix}completion_requirement=nuis-provider-completion-evidence-v1;{prefix}glm_release_requirement=nuis-provider-glm-release-evidence-v1",
        result.source_function,
        result.source_node,
        producer.request_id(),
        producer.output_buffer(),
        result.expected_i64,
        fnv1a64_hex(&expected)
    )
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

fn integer_field(source: &str, key: &str) -> Result<i64, String> {
    optional_integer_field(source, key)?
        .ok_or_else(|| format!("project Kernel request projection is missing `{key}`"))
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

fn string_array_field(source: &str, key: &str) -> Result<Vec<String>, String> {
    array_field(source, key)?
        .into_iter()
        .map(|value| {
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .map(str::to_owned)
                .ok_or_else(|| format!("invalid `{key}` item"))
        })
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

fn valid_fnv1a64(value: &str) -> bool {
    value.len() == 18
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}
