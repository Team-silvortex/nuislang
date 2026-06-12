use super::*;

fn chain_statement_effect(state: &mut LoweringState<'_>, anchor: &str) {
    if let Some(previous) = state.last_effect_anchor.as_ref() {
        if previous != anchor
            && !state.yir.edges.iter().any(|edge| {
                edge.from == *previous && edge.to == anchor && matches!(edge.kind, EdgeKind::Effect)
            })
        {
            state.yir.edges.push(Edge {
                kind: EdgeKind::Effect,
                from: previous.clone(),
                to: anchor.to_owned(),
            });
        }
    }
    state.last_effect_anchor = Some(anchor.to_owned());
}

fn expr_requires_statement_anchor(expr: &NirExpr) -> bool {
    if nir_expr_effect_class(expr) != NirExprEffectClass::Pure {
        return true;
    }

    match expr {
        NirExpr::CastI64ToI32(value)
        | NirExpr::IsNull(value)
        | NirExpr::FieldAccess { base: value, .. } => expr_requires_statement_anchor(value),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_requires_statement_anchor(lhs) || expr_requires_statement_anchor(rhs)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_requires_statement_anchor(value)),
        NirExpr::Null | NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Var(_) => {
            false
        }
        _ => false,
    }
}

fn chain_nonpure_expr_stmt(expr: &NirExpr, lowered: &str, state: &mut LoweringState<'_>) {
    if expr_requires_statement_anchor(expr) {
        chain_statement_effect(state, lowered);
    }
}

pub(super) fn lower_function_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    allow_implicit_return: bool,
) -> Result<Option<String>, String> {
    let saved_effect_anchor = state.last_effect_anchor.take();
    for stmt in &function.body {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
            }
            NirStmt::Print(value) => {
                let lowered = lower_expr(value, state, bindings)?;
                let print_name = format!("print_{}", state.print_counter);
                state.print_counter += 1;
                state.yir.nodes.push(Node {
                    name: print_name.clone(),
                    resource: "cpu0".to_owned(),
                    op: Operation {
                        module: "cpu".to_owned(),
                        instruction: "print".to_owned(),
                        args: vec![lowered.clone()],
                    },
                });
                push_dep_edges(state, &lowered, &print_name);
                state.yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: lowered,
                    to: print_name.clone(),
                });
                chain_statement_effect(state, &print_name);
            }
            NirStmt::Await(value) => {
                let awaited = match value {
                    NirExpr::Call { callee, args } => {
                        lower_async_call_boundary(callee, args, state, bindings)?
                    }
                    _ => lower_expr(value, state, bindings)?,
                };
                let await_name = push_await_node(state, &awaited);
                state.yir.edges.push(Edge {
                    kind: EdgeKind::Effect,
                    from: awaited,
                    to: await_name.clone(),
                });
                chain_statement_effect(state, &await_name);
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                if let Some(returned) =
                    lower_if_stmt(condition, then_body, else_body, state, bindings)?
                {
                    state.last_effect_anchor = saved_effect_anchor.clone();
                    return Ok(Some(returned));
                }
            }
            NirStmt::While { condition, body } => {
                if let Some(returned) = lower_while_stmt(condition, body, state, bindings)? {
                    state.last_effect_anchor = saved_effect_anchor.clone();
                    return Ok(Some(returned));
                }
            }
            NirStmt::Break => {
                state.last_effect_anchor = saved_effect_anchor.clone();
                return Err(
                    "`break` parsed successfully, but loop execution lowering is not implemented yet"
                        .to_owned(),
                );
            }
            NirStmt::Continue => {
                state.last_effect_anchor = saved_effect_anchor.clone();
                return Err(
                    "`continue` parsed successfully, but loop execution lowering is not implemented yet"
                        .to_owned(),
                );
            }
            NirStmt::Expr(expr) => {
                let lowered = lower_expr(expr, state, bindings)?;
                chain_nonpure_expr_stmt(expr, &lowered, state);
            }
            NirStmt::Return(value) => {
                let returned = match value {
                    Some(value) => Some(lower_expr(value, state, bindings)?),
                    None => None,
                };
                state.last_effect_anchor = saved_effect_anchor.clone();
                return Ok(returned);
            }
        }
    }

    state.last_effect_anchor = saved_effect_anchor.clone();
    if allow_implicit_return {
        Ok(None)
    } else {
        state.last_effect_anchor = saved_effect_anchor;
        Err(format!(
            "function `{}` ended without `return` in expression-call lowering",
            function.name
        ))
    }
}

