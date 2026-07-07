use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::optimize_expr_data::simplify_data_expr;
use super::optimize_expr_helpers::{is_inline_safe_arg, simplify_expr_vec};
use super::{
    fold_int_binary, simplify_optional_box_expr, substitute_inline_params, InlineTemplate,
};

pub(super) fn simplify_expr(
    expr: NirExpr,
    env: &BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> (NirExpr, bool) {
    if let Some(result) = simplify_data_expr(expr.clone(), env, inline_templates, active_inline) {
        return result;
    }

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
        NirExpr::HostBufferHandle(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::HostBufferHandle(Box::new(inner)), changed)
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
        NirExpr::CpuThreadJoin(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuThreadJoin(Box::new(inner)), changed)
        }
        NirExpr::CpuThreadJoinResult(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuThreadJoinResult(Box::new(inner)), changed)
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
        NirExpr::CpuMutexNew(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuMutexNew(Box::new(inner)), changed)
        }
        NirExpr::CpuMutexLock(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuMutexLock(Box::new(inner)), changed)
        }
        NirExpr::CpuMutexUnlock(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuMutexUnlock(Box::new(inner)), changed)
        }
        NirExpr::CpuMutexValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            (NirExpr::CpuMutexValue(Box::new(inner)), changed)
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
                    let (inlined, _) =
                        simplify_expr(substituted, env, inline_templates, active_inline);
                    active_inline.remove(&callee);
                    return (inlined, true);
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
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
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
            (
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                },
                changed,
            )
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
