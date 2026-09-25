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

    pub(super) fn whole_record_seed(&self, param: &NirParam, arg: &NirExpr) -> bool {
        !self.fields.is_empty() && seed_range(self, param, arg) == Some((0, self.width()))
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
        } else if is_bool(ty) {
            if !matches!(value, NirExpr::CastI64ToBool(word) if projected_word(word, result, slot))
            {
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
            && is_scalar_i64(bindings[0].ty)
            && !break_guard(tail.get(1), bindings[0].name))
    {
        return None;
    }
    Some(bindings)
}

fn is_bool(ty: &NirTypeRef) -> bool {
    ty.name == "bool" && !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty()
}

fn seed_range(binding: &Projection<'_>, param: &NirParam, arg: &NirExpr) -> Option<(usize, usize)> {
    let whole = if is_bool(binding.ty) {
        is_scalar_i64(&param.ty)
            && matches!(arg, NirExpr::CastBoolToI64(value)
                if matches!(value.as_ref(), NirExpr::Var(name) if name == binding.name))
    } else {
        &param.ty == binding.ty && matches!(arg, NirExpr::Var(name) if name == binding.name)
    };
    if whole {
        return Some((0, binding.width()));
    }
    // Only declared flat-i64 fields can name a carried slot. These are dynamic
    // backedge operands, never invariant field reads hoisted out of the loop.
    let NirExpr::FieldAccess { base, field } = arg else {
        return None;
    };
    if !is_scalar_i64(&param.ty)
        || !matches!(base.as_ref(), NirExpr::Var(name) if name == binding.name)
    {
        return None;
    }
    binding
        .fields
        .iter()
        .position(|name| name == field)
        .map(|index| (index, 1))
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
    invariant_inputs: &BTreeSet<&str>,
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
                arguments::ready(arg, invariant_inputs)
                    || carries
                        .iter()
                        .any(|binding| seed_range(binding, param, arg).is_some())
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
    if function.params.len() != args.len() {
        return Err(format!("scoped carry seed arity mismatch for `{callee}`"));
    }
    for (index, binding) in carries.iter().enumerate() {
        let control = breaking && index + 1 == carries.len();
        if control && !is_scalar_i64(binding.ty) {
            return Err(format!("scoped break flag `{}` must be i64", binding.name));
        }
        let covered = seed_coverage(binding, control, function, args, callee)?;
        if covered.iter().any(|covered| !covered) {
            return Err(format!(
                "scoped carry `{}` from `{callee}` requires exactly one same-typed {}seed per slot (bool uses an explicit i64 word)",
                binding.name,
                if control { "zero " } else { "" }
            ));
        }
    }
    Ok(())
}

fn seed_coverage(
    binding: &Projection<'_>,
    control: bool,
    function: &NirFunction,
    args: &[NirExpr],
    callee: &str,
) -> Result<Vec<bool>, String> {
    let mut covered = vec![false; binding.width()];
    for (param, arg) in function.params.iter().zip(args) {
        let range = if control {
            (&param.ty == binding.ty && param.name == binding.name && arg == &NirExpr::Int(0))
                .then_some((0, 1))
        } else {
            seed_range(binding, param, arg)
        };
        if let Some((start, width)) = range {
            for slot in &mut covered[start..start + width] {
                if std::mem::replace(slot, true) {
                    return Err(format!(
                        "scoped carry `{}` from `{callee}` has duplicate seed coverage",
                        binding.name
                    ));
                }
            }
        }
    }
    Ok(covered)
}

