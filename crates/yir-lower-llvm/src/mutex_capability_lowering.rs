use yir_core::Node;

use super::{
    fresh_reg,
    mutex_scalar::{emit_mutex_replace, emit_mutex_value, mutex_scalar_kind},
    value_ref::{coerce_to_i64, get_i64, get_mutex, get_mutex_guard, get_mutex_permit},
    LlvmLoweringState, LlvmValueRef, MutexGuardLlvmValueRef, MutexLlvmValueRef,
    MutexPermitLlvmValueRef,
};

pub(crate) fn lower_cpu_mutex_capability_node(node: &Node, state: &mut LlvmLoweringState) -> bool {
    if node.op.module != "cpu" {
        return false;
    }
    match node.op.instruction.as_str() {
        "mutex_share" => lower_share(node, state),
        "mutex_shared_close" => lower_shared_close(node, state),
        "mutex_permit" => lower_permit(node, state),
        "mutex_permit_lock" => lower_permit_lock(node, state),
        "mutex_lease_value" => lower_lease_value(node, state),
        "mutex_lease_replace" => lower_lease_replace(node, state),
        "mutex_lease_unlock" => lower_lease_unlock(node, state),
        _ => false,
    }
}

fn lower_shared_close(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(shared) = get_mutex(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_shared_close `{}` because its shared mutex is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let Some(handle) = shared.runtime_handle else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_shared_close `{}` because its shared mutex has no scheduler handle",
            node.name
        ));
        return true;
    };
    let revoked = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {revoked} = call i64 @nuis_scheduler_mutex_shared_close_i64_v1(i64 {handle})"
    ));
    state
        .registers
        .insert(node.name.clone(), LlvmValueRef::I64(revoked.clone()));
    state.last_cpu_value = Some(revoked);
    true
}

fn lower_share(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(mutex) = get_mutex(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_share `{}` because its mutex is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let static_cardinality = state.facts.get_i64(&node.op.args[1]).or_else(|| {
        get_i64(&state.registers, &node.op.args[1]).and_then(|value| value.parse().ok())
    });
    let Some(static_cardinality) = static_cardinality else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_share `{}` because its permit cardinality is not static",
            node.name
        ));
        return true;
    };
    if !(1..=64).contains(&static_cardinality) {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_share `{}` because permit cardinality {static_cardinality} is outside 1..=64",
            node.name
        ));
        return true;
    }
    let permit_cardinality = static_cardinality.to_string();
    let runtime_handle = mutex.runtime_handle.as_ref().map(|handle| {
        let shared = fresh_reg(&mut state.next_reg);
        state.body.push(format!(
            "  {shared} = call i64 @nuis_scheduler_mutex_share_i64_v1(i64 {handle}, i64 {permit_cardinality})"
        ));
        shared
    });
    state.registers.insert(
        node.name.clone(),
        LlvmValueRef::Mutex(MutexLlvmValueRef {
            runtime_handle,
            value: mutex.value,
            scalar_kind: mutex.scalar_kind,
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
    let Some(scalar_kind) = shared.scalar_kind else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_permit `{}` because its payload is outside the native scalar mutex protocol",
            node.name
        ));
        return true;
    };
    let permit = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {permit} = call i64 @nuis_scheduler_mutex_permit_i64_v1(i64 {handle}, i64 {lane})"
    ));
    state.registers.insert(
        node.name.clone(),
        LlvmValueRef::MutexPermit(MutexPermitLlvmValueRef {
            runtime_token: permit,
            scalar_kind,
        }),
    );
    true
}

fn lower_permit_lock(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(permit) = get_mutex_permit(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_permit_lock `{}` because its permit is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let guard = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {guard} = call i64 @nuis_scheduler_mutex_permit_lock_i64_v1(i64 {})",
        permit.runtime_token
    ));
    state.registers.insert(
        node.name.clone(),
        LlvmValueRef::MutexGuard(MutexGuardLlvmValueRef {
            runtime_guard: Some(guard),
            value: Box::new(permit.scalar_kind.staged_zero()),
            scalar_kind: Some(permit.scalar_kind),
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
    let value = match (&lease.runtime_guard, lease.scalar_kind) {
        (Some(guard), Some(kind)) => emit_mutex_value(guard, kind, state),
        _ => (*lease.value).clone(),
    };
    state.registers.insert(node.name.clone(), value.clone());
    if let Some(value) = coerce_to_i64(&value, &mut state.body, &mut state.next_reg) {
        state.last_cpu_value = Some(value);
    }
    true
}

fn lower_lease_replace(node: &Node, state: &mut LlvmLoweringState) -> bool {
    let Some(lease) = get_mutex_guard(&state.registers, &node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_lease_replace `{}` because its lease is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let Some(replacement) = state.registers.get(&node.op.args[1]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_lease_replace `{}` because its replacement is outside the current CPU LLVM slice",
            node.name
        ));
        return true;
    };
    let Some(scalar_kind) = lease
        .scalar_kind
        .or_else(|| mutex_scalar_kind(&replacement))
    else {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_lease_replace `{}` because its payload is outside the native scalar mutex protocol",
            node.name
        ));
        return true;
    };
    if mutex_scalar_kind(&replacement) != Some(scalar_kind) {
        state.body.push(format!(
            "  ; deferred lowering for cpu.mutex_lease_replace `{}` because its replacement scalar kind does not match the lease",
            node.name
        ));
        return true;
    }
    let old = match lease.runtime_guard.as_ref() {
        Some(guard) => match emit_mutex_replace(guard, &replacement, scalar_kind, state) {
            Some(old) => old,
            None => return true,
        },
        None if mutex_scalar_kind(lease.value.as_ref()) == Some(scalar_kind) => {
            (*lease.value).clone()
        }
        None => {
            state.body.push(format!(
                "  ; deferred lowering for cpu.mutex_lease_replace `{}` because its staged lease scalar kind does not match the replacement",
                node.name
            ));
            return true;
        }
    };
    state.registers.insert(
        node.op.args[0].clone(),
        LlvmValueRef::MutexGuard(MutexGuardLlvmValueRef {
            runtime_guard: lease.runtime_guard,
            value: Box::new(replacement),
            scalar_kind: Some(scalar_kind),
        }),
    );
    state.registers.insert(node.name.clone(), old.clone());
    if let Some(old) = coerce_to_i64(&old, &mut state.body, &mut state.next_reg) {
        state.last_cpu_value = Some(old);
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
