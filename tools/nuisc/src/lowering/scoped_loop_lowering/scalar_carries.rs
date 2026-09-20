use super::*;
use nuis_semantics::model::NirParam;

pub(super) struct Projection<'a> {
    pub name: &'a str,
    pub ty: &'a NirTypeRef,
    fields: Vec<String>,
}

impl Projection<'_> {
    fn width(&self) -> usize {
        self.fields.len().max(1)
    }
}

pub(super) fn projected_bindings<'a>(
    result: &str,
    ty: &NirTypeRef,
    tail: &'a [NirStmt],
    definitions: &BTreeMap<&str, &NirStructDef>,
) -> Option<Vec<Projection<'a>>> {
    let words = flat_fields(ty, definitions)?;
    if words
        .iter()
        .enumerate()
        .any(|(index, field)| field != &format!("carry{index}"))
    {
        return None;
    }
    let mut bindings: Vec<Projection<'a>> = Vec::new();
    let mut slot = 0;
    while slot < words.len() {
        let NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        } = tail.get(bindings.len())?
        else {
            return None;
        };
        if name == result || bindings.iter().any(|binding| binding.name == name) {
            return None;
        }
        let fields = if is_scalar_i64(ty) {
            if !projected_word(value, result, slot) {
                return None;
            }
            Vec::new()
        } else {
            let fields = flat_fields(ty, definitions)?;
            let NirExpr::StructLiteral {
                type_name,
                type_args,
                fields: values,
            } = value
            else {
                return None;
            };
            if type_name != &ty.name
                || !type_args.is_empty()
                || values.len() != fields.len()
                || !values.iter().zip(&fields).enumerate().all(
                    |(offset, ((name, value), field))| {
                        name == field && projected_word(value, result, slot + offset)
                    },
                )
            {
                return None;
            }
            fields
        };
        let projection = Projection { name, ty, fields };
        slot += projection.width();
        bindings.push(projection);
    }
    if slot != words.len()
        || (bindings.len() == 1
            && bindings[0].fields.is_empty()
            && !break_guard(tail.get(1), bindings[0].name))
    {
        return None;
    }
    Some(bindings)
}

fn projected_word(value: &NirExpr, result: &str, slot: usize) -> bool {
    matches!(value, NirExpr::FieldAccess { base, field }
        if matches!(base.as_ref(), NirExpr::Var(name) if name == result)
            && field == &format!("carry{slot}"))
}

fn flat_fields(
    ty: &NirTypeRef,
    definitions: &BTreeMap<&str, &NirStructDef>,
) -> Option<Vec<String>> {
    let definition = definitions.get(ty.name.as_str())?;
    if ty.is_ref
        || ty.is_optional
        || !ty.generic_args.is_empty()
        || !definition.generic_params.is_empty()
        || !definition.where_bounds.is_empty()
        || definition.fields.is_empty()
        || definition
            .fields
            .iter()
            .any(|field| !is_scalar_i64(&field.ty))
    {
        return None;
    }
    Some(
        definition
            .fields
            .iter()
            .map(|field| field.name.clone())
            .collect(),
    )
}

pub(super) fn supported_parameter(ty: &NirTypeRef, state: &LoweringState<'_>) -> bool {
    is_scalar_i64(ty)
        || (!ty.is_optional
            && ty.generic_args.is_empty()
            && ((!ty.is_ref && matches!(ty.name.as_str(), "bool" | "i32" | "f32" | "f64"))
                || (ty.is_ref && ty.name == "Buffer")))
        || flat_fields(ty, &state.struct_defs).is_some()
}

pub(super) fn break_guard(stmt: Option<&NirStmt>, binding: &str) -> bool {
    matches!(stmt, Some(NirStmt::If {
        condition: NirExpr::Binary { op: NirBinaryOp::Eq, lhs, rhs }, then_body, else_body,
    }) if matches!(lhs.as_ref(), NirExpr::Var(name) if name == binding)
        && rhs.as_ref() == &NirExpr::Int(1)
        && then_body == &[NirStmt::Break] && else_body.is_empty())
}

