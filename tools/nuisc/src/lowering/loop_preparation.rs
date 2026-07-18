use super::*;
use crate::lowering::loop_carries::{
    diagnose_unsupported_loop_carry_expr, loop_compare_from_binary_op,
    loop_state_ref_into_carry_source, loop_state_ref_into_cond_source,
    parse_loop_carry_branch_source, parse_loop_carry_linear,
    parse_prepared_dynamic_read_carry_source, parse_prepared_fixed_read_carry_source,
    parse_prepared_loop_state_ref_expr, parse_prepared_loop_state_ref_name,
    unsupported_loop_carry_branch_source_message,
};
use crate::lowering::loop_purity::{
    normalize_pure_bool_test_expr, substitute_branch_binding, substitute_stmt_bindings,
};

#[path = "loop_preparation_flow.rs"]
mod loop_preparation_flow;
use loop_preparation_flow::{
    diagnose_unstructured_loop_flow_control, parse_loop_flow_condition, parse_loop_flow_control,
    parse_prepared_async_loop_step, parse_prepared_loop_header, parse_prepared_loop_step,
};

#[path = "loop_preparation_carry_tree.rs"]
mod loop_preparation_carry_tree;
use loop_preparation_carry_tree::{
    collapse_carry_decision_tree, collect_loop_carry_binding_names,
    diagnose_future_carry_reference, diagnose_unsupported_stmt_carry_tree,
    extract_non_temp_loop_carry_name, extract_single_stmt_carry_name,
    normalize_pure_stmt_prefix_body, parse_helper_conditional_carry_update,
    parse_stmt_carry_decision_tree, PreparedConditionalTempBinding,
};

#[path = "loop_preparation_refs.rs"]
mod loop_preparation_refs;
use loop_preparation_refs::stmt_references_any_name;

#[path = "loop_preparation_carry_updates.rs"]
mod loop_preparation_carry_updates;
use loop_preparation_carry_updates::{
    diagnose_unsupported_loop_carry_update, parse_conditional_temp_binding,
    parse_derived_conditional_temp_binding, parse_loop_carry_update,
};

#[path = "loop_preparation_delta.rs"]
mod loop_preparation_delta;
use loop_preparation_delta::parse_conditional_temp_driven_loop_carry_update;
pub(in crate::lowering) use loop_preparation_delta::parse_loop_carry_delta_branch_source;

#[path = "loop_preparation_temps.rs"]
mod loop_preparation_temps;
use loop_preparation_temps::{
    extract_loop_match_scrutinee_temp_binding, is_loop_match_scrutinee_temp_binding,
    split_temp_prefixed_loop_flow_control, split_temp_prefixed_loop_step_bindings,
    split_trailing_loop_control_temp_bindings,
};

