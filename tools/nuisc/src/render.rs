use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstModule, AstStmt, NirBinaryOp, NirExpr, NirModule, NirStmt,
};
use yir_core::YirModule;

pub fn render_ast(module: &AstModule) -> String {
    let mut out = String::new();
    out.push_str(&format!("ast module {}::{}\n", module.domain, module.name));
    for function in &module.functions {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, param.ty.name))
            .collect::<Vec<_>>()
            .join(", ");
        let return_suffix = function
            .return_type
            .as_ref()
            .map(|ty| format!(" -> {}", ty.name))
            .unwrap_or_default();
        out.push_str(&format!("  fn {}({}){}\n", function.name, params, return_suffix));
        for stmt in &function.body {
            match stmt {
                AstStmt::Let { name, ty, value } => {
                    let type_suffix = ty
                        .as_ref()
                        .map(|ty| format!(": {}", ty.name))
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
    out.push_str(&format!("nir module {}::{}\n", module.domain, module.name));
    for function in &module.functions {
        let params = function
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, param.ty.name))
            .collect::<Vec<_>>()
            .join(", ");
        let return_suffix = function
            .return_type
            .as_ref()
            .map(|ty| format!(" -> {}", ty.name))
            .unwrap_or_default();
        out.push_str(&format!("  fn {}({}){}\n", function.name, params, return_suffix));
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { name, ty, value } => {
                    let type_suffix = ty
                        .as_ref()
                        .map(|ty| format!(": {}", ty.name))
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
        out.push_str(&format!(
            "{}.{} {} {}",
            node.op.module, node.op.instruction, node.name, node.resource
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
        AstExpr::Call { callee, args } => format!(
            "{}({})",
            callee,
            args.iter().map(render_ast_expr).collect::<Vec<_>>().join(", ")
        ),
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => format!(
            "{}.{}({})",
            render_ast_expr(receiver),
            method,
            args.iter().map(render_ast_expr).collect::<Vec<_>>().join(", ")
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

fn render_nir_expr(value: &NirExpr) -> String {
    match value {
        NirExpr::Bool(value) => value.to_string(),
        NirExpr::Text(text) => format!("\"{}\"", escape_debug(text)),
        NirExpr::Int(value) => value.to_string(),
        NirExpr::Var(name) => name.clone(),
        NirExpr::Call { callee, args } => format!(
            "{}({})",
            callee,
            args.iter().map(render_nir_expr).collect::<Vec<_>>().join(", ")
        ),
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => format!(
            "{}.{}({})",
            render_nir_expr(receiver),
            method,
            args.iter().map(render_nir_expr).collect::<Vec<_>>().join(", ")
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
            format!("const {}: {} = {}", name, render_ast_type(ty), render_ast_expr(value))
        }
        AstStmt::Print(value) => format!("print {}", render_ast_expr(value)),
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
            format!("const {}: {} = {}", name, render_nir_type(ty), render_nir_expr(value))
        }
        NirStmt::Print(value) => format!("print {}", render_nir_expr(value)),
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
