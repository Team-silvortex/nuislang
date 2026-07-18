use nuis_semantics::model::NirExpr;

pub(crate) fn walk_child_exprs(expr: &NirExpr, f: &mut dyn FnMut(&NirExpr)) {
    match expr {
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
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
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => f(inner),
        NirExpr::AllocNode { value, next } => {
            f(value);
            f(next);
        }
        NirExpr::AllocBuffer { len, fill } => {
            f(len);
            f(fill);
        }
        NirExpr::LoadAt { buffer, index } => {
            f(buffer);
            f(index);
        }
        NirExpr::StoreValue { target, value } => {
            f(target);
            f(value);
        }
        NirExpr::StoreNext { target, next } => {
            f(target);
            f(next);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            f(buffer);
            f(index);
            f(value);
        }
        NirExpr::DataResult { value: input, .. }
        | NirExpr::NetworkResult { value: input, .. }
        | NirExpr::ShaderResult { value: input, .. }
        | NirExpr::KernelResult { value: input, .. } => f(input),
        NirExpr::DataReadWindow { window, index } => {
            f(window);
            f(index);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            f(window);
            f(index);
            f(value);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            f(input);
            f(offset);
            f(len);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            f(base);
            f(delta);
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
            f(color);
            f(speed);
            f(radius);
            if let Some(accent) = accent {
                f(accent);
            }
            if let Some(toggle_state) = toggle_state {
                f(toggle_state);
            }
            if let Some(focus_index) = focus_index {
                f(focus_index);
            }
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            f(delta);
            f(scale);
            f(base);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. }
        | NirExpr::FieldAccess { base: input, .. } => f(input),
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                f(arg);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            f(task);
            f(limit);
        }
        NirExpr::CpuReadyAfter { task, delay } => {
            f(task);
            f(delay);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            f(receiver);
            for arg in args {
                f(arg);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                f(value);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            f(target);
            f(pipeline);
            f(viewport);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            f(pass);
            f(packet);
            f(vertex_count);
            f(instance_count);
        }
        _ => {}
    }
}
