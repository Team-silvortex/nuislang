use std::collections::BTreeSet;

use nuis_semantics::model::{nir_expr_effect_class, NirExpr, NirExprEffectClass, NirStmt};

pub(super) fn prune_dead_scalar_bindings(stmts: &mut Vec<NirStmt>) -> bool {
    prune_dead_scalar_bindings_with_live_after(stmts, &BTreeSet::new())
}

fn prune_dead_scalar_bindings_with_live_after(
    stmts: &mut Vec<NirStmt>,
    initial_live_after: &BTreeSet<String>,
) -> bool {
    let mut changed = false;
    let mut live_after = initial_live_after.clone();
    let mut kept = Vec::with_capacity(stmts.len());

    for stmt in stmts.drain(..).rev() {
        let (maybe_stmt, live_before, stmt_changed) = prune_stmt(stmt, &live_after);
        changed |= stmt_changed;
        if let Some(stmt) = maybe_stmt {
            kept.push(stmt);
        }
        live_after = live_before;
    }

    kept.reverse();
    *stmts = kept;
    changed
}

fn prune_stmt(
    stmt: NirStmt,
    live_after: &BTreeSet<String>,
) -> (Option<NirStmt>, BTreeSet<String>, bool) {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            let mut live_before = live_after.clone();
            if !live_after.contains(&name) && expr_is_dead_binding_safe(&value) {
                collect_used_vars_expr(&value, &mut live_before);
                return (None, live_before, true);
            }
            live_before.remove(&name);
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Let { name, ty, value }), live_before, false)
        }
        NirStmt::Const { name, ty, value } => {
            let mut live_before = live_after.clone();
            if !live_after.contains(&name) && expr_is_dead_binding_safe(&value) {
                collect_used_vars_expr(&value, &mut live_before);
                return (None, live_before, true);
            }
            live_before.remove(&name);
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Const { name, ty, value }), live_before, false)
        }
        NirStmt::Print(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Print(value)), live_before, false)
        }
        NirStmt::Await(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Await(value)), live_before, false)
        }
        NirStmt::Expr(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Expr(value)), live_before, false)
        }
        NirStmt::Return(value) => {
            let mut live_before = live_after.clone();
            if let Some(value) = &value {
                collect_used_vars_expr(value, &mut live_before);
            }
            (Some(NirStmt::Return(value)), live_before, false)
        }
        NirStmt::If {
            condition,
            mut then_body,
            mut else_body,
        } => {
            let mut changed = false;
            changed |= prune_dead_scalar_bindings_with_live_after(&mut then_body, live_after);
            changed |= prune_dead_scalar_bindings_with_live_after(&mut else_body, live_after);

            let then_live = live_before_block(&then_body, live_after);
            let else_live = live_before_block(&else_body, live_after);
            let mut live_before = live_after.clone();
            live_before.extend(then_live);
            live_before.extend(else_live);
            collect_used_vars_expr(&condition, &mut live_before);

            (
                Some(NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                }),
                live_before,
                changed,
            )
        }
        NirStmt::While { condition, body } => {
            let mut live_before = live_after.clone();
            live_before.extend(live_before_block(&body, live_after));
            collect_used_vars_expr(&condition, &mut live_before);
            (Some(NirStmt::While { condition, body }), live_before, false)
        }
        NirStmt::Break => (Some(NirStmt::Break), live_after.clone(), false),
        NirStmt::Continue => (Some(NirStmt::Continue), live_after.clone(), false),
    }
}

fn live_before_block(stmts: &[NirStmt], live_after: &BTreeSet<String>) -> BTreeSet<String> {
    let mut live = live_after.clone();
    for stmt in stmts.iter().rev() {
        live = live_before_stmt(stmt, &live);
    }
    live
}

