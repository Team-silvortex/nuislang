use super::{
    nir_expr_effect_class, nir_host_read_surface, nir_host_scheduler_bridge, NirAddressClass,
    NirContainerKind, NirDataFlowState, NirExpr, NirExprEffectClass, NirHostReadSurface,
    NirHostSchedulerBridge, NirHostSchedulerBridgeKind, NirHostTimingBridge, NirKernelFlowState,
    NirResultFamily, NirResultStage, NirShaderFlowState, NirTypeRef, NirTypeShape, NirWindowMode,
    TestClockDomain,
};

fn named(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

fn generic(name: &str, arg: NirTypeRef) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: vec![arg],
        is_optional: false,
        is_ref: false,
    }
}

fn address(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: true,
    }
}

#[test]
fn rejects_moved_data_state_with_non_pipe_payload() {
    let error = NirDataFlowState::Moved
        .validate_payload(&named("i64"))
        .unwrap_err();
    assert!(error.contains("moved"));
    assert!(error.contains("Pipe<...>"));
}

#[test]
fn result_stage_reports_owning_family() {
    assert_eq!(
        NirResultStage::from(NirDataFlowState::Windowed).family(),
        NirResultFamily::Data
    );
    assert_eq!(
        NirResultStage::from(NirShaderFlowState::FrameReady).family(),
        NirResultFamily::Shader
    );
    assert_eq!(
        NirResultStage::from(NirKernelFlowState::ConfigReady).family(),
        NirResultFamily::Kernel
    );
    assert!(NirResultFamily::Data.supports_stage(NirDataFlowState::Ready.into()));
    assert!(!NirResultFamily::Data.supports_stage(NirShaderFlowState::PassReady.into()));
}

#[test]
fn rejects_windowed_data_state_with_non_window_payload() {
    let error = NirDataFlowState::Windowed
        .validate_payload(&generic("Pipe", named("i64")))
        .unwrap_err();
    assert!(error.contains("windowed"));
    assert!(error.contains("Window<...>"));
}

#[test]
fn tracks_window_mutability_in_type_metadata() {
    let immutable = generic("Window", named("i64"));
    let mutable = generic("WindowMut", named("i64"));

    assert_eq!(immutable.window_mode(), Some(NirWindowMode::Immutable));
    assert_eq!(mutable.window_mode(), Some(NirWindowMode::Mutable));
    assert_eq!(immutable.container_kind(), Some(NirContainerKind::Window));
    assert_eq!(mutable.container_kind(), Some(NirContainerKind::Window));
    immutable.validate_container_contract().unwrap();
    mutable.validate_container_contract().unwrap();
}

#[test]
fn recognizes_staged_thread_and_mutex_families() {
    let thread = generic("Thread", named("i64"));
    let mutex = generic("Mutex", named("i64"));
    let guard = generic("MutexGuard", named("i64"));

    assert!(thread.is_thread_family());
    assert!(mutex.is_mutex_family());
    assert!(guard.is_mutex_guard_family());
    assert!(thread.is_concurrency_bridge_family());
    assert!(mutex.is_concurrency_bridge_family());
    assert!(guard.is_concurrency_bridge_family());
    assert!(!thread.is_async_boundary_safe());
    assert!(!mutex.is_async_boundary_safe());
    assert!(!guard.is_async_boundary_safe());
    thread.validate_container_contract().unwrap();
    mutex.validate_container_contract().unwrap();
    guard.validate_container_contract().unwrap();
}

#[test]
fn rejects_thread_payloads_that_are_not_staged_join_safe() {
    let error = generic("Thread", generic("TaskResult", named("i64")))
        .validate_container_contract()
        .unwrap_err();
    assert!(error.contains("Thread<...>"));
    assert!(error.contains("TaskResult<i64>"));
}

#[test]
fn rejects_mutex_payloads_that_are_nested_thread_lock_families() {
    let error = generic("Mutex", generic("Thread", named("i64")))
        .validate_container_contract()
        .unwrap_err();
    assert!(error.contains("Mutex<...>"));
    assert!(error.contains("Thread<i64>"));
}

#[test]
fn classifies_ref_types_as_address_shape() {
    let node = address("Node");
    let buffer = address("Buffer");

    assert!(node.is_address_type());
    assert!(buffer.is_address_type());
    assert_eq!(node.address_target_name(), Some("Node"));
    assert_eq!(buffer.address_target_name(), Some("Buffer"));
    assert!(node.supports_address_class(NirAddressClass::Owned));
    assert!(node.supports_address_class(NirAddressClass::Borrowed));
    assert_eq!(node.shape(), NirTypeShape::Ref);
    assert_eq!(buffer.shape(), NirTypeShape::Ref);
    assert_eq!(buffer.container_kind(), None);
}

#[test]
fn rejects_pass_ready_shader_state_with_non_pass_payload() {
    let error = NirShaderFlowState::PassReady
        .validate_payload(&named("Frame"))
        .unwrap_err();
    assert!(error.contains("pass_ready"));
    assert!(error.contains("Pass"));
}

#[test]
fn rejects_kernel_config_ready_with_non_integer_payload() {
    let error = NirKernelFlowState::ConfigReady
        .validate_payload(&named("bool"))
        .unwrap_err();
    assert!(error.contains("config_ready"));
    assert!(error.contains("integer scalar"));
}

