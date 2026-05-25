use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    nir_expr_effect_class, NirBinaryOp, NirExpr, NirExprEffectClass, NirFunction, NirModule,
    NirStmt,
};

pub fn simplify_nir_module(module: &mut NirModule) -> bool {
    let mut changed = false;
    for function in &mut module.functions {
        changed |= simplify_nir_function(function);
    }
    changed
}

fn simplify_nir_function(function: &mut NirFunction) -> bool {
    let mut env = BTreeMap::new();
    simplify_stmt_block(&mut function.body, &mut env)
}

fn simplify_stmt_block(stmts: &mut Vec<NirStmt>, env: &mut BTreeMap<String, NirExpr>) -> bool {
    let mut changed = false;
    let original = std::mem::take(stmts);
    let mut rewritten = Vec::with_capacity(original.len());
    for stmt in original {
        changed |= rewrite_stmt(stmt, &mut rewritten, env);
    }
    changed |= prune_dead_scalar_bindings(&mut rewritten);
    *stmts = rewritten;
    changed
}

fn rewrite_stmt(
    stmt: NirStmt,
    out: &mut Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
) -> bool {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            let (value, changed) = simplify_expr(value, env);
            refresh_literal_binding(env, &name, &value);
            out.push(NirStmt::Let { name, ty, value });
            changed
        }
        NirStmt::Const { name, ty, value } => {
            let (value, changed) = simplify_expr(value, env);
            refresh_literal_binding(env, &name, &value);
            out.push(NirStmt::Const { name, ty, value });
            changed
        }
        NirStmt::Print(value) => {
            let (value, changed) = simplify_expr(value, env);
            out.push(NirStmt::Print(value));
            changed
        }
        NirStmt::Await(value) => {
            let (value, changed) = simplify_expr(value, env);
            out.push(NirStmt::Await(value));
            changed
        }
        NirStmt::Expr(value) => {
            let (value, changed) = simplify_expr(value, env);
            out.push(NirStmt::Expr(value));
            changed
        }
        NirStmt::Return(value) => {
            let (value, changed) = match value {
                Some(value) => {
                    let (value, changed) = simplify_expr(value, env);
                    (Some(value), changed)
                }
                None => (None, false),
            };
            out.push(NirStmt::Return(value));
            changed
        }
        NirStmt::If {
            condition,
            mut then_body,
            mut else_body,
        } => {
            let (condition, mut changed) = simplify_expr(condition, env);
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            changed |= simplify_stmt_block(&mut then_body, &mut then_env);
            changed |= simplify_stmt_block(&mut else_body, &mut else_env);
            match condition {
                NirExpr::Bool(true) => {
                    out.extend(then_body);
                    true
                }
                NirExpr::Bool(false) => {
                    out.extend(else_body);
                    true
                }
                other => {
                    out.push(NirStmt::If {
                        condition: other,
                        then_body,
                        else_body,
                    });
                    changed
                }
            }
        }
    }
}

