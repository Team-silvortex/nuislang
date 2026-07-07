use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use crate::optimize::{simplify_expr, InlineTemplate};

pub(super) fn simplify_data_expr(
    expr: NirExpr,
    env: &BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> Option<(NirExpr, bool)> {
    match expr {
        NirExpr::DataOutputPipe(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataOutputPipe(Box::new(inner)), changed))
        }
        NirExpr::DataInputPipe(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataInputPipe(Box::new(inner)), changed))
        }
        NirExpr::DataResult { value, state } => {
            let (value, changed) = simplify_expr(*value, env, inline_templates, active_inline);
            Some((
                NirExpr::DataResult {
                    value: Box::new(value),
                    state,
                },
                changed,
            ))
        }
        NirExpr::DataReady(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataReady(Box::new(inner)), changed))
        }
        NirExpr::DataMoved(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataMoved(Box::new(inner)), changed))
        }
        NirExpr::DataWindowed(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataWindowed(Box::new(inner)), changed))
        }
        NirExpr::DataValue(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataValue(Box::new(inner)), changed))
        }
        NirExpr::DataCopyWindow { input, offset, len } => {
            let (input, a) = simplify_expr(*input, env, inline_templates, active_inline);
            let (offset, b) = simplify_expr(*offset, env, inline_templates, active_inline);
            let (len, c) = simplify_expr(*len, env, inline_templates, active_inline);
            Some((
                NirExpr::DataCopyWindow {
                    input: Box::new(input),
                    offset: Box::new(offset),
                    len: Box::new(len),
                },
                a || b || c,
            ))
        }
        NirExpr::DataReadWindow { window, index } => {
            let (window, left) = simplify_expr(*window, env, inline_templates, active_inline);
            let (index, right) = simplify_expr(*index, env, inline_templates, active_inline);
            Some((
                NirExpr::DataReadWindow {
                    window: Box::new(window),
                    index: Box::new(index),
                },
                left || right,
            ))
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            let (window, a) = simplify_expr(*window, env, inline_templates, active_inline);
            let (index, b) = simplify_expr(*index, env, inline_templates, active_inline);
            let (value, c) = simplify_expr(*value, env, inline_templates, active_inline);
            Some((
                NirExpr::DataWriteWindow {
                    window: Box::new(window),
                    index: Box::new(index),
                    value: Box::new(value),
                },
                a || b || c,
            ))
        }
        NirExpr::DataFreezeWindow(inner) => {
            let (inner, changed) = simplify_expr(*inner, env, inline_templates, active_inline);
            Some((NirExpr::DataFreezeWindow(Box::new(inner)), changed))
        }
        NirExpr::DataImmutableWindow { input, offset, len } => {
            let (input, a) = simplify_expr(*input, env, inline_templates, active_inline);
            let (offset, b) = simplify_expr(*offset, env, inline_templates, active_inline);
            let (len, c) = simplify_expr(*len, env, inline_templates, active_inline);
            Some((
                NirExpr::DataImmutableWindow {
                    input: Box::new(input),
                    offset: Box::new(offset),
                    len: Box::new(len),
                },
                a || b || c,
            ))
        }
        NirExpr::DataProfileSendUplink { unit, input } => {
            let (input, changed) = simplify_expr(*input, env, inline_templates, active_inline);
            Some((
                NirExpr::DataProfileSendUplink {
                    unit,
                    input: Box::new(input),
                },
                changed,
            ))
        }
        NirExpr::DataProfileSendDownlink { unit, input } => {
            let (input, changed) = simplify_expr(*input, env, inline_templates, active_inline);
            Some((
                NirExpr::DataProfileSendDownlink {
                    unit,
                    input: Box::new(input),
                },
                changed,
            ))
        }
        _ => None,
    }
}
