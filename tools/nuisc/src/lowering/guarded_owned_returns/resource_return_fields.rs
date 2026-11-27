use super::*;

#[cfg(test)]
#[path = "resource_return_fields_tests.rs"]
mod tests;

pub(super) fn collect(
    module: &NirModule,
    eligible: &BTreeSet<String>,
) -> BTreeMap<String, MixedResourceLayout> {
    module
        .functions
        .iter()
        .filter(|function| eligible.contains(&function.name))
        .filter_map(|function| {
            let ty = function.return_type.as_ref()?;
            if ty.is_ref || ty.is_optional {
                return None;
            }
            let encoded = module_owned_struct_layout(module, ty)?;
            let layout = match ty.generic_args.as_slice() {
                [] => MixedResourceLayout::parse(&encoded).ok()?,
                [state, error]
                    if !state.is_ref
                        && !state.is_optional
                        && state.generic_args.is_empty()
                        && root_scalar(error) =>
                {
                    MixedResourceLayout::parse_outcome(&encoded, &state.name, &ty.name).ok()?
                }
                _ => return None,
            };
            Some((ty.render(), layout))
        })
        .collect()
}

pub(super) fn split(
    value: &NirExpr,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    // One budget covers the original aggregate, not a new budget per field.
    bounded_tree(value, true)?;
    let return_type = template.return_type.as_ref()?;
    let layout = context.return_layouts.get(&return_type.render())?;
    let (record, constructor) = match (&layout.outcome, value) {
        (None, record @ NirExpr::StructLiteral { .. }) => (record, None),
        (
            Some(outcome),
            NirExpr::StructLiteral {
                type_name,
                type_args,
                fields,
            },
        ) if *type_args == return_type.generic_args
            && fields.len() == 1
            && fields[0].0 == "value" =>
        {
            if *type_name == outcome.err_variant {
                let code = &fields[0].1;
                let proof = locals.scalar_proof(code, &context.calls);
                if !admitted(&proof, scope, live, &context.calls, Kind::I64)
                    || !arguments::contains_logic(&proof)
                {
                    return None;
                }
                // Requeue the original code, not its type witnesses. Its scalar
                // declaration freezes completed projections before terminal cleanup.
                let name = fresh_left(template, scope, live, context);
                let mut wrapper = value.clone();
                let NirExpr::StructLiteral { fields, .. } = &mut wrapper else {
                    unreachable!()
                };
                fields[0].1 = NirExpr::Var(name.clone());
                return Some(vec![
                    NirStmt::Let {
                        name,
                        ty: Some(NirTypeRef {
                            name: "i64".into(),
                            is_ref: false,
                            is_optional: false,
                            generic_args: vec![],
                        }),
                        value: code.clone(),
                    },
                    NirStmt::Return(Some(wrapper)),
                ]);
            }
            if *type_name != outcome.ok_variant {
                return None;
            }
            (&fields[0].1, Some(value))
        }
        _ => return None,
    };
    if !prove_state_fields(record, layout, scope, live, &context.calls, locals)?.needs_split {
        return None;
    }
    let NirExpr::StructLiteral {
        type_name, fields, ..
    } = record
    else {
        unreachable!()
    };
    let resource = layout.resource_field.clone();
    let mut fields = fields.clone();
    let mut prefix = record_literals::prepare_return_fields(
        &mut fields,
        &resource,
        template,
        scope,
        live,
        context,
        locals,
    );
    let mut rewritten = Vec::with_capacity(fields.len());
    // Stage every field in source order, including earlier moves and copies.
    // Lazy children remain intact for the existing owned continuation pass.
    for (field, expression) in fields {
        let name = fresh_left(template, scope, live, context);
        let bytes = field == resource;
        prefix.push(NirStmt::Let {
            name: name.clone(),
            ty: Some(NirTypeRef {
                name: if bytes { "Bytes" } else { "i64" }.into(),
                is_ref: false,
                is_optional: false,
                generic_args: vec![],
            }),
            value: expression,
        });
        let value = NirExpr::Var(name);
        rewritten.push((
            field,
            if bytes {
                NirExpr::Move(Box::new(value))
            } else {
                value
            },
        ));
    }
    let record = NirExpr::StructLiteral {
        type_name: type_name.clone(),
        type_args: vec![],
        fields: rewritten,
    };
    prefix.push(NirStmt::Return(Some(match constructor {
        Some(original) => {
            let mut wrapper = original.clone();
            let NirExpr::StructLiteral { fields, .. } = &mut wrapper else {
                unreachable!()
            };
            fields[0].1 = record;
            wrapper
        }
        None => record,
    })));
    Some(prefix)
}