fn simplify_expr(expr: NirExpr, env: &BTreeMap<String, NirExpr>) -> (NirExpr, bool) {
    match expr {
        NirExpr::Var(name) => match env.get(&name) {
            Some(value) => (value.clone(), true),
            None => (NirExpr::Var(name), false),
        },
        NirExpr::Await(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::Await(Box::new(inner)), changed)
        }
        NirExpr::Borrow(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::Borrow(Box::new(inner)), changed)
        }
        NirExpr::BorrowEnd(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::BorrowEnd(Box::new(inner)), changed)
        }
        NirExpr::Move(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::Move(Box::new(inner)), changed)
        }
        NirExpr::AllocNode { value, next } => {
            let (value, left) = simplify_expr(*value, env);
            let (next, right) = simplify_expr(*next, env);
            (
                NirExpr::AllocNode {
                    value: Box::new(value),
                    next: Box::new(next),
                },
                left || right,
            )
        }
        NirExpr::AllocBuffer { len, fill } => {
            let (len, left) = simplify_expr(*len, env);
            let (fill, right) = simplify_expr(*fill, env);
            (
                NirExpr::AllocBuffer {
                    len: Box::new(len),
                    fill: Box::new(fill),
                },
                left || right,
            )
        }
        NirExpr::LoadValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::LoadValue(Box::new(inner)), changed)
        }
        NirExpr::LoadNext(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::LoadNext(Box::new(inner)), changed)
        }
        NirExpr::BufferLen(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::BufferLen(Box::new(inner)), changed)
        }
        NirExpr::LoadAt { buffer, index } => {
            let (buffer, left) = simplify_expr(*buffer, env);
            let (index, right) = simplify_expr(*index, env);
            (
                NirExpr::LoadAt {
                    buffer: Box::new(buffer),
                    index: Box::new(index),
                },
                left || right,
            )
        }
        NirExpr::StoreValue { target, value } => {
            let (target, left) = simplify_expr(*target, env);
            let (value, right) = simplify_expr(*value, env);
            (
                NirExpr::StoreValue {
                    target: Box::new(target),
                    value: Box::new(value),
                },
                left || right,
            )
        }
        NirExpr::StoreNext { target, next } => {
            let (target, left) = simplify_expr(*target, env);
            let (next, right) = simplify_expr(*next, env);
            (
                NirExpr::StoreNext {
                    target: Box::new(target),
                    next: Box::new(next),
                },
                left || right,
            )
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            let (buffer, a) = simplify_expr(*buffer, env);
            let (index, b) = simplify_expr(*index, env);
            let (value, c) = simplify_expr(*value, env);
            (
                NirExpr::StoreAt {
                    buffer: Box::new(buffer),
                    index: Box::new(index),
                    value: Box::new(value),
                },
                a || b || c,
            )
        }
        NirExpr::DataOutputPipe(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataOutputPipe(Box::new(inner)), changed)
        }
        NirExpr::DataInputPipe(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataInputPipe(Box::new(inner)), changed)
        }
        NirExpr::DataResult { value, state } => {
            let (value, changed) = simplify_expr(*value, env);
            (
                NirExpr::DataResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            )
        }
        NirExpr::DataReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataReady(Box::new(inner)), changed)
        }
        NirExpr::DataMoved(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataMoved(Box::new(inner)), changed)
        }
        NirExpr::DataWindowed(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataWindowed(Box::new(inner)), changed)
        }
        NirExpr::DataValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataValue(Box::new(inner)), changed)
        }
        NirExpr::DataCopyWindow { input, offset, len } => {
            let (input, a) = simplify_expr(*input, env);
            let (offset, b) = simplify_expr(*offset, env);
            let (len, c) = simplify_expr(*len, env);
            (
                NirExpr::DataCopyWindow {
                    input: Box::new(input),
                    offset: Box::new(offset),
                    len: Box::new(len),
                },
                a || b || c,
            )
        }
        NirExpr::DataReadWindow { window, index } => {
            let (window, left) = simplify_expr(*window, env);
            let (index, right) = simplify_expr(*index, env);
            (
                NirExpr::DataReadWindow {
                    window: Box::new(window),
                    index: Box::new(index),
                },
                left || right,
            )
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            let (window, a) = simplify_expr(*window, env);
            let (index, b) = simplify_expr(*index, env);
            let (value, c) = simplify_expr(*value, env);
            (
                NirExpr::DataWriteWindow {
                    window: Box::new(window),
                    index: Box::new(index),
                    value: Box::new(value),
                },
                a || b || c,
            )
        }
        NirExpr::DataFreezeWindow(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::DataFreezeWindow(Box::new(inner)), changed)
        }
        NirExpr::DataImmutableWindow { input, offset, len } => {
            let (input, a) = simplify_expr(*input, env);
            let (offset, b) = simplify_expr(*offset, env);
            let (len, c) = simplify_expr(*len, env);
            (
                NirExpr::DataImmutableWindow {
                    input: Box::new(input),
                    offset: Box::new(offset),
                    len: Box::new(len),
                },
                a || b || c,
            )
        }
        NirExpr::CpuJoin(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuJoin(Box::new(inner)), changed)
        }
        NirExpr::CpuCancel(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuCancel(Box::new(inner)), changed)
        }
        NirExpr::CpuJoinResult(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuJoinResult(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskCompleted(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuTaskCompleted(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskTimedOut(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuTaskTimedOut(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskCancelled(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuTaskCancelled(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuTaskValue(Box::new(inner)), changed)
        }
        NirExpr::CpuTimeout { task, limit } => {
            let (task, left) = simplify_expr(*task, env);
            let (limit, right) = simplify_expr(*limit, env);
            (
                NirExpr::CpuTimeout {
                    task: Box::new(task),
                    limit: Box::new(limit),
                },
                left || right,
            )
        }
        NirExpr::CpuPresentFrame(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::CpuPresentFrame(Box::new(inner)), changed)
        }
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => {
            let (base, left) = simplify_expr(*base, env);
            let (delta, right) = simplify_expr(*delta, env);
            (
                NirExpr::ShaderProfileColorSeed {
                    unit,
                    base: Box::new(base),
                    delta: Box::new(delta),
                },
                left || right,
            )
        }
        NirExpr::ShaderProfileSpeedSeed {
            unit,
            delta,
            scale,
            base,
        } => {
            let (delta, a) = simplify_expr(*delta, env);
            let (scale, b) = simplify_expr(*scale, env);
            let (base, c) = simplify_expr(*base, env);
            (
                NirExpr::ShaderProfileSpeedSeed {
                    unit,
                    delta: Box::new(delta),
                    scale: Box::new(scale),
                    base: Box::new(base),
                },
                a || b || c,
            )
        }
        NirExpr::ShaderProfileRadiusSeed { unit, base, delta } => {
            let (base, left) = simplify_expr(*base, env);
            let (delta, right) = simplify_expr(*delta, env);
            (
                NirExpr::ShaderProfileRadiusSeed {
                    unit,
                    base: Box::new(base),
                    delta: Box::new(delta),
                },
                left || right,
            )
        }
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
            let (color, a) = simplify_expr(*color, env);
            let (speed, b) = simplify_expr(*speed, env);
            let (radius, c) = simplify_expr(*radius, env);
            let (accent, d) = simplify_optional_box_expr(accent, env);
            let (toggle_state, e) = simplify_optional_box_expr(toggle_state, env);
            let (focus_index, f) = simplify_optional_box_expr(focus_index, env);
            (
                NirExpr::ShaderProfilePacket {
                    unit,
                    packet_type_name,
                    color: Box::new(color),
                    speed: Box::new(speed),
                    radius: Box::new(radius),
                    accent,
                    toggle_state,
                    focus_index,
                },
                a || b || c || d || e || f,
            )
        }
        NirExpr::KernelResult { value, state } => {
            let (value, changed) = simplify_expr(*value, env);
            (
                NirExpr::KernelResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            )
        }
        NirExpr::KernelConfigReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::KernelConfigReady(Box::new(inner)), changed)
        }
        NirExpr::KernelValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::KernelValue(Box::new(inner)), changed)
        }
        NirExpr::DataProfileSendUplink { unit, input } => {
            let (input, changed) = simplify_expr(*input, env);
            (
                NirExpr::DataProfileSendUplink {
                    unit,
                    input: Box::new(input),
                },
                changed,
            )
        }
        NirExpr::DataProfileSendDownlink { unit, input } => {
            let (input, changed) = simplify_expr(*input, env);
            (
                NirExpr::DataProfileSendDownlink {
                    unit,
                    input: Box::new(input),
                },
                changed,
            )
        }
        NirExpr::ShaderResult { value, state } => {
            let (value, changed) = simplify_expr(*value, env);
            (
                NirExpr::ShaderResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            )
        }
        NirExpr::ShaderPassReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::ShaderPassReady(Box::new(inner)), changed)
        }
        NirExpr::ShaderFrameReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::ShaderFrameReady(Box::new(inner)), changed)
        }
        NirExpr::ShaderValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::ShaderValue(Box::new(inner)), changed)
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            let (target, a) = simplify_expr(*target, env);
            let (pipeline, b) = simplify_expr(*pipeline, env);
            let (viewport, c) = simplify_expr(*viewport, env);
            (
                NirExpr::ShaderBeginPass {
                    target: Box::new(target),
                    pipeline: Box::new(pipeline),
                    viewport: Box::new(viewport),
                },
                a || b || c,
            )
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            let (pass, a) = simplify_expr(*pass, env);
            let (packet, b) = simplify_expr(*packet, env);
            let (vertex_count, c) = simplify_expr(*vertex_count, env);
            let (instance_count, d) = simplify_expr(*instance_count, env);
            (
                NirExpr::ShaderDrawInstanced {
                    pass: Box::new(pass),
                    packet: Box::new(packet),
                    vertex_count: Box::new(vertex_count),
                    instance_count: Box::new(instance_count),
                },
                a || b || c || d,
            )
        }
        NirExpr::ShaderProfileRender { unit, packet } => {
            let (packet, changed) = simplify_expr(*packet, env);
            (
                NirExpr::ShaderProfileRender {
                    unit,
                    packet: Box::new(packet),
                },
                changed,
            )
        }
        NirExpr::CpuExternCall {
            abi,
            interface,
            callee,
            args,
        } => {
            let (args, changed) = simplify_expr_vec(args, env);
            (
                NirExpr::CpuExternCall {
                    abi,
                    interface,
                    callee,
                    args,
                },
                changed,
            )
        }
        NirExpr::Free(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            (NirExpr::Free(Box::new(inner)), changed)
        }
        NirExpr::IsNull(inner) => {
            let (inner, changed) = simplify_expr(*inner, env);
            match inner {
                NirExpr::Null => (NirExpr::Bool(true), true),
                other => (NirExpr::IsNull(Box::new(other)), changed),
            }
        }
        NirExpr::Call { callee, args } => {
            let (args, changed) = simplify_expr_vec(args, env);
            (NirExpr::Call { callee, args }, changed)
        }
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let (receiver, left) = simplify_expr(*receiver, env);
            let (args, right) = simplify_expr_vec(args, env);
            (
                NirExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method,
                    args,
                },
                left || right,
            )
        }
        NirExpr::StructLiteral { type_name, fields } => {
            let mut changed = false;
            let fields = fields
                .into_iter()
                .map(|(name, value)| {
                    let (value, value_changed) = simplify_expr(value, env);
                    changed |= value_changed;
                    (name, value)
                })
                .collect();
            (NirExpr::StructLiteral { type_name, fields }, changed)
        }
        NirExpr::FieldAccess { base, field } => {
            let (base, changed) = simplify_expr(*base, env);
            (
                NirExpr::FieldAccess {
                    base: Box::new(base),
                    field,
                },
                changed,
            )
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let (lhs, left) = simplify_expr(*lhs, env);
            let (rhs, right) = simplify_expr(*rhs, env);
            if let (NirExpr::Int(lhs_value), NirExpr::Int(rhs_value)) = (&lhs, &rhs) {
                if let Some(folded) = fold_int_binary(op, *lhs_value, *rhs_value) {
                    return (NirExpr::Int(folded), true);
                }
            }
            (
                NirExpr::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                left || right,
            )
        }
        other => (other, false),
    }
}

