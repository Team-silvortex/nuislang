use std::collections::BTreeMap;

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
    VariantUnion(VariantUnionValue),
    DataWindow(DataWindow),
    DataPipe(DataPipe),
    DataResult(DataResultHandle),
    DataMarker(DataMarker),
    DataHandleTable(DataHandleTable),
    DataCoreBinding(DataCoreBinding),
    ShaderResult(ShaderResultHandle),
    KernelResult(KernelResultHandle),
    NetworkResult(NetworkResultHandle),
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
    Task(TaskHandle),
    Thread(ThreadHandle),
    TaskResult(TaskResultHandle),
    Mutex(MutexHandle),
    MutexGuard(MutexGuardHandle),
    Unit,
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

#[derive(Debug, Clone, PartialEq)]
pub struct VariantUnionValue {
    pub parent_type_name: String,
    pub active_variant: String,
    pub variants: BTreeMap<String, StructValue>,
}

impl Eq for Value {}

impl Eq for StructValue {}

impl Eq for VariantUnionValue {}

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
pub struct DataResultHandle {
    pub state: DataFlowState,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFlowState {
    Ready,
    Moved,
    Windowed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderResultHandle {
    pub state: ShaderFlowState,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderFlowState {
    PassReady,
    FrameReady,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelResultHandle {
    pub state: KernelFlowState,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelFlowState {
    ConfigReady,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkResultHandle {
    pub state: NetworkFlowState,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFlowState {
    ConfigReady,
    SendReady,
    RecvReady,
    ConnectReady,
    AcceptReady,
    Closed,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskHandle {
    pub label: String,
    pub result: Box<Value>,
    pub limit: Option<i64>,
    pub ready_delay: i64,
    pub state: TaskLifecycleState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadHandle {
    pub label: String,
    pub result: Box<Value>,
    pub state: TaskLifecycleState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskResultHandle {
    pub label: String,
    pub state: TaskLifecycleState,
    pub result: Option<Box<Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutexHandle {
    pub label: String,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutexGuardHandle {
    pub label: String,
    pub value: Box<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycleState {
    Pending,
    Completed,
    TimedOut,
    Cancelled,
}
