use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_meta_packet_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let (type_name, fields) = match callee {
        "nova_feedback_packet" => (
            "NovaFeedbackPacket",
            [
                ("status", 0),
                ("latency", 1),
                ("retries", 2),
                ("channel", 3),
            ],
        ),
        "nova_intent_packet" => (
            "NovaIntentPacket",
            [
                ("kind", 0),
                ("target_slot", 1),
                ("urgency", 2),
                ("policy", 3),
            ],
        ),
        "nova_reaction_packet" => (
            "NovaReactionPacket",
            [
                ("kind", 0),
                ("result_slot", 1),
                ("stability", 2),
                ("echo_mode", 3),
            ],
        ),
        "nova_outcome_packet" => (
            "NovaOutcomePacket",
            [
                ("kind", 0),
                ("final_slot", 1),
                ("confidence", 2),
                ("settle_mode", 3),
            ],
        ),
        "nova_resolution_packet" => (
            "NovaResolutionPacket",
            [
                ("kind", 0),
                ("commit_slot", 1),
                ("convergence", 2),
                ("policy_mode", 3),
            ],
        ),
        "nova_commit_packet" => (
            "NovaCommitPacket",
            [
                ("kind", 0),
                ("applied_slot", 1),
                ("durability", 2),
                ("commit_mode", 3),
            ],
        ),
        "nova_snapshot_packet" => (
            "NovaSnapshotPacket",
            [
                ("kind", 0),
                ("source_slot", 1),
                ("retention", 2),
                ("replay_mode", 3),
            ],
        ),
        "nova_checkpoint_packet" => (
            "NovaCheckpointPacket",
            [
                ("kind", 0),
                ("anchor_slot", 1),
                ("rollback_depth", 2),
                ("resume_mode", 3),
            ],
        ),
        _ => return Ok(None),
    };

    let [a0, a1, a2, a3] = args else {
        return Err(format!("{callee}(...) expects 4 args"));
    };
    let values = [
        lower_i64(a0, current_domain, bindings, signatures, struct_table)?,
        lower_i64(a1, current_domain, bindings, signatures, struct_table)?,
        lower_i64(a2, current_domain, bindings, signatures, struct_table)?,
        lower_i64(a3, current_domain, bindings, signatures, struct_table)?,
    ];
    Ok(Some(NirExpr::StructLiteral {
        type_name: type_name.to_owned(),
        type_args: Vec::new(),
        fields: fields
            .into_iter()
            .map(|(field, index)| (field.to_owned(), values[index].clone()))
            .collect(),
    }))
}

fn lower_i64(
    expr: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    lower_expr(
        expr,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&i64_type()),
    )
}