fn simplify_expr_vec(
    values: Vec<NirExpr>,
    env: &BTreeMap<String, NirExpr>,
) -> (Vec<NirExpr>, bool) {
    let mut changed = false;
    let values = values
        .into_iter()
        .map(|value| {
            let (value, value_changed) = simplify_expr(value, env);
            changed |= value_changed;
            value
        })
        .collect();
    (values, changed)
}

fn simplify_optional_box_expr(
    value: Option<Box<NirExpr>>,
    env: &BTreeMap<String, NirExpr>,
) -> (Option<Box<NirExpr>>, bool) {
    match value {
        Some(value) => {
            let (value, changed) = simplify_expr(*value, env);
            (Some(Box::new(value)), changed)
        }
        None => (None, false),
    }
}

fn refresh_literal_binding(env: &mut BTreeMap<String, NirExpr>, name: &str, value: &NirExpr) {
    match literal_binding_value(value) {
        Some(value) => {
            env.insert(name.to_owned(), value);
        }
        None => {
            env.remove(name);
        }
    }
}

fn literal_binding_value(value: &NirExpr) -> Option<NirExpr> {
    match value {
        NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Null => {
            Some(value.clone())
        }
        _ => None,
    }
}

fn prune_dead_scalar_bindings(stmts: &mut Vec<NirStmt>) -> bool {
    let mut changed = false;
    let mut live_after = BTreeSet::new();
    let mut kept = Vec::with_capacity(stmts.len());

    for stmt in stmts.drain(..).rev() {
        let (maybe_stmt, live_before, stmt_changed) = prune_stmt(stmt, &live_after);
        changed |= stmt_changed;
        if let Some(stmt) = maybe_stmt {
            kept.push(stmt);
        }
        live_after = live_before;
    }

    kept.reverse();
    *stmts = kept;
    changed
}

