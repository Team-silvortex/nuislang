use super::*;

#[cfg(test)]
#[path = "scalar_joins_iteration_tests.rs"]
mod tests;

// Unlike pure selection discovery, this table may contain admitted effectful
// callees. The consumer must keep each value arm behind its original predicate.
pub(in crate::lowering::buffer_loop_outline) fn declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Declaration<'a>> {
    let join = shape(stmt, scope, suffix)?;
    if !conditional_values::prefix::computed_call_expression_roots(
        vec![
            (join.condition, 0, true),
            (join.yes, 0, false),
            (join.no, 0, false),
        ],
        catalog,
    ) || control_values::value_type(join.condition, scope, catalog, layouts)?
        != scalar_type("bool")
        || !scalar_arms(&join, scope, catalog, layouts)
    {
        return None;
    }
    Some(join)
}

// Reuse the shared prefix shape and sequential type proof with the completed
// effect catalog. Initializers keep the iteration profile's non-logical shape.
pub(in crate::lowering::buffer_loop_outline) fn prefixed_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<prefixed::Declaration<'a>> {
    let join = prefixed::declaration(stmt, scope, suffix, catalog, layouts)?;
    let mut roots = vec![(join.condition, 0, true)];
    for stmt in join.yes.iter().chain(join.no) {
        let value = match stmt {
            NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
            _ => return None,
        };
        roots.push((value, 0, false));
    }
    conditional_values::prefix::computed_call_expression_roots(roots, catalog).then_some(join)
}

#[cfg(test)]
#[path = "scalar_joins_iteration_prefix_tests.rs"]
mod prefix_tests;
