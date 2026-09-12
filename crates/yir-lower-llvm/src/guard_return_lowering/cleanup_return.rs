use std::collections::BTreeMap;

use yir_core::{Node, OwnedStructLayout};

use super::GuardReturnLoweringOutcome;
use crate::{
    call_lowering::owned_struct_layout_template,
    call_return::{can_emit_typed_return_from_value, emit_typed_return_from_value},
    fresh_block, fresh_reg,
    task_owned_payload::{
        can_emit_owned_struct_return, emit_owned_struct_return, materialize_owned_value,
    },
    value_ref::coerce_to_i64,
    CpuCallScalarKind, LlvmValueRef,
};

enum PreparationError {
    Unavailable(String),
    Contract(&'static str),
}

impl std::fmt::Display for PreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(message) => formatter.write_str(message),
            Self::Contract(message) => formatter.write_str(message),
        }
    }
}

pub(super) fn lower(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    next_reg: &mut usize,
    next_block: &mut usize,
    kind: CpuCallScalarKind,
    layout: Option<&OwnedStructLayout>,
) -> Result<Option<GuardReturnLoweringOutcome>, String> {
    let terminal = match node.op.instruction.as_str() {
        "guard_drop_owned_bytes_return" => false,
        "branch_drop_owned_bytes_return" => true,
        _ => return Ok(None),
    };
    let expected = if terminal { 5 } else { 3 };
    if node.op.args.len() != expected {
        return Err(format!(
            "{} `{}` expects condition and {} bytes/return pair(s)",
            node.op.full_name(),
            node.name,
            (expected - 1) / 2
        ));
    }
    let prepared = (|| {
        let values = node
            .op
            .args
            .iter()
            .map(|name| {
                registers.get(name).ok_or_else(|| {
                    PreparationError::Unavailable(format!(
                        "input `{name}` is outside the current CPU LLVM slice"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut branches = Vec::new();
        // Both arms must have a valid ABI before emitting either branch or cleanup.
        for pair in values[1..].chunks_exact(2) {
            let LlvmValueRef::OwnedBytes { blob } = pair[0] else {
                return Err(PreparationError::Unavailable(
                    "cleanup input is not owned Bytes".to_owned(),
                ));
            };
            let returned = if let Some(layout) = layout {
                if kind != CpuCallScalarKind::I64 {
                    return Err(PreparationError::Contract(
                        "aggregate function return ABI must be i64",
                    ));
                }
                let template = LlvmValueRef::Struct(owned_struct_layout_template(layout.clone()));
                let Some(LlvmValueRef::Struct(value)) = materialize_owned_value(pair[1], &template)
                else {
                    return Err(PreparationError::Contract(
                        "value does not match its function return layout",
                    ));
                };
                if !can_emit_owned_struct_return(&value) {
                    return Err(PreparationError::Contract(
                        "value is outside the owned aggregate return ABI",
                    ));
                }
                LlvmValueRef::Struct(value)
            } else {
                if matches!(
                    pair[1],
                    LlvmValueRef::Struct(_) | LlvmValueRef::VariantUnion(_)
                ) {
                    return Err(PreparationError::Contract(
                        "aggregate value requires a function return layout",
                    ));
                }
                if !can_emit_typed_return_from_value(kind, pair[1]) {
                    return Err(PreparationError::Unavailable(
                        "value is outside its scalar function return ABI".to_owned(),
                    ));
                }
                pair[1].clone()
            };
            if contains_blob(&returned, blob) {
                return Err(PreparationError::Contract(
                    "return value contains the Bytes being dropped",
                ));
            }
            branches.push((blob.clone(), returned));
        }
        Ok((values[0].clone(), branches))
    })();
    let (condition, branches) = match prepared {
        Ok(prepared) => prepared,
        Err(PreparationError::Unavailable(reason)) if layout.is_none() => {
            body.push(format!(
                "  ; deferred lowering for {} `{}` because {reason}",
                node.op.full_name(),
                node.name
            ));
            return Ok(Some(GuardReturnLoweringOutcome::Continue));
        }
        Err(reason) => return Err(format!("{} `{}`: {reason}", node.op.full_name(), node.name)),
    };
    let Some(condition) = coerce_to_i64(&condition, body, next_reg) else {
        if layout.is_some() {
            return Err(format!(
                "{} `{}` has an unavailable condition for aggregate return",
                node.op.full_name(),
                node.name
            ));
        }
        body.push(format!(
            "  ; deferred lowering for {} `{}` because its condition is not coercible to i64",
            node.op.full_name(),
            node.name
        ));
        return Ok(Some(GuardReturnLoweringOutcome::Continue));
    };
    let predicate = fresh_reg(next_reg);
    body.push(format!("  {predicate} = icmp ne i64 {condition}, 0"));
    let then_label = fresh_block(
        next_block,
        if terminal {
            "branch_drop_bytes_return_then"
        } else {
            "guard_drop_bytes_return_then"
        },
    );
    let other_label = fresh_block(
        next_block,
        if terminal {
            "branch_drop_bytes_return_else"
        } else {
            "guard_drop_bytes_return_cont"
        },
    );
    body.push(format!(
        "  br i1 {predicate}, label %{then_label}, label %{other_label}"
    ));
    body.push(format!("{then_label}:"));
    emit_return(&branches[0], kind, body, next_reg);
    body.push(format!("{other_label}:"));
    if terminal {
        emit_return(&branches[1], kind, body, next_reg);
        Ok(Some(GuardReturnLoweringOutcome::TerminalReturn))
    } else {
        Ok(Some(GuardReturnLoweringOutcome::Continue))
    }
}

fn contains_blob(value: &LlvmValueRef, dropped: &str) -> bool {
    match value {
        LlvmValueRef::OwnedBytes { blob } => blob == dropped,
        LlvmValueRef::Struct(value) => value
            .fields
            .iter()
            .any(|(_, value)| contains_blob(value, dropped)),
        _ => false,
    }
}

fn emit_return(
    (blob, returned): &(String, LlvmValueRef),
    kind: CpuCallScalarKind,
    body: &mut Vec<String>,
    next_reg: &mut usize,
) {
    body.push(format!(
        "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {blob})"
    ));
    let emitted = if let LlvmValueRef::Struct(value) = returned {
        emit_owned_struct_return(value, body, next_reg)
    } else {
        emit_typed_return_from_value(body, next_reg, kind, returned)
    };
    assert!(
        emitted,
        "cleanup return ABI was checked before emitting branches"
    );
}