fn prune_stmt(
    stmt: NirStmt,
    live_after: &BTreeSet<String>,
) -> (Option<NirStmt>, BTreeSet<String>, bool) {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            let mut live_before = live_after.clone();
            if !live_after.contains(&name) && expr_is_dead_binding_safe(&value) {
                collect_used_vars_expr(&value, &mut live_before);
                return (None, live_before, true);
            }
            live_before.remove(&name);
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Let { name, ty, value }), live_before, false)
        }
        NirStmt::Const { name, ty, value } => {
            let mut live_before = live_after.clone();
            if !live_after.contains(&name) && expr_is_dead_binding_safe(&value) {
                collect_used_vars_expr(&value, &mut live_before);
                return (None, live_before, true);
            }
            live_before.remove(&name);
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Const { name, ty, value }), live_before, false)
        }
        NirStmt::Print(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Print(value)), live_before, false)
        }
        NirStmt::Await(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Await(value)), live_before, false)
        }
        NirStmt::Expr(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(&value, &mut live_before);
            (Some(NirStmt::Expr(value)), live_before, false)
        }
        NirStmt::Return(value) => {
            let mut live_before = live_after.clone();
            if let Some(value) = &value {
                collect_used_vars_expr(value, &mut live_before);
            }
            (Some(NirStmt::Return(value)), live_before, false)
        }
        NirStmt::If {
            condition,
            mut then_body,
            mut else_body,
        } => {
            let mut changed = false;
            changed |= prune_dead_scalar_bindings(&mut then_body);
            changed |= prune_dead_scalar_bindings(&mut else_body);

            let then_live = live_before_block(&then_body, live_after);
            let else_live = live_before_block(&else_body, live_after);
            let mut live_before = live_after.clone();
            live_before.extend(then_live);
            live_before.extend(else_live);
            collect_used_vars_expr(&condition, &mut live_before);

            (
                Some(NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                }),
                live_before,
                changed,
            )
        }
    }
}

