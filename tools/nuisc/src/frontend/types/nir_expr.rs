use super::*;

pub(crate) fn infer_nir_expr_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Bool(_) | NirExpr::IsNull(_) => Some(bool_type()),
        NirExpr::Text(_) => Some(string_type()),
        NirExpr::Int(_) => Some(i64_type()),
        NirExpr::F32(_) => Some(f32_type()),
        NirExpr::F64(_) => Some(f64_type()),
        NirExpr::Var(name) => bindings.get(name).cloned(),
        NirExpr::Await(value) => infer_nir_expr_type(value, bindings, signatures, struct_table)
            .and_then(|ty| {
                if ty.name == "Task" && ty.generic_args.len() == 1 {
                    ty.generic_args.first().cloned()
                } else {
                    Some(ty)
                }
            }),
        NirExpr::Instantiate { unit, .. } => {
            Some(generic_named_type("Instance", vec![named_type(unit)]))
        }
        NirExpr::Null => None,
        NirExpr::Borrow(value) | NirExpr::Move(value) => {
            infer_nir_expr_type(value, bindings, signatures, struct_table)
        }
        NirExpr::BorrowEnd(_) => Some(unit_type()),
        NirExpr::HostBufferHandle(_) => Some(i64_type()),
        NirExpr::AllocNode { .. } => Some(ref_type("Node")),
        NirExpr::AllocBuffer { .. } => Some(ref_type("Buffer")),
        NirExpr::DataBindCore(_) | NirExpr::CpuBindCore(_) => Some(unit_type()),
        NirExpr::CpuWindow { .. } => Some(named_type("Window")),
        NirExpr::CpuInputI64 { .. } | NirExpr::CpuTickI64 { .. } => Some(i64_type()),
        NirExpr::CpuSpawn { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone())
            .map(|ty| generic_named_type("Task", vec![ty])),
        NirExpr::CpuThreadSpawn { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone())
            .map(|ty| generic_named_type("Thread", vec![ty])),
        NirExpr::CpuJoin(task) => result_payload_type(task, bindings, signatures, struct_table),
        NirExpr::CpuCancel(task) => infer_nir_expr_type(task, bindings, signatures, struct_table),
        NirExpr::CpuJoinResult(task) => {
            result_payload_type(task, bindings, signatures, struct_table)
                .map(|ty| make_result_type(NirResultFamily::Task, ty))
        }
        NirExpr::CpuThreadJoin(thread) => {
            infer_nir_expr_type(thread, bindings, signatures, struct_table)
                .and_then(|ty| ty.thread_payload().cloned())
        }
        NirExpr::CpuThreadJoinResult(thread) => {
            infer_nir_expr_type(thread, bindings, signatures, struct_table)
                .and_then(|ty| ty.thread_payload().cloned())
                .map(|ty| make_result_type(NirResultFamily::Task, ty))
        }
        NirExpr::CpuMutexNew(value) => {
            infer_nir_expr_type(value, bindings, signatures, struct_table)
                .map(|ty| generic_named_type("Mutex", vec![ty]))
        }
        NirExpr::CpuMutexLock(mutex) => {
            infer_nir_expr_type(mutex, bindings, signatures, struct_table)
                .and_then(|ty| ty.mutex_payload().cloned())
                .map(|ty| generic_named_type("MutexGuard", vec![ty]))
        }
        NirExpr::CpuMutexUnlock(guard) => {
            infer_nir_expr_type(guard, bindings, signatures, struct_table)
                .and_then(|ty| ty.mutex_guard_payload().cloned())
                .map(|ty| generic_named_type("Mutex", vec![ty]))
        }
        NirExpr::CpuMutexValue(guard) => {
            infer_nir_expr_type(guard, bindings, signatures, struct_table)
                .and_then(|ty| ty.mutex_guard_payload().cloned())
        }
        NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_) => Some(bool_type()),
        NirExpr::CpuTaskValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::CpuTimeout { task, .. } | NirExpr::CpuReadyAfter { task, .. } => {
            infer_nir_expr_type(task, bindings, signatures, struct_table)
        }
        NirExpr::CpuPresentFrame(_) => Some(unit_type()),
        NirExpr::ShaderProfileTargetRef { .. } => Some(named_type("Target")),
        NirExpr::ShaderProfileViewportRef { .. } => Some(named_type("Viewport")),
        NirExpr::ShaderProfilePipelineRef { .. } => Some(named_type("Pipeline")),
        NirExpr::ShaderProfileVertexCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileInstanceCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketColorSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileSliderColorSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileSliderSpeedSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileSliderRadiusSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileHeaderAccentSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileToggleLiveSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileFocusSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketTagRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileMaterialModeRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePassKindRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketFieldCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileColorSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfileSpeedSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfileRadiusSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacket {
            unit,
            packet_type_name,
            ..
        } => {
            let packet_name = packet_type_name
                .clone()
                .unwrap_or_else(|| format!("{unit}Packet"));
            Some(named_type(&packet_name))
        }
        NirExpr::DataProfileBindCoreRef { .. } => Some(named_type("Unit")),
        NirExpr::DataProfileWindowOffsetRef { .. } => Some(i64_type()),
        NirExpr::DataProfileUplinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileDownlinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileHandleTableRef { .. } => Some(named_type("HandleTable")),
        NirExpr::DataProfileMarkerRef { .. } => Some(named_type("Marker")),
        NirExpr::NetworkProfileBindCoreRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileEndpointKindRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileTransportFamilyRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileLocalPortRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileRemotePortRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileConnectTimeoutRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileReadTimeoutRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileWriteTimeoutRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileTimeoutBudgetRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileRetryBudgetRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileStreamWindowRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileRecvWindowRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileSendWindowRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileProtocolKindRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileProtocolVersionRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileProtocolHeaderBytesRef { .. } => Some(i64_type()),
        NirExpr::NetworkResult { value, .. } => {
            expr_type(value, bindings, signatures, struct_table)
                .map(|inner| make_result_type(NirResultFamily::Network, inner))
        }
        NirExpr::NetworkConfigReady(_)
        | NirExpr::NetworkSendReady(_)
        | NirExpr::NetworkRecvReady(_)
        | NirExpr::NetworkAcceptReady(_) => Some(bool_type()),
        NirExpr::NetworkValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::KernelProfileBindCoreRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileQueueDepthRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileBatchLanesRef { .. } => Some(i64_type()),
        NirExpr::KernelResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Kernel, inner)),
        NirExpr::KernelConfigReady(_) => Some(bool_type()),
        NirExpr::KernelValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::KernelShape(_) => Some(named_type("TensorShape")),
        NirExpr::KernelRows(_) | NirExpr::KernelCols(_) | NirExpr::KernelElementAt { .. } => {
            Some(i64_type())
        }
        NirExpr::KernelTensor { .. }
        | NirExpr::KernelRow(_)
        | NirExpr::KernelCol(_)
        | NirExpr::KernelReshape { .. }
        | NirExpr::KernelBroadcast { .. }
        | NirExpr::KernelReduceSumAxis { .. }
        | NirExpr::KernelReduceMaxAxis { .. }
        | NirExpr::KernelReduceMeanAxis { .. }
        | NirExpr::KernelArgmaxAxis { .. }
        | NirExpr::KernelArgminAxis { .. }
        | NirExpr::KernelSort(_)
        | NirExpr::KernelSortAxis { .. }
        | NirExpr::KernelTopk { .. }
        | NirExpr::KernelTopkAxis { .. }
        | NirExpr::KernelMap { .. }
        | NirExpr::KernelMapAxis { .. }
        | NirExpr::KernelZip { .. }
        | NirExpr::KernelMatmul { .. }
        | NirExpr::KernelAddBias { .. }
        | NirExpr::KernelRelu(_) => Some(named_type("Tensor")),
        NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmin(_) => Some(i64_type()),
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            let window_inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            Some(generic_named_type("Window", vec![window_inner]))
        }
        NirExpr::DataResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Data, inner)),
        NirExpr::DataReady(_) | NirExpr::DataMoved(_) | NirExpr::DataWindowed(_) => {
            Some(bool_type())
        }
        NirExpr::DataValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::DataFreezeWindow(input) => {
            let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            let payload = match inner.window_mode() {
                Some(NirWindowMode::Mutable | NirWindowMode::Immutable) => {
                    inner.container_payload()?.clone()
                }
                None => return None,
            };
            Some(generic_named_type("Window", vec![payload]))
        }
        NirExpr::CastI64ToI32(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == i64_type() {
                Some(i32_type())
            } else {
                None
            }
        }
        NirExpr::CastI32ToI64(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == i32_type() {
                Some(i64_type())
            } else {
                None
            }
        }
        NirExpr::CastI64ToBool(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == i64_type() {
                Some(bool_type())
            } else {
                None
            }
        }
        NirExpr::CastBoolToI64(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == bool_type() {
                Some(i64_type())
            } else {
                None
            }
        }
        NirExpr::CastI64ToF32(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == i64_type() {
                Some(f32_type())
            } else {
                None
            }
        }
        NirExpr::CastF32ToI64(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == f32_type() {
                Some(i64_type())
            } else {
                None
            }
        }
        NirExpr::CastI64ToF64(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == i64_type() {
                Some(f64_type())
            } else {
                None
            }
        }
        NirExpr::CastF64ToI64(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == f64_type() {
                Some(i64_type())
            } else {
                None
            }
        }
        NirExpr::CpuExternCallI32 { .. } => Some(i32_type()),
        NirExpr::CpuExternCall { callee, .. }
            if callee == "host_text_handle"
                || callee == "host_text_len"
                || callee == "host_serialize_text_into"
                || callee == "host_serialize_bool_into"
                || callee == "host_serialize_i64_into"
                || callee == "host_serialize_byte_into"
                || callee == "host_fill_bytes"
                || callee == "host_copy_bytes"
                || callee == "host_compare_bytes"
                || callee == "host_buffer_find_text"
                || callee == "host_buffer_find_byte"
                || callee == "host_buffer_find_line_end"
                || callee == "host_buffer_trim_line_end" =>
        {
            Some(i64_type())
        }
        NirExpr::CpuExternCall { callee, .. }
            if callee == "host_deserialize_i64_from" || callee == "host_deserialize_byte_from" =>
        {
            Some(i64_type())
        }
        NirExpr::CpuExternCall { callee, .. }
            if callee == "host_deserialize_text_from"
                || callee == "host_parse_header_line"
                || callee == "host_find_header_value"
                || callee == "host_find_status_line_reason"
                || callee == "host_parse_http_response_summary"
                || callee == "host_parse_http_request_summary"
                || callee == "host_parse_http_roundtrip_summary" =>
        {
            Some(string_type())
        }
        NirExpr::CpuExternCall { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
        NirExpr::DataMarker(_) => Some(named_type("Marker")),
        NirExpr::DataHandleTable(_) => Some(named_type("HandleTable")),
        NirExpr::ShaderTarget { .. } => Some(named_type("Target")),
        NirExpr::ShaderViewport { .. } => Some(named_type("Viewport")),
        NirExpr::ShaderPipeline { .. } => Some(named_type("Pipeline")),
        NirExpr::ShaderTexture2d { .. } => Some(named_type("Texture")),
        NirExpr::ShaderSampler { .. } => Some(named_type("Sampler")),
        NirExpr::ShaderUv { .. } => Some(named_type("UV")),
        NirExpr::ShaderSample { .. } => Some(i64_type()),
        NirExpr::ShaderSampleUv { .. } => Some(i64_type()),
        NirExpr::ShaderBinding { .. } => Some(named_type("Binding")),
        NirExpr::ShaderBindSet { .. } => Some(named_type("BindingSet")),
        NirExpr::ShaderInlineWgsl { .. } => Some(named_type("ShaderModule")),
        NirExpr::ShaderResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Shader, inner)),
        NirExpr::ShaderPassReady(_) | NirExpr::ShaderFrameReady(_) => Some(bool_type()),
        NirExpr::ShaderValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::ShaderBeginPass { .. } => Some(named_type("Pass")),
        NirExpr::ShaderDrawInstanced { .. } => Some(named_type("Frame")),
        NirExpr::ShaderProfileRender { .. } => Some(named_type("Frame")),
        NirExpr::DataOutputPipe(value) => {
            let inner = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            Some(generic_named_type("Pipe", vec![inner]))
        }
        NirExpr::DataCopyWindow { input, .. } => infer_data_window_type(
            input,
            bindings,
            signatures,
            struct_table,
            NirWindowMode::Mutable,
        ),
        NirExpr::DataReadWindow { window, .. } => {
            let window_ty = infer_nir_expr_type(window, bindings, signatures, struct_table)?;
            window_ty.container_payload().cloned()
        }
        NirExpr::DataWriteWindow { window, value, .. } => {
            let window_ty = infer_nir_expr_type(window, bindings, signatures, struct_table)?;
            if window_ty.window_mode() != Some(NirWindowMode::Mutable) {
                return None;
            }
            let payload = window_ty.container_payload()?.clone();
            let value_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            if compatible_types(&payload, &value_ty) {
                Some(window_ty)
            } else {
                None
            }
        }
        NirExpr::DataImmutableWindow { input, .. } => infer_data_window_type(
            input,
            bindings,
            signatures,
            struct_table,
            NirWindowMode::Immutable,
        ),
        NirExpr::DataInputPipe(value) => {
            let pipe_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            pipe_ty.generic_args.first().cloned()
        }
        NirExpr::LoadValue(_) | NirExpr::BufferLen(_) => Some(i64_type()),
        NirExpr::LoadAt { buffer, .. } => {
            let target_ty = infer_nir_expr_type(buffer, bindings, signatures, struct_table)?;
            if target_ty.name == "Slice"
                && !target_ty.is_ref
                && !target_ty.is_optional
                && target_ty.generic_args.len() == 1
            {
                Some(target_ty.generic_args[0].clone())
            } else {
                Some(i64_type())
            }
        }
        NirExpr::LoadNext(_) => Some(ref_type("Node")),
        NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::Free(_) => Some(unit_type()),
        NirExpr::Call { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
        NirExpr::MethodCall { .. } => None,
        NirExpr::StructLiteral {
            type_name,
            type_args,
            ..
        } => Some(generic_named_type(type_name, type_args.clone())),
        NirExpr::FieldAccess { base, field } => {
            let base_ty = infer_nir_expr_type(base, bindings, signatures, struct_table)?;
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Node" {
                return match field.as_str() {
                    "value" => Some(i64_type()),
                    "next" => Some(ref_type("Node")),
                    _ => None,
                };
            }
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Buffer" {
                return match field.as_str() {
                    "len" => Some(i64_type()),
                    _ => None,
                };
            }
            struct_field_type(&base_ty, field, struct_table)
        }
        NirExpr::VariantIs { .. } => Some(bool_type()),
        NirExpr::VariantFieldAccess {
            base,
            variant,
            field,
        } => {
            let base_ty = infer_nir_expr_type(base, bindings, signatures, struct_table)?;
            let variant_ty = if variant
                .rsplit_once('.')
                .is_some_and(|(parent, _)| parent == base_ty.name)
            {
                NirTypeRef {
                    name: variant.clone(),
                    generic_args: base_ty.generic_args,
                    is_optional: false,
                    is_ref: false,
                }
            } else {
                named_type(variant)
            };
            struct_field_type(&variant_ty, field, struct_table)
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let lhs_ty = infer_nir_expr_type(lhs, bindings, signatures, struct_table)?;
            let rhs_ty = infer_nir_expr_type(rhs, bindings, signatures, struct_table)?;
            match op {
                NirBinaryOp::And | NirBinaryOp::Or => {
                    if compatible_types(&lhs_ty, &rhs_ty)
                        && lhs_ty.is_bool_scalar()
                        && rhs_ty.is_bool_scalar()
                    {
                        Some(bool_type())
                    } else {
                        None
                    }
                }
                NirBinaryOp::Add
                | NirBinaryOp::Sub
                | NirBinaryOp::Mul
                | NirBinaryOp::Div
                | NirBinaryOp::Rem => {
                    if compatible_types(&lhs_ty, &rhs_ty) && lhs_ty.is_numeric_scalar() {
                        Some(lhs_ty)
                    } else {
                        None
                    }
                }
                NirBinaryOp::Eq | NirBinaryOp::Ne => {
                    if compatible_types(&lhs_ty, &rhs_ty)
                        && ((lhs_ty.is_integer_scalar() && rhs_ty.is_integer_scalar())
                            || (lhs_ty.is_bool_scalar() && rhs_ty.is_bool_scalar()))
                    {
                        Some(bool_type())
                    } else {
                        None
                    }
                }
                NirBinaryOp::Lt | NirBinaryOp::Le | NirBinaryOp::Gt | NirBinaryOp::Ge => {
                    if compatible_types(&lhs_ty, &rhs_ty)
                        && lhs_ty.is_integer_scalar()
                        && rhs_ty.is_integer_scalar()
                    {
                        Some(bool_type())
                    } else {
                        None
                    }
                }
            }
        }
    }
}
