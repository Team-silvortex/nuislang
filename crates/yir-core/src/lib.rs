use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YirModule {
    pub version: String,
    pub resources: Vec<Resource>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_lanes: BTreeMap<String, String>,
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
            node_lanes: BTreeMap::new(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationDomainFamily {
    Cpu,
    Data,
    Shader,
    Kernel,
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
    ShaderBeginPass,
    ShaderDrawInstanced,
    ShaderPipeline,
    ShaderInlineWgsl,
    Other,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YirResultState {
    Task(TaskLifecycleState),
    Data(DataFlowState),
    Shader(ShaderFlowState),
    Kernel(KernelFlowState),
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

    pub fn domain_family(&self) -> OperationDomainFamily {
        match self.module.as_str() {
            "cpu" => OperationDomainFamily::Cpu,
            "data" | "fabric" => OperationDomainFamily::Data,
            "shader" => OperationDomainFamily::Shader,
            "kernel" => OperationDomainFamily::Kernel,
            "npu" => OperationDomainFamily::Npu,
            _ => OperationDomainFamily::Unknown,
        }
    }

    pub fn is_data_domain_family(&self) -> bool {
        self.domain_family() == OperationDomainFamily::Data
    }

    pub fn semantic_op(&self) -> SemanticOp {
        match (self.domain_family(), self.instruction.as_str()) {
            (OperationDomainFamily::Cpu, "alloc_node") => SemanticOp::CpuAllocNode,
            (OperationDomainFamily::Cpu, "alloc_buffer") => SemanticOp::CpuAllocBuffer,
            (OperationDomainFamily::Cpu, "borrow") => SemanticOp::CpuBorrow,
            (OperationDomainFamily::Cpu, "borrow_end") => SemanticOp::CpuBorrowEnd,
            (OperationDomainFamily::Cpu, "move_ptr") => SemanticOp::CpuMovePtr,
            (OperationDomainFamily::Cpu, "instantiate_unit") => SemanticOp::CpuInstantiateUnit,
            (OperationDomainFamily::Cpu, "project_profile_ref") => SemanticOp::CpuProjectProfileRef,
            (OperationDomainFamily::Cpu, "load_value") => SemanticOp::CpuLoadValue,
            (OperationDomainFamily::Cpu, "load_next") => SemanticOp::CpuLoadNext,
            (OperationDomainFamily::Cpu, "buffer_len") => SemanticOp::CpuBufferLen,
            (OperationDomainFamily::Cpu, "load_at") => SemanticOp::CpuLoadAt,
            (OperationDomainFamily::Cpu, "store_value") => SemanticOp::CpuStoreValue,
            (OperationDomainFamily::Cpu, "store_next") => SemanticOp::CpuStoreNext,
            (OperationDomainFamily::Cpu, "store_at") => SemanticOp::CpuStoreAt,
            (OperationDomainFamily::Cpu, "free") => SemanticOp::CpuFree,
            (OperationDomainFamily::Cpu, "join") => SemanticOp::CpuJoin,
            (OperationDomainFamily::Cpu, "cancel") => SemanticOp::CpuCancel,
            (OperationDomainFamily::Cpu, "timeout") => SemanticOp::CpuTimeout,
            (OperationDomainFamily::Cpu, "join_result") => SemanticOp::CpuJoinResult,
            (OperationDomainFamily::Cpu, "task_completed") => SemanticOp::CpuTaskCompleted,
            (OperationDomainFamily::Cpu, "task_timed_out") => SemanticOp::CpuTaskTimedOut,
            (OperationDomainFamily::Cpu, "task_cancelled") => SemanticOp::CpuTaskCancelled,
            (OperationDomainFamily::Cpu, "task_value") => SemanticOp::CpuTaskValue,
            (OperationDomainFamily::Data, "move") => SemanticOp::DataMove,
            (OperationDomainFamily::Data, "copy_window") => SemanticOp::DataCopyWindow,
            (OperationDomainFamily::Data, "immutable_window") => SemanticOp::DataImmutableWindow,
            (OperationDomainFamily::Data, "output_pipe") => SemanticOp::DataOutputPipe,
            (OperationDomainFamily::Data, "input_pipe") => SemanticOp::DataInputPipe,
            (OperationDomainFamily::Data, "observe") => SemanticOp::DataObserve,
            (OperationDomainFamily::Data, "is_ready") => SemanticOp::DataIsReady,
            (OperationDomainFamily::Data, "is_moved") => SemanticOp::DataIsMoved,
            (OperationDomainFamily::Data, "is_windowed") => SemanticOp::DataIsWindowed,
            (OperationDomainFamily::Data, "value") => SemanticOp::DataValue,
            (OperationDomainFamily::Data, "marker") => SemanticOp::DataMarker,
            (OperationDomainFamily::Data, "handle_table") => SemanticOp::DataHandleTable,
            (OperationDomainFamily::Data, "bind_core") => SemanticOp::DataBindCore,
            (OperationDomainFamily::Shader, "observe") => SemanticOp::ShaderObserve,
            (OperationDomainFamily::Shader, "is_pass_ready") => SemanticOp::ShaderIsPassReady,
            (OperationDomainFamily::Shader, "is_frame_ready") => SemanticOp::ShaderIsFrameReady,
            (OperationDomainFamily::Shader, "value") => SemanticOp::ShaderValue,
            (OperationDomainFamily::Shader, "begin_pass") => SemanticOp::ShaderBeginPass,
            (OperationDomainFamily::Shader, "draw_instanced") => SemanticOp::ShaderDrawInstanced,
            (OperationDomainFamily::Shader, "pipeline") => SemanticOp::ShaderPipeline,
            (OperationDomainFamily::Shader, "inline_wgsl") => SemanticOp::ShaderInlineWgsl,
            (OperationDomainFamily::Kernel, "observe") => SemanticOp::KernelObserve,
            (OperationDomainFamily::Kernel, "is_config_ready") => SemanticOp::KernelIsConfigReady,
            (OperationDomainFamily::Kernel, "value") => SemanticOp::KernelValue,
            _ => SemanticOp::Other,
        }
    }

    pub fn is_shader_semantic_op(&self, expected: SemanticOp) -> bool {
        self.semantic_op() == expected
    }

    pub fn is_cpu_semantic_op(&self, expected: SemanticOp) -> bool {
        self.semantic_op() == expected
    }

    pub fn is_domain_family(&self, expected: OperationDomainFamily) -> bool {
        self.domain_family() == expected
    }

    pub fn is_data_marker_tag(&self, expected: &str) -> bool {
        self.semantic_op() == SemanticOp::DataMarker
            && self.args.first().map(String::as_str) == Some(expected)
    }

    pub fn is_data_pipe_semantic_op(&self) -> bool {
        matches!(
            self.semantic_op(),
            SemanticOp::DataOutputPipe | SemanticOp::DataInputPipe
        )
    }

    pub fn is_data_window_semantic_op(&self) -> bool {
        matches!(
            self.semantic_op(),
            SemanticOp::DataCopyWindow | SemanticOp::DataImmutableWindow
        )
    }

    pub fn async_core_op(&self) -> Option<AsyncCoreOp> {
        match self.semantic_op() {
            SemanticOp::CpuJoin => Some(AsyncCoreOp::JoinTask),
            SemanticOp::CpuCancel => Some(AsyncCoreOp::CancelTask),
            SemanticOp::CpuTimeout => Some(AsyncCoreOp::TimeoutTask),
            SemanticOp::CpuJoinResult => Some(AsyncCoreOp::ObserveTaskResult),
            SemanticOp::CpuTaskCompleted => Some(AsyncCoreOp::ProbeTaskCompleted),
            SemanticOp::CpuTaskTimedOut => Some(AsyncCoreOp::ProbeTaskTimedOut),
            SemanticOp::CpuTaskCancelled => Some(AsyncCoreOp::ProbeTaskCancelled),
            SemanticOp::CpuTaskValue => Some(AsyncCoreOp::ExtractTaskValue),
            _ if self.domain_family() == OperationDomainFamily::Cpu => match self.instruction.as_str()
            {
                "await" => Some(AsyncCoreOp::Await),
                "async_call" => Some(AsyncCoreOp::ScheduleCall),
                "spawn_task" => Some(AsyncCoreOp::SpawnTask),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn is_async_core_op(&self) -> bool {
        self.async_core_op().is_some()
    }

    pub fn is_async_task_result_observer(&self) -> bool {
        matches!(
            self.async_core_op(),
            Some(
                AsyncCoreOp::ProbeTaskCompleted
                    | AsyncCoreOp::ProbeTaskTimedOut
                    | AsyncCoreOp::ProbeTaskCancelled
                    | AsyncCoreOp::ExtractTaskValue
            )
        )
    }

    pub fn semantic_name(&self) -> &'static str {
        match self.semantic_op() {
            SemanticOp::CpuProjectProfileRef => "cpu.project_profile_ref",
            SemanticOp::CpuJoinResult => "cpu.join_result",
            SemanticOp::CpuTaskCompleted => "cpu.task_completed",
            SemanticOp::CpuTaskTimedOut => "cpu.task_timed_out",
            SemanticOp::CpuTaskCancelled => "cpu.task_cancelled",
            SemanticOp::CpuTaskValue => "cpu.task_value",
            SemanticOp::DataObserve => "data.observe",
            SemanticOp::DataIsReady => "data.is_ready",
            SemanticOp::DataIsMoved => "data.is_moved",
            SemanticOp::DataIsWindowed => "data.is_windowed",
            SemanticOp::DataValue => "data.value",
            SemanticOp::ShaderObserve => "shader.observe",
            SemanticOp::ShaderIsPassReady => "shader.is_pass_ready",
            SemanticOp::ShaderIsFrameReady => "shader.is_frame_ready",
            SemanticOp::ShaderValue => "shader.value",
            SemanticOp::KernelObserve => "kernel.observe",
            SemanticOp::KernelIsConfigReady => "kernel.is_config_ready",
            SemanticOp::KernelValue => "kernel.value",
            SemanticOp::DataBindCore => "data.bind_core",
            SemanticOp::DataMarker => "data.marker",
            SemanticOp::DataHandleTable => "data.handle_table",
            SemanticOp::DataOutputPipe => "data.output_pipe",
            SemanticOp::DataInputPipe => "data.input_pipe",
            SemanticOp::DataCopyWindow => "data.copy_window",
            SemanticOp::DataImmutableWindow => "data.immutable_window",
            SemanticOp::ShaderBeginPass => "shader.begin_pass",
            SemanticOp::ShaderDrawInstanced => "shader.draw_instanced",
            _ => "other",
        }
    }

    pub fn result_family(&self) -> Option<YirResultFamily> {
        match self.semantic_op() {
            SemanticOp::CpuJoinResult
            | SemanticOp::CpuTaskCompleted
            | SemanticOp::CpuTaskTimedOut
            | SemanticOp::CpuTaskCancelled
            | SemanticOp::CpuTaskValue => Some(YirResultFamily::Task),
            SemanticOp::DataObserve
            | SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::DataValue => Some(YirResultFamily::Data),
            SemanticOp::ShaderObserve
            | SemanticOp::ShaderIsPassReady
            | SemanticOp::ShaderIsFrameReady
            | SemanticOp::ShaderValue => Some(YirResultFamily::Shader),
            SemanticOp::KernelObserve
            | SemanticOp::KernelIsConfigReady
            | SemanticOp::KernelValue => Some(YirResultFamily::Kernel),
            _ => None,
        }
    }

    pub fn result_role(&self) -> Option<YirResultRole> {
        match self.semantic_op() {
            SemanticOp::CpuJoinResult
            | SemanticOp::DataObserve
            | SemanticOp::ShaderObserve
            | SemanticOp::KernelObserve => Some(YirResultRole::Entry),
            SemanticOp::CpuTaskCompleted
            | SemanticOp::CpuTaskTimedOut
            | SemanticOp::CpuTaskCancelled
            | SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::ShaderIsPassReady
            | SemanticOp::ShaderIsFrameReady
            | SemanticOp::KernelIsConfigReady => Some(YirResultRole::StateProbe),
            SemanticOp::CpuTaskValue
            | SemanticOp::DataValue
            | SemanticOp::ShaderValue
            | SemanticOp::KernelValue => Some(YirResultRole::PayloadExtractor),
            _ => None,
        }
    }

    pub fn result_source_semantic_op(&self) -> Option<SemanticOp> {
        match self.semantic_op() {
            SemanticOp::CpuTaskCompleted
            | SemanticOp::CpuTaskTimedOut
            | SemanticOp::CpuTaskCancelled
            | SemanticOp::CpuTaskValue => Some(SemanticOp::CpuJoinResult),
            SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::DataValue => Some(SemanticOp::DataObserve),
            SemanticOp::ShaderIsPassReady | SemanticOp::ShaderIsFrameReady | SemanticOp::ShaderValue => {
                Some(SemanticOp::ShaderObserve)
            }
            SemanticOp::KernelIsConfigReady | SemanticOp::KernelValue => {
                Some(SemanticOp::KernelObserve)
            }
            _ => None,
        }
    }

    pub fn result_probe_state(&self) -> Option<YirResultState> {
        match self.semantic_op() {
            SemanticOp::CpuTaskCompleted => {
                Some(YirResultState::Task(TaskLifecycleState::Completed))
            }
            SemanticOp::CpuTaskTimedOut => Some(YirResultState::Task(TaskLifecycleState::TimedOut)),
            SemanticOp::CpuTaskCancelled => {
                Some(YirResultState::Task(TaskLifecycleState::Cancelled))
            }
            SemanticOp::DataIsReady => Some(YirResultState::Data(DataFlowState::Ready)),
            SemanticOp::DataIsMoved => Some(YirResultState::Data(DataFlowState::Moved)),
            SemanticOp::DataIsWindowed => Some(YirResultState::Data(DataFlowState::Windowed)),
            SemanticOp::ShaderIsPassReady => {
                Some(YirResultState::Shader(ShaderFlowState::PassReady))
            }
            SemanticOp::ShaderIsFrameReady => {
                Some(YirResultState::Shader(ShaderFlowState::FrameReady))
            }
            SemanticOp::KernelIsConfigReady => {
                Some(YirResultState::Kernel(KernelFlowState::ConfigReady))
            }
            _ => None,
        }
    }

    pub fn observe_state_matches_source(
        &self,
        source: &Operation,
        state: &str,
    ) -> Result<bool, String> {
        match self.semantic_op() {
            SemanticOp::DataObserve => match source.semantic_op() {
                SemanticOp::DataBindCore | SemanticOp::DataMarker | SemanticOp::DataHandleTable => {
                    Ok(state == "ready")
                }
                SemanticOp::DataOutputPipe => Ok(state == "moved"),
                SemanticOp::DataInputPipe
                | SemanticOp::DataCopyWindow
                | SemanticOp::DataImmutableWindow => Ok(matches!(state, "ready" | "windowed")),
                other => Err(format!(
                    "unsupported data observe source `{}`",
                    semantic_op_display_name(other)
                )),
            },
            SemanticOp::ShaderObserve => {
                let expected = match source.semantic_op() {
                    SemanticOp::ShaderBeginPass => "pass_ready",
                    SemanticOp::ShaderDrawInstanced => "frame_ready",
                    other => {
                        return Err(format!(
                            "unsupported shader observe source `{}`",
                            semantic_op_display_name(other)
                        ))
                    }
                };
                Ok(state == expected)
            }
            SemanticOp::KernelObserve => {
                if source.semantic_op() != SemanticOp::CpuProjectProfileRef {
                    return Ok(false);
                }
                Ok(state == "config_ready")
            }
            _ => Err(format!(
                "operation `{}` does not define an observe-state contract",
                self.full_name()
            )),
        }
    }

    pub fn cpu_llvm_lowering_class(&self) -> CpuLlvmLoweringClass {
        if self.domain_family() != OperationDomainFamily::Cpu {
            return CpuLlvmLoweringClass::NonCpu;
        }
        match self.instruction.as_str() {
            "text" | "const_bool" | "const_i32" | "const" | "const_i64" | "const_f32"
            | "const_f64" | "null" => CpuLlvmLoweringClass::Literal,
            "struct" | "field" => CpuLlvmLoweringClass::Aggregate,
            "borrow" | "borrow_end" | "move_ptr" => CpuLlvmLoweringClass::Pointer,
            "neg" | "add" | "add_i32" | "add_f32" | "add_f64" | "sub" | "sub_i32" | "sub_f32"
            | "sub_f64" | "mul" | "mul_i32" | "mul_f32" | "mul_f64" | "div" | "div_i32"
            | "div_f32" | "div_f64" | "rem" | "madd" | "select" => CpuLlvmLoweringClass::Arithmetic,
            "eq" | "eq_i32" | "eq_f32" | "eq_f64" | "ne" | "lt" | "lt_i32" | "lt_f32"
            | "lt_f64" | "gt" | "gt_i32" | "gt_f32" | "gt_f64" | "le" | "ge" => {
                CpuLlvmLoweringClass::Compare
            }
            "not" | "and" | "or" | "xor" | "shl" | "shr" => CpuLlvmLoweringClass::Bitwise,
            "cast_i32_to_i64" | "cast_i64_to_i32" | "cast_i32_to_f32" | "cast_i32_to_f64"
            | "cast_f32_to_f64" | "cast_f64_to_f32" => CpuLlvmLoweringClass::Cast,
            "alloc_node" | "alloc_buffer" | "load_value" | "load_next" | "buffer_len"
            | "load_at" | "store_value" | "store_next" | "store_at" | "is_null" | "free" => {
                CpuLlvmLoweringClass::Memory
            }
            "input_i64" | "extern_call_i64" => CpuLlvmLoweringClass::Runtime,
            "print" => CpuLlvmLoweringClass::Effect,
            _ if self.is_async_core_op() => {
                CpuLlvmLoweringClass::Effect
            }
            _ => CpuLlvmLoweringClass::Other,
        }
    }
}

fn semantic_op_display_name(op: SemanticOp) -> &'static str {
    let (module, instruction) = match op {
        SemanticOp::CpuProjectProfileRef => ("cpu", "project_profile_ref"),
        SemanticOp::CpuJoinResult => ("cpu", "join_result"),
        SemanticOp::CpuTaskCompleted => ("cpu", "task_completed"),
        SemanticOp::CpuTaskTimedOut => ("cpu", "task_timed_out"),
        SemanticOp::CpuTaskCancelled => ("cpu", "task_cancelled"),
        SemanticOp::CpuTaskValue => ("cpu", "task_value"),
        SemanticOp::DataObserve => ("data", "observe"),
        SemanticOp::DataIsReady => ("data", "is_ready"),
        SemanticOp::DataIsMoved => ("data", "is_moved"),
        SemanticOp::DataIsWindowed => ("data", "is_windowed"),
        SemanticOp::DataValue => ("data", "value"),
        SemanticOp::ShaderObserve => ("shader", "observe"),
        SemanticOp::ShaderIsPassReady => ("shader", "is_pass_ready"),
        SemanticOp::ShaderIsFrameReady => ("shader", "is_frame_ready"),
        SemanticOp::ShaderValue => ("shader", "value"),
        SemanticOp::KernelObserve => ("kernel", "observe"),
        SemanticOp::KernelIsConfigReady => ("kernel", "is_config_ready"),
        SemanticOp::KernelValue => ("kernel", "value"),
        SemanticOp::DataBindCore => ("data", "bind_core"),
        SemanticOp::DataMarker => ("data", "marker"),
        SemanticOp::DataHandleTable => ("data", "handle_table"),
        SemanticOp::DataOutputPipe => ("data", "output_pipe"),
        SemanticOp::DataInputPipe => ("data", "input_pipe"),
        SemanticOp::DataCopyWindow => ("data", "copy_window"),
        SemanticOp::DataImmutableWindow => ("data", "immutable_window"),
        SemanticOp::ShaderBeginPass => ("shader", "begin_pass"),
        SemanticOp::ShaderDrawInstanced => ("shader", "draw_instanced"),
        _ => return "other",
    };
    Operation {
        module: module.to_owned(),
        instruction: instruction.to_owned(),
        args: Vec::new(),
    }
    .semantic_name()
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
    DataResult(DataResultHandle),
    DataMarker(DataMarker),
    DataHandleTable(DataHandleTable),
    DataCoreBinding(DataCoreBinding),
    ShaderResult(ShaderResultHandle),
    KernelResult(KernelResultHandle),
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
    TaskResult(TaskResultHandle),
    Unit,
}

pub fn glm_profile_for_operation(op: &Operation) -> GlmNodeProfile {
    match op.semantic_op() {
        SemanticOp::CpuAllocNode | SemanticOp::CpuAllocBuffer => GlmNodeProfile {
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
        SemanticOp::CpuBorrow => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::CpuBorrowEnd => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::CpuProjectProfileRef | SemanticOp::CpuInstantiateUnit => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: Vec::new(),
            effect: GlmEffect::None,
        },
        SemanticOp::CpuMovePtr => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::DomainMove,
        },
        SemanticOp::CpuLoadValue
        | SemanticOp::CpuLoadNext
        | SemanticOp::CpuBufferLen
        | SemanticOp::CpuLoadAt => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::CpuStoreValue | SemanticOp::CpuStoreNext | SemanticOp::CpuStoreAt => {
            GlmNodeProfile {
                result_class: GlmValueClass::Val,
                accesses: vec![GlmAccess {
                    input: op.args[0].clone(),
                    class: GlmValueClass::Res,
                    mode: GlmUseMode::Write,
                }],
                effect: GlmEffect::None,
            }
        }
        SemanticOp::CpuFree => GlmNodeProfile {
            result_class: GlmValueClass::Val,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::LifetimeEnd,
        },
        SemanticOp::DataMove => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Val,
                mode: GlmUseMode::Own,
            }],
            effect: GlmEffect::DomainMove,
        },
        SemanticOp::DataCopyWindow | SemanticOp::DataImmutableWindow => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        SemanticOp::DataOutputPipe | SemanticOp::DataInputPipe => GlmNodeProfile {
            result_class: GlmValueClass::Res,
            accesses: vec![GlmAccess {
                input: op.args[0].clone(),
                class: GlmValueClass::Res,
                mode: GlmUseMode::Read,
            }],
            effect: GlmEffect::None,
        },
        _ if op.is_async_core_op() => GlmNodeProfile {
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

#[cfg(test)]
mod tests {
    use super::{
        AsyncCoreOp, CpuLlvmLoweringClass, DataFlowState, DataMod, DataResultHandle, DataWindow,
        ExecutionState, GlmEffect, GlmUseMode, GlmValueClass, KernelFlowState,
        KernelResultHandle, Node, Operation, OperationDomainFamily, RegisteredMod, Resource,
        ResourceKind, SemanticOp, ShaderFlowState, ShaderResultHandle, TaskLifecycleState,
        TaskResultHandle, Value, YirResultFamily, YirResultRole, YirResultState,
    };

    #[test]
    fn fabric_ops_fold_into_data_domain_family() {
        let op = Operation::parse("fabric.output_pipe", vec!["frame".to_owned()]).unwrap();
        assert_eq!(op.domain_family(), OperationDomainFamily::Data);
        assert_eq!(op.semantic_op(), SemanticOp::DataOutputPipe);
    }

    #[test]
    fn glm_profile_uses_semantic_op_classification() {
        let op = Operation::parse("cpu.move_ptr", vec!["ptr0".to_owned()]).unwrap();
        let profile = super::glm_profile_for_operation(&op);
        assert_eq!(profile.result_class, GlmValueClass::Res);
        assert_eq!(profile.accesses[0].mode, GlmUseMode::Own);
        assert_eq!(profile.effect, GlmEffect::DomainMove);
    }

    #[test]
    fn classifies_async_primitives_as_yir_core_ops() {
        let async_call = Operation::parse("cpu.async_call", vec!["ping".to_owned()]).unwrap();
        let spawn = Operation::parse(
            "cpu.spawn_task",
            vec!["ping".to_owned(), "async_call_0".to_owned()],
        )
        .unwrap();
        let join_result =
            Operation::parse("cpu.join_result", vec!["task_0".to_owned()]).unwrap();
        let task_value =
            Operation::parse("cpu.task_value", vec!["result_0".to_owned()]).unwrap();

        assert_eq!(async_call.async_core_op(), Some(AsyncCoreOp::ScheduleCall));
        assert_eq!(spawn.async_core_op(), Some(AsyncCoreOp::SpawnTask));
        assert_eq!(join_result.async_core_op(), Some(AsyncCoreOp::ObserveTaskResult));
        assert_eq!(task_value.async_core_op(), Some(AsyncCoreOp::ExtractTaskValue));
        assert_eq!(join_result.result_role(), Some(YirResultRole::Entry));
        assert_eq!(task_value.result_role(), Some(YirResultRole::PayloadExtractor));
        assert!(task_value.is_async_task_result_observer());
    }

    #[test]
    fn lowers_async_primitives_as_effectful_cpu_nodes() {
        let task_value =
            Operation::parse("cpu.task_value", vec!["result_0".to_owned()]).unwrap();
        assert_eq!(
            task_value.cpu_llvm_lowering_class(),
            CpuLlvmLoweringClass::Effect
        );

        let profile = super::glm_profile_for_operation(&task_value);
        assert_eq!(profile.result_class, GlmValueClass::Val);
        assert_eq!(profile.accesses[0].input, "result_0");
    }

    #[test]
    fn exposes_result_family_state_and_payload_from_values() {
        let task = Value::TaskResult(TaskResultHandle {
            label: "ping".to_owned(),
            state: TaskLifecycleState::Completed,
            result: Some(Box::new(Value::Int(7))),
        });
        let data = Value::DataResult(DataResultHandle {
            state: DataFlowState::Windowed,
            value: Box::new(Value::Int(11)),
        });
        let shader = Value::ShaderResult(ShaderResultHandle {
            state: ShaderFlowState::FrameReady,
            value: Box::new(Value::Int(13)),
        });
        let kernel = Value::KernelResult(KernelResultHandle {
            state: KernelFlowState::ConfigReady,
            value: Box::new(Value::Int(17)),
        });

        assert_eq!(task.result_family(), Some(YirResultFamily::Task));
        assert_eq!(
            task.result_state(),
            Some(YirResultState::Task(TaskLifecycleState::Completed))
        );
        assert_eq!(task.result_payload(), Some(&Value::Int(7)));

        assert_eq!(data.result_family(), Some(YirResultFamily::Data));
        assert_eq!(
            data.result_state(),
            Some(YirResultState::Data(DataFlowState::Windowed))
        );
        assert_eq!(data.result_payload(), Some(&Value::Int(11)));

        assert_eq!(shader.result_family(), Some(YirResultFamily::Shader));
        assert_eq!(
            shader.result_state(),
            Some(YirResultState::Shader(ShaderFlowState::FrameReady))
        );
        assert_eq!(shader.result_payload(), Some(&Value::Int(13)));

        assert_eq!(kernel.result_family(), Some(YirResultFamily::Kernel));
        assert_eq!(
            kernel.result_state(),
            Some(YirResultState::Kernel(KernelFlowState::ConfigReady))
        );
        assert_eq!(kernel.result_payload(), Some(&Value::Int(17)));
    }

    #[test]
    fn validates_observe_states_via_core_contract() {
        let data_observe =
            Operation::parse("data.observe", vec!["pipe".to_owned(), "moved".to_owned()]).unwrap();
        let data_source = Operation::parse("data.output_pipe", vec!["payload".to_owned()]).unwrap();
        assert!(data_observe
            .observe_state_matches_source(&data_source, "moved")
            .unwrap());

        let shader_observe = Operation::parse(
            "shader.observe",
            vec!["draw".to_owned(), "frame_ready".to_owned()],
        )
        .unwrap();
        let shader_source =
            Operation::parse("shader.draw_instanced", vec!["pass".to_owned()]).unwrap();
        assert!(shader_observe
            .observe_state_matches_source(&shader_source, "frame_ready")
            .unwrap());

        let kernel_observe = Operation::parse(
            "kernel.observe",
            vec!["profile".to_owned(), "config_ready".to_owned()],
        )
        .unwrap();
        let kernel_source = Operation::parse(
            "cpu.project_profile_ref",
            vec!["kernel".to_owned(), "KernelUnit".to_owned(), "queue_depth".to_owned()],
        )
        .unwrap();
        assert!(kernel_observe
            .observe_state_matches_source(&kernel_source, "config_ready")
            .unwrap());
    }

    #[test]
    fn exposes_result_probe_states_for_state_helpers() {
        let task_completed =
            Operation::parse("cpu.task_completed", vec!["result_0".to_owned()]).unwrap();
        let shader_ready =
            Operation::parse("shader.is_frame_ready", vec!["shader_result".to_owned()]).unwrap();
        let data_moved =
            Operation::parse("data.is_moved", vec!["data_result".to_owned()]).unwrap();

        assert_eq!(task_completed.result_role(), Some(YirResultRole::StateProbe));
        assert_eq!(
            task_completed.result_probe_state(),
            Some(YirResultState::Task(TaskLifecycleState::Completed))
        );
        assert_eq!(
            shader_ready.result_probe_state(),
            Some(YirResultState::Shader(ShaderFlowState::FrameReady))
        );
        assert_eq!(
            data_moved.result_probe_state(),
            Some(YirResultState::Data(DataFlowState::Moved))
        );
    }

    #[test]
    fn freeing_live_link_target_is_rejected_in_execution_state() {
        let mut state = ExecutionState::default();
        let tail = state.alloc_heap_node(20, None);
        let _head = state.alloc_heap_node(10, Some(tail));

        let error = state.free_heap_node(Some(tail)).unwrap_err();
        assert!(error.contains("still links to it"));
    }

    #[test]
    fn data_mod_rejects_nested_window_payloads() {
        let resource = Resource {
            name: "fabric0".to_owned(),
            kind: ResourceKind::parse("data.fabric"),
        };
        let data_mod = DataMod;
        let mut state = ExecutionState::default();

        state
            .values
            .insert("base".to_owned(), Value::Int(7));
        let first = data_mod
            .execute(
                &Node {
                    name: "window0".to_owned(),
                    resource: "fabric0".to_owned(),
                    op: Operation::parse(
                        "data.immutable_window",
                        vec!["base".to_owned(), "0".to_owned(), "1".to_owned()],
                    )
                    .unwrap(),
                },
                &resource,
                &mut state,
            )
            .unwrap();
        state.values.insert("window0".to_owned(), first);

        let error = data_mod
            .execute(
                &Node {
                    name: "window1".to_owned(),
                    resource: "fabric0".to_owned(),
                    op: Operation::parse(
                        "data.copy_window",
                        vec!["window0".to_owned(), "0".to_owned(), "1".to_owned()],
                    )
                    .unwrap(),
                },
                &resource,
                &mut state,
        )
            .unwrap_err();
        assert!(error.contains("cannot wrap non-window-compatible payload"));
    }

    #[test]
    fn data_mod_rejects_mutable_window_payloads_for_output_pipe() {
        let resource = Resource {
            name: "fabric0".to_owned(),
            kind: ResourceKind::parse("data.fabric"),
        };
        let data_mod = DataMod;
        let mut state = ExecutionState::default();

        state.values.insert(
            "window0".to_owned(),
            Value::DataWindow(DataWindow {
                base: Box::new(Value::Int(7)),
                offset: 0,
                len: 1,
                immutable: false,
            }),
        );

        let error = data_mod
            .execute(
                &Node {
                    name: "pipe".to_owned(),
                    resource: "fabric0".to_owned(),
                    op: Operation::parse("data.output_pipe", vec!["window0".to_owned()]).unwrap(),
                },
                &resource,
                &mut state,
            )
            .unwrap_err();
        assert!(error.contains("illegal pipe payload"));
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
    pub state: TaskLifecycleState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskResultHandle {
    pub label: String,
    pub state: TaskLifecycleState,
    pub result: Option<Box<Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskLifecycleState {
    Pending,
    Completed,
    TimedOut,
    Cancelled,
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
            Self::DataResult(result) => write!(f, "{result}"),
            Self::DataMarker(marker) => write!(f, "{marker}"),
            Self::DataHandleTable(table) => write!(f, "{table}"),
            Self::DataCoreBinding(binding) => write!(f, "{binding}"),
            Self::ShaderResult(result) => write!(f, "{result}"),
            Self::KernelResult(result) => write!(f, "{result}"),
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
            Self::Task(task) => match task.limit {
                Some(limit) => {
                    if matches!(task.state, TaskLifecycleState::Cancelled) {
                        write!(f, "task<{}; cancelled; limit={limit}>", task.label)
                    } else {
                        write!(f, "task<{}; limit={limit}>", task.label)
                    }
                }
                None if matches!(task.state, TaskLifecycleState::Cancelled) => {
                    write!(f, "task<cancelled; {}>", task.label)
                }
                None => write!(f, "task<{}>", task.label),
            },
            Self::TaskResult(result) => write!(f, "task_result<{}:{}>", result.label, result.state),
            Self::Unit => write!(f, "()"),
        }
    }
}

impl fmt::Display for TaskLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("pending"),
            Self::Completed => f.write_str("completed"),
            Self::TimedOut => f.write_str("timed_out"),
            Self::Cancelled => f.write_str("cancelled"),
        }
    }
}

impl Value {
    pub fn result_family(&self) -> Option<YirResultFamily> {
        match self {
            Self::TaskResult(_) => Some(YirResultFamily::Task),
            Self::DataResult(_) => Some(YirResultFamily::Data),
            Self::ShaderResult(_) => Some(YirResultFamily::Shader),
            Self::KernelResult(_) => Some(YirResultFamily::Kernel),
            _ => None,
        }
    }

    pub fn result_state(&self) -> Option<YirResultState> {
        match self {
            Self::TaskResult(result) => Some(YirResultState::Task(result.state)),
            Self::DataResult(result) => Some(YirResultState::Data(result.state)),
            Self::ShaderResult(result) => Some(YirResultState::Shader(result.state)),
            Self::KernelResult(result) => Some(YirResultState::Kernel(result.state)),
            _ => None,
        }
    }

    pub fn result_payload(&self) -> Option<&Value> {
        match self {
            Self::TaskResult(result) => result.result.as_deref(),
            Self::DataResult(result) => Some(result.value.as_ref()),
            Self::ShaderResult(result) => Some(result.value.as_ref()),
            Self::KernelResult(result) => Some(result.value.as_ref()),
            _ => None,
        }
    }
}

impl fmt::Display for DataResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "data_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for DataFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ready => f.write_str("ready"),
            Self::Moved => f.write_str("moved"),
            Self::Windowed => f.write_str("windowed"),
        }
    }
}

