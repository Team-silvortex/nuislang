use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DirectCallScalarKind {
    Bool,
    I32,
    I64,
    F32,
    F64,
    BorrowedBuffer,
    TraversalPointer,
    OwnedBytes,
}

pub(super) fn collect_recursive_direct_call_functions(module: &NirModule) -> BTreeSet<String> {
    collect_recursive_helper_functions(module, false)
}

pub(super) fn collect_recursive_async_helper_functions(module: &NirModule) -> BTreeSet<String> {
    collect_recursive_helper_functions(module, true)
}

pub(super) fn collect_scheduler_async_thunk_functions(module: &NirModule) -> BTreeSet<String> {
    let eligible = module
        .functions
        .iter()
        .filter(|function| function.is_async)
        .filter(|function| {
            let params_supported = function.params.iter().all(|param| {
                direct_call_scalar_kind(&param.ty).is_some_and(is_scheduler_scalar_kind)
            });
            let scalar_return = function
                .return_type
                .as_ref()
                .and_then(direct_call_scalar_kind)
                .is_some_and(is_scheduler_scalar_kind);
            let struct_return = function
                .return_type
                .as_ref()
                .and_then(|ty| module_owned_struct_layout(module, ty))
                .is_some();
            params_supported && (scalar_return || struct_return)
        })
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut spawned = BTreeSet::new();
    for function in &module.functions {
        collect_scheduler_spawned_functions_in_stmts(&function.body, &eligible, &mut spawned);
    }
    spawned
}

fn collect_scheduler_spawned_functions_in_stmts(
    body: &[NirStmt],
    eligible: &BTreeSet<&str>,
    spawned: &mut BTreeSet<String>,
) {
    for stmt in body {
        match stmt {
            NirStmt::Let { value, .. }
            | NirStmt::Const { value, .. }
            | NirStmt::Print(value)
            | NirStmt::Expr(value)
            | NirStmt::Await(value)
            | NirStmt::Return(Some(value)) => {
                collect_scheduler_spawned_functions_in_expr(value, eligible, spawned);
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_scheduler_spawned_functions_in_expr(condition, eligible, spawned);
                collect_scheduler_spawned_functions_in_stmts(then_body, eligible, spawned);
                collect_scheduler_spawned_functions_in_stmts(else_body, eligible, spawned);
            }
            NirStmt::While { condition, body } => {
                collect_scheduler_spawned_functions_in_expr(condition, eligible, spawned);
                collect_scheduler_spawned_functions_in_stmts(body, eligible, spawned);
            }
            NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
        }
    }
}

