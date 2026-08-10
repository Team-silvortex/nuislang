use super::*;

fn capability_shape(op: NirMutexCapabilityOp) -> (&'static str, &'static str, usize) {
    match op {
        NirMutexCapabilityOp::Share => ("cpu_mutex_share", "mutex_share", 2),
        NirMutexCapabilityOp::SharedClose => ("cpu_mutex_shared_close", "mutex_shared_close", 1),
        NirMutexCapabilityOp::Permit => ("cpu_mutex_permit", "mutex_permit", 2),
        NirMutexCapabilityOp::PermitLock => ("cpu_mutex_permit_lock", "mutex_permit_lock", 1),
        NirMutexCapabilityOp::LeaseValue => ("cpu_mutex_lease_value", "mutex_lease_value", 1),
        NirMutexCapabilityOp::LeaseReplace => ("cpu_mutex_lease_replace", "mutex_lease_replace", 2),
        NirMutexCapabilityOp::LeaseUnlock => ("cpu_mutex_lease_unlock", "mutex_lease_unlock", 1),
    }
}

fn validate_static_branch_argument(
    op: NirMutexCapabilityOp,
    instruction: &str,
    lhs_args: &[NirExpr],
    rhs_args: &[NirExpr],
) -> Result<(), String> {
    let (label, range) = match op {
        NirMutexCapabilityOp::Share => ("permit cardinality", 1..=64),
        NirMutexCapabilityOp::Permit => ("lane", 0..=63),
        _ => return Ok(()),
    };
    let (Some(NirExpr::Int(lhs)), Some(NirExpr::Int(rhs))) = (lhs_args.get(1), rhs_args.get(1))
    else {
        return Err(format!(
            "branch-selected {instruction}(...) requires the same static {label} literal in both branches"
        ));
    };
    if lhs != rhs || !range.contains(lhs) {
        return Err(format!(
            "branch-selected {instruction}(...) requires the same static {label} literal in both branches"
        ));
    }
    Ok(())
}

pub(super) fn lower_selected_cpu_mutex_capability_effect(
    condition_name: String,
    lhs_op: NirMutexCapabilityOp,
    lhs_args: &[NirExpr],
    rhs_op: NirMutexCapabilityOp,
    rhs_args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if lhs_op != rhs_op {
        return Ok(None);
    }
    let (prefix, instruction, arity) = capability_shape(lhs_op);
    if lhs_args.len() != arity || rhs_args.len() != arity {
        return Ok(None);
    }
    validate_static_branch_argument(lhs_op, instruction, lhs_args, rhs_args)?;

    let mut selected_args = Vec::with_capacity(arity);
    for (lhs_arg, rhs_arg) in lhs_args.iter().zip(rhs_args) {
        let selected = if lhs_arg == rhs_arg {
            lower_expr(lhs_arg, state, bindings)?
        } else if is_selectable_cpu_runtime_expr(lhs_arg) || is_selectable_cpu_runtime_expr(rhs_arg)
        {
            let Some(selected) = lower_selected_cpu_runtime_effect(
                condition_name.clone(),
                lhs_arg,
                rhs_arg,
                state,
                bindings,
            )?
            else {
                return Ok(None);
            };
            selected
        } else {
            let lhs = lower_expr(lhs_arg, state, bindings)?;
            let rhs = lower_expr(rhs_arg, state, bindings)?;
            lower_select(condition_name.clone(), lhs, rhs, state)?
        };
        selected_args.push(selected);
    }

    let name = next_name(state, prefix);
    let mut op_args = selected_args.clone();
    op_args.extend(
        yir_core::CPU_SHARED_MUTEX_RUNTIME_METADATA
            .iter()
            .map(|value| (*value).to_owned()),
    );
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: op_args,
        },
    });
    for input in selected_args {
        push_dep_edges(state, &input, &name);
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: input,
            to: name.clone(),
        });
    }
    Ok(Some(name))
}
