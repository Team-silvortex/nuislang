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
fn glm_profiles_owned_bytes_select_as_conditional_ownership_transfer() {
    let op = Operation::parse(
        "cpu.select_owned_bytes",
        vec![
            "condition".to_owned(),
            "bytes".to_owned(),
            "bytes".to_owned(),
        ],
    )
    .unwrap();
    let profile = crate::glm_profile_for_operation(&op);

    assert_eq!(profile.result_class, GlmValueClass::Res);
    assert_eq!(profile.accesses[0].class, GlmValueClass::Val);
    assert_eq!(profile.accesses[0].mode, GlmUseMode::Read);
    assert!(profile.accesses[1..]
        .iter()
        .all(|access| access.class == GlmValueClass::Res && access.mode == GlmUseMode::Own));
}

#[test]
fn glm_profiles_distinct_owned_bytes_select_as_cleanup_move() {
    let op = Operation::parse(
        "cpu.select_owned_bytes_drop_unselected",
        vec![
            "condition".to_owned(),
            "left".to_owned(),
            "right".to_owned(),
        ],
    )
    .unwrap();
    let profile = crate::glm_profile_for_operation(&op);

    assert_eq!(profile.result_class, GlmValueClass::Res);
    assert_eq!(profile.effect, GlmEffect::DomainMove);
    assert_eq!(profile.accesses[0].mode, GlmUseMode::Read);
    assert!(profile.accesses[1..]
        .iter()
        .all(|access| access.class == GlmValueClass::Res && access.mode == GlmUseMode::Own));
}

#[test]
fn glm_profiles_branch_owned_call_as_one_owner_move() {
    let op = Operation::parse(
        "cpu.branch_call_owned_bytes",
        vec![
            "condition".to_owned(),
            "left".to_owned(),
            "right".to_owned(),
            "bytes".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
        ],
    )
    .unwrap();
    let profile = crate::glm_profile_for_operation(&op);

    assert_eq!(profile.result_class, GlmValueClass::Res);
    assert_eq!(profile.accesses.len(), 2);
    assert_eq!(profile.accesses[1].input, "bytes");
    assert_eq!(profile.accesses[1].class, GlmValueClass::Res);
    assert_eq!(profile.accesses[1].mode, GlmUseMode::Own);
    assert_eq!(profile.effect, GlmEffect::DomainMove);
}

#[test]
fn branch_owned_call_argument_segments_are_protocol_checked() {
    let args = vec![
        "condition".to_owned(),
        "left".to_owned(),
        "right".to_owned(),
        "bytes".to_owned(),
        "1".to_owned(),
        "left_delta".to_owned(),
        "2".to_owned(),
        "right_factor".to_owned(),
        "right_enabled".to_owned(),
    ];
    let parsed = crate::parse_branch_owned_call_args(&args).expect("valid segmented arguments");
    assert_eq!(parsed.owner, "bytes");
    assert_eq!(parsed.then_scalar_args, ["left_delta"]);
    assert_eq!(parsed.else_scalar_args, ["right_factor", "right_enabled"]);

    let mut trailing = args;
    trailing.push("unclaimed".to_owned());
    assert!(crate::parse_branch_owned_call_args(&trailing).is_none());
}

