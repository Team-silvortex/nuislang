use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef,
};

use super::{
    lower_result_observer_call_with_consts, lower_result_wrapper_call_with_consts,
    FunctionSignature, ModuleConstValue, ResultObserverCallInput, ResultWrapperCallInput,
};

pub(super) struct NetworkBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_network_builtin_call(
    input: NetworkBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let NetworkBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    } = input;
    let expr = match callee {
        "network_profile_bind_core" => {
            let [unit] = args else {
                return Err("network_profile_bind_core(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_bind_core(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_bind_core(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::NetworkProfileBindCoreRef { unit: unit.clone() }
        }
        "network_profile_endpoint_kind" => {
            let [unit] = args else {
                return Err("network_profile_endpoint_kind(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_endpoint_kind(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_endpoint_kind(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileEndpointKindRef { unit: unit.clone() }
        }
        "network_profile_transport_family" => {
            let [unit] = args else {
                return Err("network_profile_transport_family(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_transport_family(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_transport_family(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileTransportFamilyRef { unit: unit.clone() }
        }
        "network_profile_local_port" => {
            let [unit] = args else {
                return Err("network_profile_local_port(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_local_port(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_local_port(...) expects a string literal unit name".to_owned(),
                );
            };
            NirExpr::NetworkProfileLocalPortRef { unit: unit.clone() }
        }
        "network_profile_remote_port" => {
            let [unit] = args else {
                return Err("network_profile_remote_port(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_remote_port(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_remote_port(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileRemotePortRef { unit: unit.clone() }
        }
        "network_profile_connect_timeout" => {
            let [unit] = args else {
                return Err("network_profile_connect_timeout(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_connect_timeout(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_connect_timeout(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileConnectTimeoutRef { unit: unit.clone() }
        }
        "network_profile_read_timeout" => {
            let [unit] = args else {
                return Err("network_profile_read_timeout(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_read_timeout(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_read_timeout(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileReadTimeoutRef { unit: unit.clone() }
        }
        "network_profile_write_timeout" => {
            let [unit] = args else {
                return Err("network_profile_write_timeout(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_write_timeout(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_write_timeout(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileWriteTimeoutRef { unit: unit.clone() }
        }
        "network_profile_timeout_budget" => {
            let [unit] = args else {
                return Err("network_profile_timeout_budget(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_timeout_budget(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_timeout_budget(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileTimeoutBudgetRef { unit: unit.clone() }
        }
        "network_profile_retry_budget" => {
            let [unit] = args else {
                return Err("network_profile_retry_budget(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_retry_budget(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_retry_budget(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileRetryBudgetRef { unit: unit.clone() }
        }
        "network_profile_stream_window" => {
            let [unit] = args else {
                return Err("network_profile_stream_window(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_stream_window(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_stream_window(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileStreamWindowRef { unit: unit.clone() }
        }
        "network_profile_recv_window" => {
            let [unit] = args else {
                return Err("network_profile_recv_window(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_recv_window(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_recv_window(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileRecvWindowRef { unit: unit.clone() }
        }
        "network_profile_send_window" => {
            let [unit] = args else {
                return Err("network_profile_send_window(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_send_window(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_send_window(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileSendWindowRef { unit: unit.clone() }
        }
        "network_profile_protocol_kind" => {
            let [unit] = args else {
                return Err("network_profile_protocol_kind(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_protocol_kind(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_protocol_kind(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileProtocolKindRef { unit: unit.clone() }
        }
        "network_profile_protocol_version" => {
            let [unit] = args else {
                return Err("network_profile_protocol_version(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_protocol_version(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_protocol_version(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileProtocolVersionRef { unit: unit.clone() }
        }
        "network_profile_protocol_header_bytes" => {
            let [unit] = args else {
                return Err("network_profile_protocol_header_bytes(...) expects 1 arg".to_owned());
            };
            if current_domain != "cpu" {
                return Err(
                    "network_profile_protocol_header_bytes(...) is currently only allowed inside `mod cpu <unit>`"
                        .to_owned(),
                );
            }
            let AstExpr::Text(unit) = unit else {
                return Err(
                    "network_profile_protocol_header_bytes(...) expects a string literal unit name"
                        .to_owned(),
                );
            };
            NirExpr::NetworkProfileProtocolHeaderBytesRef { unit: unit.clone() }
        }
        "network_result" => lower_result_wrapper_call_with_consts(ResultWrapperCallInput {
            name: "network_result",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Network,
            build: |value, stage| match stage {
                NirResultStage::Network(state) => Ok(NirExpr::NetworkResult { value, state }),
                other => Err(format!(
                    "expected network result stage, found `{}`",
                    other.render()
                )),
            },
            expected_shape: "expects a direct network profile/config expression",
        })?,
        "network_config_ready" => {
            lower_result_observer_call_with_consts(ResultObserverCallInput {
                name: "network_config_ready",
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                family: NirResultFamily::Network,
                build: |expr| NirExpr::NetworkConfigReady(Box::new(expr)),
            })?
        }
        "network_send_ready" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "network_send_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Network,
            build: |expr| NirExpr::NetworkSendReady(Box::new(expr)),
        })?,
        "network_recv_ready" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "network_recv_ready",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Network,
            build: |expr| NirExpr::NetworkRecvReady(Box::new(expr)),
        })?,
        "network_accept_ready" => {
            lower_result_observer_call_with_consts(ResultObserverCallInput {
                name: "network_accept_ready",
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                family: NirResultFamily::Network,
                build: |expr| NirExpr::NetworkAcceptReady(Box::new(expr)),
            })?
        }
        "network_value" => lower_result_observer_call_with_consts(ResultObserverCallInput {
            name: "network_value",
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            family: NirResultFamily::Network,
            build: |expr| NirExpr::NetworkValue(Box::new(expr)),
        })?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
