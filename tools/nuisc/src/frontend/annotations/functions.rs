use std::collections::BTreeSet;

use nuis_semantics::model::{
    AstAttributeArg, AstAttributeValue, AstConstItem, AstExpr, AstFunction, AstStmt,
};

pub(crate) fn validate_function_annotations(function: &AstFunction) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    let mut has_inline = false;
    let mut has_noinline = false;

    for attribute in &function.attributes {
        match attribute.name.as_str() {
            "doc" => {}
            "__nuisc_text_handle_rewrite" => {}
            "test" => {}
            "benchmark" => {}
            "inline" => {
                validate_zero_arg_function_annotation(function, "inline", &attribute.args)?;
                has_inline = true;
            }
            "noinline" => {
                validate_zero_arg_function_annotation(function, "noinline", &attribute.args)?;
                has_noinline = true;
            }
            "export" => validate_export_annotation(function, &attribute.args)?,
            "host_symbol" => validate_host_symbol_annotation(function, &attribute.args)?,
            other => {
                return Err(format!(
                    "function `{}` uses unknown annotation `@{other}`",
                    function.name
                ));
            }
        }

        if attribute.name != "doc" && !seen.insert(attribute.name.as_str()) {
            return Err(format!(
                "function `{}` repeats annotation `@{}`",
                function.name, attribute.name
            ));
        }
    }

    if has_inline && has_noinline {
        return Err(format!(
            "function `{}` cannot use both `@inline` and `@noinline`",
            function.name
        ));
    }

    Ok(())
}

pub(crate) fn validate_const_item(constant: &AstConstItem) -> Result<(), String> {
    validate_const_safe_expr(&constant.value).map_err(|reason| {
        format!(
            "top-level const `{}` currently requires a const-safe expression: {}",
            constant.name, reason
        )
    })
}

fn validate_const_safe_expr(expr: &AstExpr) -> Result<(), &'static str> {
    match expr {
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_) => Ok(()),
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_const_safe_expr(condition)?;
            validate_const_safe_block(then_body)?;
            validate_const_safe_block(else_body)
        }
        AstExpr::Match { value, arms } => {
            validate_const_safe_expr(value)?;
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    validate_const_safe_expr(guard)?;
                }
                validate_const_safe_block(&arm.body)?;
            }
            Ok(())
        }
        AstExpr::Lambda { .. } => Err("lambda expressions are not const-safe"),
        AstExpr::Unary { operand, .. } => validate_const_safe_expr(operand),
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_const_safe_expr(lhs)?;
            validate_const_safe_expr(rhs)
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_const_safe_expr(value)?;
            }
            Ok(())
        }
        AstExpr::FieldAccess { base, .. } => validate_const_safe_expr(base),
        AstExpr::Await(_) => Err("`await` is not const-safe"),
        AstExpr::Try(_) => Err("`?` is not const-safe"),
        AstExpr::Instantiate { .. } => Err("`instantiate` is not const-safe"),
        AstExpr::Call { .. } | AstExpr::Invoke { .. } | AstExpr::MethodCall { .. } => {
            Err("calls are not const-safe in the current MVP")
        }
    }
}

fn validate_const_safe_block(body: &[AstStmt]) -> Result<(), &'static str> {
    for stmt in body {
        match stmt {
            AstStmt::Let { value, .. }
            | AstStmt::AssignLocal { value, .. }
            | AstStmt::Const { value, .. }
            | AstStmt::Print(value)
            | AstStmt::Expr(value)
            | AstStmt::Await(value)
            | AstStmt::Return(Some(value)) => validate_const_safe_expr(value)?,
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                validate_const_safe_expr(condition)?;
                validate_const_safe_block(then_body)?;
                validate_const_safe_block(else_body)?;
            }
            AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
            AstStmt::DestructureLet { .. } | AstStmt::Match { .. } | AstStmt::While { .. } => {
                return Err("control-flow blocks are not const-safe")
            }
        }
    }
    Ok(())
}

fn validate_zero_arg_function_annotation(
    function: &AstFunction,
    annotation: &str,
    args: &[AstAttributeArg],
) -> Result<(), String> {
    if args.is_empty() {
        return Ok(());
    }
    Err(format!(
        "function `{}` annotation `@{annotation}` does not take arguments",
        function.name
    ))
}

fn validate_export_annotation(
    function: &AstFunction,
    args: &[AstAttributeArg],
) -> Result<(), String> {
    if args.len() != 1 {
        return Err(format!(
            "function `{}` annotation `@export` expects exactly one argument: `name = \"...\"`",
            function.name
        ));
    }
    let arg = &args[0];
    if arg.name.as_deref() != Some("name") {
        return Err(format!(
            "function `{}` annotation `@export` expects `name = \"...\"`",
            function.name
        ));
    }
    match &arg.value {
        AstAttributeValue::String(value) if !value.is_empty() => {
            if !is_valid_export_symbol_name(value) {
                return Err(format!(
                    "function `{}` annotation `@export(name = \"...\")` requires a C-style symbol name",
                    function.name
                ));
            }
            Ok(())
        }
        AstAttributeValue::String(_) => Err(format!(
            "function `{}` annotation `@export(name = \"...\")` requires a non-empty export name",
            function.name
        )),
        _ => Err(format!(
            "function `{}` annotation `@export` expects `name = \"...\"`",
            function.name
        )),
    }
}

fn is_valid_export_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn validate_host_symbol_annotation(
    function: &AstFunction,
    args: &[AstAttributeArg],
) -> Result<(), String> {
    if args.len() != 1 {
        return Err(format!(
            "function `{}` annotation `@host_symbol` expects exactly one string argument",
            function.name
        ));
    }
    let arg = &args[0];
    if arg.name.is_some() {
        return Err(format!(
            "function `{}` annotation `@host_symbol` expects `@host_symbol(\"...\")`",
            function.name
        ));
    }
    match &arg.value {
        AstAttributeValue::String(value) if !value.is_empty() => Ok(()),
        AstAttributeValue::String(_) => Err(format!(
            "function `{}` annotation `@host_symbol(\"...\")` requires a non-empty host symbol",
            function.name
        )),
        _ => Err(format!(
            "function `{}` annotation `@host_symbol` expects a string literal",
            function.name
        )),
    }
}
