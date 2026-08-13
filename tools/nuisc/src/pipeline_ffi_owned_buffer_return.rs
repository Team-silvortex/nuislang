use std::collections::BTreeMap;

use yir_core::{Node, YirFunction, YirFunctionRole, YirModule, YirValueOwnership};

#[derive(PartialEq, Eq)]
struct TransferIdentity<'a> {
    abi: &'a str,
    destructor: &'a str,
    destructor_hash: &'a str,
}

pub(super) fn validate_owned_buffer_function_transfers(module: &YirModule) -> Result<(), String> {
    let nodes = module
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let functions = module
        .functions
        .iter()
        .map(|function| (function.name.as_str(), function))
        .collect::<BTreeMap<_, _>>();

    for returned in module.nodes.iter().filter(|node| {
        node.op.module == "cpu" && node.op.instruction == "return_owned_external_buffer"
    }) {
        validate_return_transfer(module, returned, &nodes)?;
    }
    for call in module.nodes.iter().filter(|node| {
        node.op.module == "cpu" && node.op.instruction == "call_owned_external_buffer"
    }) {
        validate_call_transfer(module, call, &nodes, &functions)?;
    }
    Ok(())
}

fn validate_return_transfer(
    module: &YirModule,
    returned: &Node,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<(), String> {
    if returned.op.args.len() != 5 {
        return Err(format!(
            "owned-buffer function return `{}` must carry one owner and exact transfer metadata",
            returned.name
        ));
    }
    let transfer =
        yir_core::ffi::parse_owned_buffer_function_transfer_contract(&returned.op.args[1..])
            .map_err(|error| {
                format!("owned-buffer function return `{}`: {error}", returned.name)
            })?;
    if !transfer.inputs.is_empty() {
        return Err(format!(
            "owned-buffer function return `{}` has trailing transfer inputs",
            returned.name
        ));
    }
    let owner = nodes
        .get(returned.op.args[0].as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "owned-buffer function return `{}` references missing owner `{}`",
                returned.name, returned.op.args[0]
            )
        })?;
    let owner_identity = match (owner.op.module.as_str(), owner.op.instruction.as_str()) {
        ("cpu", "extern_call_owned_buffer") => {
            let producer = yir_core::ffi::parse_owned_buffer_return_contract(&owner.op.args)
                .map_err(|error| {
                    format!("owned-buffer return producer `{}`: {error}", owner.name)
                })?;
            producer_identity(&producer)
        }
        ("cpu", "call_owned_external_buffer") => {
            let call = yir_core::ffi::parse_owned_buffer_function_transfer_contract(
                owner.op.args.get(1..).unwrap_or_default(),
            )
            .map_err(|error| {
                format!("owned-buffer return helper call `{}`: {error}", owner.name)
            })?;
            transfer_identity(&call)
        }
        _ => {
            return Err(format!(
                "owned-buffer function return `{}` requires one registered producer or helper transfer",
                returned.name
            ));
        }
    };
    if transfer_identity(&transfer) != owner_identity {
        return Err(format!(
            "owned-buffer function return `{}` does not match producer ABI/destructor/hash identity",
            returned.name
        ));
    }

    let function = unique_containing_function(module, &returned.name, "function return")?;
    if function.role != YirFunctionRole::Helper
        || !function.body_nodes.contains(&owner.name)
        || function.result.as_ref().is_none_or(|result| {
            result.node != returned.name
                || result.ty != "ref Buffer"
                || result.ownership != YirValueOwnership::Owned
        })
    {
        return Err(format!(
            "owned-buffer function return `{}` must close one owned helper result",
            returned.name
        ));
    }
    if !module.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "call_owned_external_buffer"
            && node.op.args.first() == Some(&function.name)
    }) {
        return Err(format!(
            "owned-buffer helper `{}` must have at least one registered caller",
            function.name
        ));
    }
    Ok(())
}

