use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(crate) const FILTER_PLAN_CONTRACT: &str = "nuis-pixelmagic-filter-plan-v1";
pub(crate) const FILTER_PLAN_CATALOG_CONTRACT: &str = "nuis-pixelmagic-filter-plan-catalog-v1";
const PIXELMAGIC_MODULE_MANIFEST: &str = include_str!("../../../stdlib/pixelmagic/module.toml");
const DEFAULT_FILTER_PLAN_SOURCE: &str =
    include_str!("../../../stdlib/pixelmagic/provider-plans/gray8-invert-threshold.nspf");
const THRESHOLD_FILTER_PLAN_SOURCE: &str =
    include_str!("../../../stdlib/pixelmagic/provider-plans/gray8-threshold.nspf");
const FILTER_PLAN_SOURCES: &[(&str, &str)] = &[
    (
        "provider-plans/gray8-invert-threshold.nspf",
        DEFAULT_FILTER_PLAN_SOURCE,
    ),
    (
        "provider-plans/gray8-threshold.nspf",
        THRESHOLD_FILTER_PLAN_SOURCE,
    ),
];

#[derive(Clone, Debug)]
pub(crate) struct PixelMagicFilterPlan {
    pub(crate) package_id: String,
    pub(crate) plan_id: String,
    pub(crate) backend_family: String,
    pub(crate) target_device: String,
    buffer: PixelMagicFilterBuffer,
    stages: Vec<PixelMagicFilterStage>,
    source_hash: String,
    source_path: String,
    catalog_count: usize,
    catalog_hash: String,
    catalog_default_plan_id: String,
    catalog_selection_status: String,
    artifact_requested_plan_id: String,
}

#[derive(Clone, Debug)]
struct PixelMagicFilterBuffer {
    id: String,
    element_type: String,
    layout: String,
    shape: String,
    row_stride_bytes: usize,
    payload_path: String,
    payload: Vec<u8>,
}

#[derive(Clone, Debug)]
struct PixelMagicFilterStage {
    request_id: String,
    operation: String,
    scalar_bindings: String,
    output_buffer: String,
    expected_path: String,
    expected: Vec<u8>,
    input_source: String,
    producer_request_id: String,
    producer_output_buffer: String,
}

pub(crate) fn load_filter_plan() -> Result<PixelMagicFilterPlan, String> {
    let catalog = PixelMagicFilterPlanCatalog::parse(PIXELMAGIC_MODULE_MANIFEST)?;
    catalog.select_default()
}

pub(crate) fn load_filter_plan_for_artifact_metadata(
    evidence: &str,
) -> Result<PixelMagicFilterPlan, String> {
    let catalog = PixelMagicFilterPlanCatalog::parse(PIXELMAGIC_MODULE_MANIFEST)?;
    match requested_plan_id(evidence)? {
        Some(plan_id) => catalog.select_requested(plan_id),
        None => catalog.select_default(),
    }
}

#[cfg(test)]
fn load_filter_plan_by_id(plan_id: &str) -> Result<PixelMagicFilterPlan, String> {
    PixelMagicFilterPlanCatalog::parse(PIXELMAGIC_MODULE_MANIFEST)?.select_requested(plan_id)
}

struct PixelMagicFilterPlanCatalog {
    default_plan_id: String,
    plans: Vec<PixelMagicFilterPlan>,
}

