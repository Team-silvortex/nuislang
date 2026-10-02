use super::*;

#[cfg(test)]
#[path = "capture_dead_records_tests.rs"]
mod tests;

// Nested invariant promotion can reconstruct an otherwise unused input snapshot.
// Only total copies of unwritten inputs may disappear, never evaluated arithmetic,
// calls, codecs or resources. Global name counts conservatively handle shadowing.
pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if !walk::supported(&function.body) {
        return;
    }
    let Some(dead) = candidates(function, layouts, 65_536) else {
        return;
    };
    let mut pending = vec![&mut function.body];
    while let Some(body) = pending.pop() {
        body.retain(|stmt| match stmt {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => !dead.contains(name),
            _ => true,
        });
        for stmt in body {
            match stmt {
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend([then_body, else_body]);
                }
                NirStmt::While { body, .. } => pending.push(body),
                _ => {}
            }
        }
    }
}

fn candidates(
    function: &NirFunction,
    layouts: &impl ValueLayouts,
    mut remaining: usize,
) -> Option<BTreeSet<String>> {
    let mut writes = BTreeMap::<&str, usize>::new();
    let mut reads = BTreeSet::new();
    let mut bindings = Vec::new();
    let mut pending = vec![function.body.as_slice()];
    while let Some(body) = pending.pop() {
        for stmt in body {
            remaining = remaining.checked_sub(1)?;
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    *writes.entry(name).or_default() += 1;
                    bindings.push((name, ty.as_ref(), value));
                }
                NirStmt::Const { name, ty, value } => {
                    *writes.entry(name).or_default() += 1;
                    bindings.push((name, Some(ty), value));
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    pending.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { body, .. } => pending.push(body),
                _ => {}
            }
        }
    }
    let mut work = Some(remaining);
    walk::visit(&function.body, |expr| {
        work = work.and_then(|left| left.checked_sub(1));
        if let NirExpr::Var(name) = expr {
            reads.insert(name.clone());
        }
        work.is_some()
    });
    remaining = work?;
    let inputs = function
        .params
        .iter()
        .filter(|p| {
            !writes.contains_key(p.name.as_str()) && control_values::supported_type(&p.ty, layouts)
        })
        .map(|p| (p.name.as_str(), &p.ty))
        .collect::<BTreeMap<_, _>>();
    let mut dead = BTreeSet::new();
    for (name, declared, value) in bindings {
        if reads.contains(name.as_str())
            || writes[name.as_str()] != 1
            || function.params.iter().any(|p| &p.name == name)
        {
            continue;
        }
        if let Some(ty) = ready_type(value, &inputs, layouts, &mut remaining, 0) {
            if !layouts.scalar(&ty.name) && declared.is_none_or(|declared| declared == &ty) {
                dead.insert(name.clone());
            }
        }
    }
    // Exhaustion must not install a partial proof.
    (remaining > 0).then_some(dead)
}

pub(super) fn ready_type(
    value: &NirExpr,
    inputs: &BTreeMap<&str, &NirTypeRef>,
    layouts: &impl ValueLayouts,
    remaining: &mut usize,
    depth: usize,
) -> Option<NirTypeRef> {
    if depth >= 64 || *remaining == 0 {
        *remaining = 0;
        return None;
    }
    *remaining -= 1;
    match value {
        NirExpr::Var(name) => inputs.get(name.as_str()).map(|ty| (*ty).clone()),
        NirExpr::FieldAccess { base, field } => {
            let ty = ready_type(base, inputs, layouts, remaining, depth + 1)?;
            let fields = layouts.fields(&ty.name)?;
            if fields.len() > *remaining {
                *remaining = 0;
                return None;
            }
            *remaining -= fields.len();
            let result = fields
                .into_iter()
                .find(|(name, _)| *name == field)
                .map(|(_, ty)| ty);
            result
        }
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let declared = layouts.fields(type_name)?;
            if !type_args.is_empty() || fields.len() != declared.len() {
                return None;
            }
            if fields.len() > *remaining {
                *remaining = 0;
                return None;
            }
            *remaining -= fields.len();
            let mut expected = declared
                .map(|(name, ty)| (name.to_owned(), ty))
                .collect::<BTreeMap<_, _>>();
            for (name, value) in fields {
                if ready_type(value, inputs, layouts, remaining, depth + 1)?
                    != expected.remove(name)?
                {
                    return None;
                }
            }
            expected.is_empty().then(|| scalar_type(type_name))
        }
        _ => None,
    }
}
