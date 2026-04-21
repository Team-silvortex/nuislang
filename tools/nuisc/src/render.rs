use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstExternInterface, AstModule, AstStmt, AstStructDef, NirBinaryOp,
    NirExpr, NirExternInterface, NirModule, NirStmt, NirStructDef,
};
use yir_core::YirModule;

pub fn render_ast(module: &AstModule) -> String {
    let mut out = String::new();
    for item in &module.uses {
        out.push_str(&format!("use {} {}\n", item.domain, item.unit));
    }
    out.push_str(&format!("ast mod {} unit {}\n", module.domain, module.unit));
    for function in &module.externs {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "  extern \"{}\" fn {}({}) -> {}\n",
            function.abi,
            function.name,
            params,
            render_ast_type(&function.return_type)
        ));
    }
    for interface in &module.extern_interfaces {
        out.push_str(&render_ast_extern_interface(interface));
    }
    for definition in &module.structs {
        out.push_str(&render_ast_struct(definition));
    }
    for function in &module.functions {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let return_suffix = function
            .return_type
            .as_ref()
            .map(|ty| format!(" -> {}", render_ast_type(ty)))
            .unwrap_or_default();
        let async_prefix = if function.is_async { "async " } else { "" };
        out.push_str(&format!(
            "  {}fn {}({}){}\n",
            async_prefix, function.name, params, return_suffix
        ));
        for stmt in &function.body {
            match stmt {
                AstStmt::Let { name, ty, value } => {
                    let type_suffix = ty
                        .as_ref()
                        .map(|ty| format!(": {}", render_ast_type(ty)))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "    let {}{} = {}\n",
                        name,
                        type_suffix,
                        render_ast_expr(value)
                    ));
                }
                AstStmt::Const { name, ty, value } => {
                    out.push_str(&format!(
                        "    const {}: {} = {}\n",
                        name,
                        render_ast_type(ty),
                        render_ast_expr(value)
                    ));
                }
                AstStmt::Print(value) => {
                    out.push_str(&format!("    print {}\n", render_ast_expr(value)));
                }
                AstStmt::Await(value) => {
                    out.push_str(&format!("    await {}\n", render_ast_expr(value)));
                }
                AstStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    out.push_str(&format!("    if {}\n", render_ast_expr(condition)));
                    for stmt in then_body {
                        out.push_str(&format!("      then {}\n", render_ast_stmt_inline(stmt)));
                    }
                    for stmt in else_body {
                        out.push_str(&format!("      else {}\n", render_ast_stmt_inline(stmt)));
                    }
                }
                AstStmt::Expr(expr) => {
                    out.push_str(&format!("    expr {}\n", render_ast_expr(expr)));
                }
                AstStmt::Return(value) => match value {
                    Some(value) => {
                        out.push_str(&format!("    return {}\n", render_ast_expr(value)));
                    }
                    None => out.push_str("    return\n"),
                },
            }
        }
    }
    out
}

pub fn render_nir(module: &NirModule) -> String {
    let mut out = String::new();
    for item in &module.uses {
        out.push_str(&format!("use {} {}\n", item.domain, item.unit));
    }
    out.push_str(&format!("nir mod {} unit {}\n", module.domain, module.unit));
    for function in &module.externs {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "  extern \"{}\" fn {}({}) -> {}\n",
            function.abi,
            function.name,
            params,
            render_nir_type(&function.return_type)
        ));
    }
    for interface in &module.extern_interfaces {
        out.push_str(&render_nir_extern_interface(interface));
    }
    for definition in &module.structs {
        out.push_str(&render_nir_struct(definition));
    }
    for function in &module.functions {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let return_suffix = function
            .return_type
            .as_ref()
            .map(|ty| format!(" -> {}", render_nir_type(ty)))
            .unwrap_or_default();
        let async_prefix = if function.is_async { "async " } else { "" };
        out.push_str(&format!(
            "  {}fn {}({}){}\n",
            async_prefix, function.name, params, return_suffix
        ));
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    let type_suffix = ty
                        .as_ref()
                        .map(|ty| format!(": {}", render_nir_type(ty)))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "    let {}{} = {}\n",
                        name,
                        type_suffix,
                        render_nir_expr(value)
                    ));
                }
                NirStmt::Const { name, ty, value } => {
                    out.push_str(&format!(
                        "    const {}: {} = {}\n",
                        name,
                        render_nir_type(ty),
                        render_nir_expr(value)
                    ));
                }
                NirStmt::Print(value) => {
                    out.push_str(&format!("    print {}\n", render_nir_expr(value)));
                }
                NirStmt::Await(value) => {
                    out.push_str(&format!("    await {}\n", render_nir_expr(value)));
                }
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    out.push_str(&format!("    if {}\n", render_nir_expr(condition)));
                    for stmt in then_body {
                        out.push_str(&format!("      then {}\n", render_nir_stmt_inline(stmt)));
                    }
                    for stmt in else_body {
                        out.push_str(&format!("      else {}\n", render_nir_stmt_inline(stmt)));
                    }
                }
                NirStmt::Expr(expr) => {
                    out.push_str(&format!("    expr {}\n", render_nir_expr(expr)));
                }
                NirStmt::Return(value) => match value {
                    Some(value) => {
                        out.push_str(&format!("    return {}\n", render_nir_expr(value)));
                    }
                    None => out.push_str("    return\n"),
                },
            }
        }
    }
    out
}