fn validate_call_transfer(
    module: &YirModule,
    call: &Node,
    nodes: &BTreeMap<&str, &Node>,
    functions: &BTreeMap<&str, &YirFunction>,
) -> Result<(), String> {
    if call.op.args.len() < 5 {
        return Err(format!(
            "owned-buffer function call `{}` lacks exact transfer metadata",
            call.name
        ));
    }
    let callee = functions
        .get(call.op.args[0].as_str())
        .copied()
        .ok_or_else(|| {
            format!(
                "owned-buffer call `{}` references missing helper",
                call.name
            )
        })?;
    let result = callee.result.as_ref().ok_or_else(|| {
        format!(
            "owned-buffer call `{}` helper `{}` has no result",
            call.name, callee.name
        )
    })?;
    let returned = nodes.get(result.node.as_str()).copied().ok_or_else(|| {
        format!(
            "owned-buffer call `{}` helper `{}` has a missing result node",
            call.name, callee.name
        )
    })?;
    if callee.role != YirFunctionRole::Helper
        || returned.op.module != "cpu"
        || returned.op.instruction != "return_owned_external_buffer"
    {
        return Err(format!(
            "owned-buffer call `{}` must target a registered owner helper",
            call.name
        ));
    }
    let call_contract =
        yir_core::ffi::parse_owned_buffer_function_transfer_contract(&call.op.args[1..])
            .map_err(|error| format!("owned-buffer function call `{}`: {error}", call.name))?;
    let return_contract =
        yir_core::ffi::parse_owned_buffer_function_transfer_contract(&returned.op.args[1..])
            .map_err(|error| format!("owned-buffer helper `{}` return: {error}", callee.name))?;
    if transfer_identity(&call_contract) != transfer_identity(&return_contract) {
        return Err(format!(
            "owned-buffer call `{}` does not match helper `{}` ABI/destructor/hash identity",
            call.name, callee.name
        ));
    }
    if call_contract.inputs.len() != callee.parameters.len() {
        return Err(format!(
            "owned-buffer call `{}` expects {} helper input(s), found {}",
            call.name,
            callee.parameters.len(),
            call_contract.inputs.len()
        ));
    }
    let caller = unique_containing_function(module, &call.name, "function call")?;
    match caller.role {
        YirFunctionRole::Entry => {}
        YirFunctionRole::Helper => {
            let caller_result = caller.result.as_ref().ok_or_else(|| {
                format!(
                    "owned-buffer helper caller `{}` must expose its transferred owner",
                    caller.name
                )
            })?;
            let caller_return =
                nodes
                    .get(caller_result.node.as_str())
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "owned-buffer helper caller `{}` has a missing result node",
                            caller.name
                        )
                    })?;
            if caller_result.ty != "ref Buffer"
                || caller_result.ownership != YirValueOwnership::Owned
                || caller_return.op.module != "cpu"
                || caller_return.op.instruction != "return_owned_external_buffer"
                || caller_return.op.args.first() != Some(&call.name)
            {
                return Err(format!(
                    "owned-buffer helper call `{}` must move directly into caller `{}` result",
                    call.name, caller.name
                ));
            }
            let callee_owner = returned
                .op
                .args
                .first()
                .and_then(|name| nodes.get(name.as_str()))
                .copied()
                .ok_or_else(|| {
                    format!(
                        "owned-buffer helper `{}` return owner is missing",
                        callee.name
                    )
                })?;
            if callee_owner.op.module != "cpu"
                || callee_owner.op.instruction != "extern_call_owned_buffer"
            {
                return Err(format!(
                    "owned-buffer helper call `{}` exceeds the single registered helper-to-helper transfer boundary",
                    call.name
                ));
            }
        }
        _ => {
            return Err(format!(
                "owned-buffer call `{}` may transfer only into an entry or owned-result helper",
                call.name
            ));
        }
    }
    Ok(())
}

fn unique_containing_function<'a>(
    module: &'a YirModule,
    node: &str,
    label: &str,
) -> Result<&'a YirFunction, String> {
    let functions = module
        .functions
        .iter()
        .filter(|function| function.body_nodes.iter().any(|name| name == node))
        .collect::<Vec<_>>();
    let [function] = functions.as_slice() else {
        return Err(format!(
            "owned-buffer {label} `{node}` must belong to exactly one YIR function"
        ));
    };
    Ok(function)
}

fn transfer_identity<'a>(
    contract: &yir_core::ffi::OwnedBufferFunctionTransferContract<'a>,
) -> TransferIdentity<'a> {
    TransferIdentity {
        abi: contract.abi,
        destructor: contract.destructor_symbol,
        destructor_hash: contract.destructor_signature_hash,
    }
}

fn producer_identity<'a>(
    contract: &yir_core::ffi::OwnedBufferReturnContract<'a>,
) -> TransferIdentity<'a> {
    TransferIdentity {
        abi: contract.abi,
        destructor: contract.destructor_symbol,
        destructor_hash: contract.destructor_signature_hash,
    }
}