pub(super) fn admissible(
    carries: &[Projection<'_>],
    prepared: &PreparedCountedWhile,
    function: &NirFunction,
    args: &[NirExpr],
    bindings: &BTreeMap<String, String>,
    breaking: bool,
    state: &LoweringState<'_>,
) -> bool {
    let control = breaking.then(|| carries.last().expect("nonempty projections").name);
    let changed = carries.iter().map(|binding| binding.name).collect();
    carries.iter().all(|binding| {
        binding.name != prepared.binding_name
            && (control == Some(binding.name) || bindings.contains_key(binding.name))
            && (control != Some(binding.name) || binding.fields.is_empty())
    }) && !loop_purity::expr_references_names(&prepared.limit, &changed)
        && !loop_purity::expr_references_names(&prepared.step, &changed)
        && function.params.iter().zip(args).all(|(param, arg)| {
            if control == Some(param.name.as_str()) {
                arg == &NirExpr::Int(0)
            } else {
                matches!(arg, NirExpr::Var(_))
            }
        })
        && function
            .params
            .iter()
            .all(|param| supported_parameter(&param.ty, state))
}

pub(super) fn validate_seeds(
    carries: &[Projection<'_>],
    breaking: bool,
    function: &NirFunction,
    args: &[NirExpr],
    callee: &str,
) -> Result<(), String> {
    for (index, binding) in carries.iter().enumerate() {
        let control = breaking && index + 1 == carries.len();
        if control && !is_scalar_i64(binding.ty) {
            return Err(format!("scoped break flag `{}` must be i64", binding.name));
        }
        let seeds = function
            .params
            .iter()
            .zip(args)
            .filter(|(param, arg)| {
                &param.ty == binding.ty
                    && if control {
                        param.name == binding.name && *arg == &NirExpr::Int(0)
                    } else {
                        matches!(arg, NirExpr::Var(name) if name == binding.name)
                    }
            })
            .count();
        if seeds != 1 {
            return Err(format!(
                "scoped carry `{}` from `{callee}` requires exactly one same-typed {}seed",
                binding.name,
                if control { "zero " } else { "" }
            ));
        }
    }
    Ok(())
}

pub(super) fn argument_index(
    result: &ScopedLoopResult<'_>,
    param: &NirParam,
    arg: &NirExpr,
) -> Option<(usize, usize)> {
    let ScopedLoopResult::Scalars {
        bindings, breaking, ..
    } = result
    else {
        return None;
    };
    if *breaking
        && bindings.last().map(|binding| binding.name) == Some(param.name.as_str())
        && arg == &NirExpr::Int(0)
    {
        return Some((bindings.iter().map(Projection::width).sum::<usize>() - 1, 1));
    }
    let NirExpr::Var(name) = arg else {
        return None;
    };
    let mut offset = 0;
    for binding in bindings {
        if binding.name == name && binding.ty == &param.ty {
            return Some((offset, binding.width()));
        }
        offset += binding.width();
    }
    None
}

pub(super) fn field(result: &str, field: String, state: &mut LoweringState<'_>) -> String {
    let name = next_name(state, "loop_scalar_result");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "field".to_owned(),
            args: vec![result.to_owned(), field],
        },
    });
    push_dep_edges(state, result, &name);
    name
}

pub(super) fn bind_result(
    result: &str,
    carries: &[Projection<'_>],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    const_bindings: &mut BTreeMap<String, NirExpr>,
) {
    let mut slot = 0;
    for binding in carries {
        let words = (0..binding.width())
            .map(|_| {
                let value = field(result, format!("carry{slot}"), state);
                slot += 1;
                value
            })
            .collect::<Vec<_>>();
        let value = if binding.fields.is_empty() {
            words[0].clone()
        } else {
            // New value nodes preserve pre-loop snapshots, including on zero trips.
            let name = next_name(state, "loop_value_result");
            let mut args = vec![binding.ty.name.clone()];
            args.extend(
                binding
                    .fields
                    .iter()
                    .zip(&words)
                    .map(|(field, word)| format!("{field}={word}")),
            );
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "struct".to_owned(),
                    args,
                },
            });
            for word in words {
                push_dep_edges(state, &word, &name);
            }
            name
        };
        bindings.insert(binding.name.to_owned(), value);
        const_bindings.remove(binding.name);
    }
}

#[cfg(test)]
#[path = "scalar_carries_tests.rs"]
mod tests;