pub(super) fn lower_if_stmt(
    condition: &NirExpr,
    then_body: &[NirStmt],
    else_body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let condition_name = lower_expr(condition, state, bindings)?;
    let lowered = lower_if_pair(condition_name, then_body, else_body, state, bindings)?;
    match lowered {
        LoweredIfOutcome::Continued => Ok(None),
        LoweredIfOutcome::Bind { name, value } => {
            bindings.insert(name, value);
            Ok(None)
        }
        LoweredIfOutcome::Printed => Ok(None),
        LoweredIfOutcome::Returned(value) => Ok(Some(value)),
    }
}

pub(super) fn lower_while_stmt(
    condition: &NirExpr,
    body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    if let Some(prepared) = prepare_post_flow_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_post_flow_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_async_post_flow_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_async_post_flow_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_flow_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_flow_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_async_flow_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_async_flow_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_chained_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_chained_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_async_chained_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_async_chained_while(prepared, state, bindings)?;
        return Ok(None);
    }

    if let Some(prepared) = prepare_counted_while(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        lower_counted_while(prepared, state, bindings)?;
        return Ok(None);
    }

    let condition_name = lower_expr(condition, state, bindings)?;
    if let Some(prepared) = prepare_guarded_loop_body(body, &state.pure_helpers) {
        return lower_prepared_loop_body(condition_name, &prepared, state, bindings);
    }

    if expr_contains_async_loop_primitive(condition) || stmts_contain_async_loop_primitive(body) {
        return Err(
            "async/task-driven `while` loops are not supported yet in lowering; iterative backedge lowering for `await`, `spawn`, `join`, `timeout`, and related task primitives inside loop conditions/bodies is still not implemented"
                .to_owned(),
        );
    }

    Err(
        "minimal nuisc lowering can currently execute only guard-style `while` loops or simple counted `while` loops like `let i = 0; while i < limit { let i = i + 1; }`; general iterative loop/backedge lowering is still not implemented"
            .to_owned(),
    )
}

fn stmts_contain_async_loop_primitive(stmts: &[NirStmt]) -> bool {
    stmts.iter().any(stmt_contains_async_loop_primitive)
}

fn stmt_contains_async_loop_primitive(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value)
        | NirStmt::Await(value) => expr_contains_async_loop_primitive(value),
        NirStmt::Return(Some(value)) => expr_contains_async_loop_primitive(value),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_async_loop_primitive(condition)
                || stmts_contain_async_loop_primitive(then_body)
                || stmts_contain_async_loop_primitive(else_body)
        }
        NirStmt::While { condition, body } => {
            expr_contains_async_loop_primitive(condition)
                || stmts_contain_async_loop_primitive(body)
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

fn expr_contains_async_loop_primitive(expr: &NirExpr) -> bool {
    match expr {
        NirExpr::Await(_)
        | NirExpr::CpuSpawn { .. }
        | NirExpr::CpuJoin(_)
        | NirExpr::CpuCancel(_)
        | NirExpr::CpuJoinResult(_)
        | NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuTimeout { .. } => true,
        NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::CastI64ToI32(inner) => expr_contains_async_loop_primitive(inner),
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => expr_contains_async_loop_primitive(value),
        NirExpr::AllocNode { value, next } => {
            expr_contains_async_loop_primitive(value) || expr_contains_async_loop_primitive(next)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_contains_async_loop_primitive(len) || expr_contains_async_loop_primitive(fill)
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            expr_contains_async_loop_primitive(buffer) || expr_contains_async_loop_primitive(index)
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        }
        | NirExpr::StoreAt {
            buffer: window,
            index,
            value,
        } => {
            expr_contains_async_loop_primitive(window)
                || expr_contains_async_loop_primitive(index)
                || expr_contains_async_loop_primitive(value)
        }
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => {
            expr_contains_async_loop_primitive(target) || expr_contains_async_loop_primitive(value)
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            expr_contains_async_loop_primitive(input)
                || expr_contains_async_loop_primitive(offset)
                || expr_contains_async_loop_primitive(len)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::FieldAccess { base: input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. } => {
            expr_contains_async_loop_primitive(input)
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            expr_contains_async_loop_primitive(base) || expr_contains_async_loop_primitive(delta)
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            expr_contains_async_loop_primitive(delta)
                || expr_contains_async_loop_primitive(scale)
                || expr_contains_async_loop_primitive(base)
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            expr_contains_async_loop_primitive(color)
                || expr_contains_async_loop_primitive(speed)
                || expr_contains_async_loop_primitive(radius)
        }
        NirExpr::Call { args, .. } => args.iter().any(expr_contains_async_loop_primitive),
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_contains_async_loop_primitive(receiver)
                || args.iter().any(expr_contains_async_loop_primitive)
        }
        NirExpr::CpuExternCall { args, .. } => args.iter().any(expr_contains_async_loop_primitive),
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_async_loop_primitive(value)),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_async_loop_primitive(lhs) || expr_contains_async_loop_primitive(rhs)
        }
        NirExpr::Instantiate { .. }
        | NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_) => false,
        _ => false,
    }
}

