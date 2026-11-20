use super::*;

#[path = "resource_predicate_arguments.rs"]
mod arguments;
#[cfg(test)]
#[path = "resource_condition_tests.rs"]
mod condition_tests;
#[path = "resource_record_literals.rs"]
mod record_literals;
pub(super) use record_literals::LocalRecords;
#[cfg(test)]
#[path = "resource_record_initializers_tests.rs"]
mod record_tests;
#[path = "resource_return_fields.rs"]
mod return_fields;
#[cfg(test)]
#[path = "resource_scalar_initializers_tests.rs"]
mod scalar_tests;
#[cfg(test)]
#[path = "resource_predicates_tests.rs"]
mod tests;

pub(super) fn return_layouts(
    module: &NirModule,
    eligible: &BTreeSet<String>,
) -> BTreeMap<String, MixedResourceLayout> {
    return_fields::collect(module, eligible)
}

pub(super) fn checked_error_arm(
    body: &[NirStmt],
    template: &NirFunction,
    context: &OutlineContext,
) -> bool {
    let Some(ty) = template.return_type.as_ref() else {
        return false;
    };
    let Some(outcome) = context
        .return_layouts
        .get(&ty.render())
        .and_then(|l| l.outcome.as_ref())
    else {
        return false;
    };
    // Even without live Bytes, paired checked errors return owned Results, not a scalar
    // select. Match registered identity rather than a conventional type name;
    // callers keep ordinary early errors on their existing guard_return route.
    matches!(body.last(), Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields })))
        if *type_name == outcome.err_variant && *type_args == ty.generic_args
            && fields.len() == 1 && fields[0].0 == "value")
}

