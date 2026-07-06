use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

struct NovaMetaLoweringEnv<'a> {
    current_domain: &'a str,
    bindings: &'a BTreeMap<String, NirTypeRef>,
    signatures: &'a BTreeMap<String, FunctionSignature>,
    struct_table: &'a BTreeMap<String, NirStructDef>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_meta_state_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let env = NovaMetaLoweringEnv {
        current_domain,
        bindings,
        signatures,
        struct_table,
    };
    let result = match callee {
        "nova_feedback_state" => build_state(
            args,
            "nova_feedback_state(...) expects 1 arg",
            "NovaFeedbackPacket",
            "NovaFeedbackState",
            ["status", "latency", "retries", "channel"],
            &env,
        )?,
        "nova_intent_state" => build_state(
            args,
            "nova_intent_state(...) expects 1 arg",
            "NovaIntentPacket",
            "NovaIntentState",
            ["kind", "target_slot", "urgency", "policy"],
            &env,
        )?,
        "nova_reaction_state" => build_state(
            args,
            "nova_reaction_state(...) expects 1 arg",
            "NovaReactionPacket",
            "NovaReactionState",
            ["kind", "result_slot", "stability", "echo_mode"],
            &env,
        )?,
        "nova_outcome_state" => build_state(
            args,
            "nova_outcome_state(...) expects 1 arg",
            "NovaOutcomePacket",
            "NovaOutcomeState",
            ["kind", "final_slot", "confidence", "settle_mode"],
            &env,
        )?,
        "nova_resolution_state" => build_state(
            args,
            "nova_resolution_state(...) expects 1 arg",
            "NovaResolutionPacket",
            "NovaResolutionState",
            ["kind", "commit_slot", "convergence", "policy_mode"],
            &env,
        )?,
        "nova_commit_state" => build_state(
            args,
            "nova_commit_state(...) expects 1 arg",
            "NovaCommitPacket",
            "NovaCommitState",
            ["kind", "applied_slot", "durability", "commit_mode"],
            &env,
        )?,
        "nova_snapshot_state" => build_state(
            args,
            "nova_snapshot_state(...) expects 1 arg",
            "NovaSnapshotPacket",
            "NovaSnapshotState",
            ["kind", "source_slot", "retention", "replay_mode"],
            &env,
        )?,
        "nova_checkpoint_state" => build_state(
            args,
            "nova_checkpoint_state(...) expects 1 arg",
            "NovaCheckpointPacket",
            "NovaCheckpointState",
            ["kind", "anchor_slot", "rollback_depth", "resume_mode"],
            &env,
        )?,
        "nova_selection_state" => build_state(
            args,
            "nova_selection_state(...) expects 1 arg",
            "NovaSelectionPacket",
            "NovaSelectionState",
            ["selected", "span", "mode", "origin"],
            &env,
        )?,
        "nova_list_selection"
        | "nova_table_selection"
        | "nova_tree_selection"
        | "nova_inspector_selection"
        | "nova_outline_selection" => build_selection_state(
            callee,
            args,
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        _ => return Ok(None),
    };
    Ok(Some(result))
}

fn build_state(
    args: &[AstExpr],
    arg_error: &str,
    packet_type: &str,
    state_type: &str,
    fields: [&str; 4],
    env: &NovaMetaLoweringEnv<'_>,
) -> Result<NirExpr, String> {
    let [packet] = args else {
        return Err(arg_error.to_owned());
    };
    let packet = lower_expr(
        packet,
        env.current_domain,
        env.bindings,
        env.signatures,
        env.struct_table,
        Some(&named_type(packet_type)),
    )?;
    Ok(NirExpr::StructLiteral {
        type_name: state_type.to_owned(),
        type_args: Vec::new(),
        fields: vec![
            (fields[0].to_owned(), field(packet.clone(), fields[0])),
            (fields[1].to_owned(), field(packet.clone(), fields[1])),
            (fields[2].to_owned(), field(packet.clone(), fields[2])),
            (fields[3].to_owned(), field(packet, fields[3])),
        ],
    })
}

fn build_selection_state(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [packet] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let (expected_type, selected_field, span_field, mode_field, origin) = match callee {
        "nova_list_selection" => ("NovaListPacket", "selected", "items", "dense", 0),
        "nova_table_selection" => ("NovaTablePacket", "selected_row", "rows", "zebra", 1),
        "nova_tree_selection" => ("NovaTreePacket", "selected", "nodes", "expanded", 2),
        "nova_inspector_selection" => ("NovaInspectorPacket", "selected", "fields", "pinned", 3),
        _ => ("NovaOutlinePacket", "selected", "items", "collapsed", 4),
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(NirExpr::StructLiteral {
        type_name: "NovaSelectionState".to_owned(),
        type_args: Vec::new(),
        fields: vec![
            ("selected".to_owned(), field(packet.clone(), selected_field)),
            ("span".to_owned(), field(packet.clone(), span_field)),
            ("mode".to_owned(), field(packet, mode_field)),
            ("origin".to_owned(), NirExpr::Int(origin)),
        ],
    })
}

fn field(base: NirExpr, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: field.to_owned(),
    }
}
