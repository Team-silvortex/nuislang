mod render_stmt_helpers;
mod render_struct_helpers;

use self::render_stmt_helpers::{
    render_ast_destructure_let, render_ast_stmt_inline, render_ast_type_suffix,
};
use self::render_struct_helpers::{
    render_ast_enum, render_ast_struct, render_nir_enum, render_nir_struct,
    render_nir_type_arg_suffix,
};
use nuis_semantics::model::{
    AstAttribute, AstAttributeArg, AstAttributeValue, AstBinaryOp, AstExpr, AstExternInterface,
    AstFunction, AstGenericParam, AstImplDef, AstImplMethod, AstMatchPattern, AstModule, AstStmt,
    AstTraitDef, AstTraitMethodSig, AstTypeRef, AstUnaryOp, AstVisibility, AstWherePredicate,
    NirAnnotation, NirAttributeArg, NirAttributeValue, NirBinaryOp, NirExpr, NirExternInterface,
    NirFunction, NirGenericParam, NirImplDef, NirImplMethod, NirModule, NirStmt, NirTraitDef,
    NirTraitMethodSig, NirVisibility, NirWherePredicate,
};
use yir_core::YirModule;

pub fn render_ast(module: &AstModule) -> String {
    let mut out = String::new();
    out.push_str(&render_ast_doc_comments("", &module.attributes));
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
        let host_prefix = function
            .host_symbol
            .as_ref()
            .map(|symbol| format!("@host_symbol(\"{}\") ", escape_debug(symbol)))
            .unwrap_or_default();
        let visibility_prefix = render_ast_visibility(function.visibility);
        out.push_str(&format!(
            "  {}extern \"{}\" {}fn {}({}) -> {}\n",
            visibility_prefix,
            function.abi,
            host_prefix,
            function.name,
            params,
            render_ast_type(&function.return_type)
        ));
    }
    for interface in &module.extern_interfaces {
        out.push_str(&render_ast_extern_interface(interface));
    }
    for constant in &module.consts {
        out.push_str(&render_ast_doc_comments("  ", &constant.attributes));
        let attribute_prefix = render_ast_attributes(&constant.attributes);
        let visibility_prefix = render_ast_visibility(constant.visibility);
        let rendered_type = constant
            .ty
            .as_ref()
            .map(render_ast_type)
            .map(|ty| format!(": {ty}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "  {}{}const {}{} = {}\n",
            attribute_prefix,
            visibility_prefix,
            constant.name,
            rendered_type,
            render_ast_expr(&constant.value)
        ));
    }
    for alias in &module.type_aliases {
        out.push_str(&render_ast_doc_comments("  ", &alias.attributes));
        let attribute_prefix = render_ast_attributes(&alias.attributes);
        let visibility_prefix = render_ast_visibility(alias.visibility);
        let generics = if alias.generic_params.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                alias
                    .generic_params
                    .iter()
                    .map(render_ast_generic_param)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let where_suffix = render_ast_where_clause(&alias.where_bounds);
        out.push_str(&format!(
            "  {}{}type {}{}{} = {}\n",
            attribute_prefix,
            visibility_prefix,
            alias.name,
            generics,
            where_suffix,
            render_ast_type(&alias.target)
        ));
    }
    for definition in &module.structs {
        out.push_str(&render_ast_struct(definition));
    }
    for definition in &module.enums {
        out.push_str(&render_ast_enum(definition));
    }
    for definition in &module.traits {
        out.push_str(&render_ast_trait(definition));
    }
    for definition in &module.impls {
        out.push_str(&render_ast_impl(definition));
    }
    for function in &module.functions {
        out.push_str(&render_ast_function_header(function));
        for stmt in &function.body {
            match stmt {
                AstStmt::Let {
                    name,
                    ty,
                    value,
                    mutable,
                } => {
                    let type_suffix = render_ast_type_suffix(ty.as_ref());
                    let prefix = if *mutable { "let mut" } else { "let" };
                    out.push_str(&format!(
                        "    {} {}{} = {}\n",
                        prefix,
                        name,
                        type_suffix,
                        render_ast_expr(value)
                    ));
                }
                AstStmt::AssignLocal { name, value } => {
                    out.push_str(&format!("    {} = {}\n", name, render_ast_expr(value)));
                }
                AstStmt::DestructureLet {
                    type_ref,
                    fields,
                    value,
                } => out.push_str(&format!(
                    "    {}\n",
                    render_ast_destructure_let(type_ref.as_ref(), fields, value)
                )),
                AstStmt::Const { name, ty, value } => {
                    let type_suffix = render_ast_type_suffix(ty.as_ref());
                    out.push_str(&format!(
                        "    const {}{} = {}\n",
                        name,
                        type_suffix,
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
                AstStmt::Match { value, arms } => {
                    out.push_str(&format!("    match {}\n", render_ast_expr(value)));
                    for arm in arms {
                        let pattern = render_ast_match_pattern(&arm.pattern);
                        let guarded_pattern = arm
                            .guard
                            .as_ref()
                            .map(|guard| format!("{pattern} if {}", render_ast_expr(guard)))
                            .unwrap_or(pattern);
                        for stmt in &arm.body {
                            out.push_str(&format!(
                                "      arm {} {}\n",
                                guarded_pattern,
                                render_ast_stmt_inline(stmt)
                            ));
                        }
                    }
                }
                AstStmt::While { condition, body } => {
                    out.push_str(&format!("    while {}\n", render_ast_expr(condition)));
                    for stmt in body {
                        out.push_str(&format!("      do {}\n", render_ast_stmt_inline(stmt)));
                    }
                }
                AstStmt::Break => out.push_str("    break\n"),
                AstStmt::Continue => out.push_str("    continue\n"),
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
        let host_prefix = function
            .host_symbol
            .as_ref()
            .map(|symbol| format!("@host_symbol(\"{}\") ", escape_debug(symbol)))
            .unwrap_or_default();
        let visibility_prefix = render_nir_visibility(function.visibility);
        out.push_str(&format!(
            "  {}extern \"{}\" {}fn {}({}) -> {}\n",
            visibility_prefix,
            function.abi,
            host_prefix,
            function.name,
            params,
            render_nir_type(&function.return_type)
        ));
    }
    for interface in &module.extern_interfaces {
        out.push_str(&render_nir_extern_interface(interface));
    }
    for constant in &module.consts {
        let visibility_prefix = render_nir_visibility(constant.visibility);
        out.push_str(&format!(
            "  {}const {}: {} = {}\n",
            visibility_prefix,
            constant.name,
            render_nir_type(&constant.ty),
            render_nir_expr(&constant.value)
        ));
    }
    for alias in &module.type_aliases {
        let visibility_prefix = render_nir_visibility(alias.visibility);
        let generics = if alias.generic_params.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                alias
                    .generic_params
                    .iter()
                    .map(render_nir_generic_param)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let where_suffix = render_nir_where_clause(&alias.where_bounds);
        out.push_str(&format!(
            "  {}type {}{}{} = {}\n",
            visibility_prefix,
            alias.name,
            generics,
            where_suffix,
            render_nir_type(&alias.target)
        ));
    }
    for definition in &module.structs {
        out.push_str(&render_nir_struct(definition));
    }
    for definition in &module.enums {
        out.push_str(&render_nir_enum(definition));
    }
    for definition in &module.traits {
        out.push_str(&render_nir_trait(definition));
    }
    for definition in &module.impls {
        out.push_str(&render_nir_impl(definition));
    }
    for function in &module.functions {
        out.push_str(&render_nir_function_header(function));
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
                NirStmt::While { condition, body } => {
                    out.push_str(&format!("    while {}\n", render_nir_expr(condition)));
                    for stmt in body {
                        out.push_str(&format!("      do {}\n", render_nir_stmt_inline(stmt)));
                    }
                }
                NirStmt::Break => out.push_str("    break\n"),
                NirStmt::Continue => out.push_str("    continue\n"),
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
        AstExpr::Float(value) => value.clone(),
        AstExpr::Var(name) => name.clone(),
        AstExpr::Try(value) => format!("{}?", render_ast_expr(value)),
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => format!(
            "if {} {{ {} }} else {{ {} }}",
            render_ast_expr(condition),
            then_body
                .iter()
                .map(render_ast_stmt_inline)
                .collect::<Vec<_>>()
                .join(" "),
            else_body
                .iter()
                .map(render_ast_stmt_inline)
                .collect::<Vec<_>>()
                .join(" ")
        ),
        AstExpr::Match { value, arms } => format!(
            "match {} {{ {} }}",
            render_ast_expr(value),
            arms.iter()
                .map(|arm| {
                    let pattern = render_ast_match_pattern(&arm.pattern);
                    let guard = arm
                        .guard
                        .as_ref()
                        .map(|guard| format!(" if {}", render_ast_expr(guard)))
                        .unwrap_or_default();
                    let body = arm
                        .body
                        .iter()
                        .map(render_ast_stmt_inline)
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("{pattern}{guard} => {{ {body} }}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::Lambda {
            params,
            return_type,
            body,
        } => {
            let params = params
                .iter()
                .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
                .collect::<Vec<_>>()
                .join(", ");
            let return_suffix = return_type
                .as_ref()
                .map(|ty| format!(" -> {}", render_ast_type(ty)))
                .unwrap_or_default();
            let body = body
                .iter()
                .map(render_ast_stmt_inline)
                .collect::<Vec<_>>()
                .join("; ");
            format!("|{params}|{return_suffix} {{ {body} }}")
        }
        AstExpr::Await(value) => format!("await {}", render_ast_expr(value)),
        AstExpr::Instantiate { domain, unit } => format!("instantiate {} {}", domain, unit),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => format!(
            "{}{}({})",
            callee,
            render_ast_generic_args(generic_args),
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::Invoke { callee, args } => format!(
            "({})({})",
            render_ast_expr(callee),
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => format!(
            "{}.{}{}({})",
            render_ast_expr(receiver),
            method,
            render_ast_generic_args(generic_args),
            args.iter()
                .map(render_ast_expr)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => format!(
            "{}{} {{ {} }}",
            type_name,
            render_ast_generic_args(type_args),
            fields
                .iter()
                .map(|(name, value)| format!("{name}: {}", render_ast_expr(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        AstExpr::FieldAccess { base, field } => format!("{}.{}", render_ast_expr(base), field),
        AstExpr::Unary { op, operand } => {
            format!("({}{})", render_ast_unary_op(*op), render_ast_expr(operand))
        }
        AstExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_ast_expr(lhs),
            render_ast_binary_op(*op),
            render_ast_expr(rhs)
        ),
    }
}

fn render_ast_unary_op(op: AstUnaryOp) -> &'static str {
    match op {
        AstUnaryOp::Not => "!",
        AstUnaryOp::Neg => "-",
        AstUnaryOp::Deref => "*",
    }
}

fn render_ast_generic_args(args: &[AstTypeRef]) -> String {
    if args.is_empty() {
        String::new()
    } else {
        format!(
            "<{}>",
            args.iter()
                .map(render_ast_type)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn render_ast_trait(definition: &AstTraitDef) -> String {
    let mut out = String::new();
    out.push_str(&render_ast_doc_comments("  ", &definition.attributes));
    out.push_str(&format!(
        "  {}{}trait {}\n",
        render_ast_attributes(&definition.attributes),
        render_ast_visibility(definition.visibility),
        definition.name
    ));
    for method in &definition.methods {
        out.push_str(&render_ast_trait_method_sig(method));
    }
    out
}

fn render_ast_impl(definition: &AstImplDef) -> String {
    let mut out = format!(
        "  impl {} for {}\n",
        definition.trait_name,
        render_ast_type(&definition.for_type)
    );
    for method in &definition.methods {
        out.push_str(&render_ast_impl_method(method));
    }
    out
}

fn render_ast_extern_interface(interface: &AstExternInterface) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "  {}extern \"{}\" interface {}\n",
        render_ast_visibility(interface.visibility),
        interface.abi,
        interface.name
    ));
    for function in &interface.methods {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let host_prefix = function
            .host_symbol
            .as_ref()
            .map(|symbol| format!("@host_symbol(\"{}\") ", escape_debug(symbol)))
            .unwrap_or_default();
        out.push_str(&format!(
            "    {}{}fn {}({}) -> {}\n",
            render_ast_visibility(function.visibility),
            host_prefix,
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
        NirExpr::ShaderProfileSliderColorSlotRef { unit } => {
            format!(
                "shader_profile_slider_color_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileSliderSpeedSlotRef { unit } => {
            format!(
                "shader_profile_slider_speed_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileSliderRadiusSlotRef { unit } => {
            format!(
                "shader_profile_slider_radius_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileHeaderAccentSlotRef { unit } => {
            format!(
                "shader_profile_header_accent_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileToggleLiveSlotRef { unit } => {
            format!(
                "shader_profile_toggle_live_slot(\"{}\")",
                escape_debug(unit)
            )
        }
        NirExpr::ShaderProfileFocusSlotRef { unit } => {
            format!("shader_profile_focus_slot(\"{}\")", escape_debug(unit))
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
            packet_type_name,
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
        } => {
            let packet_callee = if packet_type_name.as_deref() == Some("NovaPanelPacket") {
                if unit == "__nova__" {
                    "nova_panel_packet"
                } else {
                    "shader_profile_panel_packet"
                }
            } else {
                "shader_profile_packet"
            };
            if let (Some(accent), Some(toggle_state), Some(focus_index)) =
                (accent.as_ref(), toggle_state.as_ref(), focus_index.as_ref())
            {
                if packet_callee == "nova_panel_packet" {
                    return format!(
                        "{}({}, {}, {}, {}, {}, {})",
                        packet_callee,
                        render_nir_expr(color),
                        render_nir_expr(speed),
                        render_nir_expr(radius),
                        render_nir_expr(accent),
                        render_nir_expr(toggle_state),
                        render_nir_expr(focus_index)
                    );
                }
                format!(
                    "{}(\"{}\", {}, {}, {}, {}, {}, {})",
                    packet_callee,
                    escape_debug(unit),
                    render_nir_expr(color),
                    render_nir_expr(speed),
                    render_nir_expr(radius),
                    render_nir_expr(accent),
                    render_nir_expr(toggle_state),
                    render_nir_expr(focus_index)
                )
            } else {
                format!(
                    "{}(\"{}\", {}, {}, {})",
                    packet_callee,
                    escape_debug(unit),
                    render_nir_expr(color),
                    render_nir_expr(speed),
                    render_nir_expr(radius)
                )
            }
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
        NirExpr::KernelProfileBindCoreRef { unit } => {
            format!("kernel_profile_bind_core(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelProfileQueueDepthRef { unit } => {
            format!("kernel_profile_queue_depth(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelProfileBatchLanesRef { unit } => {
            format!("kernel_profile_batch_lanes(\"{}\")", escape_debug(unit))
        }
        NirExpr::KernelResult { value, .. } => {
            format!("kernel_result({})", render_nir_expr(value))
        }
        NirExpr::KernelConfigReady(result) => {
            format!("kernel_config_ready({})", render_nir_expr(result))
        }
        NirExpr::KernelValue(result) => format!("kernel_value({})", render_nir_expr(result)),
        NirExpr::KernelTensor {
            rows,
            cols,
            elements_csv,
        } => format!(
            "kernel_tensor({}, {}, \"{}\")",
            rows,
            cols,
            escape_debug(elements_csv)
        ),
        NirExpr::KernelShape(input) => format!("kernel_shape({})", render_nir_expr(input)),
        NirExpr::KernelRows(input) => format!("kernel_rows({})", render_nir_expr(input)),
        NirExpr::KernelCols(input) => format!("kernel_cols({})", render_nir_expr(input)),
        NirExpr::KernelRow(input) => format!("kernel_row({})", render_nir_expr(input)),
        NirExpr::KernelCol(input) => format!("kernel_col({})", render_nir_expr(input)),
        NirExpr::KernelElementAt { input, row, col } => format!(
            "kernel_element_at({}, {}, {})",
            render_nir_expr(input),
            render_nir_expr(row),
            render_nir_expr(col)
        ),
        NirExpr::KernelReshape { input, rows, cols } => format!(
            "kernel_reshape({}, {}, {})",
            render_nir_expr(input),
            rows,
            cols
        ),
        NirExpr::KernelBroadcast { input, rows, cols } => format!(
            "kernel_broadcast({}, {}, {})",
            render_nir_expr(input),
            rows,
            cols
        ),
        NirExpr::KernelMap { input, op, scalar } => match scalar {
            Some(scalar) => format!(
                "kernel_map({}, \"{}\", {})",
                render_nir_expr(input),
                op.render(),
                render_nir_expr(scalar)
            ),
            None => format!(
                "kernel_map({}, \"{}\")",
                render_nir_expr(input),
                op.render()
            ),
        },
        NirExpr::KernelMapAxis {
            input,
            axis,
            op,
            scalar,
        } => match scalar {
            Some(scalar) => format!(
                "kernel_map_axis({}, \"{}\", \"{}\", {})",
                render_nir_expr(input),
                axis.render(),
                op.render(),
                render_nir_expr(scalar)
            ),
            None => format!(
                "kernel_map_axis({}, \"{}\", \"{}\")",
                render_nir_expr(input),
                axis.render(),
                op.render()
            ),
        },
        NirExpr::KernelZip { lhs, rhs, op } => format!(
            "kernel_zip({}, {}, \"{}\")",
            render_nir_expr(lhs),
            render_nir_expr(rhs),
            op.render()
        ),
        NirExpr::KernelMatmul { lhs, rhs } => format!(
            "kernel_matmul({}, {})",
            render_nir_expr(lhs),
            render_nir_expr(rhs)
        ),
        NirExpr::KernelAddBias { input, bias } => format!(
            "kernel_add_bias({}, {})",
            render_nir_expr(input),
            render_nir_expr(bias)
        ),
        NirExpr::KernelRelu(input) => format!("kernel_relu({})", render_nir_expr(input)),
        NirExpr::KernelReduceSum(input) => {
            format!("kernel_reduce_sum({})", render_nir_expr(input))
        }
        NirExpr::KernelReduceSumAxis { input, axis } => format!(
            "kernel_reduce_sum_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelReduceMaxAxis { input, axis } => format!(
            "kernel_reduce_max_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelReduceMeanAxis { input, axis } => format!(
            "kernel_reduce_mean_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelReduceMax(input) => {
            format!("kernel_reduce_max({})", render_nir_expr(input))
        }
        NirExpr::KernelReduceMean(input) => {
            format!("kernel_reduce_mean({})", render_nir_expr(input))
        }
        NirExpr::KernelArgmaxAxis { input, axis } => format!(
            "kernel_argmax_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelArgminAxis { input, axis } => format!(
            "kernel_argmin_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelArgmax(input) => format!("kernel_argmax({})", render_nir_expr(input)),
        NirExpr::KernelArgmin(input) => format!("kernel_argmin({})", render_nir_expr(input)),
        NirExpr::KernelSort(input) => format!("kernel_sort({})", render_nir_expr(input)),
        NirExpr::KernelSortAxis { input, axis } => format!(
            "kernel_sort_axis({}, \"{}\")",
            render_nir_expr(input),
            axis.render()
        ),
        NirExpr::KernelTopk { input, k } => {
            format!("kernel_topk({}, {})", render_nir_expr(input), k)
        }
        NirExpr::KernelTopkAxis { input, axis, k } => format!(
            "kernel_topk_axis({}, \"{}\", {})",
            render_nir_expr(input),
            axis.render(),
            k
        ),
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
        NirExpr::ShaderTexture2d {
            format,
            width,
            height,
            texels,
        } => format!(
            "shader_texture2d(\"{}\", {}, {}, \"{}\")",
            escape_debug(format),
            width,
            height,
            escape_debug(texels)
        ),
        NirExpr::ShaderSampler {
            filter,
            address_mode,
        } => format!(
            "shader_sampler(\"{}\", \"{}\")",
            escape_debug(filter),
            escape_debug(address_mode)
        ),
        NirExpr::ShaderUv { u, v } => format!("shader_uv({}, {})", u, v),
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            mode,
        } => format!(
            "shader_{}({}, {}, {}, {})",
            mode.render(),
            render_nir_expr(texture),
            render_nir_expr(sampler),
            render_nir_expr(x),
            render_nir_expr(y)
        ),
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            mode,
        } => format!(
            "shader_{}({}, {}, {})",
            mode.render(),
            render_nir_expr(texture),
            render_nir_expr(sampler),
            render_nir_expr(uv)
        ),
        NirExpr::ShaderBinding {
            kind,
            slot,
            layout,
            profile_contract: _,
            value,
        } => {
            let binding_callee = if kind == "uniform_binding" && layout.is_some() {
                if matches!(value.as_ref(), NirExpr::ShaderProfilePacket { .. })
                    && layout.as_deref() == Some("std140")
                {
                    "shader_packet_uniform_binding".to_owned()
                } else {
                    "shader_uniform_binding_layout".to_owned()
                }
            } else if kind == "storage_binding" && layout.is_some() {
                if matches!(value.as_ref(), NirExpr::ShaderProfilePacket { .. })
                    && layout.as_deref() == Some("std430")
                {
                    "shader_packet_storage_binding".to_owned()
                } else {
                    "shader_storage_binding_layout".to_owned()
                }
            } else {
                format!("shader_{kind}")
            };
            if let Some(layout) = layout {
                if matches!(
                    binding_callee.as_str(),
                    "shader_packet_uniform_binding" | "shader_packet_storage_binding"
                ) {
                    return format!("{}({}, {})", binding_callee, slot, render_nir_expr(value));
                }
                format!(
                    "{}({}, \"{}\", {})",
                    binding_callee,
                    slot,
                    escape_debug(layout),
                    render_nir_expr(value)
                )
            } else {
                format!("{}({}, {})", binding_callee, slot, render_nir_expr(value))
            }
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            let mut args = vec![render_nir_expr(pipeline)];
            args.extend(bindings.iter().map(render_nir_expr));
            format!("shader_bind_set({})", args.join(", "))
        }
        NirExpr::ShaderInlineWgsl { entry, source } => {
            render_shader_inline_wgsl_expr(entry, source)
        }
        NirExpr::ShaderResult { value, .. } => {
            format!("shader_result({})", render_nir_expr(value))
        }
        NirExpr::ShaderPassReady(result) => {
            format!("shader_pass_ready({})", render_nir_expr(result))
        }
        NirExpr::ShaderFrameReady(result) => {
            format!("shader_frame_ready({})", render_nir_expr(result))
        }
        NirExpr::ShaderValue(result) => format!("shader_value({})", render_nir_expr(result)),
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
        NirExpr::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            render_nir_expr(lhs),
            render_nir_binary_op(*op),
            render_nir_expr(rhs)
        ),
    }
}

fn render_nir_trait(definition: &NirTraitDef) -> String {
    let mut out = format!(
        "  {}trait {}\n",
        render_nir_visibility(definition.visibility),
        definition.name
    );
    for method in &definition.methods {
        out.push_str(&render_nir_trait_method_sig(method));
    }
    out
}

fn render_nir_impl(definition: &NirImplDef) -> String {
    let mut out = format!(
        "  impl {} for {}\n",
        definition.trait_name,
        render_nir_type(&definition.for_type)
    );
    for method in &definition.methods {
        out.push_str(&render_nir_impl_method(method));
    }
    out
}

fn render_nir_extern_interface(interface: &NirExternInterface) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "  {}extern \"{}\" interface {}\n",
        render_nir_visibility(interface.visibility),
        interface.abi,
        interface.name
    ));
    for function in &interface.methods {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
            .collect::<Vec<_>>()
            .join(", ");
        let host_prefix = function
            .host_symbol
            .as_ref()
            .map(|symbol| format!("@host_symbol(\"{}\") ", escape_debug(symbol)))
            .unwrap_or_default();
        out.push_str(&format!(
            "    {}{}fn {}({}) -> {}\n",
            render_nir_visibility(function.visibility),
            host_prefix,
            function.name,
            params,
            render_nir_type(&function.return_type)
        ));
    }
    out
}

fn render_ast_binary_op(op: AstBinaryOp) -> &'static str {
    match op {
        AstBinaryOp::And => "&&",
        AstBinaryOp::Or => "||",
        AstBinaryOp::Add => "+",
        AstBinaryOp::Sub => "-",
        AstBinaryOp::Mul => "*",
        AstBinaryOp::Div => "/",
        AstBinaryOp::Rem => "%",
        AstBinaryOp::Eq => "==",
        AstBinaryOp::Ne => "!=",
        AstBinaryOp::Lt => "<",
        AstBinaryOp::Le => "<=",
        AstBinaryOp::Gt => ">",
        AstBinaryOp::Ge => ">=",
    }
}

fn render_nir_binary_op(op: NirBinaryOp) -> &'static str {
    match op {
        NirBinaryOp::And => "&&",
        NirBinaryOp::Or => "||",
        NirBinaryOp::Add => "+",
        NirBinaryOp::Sub => "-",
        NirBinaryOp::Mul => "*",
        NirBinaryOp::Div => "/",
        NirBinaryOp::Rem => "%",
        NirBinaryOp::Eq => "==",
        NirBinaryOp::Ne => "!=",
        NirBinaryOp::Lt => "<",
        NirBinaryOp::Le => "<=",
        NirBinaryOp::Gt => ">",
        NirBinaryOp::Ge => ">=",
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

fn render_ast_generic_params(params: &[AstGenericParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let parts = params
        .iter()
        .map(render_ast_generic_param)
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", parts)
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

fn render_nir_generic_params(params: &[NirGenericParam]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let parts = params
        .iter()
        .map(render_nir_generic_param)
        .collect::<Vec<_>>()
        .join(", ");
    format!("<{}>", parts)
}

fn render_ast_generic_param(param: &AstGenericParam) -> String {
    if param.bounds.is_empty() {
        return param.name.clone();
    }
    format!(
        "{}: {}",
        param.name,
        param
            .bounds
            .iter()
            .map(render_ast_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

fn render_nir_generic_param(param: &NirGenericParam) -> String {
    if param.bounds.is_empty() {
        return param.name.clone();
    }
    format!(
        "{}: {}",
        param.name,
        param
            .bounds
            .iter()
            .map(render_nir_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

fn render_ast_function_header(function: &AstFunction) -> String {
    let mut out = render_ast_doc_comments("  ", &function.attributes);
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
    let test_prefix = function
        .test_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if function.test_ignored {
                parts.push("ignored=true".to_owned());
            }
            if function.test_should_fail {
                parts.push("should_fail=true".to_owned());
            }
            if let Some(reason) = &function.test_reason {
                parts.push(format!("reason=\"{}\"", reason));
            }
            if let Some(timeout_ms) = function.test_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.test_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.test_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("test({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let benchmark_prefix = function
        .benchmark_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if let Some(warmup_iters) = function.benchmark_warmup_iters {
                parts.push(format!("warmup_iters={warmup_iters}"));
            }
            if let Some(measure_iters) = function.benchmark_measure_iters {
                parts.push(format!("measure_iters={measure_iters}"));
            }
            if let Some(timeout_ms) = function.benchmark_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.benchmark_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.benchmark_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("benchmark({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    let attribute_prefix = render_ast_attributes(&function.attributes);
    let visibility_prefix = render_ast_visibility(function.visibility);
    let where_suffix = render_ast_where_clause(&function.where_bounds);
    out.push_str(&format!(
        "  {}{}{}{}{}fn {}{}({}){}{}\n",
        attribute_prefix,
        visibility_prefix,
        test_prefix,
        benchmark_prefix,
        async_prefix,
        function.name,
        render_ast_generic_params(&function.generic_params),
        params,
        return_suffix,
        where_suffix
    ));
    out
}

fn render_nir_function_header(function: &NirFunction) -> String {
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
    let test_prefix = function
        .test_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if function.test_ignored {
                parts.push("ignored=true".to_owned());
            }
            if function.test_should_fail {
                parts.push("should_fail=true".to_owned());
            }
            if let Some(reason) = &function.test_reason {
                parts.push(format!("reason=\"{}\"", reason));
            }
            if let Some(timeout_ms) = function.test_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.test_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.test_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("test({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let benchmark_prefix = function
        .benchmark_name
        .as_ref()
        .map(|name| {
            let mut parts = vec![format!("\"{}\"", name)];
            if let Some(warmup_iters) = function.benchmark_warmup_iters {
                parts.push(format!("warmup_iters={warmup_iters}"));
            }
            if let Some(measure_iters) = function.benchmark_measure_iters {
                parts.push(format!("measure_iters={measure_iters}"));
            }
            if let Some(timeout_ms) = function.benchmark_timeout_ms {
                parts.push(format!("timeout_ms={timeout_ms}"));
            }
            if let Some(clock_domain) = &function.benchmark_clock_domain {
                parts.push(format!("clock_domain=\"{}\"", clock_domain.as_str()));
            }
            if let Some(clock_policy) = &function.benchmark_clock_policy {
                parts.push(format!("clock_policy=\"{}\"", clock_policy.as_str()));
            }
            format!("benchmark({}) ", parts.join(", "))
        })
        .unwrap_or_default();
    let async_prefix = if function.is_async { "async " } else { "" };
    let annotation_prefix = render_nir_annotations(&function.annotations);
    let visibility_prefix = render_nir_visibility(function.visibility);
    let where_suffix = render_nir_where_clause(&function.where_bounds);
    format!(
        "  {}{}{}{}{}fn {}{}({}){}{}\n",
        annotation_prefix,
        visibility_prefix,
        test_prefix,
        benchmark_prefix,
        async_prefix,
        function.name,
        render_nir_generic_params(&function.generic_params),
        params,
        return_suffix,
        where_suffix
    )
}

fn render_ast_where_clause(predicates: &[AstWherePredicate]) -> String {
    if predicates.is_empty() {
        return String::new();
    }
    format!(
        " where {}",
        predicates
            .iter()
            .map(render_ast_where_predicate)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_nir_where_clause(predicates: &[NirWherePredicate]) -> String {
    if predicates.is_empty() {
        return String::new();
    }
    format!(
        " where {}",
        predicates
            .iter()
            .map(render_nir_where_predicate)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_ast_where_predicate(predicate: &AstWherePredicate) -> String {
    format!(
        "{}: {}",
        predicate.param_name,
        predicate
            .bounds
            .iter()
            .map(render_ast_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

fn render_nir_where_predicate(predicate: &NirWherePredicate) -> String {
    format!(
        "{}: {}",
        predicate.param_name,
        predicate
            .bounds
            .iter()
            .map(render_nir_type)
            .collect::<Vec<_>>()
            .join(" + ")
    )
}

fn render_ast_visibility(visibility: AstVisibility) -> &'static str {
    match visibility {
        AstVisibility::Private => "",
        AstVisibility::Public => "pub ",
    }
}

fn render_nir_visibility(visibility: NirVisibility) -> &'static str {
    match visibility {
        NirVisibility::Private => "",
        NirVisibility::Public => "pub ",
    }
}

fn render_ast_attributes(attributes: &[AstAttribute]) -> String {
    let rendered = attributes
        .iter()
        .filter(|attribute| !is_doc_attribute(attribute))
        .map(render_ast_attribute)
        .collect::<Vec<_>>();
    if rendered.is_empty() {
        return String::new();
    }
    format!("{} ", rendered.join(" "))
}

pub(super) fn render_ast_doc_comments(indent: &str, attributes: &[AstAttribute]) -> String {
    let mut out = String::new();
    for attribute in attributes {
        if !is_doc_attribute(attribute) {
            continue;
        }
        let Some(AstAttributeArg {
            name: None,
            value: AstAttributeValue::String(value),
        }) = attribute.args.first()
        else {
            continue;
        };
        out.push_str(&format!("{indent}/// {value}\n"));
    }
    out
}

fn is_doc_attribute(attribute: &AstAttribute) -> bool {
    attribute.name == "doc"
        && attribute.args.len() == 1
        && matches!(
            attribute.args.first(),
            Some(AstAttributeArg {
                name: None,
                value: AstAttributeValue::String(_),
            })
        )
}

fn render_ast_attribute(attribute: &AstAttribute) -> String {
    if attribute.args.is_empty() {
        return format!("@{}", attribute.name);
    }
    format!(
        "@{}({})",
        attribute.name,
        attribute
            .args
            .iter()
            .map(render_ast_attribute_arg)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_ast_attribute_arg(arg: &AstAttributeArg) -> String {
    let value = match &arg.value {
        AstAttributeValue::Bool(value) => value.to_string(),
        AstAttributeValue::Int(value) => value.to_string(),
        AstAttributeValue::String(value) => format!("\"{}\"", escape_debug(value)),
        AstAttributeValue::Ident(value) => value.clone(),
    };
    match &arg.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

fn render_nir_annotations(annotations: &[NirAnnotation]) -> String {
    if annotations.is_empty() {
        return String::new();
    }
    format!(
        "{} ",
        annotations
            .iter()
            .map(render_nir_annotation)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn render_nir_annotation(annotation: &NirAnnotation) -> String {
    if annotation.args.is_empty() {
        return format!("@{}", annotation.name);
    }
    format!(
        "@{}({})",
        annotation.name,
        annotation
            .args
            .iter()
            .map(render_nir_annotation_arg)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn render_nir_annotation_arg(arg: &NirAttributeArg) -> String {
    let value = match &arg.value {
        NirAttributeValue::Bool(value) => value.to_string(),
        NirAttributeValue::Int(value) => value.to_string(),
        NirAttributeValue::String(value) => format!("\"{}\"", escape_debug(value)),
        NirAttributeValue::Ident(value) => value.clone(),
    };
    match &arg.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

fn render_ast_trait_method_sig(method: &AstTraitMethodSig) -> String {
    let mut out = render_ast_doc_comments("    ", &method.attributes);
    let params = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type(ty)))
        .unwrap_or_default();
    out.push_str(&match &method.default_body {
        Some(body) => format!(
            "    fn {}({}){} {}\n",
            method.name,
            params,
            return_suffix,
            render_ast_stmt_block_inline(body)
        ),
        None => format!("    fn {}({}){};\n", method.name, params, return_suffix),
    });
    out
}

fn render_ast_stmt_block_inline(body: &[AstStmt]) -> String {
    format!(
        "{{ {} }}",
        body.iter()
            .map(render_ast_stmt_inline)
            .collect::<Vec<_>>()
            .join("; ")
    )
}

fn render_nir_trait_method_sig(method: &NirTraitMethodSig) -> String {
    let params = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_nir_type(ty)))
        .unwrap_or_default();
    format!("    fn {}({}){}\n", method.name, params, return_suffix)
}

fn render_ast_impl_method(method: &AstImplMethod) -> String {
    let params = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_ast_type(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_ast_type(ty)))
        .unwrap_or_default();
    format!("    fn {}({}){} ...\n", method.name, params, return_suffix)
}

fn render_nir_impl_method(method: &NirImplMethod) -> String {
    let params = method
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, render_nir_type(&param.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let return_suffix = method
        .return_type
        .as_ref()
        .map(|ty| format!(" -> {}", render_nir_type(ty)))
        .unwrap_or_default();
    format!("    fn {}({}){} ...\n", method.name, params, return_suffix)
}

fn render_ast_match_pattern(pattern: &AstMatchPattern) -> String {
    match pattern {
        AstMatchPattern::Wildcard => "_".to_owned(),
        AstMatchPattern::Bind(name) => name.clone(),
        AstMatchPattern::Bool(value) => value.to_string(),
        AstMatchPattern::Int(value) => value.to_string(),
        AstMatchPattern::IntRangeInclusive(start, end) => format!("{start}..={end}"),
        AstMatchPattern::Or(patterns) => patterns
            .iter()
            .map(render_ast_match_pattern)
            .collect::<Vec<_>>()
            .join(" | "),
        AstMatchPattern::PayloadStruct { type_ref, payload } => format!(
            "{}({})",
            render_ast_type(type_ref),
            render_ast_match_pattern(payload)
        ),
        AstMatchPattern::StructFields { type_ref, fields } => {
            let fields = fields
                .iter()
                .map(|(field, pattern)| format!("{field}: {}", render_ast_match_pattern(pattern)))
                .collect::<Vec<_>>()
                .join(", ");
            match type_ref {
                Some(type_ref) => format!("{} {{ {} }}", render_ast_type(type_ref), fields),
                None => format!("{{ {} }}", fields),
            }
        }
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
        NirStmt::While { .. } => "while ...".to_owned(),
        NirStmt::Break => "break".to_owned(),
        NirStmt::Continue => "continue".to_owned(),
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

fn render_shader_inline_wgsl_expr(entry: &str, source: &str) -> String {
    if !source.contains('\n') {
        return format!(
            "shader_inline_wgsl(\"{}\", \"{}\")",
            escape_debug(entry),
            escape_debug(source)
        );
    }

    let trimmed = source.trim();
    let mut out = String::new();
    out.push_str(&format!(
        "shader_inline_wgsl(\"{}\", wgsl {{\n",
        escape_debug(entry)
    ));
    for line in trimmed.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("})");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_mutable_and_reassigned_ast_locals() {
        let module = AstModule {
            attributes: vec![],
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![],
            type_aliases: vec![],
            structs: vec![],
            enums: vec![],
            traits: vec![],
            impls: vec![],
            functions: vec![AstFunction {
                visibility: AstVisibility::Private,
                name: "main".to_owned(),
                attributes: vec![],
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                where_bounds: vec![],
                params: vec![],
                return_type: Some(AstTypeRef {
                    name: "i64".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: false,
                }),
                body: vec![
                    AstStmt::Let {
                        mutable: true,
                        name: "value".to_owned(),
                        ty: Some(AstTypeRef {
                            name: "i64".to_owned(),
                            generic_args: vec![],
                            is_optional: false,
                            is_ref: false,
                        }),
                        value: AstExpr::Int(1),
                    },
                    AstStmt::AssignLocal {
                        name: "value".to_owned(),
                        value: AstExpr::Binary {
                            op: AstBinaryOp::Add,
                            lhs: Box::new(AstExpr::Var("value".to_owned())),
                            rhs: Box::new(AstExpr::Int(2)),
                        },
                    },
                    AstStmt::Return(Some(AstExpr::Var("value".to_owned()))),
                ],
            }],
        };

        let rendered = render_ast(&module);
        assert!(rendered.contains("let mut value: i64 = 1"), "{rendered}");
        assert!(rendered.contains("value = (value + 2)"), "{rendered}");
        assert!(rendered.contains("return value"), "{rendered}");
    }

    #[test]
    fn renders_enum_declarations_in_ast() {
        let module = AstModule {
            attributes: vec![],
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![],
            type_aliases: vec![],
            structs: vec![],
            enums: vec![nuis_semantics::model::AstEnumDef {
                visibility: AstVisibility::Public,
                attributes: vec![],
                name: "Option".to_owned(),
                generic_params: vec![AstGenericParam {
                    name: "T".to_owned(),
                    bounds: vec![],
                }],
                where_bounds: vec![],
                variants: vec![
                    nuis_semantics::model::AstEnumVariant {
                        attributes: vec![],
                        name: "None".to_owned(),
                        kind: nuis_semantics::model::AstEnumVariantKind::Unit,
                    },
                    nuis_semantics::model::AstEnumVariant {
                        attributes: vec![],
                        name: "Some".to_owned(),
                        kind: nuis_semantics::model::AstEnumVariantKind::Tuple(vec![AstTypeRef {
                            name: "T".to_owned(),
                            generic_args: vec![],
                            is_optional: false,
                            is_ref: false,
                        }]),
                    },
                ],
            }],
            traits: vec![],
            impls: vec![],
            functions: vec![],
        };

        let rendered = render_ast(&module);
        assert!(rendered.contains("pub enum Option<T>"), "{rendered}");
        assert!(rendered.contains("variant None"), "{rendered}");
        assert!(rendered.contains("variant Some(T)"), "{rendered}");
    }

    #[test]
    fn renders_doc_comments_as_triple_slash_lines() {
        let module = AstModule {
            attributes: vec![AstAttribute {
                name: "doc".to_owned(),
                args: vec![AstAttributeArg {
                    name: None,
                    value: AstAttributeValue::String("module docs".to_owned()),
                }],
            }],
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Docs".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![nuis_semantics::model::AstConstItem {
                visibility: AstVisibility::Private,
                attributes: vec![AstAttribute {
                    name: "doc".to_owned(),
                    args: vec![AstAttributeArg {
                        name: None,
                        value: AstAttributeValue::String("const docs".to_owned()),
                    }],
                }],
                name: "ANSWER".to_owned(),
                ty: Some(AstTypeRef {
                    name: "i32".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: false,
                }),
                value: AstExpr::Int(42),
            }],
            type_aliases: vec![],
            structs: vec![],
            enums: vec![nuis_semantics::model::AstEnumDef {
                visibility: AstVisibility::Private,
                attributes: vec![],
                name: "Maybe".to_owned(),
                generic_params: vec![],
                where_bounds: vec![],
                variants: vec![nuis_semantics::model::AstEnumVariant {
                    attributes: vec![AstAttribute {
                        name: "doc".to_owned(),
                        args: vec![AstAttributeArg {
                            name: None,
                            value: AstAttributeValue::String("empty docs".to_owned()),
                        }],
                    }],
                    name: "None".to_owned(),
                    kind: nuis_semantics::model::AstEnumVariantKind::Unit,
                }],
            }],
            traits: vec![AstTraitDef {
                visibility: AstVisibility::Private,
                attributes: vec![AstAttribute {
                    name: "doc".to_owned(),
                    args: vec![AstAttributeArg {
                        name: None,
                        value: AstAttributeValue::String("trait docs".to_owned()),
                    }],
                }],
                name: "Displayable".to_owned(),
                methods: vec![AstTraitMethodSig {
                    attributes: vec![AstAttribute {
                        name: "doc".to_owned(),
                        args: vec![AstAttributeArg {
                            name: None,
                            value: AstAttributeValue::String("render docs".to_owned()),
                        }],
                    }],
                    name: "render".to_owned(),
                    params: vec![nuis_semantics::model::AstParam {
                        name: "self".to_owned(),
                        ty: AstTypeRef {
                            name: "Self".to_owned(),
                            generic_args: vec![],
                            is_optional: false,
                            is_ref: false,
                        },
                    }],
                    return_type: Some(AstTypeRef {
                        name: "Text".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: false,
                    }),
                    default_body: None,
                }],
            }],
            impls: vec![],
            functions: vec![AstFunction {
                visibility: AstVisibility::Private,
                name: "answer".to_owned(),
                attributes: vec![AstAttribute {
                    name: "doc".to_owned(),
                    args: vec![AstAttributeArg {
                        name: None,
                        value: AstAttributeValue::String("function docs".to_owned()),
                    }],
                }],
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                benchmark_name: None,
                benchmark_warmup_iters: None,
                benchmark_measure_iters: None,
                benchmark_timeout_ms: None,
                benchmark_clock_domain: None,
                benchmark_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                where_bounds: vec![],
                params: vec![],
                return_type: Some(AstTypeRef {
                    name: "i32".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: false,
                }),
                body: vec![AstStmt::Return(Some(AstExpr::Int(42)))],
            }],
        };

        let rendered = render_ast(&module);
        assert!(rendered.starts_with("/// module docs\n"), "{rendered}");
        assert!(
            rendered.contains("/// const docs\n  const ANSWER: i32 = 42"),
            "{rendered}"
        );
        assert!(
            rendered.contains("/// empty docs\n    variant None"),
            "{rendered}"
        );
        assert!(
            rendered.contains("/// trait docs\n  trait Displayable"),
            "{rendered}"
        );
        assert!(
            rendered.contains("/// render docs\n    fn render(self: Self) -> Text;"),
            "{rendered}"
        );
        assert!(
            rendered.contains("/// function docs\n  fn answer() -> i32"),
            "{rendered}"
        );
        assert!(!rendered.contains("@doc("), "{rendered}");
    }

    #[test]
    fn renders_multiline_shader_inline_wgsl_as_wgsl_block() {
        let rendered = render_nir_expr(&NirExpr::ShaderInlineWgsl {
            entry: "demo_shader".to_owned(),
            source: r#"
struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main() -> VsOut {
  var out: VsOut;
  return out;
}
"#
            .trim()
            .to_owned(),
        });

        assert!(
            rendered.contains("shader_inline_wgsl(\"demo_shader\", wgsl {"),
            "{rendered}"
        );
        assert!(rendered.contains("@vertex"), "{rendered}");
        assert!(rendered.contains("\n})"), "{rendered}");
        assert!(!rendered.contains("\\n"), "{rendered}");
    }

    #[test]
    fn keeps_single_line_shader_inline_wgsl_as_string_literal() {
        let rendered = render_nir_expr(&NirExpr::ShaderInlineWgsl {
            entry: "demo_shader".to_owned(),
            source: "stub".to_owned(),
        });

        assert_eq!(rendered, "shader_inline_wgsl(\"demo_shader\", \"stub\")");
    }
}
