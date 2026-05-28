use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    nir_expr_effect_class, NirAnnotation, NirBinaryOp, NirExpr, NirExprEffectClass, NirFunction,
    NirModule, NirStmt,
};

pub fn simplify_nir_module(module: &mut NirModule) -> bool {
    let inline_templates = collect_inline_templates(module);
    let mut changed = false;
    for function in &mut module.functions {
        changed |= simplify_nir_function(function, &inline_templates);
    }
    changed
}

fn simplify_nir_function(
    function: &mut NirFunction,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> bool {
    let mut env = BTreeMap::new();
    simplify_stmt_block(&mut function.body, &mut env, inline_templates)
}

fn simplify_stmt_block(
    stmts: &mut Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> bool {
    let mut changed = false;
    let original = std::mem::take(stmts);
    let mut rewritten = Vec::with_capacity(original.len());
    for stmt in original {
        changed |= rewrite_stmt(stmt, &mut rewritten, env, inline_templates);
    }
    changed |= prune_dead_scalar_bindings(&mut rewritten);
    *stmts = rewritten;
    changed
}

fn rewrite_stmt_block_without_prune(
    stmts: Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> (Vec<NirStmt>, bool) {
    let mut changed = false;
    let mut rewritten = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        changed |= rewrite_stmt(stmt, &mut rewritten, env, inline_templates);
    }
    (rewritten, changed)
}

fn rewrite_stmt(
    stmt: NirStmt,
    out: &mut Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> bool {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            refresh_literal_binding(env, &name, &value);
            out.push(NirStmt::Let { name, ty, value });
            changed
        }
        NirStmt::Const { name, ty, value } => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            refresh_literal_binding(env, &name, &value);
            out.push(NirStmt::Const { name, ty, value });
            changed
        }
        NirStmt::Print(value) => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            out.push(NirStmt::Print(value));
            changed
        }
        NirStmt::Await(value) => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            out.push(NirStmt::Await(value));
            changed
        }
        NirStmt::Expr(value) => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            out.push(NirStmt::Expr(value));
            changed
        }
        NirStmt::Return(value) => {
            let (value, changed) = match value {
                Some(value) => {
                    let (value, changed) =
                        simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
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
            let (condition, mut changed) =
                simplify_expr(condition, env, inline_templates, &mut BTreeSet::new());
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            if env.is_empty() {
                let original_then = std::mem::take(&mut then_body);
                let original_else = std::mem::take(&mut else_body);
                let (rewritten_then, then_changed) = rewrite_stmt_block_without_prune(
                    original_then,
                    &mut then_env,
                    inline_templates,
                );
                let (rewritten_else, else_changed) = rewrite_stmt_block_without_prune(
                    original_else,
                    &mut else_env,
                    inline_templates,
                );
                then_body = rewritten_then;
                else_body = rewritten_else;
                changed |= then_changed || else_changed;
            } else {
                changed |= simplify_stmt_block(&mut then_body, &mut then_env, inline_templates);
                changed |= simplify_stmt_block(&mut else_body, &mut else_env, inline_templates);
            }
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
        NirStmt::While {
            condition,
            mut body,
        } => {
            let loop_input_env = BTreeMap::new();
            let (condition, mut changed) = simplify_expr(
                condition,
                &loop_input_env,
                inline_templates,
                &mut BTreeSet::new(),
            );
            let mut loop_env = BTreeMap::new();
            let original_body = std::mem::take(&mut body);
            let mut rewritten_body = Vec::with_capacity(original_body.len());
            for stmt in original_body {
                changed |= rewrite_stmt(stmt, &mut rewritten_body, &mut loop_env, inline_templates);
            }
            body = rewritten_body;
            out.push(NirStmt::While { condition, body });
            env.clear();
            changed
        }
        NirStmt::Break => {
            out.push(NirStmt::Break);
            false
        }
        NirStmt::Continue => {
            out.push(NirStmt::Continue);
            false
        }
    }
}

#[derive(Clone)]
struct InlineTemplate {
    params: Vec<String>,
    value: NirExpr,
}

fn collect_inline_templates(module: &NirModule) -> BTreeMap<String, InlineTemplate> {
    module
        .functions
        .iter()
        .filter(|function| has_annotation(function, "inline"))
        .filter(|function| !has_annotation(function, "noinline"))
        .filter_map(|function| match function.body.as_slice() {
            [NirStmt::Return(Some(value))]
                if is_inline_safe_expr(
                    value,
                    &function
                        .params
                        .iter()
                        .map(|param| param.name.as_str())
                        .collect::<BTreeSet<_>>(),
                ) =>
            {
                Some((
                    function.name.clone(),
                    InlineTemplate {
                        params: function
                            .params
                            .iter()
                            .map(|param| param.name.clone())
                            .collect(),
                        value: value.clone(),
                    },
                ))
            }
            _ => None,
        })
        .collect()
}

fn has_annotation(function: &NirFunction, name: &str) -> bool {
    function
        .annotations
        .iter()
        .any(|annotation: &NirAnnotation| annotation.name == name)
}

fn is_inline_safe_expr(expr: &NirExpr, params: &BTreeSet<&str>) -> bool {
    match expr {
        NirExpr::Var(name) => params.contains(name.as_str()),
        NirExpr::Bool(_) | NirExpr::Text(_) | NirExpr::Int(_) | NirExpr::Null => true,
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| is_inline_safe_expr(value, params)),
        NirExpr::FieldAccess { base, .. } => is_inline_safe_expr(base, params),
        NirExpr::Binary { lhs, rhs, .. } => {
            is_inline_safe_expr(lhs, params) && is_inline_safe_expr(rhs, params)
        }
        _ => false,
    }
}

