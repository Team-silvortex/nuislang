use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{
    lower_expr, select_expected_semantic_token_type, validate_type_ref, FunctionSignature,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_data_profile_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<Option<NirExpr>, String> {
    let expr = match callee {
        "data_profile_bind_core" => {
            let unit = require_cpu_unit_name(callee, args, current_domain)?;
            NirExpr::DataProfileBindCoreRef { unit }
        }
        "data_profile_window_offset" => {
            let unit = require_cpu_unit_name(callee, args, current_domain)?;
            NirExpr::DataProfileWindowOffsetRef { unit }
        }
        "data_profile_uplink_len" => {
            let unit = require_cpu_unit_name(callee, args, current_domain)?;
            NirExpr::DataProfileUplinkLenRef { unit }
        }
        "data_profile_downlink_len" => {
            let unit = require_cpu_unit_name(callee, args, current_domain)?;
            NirExpr::DataProfileDownlinkLenRef { unit }
        }
        "data_profile_uplink_window" => {
            let [unit, input] = args else {
                return Err("data_profile_uplink_window(...) expects 2 args".to_owned());
            };
            let unit =
                require_cpu_unit_text("data_profile_uplink_window(...)", current_domain, unit)?;
            NirExpr::DataImmutableWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(NirExpr::DataProfileWindowOffsetRef { unit: unit.clone() }),
                len: Box::new(NirExpr::DataProfileUplinkLenRef { unit }),
            }
        }
        "data_profile_send_uplink" => {
            let [unit, input] = args else {
                return Err("data_profile_send_uplink(...) expects 2 args".to_owned());
            };
            let unit =
                require_cpu_unit_text("data_profile_send_uplink(...)", current_domain, unit)?;
            let lowered_input = lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            NirExpr::DataProfileSendUplink {
                unit,
                input: Box::new(lowered_input),
            }
        }
        "data_profile_downlink_window" => {
            let [unit, input] = args else {
                return Err("data_profile_downlink_window(...) expects 2 args".to_owned());
            };
            let unit =
                require_cpu_unit_text("data_profile_downlink_window(...)", current_domain, unit)?;
            NirExpr::DataImmutableWindow {
                input: Box::new(lower_expr(
                    input,
                    current_domain,
                    bindings,
                    signatures,
                    struct_table,
                    None,
                )?),
                offset: Box::new(NirExpr::DataProfileWindowOffsetRef { unit: unit.clone() }),
                len: Box::new(NirExpr::DataProfileDownlinkLenRef { unit }),
            }
        }
        "data_profile_send_downlink" => {
            let [unit, input] = args else {
                return Err("data_profile_send_downlink(...) expects 2 args".to_owned());
            };
            let unit =
                require_cpu_unit_text("data_profile_send_downlink(...)", current_domain, unit)?;
            let lowered_input = lower_expr(
                input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            NirExpr::DataProfileSendDownlink {
                unit,
                input: Box::new(lowered_input),
            }
        }
        "data_profile_handle_table" => {
            let unit = require_cpu_unit_name(callee, args, current_domain)?;
            let handle_table_type = select_expected_semantic_token_type(expected, "HandleTable");
            validate_type_ref(&handle_table_type)?;
            NirExpr::DataProfileHandleTableRef { unit }
        }
        "data_profile_marker" => {
            let [unit, tag] = args else {
                return Err("data_profile_marker(...) expects 2 args".to_owned());
            };
            let unit = require_cpu_unit_text("data_profile_marker(...)", current_domain, unit)?;
            let AstExpr::Text(tag) = tag else {
                return Err(
                    "data_profile_marker(...) expects a string literal marker tag".to_owned(),
                );
            };
            let marker_type = select_expected_semantic_token_type(expected, "Marker");
            validate_type_ref(&marker_type)?;
            NirExpr::DataProfileMarkerRef {
                unit,
                tag: tag.clone(),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn require_cpu_unit_name(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
) -> Result<String, String> {
    let [unit] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    require_cpu_unit_text(&format!("{callee}(...)"), current_domain, unit)
}

fn require_cpu_unit_text(
    context: &str,
    current_domain: &str,
    unit: &AstExpr,
) -> Result<String, String> {
    if current_domain != "cpu" {
        return Err(format!(
            "{context} is currently only allowed inside `mod cpu <unit>`"
        ));
    }
    let AstExpr::Text(unit) = unit else {
        return Err(format!("{context} expects a string literal unit name"));
    };
    Ok(unit.clone())
}
