use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{nir_glm_profile, NirExpr, NirFunction, NirGlmUseMode, NirModule, NirStmt};

pub fn verify_nir_module(module: &NirModule) -> Result<(), String> {
    for function in &module.functions {
        verify_function(function)?;
    }
    Ok(())
}

fn verify_function(function: &NirFunction) -> Result<(), String> {
    let mut moved = BTreeSet::<String>::new();
    let mut borrows = BTreeMap::<String, usize>::new();

    for stmt in &function.body {
        verify_stmt(stmt, &mut moved, &mut borrows)?;
    }

    Ok(())
}

fn verify_stmt(
    stmt: &NirStmt,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
) -> Result<(), String> {
    match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            verify_expr(value, moved, borrows)?;
            note_binding_effects(value, moved, borrows);
            borrows.remove(name);
        }
        NirStmt::Print(value) | NirStmt::Expr(value) => {
            verify_expr(value, moved, borrows)?;
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            verify_expr(condition, moved, borrows)?;
            let mut then_moved = moved.clone();
            let mut then_borrows = borrows.clone();
            for stmt in then_body {
                verify_stmt(stmt, &mut then_moved, &mut then_borrows)?;
            }
            let mut else_moved = moved.clone();
            let mut else_borrows = borrows.clone();
            for stmt in else_body {
                verify_stmt(stmt, &mut else_moved, &mut else_borrows)?;
            }
        }
        NirStmt::Return(value) => {
            if let Some(value) = value {
                verify_expr(value, moved, borrows)?;
            }
        }
    }
    Ok(())
}

fn note_binding_effects(
    expr: &NirExpr,
    moved: &mut BTreeSet<String>,
    borrows: &mut BTreeMap<String, usize>,
) {
    match expr {
        NirExpr::Move(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                moved.insert(source.clone());
                borrows.remove(&source);
            }
        }
        NirExpr::Borrow(inner) => {
            if let Some(source) = expr_resource_key(inner) {
                *borrows.entry(source).or_insert(0) += 1;
            }
        }
        _ => {}
    }
}

