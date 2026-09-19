use super::*;

// Conditions in the scoped iteration are ordinary function expressions, not
// loop-metadata predicates. Use guarded bool helpers so their RHS stays lazy.
// One helper per logical edge gives linear growth without duplicating arm bodies.
pub(super) fn outline(
    function: &mut NirFunction,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
    catalog: &ScalarHelpers,
    layouts: &control_values::FlatLayouts,
) {
    let mut scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    for stmt in &mut function.body {
        if let NirStmt::Let { name, ty, value } = stmt {
            // Inferred boolean temporaries need the same lazy lowering as
            // explicit bool bindings and generated branch predicates.
            let inferred = ty
                .clone()
                .or_else(|| control_values::value_type(value, &scope, catalog, layouts));
            if let Some(inferred) = inferred {
                // Logical edges can also occur inside an i64 call's arguments.
                *value = predicate(value.clone(), &scope, names, helpers, guarded);
                *ty = Some(inferred.clone());
                scope.insert(name.clone(), inferred);
            }
        }
    }
}

fn predicate(
    expr: NirExpr,
    scope: &Scope,
    names: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    guarded: &mut BTreeSet<String>,
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
                    .map(|arg| predicate(arg, scope, names, helpers, guarded))
                    .collect(),
            },
            NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
                op,
                lhs: Box::new(predicate(*lhs, scope, names, helpers, guarded)),
                rhs: Box::new(predicate(*rhs, scope, names, helpers, guarded)),
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
                    .map(|(name, value)| (name, predicate(value, scope, names, helpers, guarded)))
                    .collect(),
            },
            NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
                base: Box::new(predicate(*base, scope, names, helpers, guarded)),
                field,
            },
            _ => expr,
        };
    };
    let disjunction = op == NirBinaryOp::Or;
    let mut inputs = BTreeSet::new();
    control_values::collect_inputs(&rhs, &mut inputs);
    let mut params = captured_params(inputs, scope);
    let mut bindings = scope.keys().cloned().collect();
    let gate = branches::fresh_name("__nuis_predicate_gate", &mut bindings);
    let lhs = predicate(*lhs, scope, names, helpers, guarded);
    let mut args = vec![if disjunction { negate(lhs) } else { lhs }];
    args.extend(params.iter().map(|p| NirExpr::Var(p.name.clone())));
    params.insert(
        0,
        NirParam {
            name: gate.clone(),
            ty: scalar_type("bool"),
        },
    );
    let rhs = predicate(*rhs, scope, names, helpers, guarded);
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