pub fn render_yir(module: &YirModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("yir {}\n\n", module.version));
    for resource in &module.resources {
        out.push_str(&format!(
            "resource {} {}\n",
            resource.name, resource.kind.raw
        ));
    }
    if !module.resources.is_empty() {
        out.push('\n');
    }
    for node in &module.nodes {
        let lane_suffix = module
            .node_lanes
            .get(&node.name)
            .map(|lane| format!("@{lane}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "{}.{} {} {}{}",
            node.op.module, node.op.instruction, node.name, node.resource, lane_suffix
        ));
        for arg in &node.op.args {
            if arg.chars().any(char::is_whitespace) {
                out.push_str(&format!(" \"{}\"", escape_debug(arg)));
            } else {
                out.push_str(&format!(" {}", arg));
            }
        }
        out.push('\n');
    }
    if !module.nodes.is_empty() {
        out.push('\n');
    }
    for edge in &module.edges {
        out.push_str(&format!(
            "edge {} {} {}\n",
            edge.kind.as_str(),
            edge.from,
            edge.to
        ));
    }
    out
}

fn render_ast_expr(value: &AstExpr) -> String {
    match value {
        AstExpr::Bool(value) => value.to_string(),
        AstExpr::Text(text) => format!("\"{}\"", escape_debug(text)),
        AstExpr::Int(value) => value.to_string(),
        AstExpr::Var(name) => name.clone(),
        AstExpr::Await(value) => format!("await {}", render_ast_expr(value)),
        AstExpr::Instantiate { domain, unit } => format!("instantiate {} {}", domain, unit),
        AstExpr::Call { callee, args } => format!(
            "{}({})",
            callee,
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => format!(
            "{}.{}({})",
            render_ast_expr(receiver),
            method,
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::StructLiteral { type_name, fields } => format!(
            "{} {{ {} }}",
            type_name,
            fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", render_ast_expr(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::FieldAccess { base, field } => format!("{}.{}", render_ast_expr(base), field),
        AstExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_ast_expr(lhs),
            render_ast_binary_op(*op),
            render_ast_expr(rhs)
        ),
    }
}

fn render_ast_struct(definition: &AstStructDef) -> String {
    let mut out = String::new();
    out.push_str(&format!("  struct {}\n", definition.name));
    for field in &definition.fields {
        out.push_str(&format!(
            "    field {}: {}\n",
            field.name,
            render_ast_type(&field.ty)
        ));
    }
    out
}

fn render_ast_extern_interface(interface: &AstExternInterface) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "  extern \"{}\" interface {}\n",
        interface.abi, interface.name
    ));
    for function in &interface.methods {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "    fn {}({}) -> {}\n",
            function.name,
            params,
            render_ast_type(&function.return_type)
        ));
    }
    out
}

