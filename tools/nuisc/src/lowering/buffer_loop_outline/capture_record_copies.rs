use super::*;

#[cfg(test)]
#[path = "capture_record_copies_tests.rs"]
mod tests;

// Do not move or erase computed fields. A private snapshot's unobserved, total
// input copies may become zeroes while every other field still executes in place.
// Constructors backing record-valued operands retain exact transport provenance
// for the all-caller word proofs in later (enclosing) capture plans.
pub(super) fn normalize(
    function: &mut NirFunction,
    layouts: &impl ValueLayouts,
    transport_types: &BTreeSet<String>,
) {
    if let Some(body) = normalized(function, layouts, transport_types, 65_536) {
        function.body = body;
    }
}

fn normalized(
    function: &NirFunction,
    layouts: &impl ValueLayouts,
    transport_types: &BTreeSet<String>,
    mut remaining: usize,
) -> Option<Vec<NirStmt>> {
    let mut writes = BTreeMap::<&str, usize>::new();
    let mut definitions = BTreeMap::new();
    let mut candidates = BTreeMap::new();
    let mut blocks = vec![(function.body.as_slice(), 0)];
    let mut expressions = Vec::new();
    while let Some((body, depth)) = blocks.pop() {
        if depth >= 64 {
            return None;
        }
        for stmt in body {
            charge(&mut remaining, 1)?;
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    *writes.entry(name).or_default() += 1;
                    definitions.insert(name.as_str(), value);
                    candidate(name, ty.as_ref(), value, &mut candidates, layouts);
                    expressions.push((value, 0));
                }
                NirStmt::Const { name, ty, value } => {
                    *writes.entry(name).or_default() += 1;
                    definitions.insert(name.as_str(), value);
                    candidate(name, Some(ty), value, &mut candidates, layouts);
                    expressions.push((value, 0));
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push((condition, 0));
                    blocks.extend([
                        (then_body.as_slice(), depth + 1),
                        (else_body.as_slice(), depth + 1),
                    ]);
                }
                NirStmt::While { condition, body } => {
                    expressions.push((condition, 0));
                    blocks.push((body.as_slice(), depth + 1));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push((value, 0)),
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    // Reject unsupported/deep expressions before any recursive proof or clone.
    while let Some((expr, depth)) = expressions.pop() {
        if depth >= 64 || !walk::supported_expr(expr) {
            return None;
        }
        charge(&mut remaining, 1)?;
        crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push((child, depth + 1)));
    }
    candidates.retain(|name, ty| {
        writes.get(name.as_str()) == Some(&1)
            && !function.params.iter().any(|p| p.name == *name)
            && !transport_types.contains(&ty.name)
    });
    let mut transport = BTreeSet::new();
    let mut returned = Vec::new();
    for scope in scopes::collect(&function.body) {
        for stmt in scope.body {
            charge(&mut remaining, 1)?;
            if let NirStmt::Return(Some(value @ NirExpr::StructLiteral { .. })) = stmt {
                returned.push(value);
            }
        }
    }
    for value in returned {
        transport_roots(value, &candidates, layouts, &mut transport, &mut remaining)?;
    }
    walk::visit(&function.body, |expr| {
        if charge(&mut remaining, 1).is_none() {
            return false;
        }
        if let NirExpr::Call { args, .. } = expr {
            for arg in args {
                if transport_roots(arg, &candidates, layouts, &mut transport, &mut remaining)
                    .is_none()
                {
                    break;
                }
            }
        }
        true
    });
    charge(&mut remaining, 0)?;
    // A transport may be assembled through immutable locals rather than reading
    // its source snapshot directly. Preserve that complete provenance chain;
    // mutable publications are version boundaries, not immutable aliases.
    let mut pending = transport.iter().cloned().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(name) = pending.pop() {
        charge(&mut remaining, 1)?;
        if writes.get(name.as_str()) != Some(&1) || !visited.insert(name.clone()) {
            continue;
        }
        if let Some(value) = definitions.get(name.as_str()) {
            let mut roots = BTreeSet::new();
            transport_roots(value, &candidates, layouts, &mut roots, &mut remaining)?;
            for root in roots {
                if transport.insert(root.clone()) {
                    pending.push(root);
                }
            }
        }
    }
    candidates.retain(|name, _| !transport.contains(name));
    let inputs = function
        .params
        .iter()
        .filter(|p| {
            !writes.contains_key(p.name.as_str()) && control_values::supported_type(&p.ty, layouts)
        })
        .map(|p| (p.name.as_str(), &p.ty))
        .collect::<BTreeMap<_, _>>();
    let mut reads = BTreeMap::<String, BTreeSet<Vec<String>>>::new();
    walk::visit(&function.body, |expr| {
        if charge(&mut remaining, 1).is_none() {
            return false;
        }
        let Some(path) = access(expr) else {
            return true;
        };
        if charge(&mut remaining, path.len()).is_none() {
            return false;
        }
        if candidates.contains_key(&path[0]) {
            reads
                .entry(path[0].clone())
                .or_default()
                .insert(path[1..].to_vec());
        }
        false
    });
    charge(&mut remaining, 0)?;
    for (name, paths) in &reads {
        for path in paths {
            charge(&mut remaining, path.len() + 1)?;
            field_type(&candidates[name], path, layouts)?;
        }
    }
    let empty = BTreeSet::new();
    let mut result = function.body.clone();
    let mut pending = vec![result.as_mut_slice()];
    while let Some(body) = pending.pop() {
        for stmt in body {
            charge(&mut remaining, 1)?;
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    if let Some(ty) = candidates.get(name) {
                        prune(
                            value,
                            ty,
                            &mut Vec::new(),
                            reads.get(name).unwrap_or(&empty),
                            &inputs,
                            layouts,
                            &mut remaining,
                            0,
                        )?;
                    }
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => pending.extend([then_body.as_mut_slice(), else_body.as_mut_slice()]),
                NirStmt::While { body, .. } => pending.push(body.as_mut_slice()),
                _ => {}
            }
        }
    }
    (remaining > 0).then_some(result)
}

