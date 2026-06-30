#[path = "render/attributes.rs"]
mod attributes;
#[path = "render/common.rs"]
mod common;
#[path = "render/render_ast_expr.rs"]
mod render_ast_expr;
#[path = "render/render_nir_expr.rs"]
mod render_nir_expr;
mod render_stmt_helpers;
mod render_struct_helpers;
#[path = "render/types_headers.rs"]
mod types_headers;

use self::attributes::*;
use self::common::*;
use self::render_ast_expr::render_ast_expr;
use self::render_nir_expr::render_nir_expr;
use self::render_stmt_helpers::{
    render_ast_destructure_let, render_ast_stmt_inline, render_ast_type_suffix,
};
use self::render_struct_helpers::{
    render_ast_enum, render_ast_struct, render_nir_enum, render_nir_struct,
    render_nir_type_arg_suffix,
};
use self::types_headers::*;
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

#[cfg(test)]
#[path = "render/tests.rs"]
mod tests;
