use std::collections::BTreeMap;

use nuis_semantics::model::NirExpr;

use super::super::{
    NetworkOwnedHandleBinding, NetworkOwnedHandleKind, NetworkOwnedHandleRequirement,
};

pub(super) fn validate_network_function_call_requirements(
    callee: &str,
    args: &[NirExpr],
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    function_requirements: &BTreeMap<String, Vec<Option<NetworkOwnedHandleRequirement>>>,
) -> Result<(), String> {
    let Some(requirements) = function_requirements.get(callee) else {
        return Ok(());
    };
    for (index, requirement) in requirements.iter().enumerate() {
        let Some(requirement) = requirement else {
            continue;
        };
        let arg = args.get(index);
        validate_network_call_arg_requirement(
            callee,
            index,
            arg,
            *requirement,
            from,
            to,
            bindings,
        )?;
    }
    Ok(())
}

fn validate_network_call_arg_requirement(
    callee: &str,
    index: usize,
    arg: Option<&NirExpr>,
    requirement: NetworkOwnedHandleRequirement,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) -> Result<(), String> {
    match requirement {
        NetworkOwnedHandleRequirement::OwnedAny => {
            let Some(arg) = arg else {
                return Ok(());
            };
            match arg {
                NirExpr::Var(name) if bindings.contains_key(name) => Ok(()),
                NirExpr::Var(name) => Err(format!(
                    "project link `{}` -> `{}` requires call `{}` arg {} to be an owned network handle variable, but `{}` does not come from an owned network open/accept path",
                    from, to, callee, index, name
                )),
                _ => Err(format!(
                    "project link `{}` -> `{}` requires call `{}` arg {} to be an owned network handle variable produced by an open/accept path",
                    from, to, callee, index
                )),
            }
        }
        NetworkOwnedHandleRequirement::Listener => validate_network_owned_handle_arg(
            callee,
            arg,
            NetworkOwnedHandleKind::Listener,
            from,
            to,
            bindings,
            "listener",
        ),
        NetworkOwnedHandleRequirement::Transport => {
            validate_network_transport_handle_arg(callee, arg, from, to, bindings)
        }
        NetworkOwnedHandleRequirement::StreamTransport => validate_network_owned_handle_arg(
            callee,
            arg,
            NetworkOwnedHandleKind::StreamTransport,
            from,
            to,
            bindings,
            "stream transport",
        ),
    }
}

pub(super) fn validate_network_owned_handle_call(
    callee: &str,
    args: &[NirExpr],
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) -> Result<(), String> {
    match callee {
        "host_network_accept_owned" => {
            validate_network_owned_handle_arg(
                callee,
                args.first(),
                NetworkOwnedHandleKind::Listener,
                from,
                to,
                bindings,
                "listener",
            )?;
        }
        "host_network_send_owned" | "host_network_recv_owned" => {
            validate_network_transport_handle_arg(callee, args.first(), from, to, bindings)?;
        }
        "host_network_recv_http_status_owned" => {
            validate_network_owned_handle_arg(
                callee,
                args.first(),
                NetworkOwnedHandleKind::StreamTransport,
                from,
                to,
                bindings,
                "stream transport",
            )?;
        }
        "host_network_close_owned" => {
            let Some(arg) = args.first() else {
                return Ok(());
            };
            match arg {
                NirExpr::Var(name) if bindings.contains_key(name) => {}
                NirExpr::Var(name) => {
                    return Err(format!(
                        "project link `{}` -> `{}` requires `host_network_close_owned(...)` to consume an owned handle variable, but `{}` does not come from an owned network open/accept path",
                        from, to, name
                    ));
                }
                _ => {
                    return Err(format!(
                        "project link `{}` -> `{}` requires `host_network_close_owned(...)` to consume an owned handle variable produced by an open/accept path",
                        from, to
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_network_owned_handle_arg(
    callee: &str,
    arg: Option<&NirExpr>,
    expected: NetworkOwnedHandleKind,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
    expected_label: &str,
) -> Result<(), String> {
    let Some(arg) = arg else {
        return Ok(());
    };
    match arg {
        NirExpr::Var(name) => match bindings.get(name).copied() {
            Some(NetworkOwnedHandleBinding::Concrete(kind)) if kind == expected => Ok(()),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Listener,
                ..
            }) if expected == NetworkOwnedHandleKind::Listener => Ok(()),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::StreamTransport,
                ..
            }) if expected == NetworkOwnedHandleKind::StreamTransport => Ok(()),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport))
                if expected == NetworkOwnedHandleKind::StreamTransport =>
            {
                Err(format!(
                    "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a datagram-owned source",
                    from, to, callee, expected_label, name
                ))
            }
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::Listener))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Listener,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a listener-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::StreamTransport))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::StreamTransport,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a stream-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport)) => {
                Err(format!(
                    "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` comes from a datagram-owned source",
                    from, to, callee, expected_label, name
                ))
            }
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Transport,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` only guarantees a generic transport-owned source",
                from, to, callee, expected_label, name
            )),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::OwnedAny,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` only guarantees an owned network source",
                from, to, callee, expected_label, name
            )),
            None => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a {} handle variable, but `{}` does not come from an owned network open/accept path",
                from, to, callee, expected_label, name
            )),
        },
        _ => Err(format!(
            "project link `{}` -> `{}` requires `{}` to consume a {} handle variable produced by an owned network open/accept path",
            from, to, callee, expected_label
        )),
    }
}

fn validate_network_transport_handle_arg(
    callee: &str,
    arg: Option<&NirExpr>,
    from: &str,
    to: &str,
    bindings: &BTreeMap<String, NetworkOwnedHandleBinding>,
) -> Result<(), String> {
    let Some(arg) = arg else {
        return Ok(());
    };
    match arg {
        NirExpr::Var(name) => match bindings.get(name).copied() {
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::StreamTransport))
            | Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::DatagramTransport))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Transport,
                ..
            })
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::StreamTransport,
                ..
            }) => Ok(()),
            Some(NetworkOwnedHandleBinding::Concrete(NetworkOwnedHandleKind::Listener))
            | Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::Listener,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` comes from a listener-owned source",
                from, to, callee, name
            )),
            Some(NetworkOwnedHandleBinding::Param {
                requirement: NetworkOwnedHandleRequirement::OwnedAny,
                ..
            }) => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` only guarantees an owned network source",
                from, to, callee, name
            )),
            None => Err(format!(
                "project link `{}` -> `{}` requires `{}` to consume a transport handle variable, but `{}` does not come from an owned network open/accept path",
                from, to, callee, name
            )),
        },
        _ => Err(format!(
            "project link `{}` -> `{}` requires `{}` to consume a transport handle variable produced by an owned network open/accept path",
            from, to, callee
        )),
    }
}
