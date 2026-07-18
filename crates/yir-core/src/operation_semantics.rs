use super::*;

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
            "network" => OperationDomainFamily::Network,
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
            (OperationDomainFamily::Cpu, "join") | (OperationDomainFamily::Cpu, "thread_join") => {
                SemanticOp::CpuJoin
            }
            (OperationDomainFamily::Cpu, "cancel") => SemanticOp::CpuCancel,
            (OperationDomainFamily::Cpu, "timeout") => SemanticOp::CpuTimeout,
            (OperationDomainFamily::Cpu, "ready_after") => SemanticOp::CpuReadyAfter,
            (OperationDomainFamily::Cpu, "join_result")
            | (OperationDomainFamily::Cpu, "thread_join_result") => SemanticOp::CpuJoinResult,
            (OperationDomainFamily::Cpu, "task_completed") => SemanticOp::CpuTaskCompleted,
            (OperationDomainFamily::Cpu, "task_timed_out") => SemanticOp::CpuTaskTimedOut,
            (OperationDomainFamily::Cpu, "task_cancelled") => SemanticOp::CpuTaskCancelled,
            (OperationDomainFamily::Cpu, "task_failed") => SemanticOp::CpuTaskFailed,
            (OperationDomainFamily::Cpu, "task_value") => SemanticOp::CpuTaskValue,
            (OperationDomainFamily::Data, "move") => SemanticOp::DataMove,
            (OperationDomainFamily::Data, "copy_window") => SemanticOp::DataCopyWindow,
            (OperationDomainFamily::Data, "read_window") => SemanticOp::DataReadWindow,
            (OperationDomainFamily::Data, "write_window") => SemanticOp::DataWriteWindow,
            (OperationDomainFamily::Data, "freeze_window") => SemanticOp::DataFreezeWindow,
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
            (OperationDomainFamily::Network, "observe") => SemanticOp::NetworkObserve,
            (OperationDomainFamily::Network, "connect") => SemanticOp::NetworkConnect,
            (OperationDomainFamily::Network, "accept") => SemanticOp::NetworkAccept,
            (OperationDomainFamily::Network, "close") => SemanticOp::NetworkClose,
            (OperationDomainFamily::Network, "is_config_ready") => SemanticOp::NetworkIsConfigReady,
            (OperationDomainFamily::Network, "is_send_ready") => SemanticOp::NetworkIsSendReady,
            (OperationDomainFamily::Network, "is_recv_ready") => SemanticOp::NetworkIsRecvReady,
            (OperationDomainFamily::Network, "is_connect_ready") => {
                SemanticOp::NetworkIsConnectReady
            }
            (OperationDomainFamily::Network, "is_accept_ready") => SemanticOp::NetworkIsAcceptReady,
            (OperationDomainFamily::Network, "is_closed") => SemanticOp::NetworkIsClosed,
            (OperationDomainFamily::Network, "value") => SemanticOp::NetworkValue,
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
            SemanticOp::DataCopyWindow
                | SemanticOp::DataReadWindow
                | SemanticOp::DataWriteWindow
                | SemanticOp::DataFreezeWindow
                | SemanticOp::DataImmutableWindow
        )
    }

    pub fn data_fabric_primitive(&self) -> Option<DataFabricPrimitive> {
        match self.semantic_op() {
            SemanticOp::DataBindCore => Some(DataFabricPrimitive::Bind),
            SemanticOp::DataHandleTable => Some(DataFabricPrimitive::Handle),
            SemanticOp::DataMarker => Some(DataFabricPrimitive::Marker),
            SemanticOp::DataMove => Some(DataFabricPrimitive::Move),
            SemanticOp::DataCopyWindow
            | SemanticOp::DataReadWindow
            | SemanticOp::DataWriteWindow
            | SemanticOp::DataFreezeWindow
            | SemanticOp::DataImmutableWindow => Some(DataFabricPrimitive::Window),
            SemanticOp::DataOutputPipe | SemanticOp::DataInputPipe => {
                Some(DataFabricPrimitive::Pipe)
            }
            SemanticOp::DataObserve
            | SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::DataValue => Some(DataFabricPrimitive::Observe),
            _ => None,
        }
    }

    pub fn async_core_op(&self) -> Option<AsyncCoreOp> {
        match self.semantic_op() {
            SemanticOp::CpuJoin => Some(AsyncCoreOp::JoinTask),
            SemanticOp::CpuCancel => Some(AsyncCoreOp::CancelTask),
            SemanticOp::CpuTimeout => Some(AsyncCoreOp::TimeoutTask),
            SemanticOp::CpuReadyAfter => Some(AsyncCoreOp::DelayTask),
            SemanticOp::CpuJoinResult => Some(AsyncCoreOp::ObserveTaskResult),
            SemanticOp::CpuTaskCompleted => Some(AsyncCoreOp::ProbeTaskCompleted),
            SemanticOp::CpuTaskTimedOut => Some(AsyncCoreOp::ProbeTaskTimedOut),
            SemanticOp::CpuTaskCancelled => Some(AsyncCoreOp::ProbeTaskCancelled),
            SemanticOp::CpuTaskFailed => Some(AsyncCoreOp::ProbeTaskFailed),
            SemanticOp::CpuTaskValue => Some(AsyncCoreOp::ExtractTaskValue),
            _ if self.domain_family() == OperationDomainFamily::Cpu => {
                match self.instruction.as_str() {
                    "await" => Some(AsyncCoreOp::Await),
                    "async_call" => Some(AsyncCoreOp::ScheduleCall),
                    "spawn_task" | "spawn_thread" | "thread_spawn" => Some(AsyncCoreOp::SpawnTask),
                    _ => None,
                }
            }
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
                    | AsyncCoreOp::ProbeTaskFailed
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
            SemanticOp::CpuTaskFailed => "cpu.task_failed",
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
            SemanticOp::NetworkObserve => "network.observe",
            SemanticOp::NetworkConnect => "network.connect",
            SemanticOp::NetworkAccept => "network.accept",
            SemanticOp::NetworkClose => "network.close",
            SemanticOp::NetworkIsConfigReady => "network.is_config_ready",
            SemanticOp::NetworkIsSendReady => "network.is_send_ready",
            SemanticOp::NetworkIsRecvReady => "network.is_recv_ready",
            SemanticOp::NetworkIsConnectReady => "network.is_connect_ready",
            SemanticOp::NetworkIsAcceptReady => "network.is_accept_ready",
            SemanticOp::NetworkIsClosed => "network.is_closed",
            SemanticOp::NetworkValue => "network.value",
            SemanticOp::DataBindCore => "data.bind_core",
            SemanticOp::DataMarker => "data.marker",
            SemanticOp::DataHandleTable => "data.handle_table",
            SemanticOp::DataOutputPipe => "data.output_pipe",
            SemanticOp::DataInputPipe => "data.input_pipe",
            SemanticOp::DataCopyWindow => "data.copy_window",
            SemanticOp::DataReadWindow => "data.read_window",
            SemanticOp::DataWriteWindow => "data.write_window",
            SemanticOp::DataFreezeWindow => "data.freeze_window",
            SemanticOp::DataImmutableWindow => "data.immutable_window",
            SemanticOp::ShaderBeginPass => "shader.begin_pass",
            SemanticOp::ShaderDrawInstanced => "shader.draw_instanced",
            _ => "other",
        }
    }

    pub fn cpu_llvm_lowering_class(&self) -> CpuLlvmLoweringClass {
        if self.domain_family() != OperationDomainFamily::Cpu {
            return CpuLlvmLoweringClass::NonCpu;
        }
        match self.instruction.as_str() {
            "text" | "const_bool" | "const_i32" | "const" | "const_i64" | "const_f32"
            | "const_f64" | "null" => CpuLlvmLoweringClass::Literal,
            "struct" | "field" | "variant_is" | "variant_field" | "async_value" => {
                CpuLlvmLoweringClass::Aggregate
            }
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
            "input_i64" | "extern_call_i64" | "extern_call_i32" | "param_bool" | "param_i32"
            | "param_i64" | "call_bool" | "call_i32" | "call_i64" | "call_owned_struct" => {
                CpuLlvmLoweringClass::Runtime
            }
            "print"
            | "return_bool"
            | "return_i32"
            | "return_i64"
            | "return_owned_struct"
            | "loop_while_i64"
            | "loop_while_i64_chain"
            | "loop_while_scalar_chain"
            | "loop_while_i64_async_chain"
            | "loop_while_scalar_async_chain"
            | "loop_while_i64_cond_chain"
            | "loop_while_scalar_cond_chain"
            | "loop_while_i64_async_cond_chain"
            | "loop_while_scalar_async_cond_chain"
            | "loop_while_i64_flow_chain"
            | "loop_while_scalar_flow_chain"
            | "loop_while_i64_async_flow_chain"
            | "loop_while_scalar_async_flow_chain"
            | "loop_while_i64_flow_cond_chain"
            | "loop_while_scalar_flow_cond_chain"
            | "loop_while_i64_async_flow_cond_chain"
            | "loop_while_scalar_async_flow_cond_chain"
            | "loop_while_i64_post_flow_chain"
            | "loop_while_scalar_post_flow_chain"
            | "loop_while_i64_async_post_flow_chain"
            | "loop_while_scalar_async_post_flow_chain"
            | "loop_while_i64_post_flow_cond_chain"
            | "loop_while_scalar_post_flow_cond_chain"
            | "loop_while_i64_async_post_flow_cond_chain"
            | "loop_while_scalar_async_post_flow_cond_chain"
            | "guard_print"
            | "guard_return"
            | "guard_host_call_return"
            | "guard_print_return"
            | "branch_print_return"
            | "branch_host_call_return" => CpuLlvmLoweringClass::Effect,
            _ if self.is_async_core_op() => CpuLlvmLoweringClass::Effect,
            _ => CpuLlvmLoweringClass::Other,
        }
    }
}
