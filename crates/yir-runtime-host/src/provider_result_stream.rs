use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Component, Path},
    sync::{Arc, Mutex},
};

use yir_core::{
    ExecutionState, FrameSurface, InstructionSemantics, Node, ProviderCompletionRegistration,
    ProviderPhysicalCompletion, RegisteredMod, Resource, Value,
};

pub const PROVIDER_RESULT_STREAM_CONTRACT: &str = "nuis-provider-runtime-result-stream-v1";
pub const PROVIDER_RESULT_STREAM_ENV: &str = "NUIS_YIR_PROVIDER_RESULT_STREAM";
const MAX_RUNTIME_RESULTS: usize = 256;

pub(super) fn execute_with_provider_result_stream(
    module_source: &str,
    manifest_path: &Path,
) -> Result<yir_exec::ExecutionTrace, String> {
    let stream = ProviderResultStream::load(manifest_path)?;
    if fnv1a64_hex(module_source.as_bytes()) != stream.source_yir_fnv1a64 {
        return Err("provider runtime result stream belongs to a different YIR module".to_owned());
    }
    let state = Arc::new(Mutex::new(ProviderResultQueue::new(stream.frames)?));
    let mut registry = yir_verify::default_registry();
    registry.register(ProviderResultShaderMod {
        state: Arc::clone(&state),
    });
    let trace = super::execute_module_source_with_registry(module_source, &registry)?;
    state
        .lock()
        .map_err(|_| "provider runtime result queue lock was poisoned".to_owned())?
        .ensure_consumed()?;
    Ok(trace)
}

struct ProviderResultStream {
    source_yir_fnv1a64: String,
    frames: Vec<ProviderResultFrame>,
}

struct ProviderResultFrame {
    request_id: String,
    provider_family: String,
    module: String,
    instruction: String,
    node: String,
    resource: String,
    element_type: String,
    layout: String,
    shape: Vec<usize>,
    row_stride_bytes: usize,
    payload_path: String,
    payload_hash: String,
    payload: Vec<u8>,
    completion_wire: String,
    completion: ProviderPhysicalCompletion,
}

impl ProviderResultStream {
    fn load(manifest_path: &Path) -> Result<Self, String> {
        let source = fs::read_to_string(manifest_path).map_err(|error| {
            format!(
                "failed to read provider runtime result stream `{}`: {error}",
                manifest_path.display()
            )
        })?;
        let (header, frame_fields) = parse_sections(&source)?;
        require(&header, "schema", PROVIDER_RESULT_STREAM_CONTRACT)?;
        let source_yir_fnv1a64 = string_field(&header, "source_yir_fnv1a64")?;
        let frame_count = usize_field(&header, "frame_count")?;
        let claimed_stream_hash = string_field(&header, "stream_hash")?;
        if !(1..=MAX_RUNTIME_RESULTS).contains(&frame_count)
            || frame_count != frame_fields.len()
            || !valid_hash(&source_yir_fnv1a64)
            || !valid_hash(&claimed_stream_hash)
        {
            return Err("provider runtime result stream header is invalid".to_owned());
        }
        let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        let frames = frame_fields
            .iter()
            .enumerate()
            .map(|(index, fields)| parse_frame(root, index, fields))
            .collect::<Result<Vec<_>, _>>()?;
        let observed_stream_hash = stream_hash(&source_yir_fnv1a64, &frames);
        if observed_stream_hash != claimed_stream_hash {
            return Err("provider runtime result stream identity mismatch".to_owned());
        }
        Ok(Self {
            source_yir_fnv1a64,
            frames,
        })
    }
}

struct ProviderResultQueue {
    frames: VecDeque<ProviderResultFrame>,
    targets: BTreeSet<(String, String, String, String)>,
}

impl ProviderResultQueue {
    fn new(frames: Vec<ProviderResultFrame>) -> Result<Self, String> {
        if frames.iter().any(|frame| {
            frame.module != "shader"
                || frame.instruction != "draw_instanced"
                || frame.element_type != "u8"
                || frame.layout != "image-2d-row-major:pixel-format=rgba8"
                || frame.shape.len() != 2
                || frame.row_stride_bytes != frame.shape[0].saturating_mul(4)
        }) {
            return Err("no registered runtime result adapter accepts this stream".to_owned());
        }
        let targets = frames
            .iter()
            .map(|frame| {
                (
                    frame.module.clone(),
                    frame.instruction.clone(),
                    frame.node.clone(),
                    frame.resource.clone(),
                )
            })
            .collect();
        Ok(Self {
            frames: frames.into(),
            targets,
        })
    }