impl PixelMagicFilterPlanCatalog {
    fn parse(manifest: &str) -> Result<Self, String> {
        let contract = manifest_string_value(manifest, "provider_plan_catalog_contract")?;
        if contract != FILTER_PLAN_CATALOG_CONTRACT {
            return Err(format!("unsupported filter plan catalog `{contract}`"));
        }
        let default_plan_id = manifest_string_value(manifest, "provider_plan_default")?;
        let paths = manifest_string_array(manifest, "provider_plans")?;
        if paths.is_empty() || paths.len() > 64 {
            return Err("provider_plans must contain between 1 and 64 paths".to_owned());
        }
        if paths.len() != FILTER_PLAN_SOURCES.len() {
            return Err("provider plan catalog does not cover the AOT source table".to_owned());
        }
        let mut seen_paths = BTreeSet::new();
        let mut seen_ids = BTreeSet::new();
        let mut plans = Vec::with_capacity(paths.len());
        for path in &paths {
            if !seen_paths.insert(path.clone()) {
                return Err(format!("duplicate provider plan path `{path}`"));
            }
            let source = FILTER_PLAN_SOURCES
                .iter()
                .find_map(|(registered_path, source)| (*registered_path == path).then_some(*source))
                .ok_or_else(|| format!("provider plan `{path}` is not in the AOT source table"))?;
            let mut plan = PixelMagicFilterPlan::parse(source)?;
            if plan.package_id != "nuis.pixelmagic" {
                return Err(format!(
                    "provider plan `{path}` belongs to `{}`",
                    plan.package_id
                ));
            }
            if !seen_ids.insert(plan.plan_id.clone()) {
                return Err(format!("duplicate provider plan id `{}`", plan.plan_id));
            }
            plan.source_path = path.clone();
            plans.push(plan);
        }
        if !seen_ids.contains(default_plan_id.as_str()) {
            return Err(format!(
                "default provider plan `{default_plan_id}` is not declared"
            ));
        }
        let catalog_hash = filter_plan_catalog_hash(&default_plan_id, &plans);
        let catalog_count = plans.len();
        for plan in &mut plans {
            plan.catalog_count = catalog_count;
            plan.catalog_hash.clone_from(&catalog_hash);
            plan.catalog_default_plan_id.clone_from(&default_plan_id);
        }
        Ok(Self {
            default_plan_id,
            plans,
        })
    }

    fn select_default(&self) -> Result<PixelMagicFilterPlan, String> {
        let default_plan_id = self.default_plan_id.clone();
        let mut plan = self.select(&default_plan_id)?;
        plan.catalog_selection_status = "default-selected".to_owned();
        plan.artifact_requested_plan_id = "none".to_owned();
        Ok(plan)
    }

    fn select_requested(&self, plan_id: &str) -> Result<PixelMagicFilterPlan, String> {
        let mut plan = self.select(plan_id)?;
        plan.catalog_selection_status = "artifact-request-selected".to_owned();
        plan.artifact_requested_plan_id = plan_id.to_owned();
        Ok(plan)
    }

    fn select(&self, plan_id: &str) -> Result<PixelMagicFilterPlan, String> {
        self.plans
            .iter()
            .find(|plan| plan.plan_id == plan_id)
            .cloned()
            .ok_or_else(|| format!("provider plan `{plan_id}` is not declared"))
    }
}

impl PixelMagicFilterPlan {
    fn parse(source: &str) -> Result<Self, String> {
        let fields = parse_fields(source)?;
        let protocol = required(&fields, "protocol")?;
        if protocol != FILTER_PLAN_CONTRACT {
            return Err(format!("unsupported protocol `{protocol}`"));
        }
        let package_id = required(&fields, "package_id")?.to_owned();
        let plan_id = required(&fields, "plan_id")?.to_owned();
        let backend_family = required(&fields, "backend_family")?.to_owned();
        let target_device = required(&fields, "target_device")?.to_owned();
        let buffer = PixelMagicFilterBuffer {
            id: required(&fields, "buffer.id")?.to_owned(),
            element_type: required(&fields, "buffer.element_type")?.to_owned(),
            layout: required(&fields, "buffer.layout")?.to_owned(),
            shape: required(&fields, "buffer.shape")?.to_owned(),
            row_stride_bytes: parse_usize(&fields, "buffer.row_stride_bytes")?,
            payload_path: required(&fields, "buffer.payload_path")?.to_owned(),
            payload: parse_bytes(required(&fields, "buffer.payload_bytes")?)?,
        };
        validate_token("package_id", &package_id)?;
        validate_token("plan_id", &plan_id)?;
        validate_token("backend_family", &backend_family)?;
        validate_token("target_device", &target_device)?;
        validate_buffer(&buffer)?;

        let stage_keys = parse_list(required(&fields, "stage_order")?)?;
        if stage_keys.is_empty() || stage_keys.len() > 64 {
            return Err("stage_order must contain between 1 and 64 stages".to_owned());
        }
        let mut stages = Vec::with_capacity(stage_keys.len());
        for stage_key in &stage_keys {
            validate_token("stage key", stage_key)?;
            let prefix = format!("stage.{stage_key}.");
            let stage = PixelMagicFilterStage {
                request_id: required(&fields, &format!("{prefix}request_id"))?.to_owned(),
                operation: required(&fields, &format!("{prefix}operation"))?.to_owned(),
                scalar_bindings: required(&fields, &format!("{prefix}scalar_bindings"))?.to_owned(),
                output_buffer: required(&fields, &format!("{prefix}output_buffer"))?.to_owned(),
                expected_path: required(&fields, &format!("{prefix}expected_path"))?.to_owned(),
                expected: parse_bytes(required(&fields, &format!("{prefix}expected_bytes"))?)?,
                input_source: required(&fields, &format!("{prefix}input_source"))?.to_owned(),
                producer_request_id: required(&fields, &format!("{prefix}producer_request_id"))?
                    .to_owned(),
                producer_output_buffer: required(
                    &fields,
                    &format!("{prefix}producer_output_buffer"),
                )?
                .to_owned(),
            };
            validate_stage(&stage, &buffer, &stages)?;
            stages.push(stage);
        }
        validate_unique_stages(&stages)?;
        let catalog_default_plan_id = plan_id.clone();

        Ok(Self {
            package_id,
            plan_id,
            backend_family,
            target_device,
            buffer,
            stages,
            source_hash: fnv1a64_hex(source.as_bytes()),
            source_path: "inline".to_owned(),
            catalog_count: 1,
            catalog_hash: fnv1a64_hex(source.as_bytes()),
            catalog_default_plan_id,
            catalog_selection_status: "default-selected".to_owned(),
            artifact_requested_plan_id: "none".to_owned(),
        })
    }

