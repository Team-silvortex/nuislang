use std::collections::BTreeSet;
use std::path::Path;

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::YirModule;

use super::NUSTAR_REGISTRY_ROOT;

pub(super) fn validate_instantiated_units(module: &NirModule) -> Result<(), String> {
    for (domain, unit) in collect_instantiated_units(module) {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &domain)?;
        crate::registry::validate_unit_binding(&[manifest], &domain, &unit)?;
    }
    Ok(())
}

pub(super) fn validate_used_units_with_local_units(
    module: &NirModule,
    local_units: &BTreeSet<(String, String)>,
) -> Result<(), String> {
    for item in &module.uses {
        if local_units.contains(&(item.domain.clone(), item.unit.clone())) {
            continue;
        }
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new(NUSTAR_REGISTRY_ROOT),
            &item.domain,
        )?;
        crate::registry::validate_unit_binding(&[manifest], &item.domain, &item.unit)?;
    }
    Ok(())
}

pub(super) fn collect_loaded_nustar(
    module: &NirModule,
    yir: &YirModule,
    root_package: &str,
) -> Result<Vec<String>, String> {
    let mut loaded = crate::registry::required_package_ids(yir);
    loaded.push(root_package.to_owned());
    for item in &module.uses {
        let manifest = crate::registry::load_manifest_for_domain(
            Path::new(NUSTAR_REGISTRY_ROOT),
            &item.domain,
        )?;
        loaded.push(manifest.package_id);
    }
    for (domain, _) in collect_instantiated_units(module) {
        let manifest =
            crate::registry::load_manifest_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), &domain)?;
        loaded.push(manifest.package_id);
    }
    loaded.sort();
    loaded.dedup();
    Ok(loaded)
}

fn collect_instantiated_units(module: &NirModule) -> Vec<(String, String)> {
    let mut units = Vec::new();
    for function in &module.functions {
        for stmt in &function.body {
            collect_instantiated_units_stmt(stmt, &mut units);
        }
    }
    units.sort();
    units.dedup();
    units
}

fn collect_instantiated_units_stmt(stmt: &NirStmt, units: &mut Vec<(String, String)>) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => collect_instantiated_units_expr(value, units),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_instantiated_units_expr(condition, units);
            for stmt in then_body {
                collect_instantiated_units_stmt(stmt, units);
            }
            for stmt in else_body {
                collect_instantiated_units_stmt(stmt, units);
            }
        }
        NirStmt::While { condition, body } => {
            collect_instantiated_units_expr(condition, units);
            for stmt in body {
                collect_instantiated_units_stmt(stmt, units);
            }
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                collect_instantiated_units_expr(value, units);
            }
        }
    }
}

