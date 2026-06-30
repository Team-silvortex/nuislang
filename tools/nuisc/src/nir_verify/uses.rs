use std::collections::BTreeSet;

use nuis_semantics::model::NirExpr;

pub(super) fn verify_expr_uses(expr: &NirExpr, moved: &BTreeSet<String>) -> Result<(), String> {
    match expr {
        NirExpr::Var(_) | NirExpr::FieldAccess { .. } => {
            if let Some(name) = expr_resource_key(expr) {
                if moved.contains(&name) {
                    return Err(format!("nir verify: use of moved value `{}`", name));
                }
            }
        }
        NirExpr::Instantiate { .. } => {}
        NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::CastI64ToF32(inner) | NirExpr::CastF32ToI64(inner) => {
            verify_expr_uses(inner, moved)?
        }
        NirExpr::CastI64ToF64(inner) | NirExpr::CastF64ToI64(inner) => {
            verify_expr_uses(inner, moved)?
        }
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
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
        | NirExpr::KernelTensor { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. } => {}
        NirExpr::CpuPresentFrame(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
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
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::NetworkResult { value: inner, .. } => verify_expr_uses(inner, moved)?,
        NirExpr::KernelArgmax(inner) | NirExpr::KernelArgmin(inner) => {
            verify_expr_uses(inner, moved)?
        }
        NirExpr::KernelArgmaxAxis { input, .. } | NirExpr::KernelArgminAxis { input, .. } => {
            verify_expr_uses(input, moved)?
        }
        NirExpr::KernelReduceMaxAxis { input, .. }
        | NirExpr::KernelReduceMeanAxis { input, .. } => verify_expr_uses(input, moved)?,
        NirExpr::KernelReduceSumAxis { input, .. } => verify_expr_uses(input, moved)?,
        NirExpr::KernelSort(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::KernelSortAxis { input, .. } => verify_expr_uses(input, moved)?,
        NirExpr::KernelTopkAxis { input, .. } => verify_expr_uses(input, moved)?,
        NirExpr::KernelTopk { input, .. } => verify_expr_uses(input, moved)?,
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuExternCallI32 { args, .. } => {
            for arg in args {
                verify_expr_uses(arg, moved)?;
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            verify_expr_uses(task, moved)?;
            verify_expr_uses(limit, moved)?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            verify_expr_uses(target, moved)?;
            verify_expr_uses(pipeline, moved)?;
            verify_expr_uses(viewport, moved)?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            verify_expr_uses(packet, moved)?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            verify_expr_uses(base, moved)?;
            verify_expr_uses(delta, moved)?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            verify_expr_uses(delta, moved)?;
            verify_expr_uses(scale, moved)?;
            verify_expr_uses(base, moved)?;
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            verify_expr_uses(base, moved)?;
            verify_expr_uses(delta, moved)?;
        }
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            ..
        } => {
            verify_expr_uses(texture, moved)?;
            verify_expr_uses(sampler, moved)?;
            verify_expr_uses(x, moved)?;
            verify_expr_uses(y, moved)?;
        }
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            ..
        } => {
            verify_expr_uses(texture, moved)?;
            verify_expr_uses(sampler, moved)?;
            verify_expr_uses(uv, moved)?;
        }
        NirExpr::ShaderBinding { value, .. } => verify_expr_uses(value, moved)?,
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            verify_expr_uses(pipeline, moved)?;
            for binding in bindings {
                verify_expr_uses(binding, moved)?;
            }
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
            verify_expr_uses(color, moved)?;
            verify_expr_uses(speed, moved)?;
            verify_expr_uses(radius, moved)?;
            if let Some(accent) = accent {
                verify_expr_uses(accent, moved)?;
            }
            if let Some(toggle_state) = toggle_state {
                verify_expr_uses(toggle_state, moved)?;
            }
            if let Some(focus_index) = focus_index {
                verify_expr_uses(focus_index, moved)?;
            }
        }
        NirExpr::KernelMatmul { lhs, rhs } => {
            verify_expr_uses(lhs, moved)?;
            verify_expr_uses(rhs, moved)?;
        }
        NirExpr::KernelElementAt { input, row, col } => {
            verify_expr_uses(input, moved)?;
            verify_expr_uses(row, moved)?;
            verify_expr_uses(col, moved)?;
        }
        NirExpr::KernelReshape { input, .. } => {
            verify_expr_uses(input, moved)?;
        }
        NirExpr::KernelBroadcast { input, .. } => {
            verify_expr_uses(input, moved)?;
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            verify_expr_uses(input, moved)?;
            if let Some(scalar) = scalar {
                verify_expr_uses(scalar, moved)?;
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            verify_expr_uses(input, moved)?;
            if let Some(scalar) = scalar {
                verify_expr_uses(scalar, moved)?;
            }
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            verify_expr_uses(lhs, moved)?;
            verify_expr_uses(rhs, moved)?;
        }
        NirExpr::KernelAddBias { input, bias } => {
            verify_expr_uses(input, moved)?;
            verify_expr_uses(bias, moved)?;
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            verify_expr_uses(pass, moved)?;
            verify_expr_uses(packet, moved)?;
            if let NirExpr::ShaderDrawInstanced {
                vertex_count,
                instance_count,
                ..
            } = expr
            {
                verify_expr_uses(vertex_count, moved)?;
                verify_expr_uses(instance_count, moved)?;
            }
        }
        NirExpr::DataOutputPipe(inner) | NirExpr::DataInputPipe(inner) => {
            verify_expr_uses(inner, moved)?
        }
        NirExpr::DataResult { value: inner, .. }
        | NirExpr::ShaderResult { value: inner, .. }
        | NirExpr::KernelResult { value: inner, .. } => verify_expr_uses(inner, moved)?,
        NirExpr::DataFreezeWindow(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::DataReadWindow { window, index } => {
            verify_expr_uses(window, moved)?;
            verify_expr_uses(index, moved)?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            verify_expr_uses(window, moved)?;
            verify_expr_uses(index, moved)?;
            verify_expr_uses(value, moved)?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => verify_expr_uses(input, moved)?,
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            verify_expr_uses(input, moved)?;
            verify_expr_uses(offset, moved)?;
            verify_expr_uses(len, moved)?;
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::AllocNode { value, next } => {
            verify_expr_uses(value, moved)?;
            verify_expr_uses(next, moved)?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            verify_expr_uses(len, moved)?;
            verify_expr_uses(fill, moved)?;
        }
        NirExpr::LoadAt { buffer, index } => {
            verify_expr_uses(buffer, moved)?;
            verify_expr_uses(index, moved)?;
        }
        NirExpr::StoreValue { target, value } => {
            verify_expr_uses(target, moved)?;
            verify_expr_uses(value, moved)?;
        }
        NirExpr::StoreNext { target, next } => {
            verify_expr_uses(target, moved)?;
            verify_expr_uses(next, moved)?;
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            verify_expr_uses(buffer, moved)?;
            verify_expr_uses(index, moved)?;
            verify_expr_uses(value, moved)?;
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                verify_expr_uses(arg, moved)?;
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            verify_expr_uses(receiver, moved)?;
            for arg in args {
                verify_expr_uses(arg, moved)?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                verify_expr_uses(value, moved)?;
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            verify_expr_uses(lhs, moved)?;
            verify_expr_uses(rhs, moved)?;
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => {}
    }
    Ok(())
}

pub(super) fn expr_resource_key(expr: &NirExpr) -> Option<String> {
    match expr {
        NirExpr::Var(name) => Some(name.clone()),
        NirExpr::FieldAccess { base, field } => {
            let base = expr_resource_key(base)?;
            Some(format!("{base}.{field}"))
        }
        _ => None,
    }
}