fn collect_scheduler_spawned_functions_in_expr(
    expr: &NirExpr,
    eligible: &BTreeSet<&str>,
    spawned: &mut BTreeSet<String>,
) {
    match expr {
        NirExpr::CpuSpawn { callee, args } => {
            if eligible.contains(callee.as_str()) {
                spawned.insert(callee.clone());
            }
            for arg in args {
                collect_scheduler_spawned_functions_in_expr(arg, eligible, spawned);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            collect_scheduler_spawned_functions_in_expr(task, eligible, spawned);
            collect_scheduler_spawned_functions_in_expr(limit, eligible, spawned);
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            collect_scheduler_spawned_functions_in_expr(task, eligible, spawned);
            collect_scheduler_spawned_functions_in_expr(delay, eligible, spawned);
        }
        NirExpr::Await(value)
        | NirExpr::CpuJoin(value)
        | NirExpr::CpuCancel(value)
        | NirExpr::CpuJoinResult(value)
        | NirExpr::CpuTaskCompleted(value)
        | NirExpr::CpuTaskTimedOut(value)
        | NirExpr::CpuTaskCancelled(value)
        | NirExpr::CpuTaskFailed(value)
        | NirExpr::CpuTaskValue(value) => {
            collect_scheduler_spawned_functions_in_expr(value, eligible, spawned);
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                collect_scheduler_spawned_functions_in_expr(arg, eligible, spawned);
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            collect_scheduler_spawned_functions_in_expr(receiver, eligible, spawned);
            for arg in args {
                collect_scheduler_spawned_functions_in_expr(arg, eligible, spawned);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_scheduler_spawned_functions_in_expr(lhs, eligible, spawned);
            collect_scheduler_spawned_functions_in_expr(rhs, eligible, spawned);
        }
        NirExpr::FieldAccess { base, .. } => {
            collect_scheduler_spawned_functions_in_expr(base, eligible, spawned);
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_scheduler_spawned_functions_in_expr(value, eligible, spawned);
            }
        }
        _ => {}
    }
}

pub(super) fn collect_async_loop_step_functions(module: &NirModule) -> BTreeSet<String> {
    let eligible = module
        .functions
        .iter()
        .filter(|function| function.is_async)
        .filter(|function| direct_call_signature_kind(function).is_some())
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut collected = BTreeSet::new();
    for function in &module.functions {
        collect_async_loop_step_functions_in_stmts(&function.body, &eligible, &mut collected);
    }
    collected
}

fn collect_recursive_helper_functions(module: &NirModule, is_async: bool) -> BTreeSet<String> {
    let eligible = module
        .functions
        .iter()
        .filter(|function| function.is_async == is_async)
        .filter(|function| direct_call_signature_kind(function).is_some())
        .collect::<Vec<_>>();
    let eligible_names = eligible
        .iter()
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let call_graph = eligible
        .iter()
        .map(|function| {
            (
                function.name.clone(),
                function_called_functions(function, &function.body, &eligible_names),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut recursive = BTreeSet::new();
    for function in &eligible {
        if participates_in_recursive_component(&function.name, &call_graph) {
            recursive.insert(function.name.clone());
        }
    }
    let mut closure = recursive.clone();
    let mut frontier = recursive.into_iter().collect::<Vec<_>>();
    while let Some(current) = frontier.pop() {
        let Some(neighbors) = call_graph.get(&current) else {
            continue;
        };
        for neighbor in neighbors {
            if closure.insert(neighbor.clone()) {
                frontier.push(neighbor.clone());
            }
        }
    }
    closure
}

fn collect_async_loop_step_functions_in_stmts(
    body: &[NirStmt],
    eligible_names: &BTreeSet<&str>,
    collected: &mut BTreeSet<String>,
) {
    for stmt in body {
        match stmt {
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_async_loop_step_functions_in_stmts(then_body, eligible_names, collected);
                collect_async_loop_step_functions_in_stmts(else_body, eligible_names, collected);
            }
            NirStmt::While { body, .. } => {
                collect_async_loop_step_function_in_while(body, eligible_names, collected);
                collect_async_loop_step_functions_in_stmts(body, eligible_names, collected);
            }
            _ => {}
        }
    }
}

fn collect_async_loop_step_function_in_while(
    body: &[NirStmt],
    eligible_names: &BTreeSet<&str>,
    collected: &mut BTreeSet<String>,
) {
    let Some(step_stmt) = body.first() else {
        return;
    };
    let value = match step_stmt {
        NirStmt::Let { value, .. } | NirStmt::Const { value, .. } => value,
        _ => return,
    };
    let NirExpr::Await(inner) = value else {
        return;
    };
    let NirExpr::Call { callee, args } = inner.as_ref() else {
        return;
    };
    if eligible_names.contains(callee.as_str()) && matches!(args.as_slice(), [NirExpr::Var(_)]) {
        collected.insert(callee.clone());
    }
}

fn direct_call_scalar_kind(ty: &nuis_semantics::model::NirTypeRef) -> Option<DirectCallScalarKind> {
    if ty.is_optional || !ty.generic_args.is_empty() {
        return None;
    }
    if ty.is_ref {
        return match ty.name.as_str() {
            "Buffer" => Some(DirectCallScalarKind::BorrowedBuffer),
            "Node" => Some(DirectCallScalarKind::TraversalPointer),
            _ => None,
        };
    }
    if ty.name == "Bytes" {
        return Some(DirectCallScalarKind::OwnedBytes);
    }
    if ty.is_bool_scalar() {
        Some(DirectCallScalarKind::Bool)
    } else if ty.name == "i32" {
        Some(DirectCallScalarKind::I32)
    } else if ty.name == "i64" {
        Some(DirectCallScalarKind::I64)
    } else if ty.name == "f32" {
        Some(DirectCallScalarKind::F32)
    } else if ty.name == "f64" {
        Some(DirectCallScalarKind::F64)
    } else {
        None
    }
}

fn is_scheduler_scalar_kind(kind: DirectCallScalarKind) -> bool {
    matches!(
        kind,
        DirectCallScalarKind::Bool
            | DirectCallScalarKind::I32
            | DirectCallScalarKind::I64
            | DirectCallScalarKind::F32
            | DirectCallScalarKind::F64
    )
}

pub(super) fn supports_direct_call_signature(function: &NirFunction) -> bool {
    direct_call_signature_kind(function).is_some()
}

fn direct_call_signature_kind(function: &NirFunction) -> Option<DirectCallScalarKind> {
    let return_kind = direct_call_scalar_kind(function.return_type.as_ref()?)?;
    if matches!(
        return_kind,
        DirectCallScalarKind::BorrowedBuffer | DirectCallScalarKind::TraversalPointer
    ) {
        return None;
    }
    for param in &function.params {
        direct_call_scalar_kind(&param.ty)?;
    }
    Some(return_kind)
}

fn function_called_functions(
    _function: &NirFunction,
    body: &[NirStmt],
    eligible_names: &BTreeSet<&str>,
) -> BTreeSet<String> {
    let mut called = BTreeSet::new();
    for stmt in body {
        stmt_collect_called_functions(_function, stmt, eligible_names, &mut called);
    }
    called
}

fn stmt_collect_called_functions(
    _function: &NirFunction,
    stmt: &NirStmt,
    eligible_names: &BTreeSet<&str>,
    called: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value)
        | NirStmt::Await(value) => expr_collect_called_functions(value, eligible_names, called),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_collect_called_functions(condition, eligible_names, called);
            for stmt in then_body {
                stmt_collect_called_functions(_function, stmt, eligible_names, called);
            }
            for stmt in else_body {
                stmt_collect_called_functions(_function, stmt, eligible_names, called);
            }
        }
        NirStmt::While { condition, body } => {
            expr_collect_called_functions(condition, eligible_names, called);
            for stmt in body {
                stmt_collect_called_functions(_function, stmt, eligible_names, called);
            }
        }
        NirStmt::Return(Some(value)) => {
            expr_collect_called_functions(value, eligible_names, called);
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
    }
}

fn expr_collect_called_functions(
    expr: &NirExpr,
    eligible_names: &BTreeSet<&str>,
    called: &mut BTreeSet<String>,
) {
    match expr {
        NirExpr::Call { callee, args } => {
            if eligible_names.contains(callee.as_str()) {
                called.insert(callee.clone());
            }
            for arg in args {
                expr_collect_called_functions(arg, eligible_names, called);
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_collect_called_functions(receiver, eligible_names, called);
            for arg in args {
                expr_collect_called_functions(arg, eligible_names, called);
            }
        }
        NirExpr::FieldAccess { base, .. } => {
            expr_collect_called_functions(base, eligible_names, called);
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_collect_called_functions(lhs, eligible_names, called);
            expr_collect_called_functions(rhs, eligible_names, called);
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .for_each(|(_, value)| expr_collect_called_functions(value, eligible_names, called)),
        NirExpr::Await(value)
        | NirExpr::DataOutputPipe(value)
        | NirExpr::DataInputPipe(value)
        | NirExpr::DataReady(value)
        | NirExpr::DataMoved(value)
        | NirExpr::DataWindowed(value)
        | NirExpr::DataValue(value)
        | NirExpr::DataFreezeWindow(value)
        | NirExpr::CpuJoin(value)
        | NirExpr::CpuThreadJoin(value)
        | NirExpr::CpuCancel(value)
        | NirExpr::CpuJoinResult(value)
        | NirExpr::CpuThreadJoinResult(value)
        | NirExpr::CpuTaskCompleted(value)
        | NirExpr::CpuTaskTimedOut(value)
        | NirExpr::CpuTaskCancelled(value)
        | NirExpr::CpuTaskFailed(value)
        | NirExpr::CpuTaskValue(value)
        | NirExpr::CpuMutexNew(value)
        | NirExpr::CpuMutexLock(value)
        | NirExpr::CpuMutexUnlock(value)
        | NirExpr::CpuMutexValue(value)
        | NirExpr::CpuPresentFrame(value)
        | NirExpr::Free(value)
        | NirExpr::IsNull(value)
        | NirExpr::KernelConfigReady(value)
        | NirExpr::KernelValue(value)
        | NirExpr::KernelShape(value)
        | NirExpr::KernelRows(value)
        | NirExpr::KernelCols(value)
        | NirExpr::KernelRow(value)
        | NirExpr::KernelCol(value)
        | NirExpr::KernelRelu(value)
        | NirExpr::KernelReduceSum(value)
        | NirExpr::KernelReduceMax(value)
        | NirExpr::KernelReduceMean(value)
        | NirExpr::KernelArgmax(value)
        | NirExpr::KernelArgmin(value)
        | NirExpr::KernelSort(value)
        | NirExpr::ShaderPassReady(value)
        | NirExpr::ShaderFrameReady(value)
        | NirExpr::ShaderValue(value)
        | NirExpr::NetworkConfigReady(value)
        | NirExpr::NetworkSendReady(value)
        | NirExpr::NetworkRecvReady(value)
        | NirExpr::NetworkAcceptReady(value)
        | NirExpr::NetworkValue(value)
        | NirExpr::DataResult { value, .. }
        | NirExpr::KernelResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. } => {
            expr_collect_called_functions(value, eligible_names, called);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            expr_collect_called_functions(input, eligible_names, called);
            expr_collect_called_functions(offset, eligible_names, called);
            expr_collect_called_functions(len, eligible_names, called);
        }
        NirExpr::KernelReshape { input, .. }
        | NirExpr::KernelBroadcast { input, .. }
        | NirExpr::KernelReduceSumAxis { input, .. }
        | NirExpr::KernelReduceMaxAxis { input, .. }
        | NirExpr::KernelReduceMeanAxis { input, .. }
        | NirExpr::KernelArgmaxAxis { input, .. }
        | NirExpr::KernelArgminAxis { input, .. }
        | NirExpr::KernelSortAxis { input, .. }
        | NirExpr::KernelTopk { input, .. }
        | NirExpr::KernelTopkAxis { input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. } => {
            expr_collect_called_functions(input, eligible_names, called);
        }
        NirExpr::DataReadWindow { window, index } => {
            expr_collect_called_functions(window, eligible_names, called);
            expr_collect_called_functions(index, eligible_names, called);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            expr_collect_called_functions(window, eligible_names, called);
            expr_collect_called_functions(index, eligible_names, called);
            expr_collect_called_functions(value, eligible_names, called);
        }
        NirExpr::KernelElementAt { input, row, col } => {
            expr_collect_called_functions(input, eligible_names, called);
            expr_collect_called_functions(row, eligible_names, called);
            expr_collect_called_functions(col, eligible_names, called);
        }
        NirExpr::KernelMap { input, scalar, .. } | NirExpr::KernelMapAxis { input, scalar, .. } => {
            expr_collect_called_functions(input, eligible_names, called);
            if let Some(value) = scalar.as_deref() {
                expr_collect_called_functions(value, eligible_names, called);
            }
        }
        NirExpr::KernelZip { lhs, rhs, .. } | NirExpr::KernelMatmul { lhs, rhs } => {
            expr_collect_called_functions(lhs, eligible_names, called);
            expr_collect_called_functions(rhs, eligible_names, called);
        }
        NirExpr::KernelAddBias { input, bias } => {
            expr_collect_called_functions(input, eligible_names, called);
            expr_collect_called_functions(bias, eligible_names, called);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            expr_collect_called_functions(target, eligible_names, called);
            expr_collect_called_functions(pipeline, eligible_names, called);
            expr_collect_called_functions(viewport, eligible_names, called);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            expr_collect_called_functions(pass, eligible_names, called);
            expr_collect_called_functions(packet, eligible_names, called);
            expr_collect_called_functions(vertex_count, eligible_names, called);
            expr_collect_called_functions(instance_count, eligible_names, called);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            expr_collect_called_functions(input, eligible_names, called);
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuThreadSpawn { args, .. } => {
            for arg in args {
                expr_collect_called_functions(arg, eligible_names, called);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            expr_collect_called_functions(task, eligible_names, called);
            expr_collect_called_functions(limit, eligible_names, called);
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            expr_collect_called_functions(task, eligible_names, called);
            expr_collect_called_functions(delay, eligible_names, called);
        }
        NirExpr::CpuExternCall { args, .. } => {
            for arg in args {
                expr_collect_called_functions(arg, eligible_names, called);
            }
        }
        NirExpr::DataHandleTable(_)
        | NirExpr::KernelTensor { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
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
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. } => {}
        _ => {}
    }
}

fn participates_in_recursive_component(
    start: &str,
    call_graph: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    let Some(neighbors) = call_graph.get(start) else {
        return false;
    };
    if neighbors.contains(start) {
        return true;
    }
    neighbors
        .iter()
        .any(|neighbor| path_reaches(neighbor, start, call_graph, &mut BTreeSet::new()))
}

fn path_reaches(
    current: &str,
    target: &str,
    call_graph: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
) -> bool {
    if current == target {
        return true;
    }
    if !visited.insert(current.to_owned()) {
        return false;
    }
    call_graph
        .get(current)
        .into_iter()
        .flatten()
        .any(|next| path_reaches(next, target, call_graph, visited))
}

pub(super) fn lower_direct_call_helper_function(
    function: &NirFunction,
    state: &mut LoweringState<'_>,
) -> Result<(), String> {
    let start_index = state.yir.nodes.len();
    let lane = format!("fn:{}", function.name);
    let mut bindings = BTreeMap::<String, String>::new();
    for (index, param) in function.params.iter().enumerate() {
        let node_name = format!("__fn_{}_param_{}", function.name, index);
        let instruction = match direct_call_scalar_kind(&param.ty).ok_or_else(|| {
            format!(
                "ordinary direct-call lowering only supports bool/i32/i64/f32/f64 params, found `{}` in `{}`",
                param.ty.render(),
                function.name
            )
        })? {
            DirectCallScalarKind::Bool => "param_bool",
            DirectCallScalarKind::I32 => "param_i32",
            DirectCallScalarKind::I64 => "param_i64",
            DirectCallScalarKind::F32 => "param_f32",
            DirectCallScalarKind::F64 => "param_f64",
            DirectCallScalarKind::BorrowedBuffer => "param_buffer_ref",
            DirectCallScalarKind::TraversalPointer => "param_node_ref",
            DirectCallScalarKind::OwnedBytes => "param_owned_bytes",
        };
        state.yir.nodes.push(Node {
            name: node_name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: instruction.to_owned(),
                args: vec![index.to_string()],
            },
        });
        bindings.insert(param.name.clone(), node_name);
    }
    let saved_effect_anchor = state.last_effect_anchor.take();
    let returned = lower_function_body(function, state, &mut bindings, false)?
        .ok_or_else(|| format!("function `{}` did not return a value", function.name))?;
    state.last_effect_anchor = saved_effect_anchor;
    let return_name = format!("__fn_{}_return", function.name);
    let return_instruction = if function_owned_struct_layout(function, state).is_some() {
        "return_owned_struct"
    } else {
        match direct_call_signature_kind(function).ok_or_else(|| {
            format!(
                "ordinary direct-call lowering only supports scalar or scheduler-owned recursive scalar struct return type in `{}`",
                function.name
            )
        })? {
            DirectCallScalarKind::Bool => "return_bool",
            DirectCallScalarKind::I32 => "return_i32",
            DirectCallScalarKind::I64 => "return_i64",
            DirectCallScalarKind::F32 => "return_f32",
            DirectCallScalarKind::F64 => "return_f64",
            DirectCallScalarKind::BorrowedBuffer => unreachable!("borrowed refs cannot return"),
            DirectCallScalarKind::TraversalPointer => {
                unreachable!("traversal refs cannot return")
            }
            DirectCallScalarKind::OwnedBytes => "return_owned_bytes",
        }
    };
    state.yir.nodes.push(Node {
        name: return_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: return_instruction.to_owned(),
            args: vec![returned.clone()],
        },
    });
    push_dep_edges(state, &returned, &return_name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: returned,
        to: return_name,
    });
    for node in &state.yir.nodes[start_index..] {
        state.yir.node_lanes.insert(node.name.clone(), lane.clone());
    }
    Ok(())
}

pub(super) fn push_direct_call_node(
    function: &NirFunction,
    args: &[String],
    state: &mut LoweringState<'_>,
) -> Result<String, String> {
    let name = next_name(state, "cpu_call");
    let struct_layout = function_owned_struct_layout(function, state);
    let instruction = if struct_layout.is_some() {
        "call_owned_struct"
    } else {
        match direct_call_signature_kind(function).ok_or_else(|| {
            format!(
                "ordinary direct-call lowering only supports scalar or scheduler-owned recursive scalar struct return type in `{}`",
                function.name
            )
        })? {
            DirectCallScalarKind::Bool => "call_bool",
            DirectCallScalarKind::I32 => "call_i32",
            DirectCallScalarKind::I64 => "call_i64",
            DirectCallScalarKind::F32 => "call_f32",
            DirectCallScalarKind::F64 => "call_f64",
            DirectCallScalarKind::BorrowedBuffer => unreachable!("borrowed refs cannot return"),
            DirectCallScalarKind::TraversalPointer => {
                unreachable!("traversal refs cannot return")
            }
            DirectCallScalarKind::OwnedBytes => "call_owned_bytes",
        }
    };
    let mut op_args = vec![function.name.clone()];
    if let Some(layout) = struct_layout {
        op_args.push(layout);
    }
    op_args.extend(args.iter().cloned());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: op_args,
        },
    });
    for arg in args {
        push_dep_edges(state, arg, &name);
    }
    Ok(name)
}
