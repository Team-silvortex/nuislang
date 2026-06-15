use super::*;

pub(super) fn chain_statement_effect(state: &mut LoweringState<'_>, anchor: &str) {
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
        | NirExpr::CastI32ToI64(value)
        | NirExpr::CastI64ToBool(value)
        | NirExpr::CastBoolToI64(value)
        | NirExpr::CastI64ToF32(value)
        | NirExpr::CastF32ToI64(value)
        | NirExpr::CastI64ToF64(value)
        | NirExpr::CastF64ToI64(value)
        | NirExpr::IsNull(value)
        | NirExpr::FieldAccess { base: value, .. } => expr_requires_statement_anchor(value),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_requires_statement_anchor(lhs) || expr_requires_statement_anchor(rhs)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_requires_statement_anchor(value)),
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_) => false,
        _ => false,
    }
}

fn chain_nonpure_expr_stmt(expr: &NirExpr, lowered: &str, state: &mut LoweringState<'_>) {
    if expr_requires_statement_anchor(expr) {
        chain_statement_effect(state, lowered);
    }
}

fn refresh_const_binding(
    const_bindings: &mut BTreeMap<String, NirExpr>,
    name: &str,
    value: &NirExpr,
) {
    if eval_const_i64_with_env(value, const_bindings, &mut BTreeSet::new()).is_some() {
        const_bindings.insert(name.to_owned(), value.clone());
    } else {
        const_bindings.remove(name);
    }
}

fn eval_const_i64_with_env(
    expr: &NirExpr,
    const_bindings: &BTreeMap<String, NirExpr>,
    visited: &mut BTreeSet<String>,
) -> Option<i64> {
    match expr {
        NirExpr::Int(value) => Some(*value),
        NirExpr::Bool(value) => Some(i64::from(*value)),
        NirExpr::Var(name) => {
            if !visited.insert(name.clone()) {
                return None;
            }
            let resolved = const_bindings
                .get(name)
                .and_then(|value| eval_const_i64_with_env(value, const_bindings, visited));
            visited.remove(name);
            resolved
        }
        NirExpr::CastI64ToI32(value)
        | NirExpr::CastI32ToI64(value)
        | NirExpr::CastBoolToI64(value)
        | NirExpr::CastF32ToI64(value)
        | NirExpr::CastF64ToI64(value) => {
            eval_const_i64_with_env(value, const_bindings, visited)
        }
        NirExpr::CastI64ToBool(value) => Some(i64::from(
            eval_const_i64_with_env(value, const_bindings, visited)? != 0,
        )),
        NirExpr::Binary { op, lhs, rhs } => {
            let lhs = eval_const_i64_with_env(lhs, const_bindings, visited)?;
            let rhs = eval_const_i64_with_env(rhs, const_bindings, visited)?;
            match op {
                NirBinaryOp::And => Some(i64::from(lhs != 0 && rhs != 0)),
                NirBinaryOp::Or => Some(i64::from(lhs != 0 || rhs != 0)),
                NirBinaryOp::Add => Some(lhs + rhs),
                NirBinaryOp::Sub => Some(lhs - rhs),
                NirBinaryOp::Mul => Some(lhs * rhs),
                NirBinaryOp::Div => (rhs != 0).then_some(lhs / rhs),
                NirBinaryOp::Rem => (rhs != 0).then_some(lhs % rhs),
                NirBinaryOp::Eq => Some(i64::from(lhs == rhs)),
                NirBinaryOp::Ne => Some(i64::from(lhs != rhs)),
                NirBinaryOp::Lt => Some(i64::from(lhs < rhs)),
                NirBinaryOp::Le => Some(i64::from(lhs <= rhs)),
                NirBinaryOp::Gt => Some(i64::from(lhs > rhs)),
                NirBinaryOp::Ge => Some(i64::from(lhs >= rhs)),
            }
        }
        _ => None,
    }
}

fn eval_const_bool_with_env(
    expr: &NirExpr,
    const_bindings: &BTreeMap<String, NirExpr>,
) -> Option<bool> {
    match expr {
        NirExpr::Bool(value) => Some(*value),
        _ => Some(eval_const_i64_with_env(expr, const_bindings, &mut BTreeSet::new())? != 0),
    }
}

pub(super) fn lower_linear_stmts(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let mut last_bound_name = None;
    for stmt in stmts {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                last_bound_name = Some(name.clone());
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                last_bound_name = Some(name.clone());
            }
            NirStmt::Expr(expr) => {
                let lowered = lower_expr(expr, state, bindings)?;
                chain_nonpure_expr_stmt(expr, &lowered, state);
                last_bound_name = None;
            }
            _ => {
                return Err(
                    "minimal nuisc lowering currently only supports shared branch context made of straight-line `let`, `const`, or expression statements"
                        .to_owned(),
                )
            }
        }
    }
    Ok(last_bound_name)
}