    fn targets(&self, node: &Node) -> bool {
        self.targets.contains(&(
            node.op.module.clone(),
            node.op.instruction.clone(),
            node.name.clone(),
            node.resource.clone(),
        ))
    }

    fn take(&mut self, node: &Node) -> Result<ProviderResultFrame, String> {
        let frame = self.frames.pop_front().ok_or_else(|| {
            format!(
                "provider runtime result stream is exhausted at `{}`",
                node.name
            )
        })?;
        if frame.module != node.op.module
            || frame.instruction != node.op.instruction
            || frame.node != node.name
            || frame.resource != node.resource
        {
            return Err(format!(
                "provider runtime result stream expected `{}.{}` node `{}` resource `{}`, got `{}.{}` node `{}` resource `{}`",
                frame.module,
                frame.instruction,
                frame.node,
                frame.resource,
                node.op.module,
                node.op.instruction,
                node.name,
                node.resource,
            ));
        }
        Ok(frame)
    }

    fn ensure_consumed(&self) -> Result<(), String> {
        self.frames.is_empty().then_some(()).ok_or_else(|| {
            format!(
                "provider runtime result stream retained {} unconsumed frame(s)",
                self.frames.len()
            )
        })
    }
}

struct ProviderResultShaderMod {
    state: Arc<Mutex<ProviderResultQueue>>,
}

impl RegisteredMod for ProviderResultShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn provider_completion_registration(
        &self,
        node: &Node,
    ) -> Option<ProviderCompletionRegistration> {
        let targets = self
            .state
            .lock()
            .map(|state| state.targets(node))
            .unwrap_or(true);
        if targets {
            yir_domain_shader::ShaderMod
                .provider_completion_registration(node)
                .map(|registration| {
                    ProviderCompletionRegistration::physical_fence_required(
                        registration.family,
                        registration.clock_domain,
                    )
                })
        } else {
            yir_domain_shader::ShaderMod.provider_completion_registration(node)
        }
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        yir_domain_shader::ShaderMod.describe(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        let value = yir_domain_shader::ShaderMod.execute(node, resource, state)?;
        let targets = self
            .state
            .lock()
            .map_err(|_| "provider runtime result queue lock was poisoned".to_owned())?
            .targets(node);
        if !targets {
            return Ok(value);
        }
        let reference = match value {
            Value::Frame(frame) => frame,
            other => {
                return Err(format!(
                    "provider runtime result target `{}` produced {other}, not frame",
                    node.name
                ))
            }
        };
        let frame = self
            .state
            .lock()
            .map_err(|_| "provider runtime result queue lock was poisoned".to_owned())?
            .take(node)?;
        if frame.shape != [reference.width, reference.height] {
            return Err(format!(
                "provider runtime result dimensions for `{}` disagree with YIR",
                node.name
            ));
        }
        state.stage_provider_physical_completion(node, frame.completion)?;
        FrameSurface::from_rgba8(reference.width, reference.height, frame.payload).map(Value::Frame)
    }
}

type Fields = BTreeMap<String, String>;

fn parse_sections(source: &str) -> Result<(Fields, Vec<Fields>), String> {
    let mut header = Fields::new();
    let mut frames = Vec::<Fields>::new();
    let mut frame_index = None;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[frame]]" {
            frames.push(Fields::new());
            frame_index = Some(frames.len() - 1);
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .map(|(key, value)| (key.trim(), value.trim()))
            .ok_or_else(|| format!("malformed provider runtime result line `{line}`"))?;
        let target = frame_index.map_or(&mut header, |index| &mut frames[index]);
        if key.is_empty() || target.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate provider runtime result field `{key}`"));
        }
    }
    Ok((header, frames))
}

