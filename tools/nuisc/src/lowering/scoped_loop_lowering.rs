use super::*;

enum ScopedLoopResult<'a> {
    None,
    OwnedBytes(&'a str),
    OwnedStruct {
        binding: &'a str,
        ty: &'a NirTypeRef,
        layout: String,
    },
}

pub(super) fn collect_scoped_loop_helper_functions(module: &NirModule) -> BTreeSet<String> {
    let aggregate_helpers = direct_calls::collect_aggregate_param_direct_call_functions(module);
    let eligible = module
        .functions
        .iter()
        .filter(|function| {
            direct_calls::supports_direct_call_signature(function)
                || aggregate_helpers.contains(&function.name)
        })
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut helpers = BTreeSet::new();
    for function in &module.functions {
        collect_from_stmts(&function.body, &eligible, &mut helpers);
    }
    helpers
}

fn collect_from_stmts(
    stmts: &[NirStmt],
    eligible: &BTreeSet<&str>,
    helpers: &mut BTreeSet<String>,
) {
    for stmt in stmts {
        match stmt {
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_from_stmts(then_body, eligible, helpers);
                collect_from_stmts(else_body, eligible, helpers);
            }
            NirStmt::While { body, .. } => {
                let callee = match body.first() {
                    Some(NirStmt::Expr(NirExpr::Call { callee, .. })) => Some(callee),
                    Some(NirStmt::Let {
                        value: NirExpr::Call { callee, .. },
                        ..
                    }) => Some(callee),
                    _ => None,
                };
                if let Some(callee) = callee.filter(|callee| eligible.contains(callee.as_str())) {
                    helpers.insert(callee.clone());
                }
                collect_from_stmts(body, eligible, helpers);
            }
            _ => {}
        }
    }
}

