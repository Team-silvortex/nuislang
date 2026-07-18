use super::*;

impl Operation {
    pub fn result_family(&self) -> Option<YirResultFamily> {
        match self.semantic_op() {
            SemanticOp::CpuJoinResult
            | SemanticOp::CpuTaskCompleted
            | SemanticOp::CpuTaskTimedOut
            | SemanticOp::CpuTaskCancelled
            | SemanticOp::CpuTaskFailed
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
            SemanticOp::NetworkObserve
            | SemanticOp::NetworkConnect
            | SemanticOp::NetworkAccept
            | SemanticOp::NetworkClose
            | SemanticOp::NetworkIsConfigReady
            | SemanticOp::NetworkIsConnectReady
            | SemanticOp::NetworkIsAcceptReady
            | SemanticOp::NetworkIsClosed
            | SemanticOp::NetworkValue => Some(YirResultFamily::Network),
            _ => None,
        }
    }

    pub fn result_role(&self) -> Option<YirResultRole> {
        match self.semantic_op() {
            SemanticOp::CpuJoinResult
            | SemanticOp::DataObserve
            | SemanticOp::ShaderObserve
            | SemanticOp::KernelObserve
            | SemanticOp::NetworkObserve
            | SemanticOp::NetworkConnect
            | SemanticOp::NetworkAccept
            | SemanticOp::NetworkClose => Some(YirResultRole::Entry),
            SemanticOp::CpuTaskCompleted
            | SemanticOp::CpuTaskTimedOut
            | SemanticOp::CpuTaskCancelled
            | SemanticOp::CpuTaskFailed
            | SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::ShaderIsPassReady
            | SemanticOp::ShaderIsFrameReady
            | SemanticOp::KernelIsConfigReady
            | SemanticOp::NetworkIsConfigReady
            | SemanticOp::NetworkIsConnectReady
            | SemanticOp::NetworkIsAcceptReady
            | SemanticOp::NetworkIsClosed => Some(YirResultRole::StateProbe),
            SemanticOp::CpuTaskValue
            | SemanticOp::DataValue
            | SemanticOp::ShaderValue
            | SemanticOp::KernelValue
            | SemanticOp::NetworkValue => Some(YirResultRole::PayloadExtractor),
            _ => None,
        }
    }

