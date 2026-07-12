use crate::{
    AsyncCoreOp, CpuLlvmLoweringClass, DataFlowState, DataResultHandle, GlmBridgeObjectKind,
    GlmEffect, GlmSketchValueClass, GlmUseMode, GlmValueClass, KernelFlowState, KernelResultHandle,
    NetworkFlowState, NetworkResultHandle, Operation, OperationDomainFamily, SemanticOp,
    ShaderFlowState, ShaderResultHandle, TaskLifecycleState, TaskResultHandle, Value,
    YirResultFamily, YirResultRole, YirResultState,
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
    let profile = crate::glm_profile_for_operation(&op);
    assert_eq!(profile.result_class, GlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, GlmUseMode::Own);
    assert_eq!(profile.effect, GlmEffect::DomainMove);
}

#[test]
fn exposes_bridge_object_sketch_names_without_changing_live_glm_classes() {
    assert_eq!(GlmSketchValueClass::Bridge.to_string(), "bridge");
    assert_eq!(
        GlmBridgeObjectKind::TaskExternalHandle.to_string(),
        "task-external-handle"
    );
}

#[test]
fn classifies_async_primitives_as_yir_core_ops() {
    let async_call = Operation::parse("cpu.async_call", vec!["ping".to_owned()]).unwrap();
    let spawn = Operation::parse(
        "cpu.spawn_task",
        vec!["ping".to_owned(), "async_call_0".to_owned()],
    )
    .unwrap();
    let join_result = Operation::parse("cpu.join_result", vec!["task_0".to_owned()]).unwrap();
    let task_value = Operation::parse("cpu.task_value", vec!["result_0".to_owned()]).unwrap();

    assert_eq!(async_call.async_core_op(), Some(AsyncCoreOp::ScheduleCall));
    assert_eq!(spawn.async_core_op(), Some(AsyncCoreOp::SpawnTask));
    assert_eq!(
        join_result.async_core_op(),
        Some(AsyncCoreOp::ObserveTaskResult)
    );
    assert_eq!(
        task_value.async_core_op(),
        Some(AsyncCoreOp::ExtractTaskValue)
    );
    assert_eq!(join_result.result_role(), Some(YirResultRole::Entry));
    assert_eq!(
        task_value.result_role(),
        Some(YirResultRole::PayloadExtractor)
    );
    assert!(task_value.is_async_task_result_observer());
}

#[test]
fn lowers_async_primitives_as_effectful_cpu_nodes() {
    let task_value = Operation::parse("cpu.task_value", vec!["result_0".to_owned()]).unwrap();
    assert_eq!(
        task_value.cpu_llvm_lowering_class(),
        CpuLlvmLoweringClass::Effect
    );

    let profile = crate::glm_profile_for_operation(&task_value);
    assert_eq!(profile.result_class, GlmValueClass::Val);
    assert_eq!(profile.accesses[0].input, "result_0");
}

