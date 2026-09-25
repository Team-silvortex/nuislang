use super::*;

struct Binding {
    scope: usize,
    ty: Option<NirTypeRef>,
    valid: bool,
    joins: bool,
}

pub(super) fn collect(function: &NirFunction, layouts: &impl ValueLayouts) -> Inputs {
    let mut bindings = function
        .params
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                Binding {
                    scope: 0,
                    ty: Some(p.ty.clone()),
                    valid: true,
                    joins: false,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut copies = BTreeMap::<String, BTreeSet<String>>::new();
    for (scope, info) in scopes::collect(&function.body).iter().enumerate() {
        for stmt in info.body {
            let (name, ty, value) = match stmt {
                NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
                NirStmt::Const { name, ty, value } => (name, Some(ty), value),
                _ => continue,
            };
            let rebound = bindings.contains_key(name);
            let binding = bindings.entry(name.clone()).or_insert_with(|| Binding {
                scope,
                ty: ty.cloned(),
                valid: true,
                joins: false,
            });
            binding.valid &= ty.is_some() && binding.ty.as_ref() == ty;
            // Include backedge writes and rebound per-trip snapshots alike.
            // Single-definition loop aliases still use the invariant alias pass.
            binding.joins |= (scope != binding.scope && !info.returns) || (info.in_loop && rebound);
            if let NirExpr::Var(source) = value {
                copies
                    .entry(name.clone())
                    .or_default()
                    .insert(source.clone());
                copies
                    .entry(source.clone())
                    .or_default()
                    .insert(name.clone());
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut inputs = Inputs::new();
    for (root, binding) in &bindings {
        if !binding.joins || seen.contains(root) {
            continue;
        }
        let mut names = BTreeSet::new();
        let mut pending = vec![root.clone()];
        while let Some(name) = pending.pop() {
            if names.insert(name.clone()) {
                pending.extend(copies.get(&name).into_iter().flatten().cloned());
            }
        }
        seen.extend(names.iter().cloned());
        let Some(ty) = &binding.ty else { continue };
        if layouts.scalar(&ty.name)
            || !control_values::supported_type(ty, layouts)
            || !function.params.iter().any(|p| names.contains(&p.name))
            || !names.iter().all(|name| {
                bindings
                    .get(name)
                    .is_some_and(|b| b.valid && b.ty.as_ref() == Some(ty))
            })
        {
            continue;
        }
        let Some(paths) = demand(&function.body, &names, ty, layouts) else {
            continue;
        };
        let fields = paths
            .iter()
            .map(|path| {
                let field_ty = field_type(ty, path, layouts).expect("validated demand path");
                (path, field_ty)
            })
            .collect::<Vec<_>>();
        if fields
            .iter()
            .map(|(_, ty)| leaves(ty, layouts))
            .sum::<usize>()
            >= leaves(ty, layouts)
        {
            continue;
        }
        for param in &function.params {
            if names.contains(&param.name) {
                inputs.insert(param.name.clone(), paths.clone());
            }
        }
    }
    inputs
}

fn demand(
    body: &[NirStmt],
    names: &BTreeSet<String>,
    ty: &NirTypeRef,
    layouts: &impl ValueLayouts,
) -> Option<Vec<Vec<String>>> {
    let mut blocks = vec![body];
    let mut expressions = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. }
                    if names.contains(name)
                        && matches!(value, NirExpr::Var(source) if names.contains(source)) => {}
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => expressions.push(value),
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    expressions.push(condition);
                    blocks.extend([then_body.as_slice(), else_body.as_slice()]);
                }
                NirStmt::While { condition, body } => {
                    expressions.push(condition);
                    blocks.push(body);
                }
                _ => {}
            }
        }
    }
    let mut demand = BTreeSet::new();
    while let Some(expr) = expressions.pop() {
        if let Some(path) = access(expr) {
            if names.contains(&path[0]) {
                if path.len() == 1 || field_type(ty, &path[1..], layouts).is_none() {
                    return None;
                }
                demand.insert(path[1..].to_vec());
                continue;
            }
        }
        crate::nir_walk::walk_child_exprs(expr, &mut |child| expressions.push(child));
    }
    let mut paths = Vec::<Vec<String>>::new();
    for path in demand {
        if !paths.iter().any(|parent| path.starts_with(parent)) {
            paths.push(path);
        }
    }
    Some(paths)
}
