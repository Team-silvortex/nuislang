use super::*;

#[cfg(test)]
#[path = "capture_caller_records_tests.rs"]
mod tests;

// This separate caller proof accepts only total inline input reconstructions.
// It never spills a call or grants readiness to computed/local field values.
pub(super) fn valid(
    caller: &NirFunction,
    name: &str,
    plan: &Plan,
    layouts: &impl ValueLayouts,
) -> bool {
    valid_with_budget(caller, name, plan, layouts, 65_536).is_some()
}

fn valid_with_budget(
    caller: &NirFunction,
    name: &str,
    plan: &Plan,
    layouts: &impl ValueLayouts,
    mut remaining: usize,
) -> Option<()> {
    if !walk::supported(&caller.body) {
        return None;
    }
    scalar_aliases::validate_expansion(&caller.body, &mut remaining)?;
    let mut written = BTreeSet::new();
    let mut blocks = vec![(caller.body.as_slice(), 0)];
    while let Some((body, depth)) = blocks.pop() {
        if depth >= 64 {
            return None;
        }
        for stmt in body {
            charge(&mut remaining, 1)?;
            match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    written.insert(name.as_str());
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    blocks.extend([
                        (then_body.as_slice(), depth + 1),
                        (else_body.as_slice(), depth + 1),
                    ]);
                }
                NirStmt::While { body, .. } => blocks.push((body.as_slice(), depth + 1)),
                _ => {}
            }
        }
    }
    let mut inputs = BTreeMap::new();
    let mut parameters = BTreeSet::new();
    for param in &caller.params {
        charge(&mut remaining, 1)?;
        if !parameters.insert(param.name.as_str()) {
            return None;
        }
        if !written.contains(param.name.as_str())
            && control_values::supported_type(&param.ty, layouts)
        {
            inputs.insert(param.name.as_str(), &param.ty);
        }
    }
    let mut valid = true;
    walk::visit(&caller.body, |expr| {
        if !valid || charge(&mut remaining, 1).is_none() {
            valid = false;
            return false;
        }
        if let NirExpr::Call { callee, args } = expr {
            if callee == name {
                valid = args.len() == plan.inputs.len()
                    && plan
                        .inputs
                        .iter()
                        .zip(args)
                        .enumerate()
                        .all(|(index, (input, arg))| {
                            if let Some(words) = plan.word_inputs.get(&index) {
                                return words.valid_argument(arg);
                            }
                            if matches!(input, Input::Keep(_)) || access(arg).is_some() {
                                return true;
                            }
                            let Input::Fields(fields) = input else {
                                return false;
                            };
                            if !matches!(arg, NirExpr::StructLiteral { .. })
                                || dead_records::ready_type(
                                    arg,
                                    &inputs,
                                    layouts,
                                    &mut remaining,
                                    0,
                                )
                                .as_ref()
                                    != Some(&plan.original_types[index])
                            {
                                return false;
                            }
                            // Validate the complete constructor, not just selected fields.
                            // Even an unused argument cannot discard checked work.
                            fields.iter().all(|field| {
                                let Some(selected) = selected(arg, &field.path, &mut remaining)
                                else {
                                    return false;
                                };
                                let value = project(selected.0, selected.1);
                                dead_records::ready_type(
                                    &value,
                                    &inputs,
                                    layouts,
                                    &mut remaining,
                                    0,
                                )
                                .as_ref()
                                    == Some(&field.param.ty)
                            })
                        });
            }
        }
        valid
    });
    (valid && remaining > 0).then_some(())
}

fn selected<'a>(
    mut value: &'a NirExpr,
    path: &'a [String],
    remaining: &mut usize,
) -> Option<(&'a NirExpr, &'a [String])> {
    for (index, field) in path.iter().enumerate() {
        charge(remaining, 1)?;
        if let NirExpr::StructLiteral { fields, .. } = value {
            charge(remaining, fields.len())?;
            value = &fields.iter().find(|(name, _)| name == field)?.1;
        } else {
            return Some((value, &path[index..]));
        }
    }
    Some((value, &[]))
}

// Select constructor fields directly; wrapping the entire constructor in each
// field access would retain redundant work and defeat the caller proof.
pub(super) fn project(value: &NirExpr, path: &[String]) -> NirExpr {
    let mut remaining = usize::MAX;
    let (value, path) =
        selected(value, path, &mut remaining).expect("preflighted total caller record field");
    path.iter()
        .fold(value.clone(), |base, field| NirExpr::FieldAccess {
            base: Box::new(base),
            field: field.clone(),
        })
}

fn charge(remaining: &mut usize, cost: usize) -> Option<()> {
    *remaining = remaining.checked_sub(cost)?;
    (*remaining > 0).then_some(())
}