fn live_before_stmt(stmt: &NirStmt, live_after: &BTreeSet<String>) -> BTreeSet<String> {
    match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            let mut live_before = live_after.clone();
            live_before.remove(name);
            collect_used_vars_expr(value, &mut live_before);
            live_before
        }
        NirStmt::Print(value) | NirStmt::Await(value) | NirStmt::Expr(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(value, &mut live_before);
            live_before
        }
        NirStmt::Return(value) => {
            let mut live_before = live_after.clone();
            if let Some(value) = value {
                collect_used_vars_expr(value, &mut live_before);
            }
            live_before
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let then_live = live_before_block(then_body, live_after);
            let else_live = live_before_block(else_body, live_after);
            let mut live_before = live_after.clone();
            live_before.extend(then_live);
            live_before.extend(else_live);
            collect_used_vars_expr(condition, &mut live_before);
            live_before
        }
        NirStmt::While { condition, body } => {
            let mut live_before = live_after.clone();
            live_before.extend(live_before_block(body, live_after));
            collect_used_vars_expr(condition, &mut live_before);
            live_before
        }
        NirStmt::Break | NirStmt::Continue => live_after.clone(),
    }
}

fn expr_is_dead_binding_safe(expr: &NirExpr) -> bool {
    if nir_expr_effect_class(expr) != NirExprEffectClass::Pure {
        return false;
    }
    match expr {
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_is_dead_binding_safe(lhs) && expr_is_dead_binding_safe(rhs)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| expr_is_dead_binding_safe(value)),
        NirExpr::FieldAccess { base, .. }
        | NirExpr::VariantIs { base, .. }
        | NirExpr::VariantFieldAccess { base, .. } => expr_is_dead_binding_safe(base),
        _ => true,
    }
}

