use std::collections::BTreeMap;

use nuis_semantics::model::NirExpr;

use super::super::data::{infer_data_kind, render_data_expr_name, NirDataKind};

pub(super) fn verify_data_expr_shape(
    expr: &NirExpr,
    data_bindings: &BTreeMap<String, NirDataKind>,
) -> Result<(), String> {
    match expr {
        NirExpr::DataOutputPipe(inner) => {
            let source = infer_data_kind(inner, data_bindings);
            if matches!(source, NirDataKind::PipeOutput | NirDataKind::PipeInput) {
                return Err(format!(
                    "nir verify: data_output_pipe cannot wrap nested pipe `{}`",
                    render_data_expr_name(inner)
                ));
            }
        }
        NirExpr::DataInputPipe(inner) => {
            if infer_data_kind(inner, data_bindings) != NirDataKind::PipeOutput {
                return Err(format!(
                    "nir verify: data_input_pipe expects output pipe input, got `{}`",
                    render_data_expr_name(inner)
                ));
            }
        }
        NirExpr::DataCopyWindow { input, .. } | NirExpr::DataImmutableWindow { input, .. } => {
            let source = infer_data_kind(input, data_bindings);
            if matches!(
                source,
                NirDataKind::WindowMutable
                    | NirDataKind::WindowImmutable
                    | NirDataKind::PipeOutput
                    | NirDataKind::PipeInput
                    | NirDataKind::Marker
                    | NirDataKind::HandleTable
            ) {
                return Err(format!(
                    "nir verify: cannot create nested data window from `{}`",
                    render_data_expr_name(input)
                ));
            }
        }
        NirExpr::DataReadWindow { window, index } => {
            let source = infer_data_kind(window, data_bindings);
            if !matches!(
                source,
                NirDataKind::WindowMutable | NirDataKind::WindowImmutable
            ) {
                return Err(format!(
                    "nir verify: data_read_window expects window input, got `{}`",
                    render_data_expr_name(window)
                ));
            }
            let index_kind = infer_data_kind(index, data_bindings);
            if index_kind != NirDataKind::Other {
                return Err(format!(
                    "nir verify: data_read_window expects scalar index, got `{}`",
                    render_data_expr_name(index)
                ));
            }
        }
        NirExpr::DataWriteWindow { window, index, .. } => {
            let source = infer_data_kind(window, data_bindings);
            if source != NirDataKind::WindowMutable {
                return Err(format!(
                    "nir verify: data_write_window expects mutable window input, got `{}`",
                    render_data_expr_name(window)
                ));
            }
            let index_kind = infer_data_kind(index, data_bindings);
            if index_kind != NirDataKind::Other {
                return Err(format!(
                    "nir verify: data_write_window expects scalar index, got `{}`",
                    render_data_expr_name(index)
                ));
            }
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            let source = infer_data_kind(input, data_bindings);
            if matches!(source, NirDataKind::WindowMutable) {
                return Err(format!(
                    "nir verify: data_profile_send requires immutable window payload, got `{}`",
                    render_data_expr_name(input)
                ));
            }
            if matches!(
                source,
                NirDataKind::PipeOutput
                    | NirDataKind::PipeInput
                    | NirDataKind::Marker
                    | NirDataKind::HandleTable
            ) {
                return Err(format!(
                    "nir verify: data_profile_send cannot wrap illegal window payload `{}`",
                    render_data_expr_name(input)
                ));
            }
        }
        NirExpr::DataFreezeWindow(input) => {
            let source = infer_data_kind(input, data_bindings);
            if !matches!(
                source,
                NirDataKind::WindowMutable | NirDataKind::WindowImmutable
            ) {
                return Err(format!(
                    "nir verify: data_freeze_window expects window input, got `{}`",
                    render_data_expr_name(input)
                ));
            }
        }
        _ => {}
    }

    Ok(())
}
