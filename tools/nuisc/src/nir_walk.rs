use nuis_semantics::model::NirExpr;

#[cfg(test)]
#[path = "nir_walk_tests.rs"]
mod tests;

pub(crate) fn walk_child_exprs<'a>(expr: &'a NirExpr, f: &mut dyn FnMut(&'a NirExpr)) {
    match expr {
        NirExpr::SelectOwnedPointer {
            condition,
            then_owner,
            else_owner,
            ..
        } => {
            f(condition);
            f(then_owner);
            f(else_owner);
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::OwnedObjectSize(inner)
        | NirExpr::CopyBufferOwned(inner)
        | NirExpr::BytesLen(inner)
        | NirExpr::DropBytes(inner)
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
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::KernelSort(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => f(inner),
        NirExpr::AllocNode { value, next } => {
            f(value);
            f(next);
        }
        NirExpr::AllocBuffer { len, fill } => {
            f(len);
            f(fill);
        }
        NirExpr::LoadAt { buffer, index } => {
            f(buffer);
            f(index);
        }
        NirExpr::OwnedObjectReadI64 { object, index } => {
            f(object);
            f(index);
        }
        NirExpr::StoreValue { target, value } => {
            f(target);
            f(value);
        }
        NirExpr::StoreNext { target, next } => {
            f(target);
            f(next);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            f(buffer);
            f(index);
            f(value);
        }
        NirExpr::DataResult {
            value,
            completion_clock,
            ..
        }
        | NirExpr::NetworkResult {
            value,
            completion_clock,
            ..
        }
        | NirExpr::ShaderResult {
            value,
            completion_clock,
            ..
        }
        | NirExpr::KernelResult {
            value,
            completion_clock,
            ..
        } => {
            f(value);
            if let Some(clock) = completion_clock {
                f(clock);
            }
        }
        NirExpr::ResultCompletionReceipt { result, .. } => f(result),
        NirExpr::DataReadWindow { window, index } => {
            f(window);
            f(index);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            f(window);
            f(index);
            f(value);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            f(input);
            f(offset);
            f(len);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            f(base);
            f(delta);
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
            f(color);
            f(speed);
            f(radius);
            if let Some(accent) = accent {
                f(accent);
            }
            if let Some(toggle_state) = toggle_state {
                f(toggle_state);
            }
            if let Some(focus_index) = focus_index {
                f(focus_index);
            }
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            f(delta);
            f(scale);
            f(base);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. }
        | NirExpr::FieldAccess { base: input, .. }
        | NirExpr::VariantIs { base: input, .. }
        | NirExpr::VariantFieldAccess { base: input, .. }
        | NirExpr::ShaderBinding { value: input, .. }
        | NirExpr::KernelReshape { input, .. }
        | NirExpr::KernelBroadcast { input, .. }
        | NirExpr::KernelReduceSumAxis { input, .. }
        | NirExpr::KernelReduceMaxAxis { input, .. }
        | NirExpr::KernelReduceMeanAxis { input, .. }
        | NirExpr::KernelArgmaxAxis { input, .. }
        | NirExpr::KernelArgminAxis { input, .. }
        | NirExpr::KernelSortAxis { input, .. }
        | NirExpr::KernelTopk { input, .. }
        | NirExpr::KernelTopkAxis { input, .. } => f(input),
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuMutexCapability { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuExternCallI32 { args, .. }
        | NirExpr::CpuExternCallOwnedBuffer { args, .. }
        | NirExpr::CpuExternCallOwnedUtf8 { args, .. }
        | NirExpr::CpuExternCallOwnedObject { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                f(arg);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            f(task);
            f(limit);
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            f(task);
            f(delay);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            f(receiver);
            for arg in args {
                f(arg);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                f(value);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            f(target);
            f(pipeline);
            f(viewport);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
            binding_set,
        } => {
            f(pass);
            f(packet);
            f(vertex_count);
            f(instance_count);
            if let Some(set) = binding_set {
                f(set);
            }
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
                f(value);
            }
            for value in [capsule_token, input_role_count, output_role_count]
                .into_iter()
                .flatten()
            {
                f(value);
            }
        }
        NirExpr::KernelElementAt { input, row, col } => {
            f(input);
            f(row);
            f(col);
        }
        NirExpr::KernelMap { input, scalar, .. } | NirExpr::KernelMapAxis { input, scalar, .. } => {
            f(input);
            if let Some(scalar) = scalar {
                f(scalar);
            }
        }
        NirExpr::KernelZip { lhs, rhs, .. } | NirExpr::KernelMatmul { lhs, rhs } => {
            f(lhs);
            f(rhs);
        }
        NirExpr::KernelAddBias { input, bias } => {
            f(input);
            f(bias);
        }
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            ..
        } => {
            f(texture);
            f(sampler);
            f(x);
            f(y);
        }
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            ..
        } => {
            f(texture);
            f(sampler);
            f(uv);
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            f(pipeline);
            for binding in bindings {
                f(binding);
            }
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_)
        | NirExpr::Null
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
        | NirExpr::KernelTensor { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
        | NirExpr::ShaderInlineWgsl { .. } => {}
    }
}