fn live_before_block(stmts: &[NirStmt], live_after: &BTreeSet<String>) -> BTreeSet<String> {
    let mut live = live_after.clone();
    for stmt in stmts.iter().rev() {
        live = live_before_stmt(stmt, &live);
    }
    live
}

fn live_before_stmt(stmt: &NirStmt, live_after: &BTreeSet<String>) -> BTreeSet<String> {
    match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
            let mut live_before = live_after.clone();
            live_before.remove(name);
            collect_used_vars_expr(value, &mut live_before);
            live_before
        }
        NirStmt::Print(value) | NirStmt::Await(value) | NirStmt::Expr(value) => {
            let mut live_before = live_after.clone();
            collect_used_vars_expr(value, &mut live_before);
            live_before
        }
        NirStmt::Return(value) => {
            let mut live_before = live_after.clone();
            if let Some(value) = value {
                collect_used_vars_expr(value, &mut live_before);
            }
            live_before
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let then_live = live_before_block(then_body, live_after);
            let else_live = live_before_block(else_body, live_after);
            let mut live_before = live_after.clone();
            live_before.extend(then_live);
            live_before.extend(else_live);
            collect_used_vars_expr(condition, &mut live_before);
            live_before
        }
    }
}

fn expr_is_dead_binding_safe(expr: &NirExpr) -> bool {
    if nir_expr_effect_class(expr) != NirExprEffectClass::Pure {
        return false;
    }
    match expr {
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_is_dead_binding_safe(lhs) && expr_is_dead_binding_safe(rhs)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| expr_is_dead_binding_safe(value)),
        NirExpr::FieldAccess { base, .. } => expr_is_dead_binding_safe(base),
        _ => true,
    }
}

