use super::*;

#[cfg(test)]
#[path = "effectful_selections_return_in_fields_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "effectful_selections_return_nested_in_fields_tests.rs"]
mod nested_tests;

#[cfg(test)]
#[path = "effectful_selections_return_prefix_in_fields_tests.rs"]
mod prefix_tests;

#[cfg(test)]
#[path = "effectful_selections_return_sibling_in_fields_tests.rs"]
mod sibling_tests;

#[cfg(test)]
#[path = "effectful_selections_return_tail_in_fields_tests.rs"]
mod tail_tests;

#[cfg(test)]
#[path = "effectful_selections_return_child_tail_in_fields_tests.rs"]
mod child_tail_tests;

#[cfg(test)]
#[path = "effectful_selections_return_pair_tail_in_fields_tests.rs"]
mod pair_tail_tests;

#[cfg(test)]
#[path = "effectful_selections_return_triple_tail_in_fields_tests.rs"]
mod triple_tail_tests;

#[cfg(test)]
#[path = "effectful_selections_return_quad_tail_in_fields_tests.rs"]
mod quad_tail_tests;

pub(super) fn candidate(body: &[NirStmt]) -> bool {
    body.iter().any(|s| {
        matches!(s, NirStmt::If { then_body, else_body, .. }
        if then_body.iter().chain(else_body).any(|s| matches!(s,
            NirStmt::If { then_body, else_body, .. } if then_body.iter().chain(else_body)
                .any(|s| matches!(s, NirStmt::Return(_))))))
    })
}

fn flat(body: &[NirStmt]) -> bool {
    body.iter()
        .all(|s| matches!(s, NirStmt::Let { .. } | NirStmt::Const { .. }))
}

fn continuing(body: &[NirStmt]) -> bool {
    // Whole-source preflight bounds recursion before this shape check. Nested
    // selections may continue, but cannot grant a second source exit.
    body.iter().all(|statement| match statement {
        NirStmt::Let { .. } | NirStmt::Const { .. } => true,
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => continuing(then_body) && continuing(else_body),
        _ => false,
    })
}

struct Exit<'a> {
    before: &'a [NirStmt],
    after: &'a [NirStmt],
    condition: &'a NirExpr,
    value: &'a NirExpr,
}

fn exit(body: &[NirStmt]) -> Option<Exit<'_>> {
    // Continuing selections belong to the prefix, not to exit authority.
    let index = body
        .iter()
        .position(|s| !continuing(std::slice::from_ref(s)))?;
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &body[index]
    else {
        return None;
    };
    let [NirStmt::Return(Some(value))] = then_body.as_slice() else {
        return None;
    };
    let before = &body[..index];
    let after = &body[index + 1..];
    (else_body.is_empty() && continuing(before) && continuing(after)).then_some(Exit {
        before,
        after,
        condition,
        value,
    })
}

