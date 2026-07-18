use super::*;

pub(in crate::lowering) fn unsupported_loop_control_stmt_message(keyword: &str) -> String {
    format!(
        "`{keyword}` is currently lowered only as terminal loop control inside recognized `while` flow shapes (for example guard, flow, or post-flow loop bodies); bare `{keyword}` here has no structured loop lowering target yet"
    )
}

pub(in crate::lowering) fn unsupported_async_while_message() -> String {
    "async/task-driven `while` lowering currently recognizes only structured async loop shapes such as `await` step + chained carries, flow control, or post-flow control; general async backedge execution with task primitives inside arbitrary loop conditions/bodies is not lowered yet"
        .to_owned()
}

pub(in crate::lowering) fn unsupported_sync_while_message() -> String {
    "structured `while` lowering currently recognizes guard, counted, chained-carry, flow, and post-flow loop shapes; general iterative backedge execution with arbitrary synchronous loop bodies is not lowered yet"
        .to_owned()
}

pub(in crate::lowering) fn lower_function_body(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    allow_implicit_return: bool,
) -> Result<Option<String>, String> {
    let saved_effect_anchor = state.last_effect_anchor.take();
    if function
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::If { else_body, .. } if else_body.is_empty()))
    {
        if let Some(returned) =
            super::if_lowering::lower_guard_return_chain(&function.body, state, bindings)?
        {
            state.last_effect_anchor = saved_effect_anchor;
            return Ok(Some(returned));
        }
    }
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

pub(in crate::lowering) fn lower_if_stmt(
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
    let lowered =
        lower_if_pair(condition_name, then_body, else_body, state, bindings).map_err(|error| {
            let function_name = state
                .call_stack
                .last()
                .cloned()
                .unwrap_or_else(|| "<top-level>".to_owned());
            format!("in function `{function_name}` while lowering `if`: {error}")
        })?;
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

pub(in crate::lowering) fn lower_while_stmt(
    condition: &NirExpr,
    body: &[NirStmt],
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
    _const_bindings: &mut BTreeMap<String, NirExpr>,
) -> Result<Option<String>, String> {
    if super::scoped_loop_lowering::lower_scoped_call_while(condition, body, state, bindings)? {
        return Ok(None);
    }

    if super::owned_loop_lowering::lower_owned_bytes_while(condition, body, state, bindings)? {
        return Ok(None);
    }

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

pub(in crate::lowering) fn stmts_contain_async_loop_primitive(stmts: &[NirStmt]) -> bool {
    stmts.iter().any(stmt_contains_async_loop_primitive)
}

pub(in crate::lowering) fn stmt_contains_async_loop_primitive(stmt: &NirStmt) -> bool {
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

pub(in crate::lowering) fn expr_contains_async_loop_primitive(expr: &NirExpr) -> bool {
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
        | NirExpr::CpuTaskFailed(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuMutexNew(_)
        | NirExpr::CpuMutexLock(_)
        | NirExpr::CpuMutexUnlock(_)
        | NirExpr::CpuMutexValue(_)
        | NirExpr::CpuTimeout { .. }
        | NirExpr::CpuReadyAfter { .. } => true,
        NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CopyBufferOwned(inner)
        | NirExpr::BytesLen(inner)
        | NirExpr::DropBytes(inner)
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
