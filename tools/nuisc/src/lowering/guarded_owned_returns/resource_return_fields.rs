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
                if !admitted(code, scope, live, &context.calls, Kind::I64)
                    || !arguments::contains_logic(code)
                {
                    return None;
                }
                // Keep the complete error expression ahead of scope cleanup.
                // Its lazy children reenter the ordinary owned-region pass.
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
        } else if layout.scalar_fields.contains(field)
            && predicate_kind(expression, scope, &available, &context.calls) == Some(Kind::I64)
        {
            needs_split |= arguments::contains_logic(expression)
                && admitted(expression, scope, &available, &context.calls, Kind::I64);
        } else {
            return None;
        }
    }
    if !needs_split {
        return None;
    }
    let resource = layout.resource_field.clone();
    let mut prefix = Vec::with_capacity(fields.len() + 1);
    let mut rewritten = Vec::with_capacity(fields.len());
    // Stage every field in source order, including earlier moves and copies.
    // Lazy children remain intact for the existing owned continuation pass.
    for (field, expression) in fields {
        let name = fresh_left(template, scope, live, context);
        let bytes = *field == resource;
        prefix.push(NirStmt::Let {
            name: name.clone(),
            ty: Some(NirTypeRef {
                name: if bytes { "Bytes" } else { "i64" }.into(),
                is_ref: false,
                is_optional: false,
                generic_args: vec![],
            }),
            value: expression.clone(),
        });
        let value = NirExpr::Var(name);
        rewritten.push((
            field.clone(),
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

pub(super) fn split_cleanup_binding(
    statement: &NirStmt,
    template: &NirFunction,
    remaining: &VecDeque<NirStmt>,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
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
    if Some(ty) != template.return_type.as_ref()
        || !matches!(remaining.back(), Some(NirStmt::Return(Some(NirExpr::Var(result)))) if result == name)
        || !remaining.iter().take(remaining.len() - 1).all(|s|
            matches!(s, NirStmt::Expr(NirExpr::DropBytes(value)) if matches!(value.as_ref(), NirExpr::Var(owner) if live.contains(owner))))
    {
        return None;
    }
    // Scope cleanup materializes the return before dropping unused owners.
    // Preserve that terminal binding and every drop; never move a drop ahead
    // of a field read or expose aggregate locals to arbitrary continuations.
    let mut prefix = split(value, template, scope, live, context)?;
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
