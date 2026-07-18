use super::{NirExpr, NirExprEffectClass, NirHostReadSurface, NirHostSchedulerBridge};

pub fn nir_expr_effect_class(expr: &NirExpr) -> NirExprEffectClass {
    match expr {
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_)
        | NirExpr::CastI64ToI32(_)
        | NirExpr::CastI32ToI64(_)
        | NirExpr::CastI64ToBool(_)
        | NirExpr::CastBoolToI64(_)
        | NirExpr::CastI64ToF32(_)
        | NirExpr::CastF32ToI64(_)
        | NirExpr::CastI64ToF64(_)
        | NirExpr::CastF64ToI64(_)
        | NirExpr::HostBufferHandle(_)
        | NirExpr::StructLiteral { .. }
        | NirExpr::FieldAccess { .. }
        | NirExpr::VariantIs { .. }
        | NirExpr::VariantFieldAccess { .. }
        | NirExpr::Binary { .. }
        | NirExpr::IsNull(_) => NirExprEffectClass::Pure,
        NirExpr::Borrow(_)
        | NirExpr::BorrowEnd(_)
        | NirExpr::LoadValue(_)
        | NirExpr::LoadNext(_)
        | NirExpr::BufferLen(_)
        | NirExpr::LoadAt { .. } => NirExprEffectClass::LocalReadOnly,
        NirExpr::Await(_) => NirExprEffectClass::AsyncOpaque,
        NirExpr::Call { .. } | NirExpr::MethodCall { .. } => NirExprEffectClass::CallOpaque,
        NirExpr::Instantiate { .. } => NirExprEffectClass::DomainOpaque,
        NirExpr::CpuBindCore(_)
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
        | NirExpr::ShaderBinding { .. }
        | NirExpr::ShaderInlineWgsl { .. } => NirExprEffectClass::HostReadOnly,
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::DataResult { .. }
        | NirExpr::DataReady(_)
        | NirExpr::DataMoved(_)
        | NirExpr::DataWindowed(_)
        | NirExpr::DataValue(_)
        | NirExpr::DataCopyWindow { .. }
        | NirExpr::DataReadWindow { .. }
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataInputPipe(_)
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
        | NirExpr::ShaderProfileColorSeed { .. }
        | NirExpr::ShaderProfileSpeedSeed { .. }
        | NirExpr::ShaderProfileRadiusSeed { .. }
        | NirExpr::ShaderProfilePacket { .. }
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
        | NirExpr::NetworkResult { .. }
        | NirExpr::NetworkConfigReady(_)
        | NirExpr::NetworkSendReady(_)
        | NirExpr::NetworkRecvReady(_)
        | NirExpr::NetworkAcceptReady(_)
        | NirExpr::NetworkValue(_)
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelResult { .. }
        | NirExpr::KernelConfigReady(_)
        | NirExpr::KernelValue(_)
        | NirExpr::KernelTensor { .. }
        | NirExpr::KernelShape(_)
        | NirExpr::KernelRows(_)
        | NirExpr::KernelCols(_)
        | NirExpr::KernelRow(_)
        | NirExpr::KernelCol(_)
        | NirExpr::KernelElementAt { .. }
        | NirExpr::KernelReshape { .. }
        | NirExpr::KernelBroadcast { .. }
        | NirExpr::KernelMap { .. }
        | NirExpr::KernelMapAxis { .. }
        | NirExpr::KernelZip { .. }
        | NirExpr::KernelMatmul { .. }
        | NirExpr::KernelAddBias { .. }
        | NirExpr::KernelRelu(_)
        | NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceSumAxis { .. }
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMaxAxis { .. }
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelReduceMeanAxis { .. }
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmaxAxis { .. }
        | NirExpr::KernelArgmin(_)
        | NirExpr::KernelArgminAxis { .. }
        | NirExpr::KernelSort(_)
        | NirExpr::KernelSortAxis { .. }
        | NirExpr::KernelTopk { .. }
        | NirExpr::KernelTopkAxis { .. }
        | NirExpr::ShaderResult { .. } => NirExprEffectClass::DomainReadOnly,
        NirExpr::CpuWindow { .. }
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
        | NirExpr::CpuReadyAfter { .. }
        | NirExpr::CpuPresentFrame(_)
        | NirExpr::ShaderPassReady(_)
        | NirExpr::ShaderFrameReady(_)
        | NirExpr::ShaderValue(_)
        | NirExpr::ShaderBeginPass { .. }
        | NirExpr::ShaderDrawInstanced { .. }
        | NirExpr::ShaderProfileRender { .. } => NirExprEffectClass::Stateful,
        NirExpr::ShaderSample { .. }
        | NirExpr::ShaderSampleUv { .. }
        | NirExpr::ShaderBindSet { .. } => NirExprEffectClass::DomainReadOnly,
        NirExpr::Move(_)
        | NirExpr::AllocNode { .. }
        | NirExpr::AllocBuffer { .. }
        | NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::DataOutputPipe(_)
        | NirExpr::DataWriteWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::CpuExternCall { .. }
        | NirExpr::CpuExternCallI32 { .. }
        | NirExpr::Free(_) => NirExprEffectClass::Stateful,
    }
}

pub fn nir_host_read_surface(expr: &NirExpr) -> Option<NirHostReadSurface> {
    match expr {
        NirExpr::CpuBindCore(_) => Some(NirHostReadSurface::SchedulerLane),
        NirExpr::CpuInputI64 { .. } => Some(NirHostReadSurface::InputChannel),
        NirExpr::CpuTickI64 { .. } => Some(NirHostReadSurface::ClockTick),
        NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. } => Some(NirHostReadSurface::RenderDescriptor),
        _ => None,
    }
}

pub fn nir_host_scheduler_bridge(expr: &NirExpr) -> Option<NirHostSchedulerBridge> {
    match expr {
        NirExpr::CpuBindCore(lane) => Some(NirHostSchedulerBridge::from_cpu_bind_core(*lane)),
        _ => None,
    }
}