    pub(crate) fn supports(&self, backend_family: &str, target_device: &str) -> bool {
        self.backend_family == backend_family && self.target_device == target_device
    }

    pub(crate) fn render_evidence(&self) -> String {
        let first = &self.stages[0];
        let compatibility = format!(
            "provider_filter_plan_catalog_contract={FILTER_PLAN_CATALOG_CONTRACT};provider_filter_plan_catalog_count={};provider_filter_plan_catalog_hash={};provider_filter_plan_catalog_default_id={};provider_filter_plan_catalog_selected_path={};provider_filter_plan_catalog_selection_status={};provider_filter_plan_artifact_request_id={};provider_filter_plan_contract={FILTER_PLAN_CONTRACT};provider_filter_plan_package={};provider_filter_plan_id={};provider_filter_plan_hash={};provider_filter_plan_validation_status=verified;provider_filter_plan_stage_count={};provider_filter_plan_stage_order={};provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;provider_buffer_id={};provider_buffer_element_type={};provider_buffer_layout={};provider_buffer_shape={};provider_buffer_row_stride_bytes={};provider_buffer_byte_length={};provider_buffer_payload_path={};provider_buffer_content_hash={};provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;provider_kernel_id={};provider_kernel_operation={};provider_kernel_input_buffer={};provider_kernel_output_buffer={};provider_kernel_dispatch={}x1;provider_kernel_scalar_bindings={};std-preprocessed-pgm:input_bytes=20;pixel_format=gray8;pixel_width=2;pixel_height=2;pixel_stride={};pixel_max_value=15;pixel_operation={};pixel_payload_path={};pixel_payload_bytes={};pixel_payload_hash={}",
            self.catalog_count,
            self.catalog_hash,
            self.catalog_default_plan_id,
            self.source_path,
            self.catalog_selection_status,
            self.artifact_requested_plan_id,
            self.package_id,
            self.plan_id,
            self.source_hash,
            self.stages.len(),
            self.stages
                .iter()
                .map(|stage| stage.request_id.as_str())
                .collect::<Vec<_>>()
                .join(","),
            self.buffer.id,
            self.buffer.element_type,
            self.buffer.layout,
            self.buffer.shape,
            self.buffer.row_stride_bytes,
            self.buffer.payload.len(),
            self.buffer.payload_path,
            fnv1a64_hex(&self.buffer.payload),
            first.request_id,
            first.operation,
            self.buffer.id,
            first.output_buffer,
            self.buffer.shape,
            first.scalar_bindings,
            self.buffer.row_stride_bytes,
            first.operation,
            self.buffer.payload_path,
            self.buffer.payload.len(),
            fnv1a64_hex(&self.buffer.payload),
        );
        let requests = self
            .stages
            .iter()
            .enumerate()
            .map(|(index, stage)| self.render_request(index, stage))
            .collect::<Vec<_>>()
            .join(";");
        format!(
            "{compatibility};provider_request_collection_contract=nuis-provider-request-collection-v1;provider_request_count={};{requests}",
            self.stages.len()
        )
    }

