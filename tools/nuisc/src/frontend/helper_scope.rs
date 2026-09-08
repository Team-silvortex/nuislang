use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstExpr, AstFunction, AstMatchArm, AstModule, AstStmt, AstTypeAlias};

use super::{is_public_visibility, lower_type_ref_with_aliases, FunctionSignature};

pub(super) fn implementation_functions(helper: &AstModule) -> Vec<AstFunction> {
    let functions = helper
        .functions
        .iter()
        .map(|f| (f.name.as_str(), f))
        .collect::<BTreeMap<_, _>>();
    let mut pending = helper
        .functions
        .iter()
        .filter(|function| {
            is_public_visibility(function.visibility)
                || function.name.starts_with("__hof_")
                || function.name.starts_with("__lambda_")
        })
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    for definition in &helper.impls {
        for method in &definition.methods {
            collect_body(&method.body, &mut pending);
        }
    }
    for definition in &helper.traits {
        for method in &definition.methods {
            if let Some(body) = &method.default_body {
                collect_body(body, &mut pending);
            }
        }
    }
    let prefix = format!("{}.", helper.unit);
    let mut retained = BTreeSet::new();
    while let Some(name) = pending.pop_first() {
        let name = name.strip_prefix(&prefix).unwrap_or(&name);
        if let Some(function) = functions.get(name) {
            if retained.insert(name.to_owned()) {
                collect_body(&function.body, &mut pending);
            }
        }
    }
    helper
        .functions
        .iter()
        .filter(|f| retained.contains(&f.name))
        .cloned()
        .collect()
}

pub(super) fn insert_local_signatures(
    helper: &AstModule,
    functions: &[AstFunction],
    aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &mut BTreeMap<String, FunctionSignature>,
) -> Result<(), String> {
    for function in functions {
        let name = format!("{}.{}", helper.unit, function.name);
        let signature = FunctionSignature {
            abi: "nuis".into(),
            interface: None,
            symbol_name: name.clone(),
            params: function
                .params
                .iter()
                .map(|param| lower_type_ref_with_aliases(&param.ty, aliases))
                .collect::<Result<_, _>>()?,
            return_type: function
                .return_type
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, aliases))
                .transpose()?,
            is_extern: false,
            is_async: function.is_async,
        };
        // Owner-local names win over imports and the consumer's same-named functions.
        // This map is private to lowering this helper, never its export table.
        signatures.insert(name, signature.clone());
        signatures.insert(function.name.clone(), signature);
    }
    Ok(())
}

fn collect_body(body: &[AstStmt], names: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            AstStmt::Let { value, .. }
            | AstStmt::AssignLocal { value, .. }
            | AstStmt::DestructureLet { value, .. }
            | AstStmt::Const { value, .. }
            | AstStmt::Print(value)
            | AstStmt::Await(value)
            | AstStmt::Expr(value)
            | AstStmt::Return(Some(value)) => collect_expr(value, names),
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_expr(condition, names);
                collect_body(then_body, names);
                collect_body(else_body, names);
            }
            AstStmt::While { condition, body } => {
                collect_expr(condition, names);
                collect_body(body, names);
            }
            AstStmt::Match { value, arms } => collect_match(value, arms, names),
            AstStmt::Break | AstStmt::Continue | AstStmt::Return(None) => {}
        }
    }
}

fn collect_match(value: &AstExpr, arms: &[AstMatchArm], names: &mut BTreeSet<String>) {
    collect_expr(value, names);
    for arm in arms {
        if let Some(guard) = &arm.guard {
            collect_expr(guard, names);
        }
        collect_body(&arm.body, names);
    }
}

fn qualified_name(expr: &AstExpr) -> Option<String> {
    match expr {
        AstExpr::Var(name) => Some(name.clone()),
        AstExpr::FieldAccess { base, field } => Some(format!("{}.{field}", qualified_name(base)?)),
        _ => None,
    }
}

fn collect_expr(expr: &AstExpr, names: &mut BTreeSet<String>) {
    match expr {
        // Function values may be passed to higher-order calls; over-retention here
        // does not grant call visibility or bypass later binding/type checks.
        AstExpr::Var(name) => {
            names.insert(name.clone());
        }
        AstExpr::Call { callee, args, .. } => {
            names.insert(callee.clone());
            for arg in args {
                collect_expr(arg, names);
            }
        }
        AstExpr::Invoke { callee, args } => {
            collect_expr(callee, names);
            for arg in args {
                collect_expr(arg, names);
            }
        }
        AstExpr::MethodCall {
            receiver,
            method,
            args,
            ..
        } => {
            if let Some(base) = qualified_name(receiver) {
                names.insert(format!("{base}.{method}"));
            }
            collect_expr(receiver, names);
            for arg in args {
                collect_expr(arg, names);
            }
        }
        AstExpr::FieldAccess { base, .. } => {
            if let Some(name) = qualified_name(expr) {
                names.insert(name);
            }
            collect_expr(base, names);
        }
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_expr(condition, names);
            collect_body(then_body, names);
            collect_body(else_body, names);
        }
        AstExpr::Match { value, arms } => collect_match(value, arms, names),
        AstExpr::Lambda { body, .. } => collect_body(body, names),
        AstExpr::Await(value) | AstExpr::Try(value) | AstExpr::Unary { operand: value, .. } => {
            collect_expr(value, names)
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_expr(value, names);
            }
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            collect_expr(lhs, names);
            collect_expr(rhs, names);
        }
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Instantiate { .. } => {}
    }
}
