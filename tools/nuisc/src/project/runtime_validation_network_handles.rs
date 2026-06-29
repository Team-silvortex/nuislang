use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use nuis_semantics::model::NirExpr;
use nuis_semantics::model::{NirModule, NirStmt};

use super::super::super::profile_usage::nir_uses_cpu_extern_call;

#[path = "runtime_validation_network_handle_inference.rs"]
mod inference;
use inference::{
    infer_network_function_handle_requirements, infer_network_function_return_kinds,
    infer_network_owned_handle_kind, seed_network_param_bindings,
};

#[path = "runtime_validation_network_handle_expr.rs"]
mod expr;
use expr::validate_network_owned_handle_provenance_in_expr;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleKind {
    Listener,
    StreamTransport,
    DatagramTransport,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleBinding {
    Concrete(NetworkOwnedHandleKind),
    Param {
        index: usize,
        requirement: NetworkOwnedHandleRequirement,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleRequirement {
    OwnedAny,
    Listener,
    Transport,
    StreamTransport,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NetworkOwnedHandleReturn {
    Concrete(NetworkOwnedHandleKind),
    ParamIndex(usize),
}

pub(super) fn validate_network_owned_handle_shape(
    module: &NirModule,
    from: &str,
    to: &str,
) -> Result<(), String> {
    let uses_open_tcp_stream = nir_uses_cpu_extern_call(module, "host_network_open_tcp_stream");
    let uses_open_udp_datagram = nir_uses_cpu_extern_call(module, "host_network_open_udp_datagram");
    let uses_bind_udp_datagram = nir_uses_cpu_extern_call(module, "host_network_bind_udp_datagram");
    let uses_open_tcp_listener = nir_uses_cpu_extern_call(module, "host_network_open_tcp_listener");
    let uses_accept_owned = nir_uses_cpu_extern_call(module, "host_network_accept_owned");
    let uses_send_owned = nir_uses_cpu_extern_call(module, "host_network_send_owned");
    let uses_recv_owned = nir_uses_cpu_extern_call(module, "host_network_recv_owned");
    let uses_recv_http_status_owned =
        nir_uses_cpu_extern_call(module, "host_network_recv_http_status_owned");
    let uses_close_owned = nir_uses_cpu_extern_call(module, "host_network_close_owned");

    let has_transport_owned_source = uses_open_tcp_stream
        || uses_open_udp_datagram
        || uses_bind_udp_datagram
        || uses_accept_owned;
    let has_any_owned_source = has_transport_owned_source || uses_open_tcp_listener;

    if uses_accept_owned && !uses_open_tcp_listener {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to open a listener via `host_network_open_tcp_listener(...)` before `host_network_accept_owned(...)`",
            from, to
        ));
    }
    if uses_send_owned && !has_transport_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned transport handle before `host_network_send_owned(...)`",
            from, to
        ));
    }
    if uses_recv_owned && !has_transport_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned transport handle before `host_network_recv_owned(...)`",
            from, to
        ));
    }
    if uses_recv_http_status_owned && !has_transport_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned transport handle before `host_network_recv_http_status_owned(...)`",
            from, to
        ));
    }
    if uses_close_owned && !has_any_owned_source {
        return Err(format!(
            "project link `{}` -> `{}` requires CPU entry to establish an owned network handle before `host_network_close_owned(...)`",
            from, to
        ));
    }

    Ok(())
}

pub(super) fn validate_network_owned_handle_provenance(
    module: &NirModule,
    from: &str,
    to: &str,
) -> Result<(), String> {
    let function_requirements = infer_network_function_handle_requirements(module)?;
    let function_return_kinds =
        infer_network_function_return_kinds(module, &function_requirements)?;
    for function in &module.functions {
        let mut bindings = BTreeMap::new();
        seed_network_param_bindings(function, &function_requirements, &mut bindings);
        validate_network_owned_handle_provenance_in_body(
            &function.body,
            from,
            to,
            &mut bindings,
            &function_requirements,
            &function_return_kinds,
        )?;
    }
    Ok(())
}

fn validate_network_owned_handle_provenance_in_body(
    body: &[NirStmt],
    from: &str,
    to: &str,
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } => {
                if let Some(kind) =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                {
                    bindings.insert(name.clone(), kind);
                } else {
                    bindings.remove(name);
                }
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
            NirStmt::Const { name, value, .. } => {
                if let Some(kind) =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                {
                    bindings.insert(name.clone(), kind);
                } else {
                    bindings.remove(name);
                }
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
            NirStmt::Print(value)
            | NirStmt::Await(value)
            | NirStmt::Expr(value)
            | NirStmt::Return(Some(value)) => {
                validate_network_owned_handle_provenance_in_expr(
                    value,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
            }
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                validate_network_owned_handle_provenance_in_expr(
                    condition,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let mut then_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    then_body,
                    from,
                    to,
                    &mut then_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let mut else_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    else_body,
                    from,
                    to,
                    &mut else_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                merge_network_owned_handle_bindings(bindings, &then_bindings, &else_bindings);
            }
            NirStmt::While { condition, body } => {
                validate_network_owned_handle_provenance_in_expr(
                    condition,
                    from,
                    to,
                    bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let entry_bindings = bindings.clone();
                let mut loop_bindings = bindings.clone();
                validate_network_owned_handle_provenance_in_body(
                    body,
                    from,
                    to,
                    &mut loop_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                merge_network_owned_handle_bindings(bindings, &entry_bindings, &loop_bindings);
            }
            NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
        }
    }
    Ok(())
}

fn merge_network_owned_handle_bindings(
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
    then_bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    else_bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) {
    let merged = bindings
        .keys()
        .chain(then_bindings.keys())
        .chain(else_bindings.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for name in merged {
        match (
            then_bindings.get(&name).copied(),
            else_bindings.get(&name).copied(),
        ) {
            (Some(lhs), Some(rhs)) if lhs == rhs => {
                bindings.insert(name, lhs);
            }
            _ => {
                bindings.remove(&name);
            }
        }
    }
}

#[cfg(test)]
#[path = "runtime_validation_network_handles_tests.rs"]
mod tests;