struct StateProof {
    needs_split: bool,
    has_projection: bool,
    remaining_owners: BTreeSet<String>,
}

fn prove_state_fields(
    record: &NirExpr,
    layout: &MixedResourceLayout,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
    locals: &LocalRecords,
) -> Option<StateProof> {
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = record
    else {
        return None;
    };
    if *type_name != layout.type_name
        || !type_args.is_empty()
        || fields.len() != layout.scalar_fields.len() + 1
    {
        return None;
    }
    let mut seen = BTreeSet::new();
    let mut available = live.clone();
    let mut needs_split = false;
    let mut has_projection = false;
    for (field, expression) in fields {
        if !seen.insert(field) {
            return None;
        }
        if *field == layout.resource_field {
            match expression {
                NirExpr::Move(value) => {
                    let NirExpr::Var(name) = value.as_ref() else {
                        return None;
                    };
                    if !available.remove(name) {
                        return None;
                    }
                }
                NirExpr::CopyBufferOwned(value) => {
                    let NirExpr::Var(name) = value.as_ref() else {
                        return None;
                    };
                    if !scope.iter().any(|p| {
                        p.name == *name
                            && p.ty.name == "Buffer"
                            && p.ty.is_ref
                            && !p.ty.is_optional
                            && p.ty.generic_args.is_empty()
                    }) {
                        return None;
                    }
                }
                _ => return None,
            }
        } else if layout.scalar_fields.contains(field) {
            let proof = locals.scalar_proof(expression, calls);
            if predicate_kind(&proof, scope, &available, calls) != Some(Kind::I64) {
                return None;
            }
            has_projection |= proof.as_ref() != expression;
            needs_split |= arguments::contains_logic(&proof)
                && admitted(&proof, scope, &available, calls, Kind::I64);
        } else {
            return None;
        }
    }
    Some(StateProof {
        needs_split,
        has_projection,
        remaining_owners: available,
    })
}

fn cleanup_tail<'a>(
    name: &str,
    mut suffix: impl DoubleEndedIterator<Item = &'a NirStmt>,
    live: &BTreeSet<String>,
) -> bool {
    let mut dropped = BTreeSet::new();
    matches!(suffix.next_back(), Some(NirStmt::Return(Some(NirExpr::Var(result)))) if result == name)
        && suffix.all(|s| matches!(s, NirStmt::Expr(NirExpr::DropBytes(value)) if matches!(value.as_ref(), NirExpr::Var(owner) if live.contains(owner) && dropped.insert(owner))))
}

// Prove the complete suffix, but move only total scalar field projections.
// The stores remain in the one shared continuation, not in either guard arm.
pub(super) fn stage_guarded_return_tail(
    remaining: &mut VecDeque<NirStmt>,
    binding: &str,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    if !locals.can_cross_binding(binding) {
        return None;
    }
    stage_return_tail(remaining, template, scope, live, context, locals)
}

