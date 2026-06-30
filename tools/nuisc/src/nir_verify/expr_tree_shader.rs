use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::super::data::NirDataKind;
use super::super::super::task_result_facts::{BorrowBindings, TaskResultStateFact};
use super::verify_expr;

pub(super) fn verify_shader_expr_tree(
    expr: &NirExpr,
    moved: &BTreeSet<String>,
    borrows: &BTreeMap<String, usize>,
    borrow_bindings: &BorrowBindings,
    data_bindings: &BTreeMap<String, NirDataKind>,
    task_result_facts: &BTreeMap<String, TaskResultStateFact>,
) -> Result<bool, String> {
    match expr {
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            verify_expr(
                target,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                pipeline,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                viewport,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileRender { packet, .. } => {
            verify_expr(
                packet,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. } => {
            verify_expr(
                base,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                delta,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            verify_expr(
                delta,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                scale,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                base,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            verify_expr(
                base,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                delta,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderSample {
            texture,
            sampler,
            x,
            y,
            ..
        } => {
            verify_expr(
                texture,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                sampler,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                x,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                y,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderSampleUv {
            texture,
            sampler,
            uv,
            ..
        } => {
            verify_expr(
                texture,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                sampler,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                uv,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderBinding { value, .. } => {
            verify_expr(
                value,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
        }
        NirExpr::ShaderBindSet { pipeline, bindings } => {
            verify_expr(
                pipeline,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            for binding in bindings {
                verify_expr(
                    binding,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
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
            verify_expr(
                color,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                speed,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                radius,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let Some(accent) = accent {
                verify_expr(
                    accent,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
            if let Some(toggle_state) = toggle_state {
                verify_expr(
                    toggle_state,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
            if let Some(focus_index) = focus_index {
                verify_expr(
                    focus_index,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        NirExpr::ShaderDrawInstanced { pass, packet, .. } => {
            verify_expr(
                pass,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            verify_expr(
                packet,
                moved,
                borrows,
                borrow_bindings,
                data_bindings,
                task_result_facts,
            )?;
            if let NirExpr::ShaderDrawInstanced {
                vertex_count,
                instance_count,
                ..
            } = expr
            {
                verify_expr(
                    vertex_count,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
                verify_expr(
                    instance_count,
                    moved,
                    borrows,
                    borrow_bindings,
                    data_bindings,
                    task_result_facts,
                )?;
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
