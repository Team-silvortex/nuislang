use std::collections::BTreeMap;

use nuis_semantics::model::{NirExpr, NirParam, NirStmt};

use super::super::NetworkOwnedHandleRequirement;

#[path = "runtime_validation_network_handle_param_merge.rs"]
mod param_merge;
#[path = "runtime_validation_network_handle_param_origins.rs"]
mod param_origins;

use param_origins::{
    infer_network_param_origin, infer_network_param_requirement_from_host_call,
    merge_network_param_origin_bindings, merge_network_param_requirement,
};

pub(super) fn infer_network_param_requirements_in_body(
    body: &[NirStmt],
    params: &[NirParam],
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
) -> Result<(), String> {
    let mut bindings = params
        .iter()
        .enumerate()
        .map(|(index, param)| (param.name.clone(), index))
        .collect::<BTreeMap<_, _>>();
    infer_network_param_requirements_with_bindings(
        body,
        requirements,
        function_requirements,
        &mut bindings,
    )
}

fn infer_network_param_requirements_with_bindings(
    body: &[NirStmt],
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    bindings: &mut BTreeMap<String, usize>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if let Some(origin) = infer_network_param_origin(value, bindings) {
                    bindings.insert(name.clone(), origin);
                } else {
                    bindings.remove(name);
                }
                infer_network_param_requirements_in_expr(
                    value,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
            NirStmt::Print(value)
            | NirStmt::Await(value)
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?,
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                infer_network_param_requirements_in_expr(
                    condition,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
                let mut then_bindings = bindings.clone();
                infer_network_param_requirements_with_bindings(
                    then_body,
                    requirements,
                    function_requirements,
                    &mut then_bindings,
                )?;
                let mut else_bindings = bindings.clone();
                infer_network_param_requirements_with_bindings(
                    else_body,
                    requirements,
                    function_requirements,
                    &mut else_bindings,
                )?;
                merge_network_param_origin_bindings(bindings, &then_bindings, &else_bindings);
            }
            NirStmt::While { condition, body } => {
                infer_network_param_requirements_in_expr(
                    condition,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
                let entry_bindings = bindings.clone();
                let mut loop_bindings = bindings.clone();
                infer_network_param_requirements_with_bindings(
                    body,
                    requirements,
                    function_requirements,
                    &mut loop_bindings,
                )?;
                merge_network_param_origin_bindings(bindings, &entry_bindings, &loop_bindings);
            }
            NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
        }
    }
    Ok(())
}

fn infer_network_param_requirements_in_expr(
    expr: &NirExpr,
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    bindings: &BTreeMap<String, usize>,
) -> Result<(), String> {
    match expr {
        NirExpr::CpuExternCall { callee, args, .. } => {
            infer_network_param_requirement_from_host_call(callee, args, requirements, bindings)?;
            for arg in args {
                infer_network_param_requirements_in_expr(
                    arg,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::CpuSpawn { callee, args }
        | NirExpr::CpuThreadSpawn { callee, args }
        | NirExpr::Call { callee, args } => {
            if let Some(callee_requirements) = function_requirements.get(callee) {
                for (index, arg) in args.iter().enumerate() {
                    let Some(Some(requirement)) = callee_requirements.get(index) else {
                        continue;
                    };
                    if let Some(origin) = infer_network_param_origin(arg, bindings) {
                        merge_network_param_requirement(
                            requirements,
                            origin,
                            *requirement,
                            callee,
                        )?;
                    }
                }
            }
            for arg in args {
                infer_network_param_requirements_in_expr(
                    arg,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
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
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => infer_network_param_requirements_in_expr(
            inner,
            requirements,
            function_requirements,
            bindings,
        )?,
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => infer_network_param_requirements_in_expr(
            value,
            requirements,
            function_requirements,
            bindings,
        )?,
        NirExpr::AllocNode { value, next } => {
            infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                next,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            infer_network_param_requirements_in_expr(
                len,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                fill,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            infer_network_param_requirements_in_expr(
                buffer,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                index,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        }
        | NirExpr::StoreAt {
            buffer: window,
            index,
            value,
        } => {
            infer_network_param_requirements_in_expr(
                window,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                index,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => {
            infer_network_param_requirements_in_expr(
                target,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                value,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            infer_network_param_requirements_in_expr(
                input,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                offset,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                len,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::FieldAccess { base: input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. } => {
            infer_network_param_requirements_in_expr(
                input,
                requirements,
                function_requirements,
                bindings,
            )?
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            infer_network_param_requirements_in_expr(
                base,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                delta,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            infer_network_param_requirements_in_expr(
                delta,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                scale,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                base,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            infer_network_param_requirements_in_expr(
                color,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                speed,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                radius,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::CpuTimeout { task, limit } => {
            infer_network_param_requirements_in_expr(
                task,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                limit,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            infer_network_param_requirements_in_expr(
                task,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                delay,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            infer_network_param_requirements_in_expr(
                receiver,
                requirements,
                function_requirements,
                bindings,
            )?;
            for arg in args {
                infer_network_param_requirements_in_expr(
                    arg,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                infer_network_param_requirements_in_expr(
                    value,
                    requirements,
                    function_requirements,
                    bindings,
                )?;
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            infer_network_param_requirements_in_expr(
                lhs,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                rhs,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            infer_network_param_requirements_in_expr(
                target,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                pipeline,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                viewport,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            infer_network_param_requirements_in_expr(
                pass,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                packet,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                vertex_count,
                requirements,
                function_requirements,
                bindings,
            )?;
            infer_network_param_requirements_in_expr(
                instance_count,
                requirements,
                function_requirements,
                bindings,
            )?;
        }
        _ => {}
    }
    Ok(())
}
