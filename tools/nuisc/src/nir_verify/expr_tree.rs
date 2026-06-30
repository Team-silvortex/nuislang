use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::data::NirDataKind;
use super::super::task_result_facts::{
    borrowed_address_alias_source, BorrowBindings, TaskResultStateFact,
};
use super::super::{ensure_owned_address_target, owned_structural_address_error};
use super::expr_effects::apply_guaranteed_expr_effects;
use super::{verify_expr, verify_expr_sequence};

#[path = "expr_tree_data.rs"]
mod expr_tree_data;
#[path = "expr_tree_kernel.rs"]
mod expr_tree_kernel;
#[path = "expr_tree_shader.rs"]
mod expr_tree_shader;
use expr_tree_data::verify_data_expr_tree;
use expr_tree_kernel::verify_kernel_expr_tree;
use expr_tree_shader::verify_shader_expr_tree;

pub(super) fn verify_expr_tree(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    if verify_shader_expr_tree(
        expr,
        moved,
        borrows,
        borrow_bindings,
        data_bindings,
        task_result_facts,
    )? {
        return Ok(());
    }

    if verify_kernel_expr_tree(
        expr,
        moved,
        borrows,
        borrow_bindings,
        data_bindings,
        task_result_facts,
    )? {
        return Ok(());
    }

    if verify_data_expr_tree(
        expr,
        moved,
        borrows,
        borrow_bindings,
        data_bindings,
        task_result_facts,
    )? {
        return Ok(());
    }

    match expr {
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null
        | NirExpr::Instantiate { .. } => {}
        NirExpr::Var(name) => {
            if !data_bindings.contains_key(name) {
                return Err(format!("nir verify: use of unbound value `{name}`"));
            }
            if let Some(binding) = borrow_bindings.get(name) {
                if borrows.get(&binding.source).copied().unwrap_or(0) == 0 {
                    return Err(format!(
                        "nir verify: borrow alias `{}` for `{}` is not active",
                        name, binding.source
                    ));
                }
            }
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
        | NirExpr::KernelReduceSum(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::NetworkResult { value: inner, .. } => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelArgmax(inner) | NirExpr::KernelArgmin(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelArgmaxAxis { input, .. } | NirExpr::KernelArgminAxis { input, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?
        }
        NirExpr::KernelReduceMaxAxis { input, .. }
        | NirExpr::KernelReduceMeanAxis { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelReduceSumAxis { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelSort(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelSortAxis { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelTopkAxis { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::KernelTopk { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuExternCallI32 { args, .. } => verify_expr_sequence(
            args.iter(),
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::CpuTimeout { task, limit } => {
            verify_expr(
                task,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut next_moved = moved.clone();
            let mut next_borrows = borrows.clone();
            let mut next_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                task,
                &mut next_moved,
                &mut next_borrows,
                &mut next_borrow_bindings,
                true,
            );
            verify_expr(
                limit,
                &next_moved,
                &next_borrows,
                &next_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::DataOutputPipe(inner) | NirExpr::DataInputPipe(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::DataResult { value: inner, .. }
        | NirExpr::ShaderResult { value: inner, .. }
        | NirExpr::KernelResult { value: inner, .. } => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
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
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::AllocNode { value, next } => {
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                next,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if borrowed_address_alias_source(next, borrow_bindings).is_some() {
                return Err(owned_structural_address_error(
                    "alloc_node(..., next)",
                    next,
                    borrow_bindings,
                ));
            }
        }
        NirExpr::AllocBuffer { len, fill } => {
            verify_expr(
                len,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                fill,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::LoadAt { buffer, index } => {
            verify_expr(
                buffer,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                index,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::StoreValue { target, value } => {
            verify_expr(
                target,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut next_moved = moved.clone();
            let mut next_borrows = borrows.clone();
            let mut next_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                target,
                &mut next_moved,
                &mut next_borrows,
                &mut next_borrow_bindings,
                true,
            );
            verify_expr(
                value,
                &next_moved,
                &next_borrows,
                &next_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            ensure_owned_address_target("store_value(..., target)", target, &next_borrow_bindings)?;
        }
        NirExpr::StoreNext { target, next } => {
            verify_expr(
                target,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut next_moved = moved.clone();
            let mut next_borrows = borrows.clone();
            let mut next_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                target,
                &mut next_moved,
                &mut next_borrows,
                &mut next_borrow_bindings,
                true,
            );
            verify_expr(
                next,
                &next_moved,
                &next_borrows,
                &next_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            ensure_owned_address_target("store_next(..., target)", target, &next_borrow_bindings)?;
            if borrowed_address_alias_source(next, &next_borrow_bindings).is_some() {
                return Err(owned_structural_address_error(
                    "store_next(..., next)",
                    next,
                    &next_borrow_bindings,
                ));
            }
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            verify_expr(
                buffer,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut current_moved = moved.clone();
            let mut current_borrows = borrows.clone();
            let mut current_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                buffer,
                &mut current_moved,
                &mut current_borrows,
                &mut current_borrow_bindings,
                true,
            );
            verify_expr(
                index,
                &current_moved,
                &current_borrows,
                &current_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            apply_guaranteed_expr_effects(
                index,
                &mut current_moved,
                &mut current_borrows,
                &mut current_borrow_bindings,
                true,
            );
            verify_expr(
                value,
                &current_moved,
                &current_borrows,
                &current_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            ensure_owned_address_target("store_at(..., buffer)", buffer, &current_borrow_bindings)?;
        }
        NirExpr::Call { args, .. } => verify_expr_sequence(
            args.iter(),
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::MethodCall { receiver, args, .. } => {
            let exprs = std::iter::once(receiver.as_ref()).chain(args.iter());
            verify_expr_sequence(
                exprs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::StructLiteral { fields, .. } => verify_expr_sequence(
            fields.iter().map(|(_, value)| value),
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::FieldAccess { base, .. } => verify_expr(
            base,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::Binary { lhs, rhs, .. } => {
            verify_expr(
                lhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut rhs_moved = moved.clone();
            let mut rhs_borrows = borrows.clone();
            let mut rhs_borrow_bindings = borrow_bindings.clone();
            apply_guaranteed_expr_effects(
                lhs,
                &mut rhs_moved,
                &mut rhs_borrows,
                &mut rhs_borrow_bindings,
                true,
            );
            verify_expr(
                rhs,
                &rhs_moved,
                &rhs_borrows,
                &rhs_borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        _ => {}
    }

    Ok(())
}