fn simplify_expr(
    expr: NirExpr,
    env: &BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> (NirExpr, bool) {
    match expr {
        NirExpr::Var(name) => match env.get(&name) {
            Some(value) => (value.clone(), true),
            None => (NirExpr::Var(name), false),
        },
        NirExpr::Await(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::Await(Box::new(inner)), changed)
        }
        NirExpr::Borrow(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::Borrow(Box::new(inner)), changed)
        }
        NirExpr::BorrowEnd(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::BorrowEnd(Box::new(inner)), changed)
        }
        NirExpr::Move(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::Move(Box::new(inner)), changed)
        }
        NirExpr::AllocNode { value, next } => {
            let (value, left) = simplify_expr(*value, env, inline_templates, active_inline);
            let (next, right) = simplify_expr(*next, env, inline_templates, active_inline);
            (
                NirExpr::AllocNode {
                    value: Box::new(value),
                    next: Box::new(next),
                },
                left || right,
            )
        }
        NirExpr::AllocBuffer { len, fill } => {
            let (len, left) = simplify_expr(*len, env, inline_templates, active_inline);
            let (fill, right) = simplify_expr(*fill, env, inline_templates, active_inline);
            (
                NirExpr::AllocBuffer {
                    len: Box::new(len),
                    fill: Box::new(fill),
                },
                left || right,
            )
        }
        NirExpr::LoadValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::LoadValue(Box::new(inner)), changed)
        }
        NirExpr::LoadNext(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::LoadNext(Box::new(inner)), changed)
        }
        NirExpr::BufferLen(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::BufferLen(Box::new(inner)), changed)
        }
        NirExpr::LoadAt { buffer, index } => {
            let (buffer, left) = simplify_expr(*buffer, env, inline_templates, active_inline);
            let (index, right) = simplify_expr(*index, env, inline_templates, active_inline);
            (
                NirExpr::LoadAt {
                    buffer: Box::new(buffer),
                    index: Box::new(index),
                },
                left || right,
            )
        }
        NirExpr::StoreValue { target, value } => {
            let (target, left) = simplify_expr(*target, env, inline_templates, active_inline);
            let (value, right) = simplify_expr(*value, env, inline_templates, active_inline);
            (
                NirExpr::StoreValue {
                    target: Box::new(target),
                    value: Box::new(value),
                },
                left || right,
            )
        }
        NirExpr::StoreNext { target, next } => {
            let (target, left) = simplify_expr(*target, env, inline_templates, active_inline);
            let (next, right) = simplify_expr(*next, env, inline_templates, active_inline);
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
            let (buffer, a) = simplify_expr(*buffer, env, inline_templates, active_inline);
            let (index, b) = simplify_expr(*index, env, inline_templates, active_inline);
            let (value, c) = simplify_expr(*value, env, inline_templates, active_inline);
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
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataOutputPipe(Box::new(inner)), changed)
        }
        NirExpr::DataInputPipe(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataInputPipe(Box::new(inner)), changed)
        }
        NirExpr::DataResult { value, state } => {
            let (value, changed) = simplify_expr(*value, env, inline_templates, active_inline);
            (
                NirExpr::DataResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            )
        }
        NirExpr::DataReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataReady(Box::new(inner)), changed)
        }
        NirExpr::DataMoved(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataMoved(Box::new(inner)), changed)
        }
        NirExpr::DataWindowed(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataWindowed(Box::new(inner)), changed)
        }
        NirExpr::DataValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataValue(Box::new(inner)), changed)
        }
        NirExpr::DataCopyWindow { input, offset, len } => {
            let (input, a) = simplify_expr(*input, env, inline_templates, active_inline);
            let (offset, b) = simplify_expr(*offset, env, inline_templates, active_inline);
            let (len, c) = simplify_expr(*len, env, inline_templates, active_inline);
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
            let (window, left) = simplify_expr(*window, env, inline_templates, active_inline);
            let (index, right) = simplify_expr(*index, env, inline_templates, active_inline);
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
            let (window, a) = simplify_expr(*window, env, inline_templates, active_inline);
            let (index, b) = simplify_expr(*index, env, inline_templates, active_inline);
            let (value, c) = simplify_expr(*value, env, inline_templates, active_inline);
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
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::DataFreezeWindow(Box::new(inner)), changed)
        }
        NirExpr::DataImmutableWindow { input, offset, len } => {
            let (input, a) = simplify_expr(*input, env, inline_templates, active_inline);
            let (offset, b) = simplify_expr(*offset, env, inline_templates, active_inline);
            let (len, c) = simplify_expr(*len, env, inline_templates, active_inline);
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
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuJoin(Box::new(inner)), changed)
        }
        NirExpr::CpuCancel(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuCancel(Box::new(inner)), changed)
        }
        NirExpr::CpuJoinResult(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuJoinResult(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskCompleted(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuTaskCompleted(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskTimedOut(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuTaskTimedOut(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskCancelled(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuTaskCancelled(Box::new(inner)), changed)
        }
        NirExpr::CpuTaskValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuTaskValue(Box::new(inner)), changed)
        }
        NirExpr::CpuTimeout { task, limit } => {
            let (task, left) = simplify_expr(*task, env, inline_templates, active_inline);
            let (limit, right) = simplify_expr(*limit, env, inline_templates, active_inline);
            (
                NirExpr::CpuTimeout {
                    task: Box::new(task),
                    limit: Box::new(limit),
                },
                left || right,
            )
        }
        NirExpr::CpuPresentFrame(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuPresentFrame(Box::new(inner)), changed)
        }
        NirExpr::ShaderProfileColorSeed { unit, base, delta } => {
            let (base, left) = simplify_expr(*base, env, inline_templates, active_inline);
            let (delta, right) = simplify_expr(*delta, env, inline_templates, active_inline);
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
            let (delta, a) = simplify_expr(*delta, env, inline_templates, active_inline);
            let (scale, b) = simplify_expr(*scale, env, inline_templates, active_inline);
            let (base, c) = simplify_expr(*base, env, inline_templates, active_inline);
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
            let (base, left) = simplify_expr(*base, env, inline_templates, active_inline);
            let (delta, right) = simplify_expr(*delta, env, inline_templates, active_inline);
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
            let (color, a) = simplify_expr(*color, env, inline_templates, active_inline);
            let (speed, b) = simplify_expr(*speed, env, inline_templates, active_inline);
            let (radius, c) = simplify_expr(*radius, env, inline_templates, active_inline);
            let (accent, d) =
                simplify_optional_box_expr(accent, env, inline_templates, active_inline);
            let (toggle_state, e) =
                simplify_optional_box_expr(toggle_state, env, inline_templates, active_inline);
            let (focus_index, f) =
                simplify_optional_box_expr(focus_index, env, inline_templates, active_inline);
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
            let (value, changed) = simplify_expr(*value, env, inline_templates, active_inline);
            (
                NirExpr::KernelResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            )
        }
        NirExpr::KernelConfigReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::KernelConfigReady(Box::new(inner)), changed)
        }
        NirExpr::KernelValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::KernelValue(Box::new(inner)), changed)
        }
        NirExpr::DataProfileSendUplink { unit, input } => {
            let (input, changed) = simplify_expr(*input, env, inline_templates, active_inline);
            (
                NirExpr::DataProfileSendUplink {
                    unit,
                    input: Box::new(input),
                },
                changed,
            )
        }
        NirExpr::DataProfileSendDownlink { unit, input } => {
            let (input, changed) = simplify_expr(*input, env, inline_templates, active_inline);
            (
                NirExpr::DataProfileSendDownlink {
                    unit,
                    input: Box::new(input),
                },
                changed,
            )
        }
        NirExpr::ShaderResult { value, state } => {
            let (value, changed) = simplify_expr(*value, env, inline_templates, active_inline);
            (
                NirExpr::ShaderResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            )
        }
        NirExpr::ShaderPassReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::ShaderPassReady(Box::new(inner)), changed)
        }
        NirExpr::ShaderFrameReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::ShaderFrameReady(Box::new(inner)), changed)
        }
        NirExpr::ShaderValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::ShaderValue(Box::new(inner)), changed)
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            let (target, a) = simplify_expr(*target, env, inline_templates, active_inline);
            let (pipeline, b) = simplify_expr(*pipeline, env, inline_templates, active_inline);
            let (viewport, c) = simplify_expr(*viewport, env, inline_templates, active_inline);
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
            let (pass, a) = simplify_expr(*pass, env, inline_templates, active_inline);
            let (packet, b) = simplify_expr(*packet, env, inline_templates, active_inline);
            let (vertex_count, c) =
                simplify_expr(*vertex_count, env, inline_templates, active_inline);
            let (instance_count, d) =
                simplify_expr(*instance_count, env, inline_templates, active_inline);
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
            let (packet, changed) = simplify_expr(*packet, env, inline_templates, active_inline);
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
            let (args, changed) = simplify_expr_vec(args, env, inline_templates, active_inline);
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
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::Free(Box::new(inner)), changed)
        }
        NirExpr::IsNull(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            match inner {
                NirExpr::Null => (NirExpr::Bool(true), true),
                other => (NirExpr::IsNull(Box::new(other)), changed),
            }
        }
        NirExpr::Call { callee, args } => {
            let (args, changed) = simplify_expr_vec(args, env, inline_templates, active_inline);
            if let Some(template) = inline_templates.get(&callee) {
                if !active_inline.contains(&callee)
                    && template.params.len() == args.len()
                    && args.iter().all(is_inline_safe_arg)
                {
                    let mut substitutions = BTreeMap::new();
                    for (param, arg) in template.params.iter().zip(args.iter()) {
                        substitutions.insert(param.clone(), arg.clone());
                    }
                    active_inline.insert(callee.clone());
                    let substituted = substitute_inline_params(&template.value, &substitutions);
                    let (inlined, inner_changed) =
                        simplify_expr(substituted, env, inline_templates, active_inline);
                    active_inline.remove(&callee);
                    return (inlined, true || changed || inner_changed);
                }
            }
            (NirExpr::Call { callee, args }, changed)
        }
        NirExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let (receiver, left) = simplify_expr(*receiver, env, inline_templates, active_inline);
            let (args, right) = simplify_expr_vec(args, env, inline_templates, active_inline);
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
                    let (value, value_changed) =
                        simplify_expr(value, env, inline_templates, active_inline);
                    changed |= value_changed;
                    (name, value)
                })
                .collect();
            (NirExpr::StructLiteral { type_name, fields }, changed)
        }
        NirExpr::FieldAccess { base, field } => {
            let (base, changed) = simplify_expr(*base, env, inline_templates, active_inline);
            (
                NirExpr::FieldAccess {
                    base: Box::new(base),
                    field,
                },
                changed,
            )
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let (lhs, left) = simplify_expr(*lhs, env, inline_templates, active_inline);
            let (rhs, right) = simplify_expr(*rhs, env, inline_templates, active_inline);
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
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> (Vec<NirExpr>, bool) {
    let mut changed = false;
    let values = values
        .into_iter()
        .map(|value| {
            let (value, value_changed) = simplify_expr(value, env, inline_templates, active_inline);
            changed |= value_changed;
            value
        })
        .collect();
    (values, changed)
}

fn is_inline_safe_arg(expr: &NirExpr) -> bool {
    matches!(
        nir_expr_effect_class(expr),
        NirExprEffectClass::Pure
            | NirExprEffectClass::LocalReadOnly
            | NirExprEffectClass::HostReadOnly
            | NirExprEffectClass::DomainReadOnly
    )
}

fn substitute_inline_params(expr: &NirExpr, substitutions: &BTreeMap<String, NirExpr>) -> NirExpr {
    match expr {
        NirExpr::Var(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| NirExpr::Var(name.clone())),
        NirExpr::StructLiteral { type_name, fields } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), substitute_inline_params(value, substitutions)))
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(substitute_inline_params(base, substitutions)),
            field: field.clone(),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(substitute_inline_params(lhs, substitutions)),
            rhs: Box::new(substitute_inline_params(rhs, substitutions)),
        },
        other => other.clone(),
    }
}