pub(super) fn lower_scoped_call_while(
    condition: &NirExpr,
    body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    const_bindings: &BTreeMap<String, NirExpr>,
) -> Result<bool, String> {
    let Some((action, counted_body)) = body.split_first() else {
        return Ok(false);
    };
    let (callee, args, result_binding) = match action {
        NirStmt::Expr(NirExpr::Call { callee, args }) => (callee, args, None),
        NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, args },
        } if (ty.name == "Bytes" && !ty.is_ref && ty.generic_args.is_empty())
            || state.struct_defs.contains_key(ty.name.as_str()) =>
        {
            (callee, args, Some((name.as_str(), ty)))
        }
        _ => return Ok(false),
    };
    if !state.direct_call_functions.contains(callee) {
        return Ok(false);
    }
    let Some(prepared) = prepare_counted_while(
        condition,
        counted_body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) else {
        return Ok(false);
    };
    let function = state
        .function_map
        .get(callee.as_str())
        .copied()
        .ok_or_else(|| format!("unknown scoped loop helper `{callee}`"))?;
    if function.params.len() != args.len() {
        return Err(format!(
            "scoped loop helper `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }
    let returns_owned_bytes = function
        .return_type
        .as_ref()
        .is_some_and(|ty| ty.name == "Bytes" && !ty.is_ref && ty.generic_args.is_empty());
    let owned_struct_layout = function_owned_struct_layout(function, state).filter(|_| {
        function
            .return_type
            .as_ref()
            .is_some_and(|ty| state.struct_defs.contains_key(ty.name.as_str()))
    });
    let result = match (result_binding, returns_owned_bytes, owned_struct_layout) {
        (None, false, None) => ScopedLoopResult::None,
        (Some((binding, ty)), true, None)
            if ty.name == "Bytes" && !ty.is_ref && ty.generic_args.is_empty() =>
        {
            ScopedLoopResult::OwnedBytes(binding)
        }
        (Some((binding, ty)), false, Some(layout))
            if function
                .return_type
                .as_ref()
                .is_some_and(|returned| returned.render() == ty.render()) =>
        {
            ScopedLoopResult::OwnedStruct {
                binding,
                ty,
                layout,
            }
        }
        (None, true, None) => {
            return Err(format!(
                "scoped loop helper `{callee}` with owned Bytes return requires `let <owner>: Bytes = {callee}(...)` rebinding"
            ));
        }
        (None, false, Some(_)) => {
            return Err(format!(
                "scoped loop helper `{callee}` with aggregate return requires same-name state rebinding"
            ));
        }
        _ => return Ok(false),
    };
    validate_loop_result_rebinding(&result, function, args, bindings, callee)?;

    let Some(initial) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "scoped-call loop expected an existing binding for `{}`",
            prepared.binding_name
        ));
    };
    let limit = lower_expr(&prepared.limit, state, bindings)?;
    let step = lower_expr(&prepared.step, state, bindings)?;
    let has_owned_move = function.params.iter().zip(args).any(|(param, arg)| {
        param.ty.name == "Bytes"
            && !param.ty.is_ref
            && matches!(arg, NirExpr::Move(source) if matches!(source.as_ref(), NirExpr::Var(_)))
    });
    if has_owned_move
        && !matches!(&result, ScopedLoopResult::OwnedBytes(_))
        && !counted_loop_runs_exactly_once(&prepared, const_bindings)
    {
        return Err(format!(
            "scoped loop helper `{callee}` can only move owned Bytes through a loop statically proven to execute exactly once"
        ));
    }
    let owned_result = match &result {
        ScopedLoopResult::None => None,
        ScopedLoopResult::OwnedBytes(_) => Some(next_name(state, "loop_owned_result")),
        ScopedLoopResult::OwnedStruct { .. } => Some(next_name(state, "loop_owned_struct_result")),
    };
    let mut action_args = vec![callee.clone()];
    if let Some(result) = &owned_result {
        action_args.push(result.clone());
    }
    if let ScopedLoopResult::OwnedStruct { layout, .. } = &result {
        action_args.push(layout.clone());
    }
    let mut carried_struct_leaf = 0usize;
    for (param, arg) in function.params.iter().zip(args) {
        if matches!(arg, NirExpr::Var(name) if name == &prepared.binding_name) {
            action_args.push("$current".to_owned());
        } else if state.struct_defs.contains_key(param.ty.name.as_str()) {
            let lowered = lower_expr(arg, state, bindings)?;
            let flattened = direct_calls::flatten_direct_call_argument(&param.ty, &lowered, state)?;
            let carried = matches!(
                (&result, arg),
                (
                    ScopedLoopResult::OwnedStruct { binding, ty, .. },
                    NirExpr::Var(name)
                ) if name == binding && param.ty.render() == ty.render()
            );
            for input in flattened {
                if carried {
                    action_args.push(yir_core::encode_loop_owned_struct_carry(
                        carried_struct_leaf,
                        &input,
                    ));
                    carried_struct_leaf += 1;
                } else {
                    action_args.push(input);
                }
            }
        } else if let NirExpr::CopyBufferOwned(source) = arg {
            action_args.push(format!(
                "copy_owned:{}",
                lower_expr(source, state, bindings)?
            ));
        } else if param.ty.name == "Bytes" && !param.ty.is_ref {
            if let NirExpr::Move(source) = arg {
                let NirExpr::Var(_) = source.as_ref() else {
                    return Err(format!(
                        "scoped loop helper `{callee}` requires move(Bytes) capture from a named owner"
                    ));
                };
                action_args.push(format!(
                    "move_owned:{}",
                    lower_expr(source, state, bindings)?
                ));
            } else {
                action_args.push(lower_expr(arg, state, bindings)?);
            }
        } else {
            action_args.push(lower_expr(arg, state, bindings)?);
        }
    }
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    let mut node_args = vec![
        initial,
        limit,
        step,
        render_loop_compare(prepared.compare).to_owned(),
        step_kind.to_owned(),
        "cpu".to_owned(),
        match &result {
            ScopedLoopResult::None => "scoped_call",
            ScopedLoopResult::OwnedBytes(_) => "scoped_call_owned_return",
            ScopedLoopResult::OwnedStruct { .. } => "scoped_call_owned_struct_return",
        }
        .to_owned(),
        action_args.len().to_string(),
    ];
    node_args.extend(action_args);
    let name = next_name(state, "loop_while_i64_scoped_call");
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "loop_while_i64_effect".to_owned(),
            args: node_args.clone(),
        },
    });
    for input in node_args {
        let dependency = yir_core::parse_loop_owned_struct_carry(&input)?
            .map(|(_, dependency)| dependency)
            .unwrap_or_else(|| {
                input
                    .strip_prefix("copy_owned:")
                    .or_else(|| input.strip_prefix("move_owned:"))
                    .unwrap_or(&input)
            });
        if state.yir.nodes.iter().any(|node| node.name == dependency) {
            push_dep_edges(state, dependency, &name);
        }
    }
    for (param, arg) in function.params.iter().zip(args) {
        if (param.ty.is_ref && param.ty.name == "Buffer")
            || matches!(arg, NirExpr::CopyBufferOwned(_) | NirExpr::Move(_))
        {
            let input = match arg {
                NirExpr::Var(name) => bindings.get(name),
                NirExpr::CopyBufferOwned(source) | NirExpr::Move(source) => match source.as_ref() {
                    NirExpr::Var(name) => bindings.get(name),
                    _ => None,
                },
                _ => None,
            };
            if let Some(input) = input {
                push_lifetime_edge(state, input, &name);
            }
        }
    }
    body_lowering::chain_statement_effect(state, &name);
    bindings.insert(prepared.binding_name, name.clone());
    if let Some(result_name) = owned_result {
        let (owner, instruction, args) = match &result {
            ScopedLoopResult::None => unreachable!("result projection requires an owner"),
            ScopedLoopResult::OwnedBytes(owner) => {
                (*owner, "loop_owned_result", vec![name.clone()])
            }
            ScopedLoopResult::OwnedStruct {
                binding, layout, ..
            } => (
                *binding,
                "loop_owned_struct_result",
                vec![name.clone(), layout.clone()],
            ),
        };
        state.yir.nodes.push(Node {
            name: result_name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: instruction.to_owned(),
                args,
            },
        });
        push_dep_edges(state, &name, &result_name);
        bindings.insert(owner.to_owned(), result_name);
    }
    Ok(true)
}