impl fmt::Display for ShaderResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shader_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for ShaderFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PassReady => f.write_str("pass_ready"),
            Self::FrameReady => f.write_str("frame_ready"),
        }
    }
}

impl fmt::Display for KernelResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kernel_result<{}:{}>", self.state, self.value)
    }
}

impl fmt::Display for KernelFlowState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigReady => f.write_str("config_ready"),
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
            Some(Value::TaskResult(_)) => Err(format!("`{name}` is task-result, expected int")),
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
            "observe" => {
                if node.op.args.len() != 2 {
                    return Err(format!(
                        "node `{}` expects `data.observe <name> <resource> <input> <state>`",
                        node.name
                    ));
                }
                parse_data_flow_state(&node.op.args[1]).map_err(|error| {
                    format!("node `{}` has invalid data observe state: {error}", node.name)
                })?;
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
            }
            "is_ready" | "is_moved" | "is_windowed" | "value" => {
                if node.op.args.len() != 1 {
                    return Err(format!(
                        "node `{}` expects `data.{} <name> <resource> <result>`",
                        node.name, node.op.instruction
                    ));
                }
                Ok(InstructionSemantics::pure(vec![node.op.args[0].clone()]))
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
            "observe" => {
                let value = state.expect_value(&node.op.args[0])?.clone();
                let flow = parse_data_flow_state(&node.op.args[1])?;
                Ok(Value::DataResult(DataResultHandle {
                    state: flow,
                    value: Box::new(value),
                }))
            }
            "is_ready" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(result.state, DataFlowState::Ready)))
            }
            "is_moved" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(result.state, DataFlowState::Moved)))
            }
            "is_windowed" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok(Value::Bool(matches!(result.state, DataFlowState::Windowed)))
            }
            "value" => {
                let result = state.expect_data_result(&node.op.args[0])?;
                Ok((*result.value).clone())
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

fn parse_data_flow_state(raw: &str) -> Result<DataFlowState, String> {
    match raw {
        "ready" => Ok(DataFlowState::Ready),
        "moved" => Ok(DataFlowState::Moved),
        "windowed" => Ok(DataFlowState::Windowed),
        other => Err(format!("unknown data flow state `{other}`")),
    }
}

fn is_move_value_legal(value: &Value) -> bool {
    match value {
        Value::DataWindow(_)
        | Value::DataPipe(_)
        | Value::DataResult(_)
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
        Value::DataWindow(_)
        | Value::DataHandleTable(_)
        | Value::DataMarker(_)
        | Value::DataPipe(_)
        | Value::DataResult(_) => false,
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
        Value::DataWindow(window) if !window.immutable => false,
        Value::DataPipe(_) | Value::DataResult(_) => false,
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
