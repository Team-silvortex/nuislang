use super::*;

#[cfg(test)]
#[path = "capture_aliases_tests.rs"]
mod tests;

#[derive(Clone)]
struct Origin {
    path: Vec<String>,
    ty: NirTypeRef,
}

type Origins = BTreeMap<String, Origin>;

pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if !walk::supported(&function.body) {
        return;
    }
    let writes = write_counts(&function.body);
    let parameters = function
        .params
        .iter()
        .map(|p| p.name.clone())
        .collect::<BTreeSet<_>>();
    let origins = function
        .params
        .iter()
        .filter(|p| !writes.contains_key(&p.name) && is_record(&p.ty, layouts))
        .map(|p| {
            (
                p.name.clone(),
                Origin {
                    path: vec![p.name.clone()],
                    ty: p.ty.clone(),
                },
            )
        })
        .collect();
    let context = Context {
        writes: &writes,
        parameters: &parameters,
        layouts,
    };
    let mut pending = vec![(&mut function.body, origins, false)];
    while let Some((body, mut scope, in_loop)) = pending.pop() {
        let mut nested_scopes = Vec::new();
        body.retain_mut(|stmt| match stmt {
            NirStmt::Let { name, ty, value } => {
                context.keep_binding(name, ty.as_ref(), value, &mut scope, in_loop)
            }
            NirStmt::Const { name, ty, value } => {
                context.keep_binding(name, Some(ty), value, &mut scope, in_loop)
            }
            NirStmt::If { condition, .. } | NirStmt::While { condition, .. } => {
                rewrite(condition, &scope);
                nested_scopes.push(scope.clone());
                true
            }
            NirStmt::Print(value)
            | NirStmt::Expr(value)
            | NirStmt::Await(value)
            | NirStmt::Return(Some(value)) => {
                rewrite(value, &scope);
                true
            }
            NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => true,
        });
        // Snapshot lexical environments before descending. Branch-local aliases
        // never escape; iteration-local bindings keep their per-trip snapshots.
        let mut scopes = nested_scopes.into_iter();
        for stmt in body {
            match stmt {
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    let scope = scopes.next().expect("recorded branch scope");
                    pending.push((then_body, scope.clone(), in_loop));
                    pending.push((else_body, scope, in_loop));
                }
                NirStmt::While { body, .. } => {
                    pending.push((body, scopes.next().expect("recorded loop scope"), true));
                }
                _ => {}
            }
        }
    }
}

struct Context<'a, L> {
    writes: &'a BTreeMap<String, usize>,
    parameters: &'a BTreeSet<String>,
    layouts: &'a L,
}

impl<L: ValueLayouts> Context<'_, L> {
    fn keep_binding(
        &self,
        name: &str,
        declared: Option<&NirTypeRef>,
        value: &mut NirExpr,
        scope: &mut Origins,
        in_loop: bool,
    ) -> bool {
        rewrite(value, scope);
        let origin = access(value).and_then(|path| {
            let base = scope.get(&path[0])?;
            let ty = field_type(&base.ty, &path[1..], self.layouts)?;
            let mut canonical = base.path.clone();
            canonical.extend_from_slice(&path[1..]);
            Some(Origin {
                path: canonical,
                ty,
            })
        });
        if !in_loop && self.writes.get(name) == Some(&1) && !self.parameters.contains(name) {
            if let Some(origin) = origin.filter(|origin| {
                is_record(&origin.ty, self.layouts) && declared.is_none_or(|ty| ty == &origin.ty)
            }) {
                scope.insert(name.to_owned(), origin);
                return false;
            }
        }
        scope.remove(name);
        true
    }
}

fn is_record(ty: &NirTypeRef, layouts: &impl ValueLayouts) -> bool {
    !layouts.scalar(&ty.name) && control_values::supported_type(ty, layouts)
}

fn rewrite(expr: &mut NirExpr, origins: &Origins) {
    walk::rewrite_expr(expr, |expr| {
        let Some(path) = access(expr) else {
            return;
        };
        let Some(origin) = origins.get(&path[0]) else {
            return;
        };
        if origin.path.len() == 1 && origin.path[0] == path[0] {
            return;
        }
        *expr = origin.path[1..].iter().chain(&path[1..]).fold(
            NirExpr::Var(origin.path[0].clone()),
            |base, field| NirExpr::FieldAccess {
                base: Box::new(base),
                field: field.clone(),
            },
        );
    });
}

fn write_counts(body: &[NirStmt]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let mut pending = vec![body];
    while let Some(body) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    *counts.entry(name.clone()).or_default() += 1;
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
    counts
}