fn validate_loop_result_rebinding(
    result: &ScopedLoopResult<'_>,
    function: &NirFunction,
    args: &[NirExpr],
    bindings: &BTreeMap<String, String>,
    callee: &str,
) -> Result<(), String> {
    match result {
        ScopedLoopResult::None => Ok(()),
        ScopedLoopResult::OwnedBytes(owner) => {
            if bindings.contains_key(*owner)
                && args.iter().any(
                    |arg| matches!(arg, NirExpr::Move(source) if matches!(source.as_ref(), NirExpr::Var(name) if name == owner)),
                )
            {
                return Ok(());
            }
            Err(format!(
                "scoped owned Bytes return from `{callee}` must rebind the same named owner moved into the helper"
            ))
        }
        ScopedLoopResult::OwnedStruct { binding, ty, .. } => {
            let matching_inputs = function
                .params
                .iter()
                .zip(args)
                .filter(|(param, arg)| {
                    param.ty.render() == ty.render()
                        && matches!(arg, NirExpr::Var(name) if name == binding)
                })
                .count();
            if bindings.contains_key(*binding) && matching_inputs == 1 {
                return Ok(());
            }
            Err(format!(
                "scoped aggregate return from `{callee}` must rebind exactly one same-typed state argument named `{binding}`"
            ))
        }
    }
}

fn counted_loop_runs_exactly_once(
    prepared: &PreparedCountedWhile,
    const_bindings: &BTreeMap<String, NirExpr>,
) -> bool {
    let mut visited = BTreeSet::new();
    let Some(initial) = body_lowering::eval_const_i64_with_env(
        &NirExpr::Var(prepared.binding_name.clone()),
        const_bindings,
        &mut visited,
    ) else {
        return false;
    };
    visited.clear();
    let Some(limit) =
        body_lowering::eval_const_i64_with_env(&prepared.limit, const_bindings, &mut visited)
    else {
        return false;
    };
    visited.clear();
    let Some(step) =
        body_lowering::eval_const_i64_with_env(&prepared.step, const_bindings, &mut visited)
    else {
        return false;
    };
    if !compare_i64(initial, limit, prepared.compare) {
        return false;
    }
    let next = match prepared.step_kind {
        PreparedLoopStepKind::Add => initial.checked_add(step),
        PreparedLoopStepKind::Sub => initial.checked_sub(step),
    };
    next.is_some_and(|next| !compare_i64(next, limit, prepared.compare))
}

fn compare_i64(lhs: i64, rhs: i64, compare: PreparedLoopCompare) -> bool {
    match compare {
        PreparedLoopCompare::Eq => lhs == rhs,
        PreparedLoopCompare::Ne => lhs != rhs,
        PreparedLoopCompare::Lt => lhs < rhs,
        PreparedLoopCompare::Le => lhs <= rhs,
        PreparedLoopCompare::Gt => lhs > rhs,
        PreparedLoopCompare::Ge => lhs >= rhs,
    }
}
