use super::*;

#[cfg(test)]
#[path = "iteration_prints_tests.rs"]
mod tests;

// One ordinary-root budget spans all prints, including child branches and loops.
pub(super) fn bounded(body: &[NirStmt]) -> bool {
    let mut pending = vec![body];
    let mut roots = Vec::new();
    while let Some(body) = pending.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Print(value) => roots.push((value, 0, false)),
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
    conditional_values::prefix::computed_expression_roots(roots)
}

pub(super) fn admit(
    value: &NirExpr,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<()> {
    if !conditional_values::prefix::expression(value)
        || !scalar_helpers::completed_pure_dependencies(value, catalog)
        || control_values::value_type(value, scope, catalog, layouts)? != scalar_type("i64")
    {
        return None;
    }
    Some(())
}
