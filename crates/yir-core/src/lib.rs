use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirModule {
    pub version: String,
    pub resources: Vec<Resource>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmValueClass {
    Val,
    Res,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmUseMode {
    Own,
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlmEffect {
    None,
    DomainMove,
    LifetimeEnd,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlmAccess {
    pub input: String,
    pub class: GlmValueClass,
    pub mode: GlmUseMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlmNodeProfile {
    pub result_class: GlmValueClass,
    pub accesses: Vec<GlmAccess>,
    pub effect: GlmEffect,
}

impl YirModule {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            resources: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub name: String,
    pub kind: ResourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKind {
    pub raw: String,
}

impl ResourceKind {
    pub fn parse(raw: &str) -> Self {
        Self {
            raw: raw.to_owned(),
        }
    }

    pub fn family(&self) -> &str {
        self.raw.split('.').next().unwrap_or(self.raw.as_str())
    }

    pub fn is_family(&self, expected: &str) -> bool {
        self.family() == expected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub resource: String,
    pub op: Operation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub module: String,
    pub instruction: String,
    pub args: Vec<String>,
}

impl Operation {
    pub fn parse(raw: &str, args: Vec<String>) -> Result<Self, String> {
        let (module, instruction) = raw
            .split_once('.')
            .ok_or_else(|| format!("operation `{raw}` must be written as <mod>.<instr>"))?;

        if module.is_empty() || instruction.is_empty() {
            return Err(format!(
                "operation `{raw}` must be written as <mod>.<instr>"
            ));
        }

        Ok(Self {
            module: module.to_owned(),
            instruction: instruction.to_owned(),
            args,
        })
    }

    pub fn full_name(&self) -> String {
        format!("{}.{}", self.module, self.instruction)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub kind: EdgeKind,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    Dep,
    Effect,
    Lifetime,
    CrossDomainExchange,
}

impl EdgeKind {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "dep" => Ok(Self::Dep),
            "effect" => Ok(Self::Effect),
            "lifetime" => Ok(Self::Lifetime),
            "xfer" => Ok(Self::CrossDomainExchange),
            other => Err(format!(
                "unknown edge kind `{other}`; expected dep|effect|lifetime|xfer"
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dep => "dep",
            Self::Effect => "effect",
            Self::Lifetime => "lifetime",
            Self::CrossDomainExchange => "xfer",
        }
    }
}

impl fmt::Display for GlmValueClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Val => f.write_str("val"),
            Self::Res => f.write_str("res"),
        }
    }
}

impl fmt::Display for GlmUseMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Own => f.write_str("Own"),
            Self::Read => f.write_str("Read"),
            Self::Write => f.write_str("Write"),
        }
    }
}

impl fmt::Display for GlmEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("none"),
            Self::DomainMove => f.write_str("domain-move"),
            Self::LifetimeEnd => f.write_str("lifetime-end"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    I32(i32),
    Int(i64),
    F32(f32),
    F64(f64),
    Symbol(String),
    Tensor(TensorValue),
    Pointer(Option<usize>),
    Tuple(Vec<Value>),
    Struct(StructValue),
    DataWindow(DataWindow),
    DataPipe(DataPipe),
    DataMarker(DataMarker),
    DataHandleTable(DataHandleTable),
    DataCoreBinding(DataCoreBinding),
    Target(SurfaceTarget),
    Viewport(Viewport),
    Pipeline(RenderPipeline),
    VertexLayout(VertexLayout),
    VertexBuffer(VertexBuffer),
    IndexBuffer(IndexBuffer),
    Texture(Texture2D),
    Sampler(SamplerState),
    Blend(BlendState),
    Depth(DepthState),
    Raster(RasterState),
    RenderState(RenderStateSet),
    Binding(ShaderBinding),
    BindingSet(ShaderBindingSet),
    RenderPass(RenderPass),
    Frame(FrameSurface),
    Unit,
}

pub fn glm_profile_for_operation(op: &Operation) -> GlmNodeProfile {
    match (op.module.as_str(), op.instruction.as_str()) {
        ("cpu", "alloc_node") | ("cpu", "alloc_buffer") => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: op
                .args
                .iter()
                .map(|input| GlmAccess {
                    input: input.clone(),
                    class: GlmValueClass::Val,
                    mode: GlmUseMode::Read,
                })
                .collect(),
            effect: GlmEffect::None,
        },
        ("cpu", "borrow") => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        ("cpu", "move_ptr") => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::DomainMove,
        },
        ("cpu", "load_value")
        | ("cpu", "load_next")
        | ("cpu", "buffer_len")
        | ("cpu", "load_at") => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        ("cpu", "store_value") | ("cpu", "store_next") | ("cpu", "store_at") => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Write,
            }],
            effect: GlmEffect::None,
        },
        ("cpu", "free") => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::LifetimeEnd,
        },
        ("data" | "fabric", "move") => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Val,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::DomainMove,
        },
        ("data" | "fabric", "copy_window") | ("data" | "fabric", "immutable_window") => {
            GlmNodeProfile {
                result_class: GlmValueClass::Res,
                accesses: vec![GlmAccess {
                    input: op.args[0].clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Read,
                }],
                effect: GlmEffect::None,
            }
        }
        ("data" | "fabric", "output_pipe") | ("data" | "fabric", "input_pipe") => {
            GlmNodeProfile {
                result_class: GlmValueClass::Res,
                accesses: vec![GlmAccess {
                    input: op.args[0].clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Read,
                }],
                effect: GlmEffect::None,
            }
        }
        _ => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: op
                .args
                .iter()
                .map(|input| GlmAccess {
                    input: input.clone(),
                    class: GlmValueClass::Val,
                    mode: GlmUseMode::Read,
                })
                .collect(),
            effect: GlmEffect::None,
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorValue {
    pub rows: usize,
    pub cols: usize,
    pub elements: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructValue {
    pub type_name: String,
    pub fields: Vec<(String, Value)>,
}

impl Eq for Value {}

impl Eq for StructValue {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataWindow {
    pub base: Box<Value>,
    pub offset: usize,
    pub len: usize,
    pub immutable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPipe {
    pub direction: DataPipeDirection,
    pub payload: Box<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataPipeDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataMarker {
    pub tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataHandleTable {
    pub entries: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataCoreBinding {
    pub core_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceTarget {
    pub format: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipeline {
    pub shading_model: String,
    pub topology: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexLayout {
    pub stride: usize,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexBuffer {
    pub vertex_count: usize,
    pub elements: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexBuffer {
    pub indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture2D {
    pub format: String,
    pub width: usize,
    pub height: usize,
    pub texels: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SamplerState {
    pub filter: String,
    pub address_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlendState {
    pub enabled: bool,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepthState {
    pub test_enabled: bool,
    pub write_enabled: bool,
    pub compare: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterState {
    pub cull_mode: String,
    pub front_face: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderStateSet {
    pub pipeline: RenderPipeline,
    pub blend: BlendState,
    pub depth: DepthState,
    pub raster: RasterState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderBinding {
    pub kind: String,
    pub slot: usize,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderBindingSet {
    pub pipeline: RenderPipeline,
    pub bindings: Vec<ShaderBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPass {
    pub target: SurfaceTarget,
    pub pipeline: RenderPipeline,
    pub viewport: Viewport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSurface {
    pub width: usize,
    pub height: usize,
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeapNode {
    pub value: i64,
    pub next: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeapBuffer {
    pub elements: Vec<i64>,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::I32(value) => write!(f, "{value}i32"),
            Self::Int(value) => write!(f, "{value}"),
            Self::F32(value) => write!(f, "{}f32", trim_float(*value as f64)),
            Self::F64(value) => write!(f, "{}f64", trim_float(*value)),
            Self::Symbol(value) => write!(f, "{value}"),
            Self::Tensor(tensor) => write!(f, "{tensor}"),
            Self::Pointer(pointer) => match pointer {
                Some(address) => write!(f, "&{address}"),
                None => write!(f, "null"),
            },
            Self::Tuple(values) => {
                write!(f, "(")?;
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Self::Struct(value) => write!(f, "{value}"),
            Self::DataWindow(window) => write!(f, "{window}"),
            Self::DataPipe(pipe) => write!(f, "{pipe}"),
            Self::DataMarker(marker) => write!(f, "{marker}"),
            Self::DataHandleTable(table) => write!(f, "{table}"),
            Self::DataCoreBinding(binding) => write!(f, "{binding}"),
            Self::Target(target) => write!(f, "{target}"),
            Self::Viewport(viewport) => write!(f, "{viewport}"),
            Self::Pipeline(pipeline) => write!(f, "{pipeline}"),
            Self::VertexLayout(layout) => write!(f, "{layout}"),
            Self::VertexBuffer(buffer) => write!(f, "{buffer}"),
            Self::IndexBuffer(buffer) => write!(f, "{buffer}"),
            Self::Texture(texture) => write!(f, "{texture}"),
            Self::Sampler(sampler) => write!(f, "{sampler}"),
            Self::Blend(blend) => write!(f, "{blend}"),
            Self::Depth(depth) => write!(f, "{depth}"),
            Self::Raster(raster) => write!(f, "{raster}"),
            Self::RenderState(render_state) => write!(f, "{render_state}"),
            Self::Binding(binding) => write!(f, "{binding}"),
            Self::BindingSet(binding_set) => write!(f, "{binding_set}"),
            Self::RenderPass(pass) => write!(f, "{pass}"),
            Self::Frame(frame) => write!(f, "{frame}"),
            Self::Unit => write!(f, "()"),
        }
    }
}

fn trim_float(value: f64) -> String {
    let mut rendered = value.to_string();
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.push('0');
        }
    }
    rendered
}

impl fmt::Display for TensorValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tensor[{}x{} ", self.rows, self.cols)?;
        for row in 0..self.rows {
            if row > 0 {
                write!(f, " | ")?;
            }
            for col in 0..self.cols {
                if col > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{}", self.elements[row * self.cols + col])?;
            }
        }
        write!(f, "]")
    }
}

impl fmt::Display for StructValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{{", self.type_name)?;
        for (index, (name, value)) in self.fields.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{name}: {value}")?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for DataWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mode = if self.immutable { "immutable" } else { "copy" };
        write!(
            f,
            "window[{mode} offset={} len={} base={}]",
            self.offset, self.len, self.base
        )
    }
}

impl fmt::Display for DataPipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pipe[{} {}]", self.direction, self.payload)
    }
}

impl fmt::Display for DataPipeDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
        }
    }
}

impl fmt::Display for DataMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "marker[{}]", self.tag)
    }
}

impl fmt::Display for DataHandleTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "handle_table[")?;
        for (index, (slot, resource)) in self.entries.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}={}", slot, resource)?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for DataCoreBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "core_binding[core={}]", self.core_index)
    }
}

impl fmt::Display for SurfaceTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "target[{} {}x{}]", self.format, self.width, self.height)
    }
}

impl fmt::Display for Viewport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "viewport[{}x{}]", self.width, self.height)
    }
}

impl fmt::Display for RenderPipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pipeline[{} {}]", self.shading_model, self.topology)
    }
}

