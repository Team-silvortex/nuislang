use std::collections::BTreeMap;

use nuis_semantics::model::NirExpr;

use super::{NetworkOwnedHandleBinding, NetworkOwnedHandleRequirement, NetworkOwnedHandleReturn};

#[path = "runtime_validation_network_handle_call.rs"]
mod call;
use call::{validate_network_function_call_requirements, validate_network_owned_handle_call};

pub(super) fn validate_network_owned_handle_provenance_in_expr(
    expr: &NirExpr,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Result<(), String> {
    let _known_return_kind_count = function_return_kinds.len();
    match expr {
        NirExpr::CpuExternCall { callee, args, .. } => {
            validate_network_owned_handle_call(callee, args, from, to, bindings)?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(
                    arg,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
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
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskFailed(inner)
        | NirExpr::CpuTaskValue(inner)
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
        | NirExpr::IsNull(inner) => {
            validate_network_owned_handle_provenance_in_expr(
                inner,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::AllocNode { value, next } => {
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                next,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            validate_network_owned_handle_provenance_in_expr(
                len,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                fill,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::LoadAt { buffer, index } => {
            validate_network_owned_handle_provenance_in_expr(
                buffer,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataReadWindow { window, index } => {
            validate_network_owned_handle_provenance_in_expr(
                window,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                window,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::StoreValue { target, value } => {
            validate_network_owned_handle_provenance_in_expr(
                target,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::StoreNext { target, next } => {
            validate_network_owned_handle_provenance_in_expr(
                target,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                next,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                buffer,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                index,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                value,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            validate_network_owned_handle_provenance_in_expr(
                input,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                offset,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                len,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                input,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                base,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                delta,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            validate_network_owned_handle_provenance_in_expr(
                delta,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                scale,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                base,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => {
            validate_network_owned_handle_provenance_in_expr(
                color,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                speed,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                radius,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::CpuSpawn { callee, args }
        | NirExpr::CpuThreadSpawn { callee, args }
        | NirExpr::Call { callee, args } => {
            validate_network_function_call_requirements(
                callee,
                args,
                from,
                to,
                bindings,
                function_requirements,
            )?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(
                    arg,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            validate_network_owned_handle_provenance_in_expr(
                task,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                limit,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            validate_network_owned_handle_provenance_in_expr(
                task,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                delay,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                receiver,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            for arg in args {
                validate_network_owned_handle_provenance_in_expr(
                    arg,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
        }
        NirExpr::FieldAccess { base, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                base,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                lhs,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                rhs,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                target,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                pipeline,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                viewport,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            validate_network_owned_handle_provenance_in_expr(
                pass,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                packet,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                vertex_count,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
            validate_network_owned_handle_provenance_in_expr(
                instance_count,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            validate_network_owned_handle_provenance_in_expr(
                packet,
                from,
                to,
                bindings,
                function_requirements,
                function_return_kinds,
            )?;
        }
        _ => {}
    }
    Ok(())
}