#[test]
fn classifies_scalar_binary_as_pure() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::Binary {
            op: super::NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(2)),
            rhs: Box::new(NirExpr::Int(3)),
        }),
        NirExprEffectClass::Pure
    );
}

#[test]
fn classifies_borrow_as_read_only() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned())))),
        NirExprEffectClass::LocalReadOnly
    );
}

#[test]
fn classifies_profile_ref_as_domain_read_only() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::ShaderProfileVertexCountRef {
            unit: "Main".to_owned(),
        }),
        NirExprEffectClass::DomainReadOnly
    );
}

#[test]
fn classifies_host_tick_as_host_read_only() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::CpuTickI64 { start: 0, step: 1 }),
        NirExprEffectClass::HostReadOnly
    );
    assert_eq!(
        nir_host_read_surface(&NirExpr::CpuTickI64 { start: 0, step: 1 }),
        Some(NirHostReadSurface::ClockTick)
    );
}

#[test]
fn reports_render_descriptor_host_surface() {
    assert_eq!(
        nir_host_read_surface(&NirExpr::ShaderViewport {
            width: 640,
            height: 360,
        }),
        Some(NirHostReadSurface::RenderDescriptor)
    );
}

#[test]
fn reports_scheduler_and_input_host_surfaces() {
    assert_eq!(
        nir_host_read_surface(&NirExpr::CpuBindCore(0)),
        Some(NirHostReadSurface::SchedulerLane)
    );
    assert_eq!(
        nir_host_read_surface(&NirExpr::CpuInputI64 {
            channel: "speed".to_owned(),
            default: 4,
            min: None,
            max: None,
            step: None,
        }),
        Some(NirHostReadSurface::InputChannel)
    );
}

#[test]
fn resolves_host_main_scheduler_lane_bridge() {
    let bridge = nir_host_scheduler_bridge(&NirExpr::CpuBindCore(0))
        .expect("cpu.bind_core(0) should expose a scheduler bridge");
    assert_eq!(
        bridge,
        NirHostSchedulerBridge {
            kind: NirHostSchedulerBridgeKind::HostMainLane,
            lane: 0,
        }
    );
    assert_eq!(bridge.as_str(), "host_main_lane");
    assert_eq!(bridge.resolved_source(), "cpu_bind_core_lane");
    assert_eq!(bridge.host_surface(), NirHostReadSurface::SchedulerLane);
}

#[test]
fn resolves_worker_scheduler_lane_bridge() {
    let bridge = nir_host_scheduler_bridge(&NirExpr::CpuBindCore(3))
        .expect("cpu.bind_core(3) should expose a scheduler bridge");
    assert_eq!(
        bridge,
        NirHostSchedulerBridge {
            kind: NirHostSchedulerBridgeKind::WorkerLane,
            lane: 3,
        }
    );
    assert_eq!(bridge.as_str(), "worker_lane");
    assert_eq!(bridge.resolved_source(), "cpu_bind_core_lane");
    assert_eq!(bridge.host_surface(), NirHostReadSurface::SchedulerLane);
}

#[test]
fn resolves_global_timing_bridge_to_monotonic_tick() {
    let bridge = NirHostTimingBridge::from_test_clock_domain(TestClockDomain::Global);
    assert_eq!(bridge, NirHostTimingBridge::GlobalToMonotonicTickBridge);
    assert_eq!(bridge.resolved_domain(), TestClockDomain::Monotonic);
    assert_eq!(bridge.resolved_source(), "host_monotonic_deadline");
    assert_eq!(bridge.host_surface(), NirHostReadSurface::ClockTick);
    assert_eq!(bridge.as_str(), "global_to_monotonic_tick_bridge");
}

#[test]
fn resolves_wall_timing_bridge_to_wall_deadline() {
    let bridge = NirHostTimingBridge::from_test_clock_domain(TestClockDomain::Wall);
    assert_eq!(bridge, NirHostTimingBridge::WallDeadline);
    assert_eq!(bridge.resolved_domain(), TestClockDomain::Wall);
    assert_eq!(bridge.resolved_source(), "host_wall_deadline");
    assert_eq!(bridge.host_surface(), NirHostReadSurface::ClockTick);
    assert_eq!(bridge.as_str(), "wall_deadline");
}

#[test]
fn classifies_call_as_opaque() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::Call {
            callee: "compute".to_owned(),
            args: vec![],
        }),
        NirExprEffectClass::CallOpaque
    );
}

#[test]
fn classifies_await_as_async_opaque() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::Await(Box::new(NirExpr::Var("task".to_owned())))),
        NirExprEffectClass::AsyncOpaque
    );
}

#[test]
fn classifies_instantiate_as_domain_opaque() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::Instantiate {
            domain: "data".to_owned(),
            unit: "Pipe".to_owned(),
        }),
        NirExprEffectClass::DomainOpaque
    );
}

#[test]
fn classifies_extern_call_as_stateful() {
    assert_eq!(
        nir_expr_effect_class(&NirExpr::CpuExternCall {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_side_effect".to_owned(),
            args: vec![],
        }),
        NirExprEffectClass::Stateful
    );
    assert_eq!(
        nir_expr_effect_class(&NirExpr::CpuExternCallI32 {
            abi: "c".to_owned(),
            interface: None,
            callee: "host_i32_side_effect".to_owned(),
            args: vec![],
        }),
        NirExprEffectClass::Stateful
    );
}
