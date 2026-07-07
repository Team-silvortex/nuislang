use super::*;
use std::fmt::Write as _;

pub(super) fn render_ast_unary_op(op: AstUnaryOp) -> &'static str {
    match op {
        AstUnaryOp::Not => "!",
        AstUnaryOp::Neg => "-",
        AstUnaryOp::Deref => "*",
    }
}

pub(super) fn render_ast_generic_args(args: &[AstTypeRef]) -> String {
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

pub(super) fn render_ast_trait(definition: &AstTraitDef) -> String {
    let mut out = String::new();
    out.push_str(&render_ast_doc_comments("  ", &definition.attributes));
    writeln!(
        out,
        "  {}{}trait {}",
        render_ast_attributes(&definition.attributes),
        render_ast_visibility(definition.visibility),
        definition.name
    )
    .unwrap();
    for method in &definition.methods {
        out.push_str(&render_ast_trait_method_sig(method));
    }
    out
}

pub(super) fn render_ast_impl(definition: &AstImplDef) -> String {
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

pub(super) fn render_ast_extern_interface(interface: &AstExternInterface) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "  {}extern \"{}\" interface {}",
        render_ast_visibility(interface.visibility),
        interface.abi,
        interface.name
    )
    .unwrap();
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
        writeln!(
            out,
            "    {}{}fn {}({}) -> {}",
            render_ast_visibility(function.visibility),
            host_prefix,
            function.name,
            params,
            render_ast_type(&function.return_type)
        )
        .unwrap();
    }
    out
}

pub(super) fn render_nir_trait(definition: &NirTraitDef) -> String {
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

pub(super) fn render_nir_impl(definition: &NirImplDef) -> String {
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

pub(super) fn render_nir_extern_interface(interface: &NirExternInterface) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "  {}extern \"{}\" interface {}",
        render_nir_visibility(interface.visibility),
        interface.abi,
        interface.name
    )
    .unwrap();
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
        writeln!(
            out,
            "    {}{}fn {}({}) -> {}",
            render_nir_visibility(function.visibility),
            host_prefix,
            function.name,
            params,
            render_nir_type(&function.return_type)
        )
        .unwrap();
    }
    out
}

pub(super) fn render_ast_binary_op(op: AstBinaryOp) -> &'static str {
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

pub(super) fn render_nir_binary_op(op: NirBinaryOp) -> &'static str {
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

pub(super) fn render_ast_trait_method_sig(method: &AstTraitMethodSig) -> String {
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

pub(super) fn render_ast_stmt_block_inline(body: &[AstStmt]) -> String {
    format!(
        "{{ {} }}",
        body.iter()
            .map(render_ast_stmt_inline)
            .collect::<Vec<_>>()
            .join("; ")
    )
}

pub(super) fn render_nir_trait_method_sig(method: &NirTraitMethodSig) -> String {
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

pub(super) fn render_ast_impl_method(method: &AstImplMethod) -> String {
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

pub(super) fn render_nir_impl_method(method: &NirImplMethod) -> String {
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

pub(super) fn render_ast_match_pattern(pattern: &AstMatchPattern) -> String {
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

pub(super) fn render_nir_stmt_inline(stmt: &NirStmt) -> String {
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

pub(super) fn escape_debug(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub(super) fn render_shader_inline_wgsl_expr(entry: &str, source: &str) -> String {
    if !source.contains('\n') {
        return format!(
            "shader_inline_wgsl(\"{}\", \"{}\")",
            escape_debug(entry),
            escape_debug(source)
        );
    }

    let trimmed = source.trim();
    let mut out = String::new();
    writeln!(
        out,
        "shader_inline_wgsl(\"{}\", wgsl {{",
        escape_debug(entry)
    )
    .unwrap();
    for line in trimmed.lines() {
        out.push_str("  ");
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("})");
    out
}