    pub fn result_source_semantic_op(&self) -> Option<SemanticOp> {
        match self.semantic_op() {
            SemanticOp::CpuTaskCompleted
            | SemanticOp::CpuTaskTimedOut
            | SemanticOp::CpuTaskCancelled
            | SemanticOp::CpuTaskFailed
            | SemanticOp::CpuTaskValue => Some(SemanticOp::CpuJoinResult),
            SemanticOp::DataIsReady
            | SemanticOp::DataIsMoved
            | SemanticOp::DataIsWindowed
            | SemanticOp::DataValue => Some(SemanticOp::DataObserve),
            SemanticOp::ShaderIsPassReady
            | SemanticOp::ShaderIsFrameReady
            | SemanticOp::ShaderValue => Some(SemanticOp::ShaderObserve),
            SemanticOp::KernelIsConfigReady | SemanticOp::KernelValue => {
                Some(SemanticOp::KernelObserve)
            }
            SemanticOp::NetworkIsConfigReady
            | SemanticOp::NetworkIsSendReady
            | SemanticOp::NetworkIsRecvReady
            | SemanticOp::NetworkIsAcceptReady
            | SemanticOp::NetworkValue => Some(SemanticOp::NetworkObserve),
            SemanticOp::NetworkIsConnectReady => Some(SemanticOp::NetworkConnect),
            SemanticOp::NetworkIsClosed => Some(SemanticOp::NetworkClose),
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
            SemanticOp::CpuTaskFailed => Some(YirResultState::Task(TaskLifecycleState::Failed)),
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
            SemanticOp::NetworkIsConfigReady => {
                Some(YirResultState::Network(NetworkFlowState::ConfigReady))
            }
            SemanticOp::NetworkIsSendReady => {
                Some(YirResultState::Network(NetworkFlowState::SendReady))
            }
            SemanticOp::NetworkIsRecvReady => {
                Some(YirResultState::Network(NetworkFlowState::RecvReady))
            }
            SemanticOp::NetworkIsConnectReady => {
                Some(YirResultState::Network(NetworkFlowState::ConnectReady))
            }
            SemanticOp::NetworkIsAcceptReady => {
                Some(YirResultState::Network(NetworkFlowState::AcceptReady))
            }
            SemanticOp::NetworkIsClosed => Some(YirResultState::Network(NetworkFlowState::Closed)),
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
                | SemanticOp::DataReadWindow
                | SemanticOp::DataWriteWindow
                | SemanticOp::DataFreezeWindow
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
                let direct_project_ref = source.semantic_op() == SemanticOp::CpuProjectProfileRef;
                let direct_kernel_scalar_source = source.module == "kernel"
                    && matches!(
                        source.instruction.as_str(),
                        "reduce_sum" | "reduce_max" | "reduce_mean" | "argmax" | "argmin"
                    );
                if !direct_project_ref && !direct_kernel_scalar_source {
                    return Ok(false);
                }
                Ok(state == "config_ready")
            }
            SemanticOp::NetworkObserve => {
                let direct_project_ref = source.semantic_op() == SemanticOp::CpuProjectProfileRef;
                let host_transport_probe = source.module == "cpu"
                    && matches!(
                        source.instruction.as_str(),
                        "extern_call_i64" | "extern_call_i32"
                    )
                    && source.args.len() >= 2
                    && matches!(
                        source.args[1].as_str(),
                        "host_network_send_probe"
                            | "host_network_send_owned"
                            | "host_network_recv_probe"
                            | "host_network_recv_owned"
                            | "host_network_accept_probe"
                            | "host_network_accept_owned"
                    );
                if direct_project_ref {
                    return Ok(state == "config_ready");
                }
                if host_transport_probe {
                    let expected = match source.args[1].as_str() {
                        "host_network_send_probe" => "send_ready",
                        "host_network_send_owned" => "send_ready",
                        "host_network_recv_probe" => "recv_ready",
                        "host_network_recv_owned" => "recv_ready",
                        "host_network_accept_probe" => "accept_ready",
                        "host_network_accept_owned" => "accept_ready",
                        _ => return Ok(false),
                    };
                    return Ok(state == expected);
                }
                Ok(false)
            }
            SemanticOp::NetworkConnect => Ok(state == "connect_ready"),
            SemanticOp::NetworkAccept => Ok(state == "accept_ready"),
            SemanticOp::NetworkClose => Ok(state == "closed"),
            _ => Err(format!(
                "operation `{}` does not define an observe-state contract",
                self.full_name()
            )),
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
        SemanticOp::CpuTaskFailed => ("cpu", "task_failed"),
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
        SemanticOp::NetworkObserve => ("network", "observe"),
        SemanticOp::NetworkConnect => ("network", "connect"),
        SemanticOp::NetworkAccept => ("network", "accept"),
        SemanticOp::NetworkClose => ("network", "close"),
        SemanticOp::NetworkIsConfigReady => ("network", "is_config_ready"),
        SemanticOp::NetworkIsSendReady => ("network", "is_send_ready"),
        SemanticOp::NetworkIsRecvReady => ("network", "is_recv_ready"),
        SemanticOp::NetworkIsConnectReady => ("network", "is_connect_ready"),
        SemanticOp::NetworkIsAcceptReady => ("network", "is_accept_ready"),
        SemanticOp::NetworkIsClosed => ("network", "is_closed"),
        SemanticOp::NetworkValue => ("network", "value"),
        SemanticOp::DataBindCore => ("data", "bind_core"),
        SemanticOp::DataMarker => ("data", "marker"),
        SemanticOp::DataHandleTable => ("data", "handle_table"),
        SemanticOp::DataOutputPipe => ("data", "output_pipe"),
        SemanticOp::DataInputPipe => ("data", "input_pipe"),
        SemanticOp::DataCopyWindow => ("data", "copy_window"),
        SemanticOp::DataReadWindow => ("data", "read_window"),
        SemanticOp::DataWriteWindow => ("data", "write_window"),
        SemanticOp::DataFreezeWindow => ("data", "freeze_window"),
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
