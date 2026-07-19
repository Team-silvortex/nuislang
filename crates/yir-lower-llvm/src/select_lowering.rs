use std::collections::BTreeMap;

use yir_core::{owned_select_tree_conditions, parse_owned_select_tree_args, Node, OwnedSelectTree};

use super::{
    call_lowering::branch_owned_helper_signature,
    fresh_block, fresh_reg,
    owned_tree_call_args::{lower_owned_tree_scalar_args, owned_tree_scalar_args_ready},
    value_ref::coerce_to_i64,
    variant_select::emit_select_value,
    CpuHelperSignature, KnownFacts, LlvmValueRef,
};

pub(crate) fn lower_cpu_select_node(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    delayed_registers: &mut BTreeMap<String, String>,
    facts: &mut KnownFacts,
    next_reg: &mut usize,
    next_block: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    if node.op.module != "cpu"
        || !matches!(
            node.op.instruction.as_str(),
            "select"
                | "select_owned_bytes"
                | "select_owned_bytes_drop_unselected"
                | "select_owned_bytes_tree"
        )
    {
        return Ok(false);
    }
    if node.op.instruction == "select_owned_bytes_tree" {
        return lower_owned_bytes_select_tree(
            node,
            body,
            registers,
            helper_signatures,
            next_reg,
            next_block,
        );
    }
    if node.op.args.len() != 3 {
        return Err(format!(
            "cpu.{} `{}` expects condition, then value, and else value",
            node.op.instruction, node.name
        ));
    }

    if node.op.instruction == "select_owned_bytes_drop_unselected" {
        return lower_owned_bytes_select_with_cleanup(node, body, registers, next_reg, next_block);
    }

    let cond_value = registers.get(&node.op.args[0]).cloned();
    let then_value = registers.get(&node.op.args[1]).cloned();
    let else_value = registers.get(&node.op.args[2]).cloned();
    let then_delayed = delayed_registers.get(&node.op.args[1]).cloned();
    let else_delayed = delayed_registers.get(&node.op.args[2]).cloned();
    if then_delayed.is_some() || else_delayed.is_some() {
        return lower_lazy_const_select(
            node,
            body,
            registers,
            delayed_registers,
            cond_value,
            then_value,
            else_value,
            then_delayed,
            else_delayed,
            facts,
            next_reg,
            last_cpu_value,
        );
    }
    let (Some(cond_value), Some(then_value), Some(else_value)) =
        (cond_value, then_value, else_value)
    else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because one or more inputs are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };

    let Some(cond) = coerce_to_i64(&cond_value, body, next_reg) else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because its condition is not coercible to i64",
            node.name
        ));
        return Ok(true);
    };
    let cond_bool = fresh_reg(next_reg);
    body.push(format!("  {cond_bool} = icmp ne i64 {cond}, 0"));

    let selected = match emit_select_value(&cond_bool, &then_value, &else_value, body, next_reg) {
        Some(selected) => selected,
        None => {
            if let Some(selected_name) =
                const_select_condition(&node.op.args[0], &cond_value, facts).map(|condition| {
                    if condition {
                        node.op.args[1].as_str()
                    } else {
                        node.op.args[2].as_str()
                    }
                })
            {
                let selected = if selected_name == node.op.args[1] {
                    then_value
                } else {
                    else_value
                };
                registers.insert(node.name.clone(), selected.clone());
                record_known_selected_branch(node, selected_name, &selected, facts);
                if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
                    *last_cpu_value = Some(as_i64);
                }
                return Ok(true);
            }
            body.push(format!(
                "  ; deferred lowering for cpu.select `{}` because its branch values are not select-compatible in the current CPU LLVM slice",
                node.name
            ));
            return Ok(true);
        }
    };
    registers.insert(node.name.clone(), selected.clone());
    record_known_select_value(node, &cond_value, &then_value, &else_value, facts);
    if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
        *last_cpu_value = Some(as_i64);
    }
    Ok(true)
}

