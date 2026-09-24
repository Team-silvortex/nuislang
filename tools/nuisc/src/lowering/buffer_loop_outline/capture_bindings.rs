use super::*;

#[cfg(test)]
#[path = "capture_bindings_tests.rs"]
mod tests;

pub(super) fn normalize(function: &mut NirFunction) {
    if !walk::supported(&function.body) {
        return;
    }
    let visible = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut reserved = visible.keys().cloned().collect();
    branches::collect_bindings(&function.body, &mut reserved);
    walk::visit(&function.body, |expr| {
        if let NirExpr::Var(name) = expr {
            reserved.insert(name.clone());
        }
        true
    });

    let mut pending = vec![(&mut function.body, visible, false)];
    while let Some((body, mut visible, nested)) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                    rewrite(value, &visible);
                    // Existing names are writes to the visible binding, not new
                    // declarations. Initializers always read the preceding scope.
                    let identity = visible.entry(name.clone()).or_insert_with(|| {
                        if nested {
                            branches::fresh_name("__nuis_capture_local", &mut reserved)
                        } else {
                            name.clone()
                        }
                    });
                    *name = identity.clone();
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    rewrite(condition, &visible);
                    pending.push((else_body, visible.clone(), true));
                    pending.push((then_body, visible.clone(), true));
                }
                NirStmt::While { condition, body } => {
                    rewrite(condition, &visible);
                    pending.push((body, visible.clone(), true));
                }
                NirStmt::Print(value)
                | NirStmt::Expr(value)
                | NirStmt::Await(value)
                | NirStmt::Return(Some(value)) => rewrite(value, &visible),
                NirStmt::Break | NirStmt::Continue | NirStmt::Return(None) => {}
            }
        }
    }
}

fn rewrite(expr: &mut NirExpr, visible: &BTreeMap<String, String>) {
    walk::rewrite_expr(expr, |expr| {
        if let NirExpr::Var(name) = expr {
            if let Some(identity) = visible.get(name) {
                *name = identity.clone();
            }
        }
    });
}
