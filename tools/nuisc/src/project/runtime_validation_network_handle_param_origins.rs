use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::NirExpr;

use super::super::NetworkOwnedHandleRequirement;
use super::param_merge::merge_network_owned_handle_requirement;

pub(super) fn infer_network_param_requirement_from_host_call(
    callee: &str,
    args: &[NirExpr],
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    bindings: &BTreeMap<String, usize>,
) -> Result<(), String> {
    let requirement = match callee {
        "host_network_accept_owned" => Some(NetworkOwnedHandleRequirement::Listener),
        "host_network_send_owned" | "host_network_recv_owned" => {
            Some(NetworkOwnedHandleRequirement::Transport)
        }
        "host_network_recv_http_status_owned" => {
            Some(NetworkOwnedHandleRequirement::StreamTransport)
        }
        "host_network_close_owned" => Some(NetworkOwnedHandleRequirement::OwnedAny),
        _ => None,
    };
    let Some(requirement) = requirement else {
        return Ok(());
    };
    let Some(origin) = args
        .first()
        .and_then(|arg| infer_network_param_origin(arg, bindings))
    else {
        return Ok(());
    };
    merge_network_param_requirement(requirements, origin, requirement, callee)
}

pub(super) fn merge_network_param_requirement(
    requirements: &mut [Option<NetworkOwnedHandleRequirement>],
    index: usize,
    incoming: NetworkOwnedHandleRequirement,
    context: &str,
) -> Result<(), String> {
    let slot = requirements.get_mut(index).ok_or_else(|| {
        format!(
            "network handle requirement index {} out of bounds in {}",
            index, context
        )
    })?;
    *slot = Some(match *slot {
        None => incoming,
        Some(existing) => {
            merge_network_owned_handle_requirement(existing, incoming).ok_or_else(|| {
                format!(
                    "function `{}` uses parameter {} as incompatible network handle kinds",
                    context, index
                )
            })?
        }
    });
    Ok(())
}

pub(super) fn infer_network_param_origin(
    expr: &NirExpr,
    bindings: &BTreeMap<String, usize>,
) -> Option<usize> {
    match expr {
        NirExpr::Var(name) => bindings.get(name).copied(),
        NirExpr::NetworkValue(inner) => infer_network_param_origin(inner, bindings),
        NirExpr::NetworkResult { value, .. } => infer_network_param_origin(value, bindings),
        _ => None,
    }
}

pub(super) fn merge_network_param_origin_bindings(
    bindings: &mut BTreeMap<String, usize>,
    then_bindings: &BTreeMap<String, usize>,
    else_bindings: &BTreeMap<String, usize>,
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
