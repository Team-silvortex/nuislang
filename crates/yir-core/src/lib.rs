use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirModule {
    pub version: String,
    pub resources: Vec<Resource>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
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
        Self { raw: raw.to_owned() }
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
            return Err(format!("operation `{raw}` must be written as <mod>.<instr>"));
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Symbol(String),
    Tensor(TensorValue),
    Pointer(Option<usize>),
    Tuple(Vec<Value>),
    Target(SurfaceTarget),
    Viewport(Viewport),
    Pipeline(RenderPipeline),
    RenderPass(RenderPass),
    Frame(FrameSurface),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TensorValue {
    pub rows: usize,
    pub cols: usize,
    pub elements: Vec<i64>,
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
            Self::Int(value) => write!(f, "{value}"),
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
            Self::Target(target) => write!(f, "{target}"),
            Self::Viewport(viewport) => write!(f, "{viewport}"),
            Self::Pipeline(pipeline) => write!(f, "{pipeline}"),
            Self::RenderPass(pass) => write!(f, "{pass}"),
            Self::Frame(frame) => write!(f, "{frame}"),
            Self::Unit => write!(f, "()"),
        }
    }
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
            Some(Value::Int(value)) => Ok(*value),
            Some(Value::Symbol(_)) => Err(format!("`{name}` is symbol, expected int")),
            Some(Value::Tensor(_)) => Err(format!("`{name}` is tensor, expected int")),
            Some(Value::Pointer(_)) => Err(format!("`{name}` is pointer, expected int")),
            Some(Value::Tuple(_)) => Err(format!("`{name}` is tuple, expected int")),
            Some(Value::Target(_)) => Err(format!("`{name}` is target, expected int")),
            Some(Value::Viewport(_)) => Err(format!("`{name}` is viewport, expected int")),
            Some(Value::Pipeline(_)) => Err(format!("`{name}` is pipeline, expected int")),
            Some(Value::RenderPass(_)) => Err(format!("`{name}` is render-pass, expected int")),
            Some(Value::Frame(_)) => Err(format!("`{name}` is frame, expected int")),
            Some(Value::Unit) => Err(format!("`{name}` is unit, expected int")),
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

    pub fn read_heap_buffer_at(
        &self,
        pointer: Option<usize>,
        index: usize,
    ) -> Result<i64, String> {
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

    fn describe(&self, node: &Node, _resource: &Resource) -> Result<InstructionSemantics, String> {
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
            other => Err(format!("unknown data instruction `{other}`")),
        }
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        match node.op.instruction.as_str() {
            "move" => {
                let input = &node.op.args[0];
                let target = &node.op.args[1];
                let value = state.expect_value(input)?.clone();
                state.push_resource_event(resource, format!(
                    "effect data.move @{} [{}] -> {}: {}",
                    node.resource, resource.kind.raw, target, value
                ));
                Ok(value)
            }
            other => Err(format!("unknown data instruction `{other}`")),
        }
    }
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