// Resource reads stay in owned regions, never in the pure scalar-helper ABI.
// Each lazy edge shares its suffix instead of copying the remaining source tree.
pub(super) fn split(
    statement: &NirStmt,
    template: &NirFunction,
    remaining: &mut VecDeque<NirStmt>,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    depth: usize,
    context: &mut OutlineContext,
) -> Result<Option<Vec<NirStmt>>, String> {
    if let NirStmt::Return(Some(value)) = statement {
        return Ok(return_fields::split(value, template, scope, live, context));
    }
    if let NirStmt::If { condition, .. } = statement {
        if !admitted(condition, scope, live, &context.calls, Kind::Bool)
            || !arguments::contains_logic(condition)
        {
            return Ok(None);
        }
        // Requeue the intact predicate through the declaration path. Do not
        // copy either arm into lazy edges or evaluate any child here.
        let name = fresh_left(template, scope, live, context);
        let mut branch = statement.clone();
        let NirStmt::If {
            condition: target, ..
        } = &mut branch
        else {
            unreachable!()
        };
        *target = NirExpr::Var(name.clone());
        return Ok(Some(vec![
            NirStmt::Let {
                name,
                ty: Some(NirTypeRef {
                    name: "bool".into(),
                    is_ref: false,
                    is_optional: false,
                    generic_args: vec![],
                }),
                value: condition.clone(),
            },
            branch,
        ]));
    }
    let (name, ty, value, constant) = match statement {
        NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        } => (name, ty, value, false),
        NirStmt::Const { name, ty, value } => (name, ty, value, true),
        _ => return Ok(None),
    };
    if !capture_scalar(ty) {
        if let Some(prefix) =
            record_literals::split(statement, ty, value, template, scope, live, context)
        {
            return Ok(Some(prefix));
        }
        if bounded(value).is_some()
            && context.calls.is_record(ty)
            && arguments::contains_argument_logic(value)
            && helper_value_type(value, scope, live, &context.calls).as_ref() == Some(ty)
        {
            // Freeze only scalar arguments. The original nominal record binding
            // is constructed once inside the selected continuation, not captured.
            return Ok(Some(requeue_declaration(
                statement, value, template, scope, live, context,
            )));
        }
        return Ok(return_fields::split_cleanup_binding(
            statement, template, remaining, scope, live, context,
        ));
    }
    // An i64 initializer may contain lazy bool arguments, never an i64 predicate.
    let expected = if ty.name == "bool" {
        Kind::Bool
    } else {
        Kind::I64
    };
    if !admitted(value, scope, live, &context.calls, expected) {
        return Ok(None);
    }
    if logical_parts(value).is_none() && arguments::contains_logic(value) {
        return Ok(Some(requeue_declaration(
            statement, value, template, scope, live, context,
        )));
    }
    let Some((left, right, disjunction, inverted)) =
        logical_parts(value).filter(|_| needs_guard(value))
    else {
        return Ok(None);
    };
    if needs_guard(left) {
        // Evaluate a lazy LHS once, in the old lexical scope. Requeue both
        // declarations so the ordinary owned-region machinery guards it too.
        let temporary = fresh_left(template, scope, live, context);
        let mut rewritten = statement.clone();
        let (NirStmt::Let { value, .. } | NirStmt::Const { value, .. }) = &mut rewritten else {
            unreachable!()
        };
        let logical = NirExpr::Binary {
            op: if disjunction {
                NirBinaryOp::Or
            } else {
                NirBinaryOp::And
            },
            lhs: Box::new(NirExpr::Var(temporary.clone())),
            rhs: Box::new(right.clone()),
        };
        *value = if inverted {
            NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(logical),
                rhs: Box::new(NirExpr::Bool(false)),
            }
        } else {
            logical
        };
        return Ok(Some(vec![
            NirStmt::Let {
                name: temporary,
                ty: Some(ty.clone()),
                value: left.clone(),
            },
            rewritten,
        ]));
    }
    let mut captures = scope.to_vec();
    if let Some(old) = captures.iter_mut().find(|p| p.name == *name) {
        old.ty = ty.clone();
    } else {
        captures.push(NirParam {
            name: name.clone(),
            ty: ty.clone(),
        });
    }
    // Share the suffix once. Its scalar argument is defined on each edge;
    // all existing Bytes owners move through the ordinary continuation proof.
    let tail = if remaining.len() == 1
        && matches!(remaining.front(), Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if context.outlines.owners.get(callee).is_some_and(|edges| identity_handoff(edges, live))
                && args.iter().all(|arg| matches!(arg, NirExpr::Var(_))))
    {
        // An inner predicate already ends at the outer predicate's shared
        // continuation with the complete live set and no cleanup obligation.
        // Only that identity handoff can bypass another continuation helper.
        remaining.pop_front().expect("one proved terminal call")
    } else {
        continuations::share(
            template,
            std::mem::take(remaining).into_iter().collect(),
            &captures,
            live,
            depth + 2,
            context,
        )?
    };
    let arm = |value| {
        let declaration = if constant {
            NirStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value,
            }
        } else {
            NirStmt::Let {
                name: name.clone(),
                ty: Some(ty.clone()),
                value,
            }
        };
        vec![declaration, tail.clone()]
    };
    let selected = arm(if inverted {
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(right.clone()),
            rhs: Box::new(NirExpr::Bool(false)),
        }
    } else {
        right.clone()
    });
    let skipped = arm(NirExpr::Bool(disjunction ^ inverted));
    let (then_body, else_body) = if disjunction {
        (skipped, selected)
    } else {
        (selected, skipped)
    };
    Ok(Some(vec![NirStmt::If {
        condition: left.clone(),
        then_body,
        else_body,
    }]))
}

fn requeue_declaration(
    statement: &NirStmt,
    value: &NirExpr,
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
) -> Vec<NirStmt> {
    let (mut prefix, value) = arguments::extract(value, template, scope, live, context);
    let mut declaration = statement.clone();
    let (NirStmt::Let { value: target, .. } | NirStmt::Const { value: target, .. }) =
        &mut declaration
    else {
        unreachable!()
    };
    *target = value;
    prefix.push(declaration);
    prefix
}

fn identity_handoff(edges: &OwnerEdges, live: &BTreeSet<String>) -> bool {
    edges.cleanup.is_empty()
        && edges.transfers.len() == live.len()
        && edges.transfers.iter().collect::<BTreeSet<_>>().len() == live.len()
        && edges.transfers.iter().all(|name| live.contains(name))
}

fn fresh_left(
    template: &NirFunction,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    context: &mut OutlineContext,
) -> String {
    let mut names = live.clone();
    names.extend(scope.iter().map(|p| p.name.clone()));
    let mut bodies = vec![template.body.as_slice()];
    while let Some(body) = bodies.pop() {
        for statement in body {
            match statement {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    names.insert(name.clone());
                }
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    bodies.push(then_body);
                    bodies.push(else_body);
                }
                NirStmt::While { body, .. } => bodies.push(body),
                _ => {}
            }
        }
    }
    let mut name = "__nuis_resource_predicate_left".to_owned();
    while names.contains(&name) || !context.reserved.insert(name.clone()) {
        name.push('_');
    }
    name
}

