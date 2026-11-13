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
// effect catalog. Logical roots are limited to predicates, typed initializers
// and exact bool fields; sequential type proof keeps new bindings out of scope.
pub(in crate::lowering::buffer_loop_outline) fn prefixed_declaration<'a>(
    stmt: &'a NirStmt,
    scope: &Scope,
    suffix: &[NirStmt],
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<prefixed::Declaration<'a>> {
    let join = prefixed::declaration_with_prints(stmt, scope, suffix, catalog, layouts)?;
    let (roots, _) = nested_prefix::prefix_roots(join.condition, join.yes, join.no, true, true)?;
    conditional_values::prefix::computed_record_call_expression_roots(roots, catalog, layouts)
        .then_some(join)
}

#[cfg(test)]
#[path = "scalar_joins_iteration_prefix_tests.rs"]
mod prefix_tests;

#[cfg(test)]
#[path = "scalar_joins_iteration_print_tests.rs"]
mod print_tests;

#[cfg(test)]
#[path = "scalar_joins_iteration_nested_tests.rs"]
mod nested_tests;

#[cfg(test)]
#[path = "scalar_joins_iteration_record_tests.rs"]
mod record_tests;

#[cfg(test)]
#[path = "scalar_joins_iteration_logical_record_tests.rs"]
mod logical_record_tests;

#[cfg(test)]
#[path = "scalar_joins_iteration_logical_local_tests.rs"]
mod logical_local_tests;

#[cfg(test)]
#[path = "scalar_joins_iteration_eager_local_tests.rs"]
mod eager_local_tests;