impl fmt::Display for VertexLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertex_layout[stride={} attrs=", self.stride)?;
        for (index, attr) in self.attributes.iter().enumerate() {
            if index > 0 {
                write!(f, ",")?;
            }
            write!(f, "{attr}")?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for VertexBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "vertex_buffer[count={}]", self.vertex_count)
    }
}

impl fmt::Display for IndexBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "index_buffer[count={}]", self.indices.len())
    }
}

impl fmt::Display for Texture2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "texture2d[{} {}x{}]",
            self.format, self.width, self.height
        )
    }
}

impl fmt::Display for SamplerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sampler[{} {}]", self.filter, self.address_mode)
    }
}

impl fmt::Display for BlendState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blend[enabled={} mode={}]", self.enabled, self.mode)
    }
}

impl fmt::Display for DepthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "depth[test={} write={} compare={}]",
            self.test_enabled, self.write_enabled, self.compare
        )
    }
}

impl fmt::Display for RasterState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "raster[cull={} front={}]",
            self.cull_mode, self.front_face
        )
    }
}

impl fmt::Display for RenderStateSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "render_state[pipeline={}, {}, {}, {}]",
            self.pipeline, self.blend, self.depth, self.raster
        )
    }
}

impl fmt::Display for ShaderBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}={}", self.kind, self.slot, self.value)
    }
}