    fn render_request(&self, index: usize, stage: &PixelMagicFilterStage) -> String {
        let prefix = format!("provider_request_{index}_");
        let request = format!(
            "{prefix}buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1;{prefix}buffer_id={};{prefix}buffer_element_type={};{prefix}buffer_layout={};{prefix}buffer_shape={};{prefix}buffer_row_stride_bytes={};{prefix}buffer_byte_length={};{prefix}buffer_payload_path={};{prefix}buffer_content_hash={};{prefix}kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1;{prefix}kernel_id={};{prefix}kernel_operation={};{prefix}kernel_input_buffer={};{prefix}kernel_output_buffer={};{prefix}kernel_dispatch={}x1;{prefix}kernel_scalar_bindings={};{prefix}output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1;{prefix}output_comparison_output_buffer={};{prefix}output_comparison_element_type={};{prefix}output_comparison_shape={};{prefix}output_comparison_expected_path={};{prefix}output_comparison_expected_byte_length={};{prefix}output_comparison_expected_content_hash={};{prefix}output_comparison_absolute_tolerance=0;{prefix}output_comparison_relative_tolerance=0;{prefix}output_comparison_non_finite_policy=reject",
            self.buffer.id,
            self.buffer.element_type,
            self.buffer.layout,
            self.buffer.shape,
            self.buffer.row_stride_bytes,
            self.buffer.payload.len(),
            self.buffer.payload_path,
            fnv1a64_hex(&self.buffer.payload),
            stage.request_id,
            stage.operation,
            self.buffer.id,
            stage.output_buffer,
            self.buffer.shape,
            stage.scalar_bindings,
            stage.output_buffer,
            self.buffer.element_type,
            self.buffer.shape,
            stage.expected_path,
            stage.expected.len(),
            fnv1a64_hex(&stage.expected),
        );
        format!("{request}{}", self.render_input_binding(index, stage))
    }

    fn render_input_binding(&self, index: usize, stage: &PixelMagicFilterStage) -> String {
        let prefix = format!(";provider_request_{index}_");
        if stage.input_source == "artifact" {
            return format!(
                "{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name={};{prefix}input_binding_0_source=artifact;{prefix}input_binding_0_element_type={};{prefix}input_binding_0_shape={};{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path={};{prefix}input_binding_0_producer_request_id=none;{prefix}input_binding_0_producer_output_buffer=none",
                self.buffer.id,
                self.buffer.element_type,
                self.buffer.shape,
                self.buffer.payload.len(),
                fnv1a64_hex(&self.buffer.payload),
                self.buffer.payload_path,
            );
        }
        let producer_index = self
            .stages
            .iter()
            .position(|candidate| candidate.request_id == stage.producer_request_id)
            .expect("validated dependency producer");
        let producer = &self.stages[producer_index];
        format!(
            "{prefix}dependency_contract=nuis-provider-request-dependency-v1;{prefix}dependency_count=1;{prefix}dependency_0_producer_request_id={};{prefix}dependency_0_producer_output_buffer={};{prefix}dependency_0_consumer_input_buffer={};{prefix}dependency_0_transport_contract=nuis-provider-edge-transport-v1;{prefix}dependency_0_transport_ownership_token=glm:provider-edge:{}:{}->{}:{};{prefix}dependency_0_transport_staging_mode=auto;{prefix}dependency_0_transport_producer_clock_evidence=provider-clock:request-{producer_index}:completed;{prefix}dependency_0_transport_consumer_clock_evidence=provider-clock:request-{index}:dispatch-ready;{prefix}input_binding_contract=nuis-provider-input-binding-v1;{prefix}input_binding_count=1;{prefix}input_binding_0_name={};{prefix}input_binding_0_source=dependency;{prefix}input_binding_0_element_type={};{prefix}input_binding_0_shape={};{prefix}input_binding_0_byte_length={};{prefix}input_binding_0_content_hash={};{prefix}input_binding_0_payload_path=none;{prefix}input_binding_0_producer_request_id={};{prefix}input_binding_0_producer_output_buffer={}",
            stage.producer_request_id,
            stage.producer_output_buffer,
            self.buffer.id,
            stage.producer_request_id,
            stage.producer_output_buffer,
            stage.request_id,
            self.buffer.id,
            self.buffer.id,
            self.buffer.element_type,
            self.buffer.shape,
            producer.expected.len(),
            fnv1a64_hex(&producer.expected),
            stage.producer_request_id,
            stage.producer_output_buffer,
        )
    }

