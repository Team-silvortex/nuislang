use super::*;

pub(in crate::lowering::buffer_loop_outline) struct Root<'a> {
    pub value: &'a NirExpr,
    pub depth: usize,
    pub logical: bool,
    pub eager: bool,
}

// Bound source/derived pure prefixes before recursive kind inference or cloning.
// This is not general multi-binding, effectful or outer-state branch lowering.
pub(in crate::lowering::buffer_loop_outline) fn bounded(body: &[NirStmt]) -> bool {
    if body.len() > 32 {
        return false;
    }
    let mut pending = Vec::new();
    for stmt in body {
        let value = match stmt {
            NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
            _ => return false,
        };
        pending.push((value, 0));
    }
    expressions(pending)
}

pub(in crate::lowering::buffer_loop_outline) fn expression(expr: &NirExpr) -> bool {
    expressions(vec![(expr, 0)])
}

pub(in crate::lowering::buffer_loop_outline) fn return_body(body: &[NirStmt]) -> bool {
    let [locals @ .., NirStmt::Return(Some(value))] = body else {
        return false;
    };
    if locals.is_empty() || body.len() > 32 {
        return false;
    }
    let mut pending = vec![(value, 0, true)];
    for stmt in locals {
        let value = match stmt {
            NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
            _ => return false,
        };
        pending.push((value, 0, true));
    }
    computed_expression_roots(pending)
}

fn expressions(pending: Vec<(&NirExpr, usize)>) -> bool {
    expression_roots(
        pending
            .into_iter()
            .map(|(expr, depth)| (expr, depth, false))
            .collect(),
    )
}

pub(in crate::lowering::buffer_loop_outline) fn expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
) -> bool {
    roots(pending, false)
}

pub(in crate::lowering::buffer_loop_outline) fn computed_logical_root(expr: &NirExpr) -> bool {
    computed_expression_roots(vec![(expr, 0, true)])
}

// Return trees share one budget across all roots, including duplicated suffixes.
// Only direct logical children inherit permission in ordinary profiles.
// The typed call profile separately admits eager bool operands, not new reads.
pub(in crate::lowering::buffer_loop_outline) fn computed_expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
) -> bool {
    roots(pending, true)
}

fn roots(pending: Vec<(&NirExpr, usize, bool)>, computed_gate: bool) -> bool {
    roots_with_fields(pending, computed_gate, false, |_, _| false, |_, _, _| false)
}

// This opens typed argument shape only. The caller still proves reads and laziness.
pub(in crate::lowering::buffer_loop_outline) fn computed_call_expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
    catalog: &ScalarHelpers,
) -> bool {
    call_roots(pending, catalog, |_, _| false)
}

pub(in crate::lowering::buffer_loop_outline) fn computed_record_call_expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    call_roots(pending, catalog, |record, field| {
        boolean_field(layouts, record, field)
    })
}

fn boolean_field(layouts: &impl control_values::ValueLayouts, record: &str, field: &str) -> bool {
    layouts
        .fields(record)
        .and_then(|mut fields| fields.find(|(name, _)| *name == field))
        .is_some_and(|(_, ty)| ty == scalar_type("bool"))
}

fn call_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
    catalog: &ScalarHelpers,
    bool_field: impl Fn(&str, &str) -> bool,
) -> bool {
    roots_with_fields(
        pending,
        true,
        true,
        bool_field,
        |callee, index, lazy_rhs| {
            // Auxiliary calls inherit argument shape only below an authorized RHS.
            scalar_helpers::pure_scalar_signature(callee, catalog)
                .filter(|(_, result)| {
                    matches!(result.name.as_str(), "f32" | "f64")
                        || (lazy_rhs && matches!(result.name.as_str(), "bool" | "i32" | "i64"))
                })
                .and_then(|(params, _)| params.get(index))
                == Some(&scalar_type("bool"))
        },
    )
}

pub(in crate::lowering::buffer_loop_outline) fn computed_record_expression_roots(
    pending: Vec<(&NirExpr, usize, bool)>,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    roots_with_fields(
        pending,
        true,
        false,
        |record, field| boolean_field(layouts, record, field),
        |_, _, _| false,
    )
}

fn roots_with_fields(
    pending: Vec<(&NirExpr, usize, bool)>,
    computed_gate: bool,
    computed_bool_operands: bool,
    bool_field: impl Fn(&str, &str) -> bool,
    logical_argument: impl Fn(&str, usize, bool) -> bool,
) -> bool {
    roots_with_permissions(
        pending
            .into_iter()
            .map(|(value, depth, logical)| Root {
                value,
                depth,
                logical,
                eager: false,
            })
            .collect(),
        computed_gate,
        computed_bool_operands,
        bool_field,
        logical_argument,
    )
}

