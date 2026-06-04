use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum DirectCallScalarKind {
    Bool,
    I32,
    I64,
}

pub(super) fn collect_self_recursive_functions(module: &NirModule) -> BTreeSet<String> {
    module
        .functions
        .iter()
        .filter(|function| !function.is_async)
        .filter(|function| direct_call_signature_kind(function).is_some())
        .filter(|function| function_contains_self_call(function, &function.body))
        .map(|function| function.name.clone())
        .collect()
}

fn direct_call_scalar_kind(ty: &nuis_semantics::model::NirTypeRef) -> Option<DirectCallScalarKind> {
    if ty.is_ref || ty.is_optional || !ty.generic_args.is_empty() {
        return None;
    }
    if ty.is_bool_scalar() {
        Some(DirectCallScalarKind::Bool)
    } else if ty.name == "i32" {
        Some(DirectCallScalarKind::I32)
    } else if ty.name == "i64" {
        Some(DirectCallScalarKind::I64)
    } else {
        None
    }
}

fn direct_call_signature_kind(function: &NirFunction) -> Option<DirectCallScalarKind> {
    let return_kind = direct_call_scalar_kind(function.return_type.as_ref()?)?;
    for param in &function.params {
        direct_call_scalar_kind(&param.ty)?;
    }
    Some(return_kind)
}

fn function_contains_self_call(function: &NirFunction, body: &[NirStmt]) -> bool {
    body.iter()
        .any(|stmt| stmt_contains_self_call(function, stmt))
}

fn stmt_contains_self_call(function: &NirFunction, stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value)
        | NirStmt::Await(value) => expr_contains_self_call(function, value),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_self_call(function, condition)
                || function_contains_self_call(function, then_body)
                || function_contains_self_call(function, else_body)
        }
        NirStmt::While { condition, body } => {
            expr_contains_self_call(function, condition)
                || function_contains_self_call(function, body)
        }
        NirStmt::Return(Some(value)) => expr_contains_self_call(function, value),
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

fn expr_contains_self_call(function: &NirFunction, expr: &NirExpr) -> bool {
    match expr {
        NirExpr::Call { callee, args } => {
            callee == &function.name
                || args
                    .iter()
                    .any(|arg| expr_contains_self_call(function, arg))
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_contains_self_call(function, receiver)
                || args
                    .iter()
                    .any(|arg| expr_contains_self_call(function, arg))
        }
        NirExpr::FieldAccess { base, .. } => expr_contains_self_call(function, base),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_self_call(function, lhs) || expr_contains_self_call(function, rhs)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_self_call(function, value)),
        NirExpr::Await(value)
        | NirExpr::DataOutputPipe(value)
        | NirExpr::DataInputPipe(value)
        | NirExpr::DataReady(value)
        | NirExpr::DataMoved(value)
        | NirExpr::DataWindowed(value)
        | NirExpr::DataValue(value)
        | NirExpr::DataFreezeWindow(value)
        | NirExpr::CpuJoin(value)
        | NirExpr::CpuCancel(value)
        | NirExpr::CpuJoinResult(value)
        | NirExpr::CpuTaskCompleted(value)
        | NirExpr::CpuTaskTimedOut(value)
        | NirExpr::CpuTaskCancelled(value)
        | NirExpr::CpuTaskValue(value)
        | NirExpr::CpuPresentFrame(value) => expr_contains_self_call(function, value),
        NirExpr::Free(value)
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
        | NirExpr::NetworkValue(value) => expr_contains_self_call(function, value),
        NirExpr::DataResult { value, .. } => expr_contains_self_call(function, value),
        NirExpr::KernelResult { value, .. } => expr_contains_self_call(function, value),
        NirExpr::ShaderResult { value, .. } => expr_contains_self_call(function, value),
        NirExpr::NetworkResult { value, .. } => expr_contains_self_call(function, value),
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            expr_contains_self_call(function, input)
                || expr_contains_self_call(function, offset)
                || expr_contains_self_call(function, len)
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
            expr_contains_self_call(function, input)
        }
        NirExpr::DataReadWindow { window, index } => {
            expr_contains_self_call(function, window) || expr_contains_self_call(function, index)
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            expr_contains_self_call(function, window)
                || expr_contains_self_call(function, index)
                || expr_contains_self_call(function, value)
        }
        NirExpr::KernelElementAt { input, row, col } => {
            expr_contains_self_call(function, input)
                || expr_contains_self_call(function, row)
                || expr_contains_self_call(function, col)
        }
        NirExpr::KernelMap { input, scalar, .. } | NirExpr::KernelMapAxis { input, scalar, .. } => {
            expr_contains_self_call(function, input)
                || scalar
                    .as_deref()
                    .is_some_and(|value| expr_contains_self_call(function, value))
        }
        NirExpr::KernelZip { lhs, rhs, .. } | NirExpr::KernelMatmul { lhs, rhs } => {
            expr_contains_self_call(function, lhs) || expr_contains_self_call(function, rhs)
        }
        NirExpr::KernelAddBias { input, bias } => {
            expr_contains_self_call(function, input) || expr_contains_self_call(function, bias)
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            expr_contains_self_call(function, target)
                || expr_contains_self_call(function, pipeline)
                || expr_contains_self_call(function, viewport)
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            expr_contains_self_call(function, pass)
                || expr_contains_self_call(function, packet)
                || expr_contains_self_call(function, vertex_count)
                || expr_contains_self_call(function, instance_count)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            expr_contains_self_call(function, input)
        }
        NirExpr::CpuSpawn { args, .. } => args
            .iter()
            .any(|arg| expr_contains_self_call(function, arg)),
        NirExpr::CpuTimeout { task, limit } => {
            expr_contains_self_call(function, task) || expr_contains_self_call(function, limit)
        }
        NirExpr::CpuExternCall { args, .. } => args
            .iter()
            .any(|arg| expr_contains_self_call(function, arg)),
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
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. } => false,
        _ => false,
    }
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
                "ordinary direct-call lowering only supports bool/i32/i64 params, found `{}` in `{}`",
                param.ty.render(),
                function.name
            )
        })? {
            DirectCallScalarKind::Bool => "param_bool",
            DirectCallScalarKind::I32 => "param_i32",
            DirectCallScalarKind::I64 => "param_i64",
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
    let returned = lower_function_body(function, state, &mut bindings, false)?
        .ok_or_else(|| format!("function `{}` did not return a value", function.name))?;
    let return_name = format!("__fn_{}_return", function.name);
    let return_instruction = match direct_call_signature_kind(function).ok_or_else(|| {
        format!(
            "ordinary direct-call lowering only supports bool/i32/i64 return type in `{}`",
            function.name
        )
    })? {
        DirectCallScalarKind::Bool => "return_bool",
        DirectCallScalarKind::I32 => "return_i32",
        DirectCallScalarKind::I64 => "return_i64",
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
    let instruction = match direct_call_signature_kind(function).ok_or_else(|| {
        format!(
            "ordinary direct-call lowering only supports bool/i32/i64 return type in `{}`",
            function.name
        )
    })? {
        DirectCallScalarKind::Bool => "call_bool",
        DirectCallScalarKind::I32 => "call_i32",
        DirectCallScalarKind::I64 => "call_i64",
    };
    let mut op_args = vec![function.name.clone()];
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