pub(super) fn prove(
    body: &[NirStmt],
    scope: &Scope,
    signatures: &Signatures,
    budget: &mut nested::Budget,
    depth: usize,
) -> Option<ReturnedValue> {
    let mut remaining = 64;
    early_fields::preflight(body, &mut remaining, depth)?;
    let (result, prefix) = returned(body)?;
    let index = prefix
        .iter()
        .position(|s| matches!(s, NirStmt::If { .. }))?;
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &prefix[index]
    else {
        return None;
    };
    let (selected, split, other) = match (exit(then_body), exit(else_body)) {
        (Some(split), None) if continuing(else_body) => (true, split, else_body.as_slice()),
        (None, Some(split)) if continuing(then_body) => (false, split, then_body.as_slice()),
        _ => return None,
    };
    if !flat(&prefix[..index]) || !continuing(&prefix[index + 1..]) {
        return None;
    }
    let mut staged = stage(&prefix[..index], scope, signatures, budget, depth)?;
    // Removing a proved control-only exit is for write classification only.
    // Original work is subsequently proved/charged from its original slices.
    let cleaned = split
        .before
        .iter()
        .chain(split.after)
        .cloned()
        .collect::<Vec<_>>();
    let (yes, no) = if selected {
        (cleaned.as_slice(), other)
    } else {
        (other, cleaned.as_slice())
    };
    let fields = updates::private_field_certificate(yes, no, scope, &staged, depth)?;
    nested::consume_node(budget)?;
    let (source_condition, reads, calls) =
        predicates::prove(condition, &staged.local, signatures, budget)?;
    let mut bindings = staged.local.keys().cloned().collect();
    branches::collect_bindings(body, &mut bindings);
    let gate = branches::fresh_name("__nuis_effect_field_exit_gate", &mut bindings);
    staged
        .inputs
        .extend(reads.difference(&staged.defined).cloned());
    staged.calls.extend(calls);
    staged.local.insert(gate.clone(), scalar_type("bool"));
    staged.defined.insert(gate.clone());
    staged.constants.insert(gate.clone());
    staged.statements.push(regions::Statement::Logical(
        NirStmt::Let {
            name: gate.clone(),
            ty: Some(scalar_type("bool")),
            value: NirExpr::Bool(false),
        },
        source_condition,
    ));
    let (yes, no) = if selected {
        (split.before, other)
    } else {
        (other, split.before)
    };
    fields.split(
        &gate,
        yes,
        no,
        &prefix[index + 1..],
        &mut staged,
        signatures,
        budget,
        depth,
    )?;

    nested::consume_node(budget)?;
    let (exit_condition, exit_reads, exit_calls) =
        predicates::prove(split.condition, &staged.local, signatures, budget)?;
    if exit_reads.len() > 31 {
        return None;
    }
    let params = exit_reads
        .iter()
        .map(|name| {
            Some(NirParam {
                name: name.clone(),
                ty: staged.local.get(name)?.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    // Gate the source exit predicate with the once-bound original arm choice.
    // This generated edge is transport, not another original logical operator.
    let condition = predicates::Predicate::Logical {
        left: Box::new(predicates::Predicate::Scalar(NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(NirExpr::Var(gate.clone())),
            rhs: Box::new(NirExpr::Bool(selected)),
        })),
        right: Box::new(exit_condition),
        selected: true,
        params,
    };
    nested::consume_node(budget)?;
    let (early, early_reads, early_calls) =
        predicates::prove(split.value, &staged.local, signatures, budget)?;
    let mut tail = stage_authorized(&[], &staged.local, signatures, budget, depth + 1, None)?;
    let (yes, no) = if selected {
        (split.after, &[][..])
    } else {
        (&[][..], split.after)
    };
    fields.split(
        &gate,
        yes,
        no,
        &prefix[index + 1..],
        &mut tail,
        signatures,
        budget,
        depth,
    )?;
    fields.tail(
        &prefix[index + 1..],
        &staged.local,
        &mut tail,
        signatures,
        budget,
        depth + 1,
    )?;
    nested::consume_node(budget)?;
    let (value, value_reads, value_calls) =
        predicates::prove(result, &tail.local, signatures, budget)?;
    tail.inputs
        .extend(value_reads.difference(&tail.defined).cloned());
    tail.calls.extend(value_calls);
    let all_reads = exit_reads
        .union(&early_reads)
        .cloned()
        .chain(tail.inputs.iter().cloned())
        .collect::<BTreeSet<_>>();
    staged
        .inputs
        .extend(all_reads.difference(&staged.defined).cloned());
    staged.calls.extend(exit_calls);
    staged.calls.extend(early_calls.iter().cloned());
    staged.calls.extend(tail.calls.iter().cloned());
    let plan = super::super::plan(
        condition,
        ReturnedValue {
            value: SelectedValue::Predicate(early),
            inputs: early_reads,
            calls: early_calls,
        },
        ReturnedValue {
            value: SelectedValue::StagedPredicate(tail.statements, value),
            inputs: tail.inputs,
            calls: tail.calls,
        },
        &staged.local,
    )?;
    Some(ReturnedValue {
        value: SelectedValue::StagedNested(staged.statements, Box::new(plan)),
        inputs: staged.inputs,
        calls: staged.calls,
    })
}