fn transport_roots(
    value: &NirExpr,
    candidates: &BTreeMap<String, NirTypeRef>,
    layouts: &impl ValueLayouts,
    transport: &mut BTreeSet<String>,
    remaining: &mut usize,
) -> Option<()> {
    let mut pending = vec![(value, true)];
    while let Some((arg, direct)) = pending.pop() {
        charge(remaining, 1)?;
        if let Some(path) = access(arg) {
            charge(remaining, path.len())?;
            // An exact scalar leaf is an observation, not aggregate transport.
            // The demand walk below retains it and all computed sibling fields.
            if !direct || !scalar_observation(&path, candidates, layouts, remaining)? {
                transport.insert(path[0].clone());
            }
        } else {
            // Composite/encoded transport keeps its complete source identity.
            crate::nir_walk::walk_child_exprs(arg, &mut |child| pending.push((child, false)));
        }
    }
    Some(())
}

fn scalar_observation(
    path: &[String],
    candidates: &BTreeMap<String, NirTypeRef>,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
) -> Option<bool> {
    let Some(mut ty) = candidates.get(&path[0]).cloned() else {
        return Some(false);
    };
    for field in &path[1..] {
        let Some(fields) = layouts.fields(&ty.name) else {
            return Some(false);
        };
        charge(remaining, fields.len())?;
        let Some(next) = fields
            .into_iter()
            .find(|(name, _)| *name == field)
            .map(|(_, ty)| ty)
        else {
            return Some(false);
        };
        ty = next;
    }
    Some(layouts.scalar(&ty.name) && control_values::supported_type(&ty, layouts))
}

fn candidate(
    name: &str,
    declared: Option<&NirTypeRef>,
    value: &NirExpr,
    candidates: &mut BTreeMap<String, NirTypeRef>,
    layouts: &impl ValueLayouts,
) {
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        ..
    } = value
    else {
        return;
    };
    let ty = scalar_type(type_name);
    if type_args.is_empty()
        && control_values::supported_type(&ty, layouts)
        && declared.is_none_or(|d| d == &ty)
    {
        candidates.insert(name.to_owned(), ty);
    }
}

fn prune(
    value: &mut NirExpr,
    ty: &NirTypeRef,
    path: &mut Vec<String>,
    needed: &BTreeSet<Vec<String>>,
    inputs: &BTreeMap<&str, &NirTypeRef>,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
    depth: usize,
) -> Option<()> {
    if depth >= 64 {
        return None;
    }
    charge(remaining, needed.len() + 1)?;
    if needed.iter().any(|read| path.starts_with(read)) {
        return Some(());
    }
    let before = *remaining;
    let ready = dead_records::ready_type(value, inputs, layouts, remaining, depth);
    let ready_cost = before - *remaining;
    if ready.as_ref().is_some_and(|actual| actual != ty) {
        return None;
    }
    let observed = needed.iter().any(|read| read.starts_with(path));
    if ready.is_some() && !observed {
        *value = zero(ty, layouts, remaining, depth)?;
        return Some(());
    }
    let Some(layout) = layouts.fields(&ty.name) else {
        return Some(());
    };
    charge(remaining, layout.len())?;
    let mut expected = layout
        .map(|(name, ty)| (name.to_owned(), ty))
        .collect::<BTreeMap<_, _>>();
    // A partially observed total subrecord can be expanded into ready paths.
    // Non-total subrecords/calls stay opaque unless they are exact constructors.
    if ready.is_some() && !matches!(value, NirExpr::StructLiteral { .. }) {
        charge(remaining, expected.len().checked_mul(ready_cost + 1)?)?;
        let base = value.clone();
        *value = NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: Vec::new(),
            fields: expected
                .keys()
                .map(|field| {
                    (
                        field.clone(),
                        NirExpr::FieldAccess {
                            base: Box::new(base.clone()),
                            field: field.clone(),
                        },
                    )
                })
                .collect(),
        };
    }
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    else {
        return Some(());
    };
    if type_name != &ty.name || !type_args.is_empty() || fields.len() != expected.len() {
        return None;
    }
    for (field, value) in fields {
        let ty = expected.remove(field)?;
        path.push(field.clone());
        prune(
            value,
            &ty,
            path,
            needed,
            inputs,
            layouts,
            remaining,
            depth + 1,
        )?;
        path.pop();
    }
    expected.is_empty().then_some(())
}

fn zero(
    ty: &NirTypeRef,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
    depth: usize,
) -> Option<NirExpr> {
    if depth >= 64 || !control_values::supported_type(ty, layouts) {
        return None;
    }
    charge(remaining, 1)?;
    if let Some(fields) = layouts.fields(&ty.name) {
        charge(remaining, fields.len())?;
        Some(NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: Vec::new(),
            fields: fields
                .map(|(field, ty)| {
                    Some((field.to_owned(), zero(&ty, layouts, remaining, depth + 1)?))
                })
                .collect::<Option<_>>()?,
        })
    } else {
        Some(control_values::zero_value(ty, layouts))
    }
}

fn charge(remaining: &mut usize, cost: usize) -> Option<()> {
    if *remaining <= cost {
        *remaining = 0;
        return None;
    }
    *remaining -= cost;
    Some(())
}