fn collect_used_vars_expr(expr: &NirExpr, out: &mut BTreeSet<String>) {
    match expr {
        NirExpr::Var(name) => {
            out.insert(name.clone());
        }
        NirExpr::KernelTensor { .. } => {}
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::KernelArgmaxAxis { input: inner, .. }
        | NirExpr::KernelArgminAxis { input: inner, .. }
        | NirExpr::KernelReduceMaxAxis { input: inner, .. }
        | NirExpr::KernelReduceMeanAxis { input: inner, .. }
        | NirExpr::KernelReduceSumAxis { input: inner, .. }
        | NirExpr::KernelSort(inner)
        | NirExpr::KernelSortAxis { input: inner, .. }
        | NirExpr::KernelTopkAxis { input: inner, .. }
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkResult { value: inner, .. }
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => collect_used_vars_expr(inner, out),
        NirExpr::KernelMatmul { lhs, rhs } => {
            collect_used_vars_expr(lhs, out);
            collect_used_vars_expr(rhs, out);
        }
        NirExpr::KernelElementAt { input, row, col } => {
            collect_used_vars_expr(input, out);
            collect_used_vars_expr(row, out);
            collect_used_vars_expr(col, out);
        }
        NirExpr::KernelReshape { input, .. } => {
            collect_used_vars_expr(input, out);
        }
        NirExpr::KernelBroadcast { input, .. } => {
            collect_used_vars_expr(input, out);
        }
        NirExpr::KernelMap { input, scalar, .. } => {
            collect_used_vars_expr(input, out);
            if let Some(scalar) = scalar {
                collect_used_vars_expr(scalar, out);
            }
        }
        NirExpr::KernelMapAxis { input, scalar, .. } => {
            collect_used_vars_expr(input, out);
            if let Some(scalar) = scalar {
                collect_used_vars_expr(scalar, out);
            }
        }
        NirExpr::KernelTopk { input, .. } => {
            collect_used_vars_expr(input, out);
        }
        NirExpr::KernelZip { lhs, rhs, .. } => {
            collect_used_vars_expr(lhs, out);
            collect_used_vars_expr(rhs, out);
        }
        NirExpr::KernelAddBias { input, bias } => {
            collect_used_vars_expr(input, out);
            collect_used_vars_expr(bias, out);
        }
        NirExpr::AllocNode { value, next } => {
            collect_used_vars_expr(value, out);
            collect_used_vars_expr(next, out);
        }
        NirExpr::AllocBuffer { len, fill } => {
            collect_used_vars_expr(len, out);
            collect_used_vars_expr(fill, out);
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            collect_used_vars_expr(buffer, out);
            collect_used_vars_expr(index, out);
        }
        NirExpr::StoreValue { target, value } => {
            collect_used_vars_expr(target, out);
            collect_used_vars_expr(value, out);
        }
        NirExpr::StoreNext { target, next } => {
            collect_used_vars_expr(target, out);
            collect_used_vars_expr(next, out);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        }
        | NirExpr::DataWriteWindow {
            window: buffer,
            index,
            value,
        } => {
            collect_used_vars_expr(buffer, out);
            collect_used_vars_expr(index, out);
            collect_used_vars_expr(value, out);
        }
        NirExpr::DataResult { value, .. }
        | NirExpr::KernelResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::DataProfileSendUplink { input: value, .. }
        | NirExpr::DataProfileSendDownlink { input: value, .. }
        | NirExpr::ShaderProfileRender { packet: value, .. } => collect_used_vars_expr(value, out),
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            collect_used_vars_expr(input, out);
            collect_used_vars_expr(offset, out);
            collect_used_vars_expr(len, out);
        }
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                collect_used_vars_expr(arg, out);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            collect_used_vars_expr(task, out);
            collect_used_vars_expr(limit, out);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            collect_used_vars_expr(base, out);
            collect_used_vars_expr(delta, out);
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            collect_used_vars_expr(delta, out);
            collect_used_vars_expr(scale, out);
            collect_used_vars_expr(base, out);
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
            collect_used_vars_expr(color, out);
            collect_used_vars_expr(speed, out);
            collect_used_vars_expr(radius, out);
            if let Some(accent) = accent {
                collect_used_vars_expr(accent, out);
            }
            if let Some(toggle_state) = toggle_state {
                collect_used_vars_expr(toggle_state, out);
            }
            if let Some(focus_index) = focus_index {
                collect_used_vars_expr(focus_index, out);
            }
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            collect_used_vars_expr(target, out);
            collect_used_vars_expr(pipeline, out);
            collect_used_vars_expr(viewport, out);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            collect_used_vars_expr(pass, out);
            collect_used_vars_expr(packet, out);
            collect_used_vars_expr(vertex_count, out);
            collect_used_vars_expr(instance_count, out);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            collect_used_vars_expr(receiver, out);
            for arg in args {
                collect_used_vars_expr(arg, out);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_used_vars_expr(value, out);
            }
        }
        NirExpr::FieldAccess { base, .. } => collect_used_vars_expr(base, out),
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_used_vars_expr(lhs, out);
            collect_used_vars_expr(rhs, out);
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::Instantiate { .. }
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::DataProfileBindCoreRef { .. }
        | NirExpr::DataProfileWindowOffsetRef { .. }
        | NirExpr::DataProfileUplinkLenRef { .. }
        | NirExpr::DataProfileDownlinkLenRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::Null => {}
    }
}