pub(in crate::lowering::buffer_loop_outline) fn computed_initializer_expression_roots(
    pending: Vec<Root<'_>>,
    layouts: &impl control_values::ValueLayouts,
) -> bool {
    roots_with_permissions(
        pending,
        true,
        false,
        |record, field| boolean_field(layouts, record, field),
        |_, _, _| false,
    )
}

fn roots_with_permissions(
    pending: Vec<Root<'_>>,
    computed_gate: bool,
    computed_bool_operands: bool,
    bool_field: impl Fn(&str, &str) -> bool,
    logical_argument: impl Fn(&str, usize, bool) -> bool,
) -> bool {
    let mut pending = pending
        .into_iter()
        .map(|root| {
            (
                root.value,
                root.depth,
                root.logical,
                true,
                false,
                root.eager,
            )
        })
        .collect::<Vec<_>>();
    let mut work = 4096usize;
    let mut logical_edges = 32usize;
    while let Some((expr, depth, logical_root, record_root, lazy_rhs, eager_root)) = pending.pop() {
        if depth >= 64 || work == 0 {
            return false;
        }
        work -= 1;
        match expr {
            NirExpr::Int(_)
            | NirExpr::Bool(_)
            | NirExpr::F32(_)
            | NirExpr::F64(_)
            | NirExpr::Var(_) => {}
            NirExpr::Binary {
                op: NirBinaryOp::And | NirBinaryOp::Or,
                lhs,
                rhs,
            } => {
                if !logical_root
                    || (!computed_gate && depth != 0)
                    || (computed_gate && logical_edges == 0)
                    || (!computed_gate
                        && !matches!(lhs.as_ref(), NirExpr::Bool(_) | NirExpr::Var(_)))
                    || pending.len() + 2 > work
                {
                    return false;
                }
                if computed_gate {
                    logical_edges -= 1;
                }
                pending.extend([
                    (
                        lhs.as_ref(),
                        depth + 1,
                        computed_gate,
                        false,
                        lazy_rhs,
                        eager_root,
                    ),
                    (
                        rhs.as_ref(),
                        depth + 1,
                        computed_gate,
                        false,
                        true,
                        eager_root,
                    ),
                ]);
            }
            NirExpr::Binary { op, lhs, rhs } => {
                if pending.len() + 2 > work {
                    return false;
                }
                // Eager bool operators evaluate both operands without authorizing a lazy RHS.
                let eager_bool = (computed_bool_operands || eager_root)
                    && logical_root
                    && matches!(op, NirBinaryOp::Eq | NirBinaryOp::Ne | NirBinaryOp::Xor);
                pending.extend([
                    (
                        lhs.as_ref(),
                        depth + 1,
                        eager_bool,
                        false,
                        lazy_rhs,
                        eager_root && eager_bool,
                    ),
                    (
                        rhs.as_ref(),
                        depth + 1,
                        eager_bool,
                        false,
                        lazy_rhs,
                        eager_root && eager_bool,
                    ),
                ]);
            }
            NirExpr::Call { callee, args } => {
                if pending.len() + args.len() > work {
                    return false;
                }
                pending.extend(args.iter().enumerate().map(|(index, arg)| {
                    (
                        arg,
                        depth + 1,
                        logical_argument(callee, index, lazy_rhs),
                        false,
                        lazy_rhs,
                        false,
                    )
                }));
            }
            NirExpr::StructLiteral {
                type_name, fields, ..
            } => {
                if pending.len() + fields.len() > work {
                    return false;
                }
                pending.extend(fields.iter().map(|(name, value)| {
                    (
                        value,
                        depth + 1,
                        record_root && bool_field(type_name, name),
                        record_root,
                        lazy_rhs,
                        false,
                    )
                }));
            }
            NirExpr::FieldAccess { base, .. }
            | NirExpr::CastI64ToI32(base)
            | NirExpr::CastI32ToI64(base)
            | NirExpr::PackF32Word(base)
            | NirExpr::UnpackF32Word(base)
            | NirExpr::PackF64Word(base)
            | NirExpr::UnpackF64Word(base) => {
                pending.push((base.as_ref(), depth + 1, false, false, lazy_rhs, false))
            }
            _ => return false,
        }
    }
    true
}