    pub(crate) fn persist_payloads(&self, output_dir: &Path) -> Result<(), String> {
        persist_exact(
            &output_dir.join(&self.buffer.payload_path),
            &self.buffer.payload,
            "filter input",
        )?;
        for stage in &self.stages {
            persist_exact(
                &output_dir.join(&stage.expected_path),
                &stage.expected,
                "expected output",
            )?;
        }
        Ok(())
    }

    pub(crate) fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub(crate) fn catalog_hash(&self) -> &str {
        &self.catalog_hash
    }
}

fn persist_exact(path: &Path, bytes: &[u8], role: &str) -> Result<(), String> {
    if path.exists() {
        let existing = fs::read(path).map_err(|error| {
            format!(
                "failed to read PixelMagic {role} `{}`: {error}",
                path.display()
            )
        })?;
        if existing != bytes {
            return Err(format!(
                "PixelMagic {role} path `{}` has conflicting content",
                path.display()
            ));
        }
        return Ok(());
    }
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to persist PixelMagic {role} `{}`: {error}",
            path.display()
        )
    })
}

fn requested_plan_id(evidence: &str) -> Result<Option<&str>, String> {
    let mut requested = None;
    for field in evidence.split(';') {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        if !key.starts_with("artifact_provider_metadata_")
            || key == "artifact_provider_metadata_contract"
            || key == "artifact_provider_metadata_count"
        {
            continue;
        }
        let Some((package_id, request)) = value.split_once(':') else {
            continue;
        };
        if package_id != "nuis.pixelmagic" {
            continue;
        }
        let (request_key, plan_id) = request
            .split_once('=')
            .ok_or_else(|| format!("malformed PixelMagic artifact metadata `{value}`"))?;
        if request_key != "filter-plan" {
            return Err(format!(
                "unsupported PixelMagic artifact metadata key `{request_key}`"
            ));
        }
        validate_token("artifact requested plan id", plan_id)?;
        if requested.replace(plan_id).is_some() {
            return Err("duplicate PixelMagic filter-plan artifact request".to_owned());
        }
    }
    Ok(requested)
}

fn manifest_string_value(source: &str, key: &str) -> Result<String, String> {
    let prefix = format!("{key} = ");
    source
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|value| value.strip_prefix('"')?.strip_suffix('"'))
        .map(str::to_owned)
        .ok_or_else(|| format!("module manifest is missing string `{key}`"))
}

fn manifest_string_array(source: &str, key: &str) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = [");
    let mut collecting = false;
    let mut values = Vec::new();
    for raw in source.lines() {
        let line = raw.trim();
        if !collecting {
            if line == prefix {
                collecting = true;
            }
            continue;
        }
        if line == "]" {
            return Ok(values);
        }
        let value = line
            .strip_suffix(',')
            .unwrap_or(line)
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .ok_or_else(|| format!("module manifest `{key}` contains malformed item `{line}`"))?;
        validate_evidence_value(value)?;
        values.push(value.to_owned());
    }
    Err(format!("module manifest is missing array `{key}`"))
}

fn filter_plan_catalog_hash(default_plan_id: &str, plans: &[PixelMagicFilterPlan]) -> String {
    let mut canonical = format!(
        "contract={FILTER_PLAN_CATALOG_CONTRACT}\ndefault={default_plan_id}\ncount={}\n",
        plans.len()
    );
    for plan in plans {
        canonical.push_str(&format!(
            "path={};id={};hash={}\n",
            plan.source_path, plan.plan_id, plan.source_hash
        ));
    }
    fnv1a64_hex(canonical.as_bytes())
}

