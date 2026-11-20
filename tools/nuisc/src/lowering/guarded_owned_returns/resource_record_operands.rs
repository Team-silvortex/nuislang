use super::*;

#[cfg(test)]
#[path = "resource_record_condition_tests.rs"]
mod condition_tests;
#[cfg(test)]
#[path = "resource_record_scalar_initializers_tests.rs"]
mod scalar_tests;
#[cfg(test)]
#[path = "resource_record_operands_tests.rs"]
mod tests;

pub(super) fn split_scalar(
    statement: &NirStmt,
    ty: &NirTypeRef,
    value: &NirExpr,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    capture_scalar(ty).then_some(())?;
    bounded(value)?;
    let proof = for_proof(value, locals, &context.calls);
    let expected = if ty.name == "bool" {
        Kind::Bool
    } else {
        Kind::I64
    };
    if !arguments::contains_logic(&proof)
        || !admitted(&proof, scope, live, &context.calls, expected)
    {
        return None;
    }
    let (value, mut prefix, _) =
        liveness::prepare(value, ty, template, scope, live, context, locals);
    // No local projections means the original scalar route owns this statement.
    // This also prevents requeued declarations or conditions from rewriting again.
    if prefix.is_empty() {
        return None;
    }
    let mut rewritten = statement.clone();
    let target = match &mut rewritten {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
        NirStmt::If { condition, .. } => condition,
        _ => unreachable!(),
    };
    *target = value;
    prefix.push(rewritten);
    Some(prefix)
}

// Type witnesses are only used by admission, never emitted or constant-folded.
// The caller bounds the original tree before substituting any projection.
pub(super) fn for_proof(value: &NirExpr, locals: &LocalRecords, calls: &PureValueCalls) -> NirExpr {
    if let Some(ty) = locals.value_type(value, calls).filter(capture_scalar) {
        return if ty.name == "bool" {
            NirExpr::Bool(false)
        } else {
            NirExpr::Int(0)
        };
    }
    children(value, |child| for_proof(child, locals, calls))
}

// Rebuild syntax without evaluating, extracting or reordering any operand.
pub(super) fn children(value: &NirExpr, mut visit: impl FnMut(&NirExpr) -> NirExpr) -> NirExpr {
    match value {
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(visit(lhs)),
            rhs: Box::new(visit(rhs)),
        },
        NirExpr::Call { callee, args } => NirExpr::Call {
            callee: callee.clone(),
            args: args.iter().map(&mut visit).collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(visit(base)),
            field: field.clone(),
        },
        NirExpr::LoadAt { buffer, index } => NirExpr::LoadAt {
            buffer: Box::new(visit(buffer)),
            index: Box::new(visit(index)),
        },
        NirExpr::BytesLen(value) => NirExpr::BytesLen(Box::new(visit(value))),
        NirExpr::BufferLen(value) => NirExpr::BufferLen(Box::new(visit(value))),
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), visit(value)))
                .collect(),
        },
        _ => value.clone(),
    }
}