fn collect_instantiated_units_expr(expr: &NirExpr, units: &mut Vec<(String, String)>) {
    match expr {
        NirExpr::Instantiate { domain, unit } => units.push((domain.clone(), unit.clone())),
        NirExpr::SelectOwnedPointer {
            condition,
            then_owner,
            else_owner,
            ..
        } => {
            collect_instantiated_units_expr(condition, units);
            collect_instantiated_units_expr(then_owner, units);
            collect_instantiated_units_expr(else_owner, units);
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
                collect_instantiated_units_expr(value, units);
            }
            for value in [capsule_token, input_role_count, output_role_count]
                .into_iter()
                .flatten()
            {
                collect_instantiated_units_expr(value, units);
            }
        }
        NirExpr::CpuBindCore(_)
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
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
        | NirExpr::ShaderInlineWgsl { .. } => {}
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            collect_instantiated_units_expr(base, units);
            collect_instantiated_units_expr(delta, units);
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            collect_instantiated_units_expr(delta, units);
            collect_instantiated_units_expr(scale, units);
            collect_instantiated_units_expr(base, units);
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            collect_instantiated_units_expr(base, units);
            collect_instantiated_units_expr(delta, units);
        }
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            ..
        } => {
            collect_instantiated_units_expr(texture, units);
            collect_instantiated_units_expr(sampler, units);
            collect_instantiated_units_expr(x, units);
            collect_instantiated_units_expr(y, units);
        }
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            ..
        } => {
            collect_instantiated_units_expr(texture, units);
            collect_instantiated_units_expr(sampler, units);
            collect_instantiated_units_expr(uv, units);
        }
        NirExpr::ShaderBinding { value, .. } => {
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            collect_instantiated_units_expr(pipeline, units);
            for binding in bindings {
                collect_instantiated_units_expr(binding, units);
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
            collect_instantiated_units_expr(color, units);
            collect_instantiated_units_expr(speed, units);
            collect_instantiated_units_expr(radius, units);
            if let Some(accent) = accent {
                collect_instantiated_units_expr(accent, units);
            }
            if let Some(toggle_state) = toggle_state {
                collect_instantiated_units_expr(toggle_state, units);
            }
            if let Some(focus_index) = focus_index {
                collect_instantiated_units_expr(focus_index, units);
            }
        }
        NirExpr::Borrow(inner)
        | NirExpr::Await(inner)
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
        | NirExpr::NetworkResult { value: inner, .. }
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => collect_instantiated_units_expr(inner, units),
        NirExpr::KernelMatmul { lhs, rhs } => {
            collect_instantiated_units_expr(lhs, units);
            collect_instantiated_units_expr(rhs, units);
        }
        NirExpr::KernelElementAt { input, row, col } => {
            collect_instantiated_units_expr(input, units);
            collect_instantiated_units_expr(row, units);
            collect_instantiated_units_expr(col, units);
        }
        NirExpr::KernelReshape { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::KernelBroadcast { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            collect_instantiated_units_expr(input, units);
            if let Some(scalar) = scalar {
                collect_instantiated_units_expr(scalar, units);
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            collect_instantiated_units_expr(input, units);
            if let Some(scalar) = scalar {
                collect_instantiated_units_expr(scalar, units);
            }
        }
        NirExpr::KernelTopk { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            collect_instantiated_units_expr(lhs, units);
            collect_instantiated_units_expr(rhs, units);
        }
        NirExpr::KernelAddBias { input, bias } => {
            collect_instantiated_units_expr(input, units);
            collect_instantiated_units_expr(bias, units);
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuThreadSpawn { args, .. } => {
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            collect_instantiated_units_expr(task, units);
            collect_instantiated_units_expr(limit, units);
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            collect_instantiated_units_expr(task, units);
            collect_instantiated_units_expr(delay, units);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            collect_instantiated_units_expr(target, units);
            collect_instantiated_units_expr(pipeline, units);
            collect_instantiated_units_expr(viewport, units);
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            collect_instantiated_units_expr(packet, units);
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            collect_instantiated_units_expr(pass, units);
            collect_instantiated_units_expr(packet, units);
        }
        NirExpr::CpuExternCall { args, .. } | NirExpr::CpuExternCallI32 { args, .. } => {
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::AllocNode { value, next } => {
            collect_instantiated_units_expr(value, units);
            collect_instantiated_units_expr(next, units);
        }
        NirExpr::AllocBuffer { len, fill } => {
            collect_instantiated_units_expr(len, units);
            collect_instantiated_units_expr(fill, units);
        }
        NirExpr::LoadAt { buffer, index } => {
            collect_instantiated_units_expr(buffer, units);
            collect_instantiated_units_expr(index, units);
        }
        NirExpr::StoreValue { target, value } => {
            collect_instantiated_units_expr(target, units);
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::StoreNext { target, next } => {
            collect_instantiated_units_expr(target, units);
            collect_instantiated_units_expr(next, units);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            collect_instantiated_units_expr(buffer, units);
            collect_instantiated_units_expr(index, units);
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::DataResult { value: input, .. }
        | NirExpr::ShaderResult { value: input, .. }
        | NirExpr::KernelResult { value: input, .. } => {
            collect_instantiated_units_expr(input, units)
        }
        NirExpr::DataReadWindow { window, index } => {
            collect_instantiated_units_expr(window, units);
            collect_instantiated_units_expr(index, units);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            collect_instantiated_units_expr(window, units);
            collect_instantiated_units_expr(index, units);
            collect_instantiated_units_expr(value, units);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            collect_instantiated_units_expr(input, units);
            collect_instantiated_units_expr(offset, units);
            collect_instantiated_units_expr(len, units);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            collect_instantiated_units_expr(input, units);
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            collect_instantiated_units_expr(receiver, units);
            for arg in args {
                collect_instantiated_units_expr(arg, units);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_instantiated_units_expr(value, units);
            }
        }
        NirExpr::FieldAccess { base, .. }
        | NirExpr::VariantIs { base, .. }
        | NirExpr::VariantFieldAccess { base, .. } => collect_instantiated_units_expr(base, units),
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_instantiated_units_expr(lhs, units);
            collect_instantiated_units_expr(rhs, units);
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_) => {}
    }
}
