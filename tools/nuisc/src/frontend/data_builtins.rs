use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef, NirWindowMode,
};

use super::{
    compatible_types, expr_type, i64_type, lower_expr, lower_result_observer_call_with_consts,
    lower_result_wrapper_call_with_consts, select_expected_semantic_token_type, validate_type_ref,
    FunctionSignature, ModuleConstValue, ResultObserverCallInput, ResultWrapperCallInput,
};

pub(super) struct DataBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
    pub(super) expected: Option<&'a NirTypeRef>,
}

pub(super) fn lower_data_builtin_call(
    input: DataBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let DataBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    } = input;
    let expr = match callee {
        "data_bind_core" => {
            let [core] = args else {
                return Err("data_bind_core(...) expects 1 arg".to_owned());
            };
            let AstExpr::Int(core_index) = core else {
                return Err("data_bind_core(...) currently expects an integer literal".to_owned());
            };
            NirExpr::DataBindCore(*core_index)
        }
        "data_marker" => {
            let [tag] = args else {
                return Err("data_marker(...) expects 1 arg".to_owned());
            };
            let AstExpr::Text(tag) = tag else {
                return Err("data_marker(...) currently expects a string literal".to_owned());
            };
            let marker_type = select_expected_semantic_token_type(expected, "Marker");
            validate_type_ref(&marker_type)?;
            NirExpr::DataMarker(tag.clone())
        }
        "data_output_pipe" => {
            let [value] = args else {
                return Err("data_output_pipe(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            NirExpr::DataOutputPipe(Box::new(lowered))
        }
        "data_input_pipe" => {
            let [pipe] = args else {
                return Err("data_input_pipe(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                pipe,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            NirExpr::DataInputPipe(Box::new(lowered))
        }
        "data_result" => lower_result_wrapper_call_with_consts(ResultWrapperCallInput {
            name: "data_result",
            args,
            current_domain,
            current_function_is_async: false,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Data,
            build: |value, stage| match stage {
                NirResultStage::Data(state) => Ok(NirExpr::DataResult { value, state }),
                other => Err(format!(
                    "expected data result stage, found `{}`",
                    other.render()
                )),
            },
            expected_shape: "expects a direct data operation like pipe/window/profile send",
        })?,
        "data_ready" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "data_ready",
            args,
            current_domain,
            current_function_is_async: false,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Data,
            build: |expr| NirExpr::DataReady(Box::new(expr)),
        })?,
        "data_moved" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "data_moved",
            args,
            current_domain,
            current_function_is_async: false,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Data,
            build: |expr| NirExpr::DataMoved(Box::new(expr)),
        })?,
        "data_windowed" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "data_windowed",
            args,
            current_domain,
            current_function_is_async: false,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Data,
            build: |expr| NirExpr::DataWindowed(Box::new(expr)),
        })?,
        "data_value" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "data_value",
            args,
            current_domain,
            current_function_is_async: false,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Data,
            build: |expr| NirExpr::DataValue(Box::new(expr)),
        })?,
        "data_copy_window" => {
            let [input, offset, len] = args else {
                return Err("data_copy_window(...) expects 3 args".to_owned());
            };
            NirExpr::DataCopyWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(lower_expr(
                    offset,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                len: Box::new(lower_expr(
                    len,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            }
        }
        "data_read_window" => {
            let [window, index] = args else {
                return Err("data_read_window(...) expects 2 args".to_owned());
            };
            let window_expr = lower_expr(
                window,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let index_expr = lower_expr(
                index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let Some(window_ty) = expr_type(&window_expr, bindings, signatures, struct_table)
            else {
                return Err("data_read_window(...) could not infer window type".to_owned());
            };
            if window_ty.window_mode().is_none() {
                return Err(format!(
                    "data_read_window(...) expects Window<T> or WindowMut<T>, got `{}`",
                    window_ty.render()
                ));
            }
            NirExpr::DataReadWindow {
                window: Box::new(window_expr),
                index: Box::new(index_expr),
            }
        }
        "data_write_window" => {
            let [window, index, value] = args else {
                return Err("data_write_window(...) expects 3 args".to_owned());
            };
            let window_expr = lower_expr(
                window,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let index_expr = lower_expr(
                index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let value_expr = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            let Some(window_ty) = expr_type(&window_expr, bindings, signatures, struct_table)
            else {
                return Err("data_write_window(...) could not infer window type".to_owned());
            };
            if window_ty.window_mode() != Some(NirWindowMode::Mutable) {
                return Err(format!(
                    "data_write_window(...) expects WindowMut<T>, got `{}`",
                    window_ty.render()
                ));
            }
            let payload_ty = window_ty
                .container_payload()
                .cloned()
                .ok_or_else(|| "data_write_window(...) expects window payload type".to_owned())?;
            let Some(value_ty) = expr_type(&value_expr, bindings, signatures, struct_table) else {
                return Err("data_write_window(...) could not infer value type".to_owned());
            };
            if !compatible_types(&payload_ty, &value_ty) {
                return Err(format!(
                    "data_write_window(...) expects payload `{}`, got `{}`",
                    payload_ty.render(),
                    value_ty.render()
                ));
            }
            NirExpr::DataWriteWindow {
                window: Box::new(window_expr),
                index: Box::new(index_expr),
                value: Box::new(value_expr),
            }
        }
        "data_freeze_window" => {
            let [input] = args else {
                return Err("data_freeze_window(...) expects 1 arg".to_owned());
            };
            NirExpr::DataFreezeWindow(Box::new(lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?))
        }
        "data_immutable_window" => {
            let [input, offset, len] = args else {
                return Err("data_immutable_window(...) expects 3 args".to_owned());
            };
            NirExpr::DataImmutableWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(lower_expr(
                    offset,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
                len: Box::new(lower_expr(
                    len,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    Some(&i64_type()),
                )?),
            }
        }
        "data_handle_table" => {
            if args.is_empty() {
                return Err("data_handle_table(...) expects at least 1 slot mapping".to_owned());
            }
            let mut entries = Vec::new();
            for arg in args {
                let AstExpr::Text(text) = arg else {
                    return Err(
                        "data_handle_table(...) currently expects string literals like \"slot=resource\""
                            .to_owned(),
                    );
                };
                let Some((slot, resource)) = text.split_once('=') else {
                    return Err(format!(
                        "data_handle_table(...) entry `{text}` must be `slot=resource`"
                    ));
                };
                entries.push((slot.trim().to_owned(), resource.trim().to_owned()));
            }
            let handle_table_type = select_expected_semantic_token_type(expected, "HandleTable");
            validate_type_ref(&handle_table_type)?;
            NirExpr::DataHandleTable(entries)
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
