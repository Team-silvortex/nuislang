use super::*;
use std::rc::Rc;

#[cfg(test)]
#[path = "capture_record_views_tests.rs"]
mod tests;

struct View {
    value: NirExpr,
    cost: usize,
}

type Views = BTreeMap<String, Rc<View>>;

// Resolve fields of total input reconstructions, not arbitrary computed records.
// Whole escapes retain their binding; the separate dead-record pass removes a
// snapshot only after all of its observations have become ready input paths.
pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if let Some(body) = normalized(function, layouts, 65_536) {
        function.body = body;
    }
}

fn normalized(
    function: &NirFunction,
    layouts: &impl ValueLayouts,
    mut remaining: usize,
) -> Option<Vec<NirStmt>> {
    let mut writes = BTreeMap::<&str, usize>::new();
    let mut blocks = vec![(function.body.as_slice(), 0)];
    let mut expressions = Vec::new();
    while let Some((body, depth)) = blocks.pop() {
        if depth >= 64 {
            return None;
        }
        for stmt in body {
            charge(&mut remaining, 1)?;
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    *writes.entry(name).or_default() += 1;
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
                    blocks.push((body, depth + 1));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push((value, 0)),
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    // Preflight before cloning, including expression depth and every statement.
    while let Some((expr, depth)) = expressions.pop() {
        if depth >= 64 || !walk::supported_expr(expr) {
            return None;
        }
        charge(&mut remaining, 1)?;
        crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push((child, depth + 1)));
    }
    let parameters = function
        .params
        .iter()
        .map(|p| p.name.as_str())
        .collect::<BTreeSet<_>>();
    let inputs = function
        .params
        .iter()
        .filter(|p| {
            !writes.contains_key(p.name.as_str()) && control_values::supported_type(&p.ty, layouts)
        })
        .map(|p| (p.name.as_str(), &p.ty))
        .collect();
    let mut context = Context {
        writes,
        parameters,
        inputs,
        layouts,
        remaining,
    };
    let mut result = function.body.clone();
    let mut pending = vec![(&mut result, Views::new())];
    while let Some((body, mut scope)) = pending.pop() {
        for stmt in body {
            charge(&mut context.remaining, 1)?;
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    context.binding(name, ty.as_ref(), value, &mut scope)?;
                }
                NirStmt::Const { name, ty, value } => {
                    context.binding(name, Some(ty), value, &mut scope)?;
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    context.rewrite(condition, &scope)?;
                    charge(&mut context.remaining, scope.len() * 2)?;
                    pending.extend([(then_body, scope.clone()), (else_body, scope.clone())]);
                }
                NirStmt::While { condition, body } => {
                    context.rewrite(condition, &scope)?;
                    charge(&mut context.remaining, scope.len())?;
                    pending.push((body, scope.clone()));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => context.rewrite(value, &scope)?,
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    (context.remaining > 0).then_some(result)
}

struct Context<'a, L> {
    writes: BTreeMap<&'a str, usize>,
    parameters: BTreeSet<&'a str>,
    inputs: BTreeMap<&'a str, &'a NirTypeRef>,
    layouts: &'a L,
    remaining: usize,
}

impl<L: ValueLayouts> Context<'_, L> {
    fn binding(
        &mut self,
        name: &str,
        declared: Option<&NirTypeRef>,
        value: &mut NirExpr,
        scope: &mut Views,
    ) -> Option<()> {
        self.rewrite(value, scope)?;
        if self.writes.get(name) != Some(&1) || self.parameters.contains(name) {
            scope.remove(name);
            return Some(());
        }
        // A local copy may inherit a proven view, but calls/returns keep whole
        // reads intact. No local value or loop-carried version grants readiness.
        let alias = if let NirExpr::Var(source) = value {
            scope.get(source).cloned()
        } else {
            None
        };
        if let Some(view) = &alias {
            charge(&mut self.remaining, view.cost)?;
        }
        let candidate = alias.as_ref().map_or(&*value, |view| &view.value);
        let before = self.remaining;
        let ty = dead_records::ready_type(
            candidate,
            &self.inputs,
            self.layouts,
            &mut self.remaining,
            0,
        );
        if ty
            .as_ref()
            .is_some_and(|ty| !self.layouts.scalar(&ty.name) && declared.is_none_or(|d| d == ty))
        {
            let cost = before - self.remaining;
            charge(&mut self.remaining, cost)?;
            let view = Rc::new(View {
                value: candidate.clone(),
                cost,
            });
            if alias.is_some() {
                *value = view.value.clone();
            }
            scope.insert(name.to_owned(), view);
        } else {
            scope.remove(name);
        }
        (self.remaining > 0).then_some(())
    }

    fn rewrite(&mut self, value: &mut NirExpr, scope: &Views) -> Option<()> {
        walk::rewrite_expr(value, |expr| {
            if charge(&mut self.remaining, 1).is_none() {
                return;
            }
            let Some(path) = access(expr) else {
                return;
            };
            if charge(&mut self.remaining, path.len()).is_none() || path.len() < 2 {
                return;
            }
            let Some(view) = scope.get(&path[0]) else {
                return;
            };
            if charge(&mut self.remaining, view.cost).is_none() {
                return;
            }
            let Some(projected) = select(&view.value, &path[1..]) else {
                return;
            };
            if dead_records::ready_type(
                &projected,
                &self.inputs,
                self.layouts,
                &mut self.remaining,
                0,
            )
            .is_some()
            {
                *expr = projected;
            }
        });
        (self.remaining > 0).then_some(())
    }
}

fn select(value: &NirExpr, path: &[String]) -> Option<NirExpr> {
    let mut selected = value;
    for (index, field) in path.iter().enumerate() {
        if let NirExpr::StructLiteral { fields, .. } = selected {
            selected = &fields.iter().find(|(name, _)| name == field)?.1;
        } else {
            return Some(path[index..].iter().fold(selected.clone(), |base, field| {
                NirExpr::FieldAccess {
                    base: Box::new(base),
                    field: field.clone(),
                }
            }));
        }
    }
    Some(selected.clone())
}

fn charge(remaining: &mut usize, cost: usize) -> Option<()> {
    if *remaining <= cost {
        *remaining = 0;
        return None;
    }
    *remaining -= cost;
    Some(())
}
