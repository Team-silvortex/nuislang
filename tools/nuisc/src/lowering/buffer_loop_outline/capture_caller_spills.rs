use super::*;

#[cfg(test)]
#[path = "capture_caller_spills_tests.rs"]
mod tests;

// Materialize complete operands at binding/return/if-condition roots or inside
// a proven short-circuit RHS gate, never across branches or loops. Installation is atomic
// with the callee and every other caller in the enclosing projection pass.
pub(super) fn prepare(
    caller: &NirFunction,
    name: &str,
    plan: &Plan,
    layouts: &impl ValueLayouts,
    catalog: &ScalarHelpers,
    controls: &BTreeSet<String>,
) -> Option<NirFunction> {
    prepare_with_budget(caller, name, plan, layouts, catalog, controls, 65_536)
}

fn prepare_with_budget(
    caller: &NirFunction,
    name: &str,
    plan: &Plan,
    layouts: &impl ValueLayouts,
    catalog: &ScalarHelpers,
    controls: &BTreeSet<String>,
    mut remaining: usize,
) -> Option<NirFunction> {
    if caller.is_async
        || !caller.generic_params.is_empty()
        || !caller.where_bounds.is_empty()
        || !plan.word_inputs.is_empty()
    {
        return None;
    }
    // Bound expressions before supported-kind walks, cloning or recursive edits.
    scalar_aliases::validate_expansion(&caller.body, &mut remaining)?;
    if !walk::supported(&caller.body) {
        return None;
    }
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
                } => blocks.extend([
                    (then_body.as_slice(), depth + 1),
                    (else_body.as_slice(), depth + 1),
                ]),
                NirStmt::While { body, .. } => blocks.push((body, depth + 1)),
                _ => {}
            }
        }
    }
    let mut ready = BTreeMap::new();
    let mut names = written
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<String>>();
    let mut parameters = BTreeSet::new();
    for param in &caller.params {
        charge(&mut remaining, 1)?;
        if !parameters.insert(param.name.as_str()) {
            return None;
        }
        names.insert(param.name.clone());
        if !written.contains(param.name.as_str())
            && !controls.contains(&param.name)
            && control_values::supported_type(&param.ty, layouts)
        {
            ready.insert(param.name.as_str(), &param.ty);
        }
    }
    walk::visit(&caller.body, |expr| {
        if charge(&mut remaining, 1).is_none() {
            return false;
        }
        if let NirExpr::Var(name) = expr {
            names.insert(name.clone());
        }
        true
    });
    charge(&mut remaining, 0)?;
    let mut context = Context {
        name,
        plan,
        layouts,
        catalog,
        ready,
        names,
        remaining,
        changed: false,
    };
    let mut prepared = caller.clone();
    context.block(&mut prepared.body, false)?;
    scalar_aliases::validate_expansion(&prepared.body, &mut context.remaining)?;
    if !context.changed
        || !(valid_caller(&prepared.body, name, plan)
            || caller_records::valid(&prepared, name, plan, layouts))
    {
        return None;
    }
    Some(prepared)
}

struct Context<'a, L> {
    name: &'a str,
    plan: &'a Plan,
    layouts: &'a L,
    catalog: &'a ScalarHelpers,
    ready: BTreeMap<&'a str, &'a NirTypeRef>,
    names: BTreeSet<String>,
    remaining: usize,
    changed: bool,
}