fn parse_fields(source: &str) -> Result<BTreeMap<String, String>, String> {
    let mut fields = BTreeMap::new();
    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {} is missing `=`", index + 1))?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(format!("line {} has an empty key or value", index + 1));
        }
        validate_token("field", key)?;
        validate_evidence_value(value)?;
        if fields.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate field `{key}`"));
        }
    }
    Ok(fields)
}

fn required<'a>(fields: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str, String> {
    fields
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| format!("missing field `{key}`"))
}

fn parse_usize(fields: &BTreeMap<String, String>, key: &str) -> Result<usize, String> {
    required(fields, key)?
        .parse()
        .map_err(|_| format!("field `{key}` is not an unsigned integer"))
}

fn parse_list(value: &str) -> Result<Vec<String>, String> {
    value
        .split(',')
        .map(|item| {
            let item = item.trim();
            if item.is_empty() {
                Err("list contains an empty item".to_owned())
            } else {
                Ok(item.to_owned())
            }
        })
        .collect()
}

fn parse_bytes(value: &str) -> Result<Vec<u8>, String> {
    parse_list(value)?
        .into_iter()
        .map(|item| {
            item.parse::<u8>()
                .map_err(|_| format!("byte `{item}` is outside u8"))
        })
        .collect()
}

fn validate_buffer(buffer: &PixelMagicFilterBuffer) -> Result<(), String> {
    for (name, value) in [
        ("buffer.id", buffer.id.as_str()),
        ("buffer.element_type", buffer.element_type.as_str()),
        ("buffer.layout", buffer.layout.as_str()),
        ("buffer.shape", buffer.shape.as_str()),
        ("buffer.payload_path", buffer.payload_path.as_str()),
    ] {
        validate_evidence_value(value).map_err(|error| format!("{name}: {error}"))?;
    }
    if buffer.payload.is_empty() {
        return Err("buffer.payload_bytes must not be empty".to_owned());
    }
    if buffer.row_stride_bytes == 0 {
        return Err("buffer.row_stride_bytes must be positive".to_owned());
    }
    Ok(())
}

fn validate_stage(
    stage: &PixelMagicFilterStage,
    buffer: &PixelMagicFilterBuffer,
    preceding: &[PixelMagicFilterStage],
) -> Result<(), String> {
    for (name, value) in [
        ("request_id", stage.request_id.as_str()),
        ("operation", stage.operation.as_str()),
        ("scalar_bindings", stage.scalar_bindings.as_str()),
        ("output_buffer", stage.output_buffer.as_str()),
        ("expected_path", stage.expected_path.as_str()),
    ] {
        validate_evidence_value(value).map_err(|error| format!("stage {name}: {error}"))?;
    }
    if stage.expected.len() != buffer.payload.len() {
        return Err(format!(
            "stage `{}` expected {} bytes but input has {}",
            stage.request_id,
            stage.expected.len(),
            buffer.payload.len()
        ));
    }
    match stage.input_source.as_str() {
        "artifact" => {
            if stage.producer_request_id != "none" || stage.producer_output_buffer != "none" {
                return Err(format!(
                    "artifact stage `{}` must not declare a producer",
                    stage.request_id
                ));
            }
        }
        "dependency" => {
            let producer = preceding
                .iter()
                .find(|candidate| candidate.request_id == stage.producer_request_id)
                .ok_or_else(|| {
                    format!(
                        "dependency stage `{}` references a non-preceding producer `{}`",
                        stage.request_id, stage.producer_request_id
                    )
                })?;
            if producer.output_buffer != stage.producer_output_buffer {
                return Err(format!(
                    "dependency stage `{}` producer output `{}` does not match `{}`",
                    stage.request_id, stage.producer_output_buffer, producer.output_buffer
                ));
            }
        }
        other => {
            return Err(format!(
                "stage `{}` has unsupported input source `{other}`",
                stage.request_id
            ));
        }
    }
    Ok(())
}

