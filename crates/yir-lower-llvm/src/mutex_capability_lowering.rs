use yir_core::Node;

use super::{
    fresh_reg,
    value_ref::{get_i64, get_mutex, get_mutex_guard},
    LlvmLoweringState, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef,
};

pub(crate) fn lower_cpu_mutex_capability_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    if node.op.module != "cpu" {
        return false;
    }
    match node.op.instruction.as_str() {
        "mutex_share" => lower_share(node, state),
        "mutex_permit" => lower_permit(node, state),
        "mutex_permit_lock" => lower_permit_lock(node, state),
        "mutex_lease_value" => lower_lease_value(node, state),
        "mutex_lease_unlock" => lower_lease_unlock(node, state),
        _ => false,
    }
}

fn lower_share(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(mutex) = get_mutex(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_share `{}` because its mutex is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let runtime_handle = mutex.runtime_handle.as_ref().map(|handle| {
        let shared = fresh_reg(&mut state.next_reg);
        state.body.push(format!(
            "  {shared} = call i64 @nuis_scheduler_mutex_share_i64_v1(i64 {handle})"
        ));
        shared
    });
    state.registers.insert(
        node.name.clone(),
        LlvmValueRef::Mutex(MutexLlvmValueRef {
            runtime_handle,
            value: mutex.value,
        }),
    );
    true
}

fn lower_permit(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(shared) = get_mutex(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_permit `{}` because its shared mutex is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let Some(lane) = get_i64(&state.registers, &node.op.args[1]) else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_permit `{}` because its lane is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let Some(handle) = shared.runtime_handle else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_permit `{}` because its shared mutex has no scheduler handle",
            node.name
        ));
        return true;
    };
    let permit = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {permit} = call i64 @nuis_scheduler_mutex_permit_i64_v1(i64 {handle}, i64 {lane})"
    ));
    state
        .registers
        .insert(node.name.clone(), LlvmValueRef::I64(permit.clone()));
    state.last_cpu_value = Some(permit);
    true
}

fn lower_permit_lock(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(permit) = get_i64(&state.registers, &node.op.args[0]) else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_permit_lock `{}` because its permit is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let guard = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {guard} = call i64 @nuis_scheduler_mutex_permit_lock_i64_v1(i64 {permit})"
    ));
    state.registers.insert(
        node.name.clone(),
        LlvmValueRef::MutexGuard(MutexGuardLlvmValueRef {
            runtime_guard: Some(guard),
            value: Box::new(LlvmValueRef::I64("0".to_owned())),
        }),
    );
    true
}

fn lower_lease_value(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(lease) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_lease_value `{}` because its lease is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let value = lease.runtime_guard.as_ref().map_or_else(
        || (*lease.value).clone(),
        |guard| {
            let value = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {value} = call i64 @nuis_scheduler_mutex_value_i64_v1(i64 {guard})"
            ));
            LlvmValueRef::I64(value)
        },
    );
    state.registers.insert(node.name.clone(), value.clone());
    if let LlvmValueRef::I64(value) = value {
        state.last_cpu_value = Some(value);
    }
    true
}

fn lower_lease_unlock(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(lease) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_lease_unlock `{}` because its lease is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let released = lease.runtime_guard.as_ref().map_or_else(
        || "1".to_owned(),
        |guard| {
            let released = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {released} = call i64 @nuis_scheduler_mutex_lease_unlock_i64_v1(i64 {guard})"
            ));
            released
        },
    );
    state
        .registers
        .insert(node.name.clone(), LlvmValueRef::I64(released.clone()));
    state.last_cpu_value = Some(released);
    true
}