fn parse_frame(
    root: &Path,
    expected_index: usize,
    fields: &Fields,
) -> Result<ProviderResultFrame, String> {
    if usize_field(fields, "index")? != expected_index {
        return Err("provider runtime result frame order is invalid".to_owned());
    }
    let payload_path = string_field(fields, "payload_path")?;
    if !relative_file_name(&payload_path) {
        return Err("provider runtime result payload path is not output-relative".to_owned());
    }
    let payload = fs::read(root.join(&payload_path)).map_err(|error| {
        format!("failed to read provider runtime result `{payload_path}`: {error}")
    })?;
    let payload_byte_length = usize_field(fields, "payload_byte_length")?;
    let payload_hash = string_field(fields, "payload_hash")?;
    if payload.len() != payload_byte_length || fnv1a64_hex(&payload) != payload_hash {
        return Err(format!(
            "provider runtime result payload `{payload_path}` identity mismatch"
        ));
    }
    let completion_wire = string_field(fields, "completion_wire")?;
    let completion = ProviderPhysicalCompletion::parse(&completion_wire)?;
    let frame = ProviderResultFrame {
        request_id: string_field(fields, "request_id")?,
        provider_family: string_field(fields, "provider_family")?,
        module: string_field(fields, "module")?,
        instruction: string_field(fields, "instruction")?,
        node: string_field(fields, "node")?,
        resource: string_field(fields, "resource")?,
        element_type: string_field(fields, "element_type")?,
        layout: string_field(fields, "layout")?,
        shape: parse_shape(&string_field(fields, "shape")?)?,
        row_stride_bytes: usize_field(fields, "row_stride_bytes")?,
        payload_path,
        payload_hash,
        payload,
        completion_wire,
        completion,
    };
    validate_frame(&frame)?;
    Ok(frame)
}

fn validate_frame(frame: &ProviderResultFrame) -> Result<(), String> {
    let strings = [
        frame.request_id.as_str(),
        frame.provider_family.as_str(),
        frame.module.as_str(),
        frame.instruction.as_str(),
        frame.node.as_str(),
        frame.resource.as_str(),
        frame.element_type.as_str(),
        frame.layout.as_str(),
    ];
    if strings
        .iter()
        .any(|value| value.is_empty() || value.len() > 256)
        || frame.shape.is_empty()
        || frame.shape.contains(&0)
        || frame.row_stride_bytes == 0
        || frame.payload.is_empty()
        || !valid_hash(&frame.payload_hash)
    {
        return Err("provider runtime result frame is invalid".to_owned());
    }
    Ok(())
}

fn stream_hash(source_yir_fnv1a64: &str, frames: &[ProviderResultFrame]) -> String {
    let mut material = format!(
        "{PROVIDER_RESULT_STREAM_CONTRACT}\n{source_yir_fnv1a64}\n{}",
        frames.len()
    );
    for (index, frame) in frames.iter().enumerate() {
        material.push_str(&format!(
            "\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            index,
            frame.request_id,
            frame.provider_family,
            frame.module,
            frame.instruction,
            frame.node,
            frame.resource,
            frame.element_type,
            frame.layout,
            render_shape(&frame.shape),
            frame.row_stride_bytes,
            frame.payload_path,
            frame.payload.len(),
            frame.payload_hash,
            frame.completion_wire,
        ));
    }
    fnv1a64_hex(material.as_bytes())
}

fn require(fields: &Fields, key: &str, expected: &str) -> Result<(), String> {
    (string_field(fields, key)?.as_str() == expected)
        .then_some(())
        .ok_or_else(|| format!("provider runtime result field `{key}` is incompatible"))
}

fn string_field(fields: &Fields, key: &str) -> Result<String, String> {
    let value = fields
        .get(key)
        .ok_or_else(|| format!("provider runtime result is missing `{key}`"))?;
    let quoted = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| format!("provider runtime result field `{key}` is not a string"))?;
    unescape_toml(quoted)
}

fn usize_field(fields: &Fields, key: &str) -> Result<usize, String> {
    fields
        .get(key)
        .ok_or_else(|| format!("provider runtime result is missing `{key}`"))?
        .parse()
        .map_err(|_| format!("provider runtime result field `{key}` is not an integer"))
}

fn parse_shape(value: &str) -> Result<Vec<usize>, String> {
    value
        .split('x')
        .map(|part| {
            part.parse::<usize>()
                .ok()
                .filter(|dimension| *dimension > 0)
                .ok_or_else(|| "provider runtime result shape is invalid".to_owned())
        })
        .collect()
}

fn render_shape(shape: &[usize]) -> String {
    shape
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join("x")
}

fn relative_file_name(value: &str) -> bool {
    let path = Path::new(value);
    !path.is_absolute()
        && path.components().count() == 1
        && matches!(path.components().next(), Some(Component::Normal(_)))
}

fn unescape_toml(value: &str) -> Result<String, String> {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            _ => return Err("provider runtime result string escape is invalid".to_owned()),
        }
    }
    Ok(out)
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("0x{hash:016x}")
}

fn valid_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 16 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