pub(super) fn needs_separate_seeds(
    carries: &[Projection<'_>],
    breaking: bool,
    function: &NirFunction,
    args: &[NirExpr],
    callee: &str,
) -> Result<bool, String> {
    let error = match validate_seeds(carries, breaking, function, args, callee) {
        Ok(()) => return Ok(false),
        Err(error) => error,
    };
    if function.params.len() != args.len() {
        return Err(error);
    }
    for (index, binding) in carries.iter().enumerate() {
        let control = breaking && index + 1 == carries.len();
        let covered = seed_coverage(binding, control, function, args, callee)?;
        if control && !is_scalar_i64(binding.ty) {
            return Err(error);
        }
        if covered.iter().any(|covered| !covered)
            && (control || binding.fields.is_empty() || !covered.iter().any(|covered| *covered))
        {
            return Err(error);
        }
    }
    Ok(true)
}

pub(super) fn lower_initial_seeds(
    carries: &[Projection<'_>],
    breaking: bool,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<Vec<String>, String> {
    let mut seeds = Vec::new();
    for (index, binding) in carries.iter().enumerate() {
        if breaking && index + 1 == carries.len() {
            seeds.push(lower_expr(&NirExpr::Int(0), state, bindings)?);
        } else if is_bool(binding.ty) {
            seeds.push(lower_expr(
                &NirExpr::CastBoolToI64(Box::new(NirExpr::Var(binding.name.to_owned()))),
                state,
                bindings,
            )?);
        } else {
            let initial = bindings
                .get(binding.name)
                .ok_or_else(|| format!("missing initial carry `{}`", binding.name))?;
            seeds.extend(direct_calls::flatten_direct_call_argument(
                binding.ty, initial, state,
            )?);
        }
    }
    Ok(seeds)
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
    let mut offset = 0;
    for binding in bindings {
        if let Some((field, width)) = seed_range(binding, param, arg) {
            return Some((offset + field, width));
        }
        offset += binding.width();
    }
    None
}

pub(super) fn validate_field_seed_origins(
    carries: &[Projection<'_>],
    args: &[NirExpr],
    bindings: &BTreeMap<String, String>,
    state: &LoweringState<'_>,
    require_all: bool,
) -> Result<(), String> {
    for binding in carries {
        let field_mapped = !binding.fields.is_empty()
            && (require_all
                || args.iter().any(|arg| {
                    matches!(arg, NirExpr::FieldAccess { base, .. }
                    if matches!(base.as_ref(), NirExpr::Var(name) if name == binding.name))
                }));
        if !field_mapped {
            continue;
        }
        let actual = bindings
            .get(binding.name)
            .and_then(|node| initial_record_type(node, state));
        if actual.as_deref() != Some(binding.ty.name.as_str()) {
            return Err(format!(
                "scoped field seed `{}` requires initial record `{}`, found {}",
                binding.name,
                binding.ty.name,
                actual.as_deref().unwrap_or("an unproven record type")
            ));
        }
    }
    Ok(())
}

fn initial_record_type(node: &str, state: &LoweringState<'_>) -> Option<String> {
    // Field operands erase their source's nominal type. Check that type before
    // flattening, or even a zero-trip loop could reconstruct a different record.
    let mut node = node;
    let mut path = Vec::new();
    let mut seen = BTreeSet::new();
    let mut ty = loop {
        if !seen.insert(node) {
            return None;
        }
        let op = &state.yir.nodes.iter().rev().find(|n| n.name == node)?.op;
        if op.module != "cpu" {
            return None;
        }
        let name = match op.instruction.as_str() {
            "struct" => op.args.first()?.clone(),
            "call_owned_struct" => {
                break state
                    .function_map
                    .get(op.args.first()?.as_str())?
                    .return_type
                    .clone()?;
            }
            "loop_owned_struct_result" => {
                yir_core::parse_owned_struct_layout(op.args.get(1)?)
                    .ok()?
                    .type_name
            }
            "field" => {
                path.push(op.args.get(1)?.as_str());
                node = op.args.first()?;
                continue;
            }
            _ => return None,
        };
        break NirTypeRef {
            name,
            generic_args: Vec::new(),
            is_ref: false,
            is_optional: false,
        };
    };
    for field in path.into_iter().rev() {
        if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() {
            return None;
        }
        ty = state
            .struct_defs
            .get(ty.name.as_str())?
            .fields
            .iter()
            .find(|candidate| candidate.name == field)?
            .ty
            .clone();
    }
    flat_fields(&ty, &state.struct_defs).map(|_| ty.name)
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
        let value = if is_bool(binding.ty) {
            let name = next_name(state, "loop_bool_result");
            state.yir.nodes.push(Node {
                name: name.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "cast_i64_to_bool".to_owned(),
                    args: vec![words[0].clone()],
                },
            });
            push_dep_edges(state, &words[0], &name);
            name
        } else if binding.fields.is_empty() {
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
