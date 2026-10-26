use super::*;

#[path = "logical_field_captures.rs"]
mod field_captures;

// Admitted conditions are ordinary function expressions, not loop-metadata
// predicates. Share guarded bool helpers across iteration and scalar control.
// Only a non-total RHS needs a helper; ready comparisons can use the value path.
pub(super) fn outline(
    function: &mut NirFunction,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::CarryLayouts,
) {
    let mut scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    for stmt in &mut function.body {
        if let NirStmt::Print(value) = stmt {
            *value = predicate_inner(
                value.clone(),
                &scope,
                names,
                helpers,
                guarded,
                Some(layouts),
            );
            continue;
        }
        if let NirStmt::Let { name, ty, value } = stmt {
            // Inferred boolean temporaries need the same lazy lowering as
            // explicit bool bindings and generated branch predicates.
            let inferred = ty
                .clone()
                .or_else(|| control_values::value_type(value, &scope, catalog, layouts));
            if let Some(inferred) = inferred {
                // Logical edges can also occur inside an i64 call's arguments.
                *value = predicate_inner(
                    value.clone(),
                    &scope,
                    names,
                    helpers,
                    guarded,
                    Some(layouts),
                );
                *ty = Some(inferred.clone());
                scope.insert(name.clone(), inferred);
            }
        }
    }
}

pub(super) fn predicate(
    expr: NirExpr,
    scope: &Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
) -> NirExpr {
    predicate_inner(expr, scope, names, helpers, guarded, None)
}

fn predicate_inner(
    expr: NirExpr,
    scope: &Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    layouts: Option<&control_values::CarryLayouts>,
) -> NirExpr {
    let NirExpr::Binary {
        op: op @ (NirBinaryOp::And | NirBinaryOp::Or),
        lhs,
        rhs,
    } = expr
    else {
        return match expr {
            NirExpr::Call { callee, args } => NirExpr::Call {
                callee,
                args: args
                    .into_iter()
                    .map(|arg| predicate_inner(arg, scope, names, helpers, guarded, layouts))
                    .collect(),
            },
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op,
                lhs: Box::new(predicate_inner(
                    *lhs, scope, names, helpers, guarded, layouts,
                )),
                rhs: Box::new(predicate_inner(
                    *rhs, scope, names, helpers, guarded, layouts,
                )),
            },
            NirExpr::StructLiteral {
                type_name,
                type_args,
                fields,
            } => NirExpr::StructLiteral {
                type_name,
                type_args,
                fields: fields
                    .into_iter()
                    .map(|(name, value)| {
                        (
                            name,
                            predicate_inner(value, scope, names, helpers, guarded, layouts),
                        )
                    })
                    .collect(),
            },
            NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
                base: Box::new(predicate_inner(
                    *base, scope, names, helpers, guarded, layouts,
                )),
                field,
            },
            NirExpr::PackF32Word(v) => NirExpr::PackF32Word(Box::new(predicate_inner(
                *v, scope, names, helpers, guarded, layouts,
            ))),
            NirExpr::UnpackF32Word(v) => NirExpr::UnpackF32Word(Box::new(predicate_inner(
                *v, scope, names, helpers, guarded, layouts,
            ))),
            NirExpr::PackF64Word(v) => NirExpr::PackF64Word(Box::new(predicate_inner(
                *v, scope, names, helpers, guarded, layouts,
            ))),
            NirExpr::UnpackF64Word(v) => NirExpr::UnpackF64Word(Box::new(predicate_inner(
                *v, scope, names, helpers, guarded, layouts,
            ))),
            NirExpr::CastI64ToI32(v) => NirExpr::CastI64ToI32(Box::new(predicate_inner(
                *v, scope, names, helpers, guarded, layouts,
            ))),
            NirExpr::CastI32ToI64(v) => NirExpr::CastI32ToI64(Box::new(predicate_inner(
                *v, scope, names, helpers, guarded, layouts,
            ))),
            _ => expr,
        };
    };
    if ready_values::total_predicate(&rhs, scope) {
        return NirExpr::Binary {
            op,
            lhs: Box::new(predicate_inner(
                *lhs, scope, names, helpers, guarded, layouts,
            )),
            rhs,
        };
    }
    let disjunction = op == NirBinaryOp::Or;
    let projected = layouts.and_then(|layouts| field_captures::collect(&rhs, scope, layouts));
    let (rhs, mut params, captures) = if let Some(projected) = projected {
        (projected.value, projected.params, projected.args)
    } else {
        let mut inputs = BTreeSet::new();
        control_values::collect_inputs(&rhs, &mut inputs);
        let params = captured_params(inputs, scope);
        let args = params
            .iter()
            .map(|p| NirExpr::Var(p.name.clone()))
            .collect();
        (*rhs, params, args)
    };
    let mut bindings: BTreeSet<String> = scope.keys().cloned().collect();
    bindings.extend(params.iter().map(|p| p.name.clone()));
    let gate = branches::fresh_name("__nuis_predicate_gate", &mut bindings);
    let lhs = predicate_inner(*lhs, scope, names, helpers, guarded, layouts);
    let mut args = vec![if disjunction { negate(lhs) } else { lhs }];
    args.extend(captures);
    params.insert(
        0,
        NirParam {
            name: gate.clone(),
            ty: scalar_type("bool"),
        },
    );
    let rhs_scope = params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let rhs = predicate_inner(rhs, &rhs_scope, names, helpers, guarded, layouts);
    let name = branches::fresh_name("__nuis_scalar_predicate", names);
    let mut function = helper(
        name.clone(),
        params,
        vec![
            NirStmt::If {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Ne,
                    lhs: Box::new(NirExpr::Var(gate)),
                    rhs: Box::new(NirExpr::Bool(true)),
                },
                then_body: vec![NirStmt::Return(Some(NirExpr::Bool(false)))],
                else_body: vec![],
            },
            NirStmt::Return(Some(if disjunction { negate(rhs) } else { rhs })),
        ],
    );
    function.return_type = Some(scalar_type("bool"));
    guarded.insert(name.clone());
    helpers.push(function);
    let call = NirExpr::Call { callee: name, args };
    // a || b = !(!a && !b). Both forms use the existing neutral-false guard;
    // no new return authority or duplicated evaluation is required.
    if disjunction {
        negate(call)
    } else {
        call
    }
}

fn negate(value: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op: NirBinaryOp::Eq,
        lhs: Box::new(value),
        rhs: Box::new(NirExpr::Bool(false)),
    }
}
