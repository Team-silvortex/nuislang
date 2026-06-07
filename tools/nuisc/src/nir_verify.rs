mod effects;
mod task_result_facts;

use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    nir_glm_profile, NirExpr, NirFunction, NirGlmUseMode, NirModule, NirStmt, NirTypeRef,
};

use self::effects::{ensure_binding_can_be_rebound, merge_branch_state, note_binding_effects};
use self::task_result_facts::{
    apply_task_result_condition_facts, expr_is_borrowed_pointer, TaskResultStateFact,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NirDataKind {
    Other,
    WindowMutable,
    WindowImmutable,
    Marker,
    HandleTable,
    PipeOutput,
    PipeInput,
}

impl NirDataKind {
    fn merge_with_type_hint(self, hint: Option<NirDataKind>) -> NirDataKind {
        if self == NirDataKind::Other {
            hint.unwrap_or(self)
        } else {
            self
        }
    }
}

pub fn verify_nir_module(module: &NirModule) -> Result<(), String> {
    verify_declared_types(module)?;
    for function in &module.functions {
        verify_function(function)?;
    }
    Ok(())
}

fn verify_declared_types(module: &NirModule) -> Result<(), String> {
    for function in &module.externs {
        for param in &function.params {
            verify_type_ref(&param.ty)?;
        }
        verify_type_ref(&function.return_type)?;
    }
    for interface in &module.extern_interfaces {
        for method in &interface.methods {
            for param in &method.params {
                verify_type_ref(&param.ty)?;
            }
            verify_type_ref(&method.return_type)?;
        }
    }
    for definition in &module.structs {
        for field in &definition.fields {
            verify_type_ref(&field.ty)?;
        }
    }
    for function in &module.functions {
        for param in &function.params {
            verify_type_ref(&param.ty)?;
        }
        if let Some(return_type) = &function.return_type {
            verify_type_ref(return_type)?;
        }
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { ty, .. } => {
                    if let Some(ty) = ty {
                        verify_type_ref(ty)?;
                    }
                }
                NirStmt::Const { ty, .. } => verify_type_ref(ty)?,
                NirStmt::Print(_)
                | NirStmt::Await(_)
                | NirStmt::Expr(_)
                | NirStmt::Return(_)
                | NirStmt::If { .. }
                | NirStmt::While { .. }
                | NirStmt::Break
                | NirStmt::Continue => {}
            }
        }
    }
    Ok(())
}

fn verify_type_ref(ty: &NirTypeRef) -> Result<(), String> {
    ty.validate_container_contract()
        .map_err(|error| format!("nir verify: invalid type `{}`: {error}", ty.render()))
}

fn verify_function(function: &NirFunction) -> Result<(), String> {
    let mut moved = BTreeSet::<String>::new();
    let mut borrows = BTreeMap::<String, usize>::new();
    let mut borrow_bindings = BTreeMap::<String, String>::new();
    let mut data_bindings = BTreeMap::<String, NirDataKind>::new();
    let mut task_result_facts = BTreeMap::<String, TaskResultStateFact>::new();

    for stmt in &function.body {
        verify_stmt(
            stmt,
            &mut moved,
            &mut borrows,
            &mut borrow_bindings,
            &mut data_bindings,
            &mut task_result_facts,
        )?;
    }

    Ok(())
}