pub(super) fn stage_return_tail(
    remaining: &mut VecDeque<NirStmt>,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    let return_type = template.return_type.as_ref()?;
    let write_count = remaining
        .iter()
        .take_while(|s| matches!(s, NirStmt::Expr(NirExpr::StoreAt { .. })))
        .count();
    let (stores, tail) = remaining.make_contiguous().split_at_mut(write_count);
    let mut suffix = tail.iter_mut();
    let (target, result_name) = match suffix.next()? {
        NirStmt::Return(Some(value)) if suffix.len() == 0 => (value, None),
        NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        }
        | NirStmt::Const { name, ty, value }
            if ty == return_type =>
        {
            (value, Some(name.as_str()))
        }
        _ => return None,
    };
    let cleanup_ok = |owners: &BTreeSet<String>| {
        result_name.is_none_or(|name| cleanup_tail(name, suffix.map(|s| &*s), owners))
    };
    let layout = context.return_layouts.get(&return_type.render())?;
    let outcome = layout.outcome.as_ref()?;
    bounded_tree(target, true)?;
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = target
    else {
        return None;
    };
    if *type_args != return_type.generic_args || fields.len() != 1 || fields[0].0 != "value" {
        return None;
    }
    let mut store_projections = false;
    for store in stores.iter() {
        store_projections |=
            stores::prove(store, scope, live, &context.calls, locals)?.has_projection;
    }
    let write_operands = stores.iter_mut().flat_map(|stmt| {
        let NirStmt::Expr(NirExpr::StoreAt { index, value, .. }) = stmt else {
            unreachable!()
        };
        [index.as_mut(), value.as_mut()]
    });
    let payload = &mut fields[0].1;
    if *type_name == outcome.ok_variant {
        let proof = prove_state_fields(payload, layout, scope, live, &context.calls, locals)?;
        if !(proof.has_projection || store_projections) || !cleanup_ok(&proof.remaining_owners) {
            return None;
        }
        let resource = layout.resource_field.clone();
        let NirExpr::StructLiteral { fields, .. } = payload else {
            unreachable!()
        };
        let scalars = fields
            .iter_mut()
            .filter_map(|(f, v)| (f != &resource).then_some(v));
        let snapshots = record_literals::prepare_operands(
            write_operands.chain(scalars),
            template,
            scope,
            live,
            context,
            locals,
        );
        return (!snapshots.is_empty()).then_some(snapshots);
    }
    if *type_name != outcome.err_variant || !cleanup_ok(live) {
        return None;
    }
    let code = payload;
    let proof = locals.scalar_proof(code, &context.calls);
    if (!store_projections && proof.as_ref() == code)
        || predicate_kind(&proof, scope, live, &context.calls) != Some(Kind::I64)
    {
        return None;
    }
    let snapshots = record_literals::prepare_operands(
        write_operands.chain(std::iter::once(code)),
        template,
        scope,
        live,
        context,
        locals,
    );
    (!snapshots.is_empty()).then_some(snapshots)
}

pub(super) fn split_cleanup_binding(
    statement: &NirStmt,
    template: &NirFunction,
    remaining: &VecDeque<NirStmt>,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
    locals: &LocalRecords,
) -> Option<Vec<NirStmt>> {
    let (name, ty, value) = match statement {
        NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        }
        | NirStmt::Const { name, ty, value } => (name, ty, value),
        _ => return None,
    };
    if Some(ty) != template.return_type.as_ref() || !cleanup_tail(name, remaining.iter(), live) {
        return None;
    }
    // Scope cleanup materializes the return before dropping unused owners.
    // Preserve that terminal binding and every drop; never move a drop ahead
    // of a field read or expose aggregate locals to arbitrary continuations.
    let mut prefix = split(value, template, scope, live, context, locals)?;
    let NirStmt::Return(Some(value)) = prefix.pop()? else {
        unreachable!()
    };
    let mut binding = statement.clone();
    let (NirStmt::Let { value: target, .. } | NirStmt::Const { value: target, .. }) = &mut binding
    else {
        unreachable!()
    };
    *target = value;
    prefix.push(binding);
    Some(prefix)
}