fn validate_unique_stages(stages: &[PixelMagicFilterStage]) -> Result<(), String> {
    let mut request_ids = BTreeSet::new();
    let mut output_buffers = BTreeSet::new();
    for stage in stages {
        if !request_ids.insert(stage.request_id.as_str()) {
            return Err(format!("duplicate request id `{}`", stage.request_id));
        }
        if !output_buffers.insert(stage.output_buffer.as_str()) {
            return Err(format!("duplicate output buffer `{}`", stage.output_buffer));
        }
    }
    Ok(())
}

fn validate_token(name: &str, value: &str) -> Result<(), String> {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
    {
        Ok(())
    } else {
        Err(format!(
            "{name} `{value}` contains an unsupported character"
        ))
    }
}

fn validate_evidence_value(value: &str) -> Result<(), String> {
    if value.contains([';', '\n', '\r']) {
        Err(format!(
            "value `{value}` cannot be embedded in provider evidence"
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_plan_validates_ordered_dependency_and_payloads() {
        let plan = load_filter_plan().expect("checked-in PixelMagic filter plan");
        assert_eq!(plan.package_id, "nuis.pixelmagic");
        assert_eq!(plan.stages.len(), 2);
        assert_eq!(plan.stages[0].request_id, "pixelmagic.gray8.invert");
        assert_eq!(plan.stages[1].input_source, "dependency");
        assert_eq!(
            plan.stages[1].producer_request_id,
            "pixelmagic.gray8.invert"
        );
        assert_eq!(fnv1a64_hex(&plan.stages[1].expected), "0xfc6f93a90d12d41b");
        assert_eq!(plan.catalog_count, 2);
        assert_eq!(
            plan.catalog_default_plan_id,
            "pixelmagic.gray8.invert-threshold"
        );
        assert_eq!(
            plan.source_path,
            "provider-plans/gray8-invert-threshold.nspf"
        );
    }

    #[test]
    fn package_catalog_selects_second_declared_plan() {
        let plan =
            load_filter_plan_by_id("pixelmagic.gray8.threshold-only").expect("threshold plan");
        assert_eq!(plan.stages.len(), 1);
        assert_eq!(plan.stages[0].operation, "threshold");
        assert_eq!(plan.stages[0].input_source, "artifact");
        assert_eq!(plan.stages[0].expected, [0, 0, 15, 15]);
        assert_eq!(plan.catalog_count, 2);
        assert_eq!(plan.source_path, "provider-plans/gray8-threshold.nspf");
        assert_eq!(plan.catalog_selection_status, "artifact-request-selected");
        assert_eq!(
            plan.artifact_requested_plan_id,
            "pixelmagic.gray8.threshold-only"
        );
    }

    #[test]
    fn artifact_metadata_selects_only_declared_pixelmagic_plan() {
        let evidence = "artifact_provider_metadata_contract=nuis-artifact-provider-metadata-v1;artifact_provider_metadata_count=2;artifact_provider_metadata_0=nuis.other:key=value;artifact_provider_metadata_1=nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only";
        let plan = load_filter_plan_for_artifact_metadata(evidence).expect("requested plan");

        assert_eq!(plan.plan_id, "pixelmagic.gray8.threshold-only");
        assert_eq!(plan.stages.len(), 1);
        assert_eq!(plan.catalog_selection_status, "artifact-request-selected");
    }

    #[test]
    fn artifact_metadata_rejects_undeclared_and_duplicate_plan_requests() {
        let undeclared =
            "artifact_provider_metadata_0=nuis.pixelmagic:filter-plan=pixelmagic.gray8.missing";
        assert!(load_filter_plan_for_artifact_metadata(undeclared)
            .unwrap_err()
            .contains("is not declared"));

        let duplicate = "artifact_provider_metadata_0=nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only;artifact_provider_metadata_1=nuis.pixelmagic:filter-plan=pixelmagic.gray8.invert-threshold";
        assert!(load_filter_plan_for_artifact_metadata(duplicate)
            .unwrap_err()
            .contains("duplicate PixelMagic filter-plan"));
    }

    #[test]
    fn package_plan_rejects_forward_dependency() {
        let invalid = DEFAULT_FILTER_PLAN_SOURCE.replace(
            "stage_order=invert,threshold",
            "stage_order=threshold,invert",
        );
        let error = PixelMagicFilterPlan::parse(&invalid).unwrap_err();
        assert!(error.contains("non-preceding producer"));
    }
}