fn verify_expr(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
) -> Result<(), String> {
    verify_expr_uses(expr, moved)?;

    if let Some(profile) = nir_glm_profile(expr) {
        if let Some(first_access) = profile.accesses.first() {
            match expr {
                NirExpr::Move(inner) | NirExpr::Free(inner) => {
                    if matches!(first_access.mode, NirGlmUseMode::Own) {
                        if let Some(source) = expr_resource_key(inner) {
                            if borrows.get(&source).copied().unwrap_or(0) > 0 {
                                return Err(format!(
                                    "nir verify: cannot consume `{}` while borrow(s) are active",
                                    source
                                ));
                            }
                        }
                    }
                }
                NirExpr::StoreValue { target, .. }
                | NirExpr::StoreNext { target, .. } => {
                    if let Some(source) = expr_resource_key(target) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot write `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::StoreAt { buffer, .. } => {
                    if let Some(source) = expr_resource_key(buffer) {
                        if borrows.get(&source).copied().unwrap_or(0) > 0 {
                            return Err(format!(
                                "nir verify: cannot write `{}` while borrow(s) are active",
                                source
                            ));
                        }
                    }
                }
                NirExpr::Borrow(inner) => {
                    if let Some(source) = expr_resource_key(inner) {
                        if moved.contains(&source) {
                            return Err(format!(
                                "nir verify: cannot borrow moved value `{}`",
                                source
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    match expr {
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::Instantiate { .. } => {}
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. } => {}
        NirExpr::CpuPresentFrame(inner) => verify_expr(inner, moved, borrows)?,
        NirExpr::CpuExternCall { args, .. } => {
            for arg in args {
                verify_expr(arg, moved, borrows)?;
            }
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            verify_expr(target, moved, borrows)?;
            verify_expr(pipeline, moved, borrows)?;
            verify_expr(viewport, moved, borrows)?;
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            verify_expr(pass, moved, borrows)?;
            verify_expr(packet, moved, borrows)?;
        }
        NirExpr::DataOutputPipe(inner) | NirExpr::DataInputPipe(inner) => {
            verify_expr(inner, moved, borrows)?
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            verify_expr(input, moved, borrows)?;
            verify_expr(offset, moved, borrows)?;
            verify_expr(len, moved, borrows)?;
        }
        NirExpr::Borrow(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => verify_expr(inner, moved, borrows)?,
        NirExpr::AllocNode { value, next } => {
            verify_expr(value, moved, borrows)?;
            verify_expr(next, moved, borrows)?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            verify_expr(len, moved, borrows)?;
            verify_expr(fill, moved, borrows)?;
        }
        NirExpr::LoadAt { buffer, index } => {
            verify_expr(buffer, moved, borrows)?;
            verify_expr(index, moved, borrows)?;
        }
        NirExpr::StoreValue { target, value } => {
            verify_expr(target, moved, borrows)?;
            verify_expr(value, moved, borrows)?;
        }
        NirExpr::StoreNext { target, next } => {
            verify_expr(target, moved, borrows)?;
            verify_expr(next, moved, borrows)?;
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            verify_expr(buffer, moved, borrows)?;
            verify_expr(index, moved, borrows)?;
            verify_expr(value, moved, borrows)?;
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                verify_expr(arg, moved, borrows)?;
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            verify_expr(receiver, moved, borrows)?;
            for arg in args {
                verify_expr(arg, moved, borrows)?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                verify_expr(value, moved, borrows)?;
            }
        }
        NirExpr::FieldAccess { base, .. } => verify_expr(base, moved, borrows)?,
        NirExpr::Binary { lhs, rhs, .. } => {
            verify_expr(lhs, moved, borrows)?;
            verify_expr(rhs, moved, borrows)?;
        }
    }

    Ok(())
}

fn verify_expr_uses(expr: &NirExpr, moved: &BTreeSet<String>) -> Result<(), String> {
    match expr {
        NirExpr::Var(_) | NirExpr::FieldAccess { .. } => {
            if let Some(name) = expr_resource_key(expr) {
                if moved.contains(&name) {
                    return Err(format!("nir verify: use of moved value `{}`", name));
                }
            }
        }
        NirExpr::Instantiate { .. } => {}
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. } => {}
        NirExpr::CpuPresentFrame(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::CpuExternCall { args, .. } => {
            for arg in args {
                verify_expr_uses(arg, moved)?;
            }
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            verify_expr_uses(target, moved)?;
            verify_expr_uses(pipeline, moved)?;
            verify_expr_uses(viewport, moved)?;
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            verify_expr_uses(pass, moved)?;
            verify_expr_uses(packet, moved)?;
        }
        NirExpr::DataOutputPipe(inner) | NirExpr::DataInputPipe(inner) => {
            verify_expr_uses(inner, moved)?
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            verify_expr_uses(input, moved)?;
            verify_expr_uses(offset, moved)?;
            verify_expr_uses(len, moved)?;
        }
        NirExpr::Borrow(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => verify_expr_uses(inner, moved)?,
        NirExpr::AllocNode { value, next } => {
            verify_expr_uses(value, moved)?;
            verify_expr_uses(next, moved)?;
        }
        NirExpr::AllocBuffer { len, fill } => {
            verify_expr_uses(len, moved)?;
            verify_expr_uses(fill, moved)?;
        }
        NirExpr::LoadAt { buffer, index } => {
            verify_expr_uses(buffer, moved)?;
            verify_expr_uses(index, moved)?;
        }
        NirExpr::StoreValue { target, value } => {
            verify_expr_uses(target, moved)?;
            verify_expr_uses(value, moved)?;
        }
        NirExpr::StoreNext { target, next } => {
            verify_expr_uses(target, moved)?;
            verify_expr_uses(next, moved)?;
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            verify_expr_uses(buffer, moved)?;
            verify_expr_uses(index, moved)?;
            verify_expr_uses(value, moved)?;
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                verify_expr_uses(arg, moved)?;
            }
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            verify_expr_uses(receiver, moved)?;
            for arg in args {
                verify_expr_uses(arg, moved)?;
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                verify_expr_uses(value, moved)?;
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            verify_expr_uses(lhs, moved)?;
            verify_expr_uses(rhs, moved)?;
        }
        NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Null => {}
    }
    Ok(())
}

fn expr_resource_key(expr: &NirExpr) -> Option<String> {
    match expr {
        NirExpr::Var(name) => Some(name.clone()),
        NirExpr::FieldAccess { base, field } => {
            let base = expr_resource_key(base)?;
            Some(format!("{base}.{field}"))
        }
        _ => None,
    }
}
