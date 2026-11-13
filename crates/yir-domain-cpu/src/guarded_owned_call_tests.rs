use super::*;

#[test]
fn owned_continuation_consumes_its_resources_and_never_simulates_success() {
    let node = cpu_node(
        "continue",
        "cpu.tail_call_owned_struct_return",
        vec![
            "next",
            "State{pixels:Bytes}",
            "buffer",
            "width",
            "transfer_bytes_v1=1",
            "saved",
            "cleanup_bytes_v1=1",
            "spare",
        ],
    );
    let semantics = CpuMod.describe(&node, &cpu_resource()).unwrap();
    assert_eq!(
        semantics.dependencies,
        ["buffer", "width", "saved", "spare"]
    );
    assert!(semantics.has_effect);
    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert_eq!(profile.accesses[0].mode, yir_core::GlmUseMode::Write);
    assert!(profile.accesses[2..]
        .iter()
        .all(|a| a.class == yir_core::GlmValueClass::Res && a.mode == yir_core::GlmUseMode::Own));
    assert_eq!(profile.effect, yir_core::GlmEffect::DomainMove);
    assert!(CpuMod
        .execute(&node, &cpu_resource(), &mut ExecutionState::default())
        .unwrap_err()
        .contains("native function execution"));
}

#[test]
fn guarded_owned_call_describes_only_runtime_inputs_and_borrows_buffer() {
    let node = cpu_node(
        "exit",
        "cpu.guard_call_owned_struct_return",
        vec!["ready", "helper", "State{pixels:Bytes}", "buffer", "width"],
    );
    let semantics = CpuMod.describe(&node, &cpu_resource()).unwrap();
    assert_eq!(semantics.dependencies, ["ready", "buffer", "width"]);
    assert!(semantics.has_effect);
    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert_eq!(profile.accesses.len(), 3);
    assert_eq!(profile.accesses[1].class, yir_core::GlmValueClass::Res);
    assert_eq!(profile.accesses[1].mode, yir_core::GlmUseMode::Write);
    assert_eq!(profile.effect, yir_core::GlmEffect::None);
    let mut state = ExecutionState::default();
    state.values.insert("ready".into(), Value::Bool(false));
    assert_eq!(
        CpuMod.execute(&node, &cpu_resource(), &mut state).unwrap(),
        Value::Unit
    );
    state.values.insert("ready".into(), Value::Bool(true));
    assert!(CpuMod
        .execute(&node, &cpu_resource(), &mut state)
        .unwrap_err()
        .contains("native function execution"));
}

#[test]
fn guarded_owned_call_rejects_malformed_metadata() {
    for args in [
        vec![],
        vec!["ready", "helper", "State{pixels:Bytes}"],
        vec!["ready", "bad-name", "State{pixels:Bytes}", "buffer"],
        vec!["ready", "helper", "invalid", "buffer"],
        vec!["arg"; 37],
    ] {
        let node = cpu_node("exit", "cpu.guard_call_owned_struct_return", args);
        assert!(CpuMod.describe(&node, &cpu_resource()).is_err());
    }
}

#[test]
fn guarded_cleanup_is_a_lifetime_access_not_an_unconditional_consume() {
    let node = cpu_node(
        "exit",
        "cpu.guard_call_owned_struct_return",
        vec![
            "ready",
            "helper",
            "State{pixels:Bytes}",
            "buffer",
            "width",
            "cleanup_bytes_v1=2",
            "saved",
            "spare",
        ],
    );
    let semantics = CpuMod.describe(&node, &cpu_resource()).unwrap();
    assert_eq!(
        semantics.dependencies,
        ["ready", "buffer", "width", "saved", "spare"]
    );
    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert!(profile.accesses[3..]
        .iter()
        .all(|a| a.class == yir_core::GlmValueClass::Res && a.mode == yir_core::GlmUseMode::Write));
    assert_eq!(profile.effect, yir_core::GlmEffect::None);
    let mut state = ExecutionState::default();
    state.values.insert("ready".into(), Value::Bool(false));
    state
        .values
        .insert("saved".into(), Value::OwnedBytes(vec![7]));
    assert_eq!(
        CpuMod.execute(&node, &cpu_resource(), &mut state).unwrap(),
        Value::Unit
    );
    assert_eq!(state.values["saved"], Value::OwnedBytes(vec![7]));
}

#[test]
fn guarded_transfer_is_not_a_fallthrough_move_or_a_scalar_capture() {
    let node = cpu_node(
        "exit",
        "cpu.guard_call_owned_struct_return",
        vec![
            "ready",
            "helper",
            "State{pixels:Bytes}",
            "buffer",
            "width",
            "transfer_bytes_v1=1",
            "saved",
            "cleanup_bytes_v1=1",
            "spare",
        ],
    );
    assert_eq!(
        CpuMod
            .describe(&node, &cpu_resource())
            .unwrap()
            .dependencies,
        ["ready", "buffer", "width", "saved", "spare"]
    );
    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert!(profile.accesses[3..]
        .iter()
        .all(|a| a.class == yir_core::GlmValueClass::Res && a.mode == yir_core::GlmUseMode::Write));
    let mut state = ExecutionState::default();
    state.values.insert("ready".into(), Value::Bool(false));
    state
        .values
        .insert("saved".into(), Value::OwnedBytes(vec![7]));
    CpuMod.execute(&node, &cpu_resource(), &mut state).unwrap();
    assert_eq!(state.values["saved"], Value::OwnedBytes(vec![7]));
}