#[test]
fn owned_select_tree_protocol_rejects_invalid_owner_paths() {
    let valid = vec![
        "2".to_owned(),
        "left".to_owned(),
        "right".to_owned(),
        "if".to_owned(),
        "outer".to_owned(),
        "owner".to_owned(),
        "0".to_owned(),
        "owner".to_owned(),
        "1".to_owned(),
    ];
    let parsed = crate::parse_owned_select_tree_args(&valid).expect("valid owned select tree");
    assert_eq!(parsed.owners, ["left", "right"]);

    let mut out_of_range = valid.clone();
    *out_of_range.last_mut().unwrap() = "2".to_owned();
    assert!(crate::parse_owned_select_tree_args(&out_of_range).is_none());
    let mut trailing = valid;
    trailing.push("owner".to_owned());
    assert!(crate::parse_owned_select_tree_args(&trailing).is_none());

    let call = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "call".to_owned(),
        "transform".to_owned(),
        "0".to_owned(),
        "2".to_owned(),
        "value".to_owned(),
        "delta".to_owned(),
        "value".to_owned(),
        "enabled".to_owned(),
    ];
    let parsed = crate::parse_owned_select_tree_args(&call).expect("valid call leaf");
    let crate::OwnedSelectTree::Call {
        callee,
        owner,
        scalar_args,
    } = parsed.tree
    else {
        panic!("expected call leaf");
    };
    assert_eq!(callee, "transform");
    assert_eq!(owner, 0);
    assert_eq!(
        scalar_args,
        [
            crate::OwnedSelectScalarArg::Value("delta"),
            crate::OwnedSelectScalarArg::Value("enabled")
        ]
    );

    let projected = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "call".to_owned(),
        "transform".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "variant_field".to_owned(),
        "route".to_owned(),
        "Route.Left".to_owned(),
        "value".to_owned(),
    ];
    let parsed = crate::parse_owned_select_tree_args(&projected).expect("projected call leaf");
    let crate::OwnedSelectTree::Call { scalar_args, .. } = &parsed.tree else {
        panic!("expected projected call leaf");
    };
    assert_eq!(
        scalar_args,
        &[crate::OwnedSelectScalarArg::VariantField {
            base: "route",
            variant: "Route.Left",
            field: "value",
        }]
    );
    let profile = crate::glm_profile_for_operation(
        &Operation::parse("cpu.select_owned_bytes_tree", projected).unwrap(),
    );
    assert!(profile
        .accesses
        .iter()
        .any(|access| access.input == "route" && access.mode == GlmUseMode::Read));
    assert!(profile
        .accesses
        .iter()
        .any(|access| access.input == "bytes" && access.mode == GlmUseMode::Own));

    let nested = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "call".to_owned(),
        "transform".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "struct_field".to_owned(),
        "score".to_owned(),
        "variant_field".to_owned(),
        "route".to_owned(),
        "Route.Left".to_owned(),
        "value".to_owned(),
    ];
    let parsed = crate::parse_owned_select_tree_args(&nested).expect("nested projected leaf");
    let crate::OwnedSelectTree::Call { scalar_args, .. } = &parsed.tree else {
        panic!("expected nested projected call leaf");
    };
    assert_eq!(
        scalar_args,
        &[crate::OwnedSelectScalarArg::StructField {
            field: "score",
            base: Box::new(crate::OwnedSelectScalarArg::VariantField {
                base: "route",
                variant: "Route.Left",
                field: "value",
            }),
        }]
    );
    let mut inputs = Vec::new();
    crate::owned_select_tree_scalar_args(&parsed.tree, &mut inputs);
    assert_eq!(inputs, ["route"]);

    let cast = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "call".to_owned(),
        "transform".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "cast".to_owned(),
        "i32_to_i64".to_owned(),
        "struct_field".to_owned(),
        "score".to_owned(),
        "variant_field".to_owned(),
        "route".to_owned(),
        "Route.Left".to_owned(),
        "value".to_owned(),
    ];
    let parsed = crate::parse_owned_select_tree_args(&cast).expect("cast projected leaf");
    let crate::OwnedSelectTree::Call { scalar_args, .. } = &parsed.tree else {
        panic!("expected cast projected call leaf");
    };
    assert!(matches!(
        &scalar_args[0],
        crate::OwnedSelectScalarArg::Cast {
            kind: crate::OwnedSelectScalarCast::I32ToI64,
            ..
        }
    ));
    let mut invalid_cast = cast;
    invalid_cast[7] = "i32_to_ptr".to_owned();
    assert!(crate::parse_owned_select_tree_args(&invalid_cast).is_none());

    let non_null = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "call".to_owned(),
        "retain".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "non_null".to_owned(),
        "value".to_owned(),
        "scratch".to_owned(),
    ];
    let parsed = crate::parse_owned_select_tree_args(&non_null).expect("non-null leaf proof");
    let crate::OwnedSelectTree::Call { scalar_args, .. } = &parsed.tree else {
        panic!("expected non-null call leaf");
    };
    assert!(matches!(
        &scalar_args[0],
        crate::OwnedSelectScalarArg::NonNull { value }
            if matches!(value.as_ref(), crate::OwnedSelectScalarArg::Value("scratch"))
    ));
    let mut inputs = Vec::new();
    crate::owned_select_tree_scalar_args(&parsed.tree, &mut inputs);
    assert_eq!(inputs, ["scratch"]);

    let traversal_borrow = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "call".to_owned(),
        "retain".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "traversal_borrow".to_owned(),
        "value".to_owned(),
        "head".to_owned(),
    ];
    let parsed =
        crate::parse_owned_select_tree_args(&traversal_borrow).expect("traversal borrow leaf");
    let crate::OwnedSelectTree::Call { scalar_args, .. } = &parsed.tree else {
        panic!("expected traversal borrow call leaf");
    };
    assert!(matches!(
        &scalar_args[0],
        crate::OwnedSelectScalarArg::TraversalBorrow { value }
            if matches!(value.as_ref(), crate::OwnedSelectScalarArg::Value("head"))
    ));
    let mut inputs = Vec::new();
    crate::owned_select_tree_scalar_args(&parsed.tree, &mut inputs);
    assert_eq!(inputs, ["head"]);

    let owned_transfer = vec![
        "1".to_owned(),
        "bytes".to_owned(),
        "if".to_owned(),
        "choose".to_owned(),
        "call".to_owned(),
        "consume_left".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "owned_transfer".to_owned(),
        "head".to_owned(),
        "call".to_owned(),
        "consume_right".to_owned(),
        "0".to_owned(),
        "1".to_owned(),
        "owned_transfer".to_owned(),
        "head".to_owned(),
    ];
    let parsed =
        crate::parse_owned_select_tree_args(&owned_transfer).expect("exact-one pointer transfer");
    let mut transfers = Vec::new();
    crate::owned_select_tree_transfers(&parsed.tree, &mut transfers);
    assert_eq!(transfers, ["head"]);
    let profile = crate::glm_profile_for_operation(
        &Operation::parse("cpu.select_owned_bytes_tree", owned_transfer.clone()).unwrap(),
    );
    assert!(profile.accesses.iter().any(|access| {
        access.input == "head"
            && access.class == GlmValueClass::Res
            && access.mode == GlmUseMode::Own
    }));
    assert!(!profile
        .accesses
        .iter()
        .any(|access| access.input == "head" && access.mode == GlmUseMode::Read));

    let mut asymmetric = owned_transfer.clone();
    asymmetric.truncate(asymmetric.len() - 2);
    asymmetric.extend(["owner".to_owned(), "0".to_owned()]);
    assert!(crate::parse_owned_select_tree_args(&asymmetric).is_none());

    let mut duplicate = owned_transfer;
    duplicate[7] = "2".to_owned();
    duplicate.splice(10..10, ["owned_transfer".to_owned(), "head".to_owned()]);
    assert!(crate::parse_owned_select_tree_args(&duplicate).is_none());
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
fn parses_open_branch_effect_action_sequences_and_profiles_registered_operands() {
    let encoded = vec![
        "choose_left".to_owned(),
        "unit".to_owned(),
        "1".to_owned(),
        "cpu".to_owned(),
        "free".to_owned(),
        "unit".to_owned(),
        "1".to_owned(),
        "resource_own".to_owned(),
        "head".to_owned(),
        "2".to_owned(),
        "cpu".to_owned(),
        "load_value".to_owned(),
        "i64".to_owned(),
        "1".to_owned(),
        "resource_read".to_owned(),
        "head".to_owned(),
        "cpu".to_owned(),
        "free".to_owned(),
        "unit".to_owned(),
        "1".to_owned(),
        "resource_own".to_owned(),
        "head".to_owned(),
    ];
    let parsed = crate::parse_branch_effect_args(&encoded).expect("branch effect protocol");
    assert_eq!(parsed.then_actions.len(), 1);
    assert_eq!(parsed.else_actions.len(), 2);
    assert_eq!(parsed.else_actions[0].instruction, "load_value");
    assert_eq!(
        parsed.else_actions[0].result,
        crate::BranchEffectResult::I64
    );
    assert_eq!(
        parsed.else_actions[0].operands[0].access,
        crate::BranchEffectAccess::ResourceRead
    );
    assert_eq!(
        crate::branch_effect_inputs(&parsed),
        ["choose_left", "head", "head", "head"]
    );

    let profile = crate::glm_profile_for_operation(
        &Operation::parse("cpu.branch_effect", encoded.clone()).unwrap(),
    );
    assert!(profile
        .accesses
        .iter()
        .any(|access| access.input == "head" && access.mode == GlmUseMode::Read));
    assert_eq!(
        profile
            .accesses
            .iter()
            .filter(|access| access.input == "head" && access.mode == GlmUseMode::Own)
            .count(),
        1
    );

    let mut truncated = encoded;
    truncated.pop();
    assert!(crate::parse_branch_effect_args(&truncated).is_none());
}

