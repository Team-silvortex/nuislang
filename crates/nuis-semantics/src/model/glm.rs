use super::{NirExpr, NirGlmAccess, NirGlmEffect, NirGlmProfile, NirGlmUseMode, NirGlmValueClass};

pub fn nir_glm_profile(expr: &NirExpr) -> Option<NirGlmProfile> {
    match expr {
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::CastI64ToI32(_)
        | NirExpr::CastI32ToI64(_)
        | NirExpr::CastI64ToBool(_)
        | NirExpr::CastBoolToI64(_)
        | NirExpr::CastI64ToF32(_)
        | NirExpr::CastF32ToI64(_)
        | NirExpr::CastI64ToF64(_)
        | NirExpr::CastF64ToI64(_)
        | NirExpr::Var(_)
        | NirExpr::Await(_)
        | NirExpr::Instantiate { .. }
        | NirExpr::Call { .. }
        | NirExpr::CpuExternCallI32 { .. }
        | NirExpr::MethodCall { .. }
        | NirExpr::StructLiteral { .. }
        | NirExpr::FieldAccess { .. }
        | NirExpr::VariantIs { .. }
        | NirExpr::VariantFieldAccess { .. }
        | NirExpr::Binary { .. }
        | NirExpr::IsNull(_) => None,
        NirExpr::CpuJoin(_) | NirExpr::CpuJoinResult(_) | NirExpr::CpuThreadJoin(_) => {
            Some(NirGlmProfile {
                result_class: NirGlmValueClass::Val,
                accesses: vec![NirGlmAccess {
                    class: NirGlmValueClass::Res,
                    mode: NirGlmUseMode::Own,
                }],
                effect: NirGlmEffect::None,
            })
        }
        NirExpr::CpuThreadJoinResult(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_)
        | NirExpr::CpuTaskFailed(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuMutexValue(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::CpuCancel(_)
        | NirExpr::CpuTimeout { .. }
        | NirExpr::CpuReadyAfter { .. }
        | NirExpr::CpuMutexUnlock(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::CpuThreadSpawn { .. } | NirExpr::CpuMutexNew(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::CpuMutexLock(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::Borrow(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::BorrowEnd(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::HostBufferHandle(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::Move(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::SelectOwnedPointer { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![
                NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Read,
                },
                NirGlmAccess {
                    class: NirGlmValueClass::Res,
                    mode: NirGlmUseMode::Own,
                },
                NirGlmAccess {
                    class: NirGlmValueClass::Res,
                    mode: NirGlmUseMode::Own,
                },
            ],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::AllocNode { .. } | NirExpr::AllocBuffer { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::CopyBufferOwned(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Res,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::BytesLen(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::DataResult { .. }
        | NirExpr::DataReady(_)
        | NirExpr::DataMoved(_)
        | NirExpr::DataWindowed(_)
        | NirExpr::DataValue(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::CpuSpawn { .. }
        | NirExpr::CpuPresentFrame(_)
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
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::CpuExternCall { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
        | NirExpr::ShaderSample { .. }
        | NirExpr::ShaderSampleUv { .. }
        | NirExpr::ShaderBinding { .. }
        | NirExpr::ShaderBindSet { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::ShaderResult { .. }
        | NirExpr::ShaderPassReady(_)
        | NirExpr::ShaderFrameReady(_)
        | NirExpr::ShaderValue(_)
        | NirExpr::ShaderBeginPass { .. }
        | NirExpr::ShaderDrawInstanced { .. }
        | NirExpr::ShaderProfileRender { .. } => None,
        NirExpr::DataProviderRequestIngress { capsule_token, .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![
                NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Read,
                };
                if capsule_token.is_some() { 8 } else { 5 }
            ],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataOutputPipe(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::DomainMove,
        }),
        NirExpr::DataInputPipe(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataCopyWindow { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataReadWindow { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![
                NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Read,
                },
                NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Read,
                },
            ],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataWriteWindow { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![
                NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Write,
                },
                NirGlmAccess {
                    class: NirGlmValueClass::Val,
                    mode: NirGlmUseMode::Read,
                },
            ],
            effect: NirGlmEffect::None,
        }),
        NirExpr::DataFreezeWindow(_) | NirExpr::DataImmutableWindow { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Val,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::LoadValue(_)
        | NirExpr::LoadNext(_)
        | NirExpr::BufferLen(_)
        | NirExpr::LoadAt { .. } => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Read,
            }],
            effect: NirGlmEffect::None,
        }),
        NirExpr::StoreValue { .. } | NirExpr::StoreNext { .. } | NirExpr::StoreAt { .. } => {
            Some(NirGlmProfile {
                result_class: NirGlmValueClass::Val,
                accesses: vec![NirGlmAccess {
                    class: NirGlmValueClass::Res,
                    mode: NirGlmUseMode::Write,
                }],
                effect: NirGlmEffect::None,
            })
        }
        NirExpr::Free(_) | NirExpr::DropBytes(_) => Some(NirGlmProfile {
            result_class: NirGlmValueClass::Val,
            accesses: vec![NirGlmAccess {
                class: NirGlmValueClass::Res,
                mode: NirGlmUseMode::Own,
            }],
            effect: NirGlmEffect::LifetimeEnd,
        }),
    }
}