fn prepare_loop_carry_sequence(
    stmts: &[NirStmt],
    binding_name: &str,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<Vec<PreparedCarryUpdate>> {
    let carry_names =
        collect_loop_carry_binding_names(stmts, pure_helpers, inlineable_pure_helpers)?;
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let mut conditional_temps = BTreeMap::<String, PreparedConditionalTempBinding>::new();
    let mut carry_index = 0usize;
    for stmt in stmts {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(current_carry_name) =
            extract_non_temp_loop_carry_name(stmt, pure_helpers, inlineable_pure_helpers)
        {
            if diagnose_future_carry_reference(
                &substituted,
                &current_carry_name,
                &carry_names[carry_index + 1..],
            )
            .is_some()
            {
                return None;
            }
            carry_index += 1;
        }
        if let Some(prepared) = parse_loop_carry_update(
            &substituted,
            binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(prepared) = parse_conditional_temp_driven_loop_carry_update(
            &substituted,
            binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(temp) = parse_conditional_temp_binding(
            &substituted,
            binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        if let Some(temp) = parse_derived_conditional_temp_binding(
            &substituted,
            binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        let (temp_name, temp_expr) = extract_pure_branch_binding(&substituted, pure_helpers)?;
        if !is_loop_match_scrutinee_temp_binding(&temp_name) {
            return None;
        }
        temp_bindings.push((temp_name, temp_expr));
    }
    Some(carries)
}

pub(super) fn diagnose_unsupported_prepared_while_carry(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<String> {
    let (binding_name, _, _) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (step_temp_bindings, step_binding, carry_bindings) =
        split_temp_prefixed_loop_step_bindings(
            body,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
    if carry_bindings.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &step_temp_bindings);
    let sync_step = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    );
    let async_step = parse_prepared_async_loop_step(&substituted_step, &binding_name);
    if sync_step.is_none() && async_step.is_none() {
        return None;
    }
    let substituted_carry_bindings = carry_bindings
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &step_temp_bindings))
        .collect::<Vec<_>>();

    let carry_names = collect_loop_carry_binding_names(
        &substituted_carry_bindings,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let mut carries = Vec::<PreparedCarryUpdate>::new();
    let mut temp_bindings = Vec::<(String, NirExpr)>::new();
    let mut conditional_temps = BTreeMap::<String, PreparedConditionalTempBinding>::new();
    let mut carry_index = 0usize;
    for stmt in &substituted_carry_bindings {
        let substituted = substitute_stmt_bindings(stmt, &temp_bindings);
        if let Some(current_carry_name) =
            extract_non_temp_loop_carry_name(stmt, pure_helpers, inlineable_pure_helpers)
        {
            if let Some(diagnostic) = diagnose_future_carry_reference(
                &substituted,
                &current_carry_name,
                &carry_names[carry_index + 1..],
            ) {
                return Some(diagnostic);
            }
            carry_index += 1;
        }
        if let Some(diagnostic) = diagnose_unsupported_loop_carry_update(
            &substituted,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        ) {
            return Some(diagnostic);
        }
        if let Some(prepared) = parse_loop_carry_update(
            &substituted,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(prepared) = parse_conditional_temp_driven_loop_carry_update(
            &substituted,
            &binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            carries.push(prepared);
            continue;
        }
        if let Some(temp) = parse_conditional_temp_binding(
            &substituted,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        if let Some(temp) = parse_derived_conditional_temp_binding(
            &substituted,
            &binding_name,
            &carries,
            &conditional_temps,
            pure_helpers,
            inlineable_pure_helpers,
        ) {
            conditional_temps.insert(temp.binding_name.clone(), temp);
            continue;
        }
        let (temp_name, temp_expr) = extract_pure_branch_binding(&substituted, pure_helpers)?;
        if !is_loop_match_scrutinee_temp_binding(&temp_name) {
            return None;
        }
        temp_bindings.push((temp_name, temp_expr));
    }
    None
}

pub(super) fn diagnose_unstructured_while_shape(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    let (binding_name, _, _) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;

    let Some((step_temp_bindings, step_binding, rest)) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    ) else {
        let Some((first_stmt, _)) = body.split_first() else {
            return Some(
                "structured `while` lowering recognized the loop header, but the body is empty; expected a loop-state step, a guarded terminal body, or a structured carry/control sequence"
                    .to_owned(),
            );
        };
        let first_binding_name = match first_stmt {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => Some(name.as_str()),
            _ => None,
        };
        return Some(match first_binding_name {
            Some(name) => format!(
                "structured `while` lowering recognized loop state `{binding_name}`, but the first body binding `{name}` is not a supported temp/step prefix; expected pure temp bindings followed by `{binding_name}` updated via `{binding_name} +/- ...` or `await callee({binding_name})`"
            ),
            None => format!(
                "structured `while` lowering recognized loop state `{binding_name}`, but the body does not begin with a supported step prefix; expected pure temp bindings followed by `let {binding_name} = {binding_name} +/- ...`, `let {binding_name} = await callee({binding_name})`, or a guarded terminal body"
            ),
        });
    };
    let substituted_step = substitute_stmt_bindings(step_binding, &step_temp_bindings);
    let sync_step = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    );
    let async_step = parse_prepared_async_loop_step(&substituted_step, &binding_name);
    if sync_step.is_none() && async_step.is_none() {
        return Some(
            format!(
                "structured `while` lowering recognized loop state `{binding_name}`, but the step binding is not supported after the pure temp prefix; expected `{binding_name}` to be updated via `{binding_name} +/- ...` or `await callee({binding_name})`"
            ),
        );
    }

    if rest.is_empty() {
        if async_step.is_some() {
            return Some(
                "structured async `while` lowering recognized the loop header and awaited step, but the remaining body is empty; expected a structured carry/control sequence after the async step"
                    .to_owned(),
            );
        }
        return None;
    }

    let substituted_rest = rest
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &step_temp_bindings))
        .collect::<Vec<_>>();

    if substituted_rest.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Print(_)
                | NirStmt::Expr(_)
                | NirStmt::Await(_)
                | NirStmt::Return(_)
                | NirStmt::While { .. }
        )
    }) {
        return Some(format!(
            "structured `while` lowering recognized loop state `{binding_name}` and its step, but the remaining body still contains arbitrary executable statements; only pure temp prefixes before the step plus structured carry updates and flow/post-flow control after the step are lowered"
        ));
    }

    let carries = prepare_loop_carry_sequence(
        &substituted_rest,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        &BTreeMap::<String, PureHelperBlock>::new(),
    )
    .unwrap_or_default();
    if let Some(diagnostic) = substituted_rest.iter().find_map(|stmt| {
        diagnose_unstructured_loop_flow_control(
            stmt,
            &binding_name,
            &carries,
            pure_helpers,
            inlineable_pure_helpers,
        )
    }) {
        return Some(diagnostic);
    }

    Some(format!(
        "structured `while` lowering recognized loop state `{binding_name}` and its step, but the remaining body is not reducible to supported carry updates or flow/post-flow control"
    ))
}

#[path = "loop_preparation_entries.rs"]
mod loop_preparation_entries;
pub(super) use loop_preparation_entries::{
    prepare_async_chained_while, prepare_async_flow_while, prepare_async_post_flow_while,
    prepare_chained_while, prepare_counted_while, prepare_flow_while, prepare_post_flow_while,
};

#[cfg(test)]
#[path = "loop_preparation_tests.rs"]
mod tests;
