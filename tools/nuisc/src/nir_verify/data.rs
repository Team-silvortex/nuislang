use std::collections::BTreeMap;

use nuis_semantics::model::{NirExpr, NirTypeRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NirDataKind {
    Other,
    WindowMutable,
    WindowImmutable,
    Marker,
    HandleTable,
    PipeOutput,
    PipeInput,
}

impl NirDataKind {
    pub(super) fn merge_with_type_hint(self, hint: Option<NirDataKind>) -> NirDataKind {
        if self == NirDataKind::Other {
            hint.unwrap_or(self)
        } else {
            self
        }
    }
}

pub(super) fn infer_data_kind(
    expr: &NirExpr,
    data_bindings: &BTreeMap<String, NirDataKind>,
) -> NirDataKind {
    match expr {
        NirExpr::Await(inner) => infer_data_kind(inner, data_bindings),
        NirExpr::DataResult { value, .. } => infer_data_kind(value, data_bindings),
        NirExpr::DataValue(inner) => infer_data_kind(inner, data_bindings),
        NirExpr::Var(name) => data_bindings
            .get(name)
            .copied()
            .unwrap_or(NirDataKind::Other),
        NirExpr::DataMarker(_) | NirExpr::DataProfileMarkerRef { .. } => NirDataKind::Marker,
        NirExpr::DataHandleTable(_) | NirExpr::DataProfileHandleTableRef { .. } => {
            NirDataKind::HandleTable
        }
        NirExpr::DataOutputPipe(_) => NirDataKind::PipeOutput,
        NirExpr::DataInputPipe(_) => NirDataKind::PipeInput,
        NirExpr::DataCopyWindow { .. } => NirDataKind::WindowMutable,
        NirExpr::DataWriteWindow { .. } => NirDataKind::WindowMutable,
        NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. } => NirDataKind::WindowImmutable,
        _ => NirDataKind::Other,
    }
}

pub(super) fn infer_data_kind_from_type(ty: &NirTypeRef) -> NirDataKind {
    if let Some(mode) = ty.window_mode() {
        return match mode {
            nuis_semantics::model::NirWindowMode::Mutable => NirDataKind::WindowMutable,
            nuis_semantics::model::NirWindowMode::Immutable => NirDataKind::WindowImmutable,
        };
    }
    if ty.name == "Pipe" && ty.generic_args.len() == 1 {
        return NirDataKind::PipeOutput;
    }
    if ty.is_marker_type() {
        return NirDataKind::Marker;
    }
    if ty.is_handle_table_type() {
        return NirDataKind::HandleTable;
    }
    if matches!(
        ty.result_family(),
        Some(nuis_semantics::model::NirResultFamily::Data)
    ) && ty.generic_args.len() == 1
    {
        return infer_data_kind_from_type(&ty.generic_args[0]);
    }
    NirDataKind::Other
}

pub(super) fn render_data_expr_name(expr: &NirExpr) -> String {
    match expr {
        NirExpr::Await(inner) => render_data_expr_name(inner),
        NirExpr::Var(name) => name.clone(),
        NirExpr::DataMarker(tag) => format!("marker:{tag}"),
        NirExpr::DataProfileMarkerRef { unit, tag } => format!("{unit}.marker:{tag}"),
        NirExpr::DataHandleTable(_) => "handle_table".to_owned(),
        NirExpr::DataProfileHandleTableRef { unit } => format!("{unit}.handle_table"),
        NirExpr::DataOutputPipe(_) => "output_pipe".to_owned(),
        NirExpr::DataInputPipe(_) => "input_pipe".to_owned(),
        NirExpr::DataCopyWindow { .. } => "copy_window".to_owned(),
        NirExpr::DataReadWindow { .. } => "read_window".to_owned(),
        NirExpr::DataWriteWindow { .. } => "write_window".to_owned(),
        NirExpr::DataImmutableWindow { .. } => "immutable_window".to_owned(),
        NirExpr::DataFreezeWindow(_) => "freeze_window".to_owned(),
        NirExpr::DataProfileSendUplink { .. } => "profile_send_uplink".to_owned(),
        NirExpr::DataProfileSendDownlink { .. } => "profile_send_downlink".to_owned(),
        _ => "value".to_owned(),
    }
}