impl fmt::Display for ShaderBindingSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bind_set[pipeline={}, bindings=", self.pipeline)?;
        for (index, binding) in self.bindings.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{binding}")?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for RenderPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pass[target={}, pipeline={}, viewport={}]",
            self.target, self.pipeline, self.viewport
        )
    }
}

impl fmt::Display for FrameSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frame[{}x{}] ", self.width, self.height)?;
        for (index, row) in self.rows.iter().enumerate() {
            if index > 0 {
                write!(f, "|")?;
            }
            write!(f, "{row}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionSemantics {
    pub dependencies: Vec<String>,
    pub has_effect: bool,
}

impl InstructionSemantics {
    pub fn pure(dependencies: Vec<String>) -> Self {
        Self {
            dependencies,
            has_effect: false,
        }
    }

    pub fn effect(dependencies: Vec<String>) -> Self {
        Self {
            dependencies,
            has_effect: true,
        }
    }
}

pub trait RegisteredMod: Send + Sync {
    fn module_name(&self) -> &'static str;

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String>;

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String>;
}

#[derive(Default)]
pub struct ModRegistry {
    mods: BTreeMap<String, Box<dyn RegisteredMod>>,
}

impl ModRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<M>(&mut self, module: M)
    where
        M: RegisteredMod + 'static,
    {
        self.mods
            .insert(module.module_name().to_owned(), Box::new(module));
    }

    pub fn lookup(&self, name: &str) -> Option<&dyn RegisteredMod> {
        self.mods.get(name).map(|module| module.as_ref())
    }
}

