use super::*;
use control_values::ValueLayouts;

#[path = "capture_aliases.rs"]
mod aliases;
#[path = "capture_snapshots.rs"]
mod snapshots;
#[cfg(test)]
#[path = "capture_projection_tests.rs"]
mod tests;
#[path = "capture_projection_walk.rs"]
mod walk;

struct Projection {
    path: Vec<String>,
    param: NirParam,
}

enum Input {
    Keep(NirParam),
    Fields(Vec<Projection>),
}

struct Plan {
    inputs: Vec<Input>,
    replacements: BTreeMap<Vec<String>, String>,
}

pub(super) fn project(
    module: &mut NirModule,
    generated: &BTreeSet<String>,
    layouts: &impl ValueLayouts,
) -> bool {
    if generated.is_empty() {
        return false;
    }
    let eligible = generated.iter().map(String::as_str).collect();
    let scoped = scoped_loop_lowering::collect_scoped_call_targets(module, &eligible);
    let call_graph = module
        .functions
        .iter()
        .map(|f| calls(&f.body))
        .collect::<Vec<_>>();
    let mut pending = module
        .functions
        .iter()
        .zip(&call_graph)
        .filter(|(f, _)| generated.contains(&f.name) && !scoped.contains(&f.name))
        .map(|(f, calls)| (f.name.clone(), calls.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut changed = false;
    // Rewrite callees first: their projected call arguments expose demand to
    // enclosing helpers. Cycles (and their dependants) conservatively stay whole.
    while let Some(name) = pending
        .iter()
        .find(|(_, calls)| calls.iter().all(|callee| !pending.contains_key(callee)))
        .map(|(name, _)| name.clone())
    {
        pending.remove(&name);
        let index = module
            .functions
            .iter()
            .position(|f| f.name == name)
            .unwrap();
        // Normalize only a candidate copy. A whole use or an unrewritable caller
        // must keep both the original signature and its original body.
        let mut candidate = module.functions[index].clone();
        snapshots::normalize(&mut candidate, layouts);
        aliases::normalize(&mut candidate, layouts);
        let Some(plan) = plan(&candidate, layouts) else {
            continue;
        };
        let callers = call_graph
            .iter()
            .enumerate()
            .filter(|(_, calls)| calls.contains(&name))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if !callers
            .iter()
            .all(|i| valid_caller(&module.functions[*i].body, &name, &plan))
        {
            continue;
        }
        let function = &mut candidate;
        walk::rewrite(&mut function.body, |expr| {
            if let Some(replacement) = access(expr).and_then(|path| plan.replacements.get(&path)) {
                *expr = NirExpr::Var(replacement.clone());
            }
        });
        function.params = plan
            .inputs
            .iter()
            .flat_map(|input| match input {
                Input::Keep(param) => vec![param.clone()],
                Input::Fields(fields) => fields.iter().map(|field| field.param.clone()).collect(),
            })
            .collect();
        module.functions[index] = candidate;
        for index in callers {
            walk::rewrite(&mut module.functions[index].body, |expr| {
                let NirExpr::Call { callee, args } = expr else {
                    return;
                };
                if callee != &name {
                    return;
                }
                *args = plan
                    .inputs
                    .iter()
                    .zip(std::mem::take(args))
                    .flat_map(|(input, arg)| match input {
                        Input::Keep(_) => vec![arg],
                        Input::Fields(fields) => fields
                            .iter()
                            .map(|field| {
                                field.path.iter().fold(arg.clone(), |base, field| {
                                    NirExpr::FieldAccess {
                                        base: Box::new(base),
                                        field: field.clone(),
                                    }
                                })
                            })
                            .collect(),
                    })
                    .collect();
            });
        }
        changed = true;
    }
    changed
}

fn calls(body: &[NirStmt]) -> BTreeSet<String> {
    let mut calls = BTreeSet::new();
    walk::visit(body, |expr| {
        if let NirExpr::Call { callee, .. }
        | NirExpr::CpuSpawn { callee, .. }
        | NirExpr::CpuThreadSpawn { callee, .. } = expr
        {
            calls.insert(callee.clone());
        }
        true
    });
    calls
}

fn valid_caller(body: &[NirStmt], name: &str, plan: &Plan) -> bool {
    if !walk::supported(body) {
        return false;
    }
    let mut valid = true;
    walk::visit(body, |expr| {
        if let NirExpr::Call { callee, args } = expr {
            if callee == name {
                // Only ready immutable value paths may be duplicated or dropped.
                // Keep scalar/predicate arguments exactly once, in source order.
                valid &= args.len() == plan.inputs.len()
                    && plan.inputs.iter().zip(args).all(|(input, arg)| {
                        matches!(input, Input::Keep(_)) || access(arg).is_some()
                    });
            }
        }
        true
    });
    valid
}

fn plan(function: &NirFunction, layouts: &impl ValueLayouts) -> Option<Plan> {
    if !walk::supported(&function.body) {
        return None;
    }
    let mut written = BTreeSet::new();
    branches::collect_bindings(&function.body, &mut written);
    let candidates = function
        .params
        .iter()
        .filter(|p| {
            !written.contains(&p.name)
                && !layouts.scalar(&p.ty.name)
                && control_values::supported_type(&p.ty, layouts)
        })
        .map(|p| p.name.clone())
        .collect::<BTreeSet<_>>();
    let mut demand = BTreeMap::<String, BTreeSet<Vec<String>>>::new();
    walk::visit(&function.body, |expr| {
        if let Some(path) = access(expr) {
            if candidates.contains(&path[0]) {
                demand
                    .entry(path[0].clone())
                    .or_default()
                    .insert(path[1..].to_vec());
                return false;
            }
        }
        true
    });
    let mut reserved = written;
    reserved.extend(function.params.iter().map(|p| p.name.clone()));
    let mut inputs = Vec::new();
    let mut replacements = BTreeMap::new();
    let mut changed = false;
    for param in &function.params {
        if !candidates.contains(&param.name) {
            inputs.push(Input::Keep(param.clone()));
            continue;
        }
        let mut paths = Vec::<Vec<String>>::new();
        for path in demand.remove(&param.name).unwrap_or_default() {
            if !paths.iter().any(|parent| path.starts_with(parent)) {
                paths.push(path);
            }
        }
        let types = paths
            .iter()
            .map(|path| field_type(&param.ty, path, layouts))
            .collect::<Option<Vec<_>>>()?;
        let selected = types.iter().map(|ty| leaves(ty, layouts)).sum::<usize>();
        if selected >= leaves(&param.ty, layouts) {
            inputs.push(Input::Keep(param.clone()));
            continue;
        }
        let fields = paths
            .into_iter()
            .zip(types)
            .map(|(path, ty)| {
                let name = branches::fresh_name("__nuis_capture_field", &mut reserved);
                let mut full_path = vec![param.name.clone()];
                full_path.extend(path.iter().cloned());
                replacements.insert(full_path, name.clone());
                Projection {
                    path,
                    param: NirParam { name, ty },
                }
            })
            .collect();
        inputs.push(Input::Fields(fields));
        changed = true;
    }
    changed.then_some(Plan {
        inputs,
        replacements,
    })
}

fn access(mut expr: &NirExpr) -> Option<Vec<String>> {
    let mut path = Vec::new();
    loop {
        match expr {
            NirExpr::Var(name) => {
                path.push(name.clone());
                path.reverse();
                return Some(path);
            }
            NirExpr::FieldAccess { base, field } => {
                path.push(field.clone());
                expr = base;
            }
            _ => return None,
        }
    }
}

fn field_type(ty: &NirTypeRef, path: &[String], layouts: &impl ValueLayouts) -> Option<NirTypeRef> {
    let mut ty = ty.clone();
    for field in path {
        let next = layouts.fields(&ty.name)?.find(|(name, _)| name == field)?.1;
        ty = next;
    }
    Some(ty)
}

fn leaves(ty: &NirTypeRef, layouts: &impl ValueLayouts) -> usize {
    let mut pending = vec![ty.clone()];
    let mut count = 0;
    while let Some(ty) = pending.pop() {
        if layouts.scalar(&ty.name) {
            count += 1;
        } else {
            pending.extend(
                layouts
                    .fields(&ty.name)
                    .expect("admitted value layout")
                    .map(|(_, ty)| ty),
            );
        }
    }
    count
}
