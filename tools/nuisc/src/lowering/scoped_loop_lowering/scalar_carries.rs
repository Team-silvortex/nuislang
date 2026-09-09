use super::*;
use nuis_semantics::model::NirParam;

pub(super) fn projected_bindings<'a>(
    result: &str,
    ty: &NirTypeRef,
    tail: &'a [NirStmt],
    state: &LoweringState<'_>,
) -> Option<Vec<&'a str>> {
    let definition = state.struct_defs.get(ty.name.as_str())?;
    if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() || definition.fields.is_empty() {
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
    if bindings.len() == 1 && !break_guard(tail.get(1), bindings[0]) {
        return None;
    }
    Some(bindings)
}

pub(super) fn break_guard(stmt: Option<&NirStmt>, binding: &str) -> bool {
    matches!(stmt, Some(NirStmt::If {
        condition: NirExpr::Binary { op: NirBinaryOp::Eq, lhs, rhs }, then_body, else_body,
    }) if matches!(lhs.as_ref(), NirExpr::Var(name) if name == binding)
        && rhs.as_ref() == &NirExpr::Int(1)
        && then_body == &[NirStmt::Break] && else_body.is_empty())
}

pub(super) fn admissible(
    carries: &[&str],
    prepared: &PreparedCountedWhile,
    function: &NirFunction,
    args: &[NirExpr],
    bindings: &BTreeMap<String, String>,
    breaking: bool,
) -> bool {
    let control = breaking.then(|| *carries.last().expect("nonempty projections"));
    let changed = carries.iter().copied().collect();
    carries.iter().all(|binding| {
        *binding != prepared.binding_name
            && (control == Some(*binding) || bindings.contains_key(*binding))
    }) && !loop_purity::expr_references_names(&prepared.limit, &changed)
        && !loop_purity::expr_references_names(&prepared.step, &changed)
        && function.params.iter().zip(args).all(|(param, arg)| {
            if control == Some(param.name.as_str()) {
                arg == &NirExpr::Int(0)
            } else {
                matches!(arg, NirExpr::Var(_))
            }
        })
        && function.params.iter().all(|param| {
            is_scalar_i64(&param.ty)
                || (!param.ty.is_optional
                    && param.ty.generic_args.is_empty()
                    && ((!param.ty.is_ref && param.ty.name == "bool")
                        || (param.ty.is_ref && param.ty.name == "Buffer")))
        })
}

pub(super) fn argument_index(
    result: &ScopedLoopResult<'_>,
    param: &NirParam,
    arg: &NirExpr,
) -> Option<usize> {
    let ScopedLoopResult::Scalars {
        bindings, breaking, ..
    } = result
    else {
        return None;
    };
    if *breaking && bindings.last().copied() == Some(param.name.as_str()) && arg == &NirExpr::Int(0)
    {
        return Some(bindings.len() - 1);
    }
    let NirExpr::Var(name) = arg else {
        return None;
    };
    bindings.iter().position(|binding| *binding == name)
}