#[derive(Debug, Default)]
pub struct ExecutionState {
    pub events: Vec<String>,
    pub lane_events: BTreeMap<String, Vec<String>>,
    pub values: BTreeMap<String, Value>,
    pub heap: BTreeMap<usize, HeapNode>,
    pub buffers: BTreeMap<usize, HeapBuffer>,
    pub next_heap_address: usize,
}

impl ExecutionState {
    pub fn expect_int(&self, name: &str) -> Result<i64, String> {
        match self.values.get(name) {
            Some(Value::Bool(_)) => Err(format!("`{name}` is bool, expected int")),
            Some(Value::I32(value)) => Ok(*value as i64),
            Some(Value::Int(value)) => Ok(*value),
            Some(Value::F32(_)) => Err(format!("`{name}` is f32, expected int")),
            Some(Value::F64(_)) => Err(format!("`{name}` is f64, expected int")),
            Some(Value::Symbol(_)) => Err(format!("`{name}` is symbol, expected int")),
            Some(Value::Tensor(_)) => Err(format!("`{name}` is tensor, expected int")),
            Some(Value::Pointer(_)) => Err(format!("`{name}` is pointer, expected int")),
            Some(Value::Tuple(_)) => Err(format!("`{name}` is tuple, expected int")),
            Some(Value::Struct(_)) => Err(format!("`{name}` is struct, expected int")),
            Some(Value::DataWindow(_)) => Err(format!("`{name}` is window, expected int")),
            Some(Value::DataPipe(_)) => Err(format!("`{name}` is pipe, expected int")),
            Some(Value::DataMarker(_)) => Err(format!("`{name}` is marker, expected int")),
            Some(Value::DataHandleTable(_)) => {
                Err(format!("`{name}` is handle-table, expected int"))
            }
            Some(Value::DataCoreBinding(_)) => {
                Err(format!("`{name}` is core-binding, expected int"))
            }
            Some(Value::Target(_)) => Err(format!("`{name}` is target, expected int")),
            Some(Value::Viewport(_)) => Err(format!("`{name}` is viewport, expected int")),
            Some(Value::Pipeline(_)) => Err(format!("`{name}` is pipeline, expected int")),
            Some(Value::VertexLayout(_)) => Err(format!("`{name}` is vertex-layout, expected int")),
            Some(Value::VertexBuffer(_)) => Err(format!("`{name}` is vertex-buffer, expected int")),
            Some(Value::IndexBuffer(_)) => Err(format!("`{name}` is index-buffer, expected int")),
            Some(Value::Texture(_)) => Err(format!("`{name}` is texture, expected int")),
            Some(Value::Sampler(_)) => Err(format!("`{name}` is sampler, expected int")),
            Some(Value::Blend(_)) => Err(format!("`{name}` is blend-state, expected int")),
            Some(Value::Depth(_)) => Err(format!("`{name}` is depth-state, expected int")),
            Some(Value::Raster(_)) => Err(format!("`{name}` is raster-state, expected int")),
            Some(Value::RenderState(_)) => Err(format!("`{name}` is render-state, expected int")),
            Some(Value::Binding(_)) => Err(format!("`{name}` is binding, expected int")),
            Some(Value::BindingSet(_)) => Err(format!("`{name}` is binding-set, expected int")),
            Some(Value::RenderPass(_)) => Err(format!("`{name}` is render-pass, expected int")),
            Some(Value::Frame(_)) => Err(format!("`{name}` is frame, expected int")),
            Some(Value::Unit) => Err(format!("`{name}` is unit, expected int")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_struct(&self, name: &str) -> Result<&StructValue, String> {
        match self.values.get(name) {
            Some(Value::Struct(value)) => Ok(value),
            Some(other) => Err(format!("`{name}` is {other}, expected struct")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_bool(&self, name: &str) -> Result<bool, String> {
        match self.values.get(name) {
            Some(Value::Bool(value)) => Ok(*value),
            Some(other) => Err(format!("`{name}` is {other}, expected bool")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_i32(&self, name: &str) -> Result<i32, String> {
        match self.values.get(name) {
            Some(Value::I32(value)) => Ok(*value),
            Some(other) => Err(format!("`{name}` is {other}, expected i32")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_f32(&self, name: &str) -> Result<f32, String> {
        match self.values.get(name) {
            Some(Value::F32(value)) => Ok(*value),
            Some(other) => Err(format!("`{name}` is {other}, expected f32")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_f64(&self, name: &str) -> Result<f64, String> {
        match self.values.get(name) {
            Some(Value::F64(value)) => Ok(*value),
            Some(other) => Err(format!("`{name}` is {other}, expected f64")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_value(&self, name: &str) -> Result<&Value, String> {
        self.values
            .get(name)
            .ok_or_else(|| format!("missing value for `{name}`"))
    }

    pub fn expect_pointer(&self, name: &str) -> Result<Option<usize>, String> {
        match self.values.get(name) {
            Some(Value::Pointer(pointer)) => Ok(*pointer),
            Some(other) => Err(format!("`{name}` is {other}, expected pointer")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_tensor(&self, name: &str) -> Result<&TensorValue, String> {
        match self.values.get(name) {
            Some(Value::Tensor(tensor)) => Ok(tensor),
            Some(other) => Err(format!("`{name}` is {other}, expected tensor")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn push_event(&mut self, event: impl Into<String>) {
        self.events.push(event.into());
    }

    pub fn push_resource_event(&mut self, resource: &Resource, event: impl Into<String>) {
        let event = event.into();
        self.events.push(event.clone());
        self.lane_events
            .entry(resource.kind.family().to_owned())
            .or_default()
            .push(event);
    }

    pub fn alloc_heap_node(&mut self, value: i64, next: Option<usize>) -> usize {
        let address = self.next_heap_address.max(1);
        self.next_heap_address = address + 1;
        self.heap.insert(address, HeapNode { value, next });
        address
    }

    pub fn read_heap_node(&self, pointer: Option<usize>) -> Result<&HeapNode, String> {
        let Some(address) = pointer else {
            return Err("null pointer dereference".to_owned());
        };
        if self.buffers.contains_key(&address) {
            return Err(format!("buffer pointer `&{address}` used as node pointer"));
        }
        self.heap
            .get(&address)
            .ok_or_else(|| format!("dangling pointer dereference `&{address}`"))
    }

    pub fn write_heap_value(&mut self, pointer: Option<usize>, value: i64) -> Result<(), String> {
        let Some(address) = pointer else {
            return Err("null pointer store".to_owned());
        };
        if self.buffers.contains_key(&address) {
            return Err(format!("buffer pointer `&{address}` used as node pointer"));
        }
        let node = self
            .heap
            .get_mut(&address)
            .ok_or_else(|| format!("dangling pointer store `&{address}`"))?;
        node.value = value;
        Ok(())
    }

    pub fn write_heap_next(
        &mut self,
        pointer: Option<usize>,
        next: Option<usize>,
    ) -> Result<(), String> {
        let Some(address) = pointer else {
            return Err("null pointer next-store".to_owned());
        };
        if self.buffers.contains_key(&address) {
            return Err(format!("buffer pointer `&{address}` used as node pointer"));
        }
        let node = self
            .heap
            .get_mut(&address)
            .ok_or_else(|| format!("dangling pointer next-store `&{address}`"))?;
        node.next = next;
        Ok(())
    }

    pub fn alloc_heap_buffer(&mut self, len: usize, fill: i64) -> usize {
        let address = self.next_heap_address.max(1);
        self.next_heap_address = address + 1;
        self.buffers.insert(
            address,
            HeapBuffer {
                elements: vec![fill; len],
            },
        );
        address
    }

    pub fn read_heap_buffer(&self, pointer: Option<usize>) -> Result<&HeapBuffer, String> {
        let Some(address) = pointer else {
            return Err("null buffer dereference".to_owned());
        };
        if self.heap.contains_key(&address) {
            return Err(format!("node pointer `&{address}` used as buffer pointer"));
        }
        self.buffers
            .get(&address)
            .ok_or_else(|| format!("dangling buffer dereference `&{address}`"))
    }

    pub fn read_heap_buffer_at(&self, pointer: Option<usize>, index: usize) -> Result<i64, String> {
        let buffer = self.read_heap_buffer(pointer)?;
        buffer
            .elements
            .get(index)
            .copied()
            .ok_or_else(|| format!("buffer index `{index}` out of bounds"))
    }

    pub fn write_heap_buffer_at(
        &mut self,
        pointer: Option<usize>,
        index: usize,
        value: i64,
    ) -> Result<(), String> {
        let Some(address) = pointer else {
            return Err("null buffer store".to_owned());
        };
        if self.heap.contains_key(&address) {
            return Err(format!("node pointer `&{address}` used as buffer pointer"));
        }
        let buffer = self
            .buffers
            .get_mut(&address)
            .ok_or_else(|| format!("dangling buffer store `&{address}`"))?;
        let slot = buffer
            .elements
            .get_mut(index)
            .ok_or_else(|| format!("buffer index `{index}` out of bounds"))?;
        *slot = value;
        Ok(())
    }

    pub fn heap_buffer_len(&self, pointer: Option<usize>) -> Result<usize, String> {
        Ok(self.read_heap_buffer(pointer)?.elements.len())
    }

    pub fn free_heap_node(&mut self, pointer: Option<usize>) -> Result<(), String> {
        let Some(address) = pointer else {
            return Err("null pointer free".to_owned());
        };
        if self.heap.remove(&address).is_some() || self.buffers.remove(&address).is_some() {
            Ok(())
        } else {
            Err(format!("double free or dangling pointer `&{address}`"))
        }
    }
}

pub struct DataMod;

impl RegisteredMod for DataMod {
    fn module_name(&self) -> &'static str {
        "data"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        if node.op.instruction != "move" {
            require_data_resource(node, resource)?;
        }
        match node.op.instruction.as_str() {
            "move" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `data.move <name> <resource> <input> <to>`",
                        node.name
                    ));
                }

                Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
            }
            "copy_window" | "immutable_window" => {
                if node.op.args.len() != 3 {
                    return Err(format!(
                        "node `{}` expects `data.{} <name> <resource> <input> <offset> <len>`",
                        node.name, node.op.instruction
                    ));
                }
                let mut deps = vec![node.op.args[0].clone()];
                if node.op.args[1].parse::<usize>().is_err() {
                    deps.push(node.op.args[1].clone());
                }
                if node.op.args[2].parse::<usize>().is_err() {
                    deps.push(node.op.args[2].clone());
                }
                Ok(InstructionSemantics::pure(deps))
            }
            "marker" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `data.marker <name> <resource> <tag>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "output_pipe" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `data.output_pipe <name> <resource> <input>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
            }
            "input_pipe" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `data.input_pipe <name> <resource> <pipe>`",
                        node.name
                    ));
                }
                Ok(InstructionSemantics::effect(vec![node.op.args[0].clone()]))
            }
            "handle_table" => {
                if node.op.args.is_empty() {
                    return Err(format!(
                        "node `{}` expects `data.handle_table <name> <resource> <slot=resource> [slot=resource...]`",
                        node.name
                    ));
                }
                for entry in &node.op.args {
                    if entry.split_once('=').is_none() {
                        return Err(format!(
                            "node `{}` has invalid handle-table entry `{}`",
                            node.name, entry
                        ));
                    }
                }
                Ok(InstructionSemantics::pure(Vec::new()))
            }
            "bind_core" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `data.bind_core <name> <resource> <core_index>`",
                        node.name
                    ));
                }
                node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid fabric core index `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                Ok(InstructionSemantics::effect(Vec::new()))
            }
            other => Err(format!("unknown data instruction `{other}`")),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if node.op.instruction != "move" {
            require_data_resource(node, resource)?;
        }
        match node.op.instruction.as_str() {
            "move" => {
                let input = &node.op.args[0];
                let target = &node.op.args[1];
                let value = state.expect_value(input)?.clone();
                if !is_move_value_legal(&value) {
                    return Err(format!(
                        "data.move only accepts Value payloads, got {}",
                        value
                    ));
                }
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.move @{} [{}] -> {}: {}",
                        node.resource, resource.kind.raw, target, value
                    ),
                );
                Ok(value)
            }
            "copy_window" | "immutable_window" => {
                let base = state.expect_value(&node.op.args[0])?.clone();
                if !is_window_base_legal(&base) {
                    return Err(format!(
                        "data.{} cannot wrap non-window-compatible payload {}",
                        node.op.instruction, base
                    ));
                }
                let offset = resolve_window_usize_arg(state, node, 1, "offset")?;
                let len = resolve_window_usize_arg(state, node, 2, "len")?;
                let window = Value::DataWindow(DataWindow {
                    base: Box::new(base),
                    offset,
                    len,
                    immutable: node.op.instruction == "immutable_window",
                });
                Ok(window)
            }
            "marker" => Ok(Value::DataMarker(DataMarker {
                tag: node.op.args[0].clone(),
            })),
            "output_pipe" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                if !is_pipe_payload_legal(&value) {
                    return Err(format!(
                        "data.output_pipe cannot wrap illegal pipe payload {}",
                        value
                    ));
                }
                let pipe = Value::DataPipe(DataPipe {
                    direction: DataPipeDirection::Output,
                    payload: Box::new(value),
                });
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.output_pipe @{} [{}]: {}",
                        node.resource, resource.kind.raw, pipe
                    ),
                );
                Ok(pipe)
            }
            "input_pipe" => {
                let pipe = state.expect_value(&node.op.args[0])?.clone();
                match pipe {
                    Value::DataPipe(DataPipe {
                        direction: DataPipeDirection::Output,
                        payload,
                    }) => {
                        let value = (*payload).clone();
                        state.push_resource_event(
                            resource,
                            format!(
                                "effect data.input_pipe @{} [{}]: {}",
                                node.resource, resource.kind.raw, value
                            ),
                        );
                        Ok(value)
                    }
                    other => Err(format!(
                        "data.input_pipe expects output pipe, got {}",
                        other
                    )),
                }
            }
            "handle_table" => {
                let mut entries = Vec::with_capacity(node.op.args.len());
                for entry in &node.op.args {
                    let Some((slot, resource_name)) = entry.split_once('=') else {
                        return Err(format!(
                            "node `{}` has invalid handle-table entry `{}`",
                            node.name, entry
                        ));
                    };
                    let slot = slot.trim();
                    let resource_name = resource_name.trim();
                    if slot.is_empty() || resource_name.is_empty() {
                        return Err(format!(
                            "node `{}` has empty handle-table slot/resource in `{}`",
                            node.name, entry
                        ));
                    }
                    entries.push((slot.to_owned(), resource_name.to_owned()));
                }
                Ok(Value::DataHandleTable(DataHandleTable { entries }))
            }
            "bind_core" => {
                let core_index = node.op.args[0].parse::<usize>().map_err(|_| {
                    format!(
                        "node `{}` has invalid fabric core index `{}`",
                        node.name, node.op.args[0]
                    )
                })?;
                let binding = Value::DataCoreBinding(DataCoreBinding { core_index });
                state.push_resource_event(
                    resource,
                    format!(
                        "effect data.bind_core @{} [{}]: {}",
                        node.resource, resource.kind.raw, binding
                    ),
                );
                Ok(binding)
            }
            other => Err(format!("unknown data instruction `{other}`")),
        }
    }
}

fn require_data_resource(node: &Node, resource: &Resource) -> Result<(), String> {
    if resource.kind.is_family("data") || resource.kind.is_family("fabric") {
        Ok(())
    } else {
        Err(format!(
            "node `{}` uses data mod on non-data resource `{}` ({})",
            node.name, resource.name, resource.kind.raw
        ))
    }
}

fn is_move_value_legal(value: &Value) -> bool {
    match value {
        Value::DataWindow(_)
        | Value::DataPipe(_)
        | Value::DataMarker(_)
        | Value::DataHandleTable(_) => false,
        Value::Tuple(items) => items.iter().all(is_move_value_legal),
        Value::Struct(value) => value
            .fields
            .iter()
            .all(|(_, value)| is_move_value_legal(value)),
        _ => true,
    }
}

fn is_window_base_legal(value: &Value) -> bool {
    match value {
        Value::DataHandleTable(_) | Value::DataMarker(_) | Value::DataPipe(_) => false,
        Value::Tuple(items) => items.iter().all(is_move_value_legal),
        Value::Struct(value) => value
            .fields
            .iter()
            .all(|(_, value)| is_move_value_legal(value)),
        _ => true,
    }
}

fn is_pipe_payload_legal(value: &Value) -> bool {
    match value {
        Value::DataPipe(_) => false,
        Value::Tuple(items) => items.iter().all(is_move_value_legal),
        Value::Struct(value) => value
            .fields
            .iter()
            .all(|(_, value)| is_move_value_legal(value)),
        _ => true,
    }
}

fn resolve_window_usize_arg(
    state: &ExecutionState,
    node: &Node,
    index: usize,
    label: &str,
) -> Result<usize, String> {
    let raw = &node.op.args[index];
    if let Ok(value) = raw.parse::<usize>() {
        return Ok(value);
    }
    let value = state.expect_int(raw)?;
    usize::try_from(value).map_err(|_| {
        format!(
            "node `{}` has invalid window {} `{}`",
            node.name, label, raw
        )
    })
}

pub struct LegacyFabricMod;

impl RegisteredMod for LegacyFabricMod {
    fn module_name(&self) -> &'static str {
        "fabric"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        DataMod.describe(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        DataMod.execute(node, resource, state)
    }
}
