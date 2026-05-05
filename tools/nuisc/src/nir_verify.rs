use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    nir_glm_profile, NirExpr, NirFunction, NirGlmUseMode, NirModule, NirStmt, NirTypeRef,
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
                | NirStmt::If { .. } => {}
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

    for stmt in &function.body {
        verify_stmt(
            stmt,
            &mut moved,
            &mut borrows,
            &mut borrow_bindings,
            &mut data_bindings,
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
) -> Result<(), String> {
    match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            borrows.remove(name);
            borrow_bindings.remove(name);
            data_bindings.remove(name);
            verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
            note_binding_effects(value, name, moved, borrows, borrow_bindings);
            data_bindings.insert(name.clone(), infer_data_kind(value, data_bindings));
        }
        NirStmt::Print(value) | NirStmt::Await(value) | NirStmt::Expr(value) => {
            verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
            if let NirExpr::BorrowEnd(_) = value {
                note_binding_effects(value, "_", moved, borrows, borrow_bindings);
            }
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            verify_expr(condition, moved, borrows, borrow_bindings, data_bindings)?;
            let mut then_moved = moved.clone();
            let mut then_borrows = borrows.clone();
            let mut then_borrow_bindings = borrow_bindings.clone();
            let mut then_data_bindings = data_bindings.clone();
            for stmt in then_body {
                verify_stmt(
                    stmt,
                    &mut then_moved,
                    &mut then_borrows,
                    &mut then_borrow_bindings,
                    &mut then_data_bindings,
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
                )?;
            }
            merge_branch_state(moved, borrows, &then_moved, &then_borrows, &else_moved, &else_borrows);
        }
        NirStmt::Return(value) => {
            if let Some(value) = value {
                verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
            }
        }
    }
    Ok(())
}

fn merge_branch_state(
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    then_moved: &BTreeSet<String>,
    then_borrows: &BTreeMap<String, usize>,
    else_moved: &BTreeSet<String>,
    else_borrows: &BTreeMap<String, usize>,
) {
    moved.extend(then_moved.iter().cloned());
    moved.extend(else_moved.iter().cloned());

    let mut merged_borrows = BTreeMap::<String, usize>::new();
    for name in then_borrows.keys().chain(else_borrows.keys()) {
        let then_count = then_borrows.get(name).copied().unwrap_or(0);
        let else_count = else_borrows.get(name).copied().unwrap_or(0);
        let merged = then_count.max(else_count);
        if merged > 0 {
            merged_borrows.insert(name.clone(), merged);
        }
    }

    *borrows = merged_borrows;
}

fn note_binding_effects(
    expr: &NirExpr,
    binding_name: &str,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
    borrow_bindings: &mut BTreeMap<String, String>,
) {
    match expr {
        NirExpr::Move(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                moved.insert(source.clone());
                borrows.remove(&source);
            }
        }
        NirExpr::Borrow(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                *borrows.entry(source.clone()).or_insert(0) += 1;
                if binding_name != "_" {
                    borrow_bindings.insert(binding_name.to_owned(), source);
                }
            }
        }
        NirExpr::BorrowEnd(inner) => {
            let source = expr_resource_key(inner)
                .and_then(|name| borrow_bindings.get(&name).cloned().or(Some(name)));
            if let Some(source) = source {
                let next = borrows.get(&source).copied().unwrap_or(0).saturating_sub(1);
                if next == 0 {
                    borrows.remove(&source);
                } else {
                    borrows.insert(source.clone(), next);
                }
                if binding_name != "_" {
                    borrow_bindings.remove(binding_name);
                }
            }
        }
        _ => {}
    }
}

