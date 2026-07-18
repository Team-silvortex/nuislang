use super::*;

#[path = "render_nir_expr_kernel.rs"]
mod render_nir_expr_kernel;
#[path = "render_nir_expr_shader.rs"]
mod render_nir_expr_shader;
use render_nir_expr_kernel::render_kernel_nir_expr;
use render_nir_expr_shader::render_shader_nir_expr;

pub(super) fn render_nir_expr(value: &NirExpr) -> String {
    if let Some(rendered) = render_shader_nir_expr(value) {
        return rendered;
    }

    if let Some(rendered) = render_kernel_nir_expr(value) {
        return rendered;
    }

    match value {
        NirExpr::Bool(value) => value.to_string(),
        NirExpr::Text(text) => format!("\"{}\"", escape_debug(text)),
        NirExpr::Int(value) => value.to_string(),
        NirExpr::F32(value) | NirExpr::F64(value) => value.clone(),
        NirExpr::CastI64ToI32(value) => format!("i32_from_i64({})", render_nir_expr(value)),
        NirExpr::CastI32ToI64(value) => format!("i64_from_i32({})", render_nir_expr(value)),
        NirExpr::CastI64ToBool(value) => format!("bool_from_i64({})", render_nir_expr(value)),
        NirExpr::CastBoolToI64(value) => format!("i64_from_bool({})", render_nir_expr(value)),
        NirExpr::CastI64ToF32(value) => format!("f32_from_i64({})", render_nir_expr(value)),
        NirExpr::CastF32ToI64(value) => format!("i64_from_f32({})", render_nir_expr(value)),
        NirExpr::CastI64ToF64(value) => format!("f64_from_i64({})", render_nir_expr(value)),
        NirExpr::CastF64ToI64(value) => format!("i64_from_f64({})", render_nir_expr(value)),
        NirExpr::Var(name) => name.clone(),
        NirExpr::Await(value) => format!("await {}", render_nir_expr(value)),
        NirExpr::Instantiate { domain, unit } => format!("instantiate {} {}", domain, unit),
        NirExpr::Null => "null()".to_owned(),
        NirExpr::Borrow(value) => format!("borrow({})", render_nir_expr(value)),
        NirExpr::BorrowEnd(value) => format!("borrow_end({})", render_nir_expr(value)),
        NirExpr::Move(value) => format!("move({})", render_nir_expr(value)),
        NirExpr::HostBufferHandle(value) => {
            format!("host_buffer_handle({})", render_nir_expr(value))
        }
        NirExpr::AllocNode { value, next } => {
            format!(
                "alloc_node({}, {})",
                render_nir_expr(value),
                render_nir_expr(next)
            )
        }
        NirExpr::AllocBuffer { len, fill } => {
            format!(
                "alloc_buffer({}, {})",
                render_nir_expr(len),
                render_nir_expr(fill)
            )
        }
        NirExpr::DataBindCore(core) => format!("data_bind_core({core})"),
        NirExpr::DataMarker(tag) => format!("data_marker(\"{}\")", escape_debug(tag)),
        NirExpr::DataOutputPipe(value) => format!("data_output_pipe({})", render_nir_expr(value)),
        NirExpr::DataInputPipe(value) => format!("data_input_pipe({})", render_nir_expr(value)),
        NirExpr::DataResult { value, .. } => format!("data_result({})", render_nir_expr(value)),
        NirExpr::DataReady(result) => format!("data_ready({})", render_nir_expr(result)),
        NirExpr::DataMoved(result) => format!("data_moved({})", render_nir_expr(result)),
        NirExpr::DataWindowed(result) => format!("data_windowed({})", render_nir_expr(result)),
        NirExpr::DataValue(result) => format!("data_value({})", render_nir_expr(result)),
        NirExpr::DataCopyWindow { input, offset, len } => format!(
            "data_copy_window({}, {}, {})",
            render_nir_expr(input),
            render_nir_expr(offset),
            render_nir_expr(len)
        ),
        NirExpr::DataReadWindow { window, index } => format!(
            "data_read_window({}, {})",
            render_nir_expr(window),
            render_nir_expr(index)
        ),
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => format!(
            "data_write_window({}, {}, {})",
            render_nir_expr(window),
            render_nir_expr(index),
            render_nir_expr(value)
        ),
        NirExpr::DataFreezeWindow(input) => {
            format!("data_freeze_window({})", render_nir_expr(input))
        }
        NirExpr::DataImmutableWindow { input, offset, len } => format!(
            "data_immutable_window({}, {}, {})",
            render_nir_expr(input),
            render_nir_expr(offset),
            render_nir_expr(len)
        ),
        NirExpr::DataHandleTable(entries) => format!(
            "data_handle_table({})",
            entries
                .iter()
                .map(|(slot, resource)| format!(
                    "\"{}={}\"",
                    escape_debug(slot),
                    escape_debug(resource)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::CpuBindCore(core) => format!("cpu_bind_core({core})"),
        NirExpr::CpuWindow {
            width,
            height,
            title,
        } => format!(
            "cpu_window({}, {}, \"{}\")",
            width,
            height,
            escape_debug(title)
        ),
        NirExpr::CpuInputI64 {
            channel,
            default,
            min,
            max,
            step,
        } => match (min, max, step) {
            (Some(min), Some(max), Some(step)) => format!(
                "cpu_input_i64(\"{}\", {}, {}, {}, {})",
                escape_debug(channel),
                default,
                min,
                max,
                step
            ),
            _ => format!("cpu_input_i64(\"{}\", {})", escape_debug(channel), default),
        },
        NirExpr::CpuTickI64 { start, step } => format!("cpu_tick_i64({}, {})", start, step),
        NirExpr::CpuSpawn { callee, args } => format!(
            "spawn({}({}))",
            callee,
            args.iter()
                .map(render_nir_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::CpuThreadSpawn { callee, args } => format!(
            "thread_spawn({}({}))",
            callee,
            args.iter()
                .map(render_nir_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::CpuJoin(task) => format!("join({})", render_nir_expr(task)),
        NirExpr::CpuThreadJoin(thread) => format!("thread_join({})", render_nir_expr(thread)),
        NirExpr::CpuCancel(task) => format!("cancel({})", render_nir_expr(task)),
        NirExpr::CpuJoinResult(task) => format!("join_result({})", render_nir_expr(task)),
        NirExpr::CpuThreadJoinResult(thread) => {
            format!("thread_join_result({})", render_nir_expr(thread))
        }
        NirExpr::CpuTaskCompleted(result) => {
            format!("task_completed({})", render_nir_expr(result))
        }
        NirExpr::CpuTaskTimedOut(result) => {
            format!("task_timed_out({})", render_nir_expr(result))
        }
        NirExpr::CpuTaskCancelled(result) => {
            format!("task_cancelled({})", render_nir_expr(result))
        }
        NirExpr::CpuTaskFailed(result) => format!("task_failed({})", render_nir_expr(result)),
        NirExpr::CpuTaskValue(result) => format!("task_value({})", render_nir_expr(result)),
        NirExpr::CpuMutexNew(value) => format!("mutex_new({})", render_nir_expr(value)),
        NirExpr::CpuMutexLock(mutex) => format!("mutex_lock({})", render_nir_expr(mutex)),
        NirExpr::CpuMutexUnlock(guard) => format!("mutex_unlock({})", render_nir_expr(guard)),
        NirExpr::CpuMutexValue(guard) => format!("mutex_value({})", render_nir_expr(guard)),
        NirExpr::CpuTimeout { task, limit } => format!(
            "timeout({}, {})",
            render_nir_expr(task),
            render_nir_expr(limit)
        ),
        NirExpr::CpuReadyAfter { task, delay } => format!(
            "ready_after({}, {})",
            render_nir_expr(task),
            render_nir_expr(delay)
        ),
        NirExpr::CpuPresentFrame(value) => {
            format!("cpu_present_frame({})", render_nir_expr(value))
        }
        NirExpr::DataProfileBindCoreRef { unit } => {
            format!("data_profile_bind_core(\"{}\")", escape_debug(unit))
        }
        NirExpr::DataProfileWindowOffsetRef { unit } => {
            format!("data_profile_window_offset(\"{}\")", escape_debug(unit))
        }
        NirExpr::DataProfileUplinkLenRef { unit } => {
            format!("data_profile_uplink_len(\"{}\")", escape_debug(unit))
        }
        NirExpr::DataProfileDownlinkLenRef { unit } => {
            format!("data_profile_downlink_len(\"{}\")", escape_debug(unit))
        }
        NirExpr::DataProfileHandleTableRef { unit } => {
            format!("data_profile_handle_table(\"{}\")", escape_debug(unit))
        }
        NirExpr::DataProfileMarkerRef { unit, tag } => {
            format!(
                "data_profile_marker(\"{}\", \"{}\")",
                escape_debug(unit),
                escape_debug(tag)
            )
        }
        NirExpr::NetworkProfileBindCoreRef { unit } => {
            format!("network_profile_bind_core(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileEndpointKindRef { unit } => {
            format!("network_profile_endpoint_kind(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileTransportFamilyRef { unit } => {
            format!(
                "network_profile_transport_family(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::NetworkProfileLocalPortRef { unit } => {
            format!("network_profile_local_port(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileRemotePortRef { unit } => {
            format!("network_profile_remote_port(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileConnectTimeoutRef { unit } => {
            format!(
                "network_profile_connect_timeout(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::NetworkProfileReadTimeoutRef { unit } => {
            format!("network_profile_read_timeout(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileWriteTimeoutRef { unit } => {
            format!("network_profile_write_timeout(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileTimeoutBudgetRef { unit } => {
            format!("network_profile_timeout_budget(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileRetryBudgetRef { unit } => {
            format!("network_profile_retry_budget(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileStreamWindowRef { unit } => {
            format!("network_profile_stream_window(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileRecvWindowRef { unit } => {
            format!("network_profile_recv_window(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileSendWindowRef { unit } => {
            format!("network_profile_send_window(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileProtocolKindRef { unit } => {
            format!("network_profile_protocol_kind(\"{}\")", escape_debug(unit))
        }
        NirExpr::NetworkProfileProtocolVersionRef { unit } => {
            format!(
                "network_profile_protocol_version(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::NetworkProfileProtocolHeaderBytesRef { unit } => {
            format!(
                "network_profile_protocol_header_bytes(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::NetworkResult { value, .. } => {
            format!("network_result({})", render_nir_expr(value))
        }
        NirExpr::NetworkConfigReady(result) => {
            format!("network_config_ready({})", render_nir_expr(result))
        }
        NirExpr::NetworkSendReady(result) => {
            format!("network_send_ready({})", render_nir_expr(result))
        }
        NirExpr::NetworkRecvReady(result) => {
            format!("network_recv_ready({})", render_nir_expr(result))
        }
        NirExpr::NetworkAcceptReady(result) => {
            format!("network_accept_ready({})", render_nir_expr(result))
        }
        NirExpr::NetworkValue(result) => format!("network_value({})", render_nir_expr(result)),
        NirExpr::DataProfileSendUplink { unit, input } => format!(
            "data_profile_send_uplink(\"{}\", {})",
            escape_debug(unit),
            render_nir_expr(input)
        ),
        NirExpr::DataProfileSendDownlink { unit, input } => format!(
            "data_profile_send_downlink(\"{}\", {})",
            escape_debug(unit),
            render_nir_expr(input)
        ),
        NirExpr::CpuExternCall {
            abi,
            interface,
            callee,
            args,
        } => format!(
            "extern \"{}\" {}{}({})",
            abi,
            interface
                .as_ref()
                .map(|name| format!("{name}::"))
                .unwrap_or_default(),
            callee,
            args.iter()
                .map(render_nir_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::CpuExternCallI32 {
            abi,
            interface,
            callee,
            args,
        } => format!(
            "extern_i32 \"{}\" {}{}({})",
            abi,
            interface
                .as_ref()
                .map(|name| format!("{name}::"))
                .unwrap_or_default(),
            callee,
            args.iter()
                .map(render_nir_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::LoadValue(value) => format!("load_value({})", render_nir_expr(value)),
        NirExpr::LoadNext(value) => format!("load_next({})", render_nir_expr(value)),
        NirExpr::BufferLen(value) => format!("buffer_len({})", render_nir_expr(value)),
        NirExpr::CopyBufferOwned(value) => format!("copy_bytes({})", render_nir_expr(value)),
        NirExpr::BytesLen(value) => format!("bytes_len({})", render_nir_expr(value)),
        NirExpr::DropBytes(value) => format!("drop_bytes({})", render_nir_expr(value)),
        NirExpr::LoadAt { buffer, index } => {
            format!(
                "load_at({}, {})",
                render_nir_expr(buffer),
                render_nir_expr(index)
            )
        }
        NirExpr::StoreValue { target, value } => {
            format!(
                "store_value({}, {})",
                render_nir_expr(target),
                render_nir_expr(value)
            )
        }
        NirExpr::StoreNext { target, next } => {
            format!(
                "store_next({}, {})",
                render_nir_expr(target),
                render_nir_expr(next)
            )
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => format!(
            "store_at({}, {}, {})",
            render_nir_expr(buffer),
            render_nir_expr(index),
            render_nir_expr(value)
        ),
        NirExpr::Free(value) => format!("free({})", render_nir_expr(value)),
        NirExpr::IsNull(value) => format!("is_null({})", render_nir_expr(value)),
        NirExpr::Call { callee, args } => format!(
            "{}({})",
            callee,
            args.iter()
                .map(render_nir_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => format!(
            "{}.{}({})",
            render_nir_expr(receiver),
            method,
            args.iter()
                .map(render_nir_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => format!(
            "{}{} {{ {} }}",
            type_name,
            render_nir_type_arg_suffix(type_args),
            fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", render_nir_expr(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::FieldAccess { base, field } => format!("{}.{}", render_nir_expr(base), field),
        NirExpr::VariantIs { base, variant } => {
            format!("variant_is({}, {})", render_nir_expr(base), variant)
        }
        NirExpr::VariantFieldAccess {
            base,
            variant,
            field,
        } => format!(
            "variant_field({}, {}, {})",
            render_nir_expr(base),
            variant,
            field
        ),
        NirExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_nir_expr(lhs),
            render_nir_binary_op(*op),
            render_nir_expr(rhs)
        ),
        _ => unreachable!("render_nir_expr pre-dispatch should handle this expression family"),
    }
}