fn negated(value: &NirExpr) -> Option<&NirExpr> {
    match value {
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs,
            rhs,
        } if matches!(rhs.as_ref(), NirExpr::Bool(false)) => Some(lhs),
        _ => None,
    }
}

#[cfg(test)]
fn predicate<'a>(
    value: &'a NirExpr,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
) -> Option<(&'a NirExpr, &'a NirExpr, bool, bool)> {
    admitted(value, scope, live, calls, Kind::Bool).then_some(())?;
    logical_parts(value).filter(|_| needs_guard(value))
}

fn admitted(
    value: &NirExpr,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
    expected: Kind,
) -> bool {
    bounded(value).is_some_and(|reads| {
        // Keep ordinary pure predicates on their existing scalar-helper route.
        (reads || arguments::contains_argument_logic(value))
            && predicate_kind(value, scope, live, calls) == Some(expected)
    })
}

fn bounded(value: &NirExpr) -> Option<bool> {
    bounded_tree(value, false)
}

fn bounded_tree(value: &NirExpr, owned_return: bool) -> Option<bool> {
    // Bound the original tree before recursive type checking or cloning it.
    let mut work = vec![(value, 0)];
    let mut visited = 0;
    let mut reads = false;
    let mut logical_edges = 0;
    while let Some((value, depth)) = work.pop() {
        visited += 1;
        if visited > 4096 || depth >= 64 {
            return None;
        }
        match value {
            NirExpr::Binary { op, lhs, rhs } => {
                if matches!(op, NirBinaryOp::And | NirBinaryOp::Or) {
                    logical_edges += 1;
                    if logical_edges > 32 {
                        return None;
                    }
                }
                work.push((rhs, depth + 1));
                work.push((lhs, depth + 1));
            }
            NirExpr::LoadAt { buffer, index } => {
                reads = true;
                work.push((buffer, depth + 1));
                work.push((index, depth + 1));
            }
            NirExpr::BytesLen(value) | NirExpr::BufferLen(value) => {
                reads = true;
                work.push((value, depth + 1));
            }
            NirExpr::Call { args, .. } => {
                if args.len() > 4096usize.saturating_sub(visited + work.len()) {
                    return None;
                }
                work.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            NirExpr::FieldAccess { base, .. } => work.push((base, depth + 1)),
            NirExpr::StructLiteral { fields, .. } if owned_return => {
                if fields.len() > 4096usize.saturating_sub(visited + work.len()) {
                    return None;
                }
                work.extend(fields.iter().map(|(_, value)| (value, depth + 1)));
            }
            NirExpr::Move(value) | NirExpr::CopyBufferOwned(value) if owned_return => {
                work.push((value, depth + 1));
            }
            NirExpr::Bool(_) | NirExpr::Int(_) | NirExpr::Var(_) => {}
            _ => return None,
        }
    }
    Some(reads)
}

fn logical_parts(value: &NirExpr) -> Option<(&NirExpr, &NirExpr, bool, bool)> {
    let mut root = value;
    let mut inverted = false;
    while let Some(inner) = negated(root) {
        root = inner;
        inverted = !inverted;
    }
    let NirExpr::Binary {
        op: op @ (NirBinaryOp::And | NirBinaryOp::Or),
        lhs,
        rhs,
    } = root
    else {
        return None;
    };
    Some((lhs, rhs, *op == NirBinaryOp::Or, inverted))
}

// Only used after the complete source tree has passed the work/depth bound.
fn needs_guard(value: &NirExpr) -> bool {
    if let Some(inner) = negated(value) {
        return needs_guard(inner);
    }
    let NirExpr::Binary {
        op: NirBinaryOp::And | NirBinaryOp::Or,
        lhs,
        rhs,
    } = value
    else {
        return arguments::contains_logic(value);
    };
    let mut pending = vec![rhs.as_ref()];
    while let Some(value) = pending.pop() {
        match value {
            NirExpr::LoadAt { .. }
            | NirExpr::BytesLen(_)
            | NirExpr::BufferLen(_)
            | NirExpr::Call { .. }
            | NirExpr::Binary {
                op: NirBinaryOp::Div | NirBinaryOp::Rem,
                ..
            } => return true,
            NirExpr::Binary { lhs, rhs, .. } => {
                pending.push(lhs);
                pending.push(rhs);
            }
            NirExpr::FieldAccess { base, .. } => pending.push(base),
            _ => {}
        }
    }
    needs_guard(lhs)
}

fn predicate_kind(
    value: &NirExpr,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
) -> Option<Kind> {
    if let Some(inner) = negated(value) {
        return (predicate_kind(inner, scope, live, calls)? == Kind::Bool).then_some(Kind::Bool);
    }
    if let NirExpr::Binary {
        op: NirBinaryOp::And | NirBinaryOp::Or,
        lhs,
        rhs,
    } = value
    {
        return (predicate_kind(lhs, scope, live, calls)? == Kind::Bool
            && predicate_kind(rhs, scope, live, calls)? == Kind::Bool)
            .then_some(Kind::Bool);
    }
    leaf_kind(value, scope, live, calls)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bool,
    I64,
}

fn leaf_kind(
    value: &NirExpr,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
) -> Option<Kind> {
    match value {
        NirExpr::Bool(_) => Some(Kind::Bool),
        NirExpr::Int(_) => Some(Kind::I64),
        NirExpr::Var(name) => {
            let param = scope.iter().find(|p| p.name == *name)?;
            capture_scalar(&param.ty).then_some(if param.ty.name == "bool" {
                Kind::Bool
            } else {
                Kind::I64
            })
        }
        NirExpr::LoadAt { buffer, index } => (borrowed_buffer(buffer, scope)
            && leaf_kind(index, scope, live, calls)? == Kind::I64)
            .then_some(Kind::I64),
        NirExpr::BufferLen(value) => borrowed_buffer(value, scope).then_some(Kind::I64),
        NirExpr::BytesLen(value) => {
            let NirExpr::Var(name) = value.as_ref() else {
                return None;
            };
            live.contains(name).then_some(Kind::I64)
        }
        NirExpr::Binary { op, lhs, rhs } => {
            // Eager wrappers can contain lazy predicates. Prove both complete
            // operands before the ordered extractor stages either of them.
            let left = predicate_kind(lhs, scope, live, calls)?;
            if left != predicate_kind(rhs, scope, live, calls)? {
                return None;
            }
            match op {
                NirBinaryOp::Eq | NirBinaryOp::Ne => Some(Kind::Bool),
                NirBinaryOp::Lt | NirBinaryOp::Le | NirBinaryOp::Gt | NirBinaryOp::Ge
                    if left == Kind::I64 =>
                {
                    Some(Kind::Bool)
                }
                NirBinaryOp::Add
                | NirBinaryOp::Sub
                | NirBinaryOp::Mul
                | NirBinaryOp::Div
                | NirBinaryOp::Rem
                    if left == Kind::I64 =>
                {
                    Some(Kind::I64)
                }
                NirBinaryOp::Xor if left == Kind::Bool => Some(Kind::Bool),
                _ => None,
            }
        }
        NirExpr::Call { .. } | NirExpr::FieldAccess { .. } => {
            let ty = helper_value_type(value, scope, live, calls)?;
            capture_scalar(&ty).then_some(if ty.name == "bool" {
                Kind::Bool
            } else {
                Kind::I64
            })
        }
        _ => None,
    }
}

fn borrowed_buffer(value: &NirExpr, scope: &[NirParam]) -> bool {
    let NirExpr::Var(name) = value else {
        return false;
    };
    scope.iter().find(|p| p.name == *name).is_some_and(|p| {
        p.ty.name == "Buffer" && p.ty.is_ref && !p.ty.is_optional && p.ty.generic_args.is_empty()
    })
}

fn helper_value_type(
    value: &NirExpr,
    scope: &[NirParam],
    live: &BTreeSet<String>,
    calls: &PureValueCalls,
) -> Option<NirTypeRef> {
    match value {
        NirExpr::Call { callee, args } => {
            // Arguments are evaluated inside the selected owned region. Only
            // their scalar results cross the pure helper's call boundary.
            let args = args
                .iter()
                .map(|arg| {
                    let name = match predicate_kind(arg, scope, live, calls)? {
                        Kind::Bool => "bool",
                        Kind::I64 => "i64",
                    };
                    Some(NirTypeRef {
                        name: name.into(),
                        is_ref: false,
                        is_optional: false,
                        generic_args: vec![],
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            calls.call_type(callee, &args)
        }
        NirExpr::FieldAccess { base, field } => {
            calls.field_type(&helper_value_type(base, scope, live, calls)?, field)
        }
        _ => None,
    }
}