fn verify_expr(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BTreeMap<String, String>,
    data_bindings: &BTreeMap<String, NirDataKind>,
) -> Result<(), String> {
    verify_expr_uses(expr, moved)?;

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
        _ => {}
    }

    if let Some(profile) = nir_glm_profile(expr) {
        if let Some(first_access) = profile.accesses.first() {
            match expr {
                NirExpr::Move(inner) | NirExpr::Free(inner) => {
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
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
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
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner) => {
            verify_expr(inner, moved, borrows, borrow_bindings, data_bindings)?
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuExternCall { args, .. } => {
            for arg in args {
                verify_expr(arg, moved, borrows, borrow_bindings, data_bindings)?;
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            verify_expr(task, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(limit, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            verify_expr(target, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(pipeline, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(viewport, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            verify_expr(packet, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            verify_expr(base, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(delta, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            verify_expr(delta, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(scale, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(base, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            verify_expr(base, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(delta, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            verify_expr(color, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(speed, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(radius, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            verify_expr(pass, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(packet, moved, borrows, borrow_bindings, data_bindings)?;
            if let NirExpr::ShaderDrawInstanced {
                vertex_count,
                instance_count,
                ..
            } = expr
            {
                verify_expr(vertex_count, moved, borrows, borrow_bindings, data_bindings)?;
                verify_expr(
                    instance_count,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                )?;
            }
        }
        NirExpr::DataOutputPipe(inner) | NirExpr::DataInputPipe(inner) => {
            verify_expr(inner, moved, borrows, borrow_bindings, data_bindings)?
        }
        NirExpr::DataResult { value: inner, .. }
        | NirExpr::ShaderResult { value: inner, .. }
        | NirExpr::KernelResult { value: inner, .. } => {
            verify_expr(inner, moved, borrows, borrow_bindings, data_bindings)?
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            verify_expr(input, moved, borrows, borrow_bindings, data_bindings)?
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            verify_expr(input, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(offset, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(len, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => {
            verify_expr(inner, moved, borrows, borrow_bindings, data_bindings)?
        }
        NirExpr::AllocNode { value, next } => {
            verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(next, moved, borrows, borrow_bindings, data_bindings)?;
            if expr_is_borrowed_pointer(next, borrow_bindings) {
                return Err(format!(
                    "nir verify: alloc_node cannot capture borrowed pointer `{}` as structural next link",
                    render_data_expr_name(next)
                ));
            }
        }
        NirExpr::AllocBuffer { len, fill } => {
            verify_expr(len, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(fill, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::LoadAt { buffer, index } => {
            verify_expr(buffer, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(index, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::StoreValue { target, value } => {
            verify_expr(target, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::StoreNext { target, next } => {
            verify_expr(target, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(next, moved, borrows, borrow_bindings, data_bindings)?;
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
            verify_expr(buffer, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(index, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                verify_expr(arg, moved, borrows, borrow_bindings, data_bindings)?;
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            verify_expr(receiver, moved, borrows, borrow_bindings, data_bindings)?;
            for arg in args {
                verify_expr(arg, moved, borrows, borrow_bindings, data_bindings)?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                verify_expr(value, moved, borrows, borrow_bindings, data_bindings)?;
            }
        }
        NirExpr::FieldAccess { base, .. } => {
            verify_expr(base, moved, borrows, borrow_bindings, data_bindings)?
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            verify_expr(lhs, moved, borrows, borrow_bindings, data_bindings)?;
            verify_expr(rhs, moved, borrows, borrow_bindings, data_bindings)?;
        }
    }

    Ok(())
}

fn expr_is_borrowed_pointer(
    expr: &NirExpr,
    borrow_bindings: &BTreeMap<String, String>,
) -> bool {
    match expr {
        NirExpr::Borrow(_) => true,
        NirExpr::Var(name) => borrow_bindings.contains_key(name),
        NirExpr::Await(inner) => expr_is_borrowed_pointer(inner, borrow_bindings),
        _ => false,
    }
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
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
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
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner) => {
            verify_expr_uses(inner, moved)?
        }
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
            ..
        } => {
            verify_expr_uses(color, moved)?;
            verify_expr_uses(speed, moved)?;
            verify_expr_uses(radius, moved)?;
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
        | NirExpr::KernelResult { value: inner, .. } => {
            verify_expr_uses(inner, moved)?
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
        NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. } => NirDataKind::WindowImmutable,
        _ => NirDataKind::Other,
    }
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
        NirExpr::DataImmutableWindow { .. } => "immutable_window".to_owned(),
        NirExpr::DataProfileSendUplink { .. } => "profile_send_uplink".to_owned(),
        NirExpr::DataProfileSendDownlink { .. } => "profile_send_downlink".to_owned(),
        _ => "value".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::verify_nir_module;
    use nuis_semantics::model::{NirExpr, NirFunction, NirModule, NirStmt};

    fn module_with_body(body: Vec<NirStmt>) -> NirModule {
        NirModule {
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            structs: vec![],
            functions: vec![NirFunction {
                name: "main".to_owned(),
                is_async: false,
                params: vec![],
                return_type: None,
                body,
            }],
        }
    }

    #[test]
    fn explicit_borrow_end_allows_owner_write() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            },
            NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                "head_ref".to_owned(),
            )))),
            NirStmt::Expr(NirExpr::StoreValue {
                target: Box::new(NirExpr::Var("head".to_owned())),
                value: Box::new(NirExpr::Int(77)),
            }),
        ]);

        verify_nir_module(&module).unwrap();
    }

    #[test]
    fn owner_write_while_borrowed_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            },
            NirStmt::Expr(NirExpr::StoreValue {
                target: Box::new(NirExpr::Var("head".to_owned())),
                value: Box::new(NirExpr::Int(77)),
            }),
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("cannot write `head` while borrow(s) are active"));
    }

    #[test]
    fn owner_write_after_conditional_borrow_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "cond".to_owned(),
                ty: None,
                value: NirExpr::Bool(true),
            },
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::If {
                condition: NirExpr::Var("cond".to_owned()),
                then_body: vec![NirStmt::Let {
                    name: "head_ref".to_owned(),
                    ty: None,
                    value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
                }],
                else_body: vec![],
            },
            NirStmt::Expr(NirExpr::StoreValue {
                target: Box::new(NirExpr::Var("head".to_owned())),
                value: Box::new(NirExpr::Int(77)),
            }),
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("cannot write `head` while borrow(s) are active"));
    }

    #[test]
    fn owner_use_after_conditional_move_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "cond".to_owned(),
                ty: None,
                value: NirExpr::Bool(true),
            },
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::If {
                condition: NirExpr::Var("cond".to_owned()),
                then_body: vec![NirStmt::Let {
                    name: "taken".to_owned(),
                    ty: None,
                    value: NirExpr::Move(Box::new(NirExpr::Var("head".to_owned()))),
                }],
                else_body: vec![],
            },
            NirStmt::Expr(NirExpr::LoadValue(Box::new(NirExpr::Var("head".to_owned())))),
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("use of moved value `head`"));
    }

    #[test]
    fn owner_write_after_branch_ended_borrow_is_allowed() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "cond".to_owned(),
                ty: None,
                value: NirExpr::Bool(true),
            },
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            },
            NirStmt::If {
                condition: NirExpr::Var("cond".to_owned()),
                then_body: vec![NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                    "head_ref".to_owned(),
                ))))],
                else_body: vec![NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                    "head_ref".to_owned(),
                ))))],
            },
            NirStmt::Expr(NirExpr::StoreValue {
                target: Box::new(NirExpr::Var("head".to_owned())),
                value: Box::new(NirExpr::Int(77)),
            }),
        ]);

        verify_nir_module(&module).unwrap();
    }

    #[test]
    fn move_of_borrowed_pointer_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            },
            NirStmt::Let {
                name: "taken".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::Var("head_ref".to_owned()))),
            },
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("cannot move borrowed pointer"));
    }

    #[test]
    fn alloc_node_with_borrowed_next_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "tail".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(30)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "tail_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("tail".to_owned()))),
            },
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Var("tail_ref".to_owned())),
                },
            },
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("alloc_node cannot capture borrowed pointer"));
    }

    #[test]
    fn store_next_with_borrowed_pointer_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "tail".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(30)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Let {
                name: "tail_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("tail".to_owned()))),
            },
            NirStmt::Expr(NirExpr::StoreNext {
                target: Box::new(NirExpr::Var("head".to_owned())),
                next: Box::new(NirExpr::Var("tail_ref".to_owned())),
            }),
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("store_next cannot write borrowed pointer"));
    }

    #[test]
    fn borrow_end_without_active_borrow_is_rejected() {
        let module = module_with_body(vec![
            NirStmt::Let {
                name: "head".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                "head".to_owned(),
            )))),
        ]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("cannot end borrow"));
    }

    #[test]
    fn data_input_pipe_requires_output_pipe_source() {
        let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataInputPipe(Box::new(
            NirExpr::Int(7),
        )))]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("data_input_pipe expects output pipe input"));
    }

    #[test]
    fn data_output_pipe_rejects_nested_pipe() {
        let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataOutputPipe(Box::new(
            NirExpr::DataOutputPipe(Box::new(NirExpr::Int(7))),
        )))]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("data_output_pipe cannot wrap nested pipe"));
    }

    #[test]
    fn data_window_rejects_marker_source() {
        let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataCopyWindow {
            input: Box::new(NirExpr::DataMarker("ready".to_owned())),
            offset: Box::new(NirExpr::Int(0)),
            len: Box::new(NirExpr::Int(1)),
        })]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("cannot create nested data window"));
    }

    #[test]
    fn data_window_rejects_nested_window_source() {
        let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataCopyWindow {
            input: Box::new(NirExpr::DataImmutableWindow {
                input: Box::new(NirExpr::Int(7)),
                offset: Box::new(NirExpr::Int(0)),
                len: Box::new(NirExpr::Int(1)),
            }),
            offset: Box::new(NirExpr::Int(0)),
            len: Box::new(NirExpr::Int(1)),
        })]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("cannot create nested data window"));
    }

    #[test]
    fn data_profile_send_rejects_handle_table_source() {
        let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataProfileSendUplink {
            unit: "FabricPlane".to_owned(),
            input: Box::new(NirExpr::DataHandleTable(vec![(
                "host".to_owned(),
                "cpu0".to_owned(),
            )])),
        })]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("data_profile_send cannot wrap illegal window payload"));
    }

    #[test]
    fn data_profile_send_rejects_mutable_window_source() {
        let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataProfileSendUplink {
            unit: "FabricPlane".to_owned(),
            input: Box::new(NirExpr::DataCopyWindow {
                input: Box::new(NirExpr::Int(7)),
                offset: Box::new(NirExpr::Int(0)),
                len: Box::new(NirExpr::Int(1)),
            }),
        })]);

        let error = verify_nir_module(&module).unwrap_err();
        assert!(error.contains("requires immutable window payload"));
    }
}