fn lower_inline_stmts(
    stmts: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    const_bindings: &mut BTreeMap<String, NirExpr>,
) -> Result<Option<String>, String> {
    for stmt in stmts {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                refresh_const_binding(const_bindings, name, value);
            }
            NirStmt::Const { name, value, .. } => {
                let lowered = lower_expr(value, state, bindings)?;
                chain_nonpure_expr_stmt(value, &lowered, state);
                bindings.insert(name.clone(), lowered);
                refresh_const_binding(const_bindings, name, value);
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
                    lower_if_stmt(
                        condition,
                        then_body,
                        else_body,
                        state,
                        bindings,
                        const_bindings,
                    )?
                {
                    return Ok(Some(returned));
                }
            }
            NirStmt::While { condition, body } => {
                if let Some(returned) =
                    lower_while_stmt(condition, body, state, bindings, const_bindings)?
                {
                    return Ok(Some(returned));
                }
            }
            NirStmt::Break => return Err(unsupported_loop_control_stmt_message("break")),
            NirStmt::Continue => return Err(unsupported_loop_control_stmt_message("continue")),
            NirStmt::Expr(expr) => {
                let lowered = lower_expr(expr, state, bindings)?;
                chain_nonpure_expr_stmt(expr, &lowered, state);
            }
            NirStmt::Return(value) => {
                let returned = match value {
                    Some(value) => Some(lower_expr(value, state, bindings)?),
                    None => None,
                };
                return Ok(returned);
            }
        }
    }

    Ok(None)
}

fn unsupported_loop_control_stmt_message(keyword: &str) -> String {
    format!(
        "`{keyword}` is currently lowered only as terminal loop control inside recognized `while` flow shapes (for example guard, flow, or post-flow loop bodies); bare `{keyword}` here has no structured loop lowering target yet"
    )
}

fn unsupported_async_while_message() -> String {
    "async/task-driven `while` lowering currently recognizes only structured async loop shapes such as `await` step + chained carries, flow control, or post-flow control; general async backedge execution with task primitives inside arbitrary loop conditions/bodies is not lowered yet"
        .to_owned()
}

fn unsupported_sync_while_message() -> String {
    "structured `while` lowering currently recognizes guard, counted, chained-carry, flow, and post-flow loop shapes; general iterative backedge execution with arbitrary synchronous loop bodies is not lowered yet"
        .to_owned()
}

pub(super) fn lower_function_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    allow_implicit_return: bool,
) -> Result<Option<String>, String> {
    let saved_effect_anchor = state.last_effect_anchor.take();
    let mut const_bindings = BTreeMap::new();
    if let Some(returned) =
        lower_inline_stmts(&function.body, state, bindings, &mut const_bindings)?
    {
        state.last_effect_anchor = saved_effect_anchor.clone();
        return Ok(Some(returned));
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
    const_bindings: &mut BTreeMap<String, NirExpr>,
) -> Result<Option<String>, String> {
    let branch_has_conditional_effect =
        super::if_lowering::stmts_contain_conditional_effect_primitive(then_body)
            || super::if_lowering::stmts_contain_conditional_effect_primitive(else_body);
    if branch_has_conditional_effect {
        if let Some(value) = eval_const_bool_with_env(condition, const_bindings) {
            let active_body = if value { then_body } else { else_body };
            let mut branch_const_bindings = const_bindings.clone();
            return lower_inline_stmts(active_body, state, bindings, &mut branch_const_bindings);
        }
    }
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
    _const_bindings: &mut BTreeMap<String, NirExpr>,
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

    if let Some(diagnostic) = diagnose_unsupported_prepared_while_carry(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
        &state.pure_helper_blocks,
    ) {
        return Err(diagnostic);
    }

    if let Some(diagnostic) = super::loop_preparation::diagnose_unstructured_while_shape(
        condition,
        body,
        &state.pure_helpers,
        &state.inlineable_pure_helpers,
    ) {
        return Err(diagnostic);
    }

    if expr_contains_async_loop_primitive(condition) || stmts_contain_async_loop_primitive(body) {
        return Err(unsupported_async_while_message());
    }

    Err(unsupported_sync_while_message())
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
        | NirExpr::CpuThreadSpawn { .. }
        | NirExpr::CpuJoin(_)
        | NirExpr::CpuThreadJoin(_)
        | NirExpr::CpuCancel(_)
        | NirExpr::CpuJoinResult(_)
        | NirExpr::CpuThreadJoinResult(_)
        | NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuMutexNew(_)
        | NirExpr::CpuMutexLock(_)
        | NirExpr::CpuMutexUnlock(_)
        | NirExpr::CpuMutexValue(_)
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
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner) => expr_contains_async_loop_primitive(inner),
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
