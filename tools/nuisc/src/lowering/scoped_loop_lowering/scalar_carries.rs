use super::*;

pub(super) fn projected_bindings<'a>(
    result: &str,
    ty: &NirTypeRef,
    tail: &'a [NirStmt],
    state: &LoweringState<'_>,
) -> Option<Vec<&'a str>> {
    let definition = state.struct_defs.get(ty.name.as_str())?;
    if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() || definition.fields.len() < 2 {
        return None;
    }
    let mut bindings = Vec::new();
    for (index, field) in definition.fields.iter().enumerate() {
        let NirStmt::Let {
            name,
            ty,
            value:
                NirExpr::FieldAccess {
                    base,
                    field: projected,
                },
        } = tail.get(index)?
        else {
            return None;
        };
        if !is_scalar_i64(&field.ty)
            || field.name != format!("carry{index}")
            || projected != &field.name
            || ty.as_ref().is_none_or(|ty| !is_scalar_i64(ty))
            || !matches!(base.as_ref(), NirExpr::Var(name) if name == result)
            || name == result
            || bindings.contains(&name.as_str())
        {
            return None;
        }
        bindings.push(name.as_str());
    }
    Some(bindings)
}

pub(super) fn admissible(
    carries: &[&str],
    prepared: &PreparedCountedWhile,
    function: &NirFunction,
    args: &[NirExpr],
    bindings: &BTreeMap<String, String>,
) -> bool {
    let changed = carries.iter().copied().collect();
    carries
        .iter()
        .all(|binding| *binding != prepared.binding_name && bindings.contains_key(*binding))
        && !loop_purity::expr_references_names(&prepared.limit, &changed)
        && !loop_purity::expr_references_names(&prepared.step, &changed)
        && args.iter().all(|arg| matches!(arg, NirExpr::Var(_)))
        && function.params.iter().all(|param| {
            is_scalar_i64(&param.ty)
                || (!param.ty.is_optional
                    && param.ty.generic_args.is_empty()
                    && ((!param.ty.is_ref && param.ty.name == "bool")
                        || (param.ty.is_ref && param.ty.name == "Buffer")))
        })
}

pub(super) fn argument_index(result: &ScopedLoopResult<'_>, arg: &NirExpr) -> Option<usize> {
    let (ScopedLoopResult::Scalars { bindings, .. }, NirExpr::Var(name)) = (result, arg) else {
        return None;
    };
    bindings.iter().position(|binding| *binding == name)
}
