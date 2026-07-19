use super::*;

struct EncodedBranchAction {
    plan: yir_core::PlannedBranchEffectAction,
    binding: Option<String>,
}

pub(super) fn lower_branch_effect(
    condition: String,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Option<LoweredIfOutcome>, String> {
    let Some(then_actions) =
        encode_branch_actions(then_body, bindings, state.branch_action_registry)?
    else {
        return Ok(None);
    };
    let Some(else_actions) =
        encode_branch_actions(else_body, bindings, state.branch_action_registry)?
    else {
        return Ok(None);
    };
    if then_actions.is_empty() && else_actions.is_empty() {
        return Ok(None);
    }

    let merged_binding = merged_action_binding(&then_actions, &else_actions);
    let merge_result = merged_binding
        .as_ref()
        .map_or(yir_core::BranchEffectResult::Unit, |(_, result)| *result);

    let name = next_name(state, "branch_effect");
    let mut args = vec![condition.clone(), merge_result.as_str().to_owned()];
    encode_actions(&then_actions, &mut args);
    encode_actions(&else_actions, &mut args);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "branch_effect".to_owned(),
            args,
        },
    });
    push_dep_edges(state, &condition, &name);
    for action in then_actions.iter().chain(&else_actions) {
        for operand in &action.plan.operands {
            push_dep_edges(state, &operand.value, &name);
            if operand.access == yir_core::BranchEffectAccess::ResourceOwn {
                push_lifetime_edge(state, &operand.value, &name);
            }
        }
    }
    chain_statement_effect(state, &name);
    Ok(Some(match merged_binding {
        Some((binding, _)) => LoweredIfOutcome::Bind {
            name: binding,
            value: name,
        },
        None => LoweredIfOutcome::Continued,
    }))
}

fn merged_action_binding(
    then_actions: &[EncodedBranchAction],
    else_actions: &[EncodedBranchAction],
) -> Option<(String, yir_core::BranchEffectResult)> {
    let then_action = then_actions.last()?;
    let else_action = else_actions.last()?;
    let then_binding = then_action.binding.as_ref()?;
    let else_binding = else_action.binding.as_ref()?;
    (then_binding == else_binding
        && then_action.plan.result == else_action.plan.result
        && then_action.plan.result != yir_core::BranchEffectResult::Unit)
        .then(|| (then_binding.clone(), then_action.plan.result))
}

fn encode_branch_actions(
    stmts: &[NirStmt],
    bindings: &BTreeMap<String, String>,
    registry: &yir_core::ModRegistry,
) -> Result<Option<Vec<EncodedBranchAction>>, String> {
    let mut actions = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        let (expr, binding) = match stmt {
            NirStmt::Let { name, value, .. } => (value, Some(name.clone())),
            NirStmt::Expr(expr) => (expr, None),
            _ => return Ok(None),
        };
        let Some(source_action) = expr.branch_effect_action() else {
            return Ok(None);
        };
        let Some(operands) = source_action
            .operands
            .iter()
            .map(|operand| named_operand(operand, bindings))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(None);
        };
        actions.push(EncodedBranchAction {
            plan: registry.plan_branch_effect_action(
                source_action.module,
                source_action.instruction,
                operands,
            )?,
            binding,
        });
    }
    Ok(Some(actions))
}

fn named_operand(expr: &NirExpr, bindings: &BTreeMap<String, String>) -> Option<String> {
    let NirExpr::Var(name) = expr else {
        return None;
    };
    bindings.get(name).cloned()
}

fn encode_actions(actions: &[EncodedBranchAction], out: &mut Vec<String>) {
    out.push(actions.len().to_string());
    for action in actions {
        out.extend([
            action.plan.module.clone(),
            action.plan.instruction.clone(),
            action.plan.result.as_str().to_owned(),
            action.plan.operands.len().to_string(),
        ]);
        for operand in &action.plan.operands {
            out.push(operand.access.as_str().to_owned());
            out.push(operand.value.clone());
        }
    }
}
