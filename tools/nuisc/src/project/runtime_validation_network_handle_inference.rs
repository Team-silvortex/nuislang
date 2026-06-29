use std::collections::BTreeMap;

use nuis_semantics::model::{NirExpr, NirModule, NirStmt};

use super::{
    merge_network_owned_handle_bindings, NetworkOwnedHandleBinding, NetworkOwnedHandleKind,
    NetworkOwnedHandleRequirement, NetworkOwnedHandleReturn,
};

#[path = "runtime_validation_network_handle_param_inference.rs"]
mod param_inference;
use param_inference::infer_network_param_requirements_in_body;

pub(super) fn infer_network_owned_handle_kind(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Option<NetworkOwnedHandleBinding> {
    match expr {
        NirExpr::CpuExternCall { callee, .. } => match callee.as_str() {
            "host_network_open_tcp_listener" => Some(NetworkOwnedHandleBinding::Concrete(
                NetworkOwnedHandleKind::Listener,
            )),
            "host_network_open_tcp_stream" => Some(NetworkOwnedHandleBinding::Concrete(
                NetworkOwnedHandleKind::StreamTransport,
            )),
            "host_network_open_udp_datagram" | "host_network_bind_udp_datagram" => Some(
                NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport),
            ),
            "host_network_accept_owned" => Some(NetworkOwnedHandleBinding::Concrete(
                NetworkOwnedHandleKind::StreamTransport,
            )),
            _ => None,
        },
        NirExpr::Call { callee, args } => function_return_kinds
            .get(callee)
            .copied()
            .flatten()
            .and_then(|summary| {
                resolve_network_owned_handle_return(summary, args, bindings, function_return_kinds)
            }),
        NirExpr::NetworkValue(inner) => {
            infer_network_owned_handle_kind(inner, bindings, function_return_kinds)
        }
        NirExpr::NetworkResult { value, .. } => {
            infer_network_owned_handle_kind(value, bindings, function_return_kinds)
        }
        NirExpr::Var(name) => bindings.get(name).copied(),
        _ => None,
    }
}

fn resolve_network_owned_handle_return(
    summary: NetworkOwnedHandleReturn,
    args: &[NirExpr],
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Option<NetworkOwnedHandleBinding> {
    match summary {
        NetworkOwnedHandleReturn::Concrete(kind) => Some(NetworkOwnedHandleBinding::Concrete(kind)),
        NetworkOwnedHandleReturn::ParamIndex(index) => args
            .get(index)
            .and_then(|arg| infer_network_owned_handle_kind(arg, bindings, function_return_kinds)),
    }
}

pub(super) fn infer_network_function_handle_requirements(
    module: &NirModule,
) -> Result<BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>, String> {
    let mut requirements = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), vec![None; function.params.len()]))
        .collect::<BTreeMap<_, _>>();
    let mut changed = true;
    while changed {
        changed = false;
        for function in &module.functions {
            let mut next = requirements
                .get(&function.name)
                .cloned()
                .unwrap_or_else(|| vec![None; function.params.len()]);
            infer_network_param_requirements_in_body(
                &function.body,
                &function.params,
                &mut next,
                &requirements,
            )?;
            if requirements.get(&function.name) != Some(&next) {
                requirements.insert(function.name.clone(), next);
                changed = true;
            }
        }
    }
    Ok(requirements)
}

pub(super) fn infer_network_function_return_kinds(
    module: &NirModule,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
) -> Result<BTreeMap<String, Option<NetworkOwnedHandleReturn>>, String> {
    let mut return_kinds = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), None))
        .collect::<BTreeMap<_, _>>();
    let mut changed = true;
    while changed {
        changed = false;
        for function in &module.functions {
            let mut bindings = BTreeMap::new();
            seed_network_param_bindings(function, function_requirements, &mut bindings);
            let next = infer_network_return_kind_in_body(
                &function.body,
                &mut bindings,
                function_requirements,
                &return_kinds,
            )?;
            if return_kinds.get(&function.name) != Some(&next) {
                return_kinds.insert(function.name.clone(), next);
                changed = true;
            }
        }
    }
    Ok(return_kinds)
}

fn infer_network_return_kind_in_body(
    body: &[NirStmt],
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    function_return_kinds: &BTreeMap<String, Option<NetworkOwnedHandleReturn>>,
) -> Result<Option<NetworkOwnedHandleReturn>, String> {
    let mut return_kind = None;
    for stmt in body {
        match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                if let Some(kind) =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                {
                    bindings.insert(name.clone(), kind);
                } else {
                    bindings.remove(name);
                }
            }
            NirStmt::Return(Some(value)) => {
                let current =
                    infer_network_owned_handle_kind(value, bindings, function_return_kinds)
                        .and_then(binding_to_network_owned_handle_return);
                return_kind = merge_optional_network_owned_handle_kind(return_kind, current);
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                let mut then_bindings = bindings.clone();
                let then_kind = infer_network_return_kind_in_body(
                    then_body,
                    &mut then_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                let mut else_bindings = bindings.clone();
                let else_kind = infer_network_return_kind_in_body(
                    else_body,
                    &mut else_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                return_kind = merge_optional_network_owned_handle_kind(return_kind, then_kind);
                return_kind = merge_optional_network_owned_handle_kind(return_kind, else_kind);
                merge_network_owned_handle_bindings(bindings, &then_bindings, &else_bindings);
            }
            NirStmt::While { body, .. } => {
                let entry_bindings = bindings.clone();
                let mut loop_bindings = bindings.clone();
                let loop_kind = infer_network_return_kind_in_body(
                    body,
                    &mut loop_bindings,
                    function_requirements,
                    function_return_kinds,
                )?;
                return_kind = merge_optional_network_owned_handle_kind(return_kind, loop_kind);
                merge_network_owned_handle_bindings(bindings, &entry_bindings, &loop_bindings);
            }
            NirStmt::Print(_)
            | NirStmt::Await(_)
            | NirStmt::Expr(_)
            | NirStmt::Return(None)
            | NirStmt::Break
            | NirStmt::Continue => {}
        }
    }
    Ok(return_kind)
}

fn merge_optional_network_owned_handle_kind(
    lhs: Option<NetworkOwnedHandleReturn>,
    rhs: Option<NetworkOwnedHandleReturn>,
) -> Option<NetworkOwnedHandleReturn> {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) if lhs == rhs => Some(lhs),
        (Some(_), Some(_)) => None,
        (Some(lhs), None) => Some(lhs),
        (None, Some(rhs)) => Some(rhs),
        (None, None) => None,
    }
}

fn binding_to_network_owned_handle_return(
    binding: NetworkOwnedHandleBinding,
) -> Option<NetworkOwnedHandleReturn> {
    match binding {
        NetworkOwnedHandleBinding::Concrete(kind) => Some(NetworkOwnedHandleReturn::Concrete(kind)),
        NetworkOwnedHandleBinding::Param { index, .. } => {
            Some(NetworkOwnedHandleReturn::ParamIndex(index))
        }
    }
}

pub(super) fn seed_network_param_bindings(
    function: &nuis_semantics::model::NirFunction,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
    bindings: &mut BTreeMap<String, NetworkOwnedHandleBinding>,
) {
    let Some(requirements) = function_requirements.get(&function.name) else {
        return;
    };
    for (index, param) in function.params.iter().enumerate() {
        let Some(Some(requirement)) = requirements.get(index) else {
            continue;
        };
        bindings.insert(
            param.name.clone(),
            NetworkOwnedHandleBinding::Param {
                index,
                requirement: *requirement,
            },
        );
    }
}