impl<L: ValueLayouts> Context<'_, L> {
    fn block(&mut self, body: &mut Vec<NirStmt>, in_loop: bool) -> Option<()> {
        let mut result = Vec::new();
        for mut stmt in std::mem::take(body) {
            charge(&mut self.remaining, 1)?;
            match &mut stmt {
                NirStmt::Let { value, .. } | NirStmt::Const { value, .. } if !in_loop => {
                    if let NirExpr::Call { callee, args } = value {
                        if callee == self.name {
                            result.extend(self.operands(args)?);
                        }
                    }
                }
                NirStmt::Return(Some(NirExpr::Call { callee, args }))
                    if !in_loop && callee == self.name =>
                {
                    result.extend(self.operands(args)?);
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    if !in_loop {
                        if let NirExpr::Call { callee, args } = condition {
                            if callee == self.name {
                                if !self.plan.predicate_result {
                                    return None;
                                }
                                result.extend(self.operands(args)?);
                            }
                        } else {
                            result.extend(self.gated_predicate(condition)?);
                        }
                    }
                    self.block(then_body, in_loop)?;
                    self.block(else_body, in_loop)?;
                }
                NirStmt::While { body, .. } => self.block(body, true)?,
                _ => {}
            }
            result.push(stmt);
        }
        *body = result;
        Some(())
    }

    fn gated_predicate(&mut self, condition: &mut NirExpr) -> Option<Vec<NirStmt>> {
        let NirExpr::Binary {
            op: op @ (NirBinaryOp::And | NirBinaryOp::Or),
            lhs,
            rhs,
        } = condition
        else {
            return Some(Vec::new());
        };
        let NirExpr::Call { callee, args } = rhs.as_mut() else {
            return Some(Vec::new());
        };
        if callee != self.name {
            return Some(Vec::new());
        }
        let gate_ready = match lhs.as_ref() {
            NirExpr::Bool(_) => true,
            NirExpr::Var(name) => {
                self.ready.get(name.as_str()).copied() == Some(&scalar_type("bool"))
            }
            _ => false,
        };
        if !self.plan.predicate_result || !gate_ready {
            return None;
        }
        let mut body = self.operands(args)?;
        if body.is_empty() {
            return Some(Vec::new());
        }
        charge(&mut self.remaining, self.names.len() + 1)?;
        let name = branches::fresh_name("__nuis_caller_predicate", &mut self.names);
        let disjunction = *op == NirBinaryOp::Or;
        let gate = std::mem::replace(lhs.as_mut(), NirExpr::Bool(false));
        let predicate = std::mem::replace(rhs.as_mut(), NirExpr::Bool(false));
        body.push(NirStmt::Let {
            name: name.clone(),
            ty: Some(scalar_type("bool")),
            value: predicate,
        });
        if !conditional_values::prefix::bounded(&body) {
            return None;
        }
        *condition = NirExpr::Var(name.clone());
        // The complete RHS stays in its selected arm. The fresh bool joins only
        // the predicate result, without duplicating either original branch body.
        let (then_body, else_body) = if disjunction {
            (Vec::new(), body)
        } else {
            (body, Vec::new())
        };
        Some(vec![
            NirStmt::Let {
                name,
                ty: Some(scalar_type("bool")),
                value: NirExpr::Bool(disjunction),
            },
            NirStmt::If {
                condition: gate,
                then_body,
                else_body,
            },
        ])
    }

    fn operands(&mut self, args: &mut [NirExpr]) -> Option<Vec<NirStmt>> {
        if args.len() != self.plan.inputs.len() || args.len() != self.plan.original_types.len() {
            return None;
        }
        if !self
            .plan
            .inputs
            .iter()
            .zip(args.iter())
            .any(|(input, arg)| matches!(input, Input::Fields(_)) && access(arg).is_none())
        {
            return Some(Vec::new());
        }
        // Kind inference is not elision authority. Every original operand,
        // including complete constructors and unused checked fields, still runs.
        for (arg, ty) in args.iter().zip(&self.plan.original_types) {
            if !control_values::supported_type(ty, self.layouts)
                || record_views::materialized_operand_type(
                    arg,
                    &self.ready,
                    self.catalog,
                    self.layouts,
                    &mut self.remaining,
                )
                .as_ref()
                    != Some(ty)
            {
                return None;
            }
        }
        let mut bindings = Vec::new();
        for (arg, ty) in args.iter_mut().zip(&self.plan.original_types) {
            // Reserve the bounded worst-case fresh-name search before editing.
            charge(&mut self.remaining, self.names.len() + 1)?;
            let name = branches::fresh_name("__nuis_caller_operand", &mut self.names);
            let value = std::mem::replace(arg, NirExpr::Var(name.clone()));
            bindings.push(NirStmt::Let {
                name,
                ty: Some(ty.clone()),
                value,
            });
        }
        self.changed = true;
        Some(bindings)
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
