use super::*;

#[path = "capture_join_plan.rs"]
mod plan;

type Inputs = BTreeMap<String, Vec<Vec<String>>>;

// Keep record assignments and their existing joins intact. Only an input whose
// complete copy family has field-only uses may be rebuilt from demanded paths.
pub(super) fn normalize(function: &mut NirFunction, layouts: &impl ValueLayouts) {
    if !walk::supported(&function.body) {
        return;
    }
    let inputs = plan::collect(function, layouts);
    if inputs.is_empty() {
        return;
    }
    let mut reserved = function.params.iter().map(|p| p.name.clone()).collect();
    branches::collect_bindings(&function.body, &mut reserved);
    walk::visit(&function.body, |expr| {
        if let NirExpr::Var(name) = expr {
            reserved.insert(name.clone());
        }
        true
    });
    let mut names = BTreeMap::new();
    let mut seeds = Vec::new();
    for param in &function.params {
        let Some(paths) = inputs.get(&param.name) else {
            continue;
        };
        let name = branches::fresh_name("__nuis_capture_join_input", &mut reserved);
        let mut value = control_values::zero_value(&param.ty, layouts);
        // Omitted fields are proven unobservable across every alias and write.
        // Loop writes keep their original bodies/backedges; full uses and
        // unknown types veto this entry-only reconstruction.
        for path in paths {
            let mut selected = &mut value;
            for field in path {
                let NirExpr::StructLiteral { fields, .. } = selected else {
                    unreachable!("validated nominal field path")
                };
                selected = &mut fields.iter_mut().find(|(name, _)| name == field).unwrap().1;
            }
            *selected = path
                .iter()
                .fold(NirExpr::Var(param.name.clone()), |base, field| {
                    NirExpr::FieldAccess {
                        base: Box::new(base),
                        field: field.clone(),
                    }
                });
        }
        seeds.push(NirStmt::Let {
            name: name.clone(),
            ty: Some(param.ty.clone()),
            value,
        });
        names.insert(param.name.clone(), name);
    }
    walk::rewrite(&mut function.body, |expr| {
        if let NirExpr::Var(name) = expr {
            if let Some(replacement) = names.get(name) {
                *name = replacement.clone();
            }
        }
    });
    let mut pending = vec![&mut function.body];
    while let Some(body) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    if let Some(replacement) = names.get(name) {
                        *name = replacement.clone();
                    }
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => pending.extend([then_body, else_body]),
                NirStmt::While { body, .. } => pending.push(body),
                _ => {}
            }
        }
    }
    // Do not rewrite the seed's immutable parameter reads into its local copy.
    function.body.splice(0..0, seeds);
}
