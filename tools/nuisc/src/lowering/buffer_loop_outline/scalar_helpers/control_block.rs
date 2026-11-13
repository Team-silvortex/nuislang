use super::*;

// Only a complete, fresh paired value result can leave a branch scope.
pub(super) fn validate_block(
    body: &[NirStmt],
    locals: &mut Scope,
    loop_bindings: &mut BTreeSet<String>,
    result: &NirTypeRef,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
    loop_layouts: Option<&control_values::CarryLayouts>,
    profile: Profile,
) -> Option<bool> {
    let mut returned = false;
    for (index, stmt) in body.iter().enumerate() {
        if returned {
            return None;
        }
        if matches!(profile, Profile::IterationPrints) {
            if let Some(join) = scalar_joins::iteration::declaration(
                stmt,
                locals,
                &body[index + 1..],
                catalog,
                layouts,
            ) {
                locals.insert(join.name.to_owned(), join.ty.clone());
                loop_bindings.insert(join.name.to_owned());
                continue;
            }
            if let Some(join) = scalar_joins::iteration::prefixed_declaration(
                stmt,
                locals,
                &body[index + 1..],
                catalog,
                layouts,
            ) {
                locals.insert(join.name.to_owned(), join.ty.clone());
                loop_bindings.insert(join.name.to_owned());
                continue;
            }
        }
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                let declared = match stmt {
                    NirStmt::Let { ty, .. } => ty.as_ref(),
                    NirStmt::Const { ty, .. } => Some(ty),
                    _ => unreachable!(),
                };
                let inferred = control_values::value_type(value, locals, catalog, layouts)?;
                if declared.is_some_and(|ty| ty != &inferred) {
                    return None;
                }
                // Record writes retain their exact nominal value type. Guarded
                // helpers preserve evaluation order; capture snapshot identity
                // is normalized separately, without changing scalar/loop rules.
                if let Some(previous) = locals.get(name) {
                    if !matches!(stmt, NirStmt::Let { .. })
                        || previous != &inferred
                        || layouts.scalar(&inferred.name)
                    {
                        return None;
                    }
                }
                locals.insert(name.clone(), inferred);
                if matches!(stmt, NirStmt::Let { .. }) {
                    loop_bindings.insert(name.clone());
                }
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if control_values::value_type(condition, locals, catalog, layouts)?
                    != scalar_type("bool")
                {
                    return None;
                }
                let validate_arm = |body: &[NirStmt]| {
                    validate_block(
                        body,
                        &mut locals.clone(),
                        &mut loop_bindings.clone(),
                        result,
                        catalog,
                        layouts,
                        loop_layouts,
                        profile,
                    )
                };
                let then_returns = validate_arm(then_body)?;
                let else_returns = validate_arm(else_body)?;
                returned = then_returns && else_returns;
            }
            NirStmt::While { condition, body } => {
                control_loops::validate(
                    condition,
                    body,
                    locals,
                    loop_bindings,
                    catalog,
                    loop_layouts?,
                    match profile {
                        Profile::Pure => validation::LiteralPrints::Reject,
                        Profile::IterationPrints => validation::LiteralPrints::Scalars,
                    },
                )?;
            }
            NirStmt::Return(Some(value)) => {
                if &control_values::value_type(value, locals, catalog, layouts)? != result {
                    return None;
                }
                returned = true;
            }
            NirStmt::Print(value) if matches!(profile, Profile::IterationPrints) => {
                iteration_prints::admit(value, locals, catalog, layouts)?;
            }
            _ => return None,
        }
    }
    Some(returned)
}