fn verify_stmt(
    stmt: &NirStmt,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    borrow_bindings: &mut BTreeMap<String, String>,
    data_bindings: &mut BTreeMap<String, NirDataKind>,
    task_result_facts: &mut BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            ensure_binding_can_be_rebound(name, borrows, borrow_bindings)?;
            borrows.remove(name);
            borrow_bindings.remove(name);
            data_bindings.remove(name);
            task_result_facts.remove(name);
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            note_binding_effects(value, name, moved, borrows, borrow_bindings);
            data_bindings.insert(
                name.clone(),
                infer_data_kind(value, data_bindings)
                    .merge_with_type_hint(ty.as_ref().map(infer_data_kind_from_type)),
            );
        }
        NirStmt::Const { name, ty, value } => {
            ensure_binding_can_be_rebound(name, borrows, borrow_bindings)?;
            borrows.remove(name);
            borrow_bindings.remove(name);
            data_bindings.remove(name);
            task_result_facts.remove(name);
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            note_binding_effects(value, name, moved, borrows, borrow_bindings);
            data_bindings.insert(
                name.clone(),
                infer_data_kind(value, data_bindings)
                    .merge_with_type_hint(Some(infer_data_kind_from_type(ty))),
            );
        }
        NirStmt::Print(value) | NirStmt::Await(value) | NirStmt::Expr(value) => {
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            note_binding_effects(value, "_", moved, borrows, borrow_bindings);
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            verify_expr(
                condition,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut then_moved = moved.clone();
            let mut then_borrows = borrows.clone();
            let mut then_borrow_bindings = borrow_bindings.clone();
            let mut then_data_bindings = data_bindings.clone();
            let mut then_task_result_facts = task_result_facts.clone();
            let mut else_task_result_facts = task_result_facts.clone();
            apply_task_result_condition_facts(
                condition,
                &mut then_task_result_facts,
                &mut else_task_result_facts,
            );
            for stmt in then_body {
                verify_stmt(
                    stmt,
                    &mut then_moved,
                    &mut then_borrows,
                    &mut then_borrow_bindings,
                    &mut then_data_bindings,
                    &mut then_task_result_facts,
                )?;
            }
            let mut else_moved = moved.clone();
            let mut else_borrows = borrows.clone();
            let mut else_borrow_bindings = borrow_bindings.clone();
            let mut else_data_bindings = data_bindings.clone();
            for stmt in else_body {
                verify_stmt(
                    stmt,
                    &mut else_moved,
                    &mut else_borrows,
                    &mut else_borrow_bindings,
                    &mut else_data_bindings,
                    &mut else_task_result_facts,
                )?;
            }
            merge_branch_state(
                moved,
                borrows,
                &then_moved,
                &then_borrows,
                &else_moved,
                &else_borrows,
            );
        }
        NirStmt::While { condition, body } => {
            verify_expr(
                condition,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            let mut loop_moved = moved.clone();
            let mut loop_borrows = borrows.clone();
            let mut loop_borrow_bindings = borrow_bindings.clone();
            let mut loop_data_bindings = data_bindings.clone();
            let mut loop_task_result_facts = task_result_facts.clone();
            for stmt in body {
                verify_stmt(
                    stmt,
                    &mut loop_moved,
                    &mut loop_borrows,
                    &mut loop_borrow_bindings,
                    &mut loop_data_bindings,
                    &mut loop_task_result_facts,
                )?;
            }
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                verify_expr(
                    value,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
    }
    Ok(())
}

fn verify_expr(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BTreeMap<String, String>,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<(), String> {
    verify_expr_uses(expr, moved)?;

    if let NirExpr::CpuTaskValue(inner) = expr {
        if let Some(source) = expr_resource_key(inner) {
            if matches!(
                task_result_facts.get(&source),
                Some(TaskResultStateFact::TimedOut | TaskResultStateFact::Cancelled)
            ) {
                return Err(format!(
                    "nir verify: cannot extract task_value from `{}` on a non-completed lifecycle path",
                    source
                ));
            }
        }
    }

    match expr {
        NirExpr::DataOutputPipe(inner) => {
            let source = infer_data_kind(inner, data_bindings);
            if matches!(source, NirDataKind::PipeOutput | NirDataKind::PipeInput) {
                return Err(format!(
                    "nir verify: data_output_pipe cannot wrap nested pipe `{}`",
                    render_data_expr_name(inner)
                ));
            }
        }
        NirExpr::DataInputPipe(inner) => {
            if infer_data_kind(inner, data_bindings) != NirDataKind::PipeOutput {
                return Err(format!(
                    "nir verify: data_input_pipe expects output pipe input, got `{}`",
                    render_data_expr_name(inner)
                ));
            }
        }
        NirExpr::DataCopyWindow { input, .. } | NirExpr::DataImmutableWindow { input, .. } => {
            let source = infer_data_kind(input, data_bindings);
            if matches!(
                source,
                NirDataKind::WindowMutable
                    | NirDataKind::WindowImmutable
                    | NirDataKind::PipeOutput
                    | NirDataKind::PipeInput
                    | NirDataKind::Marker
                    | NirDataKind::HandleTable
            ) {
                return Err(format!(
                    "nir verify: cannot create nested data window from `{}`",
                    render_data_expr_name(input)
                ));
            }
        }
        NirExpr::DataReadWindow { window, index } => {
            let source = infer_data_kind(window, data_bindings);
            if !matches!(
                source,
                NirDataKind::WindowMutable | NirDataKind::WindowImmutable
            ) {
                return Err(format!(
                    "nir verify: data_read_window expects window input, got `{}`",
                    render_data_expr_name(window)
                ));
            }
            let index_kind = infer_data_kind(index, data_bindings);
            if index_kind != NirDataKind::Other {
                return Err(format!(
                    "nir verify: data_read_window expects scalar index, got `{}`",
                    render_data_expr_name(index)
                ));
            }
        }
        NirExpr::DataWriteWindow { window, index, .. } => {
            let source = infer_data_kind(window, data_bindings);
            if source != NirDataKind::WindowMutable {
                return Err(format!(
                    "nir verify: data_write_window expects mutable window input, got `{}`",
                    render_data_expr_name(window)
                ));
            }
            let index_kind = infer_data_kind(index, data_bindings);
            if index_kind != NirDataKind::Other {
                return Err(format!(
                    "nir verify: data_write_window expects scalar index, got `{}`",
                    render_data_expr_name(index)
                ));
            }
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            let source = infer_data_kind(input, data_bindings);
            if matches!(source, NirDataKind::WindowMutable) {
                return Err(format!(
                    "nir verify: data_profile_send requires immutable window payload, got `{}`",
                    render_data_expr_name(input)
                ));
            }
            if matches!(
                source,
                NirDataKind::PipeOutput
                    | NirDataKind::PipeInput
                    | NirDataKind::Marker
                    | NirDataKind::HandleTable
            ) {
                return Err(format!(
                    "nir verify: data_profile_send cannot wrap illegal window payload `{}`",
                    render_data_expr_name(input)
                ));
            }
        }
        NirExpr::DataFreezeWindow(input) => {
            let source = infer_data_kind(input, data_bindings);
            if !matches!(
                source,
                NirDataKind::WindowMutable | NirDataKind::WindowImmutable
            ) {
                return Err(format!(
                    "nir verify: data_freeze_window expects window input, got `{}`",
                    render_data_expr_name(input)
                ));
            }
        }
        _ => {}
    }

    if let Some(profile) = nir_glm_profile(expr) {
        if let Some(first_access) = profile.accesses.first() {
            match expr {
                NirExpr::Move(inner)
                | NirExpr::Free(inner)
                | NirExpr::CpuJoin(inner)
                | NirExpr::CpuCancel(inner)
                | NirExpr::CpuJoinResult(inner) => {
                    if matches!(expr, NirExpr::Move(_))
                        && expr_is_borrowed_pointer(inner, borrow_bindings)
                    {
                        return Err(format!(
                            "nir verify: cannot move borrowed pointer `{}`",
                            render_data_expr_name(inner)
                        ));
                    }
                    if matches!(first_access.mode, NirGlmUseMode::Own) {
                        if let Some(source) = expr_resource_key(inner) {
                            if borrows.get(&source).copied().unwrap_or(0) > 0 {
                                return Err(format!(
                                    "nir verify: cannot consume `{}` while borrow(s) are active",
                                    source
                                ));
                            }
                        }
                    }
                }
                NirExpr::CpuTimeout { task, .. } => {
                    if let Some(source) = expr_resource_key(task) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot consume `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::StoreValue { target, .. } | NirExpr::StoreNext { target, .. } => {
                    if let Some(source) = expr_resource_key(target) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot write `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::StoreAt { buffer, .. } => {
                    if let Some(source) = expr_resource_key(buffer) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot write `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::Borrow(inner) => {
                    if let Some(source) = expr_resource_key(inner) {
                        if moved.contains(&source) {
                            return Err(format!(
                                "nir verify: cannot borrow moved value `{}`",
                                source
                            ));
                        }
                    }
                }
                NirExpr::BorrowEnd(inner) => {
                    let source = expr_resource_key(inner)
                        .and_then(|name| borrow_bindings.get(&name).cloned().or(Some(name)));
                    if let Some(source) = source {
                        if borrows.get(&source).copied().unwrap_or(0) == 0 {
                            return Err(format!(
                                "nir verify: cannot end borrow for `{}` with no active borrow",
                                source
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    match expr {
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::Instantiate { .. } => {}
        NirExpr::DataBindCore(_)
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
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
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
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuExternCall { args, .. } => {
            for arg in args {
                verify_expr(
                    arg,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            verify_expr(
                task,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                limit,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            verify_expr(
                target,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                pipeline,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                viewport,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            verify_expr(
                packet,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            verify_expr(
                base,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                delta,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            verify_expr(
                delta,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                scale,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                base,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            verify_expr(
                base,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                delta,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
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
            verify_expr(
                color,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                speed,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                radius,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let Some(accent) = accent {
                verify_expr(
                    accent,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
            if let Some(toggle_state) = toggle_state {
                verify_expr(
                    toggle_state,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
            if let Some(focus_index) = focus_index {
                verify_expr(
                    focus_index,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            verify_expr(
                pass,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                packet,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let NirExpr::ShaderDrawInstanced {
                vertex_count,
                instance_count,
                ..
            } = expr
            {
                verify_expr(
                    vertex_count,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
                verify_expr(
                    instance_count,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
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
        NirExpr::KernelMatmul { lhs, rhs } => {
            verify_expr(
                lhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                rhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelElementAt { input, row, col } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                row,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                col,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelReshape { input, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelBroadcast { input, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let Some(scalar) = scalar {
                verify_expr(
                    scalar,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let Some(scalar) = scalar {
                verify_expr(
                    scalar,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            verify_expr(
                lhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                rhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::KernelAddBias { input, bias } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                bias,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::DataFreezeWindow(inner) => verify_expr(
            inner,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::DataReadWindow { window, index } => {
            verify_expr(
                window,
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
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            verify_expr(
                window,
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
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => verify_expr(
            input,
            moved,
            borrows,
            borrow_bindings,
            data_bindings,
            task_result_facts,
        )?,
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            verify_expr(
                input,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                offset,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                len,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
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
            if expr_is_borrowed_pointer(next, borrow_bindings) {
                return Err(format!(
                    "nir verify: alloc_node cannot capture borrowed pointer `{}` as structural next link",
                    render_data_expr_name(next)
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
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
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
            verify_expr(
                next,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if expr_is_borrowed_pointer(next, borrow_bindings) {
                return Err(format!(
                    "nir verify: store_next cannot write borrowed pointer `{}` into structural next link",
                    render_data_expr_name(next)
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
            verify_expr(
                index,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                verify_expr(
                    arg,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            verify_expr(
                receiver,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            for arg in args {
                verify_expr(
                    arg,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                verify_expr(
                    value,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
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
            verify_expr(
                rhs,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
    }

    Ok(())
}

fn verify_expr_uses(expr: &NirExpr, moved: &BTreeSet<String>) -> Result<(), String> {
    match expr {
        NirExpr::Var(_) | NirExpr::FieldAccess { .. } => {
            if let Some(name) = expr_resource_key(expr) {
                if moved.contains(&name) {
                    return Err(format!("nir verify: use of moved value `{}`", name));
                }
            }
        }
        NirExpr::Instantiate { .. } => {}
        NirExpr::CastI64ToI32(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::DataBindCore(_)
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
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
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
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuExternCall { args, .. } => {
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
        NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Null => {}
    }
    Ok(())
}

fn expr_resource_key(expr: &NirExpr) -> Option<String> {
    match expr {
        NirExpr::Var(name) => Some(name.clone()),
        NirExpr::FieldAccess { base, field } => {
            let base = expr_resource_key(base)?;
            Some(format!("{base}.{field}"))
        }
        _ => None,
    }
}

fn infer_data_kind(expr: &NirExpr, data_bindings: &BTreeMap<String, NirDataKind>) -> NirDataKind {
    match expr {
        NirExpr::Await(inner) => infer_data_kind(inner, data_bindings),
        NirExpr::DataResult { value, .. } => infer_data_kind(value, data_bindings),
        NirExpr::DataValue(inner) => infer_data_kind(inner, data_bindings),
        NirExpr::Var(name) => data_bindings
            .get(name)
            .copied()
            .unwrap_or(NirDataKind::Other),
        NirExpr::DataMarker(_) | NirExpr::DataProfileMarkerRef { .. } => NirDataKind::Marker,
        NirExpr::DataHandleTable(_) | NirExpr::DataProfileHandleTableRef { .. } => {
            NirDataKind::HandleTable
        }
        NirExpr::DataOutputPipe(_) => NirDataKind::PipeOutput,
        NirExpr::DataInputPipe(_) => NirDataKind::PipeInput,
        NirExpr::DataCopyWindow { .. } => NirDataKind::WindowMutable,
        NirExpr::DataWriteWindow { .. } => NirDataKind::WindowMutable,
        NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. } => NirDataKind::WindowImmutable,
        _ => NirDataKind::Other,
    }
}

fn infer_data_kind_from_type(ty: &NirTypeRef) -> NirDataKind {
    if let Some(mode) = ty.window_mode() {
        return match mode {
            nuis_semantics::model::NirWindowMode::Mutable => NirDataKind::WindowMutable,
            nuis_semantics::model::NirWindowMode::Immutable => NirDataKind::WindowImmutable,
        };
    }
    if ty.name == "Pipe" && ty.generic_args.len() == 1 {
        return NirDataKind::PipeOutput;
    }
    if ty.is_marker_type() {
        return NirDataKind::Marker;
    }
    if ty.is_handle_table_type() {
        return NirDataKind::HandleTable;
    }
    if matches!(
        ty.result_family(),
        Some(nuis_semantics::model::NirResultFamily::Data)
    ) && ty.generic_args.len() == 1
    {
        return infer_data_kind_from_type(&ty.generic_args[0]);
    }
    NirDataKind::Other
}

fn render_data_expr_name(expr: &NirExpr) -> String {
    match expr {
        NirExpr::Await(inner) => render_data_expr_name(inner),
        NirExpr::Var(name) => name.clone(),
        NirExpr::DataMarker(tag) => format!("marker:{tag}"),
        NirExpr::DataProfileMarkerRef { unit, tag } => format!("{unit}.marker:{tag}"),
        NirExpr::DataHandleTable(_) => "handle_table".to_owned(),
        NirExpr::DataProfileHandleTableRef { unit } => format!("{unit}.handle_table"),
        NirExpr::DataOutputPipe(_) => "output_pipe".to_owned(),
        NirExpr::DataInputPipe(_) => "input_pipe".to_owned(),
        NirExpr::DataCopyWindow { .. } => "copy_window".to_owned(),
        NirExpr::DataReadWindow { .. } => "read_window".to_owned(),
        NirExpr::DataWriteWindow { .. } => "write_window".to_owned(),
        NirExpr::DataImmutableWindow { .. } => "immutable_window".to_owned(),
        NirExpr::DataFreezeWindow(_) => "freeze_window".to_owned(),
        NirExpr::DataProfileSendUplink { .. } => "profile_send_uplink".to_owned(),
        NirExpr::DataProfileSendDownlink { .. } => "profile_send_downlink".to_owned(),
        _ => "value".to_owned(),
    }
}

#[cfg(test)]
mod tests;
