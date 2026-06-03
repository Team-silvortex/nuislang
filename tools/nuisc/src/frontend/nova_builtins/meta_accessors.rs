use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_meta_accessor_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let Some((expected_type, field_name)) = meta_state_accessor_target(callee) else {
        return Ok(None);
    };
    let [state] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let state = lower_expr(
        state,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(state),
        field: field_name.to_owned(),
    }))
}

fn meta_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_feedback_state_status" => ("NovaFeedbackState", "status"),
        "nova_feedback_state_latency" => ("NovaFeedbackState", "latency"),
        "nova_feedback_state_retries" => ("NovaFeedbackState", "retries"),
        "nova_feedback_state_channel" => ("NovaFeedbackState", "channel"),
        "nova_intent_state_kind" => ("NovaIntentState", "kind"),
        "nova_intent_state_target" => ("NovaIntentState", "target_slot"),
        "nova_intent_state_urgency" => ("NovaIntentState", "urgency"),
        "nova_intent_state_policy" => ("NovaIntentState", "policy"),
        "nova_reaction_state_kind" => ("NovaReactionState", "kind"),
        "nova_reaction_state_result" => ("NovaReactionState", "result_slot"),
        "nova_reaction_state_stability" => ("NovaReactionState", "stability"),
        "nova_reaction_state_echo_mode" => ("NovaReactionState", "echo_mode"),
        "nova_outcome_state_kind" => ("NovaOutcomeState", "kind"),
        "nova_outcome_state_final" => ("NovaOutcomeState", "final_slot"),
        "nova_outcome_state_confidence" => ("NovaOutcomeState", "confidence"),
        "nova_outcome_state_settle_mode" => ("NovaOutcomeState", "settle_mode"),
        "nova_resolution_state_kind" => ("NovaResolutionState", "kind"),
        "nova_resolution_state_commit" => ("NovaResolutionState", "commit_slot"),
        "nova_resolution_state_convergence" => ("NovaResolutionState", "convergence"),
        "nova_resolution_state_policy_mode" => ("NovaResolutionState", "policy_mode"),
        "nova_commit_state_kind" => ("NovaCommitState", "kind"),
        "nova_commit_state_applied" => ("NovaCommitState", "applied_slot"),
        "nova_commit_state_durability" => ("NovaCommitState", "durability"),
        "nova_commit_state_commit_mode" => ("NovaCommitState", "commit_mode"),
        "nova_snapshot_state_kind" => ("NovaSnapshotState", "kind"),
        "nova_snapshot_state_source" => ("NovaSnapshotState", "source_slot"),
        "nova_snapshot_state_retention" => ("NovaSnapshotState", "retention"),
        "nova_snapshot_state_replay_mode" => ("NovaSnapshotState", "replay_mode"),
        "nova_checkpoint_state_kind" => ("NovaCheckpointState", "kind"),
        "nova_checkpoint_state_anchor" => ("NovaCheckpointState", "anchor_slot"),
        "nova_checkpoint_state_rollback_depth" => ("NovaCheckpointState", "rollback_depth"),
        "nova_checkpoint_state_resume_mode" => ("NovaCheckpointState", "resume_mode"),
        "nova_selection_state_selected" => ("NovaSelectionState", "selected"),
        "nova_selection_state_span" => ("NovaSelectionState", "span"),
        "nova_selection_state_mode" => ("NovaSelectionState", "mode"),
        "nova_selection_state_origin" => ("NovaSelectionState", "origin"),
        _ => return None,
    })
}
