use std::collections::BTreeMap;
mod data_mod;
mod data_mod_describe;
pub mod ffi;
mod glm;
mod module_graph;
mod operation_results;
mod operation_semantics;
mod registry;
mod value_display;
mod value_types;

pub use data_mod::{DataMod, LegacyFabricMod};
pub use glm::*;
pub use module_graph::*;
pub use registry::*;
pub use value_types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub module: String,
    pub instruction: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationDomainFamily {
    Cpu,
    Data,
    Shader,
    Kernel,
    Network,
    Npu,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticOp {
    CpuAllocNode,
    CpuAllocBuffer,
    CpuBorrow,
    CpuBorrowEnd,
    CpuMovePtr,
    CpuInstantiateUnit,
    CpuProjectProfileRef,
    CpuLoadValue,
    CpuLoadNext,
    CpuBufferLen,
    CpuLoadAt,
    CpuStoreValue,
    CpuStoreNext,
    CpuStoreAt,
    CpuFree,
    CpuJoin,
    CpuCancel,
    CpuTimeout,
    CpuJoinResult,
    CpuTaskCompleted,
    CpuTaskTimedOut,
    CpuTaskCancelled,
    CpuTaskValue,
    DataMove,
    DataCopyWindow,
    DataReadWindow,
    DataWriteWindow,
    DataFreezeWindow,
    DataImmutableWindow,
    DataOutputPipe,
    DataInputPipe,
    DataObserve,
    DataIsReady,
    DataIsMoved,
    DataIsWindowed,
    DataValue,
    DataMarker,
    DataHandleTable,
    DataBindCore,
    ShaderObserve,
    ShaderIsPassReady,
    ShaderIsFrameReady,
    ShaderValue,
    KernelObserve,
    KernelIsConfigReady,
    KernelValue,
    NetworkObserve,
    NetworkConnect,
    NetworkAccept,
    NetworkClose,
    NetworkIsConfigReady,
    NetworkIsSendReady,
    NetworkIsRecvReady,
    NetworkIsConnectReady,
    NetworkIsAcceptReady,
    NetworkIsClosed,
    NetworkValue,
    ShaderBeginPass,
    ShaderDrawInstanced,
    ShaderPipeline,
    ShaderInlineWgsl,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFabricPrimitive {
    Bind,
    Handle,
    Marker,
    Move,
    Window,
    Pipe,
    Observe,
}

impl DataFabricPrimitive {
    pub fn render(self) -> &'static str {
        match self {
            Self::Bind => "bind",
            Self::Handle => "handle",
            Self::Marker => "marker",
            Self::Move => "move",
            Self::Window => "window",
            Self::Pipe => "pipe",
            Self::Observe => "observe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsyncCoreOp {
    Await,
    ScheduleCall,
    SpawnTask,
    JoinTask,
    CancelTask,
    TimeoutTask,
    ObserveTaskResult,
    ProbeTaskCompleted,
    ProbeTaskTimedOut,
    ProbeTaskCancelled,
    ExtractTaskValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YirResultFamily {
    Task,
    Data,
    Shader,
    Kernel,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YirResultState {
    Task(TaskLifecycleState),
    Data(DataFlowState),
    Shader(ShaderFlowState),
    Kernel(KernelFlowState),
    Network(NetworkFlowState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YirResultRole {
    Entry,
    StateProbe,
    PayloadExtractor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuLlvmLoweringClass {
    NonCpu,
    Literal,
    Aggregate,
    Pointer,
    Arithmetic,
    Compare,
    Bitwise,
    Cast,
    Memory,
    Runtime,
    Effect,
    Other,
}

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct ExecutionState {
    pub events: Vec<String>,
    pub lane_events: BTreeMap<String, Vec<String>>,
    pub values: BTreeMap<String, Value>,
    pub heap: BTreeMap<usize, HeapNode>,
    pub buffers: BTreeMap<usize, HeapBuffer>,
    pub next_heap_address: usize,
    pub current_lane: Option<String>,
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
            Some(Value::VariantUnion(_)) => Err(format!("`{name}` is variant-union, expected int")),
            Some(Value::DataWindow(_)) => Err(format!("`{name}` is window, expected int")),
            Some(Value::DataPipe(_)) => Err(format!("`{name}` is pipe, expected int")),
            Some(Value::DataResult(_)) => Err(format!("`{name}` is data-result, expected int")),
            Some(Value::DataMarker(_)) => Err(format!("`{name}` is marker, expected int")),
            Some(Value::DataHandleTable(_)) => {
                Err(format!("`{name}` is handle-table, expected int"))
            }
            Some(Value::DataCoreBinding(_)) => {
                Err(format!("`{name}` is core-binding, expected int"))
            }
            Some(Value::ShaderResult(_)) => Err(format!("`{name}` is shader-result, expected int")),
            Some(Value::KernelResult(_)) => Err(format!("`{name}` is kernel-result, expected int")),
            Some(Value::NetworkResult(_)) => {
                Err(format!("`{name}` is network-result, expected int"))
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
            Some(Value::Task(_)) => Err(format!("`{name}` is task, expected int")),
            Some(Value::Thread(_)) => Err(format!("`{name}` is thread, expected int")),
            Some(Value::TaskResult(_)) => Err(format!("`{name}` is task-result, expected int")),
            Some(Value::Mutex(_)) => Err(format!("`{name}` is mutex, expected int")),
            Some(Value::MutexGuard(_)) => Err(format!("`{name}` is mutex-guard, expected int")),
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

    pub fn bind_value(&mut self, name: impl Into<String>, value: Value) {
        self.values.insert(name.into(), value);
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

    pub fn expect_task(&self, name: &str) -> Result<&TaskHandle, String> {
        match self.values.get(name) {
            Some(Value::Task(task)) => Ok(task),
            Some(other) => Err(format!("`{name}` is {other}, expected task")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_task_result(&self, name: &str) -> Result<&TaskResultHandle, String> {
        match self.values.get(name) {
            Some(Value::TaskResult(result)) => Ok(result),
            Some(other) => Err(format!("`{name}` is {other}, expected task-result")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_thread(&self, name: &str) -> Result<&ThreadHandle, String> {
        match self.values.get(name) {
            Some(Value::Thread(thread)) => Ok(thread),
            Some(other) => Err(format!("`{name}` is {other}, expected thread")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_mutex(&self, name: &str) -> Result<&MutexHandle, String> {
        match self.values.get(name) {
            Some(Value::Mutex(mutex)) => Ok(mutex),
            Some(other) => Err(format!("`{name}` is {other}, expected mutex")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_mutex_guard(&self, name: &str) -> Result<&MutexGuardHandle, String> {
        match self.values.get(name) {
            Some(Value::MutexGuard(guard)) => Ok(guard),
            Some(other) => Err(format!("`{name}` is {other}, expected mutex-guard")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_data_result(&self, name: &str) -> Result<&DataResultHandle, String> {
        match self.values.get(name) {
            Some(Value::DataResult(result)) => Ok(result),
            Some(other) => Err(format!("`{name}` is {other}, expected data-result")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_shader_result(&self, name: &str) -> Result<&ShaderResultHandle, String> {
        match self.values.get(name) {
            Some(Value::ShaderResult(result)) => Ok(result),
            Some(other) => Err(format!("`{name}` is {other}, expected shader-result")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_kernel_result(&self, name: &str) -> Result<&KernelResultHandle, String> {
        match self.values.get(name) {
            Some(Value::KernelResult(result)) => Ok(result),
            Some(other) => Err(format!("`{name}` is {other}, expected kernel-result")),
            None => Err(format!("missing value for `{name}`")),
        }
    }

    pub fn expect_network_result(&self, name: &str) -> Result<&NetworkResultHandle, String> {
        match self.values.get(name) {
            Some(Value::NetworkResult(result)) => Ok(result),
            Some(other) => Err(format!("`{name}` is {other}, expected network-result")),
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
            .entry(
                self.current_lane
                    .clone()
                    .unwrap_or_else(|| resource.kind.family().to_owned()),
            )
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
        for (owner_id, node) in &self.heap {
            if *owner_id == address {
                continue;
            }
            if node.next == Some(address) {
                return Err(format!(
                    "cannot free `&{address}` while live node `&{owner_id}` still links to it"
                ));
            }
        }
        if self.heap.remove(&address).is_some() || self.buffers.remove(&address).is_some() {
            Ok(())
        } else {
            Err(format!("double free or dangling pointer `&{address}`"))
        }
    }
}