fn fold_int_binary(op: NirBinaryOp, lhs: i64, rhs: i64) -> Option<i64> {
    match op {
        NirBinaryOp::Add => Some(lhs + rhs),
        NirBinaryOp::Sub => Some(lhs - rhs),
        NirBinaryOp::Mul => Some(lhs * rhs),
        NirBinaryOp::Div => (rhs != 0).then_some(lhs / rhs),
        NirBinaryOp::Eq => Some((lhs == rhs) as i64),
        NirBinaryOp::Lt => Some((lhs < rhs) as i64),
        NirBinaryOp::Gt => Some((lhs > rhs) as i64),
    }
}

#[cfg(test)]
mod tests {
    use super::simplify_nir_module;
    use nuis_semantics::model::{NirBinaryOp, NirExpr, NirFunction, NirModule, NirStmt};

    fn sample_module(body: Vec<NirStmt>) -> NirModule {
        NirModule {
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            structs: vec![],
            functions: vec![NirFunction {
                name: "main".to_owned(),
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                is_async: false,
                params: vec![],
                return_type: None,
                body,
            }],
        }
    }

    #[test]
    fn folds_integer_binary_constants() {
        let mut module = sample_module(vec![NirStmt::Return(Some(NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Int(2)),
            rhs: Box::new(NirExpr::Int(3)),
        }))]);
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Return(Some(NirExpr::Int(5)))]
        );
    }

    #[test]
    fn folds_integer_comparison_constants() {
        let mut module = sample_module(vec![NirStmt::Return(Some(NirExpr::Binary {
            op: NirBinaryOp::Lt,
            lhs: Box::new(NirExpr::Int(2)),
            rhs: Box::new(NirExpr::Int(5)),
        }))]);
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Return(Some(NirExpr::Int(1)))]
        );
    }

    #[test]
    fn normalizes_if_true_into_then_branch() {
        let mut module = sample_module(vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Return(Some(NirExpr::Int(1)))],
            else_body: vec![NirStmt::Return(Some(NirExpr::Int(0)))],
        }]);
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Return(Some(NirExpr::Int(1)))]
        );
    }

    #[test]
    fn folds_is_null_of_null() {
        let mut module = sample_module(vec![NirStmt::Return(Some(NirExpr::IsNull(Box::new(
            NirExpr::Null,
        ))))]);
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Return(Some(NirExpr::Bool(true)))]
        );
    }

    #[test]
    fn propagates_literal_bindings_into_later_expressions() {
        let mut module = sample_module(vec![
            NirStmt::Let {
                name: "base".to_owned(),
                ty: None,
                value: NirExpr::Int(2),
            },
            NirStmt::Return(Some(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("base".to_owned())),
                rhs: Box::new(NirExpr::Int(3)),
            })),
        ]);
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Return(Some(NirExpr::Int(5)))]
        );
    }

    #[test]
    fn prunes_dead_scalar_binding_after_constant_propagation() {
        let mut module = sample_module(vec![
            NirStmt::Let {
                name: "base".to_owned(),
                ty: None,
                value: NirExpr::Int(2),
            },
            NirStmt::Print(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("base".to_owned())),
                rhs: Box::new(NirExpr::Int(3)),
            }),
        ]);
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Print(NirExpr::Int(5))]
        );
    }

    #[test]
    fn keeps_dead_binding_with_side_effectful_value() {
        let mut module = sample_module(vec![NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::CpuExternCall {
                abi: "c".to_owned(),
                interface: None,
                callee: "host_side_effect".to_owned(),
                args: vec![],
            },
        }]);
        let changed = simplify_nir_module(&mut module);
        assert!(!changed);
        assert_eq!(module.functions[0].body.len(), 1);
    }
}
