use super::*;

pub(super) struct StoreProof {
    pub(super) has_projection: bool,
    needs_split: bool,
}

pub(super) fn prove(
    statement: &NirStmt,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
    locals: &LocalRecords,
) -> Option<StoreProof> {
    let NirStmt::Expr(NirExpr::StoreAt {
        buffer,
        index,
        value,
    }) = statement
    else {
        return None;
    };
    if !borrowed_buffer(buffer, scope) {
        return None;
    }
    let mut has_projection = false;
    let mut needs_split = false;
    for operand in [index.as_ref(), value.as_ref()] {
        bounded(operand)?;
        let proof = locals.scalar_proof(operand, calls);
        if arguments::contains_logic(&proof) {
            if !admitted(&proof, scope, live, calls, Kind::I64) {
                return None;
            }
            needs_split = true;
        } else if predicate_kind(&proof, scope, live, calls) != Some(Kind::I64) {
            return None;
        }
        has_projection |= proof.as_ref() != operand;
    }
    Some(StoreProof {
        has_projection,
        needs_split,
    })
}

pub(super) fn split(
    statement: &NirStmt,
    remaining: &mut VecDeque<NirStmt>,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    let proof = prove(statement, scope, live, &context.calls, locals)?;
    if !proof.needs_split {
        return None;
    }
    let mut store = statement.clone();
    let mut prefix = Vec::new();
    if !locals.is_empty() {
        // The store's own guard can be the first outline. Share leaf snapshots
        // with later stores and the checked return before introducing that guard.
        remaining.push_front(store);
        prefix =
            return_fields::stage_return_tail(remaining, template, scope, live, context, locals)
                .unwrap_or_default();
        store = remaining.pop_front().expect("prepended store");
    }
    let NirStmt::Expr(NirExpr::StoreAt { index, value, .. }) = &mut store else {
        unreachable!()
    };
    if prefix.is_empty() && proof.has_projection {
        prefix = record_literals::prepare_operands(
            [index.as_mut(), value.as_mut()],
            template,
            scope,
            live,
            context,
            locals,
        );
    }
    // Finish the index (including its selected reads) before starting the
    // value. Both guarded i64 initializers lead to the same final store.
    for operand in [index, value] {
        let name = fresh_left(template, scope, live, context);
        prefix.push(NirStmt::Let {
            value: std::mem::replace(operand.as_mut(), NirExpr::Var(name.clone())),
            name,
            ty: Some(NirTypeRef {
                name: "i64".into(),
                is_ref: false,
                is_optional: false,
                generic_args: vec![],
            }),
        });
    }
    prefix.push(store);
    Some(prefix)
}