pub(super) fn lower_call_expr(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if callee == "print" {
        return Err("`print(...)` is only valid as a statement".to_owned());
    }

    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;

    if state.direct_call_functions.contains(callee) {
        let lowered_args = args
            .iter()
            .map(|arg| lower_expr(arg, state, bindings))
            .collect::<Result<Vec<_>, _>>()?;
        return push_direct_call_node(function, &lowered_args, state);
    }

    if state.call_stack.iter().any(|active| active == callee) {
        return Err(format!(
            "recursive function call `{callee}` is not yet supported by minimal nuisc lowering"
        ));
    }

    if function.params.len() != args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let mut local_bindings = BTreeMap::new();
    for (param, arg) in function.params.iter().zip(args.iter()) {
        let lowered = lower_expr(arg, state, bindings)?;
        local_bindings.insert(param.name.clone(), lowered);
    }

    let caller_effect_anchor = state.last_effect_anchor.take();
    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();
    let callee_effect_anchor = state.last_effect_anchor.take();
    state.last_effect_anchor = callee_effect_anchor.or(caller_effect_anchor);

    returned.ok_or_else(|| format!("function `{callee}` did not return a value"))
}

pub(super) fn lower_async_call_boundary(
    callee: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let function = state
        .function_map
        .get(callee)
        .copied()
        .ok_or_else(|| format!("unknown function `{callee}`"))?;
    if !function.is_async {
        return lower_call_expr(callee, args, state, bindings);
    }
    if state.call_stack.iter().any(|active| active == callee) {
        return Err(format!(
            "recursive async function call `{callee}` is not yet supported by minimal nuisc lowering"
        ));
    }
    if function.params.len() != args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            function.params.len(),
            args.len()
        ));
    }

    let lowered_args = args
        .iter()
        .map(|arg| lower_expr(arg, state, bindings))
        .collect::<Result<Vec<_>, _>>()?;

    if state.async_helper_functions.contains(callee) {
        let call_name = next_name(state, "async_call");
        let mut op_args = vec![callee.to_owned()];
        op_args.extend(lowered_args.clone());
        state.yir.nodes.push(Node {
            name: call_name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "async_call".to_owned(),
                args: op_args,
            },
        });
        for arg in &lowered_args {
            push_dep_edges(state, arg, &call_name);
        }

        let returned = push_direct_call_node(function, &lowered_args, state)?;
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: call_name,
            to: returned.clone(),
        });
        return Ok(returned);
    }

    let mut local_bindings = BTreeMap::new();
    for (param, lowered) in function.params.iter().zip(lowered_args.iter()) {
        local_bindings.insert(param.name.clone(), lowered.clone());
    }

    let call_name = next_name(state, "async_call");
    let mut op_args = vec![callee.to_owned()];
    op_args.extend(lowered_args.clone());
    state.yir.nodes.push(Node {
        name: call_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "async_call".to_owned(),
            args: op_args,
        },
    });
    for arg in &lowered_args {
        push_dep_edges(state, arg, &call_name);
    }

    state.call_stack.push(callee.to_owned());
    let returned = lower_function_body(function, state, &mut local_bindings, false)?;
    state.call_stack.pop();
    let returned = returned.ok_or_else(|| format!("function `{callee}` did not return a value"))?;
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: call_name,
        to: returned.clone(),
    });
    Ok(returned)
}

pub(super) fn lower_unary_cpu_expr(
    instruction: &str,
    value: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    let lowered = lower_expr(value, state, bindings)?;
    let name = next_name(state, instruction);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![lowered.clone()],
        },
    });
    push_dep_edges(state, &lowered, &name);
    Ok(name)
}
