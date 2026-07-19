use super::*;

fn branch_registry() -> yir_core::ModRegistry {
    let mut registry = yir_core::ModRegistry::new();
    registry.register(CpuMod);
    registry
}

#[test]
fn branch_effect_executes_only_the_selected_action_sequence() {
    let registry = branch_registry();
    let resource = cpu_resource();
    let node = cpu_node(
        "consume",
        "cpu.branch_effect",
        vec![
            "choose_then",
            "unit",
            "1",
            "cpu",
            "free",
            "unit",
            "1",
            "resource_own",
            "head",
            "2",
            "cpu",
            "load_value",
            "i64",
            "1",
            "resource_read",
            "missing",
            "cpu",
            "free",
            "unit",
            "1",
            "resource_own",
            "head",
        ],
    );
    let mut state = ExecutionState::default();
    let head = state.alloc_heap_node(41, None);
    state
        .values
        .insert("choose_then".to_owned(), Value::Bool(true));
    state
        .values
        .insert("head".to_owned(), Value::Pointer(Some(head)));
    assert_eq!(
        registry
            .execute_branch_effect_node(&node, &resource, &mut state)
            .unwrap()
            .unwrap(),
        Value::Unit
    );

    state
        .values
        .insert("choose_then".to_owned(), Value::Bool(false));
    let error = registry
        .execute_branch_effect_node(&node, &resource, &mut state)
        .unwrap_err();
    assert!(error.contains("missing"), "unexpected diagnostic: {error}");
}

#[test]
fn branch_effect_rejects_unregistered_leaf_actions() {
    let registry = branch_registry();
    let node = cpu_node(
        "consume",
        "cpu.branch_effect",
        vec![
            "choose_then",
            "unit",
            "1",
            "cpu",
            "store_value",
            "unit",
            "1",
            "resource_own",
            "head",
            "0",
        ],
    );
    assert!(registry
        .describe_branch_effect_node(&node)
        .unwrap_err()
        .contains("branch action `cpu.store_value` with an undeclared"));
}

#[test]
fn cpu_registers_branch_action_result_and_access_contracts() {
    let mut registry = yir_core::ModRegistry::new();
    registry.register(CpuMod);

    let load = registry
        .branch_effect_action_capability("cpu", "load_value")
        .expect("registered load action");
    assert_eq!(load.result, yir_core::BranchEffectResult::I64);
    assert_eq!(
        load.operand_accesses,
        [yir_core::BranchEffectAccess::ResourceRead]
    );

    let free = registry
        .branch_effect_action_capability("cpu", "free")
        .expect("registered free action");
    assert_eq!(free.result, yir_core::BranchEffectResult::Unit);
    assert_eq!(
        free.operand_accesses,
        [yir_core::BranchEffectAccess::ResourceOwn]
    );

    let planned = registry
        .plan_branch_effect_action("cpu", "load_value", vec!["head".to_owned()])
        .expect("registry-backed branch action plan");
    assert_eq!(planned.result, yir_core::BranchEffectResult::I64);
    assert_eq!(planned.operands[0].value, "head");
    assert_eq!(
        planned.operands[0].access,
        yir_core::BranchEffectAccess::ResourceRead
    );
    assert!(registry
        .plan_branch_effect_action("cpu", "free", Vec::new())
        .unwrap_err()
        .contains("expects 1 operands, got 0"));
}

#[test]
fn branch_effect_rejects_forged_access_metadata_for_registered_action() {
    let registry = branch_registry();
    let node = cpu_node(
        "consume",
        "cpu.branch_effect",
        vec![
            "choose_then",
            "unit",
            "1",
            "cpu",
            "free",
            "unit",
            "1",
            "resource_read",
            "head",
            "0",
        ],
    );
    assert!(registry
        .describe_branch_effect_node(&node)
        .unwrap_err()
        .contains("branch action `cpu.free` with an undeclared"));
}

#[test]
fn branch_effect_returns_the_selected_i64_action_result() {
    let registry = branch_registry();
    let resource = cpu_resource();
    let node = cpu_node(
        "selected_value",
        "cpu.branch_effect",
        vec![
            "choose_then",
            "i64",
            "1",
            "cpu",
            "load_value",
            "i64",
            "1",
            "resource_read",
            "left",
            "1",
            "cpu",
            "load_value",
            "i64",
            "1",
            "resource_read",
            "right",
        ],
    );
    let mut state = ExecutionState::default();
    let left = state.alloc_heap_node(41, None);
    let right = state.alloc_heap_node(73, None);
    state
        .values
        .insert("left".to_owned(), Value::Pointer(Some(left)));
    state
        .values
        .insert("right".to_owned(), Value::Pointer(Some(right)));
    state
        .values
        .insert("choose_then".to_owned(), Value::Bool(false));

    assert_eq!(
        registry
            .execute_branch_effect_node(&node, &resource, &mut state)
            .unwrap()
            .unwrap(),
        Value::Int(73)
    );
}

#[test]
fn branch_effect_rejects_missing_declared_merge_result() {
    let registry = branch_registry();
    let node = cpu_node(
        "selected_value",
        "cpu.branch_effect",
        vec![
            "choose_then",
            "i64",
            "1",
            "cpu",
            "load_value",
            "i64",
            "1",
            "resource_read",
            "left",
            "1",
            "cpu",
            "free",
            "unit",
            "1",
            "resource_own",
            "right",
        ],
    );
    assert!(registry
        .describe_branch_effect_node(&node)
        .unwrap_err()
        .contains("do not produce the declared I64 merge result"));
}