fn simplify_optional_box_expr(
    value: Option<Box<NirExpr>>,
    env: &BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> (Option<Box<NirExpr>>, bool) {
    match value {
        Some(value) => {
            let (value, changed) = simplify_expr(*value, env, inline_templates, active_inline);
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
        NirStmt::While { condition, body } => {
            let mut live_before = live_after.clone();
            live_before.extend(live_before_block(&body, live_after));
            collect_used_vars_expr(&condition, &mut live_before);
            (Some(NirStmt::While { condition, body }), live_before, false)
        }
        NirStmt::Break => (Some(NirStmt::Break), live_after.clone(), false),
        NirStmt::Continue => (Some(NirStmt::Continue), live_after.clone(), false),
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
        NirStmt::While { condition, body } => {
            let mut live_before = live_after.clone();
            live_before.extend(live_before_block(body, live_after));
            collect_used_vars_expr(condition, &mut live_before);
            live_before
        }
        NirStmt::Break | NirStmt::Continue => live_after.clone(),
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
        | NirExpr::NetworkAcceptReady(inner)
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
    use nuis_semantics::model::{
        NirAnnotation, NirBinaryOp, NirExpr, NirFunction, NirModule, NirParam, NirStmt, NirTypeRef,
        NirVisibility,
    };

    fn i64_type() -> NirTypeRef {
        NirTypeRef {
            name: "i64".to_owned(),
            generic_args: vec![],
            is_optional: false,
            is_ref: false,
        }
    }

    fn sample_module(body: Vec<NirStmt>) -> NirModule {
        NirModule {
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![],
            type_aliases: vec![],
            structs: vec![],
            traits: vec![],
            impls: vec![],
            functions: vec![NirFunction {
                name: "main".to_owned(),
                annotations: vec![],
                visibility: NirVisibility::Private,
                test_name: None,
                test_ignored: false,
                test_should_fail: false,
                test_reason: None,
                test_timeout_ms: None,
                test_clock_domain: None,
                test_clock_policy: None,
                is_async: false,
                generic_params: vec![],
                params: vec![],
                return_type: None,
                body,
            }],
        }
    }

    fn annotation(name: &str) -> NirAnnotation {
        NirAnnotation {
            name: name.to_owned(),
            args: vec![],
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
    fn does_not_propagate_outer_literal_into_while_condition_or_body() {
        let mut module = sample_module(vec![
            NirStmt::Let {
                name: "value".to_owned(),
                ty: None,
                value: NirExpr::Int(0),
            },
            NirStmt::While {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Lt,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(4)),
                },
                body: vec![NirStmt::Let {
                    name: "value".to_owned(),
                    ty: None,
                    value: NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(1)),
                    },
                }],
            },
            NirStmt::Print(NirExpr::Var("value".to_owned())),
            NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
        ]);
        let _changed = simplify_nir_module(&mut module);
        let NirStmt::While { condition, body } = &module.functions[0].body[1] else {
            panic!("expected while statement to remain in place");
        };
        assert_eq!(
            condition,
            &NirExpr::Binary {
                op: NirBinaryOp::Lt,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(4)),
            }
        );
        let NirStmt::Let { value, .. } = &body[0] else {
            panic!("expected loop body binding");
        };
        assert_eq!(
            value,
            &NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            }
        );
        assert_eq!(
            module.functions[0].body[2],
            NirStmt::Print(NirExpr::Var("value".to_owned()))
        );
        assert_eq!(
            module.functions[0].body[3],
            NirStmt::Return(Some(NirExpr::Var("value".to_owned())))
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

    #[test]
    fn preserves_branch_local_carry_updates_inside_while() {
        let mut module = sample_module(vec![
            NirStmt::Let {
                name: "value".to_owned(),
                ty: None,
                value: NirExpr::Int(0),
            },
            NirStmt::Let {
                name: "acc".to_owned(),
                ty: None,
                value: NirExpr::Int(0),
            },
            NirStmt::While {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Lt,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(5)),
                },
                body: vec![
                    NirStmt::Let {
                        name: "value".to_owned(),
                        ty: None,
                        value: NirExpr::Binary {
                            op: NirBinaryOp::Add,
                            lhs: Box::new(NirExpr::Var("value".to_owned())),
                            rhs: Box::new(NirExpr::Int(1)),
                        },
                    },
                    NirStmt::If {
                        condition: NirExpr::Binary {
                            op: NirBinaryOp::Gt,
                            lhs: Box::new(NirExpr::Var("value".to_owned())),
                            rhs: Box::new(NirExpr::Int(2)),
                        },
                        then_body: vec![NirStmt::Let {
                            name: "acc".to_owned(),
                            ty: None,
                            value: NirExpr::Binary {
                                op: NirBinaryOp::Add,
                                lhs: Box::new(NirExpr::Var("acc".to_owned())),
                                rhs: Box::new(NirExpr::Var("value".to_owned())),
                            },
                        }],
                        else_body: vec![NirStmt::Let {
                            name: "acc".to_owned(),
                            ty: None,
                            value: NirExpr::Var("acc".to_owned()),
                        }],
                    },
                ],
            },
            NirStmt::Return(Some(NirExpr::Var("acc".to_owned()))),
        ]);
        let _changed = simplify_nir_module(&mut module);
        let NirStmt::While { body, .. } = &module.functions[0].body[2] else {
            panic!("expected while statement to remain in place");
        };
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &body[1]
        else {
            panic!("expected inner if statement to remain in loop body");
        };
        assert!(matches!(then_body.first(), Some(NirStmt::Let { name, .. }) if name == "acc"));
        assert!(matches!(else_body.first(), Some(NirStmt::Let { name, .. }) if name == "acc"));
    }

    #[test]
    fn inlines_annotated_pure_function_calls() {
        let mut module = NirModule {
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![],
            type_aliases: vec![],
            structs: vec![],
            traits: vec![],
            impls: vec![],
            functions: vec![
                NirFunction {
                    name: "add_one".to_owned(),
                    annotations: vec![annotation("inline")],
                    visibility: NirVisibility::Private,
                    test_name: None,
                    test_ignored: false,
                    test_should_fail: false,
                    test_reason: None,
                    test_timeout_ms: None,
                    test_clock_domain: None,
                    test_clock_policy: None,
                    is_async: false,
                    generic_params: vec![],
                    params: vec![NirParam {
                        name: "value".to_owned(),
                        ty: i64_type(),
                    }],
                    return_type: Some(i64_type()),
                    body: vec![NirStmt::Return(Some(NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(1)),
                    }))],
                },
                NirFunction {
                    name: "main".to_owned(),
                    annotations: vec![],
                    visibility: NirVisibility::Private,
                    test_name: None,
                    test_ignored: false,
                    test_should_fail: false,
                    test_reason: None,
                    test_timeout_ms: None,
                    test_clock_domain: None,
                    test_clock_policy: None,
                    is_async: false,
                    generic_params: vec![],
                    params: vec![],
                    return_type: Some(i64_type()),
                    body: vec![NirStmt::Return(Some(NirExpr::Call {
                        callee: "add_one".to_owned(),
                        args: vec![NirExpr::Int(41)],
                    }))],
                },
            ],
        };
        let changed = simplify_nir_module(&mut module);
        assert!(changed);
        assert_eq!(
            module.functions[1].body,
            vec![NirStmt::Return(Some(NirExpr::Int(42)))]
        );
    }

    #[test]
    fn does_not_inline_noinline_annotated_function_calls() {
        let mut module = NirModule {
            uses: vec![],
            domain: "cpu".to_owned(),
            unit: "Main".to_owned(),
            externs: vec![],
            extern_interfaces: vec![],
            consts: vec![],
            type_aliases: vec![],
            structs: vec![],
            traits: vec![],
            impls: vec![],
            functions: vec![
                NirFunction {
                    name: "add_one".to_owned(),
                    annotations: vec![annotation("inline"), annotation("noinline")],
                    visibility: NirVisibility::Private,
                    test_name: None,
                    test_ignored: false,
                    test_should_fail: false,
                    test_reason: None,
                    test_timeout_ms: None,
                    test_clock_domain: None,
                    test_clock_policy: None,
                    is_async: false,
                    generic_params: vec![],
                    params: vec![NirParam {
                        name: "value".to_owned(),
                        ty: i64_type(),
                    }],
                    return_type: Some(i64_type()),
                    body: vec![NirStmt::Return(Some(NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(1)),
                    }))],
                },
                NirFunction {
                    name: "main".to_owned(),
                    annotations: vec![],
                    visibility: NirVisibility::Private,
                    test_name: None,
                    test_ignored: false,
                    test_should_fail: false,
                    test_reason: None,
                    test_timeout_ms: None,
                    test_clock_domain: None,
                    test_clock_policy: None,
                    is_async: false,
                    generic_params: vec![],
                    params: vec![],
                    return_type: Some(i64_type()),
                    body: vec![NirStmt::Return(Some(NirExpr::Call {
                        callee: "add_one".to_owned(),
                        args: vec![NirExpr::Int(41)],
                    }))],
                },
            ],
        };
        let changed = simplify_nir_module(&mut module);
        assert!(!changed);
        assert_eq!(
            module.functions[1].body,
            vec![NirStmt::Return(Some(NirExpr::Call {
                callee: "add_one".to_owned(),
                args: vec![NirExpr::Int(41)],
            }))]
        );
    }
}
