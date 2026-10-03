use super::*;

#[cfg(test)]
#[path = "capture_scalar_aliases_tests.rs"]
mod tests;

#[derive(Clone)]
struct Origin {
    path: Vec<String>,
    ty: NirTypeRef,
}

type Origins = BTreeMap<String, Origin>;

// Inline only total scalar reads of unwritten value inputs. Evaluated locals,
// calls and codecs are not origins, even when their binding has one definition.
pub(super) fn normalize(
    function: &mut NirFunction,
    layouts: &impl ValueLayouts,
    controls: &BTreeSet<String>,
) {
    if let Some(body) = normalized(function, layouts, controls, 65_536) {
        function.body = body;
    }
}

fn normalized(
    function: &NirFunction,
    layouts: &impl ValueLayouts,
    controls: &BTreeSet<String>,
    mut remaining: usize,
) -> Option<Vec<NirStmt>> {
    let mut writes = BTreeMap::<String, usize>::new();
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
                    *writes.entry(name.clone()).or_default() += 1;
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
    while let Some((expr, depth)) = expressions.pop() {
        if depth >= 64 || !walk::supported_expr(expr) {
            return None;
        }
        charge(&mut remaining, 1)?;
        crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push((child, depth + 1)));
    }
    let mut parameters = BTreeSet::new();
    let mut inputs = Origins::new();
    for param in &function.params {
        charge(&mut remaining, 1)?;
        if !parameters.insert(param.name.clone()) {
            return None;
        }
        if !writes.contains_key(&param.name) && control_values::supported_type(&param.ty, layouts) {
            inputs.insert(
                param.name.clone(),
                Origin {
                    path: vec![param.name.clone()],
                    ty: param.ty.clone(),
                },
            );
        }
    }
    parameters.extend(controls.iter().cloned());
    let mut context = Context {
        writes,
        parameters,
        layouts,
        remaining,
    };
    let mut result = function.body.clone();
    let mut pending = vec![(&mut result, inputs)];
    while let Some((body, mut scope)) = pending.pop() {
        let mut children = Vec::new();
        body.retain_mut(|stmt| {
            if charge(&mut context.remaining, 1).is_none() {
                return true;
            }
            let keep = match stmt {
                NirStmt::Let { name, ty, value } => {
                    context.binding(name, ty.as_ref(), value, &mut scope)
                }
                NirStmt::Const { name, ty, value } => {
                    context.binding(name, Some(ty), value, &mut scope)
                }
                NirStmt::If { condition, .. } | NirStmt::While { condition, .. } => {
                    context.rewrite(condition, &scope).and_then(|()| {
                        let cost = scope.values().try_fold(0usize, |cost, origin| {
                            cost.checked_add(origin.path.len() + 1)
                        })?;
                        charge(&mut context.remaining, cost)?;
                        children.push(scope.clone());
                        Some(true)
                    })
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => context.rewrite(value, &scope).map(|()| true),
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => Some(true),
            };
            if let Some(keep) = keep {
                keep
            } else {
                context.remaining = 0;
                true
            }
        });
        if context.remaining == 0 {
            return None;
        }
        let mut children = children.into_iter();
        for stmt in body {
            match stmt {
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    let scope = children.next()?;
                    let cost = scope.values().map(|origin| origin.path.len() + 1).sum();
                    charge(&mut context.remaining, cost)?;
                    pending.extend([(then_body, scope.clone()), (else_body, scope)]);
                }
                NirStmt::While { body, .. } => pending.push((body, children.next()?)),
                _ => {}
            }
        }
    }
    validate_expansion(&result, &mut context.remaining)?;
    (context.remaining > 0).then_some(result)
}

fn validate_expansion(body: &[NirStmt], remaining: &mut usize) -> Option<()> {
    let mut blocks = vec![body];
    let mut expressions = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            charge(remaining, 1)?;
            match stmt {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push((value, 0)),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push((condition, 0));
                    blocks.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { condition, body } => {
                    expressions.push((condition, 0));
                    blocks.push(body.as_slice());
                }
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
    while let Some((expr, depth)) = expressions.pop() {
        if depth >= 64 {
            return None;
        }
        charge(remaining, 1)?;
        crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push((child, depth + 1)));
    }
    Some(())
}

struct Context<'a, L> {
    writes: BTreeMap<String, usize>,
    parameters: BTreeSet<String>,
    layouts: &'a L,
    remaining: usize,
}

impl<L: ValueLayouts> Context<'_, L> {
    fn binding(
        &mut self,
        name: &str,
        declared: Option<&NirTypeRef>,
        value: &mut NirExpr,
        scope: &mut Origins,
    ) -> Option<bool> {
        self.rewrite(value, scope)?;
        let origin = self.origin(value, scope)?;
        if self.writes.get(name) == Some(&1) && !self.parameters.contains(name) {
            if let Some(origin) = origin.filter(|origin| {
                self.layouts.scalar(&origin.ty.name) && declared.is_none_or(|ty| ty == &origin.ty)
            }) {
                charge(&mut self.remaining, origin.path.len() + 1)?;
                scope.insert(name.to_owned(), origin);
                return Some(false);
            }
        }
        scope.remove(name);
        Some(true)
    }

    fn origin(&mut self, value: &NirExpr, scope: &Origins) -> Option<Option<Origin>> {
        let Some(path) = access(value) else {
            return Some(None);
        };
        charge(&mut self.remaining, path.len())?;
        let Some(base) = scope.get(&path[0]) else {
            return Some(None);
        };
        let mut ty = base.ty.clone();
        for field in &path[1..] {
            let Some(fields) = self.layouts.fields(&ty.name) else {
                return Some(None);
            };
            charge(&mut self.remaining, fields.len())?;
            let Some(next) = fields
                .into_iter()
                .find(|(name, _)| *name == field)
                .map(|(_, ty)| ty)
            else {
                return Some(None);
            };
            ty = next;
        }
        if !control_values::supported_type(&ty, self.layouts) {
            return Some(None);
        }
        charge(&mut self.remaining, base.path.len() + path.len())?;
        let mut canonical = base.path.clone();
        canonical.extend_from_slice(&path[1..]);
        Some(Some(Origin {
            path: canonical,
            ty,
        }))
    }

    fn rewrite(&mut self, value: &mut NirExpr, scope: &Origins) -> Option<()> {
        walk::rewrite_expr(value, |expr| {
            if charge(&mut self.remaining, 1).is_none() {
                return;
            }
            let NirExpr::Var(name) = expr else { return };
            let Some(origin) = scope.get(name) else {
                return;
            };
            if origin.path.len() == 1 && origin.path[0] == *name {
                return;
            }
            if charge(&mut self.remaining, origin.path.len()).is_none() {
                return;
            }
            *expr = crate::lowering::scalar_record_shape::source_value(
                &origin.path[0],
                &origin.path[1..],
            );
        });
        (self.remaining > 0).then_some(())
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