fn collect_used_vars_expr(expr: &NirExpr, out: &mut BTreeSet<String>) {
    match expr {
        NirExpr::Var(name) => {
            out.insert(name.clone());
        }
        NirExpr::KernelTensor { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. } => {}
        NirExpr::SelectOwnedPointer {
            condition,
            then_owner,
            else_owner,
            ..
        } => {
            collect_used_vars_expr(condition, out);
            collect_used_vars_expr(then_owner, out);
            collect_used_vars_expr(else_owner, out);
        }
        NirExpr::DataProviderRequestIngress {
            request_handle,
            descriptor_table_handle,
            descriptor_count,
            provider_key,
            capability_hash,
            capsule_token,
            input_role_count,
            output_role_count,
        } => {
            for value in [
                request_handle,
                descriptor_table_handle,
                descriptor_count,
                provider_key,
                capability_hash,
            ] {
                collect_used_vars_expr(value, out);
            }
            for value in [capsule_token, input_role_count, output_role_count]
                .into_iter()
                .flatten()
            {
                collect_used_vars_expr(value, out);
            }
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CopyBufferOwned(inner)
        | NirExpr::BytesLen(inner)
        | NirExpr::DropBytes(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::VariantIs { base: inner, .. }
        | NirExpr::VariantFieldAccess { base: inner, .. }
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskFailed(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::KernelArgmaxAxis { input: inner, .. }
        | NirExpr::KernelArgminAxis { input: inner, .. }
        | NirExpr::KernelReduceMaxAxis { input: inner, .. }
        | NirExpr::KernelReduceMeanAxis { input: inner, .. }
        | NirExpr::KernelReduceSumAxis { input: inner, .. }
        | NirExpr::KernelSort(inner)
        | NirExpr::KernelSortAxis { input: inner, .. }
        | NirExpr::KernelTopkAxis { input: inner, .. }
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkResult { value: inner, .. }
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => collect_used_vars_expr(inner, out),
        NirExpr::KernelMatmul { lhs, rhs } => {
            collect_used_vars_expr(lhs, out);
            collect_used_vars_expr(rhs, out);
        }
        NirExpr::KernelElementAt { input, row, col } => {
            collect_used_vars_expr(input, out);
            collect_used_vars_expr(row, out);
            collect_used_vars_expr(col, out);
        }
        NirExpr::KernelReshape { input, .. } => {
            collect_used_vars_expr(input, out);
        }
        NirExpr::KernelBroadcast { input, .. } => {
            collect_used_vars_expr(input, out);
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            collect_used_vars_expr(input, out);
            if let Some(scalar) = scalar {
                collect_used_vars_expr(scalar, out);
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            collect_used_vars_expr(input, out);
            if let Some(scalar) = scalar {
                collect_used_vars_expr(scalar, out);
            }
        }
        NirExpr::KernelTopk { input, .. } => {
            collect_used_vars_expr(input, out);
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            collect_used_vars_expr(lhs, out);
            collect_used_vars_expr(rhs, out);
        }
        NirExpr::KernelAddBias { input, bias } => {
            collect_used_vars_expr(input, out);
            collect_used_vars_expr(bias, out);
        }
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            ..
        } => {
            collect_used_vars_expr(texture, out);
            collect_used_vars_expr(sampler, out);
            collect_used_vars_expr(x, out);
            collect_used_vars_expr(y, out);
        }
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            ..
        } => {
            collect_used_vars_expr(texture, out);
            collect_used_vars_expr(sampler, out);
            collect_used_vars_expr(uv, out);
        }
        NirExpr::ShaderBinding { value, .. } => {
            collect_used_vars_expr(value, out);
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            collect_used_vars_expr(pipeline, out);
            for binding in bindings {
                collect_used_vars_expr(binding, out);
            }
        }
        NirExpr::AllocNode { value, next } => {
            collect_used_vars_expr(value, out);
            collect_used_vars_expr(next, out);
        }
        NirExpr::AllocBuffer { len, fill } => {
            collect_used_vars_expr(len, out);
            collect_used_vars_expr(fill, out);
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            collect_used_vars_expr(buffer, out);
            collect_used_vars_expr(index, out);
        }
        NirExpr::StoreValue { target, value } => {
            collect_used_vars_expr(target, out);
            collect_used_vars_expr(value, out);
        }
        NirExpr::StoreNext { target, next } => {
            collect_used_vars_expr(target, out);
            collect_used_vars_expr(next, out);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        }
        | NirExpr::DataWriteWindow {
            window: buffer,
            index,
            value,
        } => {
            collect_used_vars_expr(buffer, out);
            collect_used_vars_expr(index, out);
            collect_used_vars_expr(value, out);
        }
        NirExpr::DataResult { value, .. }
        | NirExpr::KernelResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::DataProfileSendUplink { input: value, .. }
        | NirExpr::DataProfileSendDownlink { input: value, .. }
        | NirExpr::ShaderProfileRender { packet: value, .. } => collect_used_vars_expr(value, out),
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            collect_used_vars_expr(input, out);
            collect_used_vars_expr(offset, out);
            collect_used_vars_expr(len, out);
        }
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuExternCallI32 { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                collect_used_vars_expr(arg, out);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            collect_used_vars_expr(task, out);
            collect_used_vars_expr(limit, out);
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            collect_used_vars_expr(task, out);
            collect_used_vars_expr(delay, out);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            collect_used_vars_expr(base, out);
            collect_used_vars_expr(delta, out);
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            collect_used_vars_expr(delta, out);
            collect_used_vars_expr(scale, out);
            collect_used_vars_expr(base, out);
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
            ..
        } => {
            collect_used_vars_expr(color, out);
            collect_used_vars_expr(speed, out);
            collect_used_vars_expr(radius, out);
            if let Some(accent) = accent {
                collect_used_vars_expr(accent, out);
            }
            if let Some(toggle_state) = toggle_state {
                collect_used_vars_expr(toggle_state, out);
            }
            if let Some(focus_index) = focus_index {
                collect_used_vars_expr(focus_index, out);
            }
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            collect_used_vars_expr(target, out);
            collect_used_vars_expr(pipeline, out);
            collect_used_vars_expr(viewport, out);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            collect_used_vars_expr(pass, out);
            collect_used_vars_expr(packet, out);
            collect_used_vars_expr(vertex_count, out);
            collect_used_vars_expr(instance_count, out);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            collect_used_vars_expr(receiver, out);
            for arg in args {
                collect_used_vars_expr(arg, out);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_used_vars_expr(value, out);
            }
        }
        NirExpr::FieldAccess { base, .. } => collect_used_vars_expr(base, out),
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_used_vars_expr(lhs, out);
            collect_used_vars_expr(rhs, out);
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Instantiate { .. }
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfileSliderColorSlotRef { .. }
        | NirExpr::ShaderProfileSliderSpeedSlotRef { .. }
        | NirExpr::ShaderProfileSliderRadiusSlotRef { .. }
        | NirExpr::ShaderProfileHeaderAccentSlotRef { .. }
        | NirExpr::ShaderProfileToggleLiveSlotRef { .. }
        | NirExpr::ShaderProfileFocusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::DataProfileBindCoreRef { .. }
        | NirExpr::DataProfileWindowOffsetRef { .. }
        | NirExpr::DataProfileUplinkLenRef { .. }
        | NirExpr::DataProfileDownlinkLenRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::Null => {}
    }
}