#[test]
fn owned_pointer_branch_result_is_a_glm_resource() {
    let encoded = vec![
        "choose_left".to_owned(),
        "owned_ptr".to_owned(),
        "address_kind=node".to_owned(),
        "nullable=true".to_owned(),
        "1".to_owned(),
        "cpu".to_owned(),
        "take_ptr_drop_other".to_owned(),
        "owned_ptr".to_owned(),
        "2".to_owned(),
        "resource_own".to_owned(),
        "left".to_owned(),
        "resource_own".to_owned(),
        "right".to_owned(),
        "1".to_owned(),
        "cpu".to_owned(),
        "take_ptr_drop_other".to_owned(),
        "owned_ptr".to_owned(),
        "2".to_owned(),
        "resource_own".to_owned(),
        "right".to_owned(),
        "resource_own".to_owned(),
        "left".to_owned(),
    ];
    let parsed = crate::parse_branch_effect_args(&encoded).unwrap();
    assert_eq!(parsed.merge_result, crate::BranchEffectResult::OwnedPointer);
    assert_eq!(parsed.address_kind, Some("node"));
    assert!(parsed.nullable);
    assert!(crate::branch_effect_merge_is_valid(&parsed));
    let profile =
        crate::glm_profile_for_operation(&Operation::parse("cpu.branch_effect", encoded).unwrap());
    assert_eq!(profile.result_class, crate::GlmValueClass::Res);
    assert_eq!(
        profile
            .accesses
            .iter()
            .filter(|access| access.mode == GlmUseMode::Own)
            .count(),
        2
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