fn lower_owned_bytes_select_tree(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<bool, String> {
    let args = parse_owned_select_tree_args(&node.op.args).ok_or_else(|| {
        format!(
            "cpu.select_owned_bytes_tree `{}` has invalid tree arguments",
            node.name
        )
    })?;
    let owner_blobs = args
        .owners
        .iter()
        .map(|owner| match registers.get(owner) {
            Some(LlvmValueRef::OwnedBytes { blob }) => Some(blob.clone()),
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    let Some(owner_blobs) = owner_blobs else {
        body.push(format!(
            "  ; deferred lowering for cpu.select_owned_bytes_tree `{}` because one or more owners are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let mut condition_names = Vec::new();
    owned_select_tree_conditions(&args.tree, &mut condition_names);
    let mut condition_values = BTreeMap::<String, String>::new();
    for condition_name in condition_names {
        if condition_values.contains_key(condition_name) {
            continue;
        }
        let Some(condition) = registers.get(condition_name) else {
            body.push(format!(
                "  ; deferred lowering for cpu.select_owned_bytes_tree `{}` because condition `{condition_name}` is outside the current CPU LLVM slice",
                node.name
            ));
            return Ok(true);
        };
        let Some(condition) = coerce_to_i64(condition, body, next_reg) else {
            body.push(format!(
                "  ; deferred lowering for cpu.select_owned_bytes_tree `{}` because condition `{condition_name}` is not coercible to i64",
                node.name
            ));
            return Ok(true);
        };
        let condition_i1 = fresh_reg(next_reg);
        body.push(format!("  {condition_i1} = icmp ne i64 {condition}, 0"));
        condition_values.insert(condition_name.to_owned(), condition_i1);
    }
    if !owned_select_tree_calls_ready(&args.tree, node, registers, helper_signatures)? {
        body.push(format!(
            "  ; deferred lowering for cpu.select_owned_bytes_tree `{}` because one or more call-leaf scalar inputs are outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    }

    if let OwnedSelectTree::Owner(index) = &args.tree {
        emit_unselected_owner_drops(*index, &owner_blobs, body);
        registers.insert(
            node.name.clone(),
            LlvmValueRef::OwnedBytes {
                blob: owner_blobs[*index].clone(),
            },
        );
        return Ok(true);
    }
    if let OwnedSelectTree::Call {
        callee,
        owner,
        scalar_args,
    } = &args.tree
    {
        emit_unselected_owner_drops(*owner, &owner_blobs, body);
        let signature =
            branch_owned_helper_signature(node, callee, scalar_args.len(), helper_signatures)?;
        let scalar_args = lower_owned_tree_scalar_args(
            registers,
            scalar_args,
            &signature.params[1..],
            body,
            next_reg,
        )
        .expect("owned tree call inputs were prevalidated");
        let call_args = std::iter::once(format!("ptr {}", owner_blobs[*owner]))
            .chain(scalar_args)
            .collect::<Vec<_>>()
            .join(", ");
        let result = fresh_reg(next_reg);
        body.push(format!(
            "  {result} = call ptr @nuis_fn_{callee}({call_args})"
        ));
        registers.insert(node.name.clone(), LlvmValueRef::OwnedBytes { blob: result });
        return Ok(true);
    }

    let merge_label = fresh_block(next_block, "select_owned_tree_merge");
    let mut incoming = Vec::<(String, String)>::new();
    emit_owned_select_tree(
        &args.tree,
        body,
        &condition_values,
        &owner_blobs,
        registers,
        helper_signatures,
        node,
        next_reg,
        next_block,
        &merge_label,
        None,
        &mut incoming,
    );
    body.push(format!("{merge_label}:"));
    let result = fresh_reg(next_reg);
    let incoming = incoming
        .iter()
        .map(|(blob, label)| format!("[ {blob}, %{label} ]"))
        .collect::<Vec<_>>()
        .join(", ");
    body.push(format!("  {result} = phi ptr {incoming}"));
    registers.insert(node.name.clone(), LlvmValueRef::OwnedBytes { blob: result });
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn emit_owned_select_tree(
    tree: &OwnedSelectTree<'_>,
    body: &mut Vec<String>,
    conditions: &BTreeMap<String, String>,
    owner_blobs: &[String],
    registers: &BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
    node: &Node,
    next_reg: &mut usize,
    next_block: &mut usize,
    merge_label: &str,
    current_label: Option<String>,
    incoming: &mut Vec<(String, String)>,
) {
    match tree {
        OwnedSelectTree::Owner(index) => {
            emit_unselected_owner_drops(*index, owner_blobs, body);
            body.push(format!("  br label %{merge_label}"));
            incoming.push((
                owner_blobs[*index].clone(),
                current_label.expect("owned select tree leaf must follow a branch label"),
            ));
        }
        OwnedSelectTree::Call {
            callee,
            owner,
            scalar_args,
        } => {
            emit_unselected_owner_drops(*owner, owner_blobs, body);
            let signature =
                branch_owned_helper_signature(node, callee, scalar_args.len(), helper_signatures)
                    .expect("owned tree helper signature was prevalidated");
            let scalar_args = lower_owned_tree_scalar_args(
                registers,
                scalar_args,
                &signature.params[1..],
                body,
                next_reg,
            )
            .expect("owned tree call inputs were prevalidated");
            let call_args = std::iter::once(format!("ptr {}", owner_blobs[*owner]))
                .chain(scalar_args)
                .collect::<Vec<_>>()
                .join(", ");
            let result = fresh_reg(next_reg);
            body.push(format!(
                "  {result} = call ptr @nuis_fn_{callee}({call_args})"
            ));
            body.push(format!("  br label %{merge_label}"));
            incoming.push((
                result,
                current_label.expect("owned select tree leaf must follow a branch label"),
            ));
        }
        OwnedSelectTree::If {
            condition,
            then_tree,
            else_tree,
        } => {
            let then_label = fresh_block(next_block, "select_owned_tree_then");
            let else_label = fresh_block(next_block, "select_owned_tree_else");
            body.push(format!(
                "  br i1 {}, label %{then_label}, label %{else_label}",
                conditions
                    .get(*condition)
                    .expect("owned select tree condition was prevalidated")
            ));
            body.push(format!("{then_label}:"));
            emit_owned_select_tree(
                then_tree,
                body,
                conditions,
                owner_blobs,
                registers,
                helper_signatures,
                node,
                next_reg,
                next_block,
                merge_label,
                Some(then_label),
                incoming,
            );
            body.push(format!("{else_label}:"));
            emit_owned_select_tree(
                else_tree,
                body,
                conditions,
                owner_blobs,
                registers,
                helper_signatures,
                node,
                next_reg,
                next_block,
                merge_label,
                Some(else_label),
                incoming,
            );
        }
    }
}

fn emit_unselected_owner_drops(selected: usize, owner_blobs: &[String], body: &mut Vec<String>) {
    for (index, blob) in owner_blobs.iter().enumerate() {
        if index != selected {
            body.push(format!(
                "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {blob})"
            ));
        }
    }
}

fn owned_select_tree_calls_ready(
    tree: &OwnedSelectTree<'_>,
    node: &Node,
    registers: &BTreeMap<String, LlvmValueRef>,
    helper_signatures: &BTreeMap<String, CpuHelperSignature>,
) -> Result<bool, String> {
    match tree {
        OwnedSelectTree::Owner(_) => Ok(true),
        OwnedSelectTree::Call {
            callee,
            scalar_args,
            ..
        } => {
            let signature =
                branch_owned_helper_signature(node, callee, scalar_args.len(), helper_signatures)?;
            Ok(owned_tree_scalar_args_ready(
                registers,
                scalar_args,
                &signature.params[1..],
            ))
        }
        OwnedSelectTree::If {
            then_tree,
            else_tree,
            ..
        } => Ok(
            owned_select_tree_calls_ready(then_tree, node, registers, helper_signatures)?
                && owned_select_tree_calls_ready(else_tree, node, registers, helper_signatures)?,
        ),
    }
}

fn lower_owned_bytes_select_with_cleanup(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<bool, String> {
    let Some(condition) = registers.get(&node.op.args[0]) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because its condition is outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };
    let Some(condition) = coerce_to_i64(condition, body, next_reg) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because its condition is not coercible to i64",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };
    let Some(LlvmValueRef::OwnedBytes { blob: then_blob }) = registers.get(&node.op.args[1]) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because its then owner is outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };
    let Some(LlvmValueRef::OwnedBytes { blob: else_blob }) = registers.get(&node.op.args[2]) else {
        body.push(format!(
            "  ; deferred lowering for cpu.{} `{}` because its else owner is outside the current CPU LLVM slice",
            node.op.instruction, node.name
        ));
        return Ok(true);
    };
    let then_blob = then_blob.clone();
    let else_blob = else_blob.clone();
    let condition_i1 = fresh_reg(next_reg);
    let result = fresh_reg(next_reg);
    let then_label = fresh_block(next_block, "select_owned_cleanup_then");
    let else_label = fresh_block(next_block, "select_owned_cleanup_else");
    let merge_label = fresh_block(next_block, "select_owned_cleanup_merge");
    body.push(format!("  {condition_i1} = icmp ne i64 {condition}, 0"));
    body.push(format!(
        "  br i1 {condition_i1}, label %{then_label}, label %{else_label}"
    ));
    body.push(format!("{then_label}:"));
    body.push(format!(
        "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {else_blob})"
    ));
    body.push(format!("  br label %{merge_label}"));
    body.push(format!("{else_label}:"));
    body.push(format!(
        "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {then_blob})"
    ));
    body.push(format!("  br label %{merge_label}"));
    body.push(format!("{merge_label}:"));
    body.push(format!(
        "  {result} = phi ptr [ {then_blob}, %{then_label} ], [ {else_blob}, %{else_label} ]"
    ));
    registers.insert(
        node.name.clone(),
        LlvmValueRef::OwnedBytes {
            blob: result.clone(),
        },
    );
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn lower_lazy_const_select(
    node: &Node,
    body: &mut Vec<String>,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    delayed_registers: &mut BTreeMap<String, String>,
    cond_value: Option<LlvmValueRef>,
    then_value: Option<LlvmValueRef>,
    else_value: Option<LlvmValueRef>,
    then_delayed: Option<String>,
    else_delayed: Option<String>,
    facts: &mut KnownFacts,
    next_reg: &mut usize,
    last_cpu_value: &mut Option<String>,
) -> Result<bool, String> {
    let Some(cond_value) = cond_value else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because its condition is outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    let Some(cond) = const_select_condition(&node.op.args[0], &cond_value, facts) else {
        let delayed_branches =
            delayed_select_branches(node, then_delayed.as_deref(), else_delayed.as_deref());
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because delayed branch lowering requires a compile-time constant condition ({delayed_branches})",
            node.name
        ));
        return Ok(true);
    };
    let (selected_name, selected_value, selected_delayed, unselected_name) = if cond {
        (
            node.op.args[1].as_str(),
            then_value,
            then_delayed,
            node.op.args[2].as_str(),
        )
    } else {
        (
            node.op.args[2].as_str(),
            else_value,
            else_delayed,
            node.op.args[1].as_str(),
        )
    };
    if let Some(reason) = selected_delayed {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because selected branch `{selected_name}` is delayed: {reason}",
            node.name
        ));
        return Ok(true);
    }
    let Some(selected) = selected_value else {
        body.push(format!(
            "  ; deferred lowering for cpu.select `{}` because selected branch `{selected_name}` is outside the current CPU LLVM slice",
            node.name
        ));
        return Ok(true);
    };
    delayed_registers.remove(unselected_name);
    registers.insert(node.name.clone(), selected.clone());
    record_known_selected_branch(node, selected_name, &selected, facts);
    if let Some(as_i64) = coerce_to_i64(&selected, body, next_reg) {
        *last_cpu_value = Some(as_i64);
    }
    Ok(true)
}

fn const_select_condition(
    cond_name: &str,
    value: &LlvmValueRef,
    facts: &KnownFacts,
) -> Option<bool> {
    if let Some(value) = facts.get_bool(cond_name) {
        return Some(value);
    }
    if let Some(value) = facts.get_i64(cond_name) {
        return Some(value != 0);
    }
    match value {
        LlvmValueRef::Bool { i1, .. } if i1 == "true" => Some(true),
        LlvmValueRef::Bool { i1, .. } if i1 == "false" => Some(false),
        LlvmValueRef::I64(value) => value.parse::<i64>().ok().map(|value| value != 0),
        _ => None,
    }
}

fn record_known_select_value(
    node: &Node,
    cond_value: &LlvmValueRef,
    then_value: &LlvmValueRef,
    else_value: &LlvmValueRef,
    facts: &mut KnownFacts,
) {
    let cond = const_select_condition(&node.op.args[0], cond_value, facts);
    if let Some(selected_name) = cond.map(|value| {
        if value {
            node.op.args[1].as_str()
        } else {
            node.op.args[2].as_str()
        }
    }) {
        let selected_value = if selected_name == node.op.args[1] {
            then_value
        } else {
            else_value
        };
        record_known_selected_branch(node, selected_name, selected_value, facts);
        return;
    }

    if let (Some(then_value), Some(else_value)) = (
        facts.get_i64(&node.op.args[1]),
        facts.get_i64(&node.op.args[2]),
    ) {
        if then_value == else_value {
            facts.record_i64(node.name.clone(), then_value);
        }
    }

    if let (Some(then_value), Some(else_value)) = (
        facts.get_bool(&node.op.args[1]),
        facts.get_bool(&node.op.args[2]),
    ) {
        if then_value == else_value {
            facts.record_bool(node.name.clone(), then_value);
        }
    }

    if let (LlvmValueRef::I64(then_i64), LlvmValueRef::I64(else_i64)) = (then_value, else_value) {
        if let (Ok(then_value), Ok(else_value)) = (then_i64.parse::<i64>(), else_i64.parse::<i64>())
        {
            if then_value == else_value {
                facts.record_i64(node.name.clone(), then_value);
            }
        }
    }
}

fn record_known_selected_branch(
    node: &Node,
    selected_name: &str,
    selected_value: &LlvmValueRef,
    facts: &mut KnownFacts,
) {
    if let Some(value) = facts.get_i64(selected_name) {
        facts.record_i64(node.name.clone(), value);
    }
    if let Some(value) = facts.get_bool(selected_name) {
        facts.record_bool(node.name.clone(), value);
    }
    if let Some(value) = facts.get_variant_type(selected_name).map(str::to_owned) {
        facts.record_variant_type(node.name.clone(), value);
    }
    if let LlvmValueRef::Struct(struct_value) = selected_value {
        for (field_name, _) in &struct_value.fields {
            let from = KnownFacts::struct_field_key(selected_name, field_name);
            let to = KnownFacts::struct_field_key(&node.name, field_name);
            if let Some(value) = facts.get_i64(&from) {
                facts.record_i64(to.clone(), value);
            }
            if let Some(value) = facts.get_bool(&from) {
                facts.record_bool(to.clone(), value);
            }
            if let Some(value) = facts.get_variant_type(&from).map(str::to_owned) {
                facts.record_variant_type(to.clone(), value);
            }
        }
    }
}

fn delayed_select_branches(
    node: &Node,
    then_delayed: Option<&str>,
    else_delayed: Option<&str>,
) -> String {
    let mut branches = Vec::new();
    if let Some(reason) = then_delayed {
        branches.push(format!("then `{}`: {reason}", node.op.args[1]));
    }
    if let Some(reason) = else_delayed {
        branches.push(format!("else `{}`: {reason}", node.op.args[2]));
    }
    branches.join("; ")
}