fn render_nir_expr(value: &NirExpr) -> String {
    match value {
        NirExpr::Bool(value) => value.to_string(),
        NirExpr::Text(text) => format!("\"{}\"", escape_debug(text)),
        NirExpr::Int(value) => value.to_string(),
        NirExpr::Var(name) => name.clone(),
        NirExpr::Await(value) => format!("await {}", render_nir_expr(value)),
        NirExpr::Instantiate { domain, unit } => format!("instantiate {} {}", domain, unit),
        NirExpr::Null => "null()".to_owned(),
        NirExpr::Borrow(value) => format!("borrow({})", render_nir_expr(value)),
        NirExpr::BorrowEnd(value) => format!("borrow_end({})", render_nir_expr(value)),
        NirExpr::Move(value) => format!("move({})", render_nir_expr(value)),
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
        NirExpr::DataCopyWindow { input, offset, len } => format!(
            "data_copy_window({}, {}, {})",
            render_nir_expr(input),
            render_nir_expr(offset),
            render_nir_expr(len)
        ),
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
        NirExpr::CpuJoin(task) => format!("join({})", render_nir_expr(task)),
        NirExpr::CpuCancel(task) => format!("cancel({})", render_nir_expr(task)),
        NirExpr::CpuPresentFrame(value) => {
            format!("cpu_present_frame({})", render_nir_expr(value))
        }
        NirExpr::ShaderProfileTargetRef { unit } => {
            format!("shader_profile_target(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileViewportRef { unit } => {
            format!("shader_profile_viewport(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePipelineRef { unit } => {
            format!("shader_profile_pipeline(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileVertexCountRef { unit } => {
            format!("shader_profile_vertex_count(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileInstanceCountRef { unit } => {
            format!("shader_profile_instance_count(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePacketColorSlotRef { unit } => {
            format!(
                "shader_profile_packet_color_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { unit } => {
            format!(
                "shader_profile_packet_speed_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { unit } => {
            format!(
                "shader_profile_packet_radius_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfilePacketTagRef { unit } => {
            format!("shader_profile_packet_tag(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfileMaterialModeRef { unit } => {
            format!("shader_profile_material_mode(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePassKindRef { unit } => {
            format!("shader_profile_pass_kind(\"{}\")", escape_debug(unit))
        }
        NirExpr::ShaderProfilePacketFieldCountRef { unit } => {
            format!(
                "shader_profile_packet_field_count(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => format!(
            "shader_profile_color_seed(\"{}\", {}, {})",
            escape_debug(unit),
            render_nir_expr(base),
            render_nir_expr(delta)
        ),
        NirExpr::ShaderProfileSpeedSeed {
            unit,
            delta,
            scale,
            base,
        } => format!(
            "shader_profile_speed_seed(\"{}\", {}, {}, {})",
            escape_debug(unit),
            render_nir_expr(delta),
            render_nir_expr(scale),
            render_nir_expr(base)
        ),
        NirExpr::ShaderProfileRadiusSeed { unit, base, delta } => format!(
            "shader_profile_radius_seed(\"{}\", {}, {})",
            escape_debug(unit),
            render_nir_expr(base),
            render_nir_expr(delta)
        ),
        NirExpr::ShaderProfilePacket {
            unit,
            color,
            speed,
            radius,
        } => format!(
            "shader_profile_packet(\"{}\", {}, {}, {})",
            escape_debug(unit),
            render_nir_expr(color),
            render_nir_expr(speed),
            render_nir_expr(radius)
        ),
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
        NirExpr::KernelProfileBindCoreRef { unit } => {
            format!("kernel_profile_bind_core(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelProfileQueueDepthRef { unit } => {
            format!("kernel_profile_queue_depth(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelProfileBatchLanesRef { unit } => {
            format!("kernel_profile_batch_lanes(\"{}\")", escape_debug(unit))
        }
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
        NirExpr::ShaderTarget {
            format,
            width,
            height,
        } => format!(
            "shader_target(\"{}\", {}, {})",
            escape_debug(format),
            width,
            height
        ),
        NirExpr::ShaderViewport { width, height } => {
            format!("shader_viewport({}, {})", width, height)
        }
        NirExpr::ShaderPipeline { name, topology } => format!(
            "shader_pipeline(\"{}\", \"{}\")",
            escape_debug(name),
            escape_debug(topology)
        ),
        NirExpr::ShaderInlineWgsl { entry, source } => format!(
            "shader_inline_wgsl(\"{}\", \"{}\")",
            escape_debug(entry),
            escape_debug(source)
        ),
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => format!(
            "shader_begin_pass({}, {}, {})",
            render_nir_expr(target),
            render_nir_expr(pipeline),
            render_nir_expr(viewport)
        ),
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => format!(
            "shader_draw_instanced({}, {}, {}, {})",
            render_nir_expr(pass),
            render_nir_expr(packet),
            render_nir_expr(vertex_count),
            render_nir_expr(instance_count)
        ),
        NirExpr::ShaderProfileRender { unit, packet } => format!(
            "shader_profile_render(\"{}\", {})",
            escape_debug(unit),
            render_nir_expr(packet)
        ),
        NirExpr::LoadValue(value) => format!("load_value({})", render_nir_expr(value)),
        NirExpr::LoadNext(value) => format!("load_next({})", render_nir_expr(value)),
        NirExpr::BufferLen(value) => format!("buffer_len({})", render_nir_expr(value)),
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
        NirExpr::StructLiteral { type_name, fields } => format!(
            "{} {{ {} }}",
            type_name,
            fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", render_nir_expr(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        NirExpr::FieldAccess { base, field } => format!("{}.{}", render_nir_expr(base), field),
        NirExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_nir_expr(lhs),
            render_nir_binary_op(*op),
            render_nir_expr(rhs)
        ),
    }
}

fn render_nir_struct(definition: &NirStructDef) -> String {
    let mut out = String::new();
    out.push_str(&format!("  struct {}\n", definition.name));
    for field in &definition.fields {
        out.push_str(&format!(
            "    field {}: {}\n",
            field.name,
            render_nir_type(&field.ty)
        ));
    }
    out
}

fn render_nir_extern_interface(interface: &NirExternInterface) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "  extern \"{}\" interface {}\n",
        interface.abi, interface.name
    ));
    for function in &interface.methods {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "    fn {}({}) -> {}\n",
            function.name,
            params,
            render_nir_type(&function.return_type)
        ));
    }
    out
}

fn render_ast_binary_op(op: AstBinaryOp) -> &'static str {
    match op {
        AstBinaryOp::Add => "+",
        AstBinaryOp::Sub => "-",
        AstBinaryOp::Mul => "*",
        AstBinaryOp::Div => "/",
    }
}

fn render_nir_binary_op(op: NirBinaryOp) -> &'static str {
    match op {
        NirBinaryOp::Add => "+",
        NirBinaryOp::Sub => "-",
        NirBinaryOp::Mul => "*",
        NirBinaryOp::Div => "/",
    }
}

fn render_ast_type(ty: &nuis_semantics::model::AstTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_ast_type)
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('>');
    }
    if ty.is_optional {
        out.push('?');
    }
    out
}

fn render_nir_type(ty: &nuis_semantics::model::NirTypeRef) -> String {
    let mut out = String::new();
    if ty.is_ref {
        out.push_str("ref ");
    }
    out.push_str(&ty.name);
    if !ty.generic_args.is_empty() {
        out.push('<');
        out.push_str(
            &ty.generic_args
                .iter()
                .map(render_nir_type)
                .collect::<Vec<_>>()
                .join(", "),
        );
        out.push('>');
    }
    if ty.is_optional {
        out.push('?');
    }
    out
}

fn render_ast_stmt_inline(stmt: &AstStmt) -> String {
    match stmt {
        AstStmt::Let { name, ty, value } => {
            let suffix = ty
                .as_ref()
                .map(|ty| format!(": {}", render_ast_type(ty)))
                .unwrap_or_default();
            format!("let {}{} = {}", name, suffix, render_ast_expr(value))
        }
        AstStmt::Const { name, ty, value } => {
            format!(
                "const {}: {} = {}",
                name,
                render_ast_type(ty),
                render_ast_expr(value)
            )
        }
        AstStmt::Print(value) => format!("print {}", render_ast_expr(value)),
        AstStmt::Await(value) => format!("await {}", render_ast_expr(value)),
        AstStmt::Expr(expr) => render_ast_expr(expr),
        AstStmt::If { .. } => "if ...".to_owned(),
        AstStmt::Return(value) => match value {
            Some(value) => format!("return {}", render_ast_expr(value)),
            None => "return".to_owned(),
        },
    }
}

fn render_nir_stmt_inline(stmt: &NirStmt) -> String {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            let suffix = ty
                .as_ref()
                .map(|ty| format!(": {}", render_nir_type(ty)))
                .unwrap_or_default();
            format!("let {}{} = {}", name, suffix, render_nir_expr(value))
        }
        NirStmt::Const { name, ty, value } => {
            format!(
                "const {}: {} = {}",
                name,
                render_nir_type(ty),
                render_nir_expr(value)
            )
        }
        NirStmt::Print(value) => format!("print {}", render_nir_expr(value)),
        NirStmt::Await(value) => format!("await {}", render_nir_expr(value)),
        NirStmt::Expr(expr) => render_nir_expr(expr),
        NirStmt::If { .. } => "if ...".to_owned(),
        NirStmt::Return(value) => match value {
            Some(value) => format!("return {}", render_nir_expr(value)),
            None => "return".to_owned(),
        },
    }
}

fn escape_debug(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