#[test]
fn glm_profiles_task_observation_ops_as_val_reads() {
    let join_result = Operation::parse("cpu.join_result", vec!["task_0".to_owned()]).unwrap();
    let task_completed =
        Operation::parse("cpu.task_completed", vec!["result_0".to_owned()]).unwrap();
    let task_value = Operation::parse("cpu.task_value", vec!["result_0".to_owned()]).unwrap();

    let join_result_profile = crate::glm_profile_for_operation(&join_result);
    assert_eq!(join_result_profile.result_class, GlmValueClass::Val);
    assert_eq!(join_result_profile.accesses.len(), 1);
    assert_eq!(join_result_profile.accesses[0].input, "task_0");
    assert_eq!(join_result_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(join_result_profile.accesses[0].mode, GlmUseMode::Read);

    let task_completed_profile = crate::glm_profile_for_operation(&task_completed);
    assert_eq!(task_completed_profile.result_class, GlmValueClass::Val);
    assert_eq!(task_completed_profile.accesses.len(), 1);
    assert_eq!(task_completed_profile.accesses[0].input, "result_0");
    assert_eq!(task_completed_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(task_completed_profile.accesses[0].mode, GlmUseMode::Read);

    let task_value_profile = crate::glm_profile_for_operation(&task_value);
    assert_eq!(task_value_profile.result_class, GlmValueClass::Val);
    assert_eq!(task_value_profile.accesses.len(), 1);
    assert_eq!(task_value_profile.accesses[0].input, "result_0");
    assert_eq!(task_value_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(task_value_profile.accesses[0].mode, GlmUseMode::Read);
}

#[test]
fn glm_profiles_spawn_path_as_val_reads_not_task_resource_origin() {
    let async_call = Operation::parse("cpu.async_call", vec!["ping".to_owned()]).unwrap();
    let spawn = Operation::parse(
        "cpu.spawn_task",
        vec!["ping".to_owned(), "async_call_0".to_owned()],
    )
    .unwrap();

    let async_call_profile = crate::glm_profile_for_operation(&async_call);
    assert_eq!(async_call_profile.result_class, GlmValueClass::Val);
    assert_eq!(async_call_profile.accesses.len(), 1);
    assert_eq!(async_call_profile.accesses[0].input, "ping");
    assert_eq!(async_call_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(async_call_profile.accesses[0].mode, GlmUseMode::Read);
    assert_eq!(async_call_profile.effect, GlmEffect::None);

    let spawn_profile = crate::glm_profile_for_operation(&spawn);
    assert_eq!(spawn_profile.result_class, GlmValueClass::Val);
    assert_eq!(spawn_profile.accesses.len(), 2);
    assert_eq!(spawn_profile.accesses[0].input, "ping");
    assert_eq!(spawn_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(spawn_profile.accesses[0].mode, GlmUseMode::Read);
    assert_eq!(spawn_profile.accesses[1].input, "async_call_0");
    assert_eq!(spawn_profile.accesses[1].class, GlmValueClass::Val);
    assert_eq!(spawn_profile.accesses[1].mode, GlmUseMode::Read);
    assert_eq!(spawn_profile.effect, GlmEffect::None);
}

#[test]
fn glm_profiles_direct_join_as_val_read_not_own_consume() {
    let join = Operation::parse("cpu.join", vec!["task_0".to_owned()]).unwrap();
    let profile = crate::glm_profile_for_operation(&join);

    assert_eq!(profile.result_class, GlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].input, "task_0");
    assert_eq!(profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(profile.accesses[0].mode, GlmUseMode::Read);
    assert_eq!(profile.effect, GlmEffect::None);
}

#[test]
fn glm_profiles_cancel_and_timeout_as_val_reads_not_lifetime_end() {
    let cancel = Operation::parse("cpu.cancel", vec!["task_0".to_owned()]).unwrap();
    let timeout = Operation::parse(
        "cpu.timeout",
        vec!["task_0".to_owned(), "limit_0".to_owned()],
    )
    .unwrap();

    let cancel_profile = crate::glm_profile_for_operation(&cancel);
    assert_eq!(cancel_profile.result_class, GlmValueClass::Val);
    assert_eq!(cancel_profile.accesses.len(), 1);
    assert_eq!(cancel_profile.accesses[0].input, "task_0");
    assert_eq!(cancel_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(cancel_profile.accesses[0].mode, GlmUseMode::Read);
    assert_eq!(cancel_profile.effect, GlmEffect::None);

    let timeout_profile = crate::glm_profile_for_operation(&timeout);
    assert_eq!(timeout_profile.result_class, GlmValueClass::Val);
    assert_eq!(timeout_profile.accesses.len(), 2);
    assert_eq!(timeout_profile.accesses[0].input, "task_0");
    assert_eq!(timeout_profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(timeout_profile.accesses[0].mode, GlmUseMode::Read);
    assert_eq!(timeout_profile.accesses[1].input, "limit_0");
    assert_eq!(timeout_profile.accesses[1].class, GlmValueClass::Val);
    assert_eq!(timeout_profile.accesses[1].mode, GlmUseMode::Read);
    assert_eq!(timeout_profile.effect, GlmEffect::None);
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
    let network = Value::NetworkResult(NetworkResultHandle {
        state: NetworkFlowState::AcceptReady,
        value: Box::new(Value::Int(19)),
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

    assert_eq!(network.result_family(), Some(YirResultFamily::Network));
    assert_eq!(
        network.result_state(),
        Some(YirResultState::Network(NetworkFlowState::AcceptReady))
    );
    assert_eq!(network.result_payload(), Some(&Value::Int(19)));
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
    let shader_source = Operation::parse("shader.draw_instanced", vec!["pass".to_owned()]).unwrap();
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
        vec![
            "kernel".to_owned(),
            "KernelUnit".to_owned(),
            "queue_depth".to_owned(),
        ],
    )
    .unwrap();
    assert!(kernel_observe
        .observe_state_matches_source(&kernel_source, "config_ready")
        .unwrap());

    let network_connect = Operation::parse(
        "network.connect",
        vec![
            "local_port".to_owned(),
            "remote_port".to_owned(),
            "connect_timeout".to_owned(),
        ],
    )
    .unwrap();
    assert!(network_connect
        .observe_state_matches_source(&network_connect, "connect_ready")
        .unwrap());
}

#[test]
fn exposes_result_probe_states_for_state_helpers() {
    let task_completed =
        Operation::parse("cpu.task_completed", vec!["result_0".to_owned()]).unwrap();
    let shader_ready =
        Operation::parse("shader.is_frame_ready", vec!["shader_result".to_owned()]).unwrap();
    let data_moved = Operation::parse("data.is_moved", vec!["data_result".to_owned()]).unwrap();
    let network_closed =
        Operation::parse("network.is_closed", vec!["network_result".to_owned()]).unwrap();

    assert_eq!(
        task_completed.result_role(),
        Some(YirResultRole::StateProbe)
    );
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
    assert_eq!(
        network_closed.result_probe_state(),
        Some(YirResultState::Network(NetworkFlowState::Closed))
    );
}
