use super::*;

pub(super) fn direct_call_scalar_kind(ty: &NirTypeRef) -> Option<DirectCallScalarKind> {
    if ty.is_mutex_permit_family() && ty.generic_args.len() == 1 && !ty.is_optional {
        return Some(DirectCallScalarKind::I64);
    }
    if ty.is_optional || !ty.generic_args.is_empty() {
        return None;
    }
    if ty.is_ref {
        return match ty.name.as_str() {
            "Buffer" => Some(DirectCallScalarKind::BorrowedBuffer),
            "Node" => Some(DirectCallScalarKind::TraversalPointer),
            _ => None,
        };
    }
    if ty.name == "Bytes" {
        return Some(DirectCallScalarKind::OwnedBytes);
    }
    if ty.is_bool_scalar() {
        Some(DirectCallScalarKind::Bool)
    } else if ty.name == "i32" {
        Some(DirectCallScalarKind::I32)
    } else if ty.name == "i64" {
        Some(DirectCallScalarKind::I64)
    } else if ty.name == "f32" {
        Some(DirectCallScalarKind::F32)
    } else if ty.name == "f64" {
        Some(DirectCallScalarKind::F64)
    } else {
        None
    }
}

pub(super) fn is_scheduler_scalar_kind(kind: DirectCallScalarKind) -> bool {
    matches!(
        kind,
        DirectCallScalarKind::Bool
            | DirectCallScalarKind::I32
            | DirectCallScalarKind::I64
            | DirectCallScalarKind::F32
            | DirectCallScalarKind::F64
    )
}

pub(in crate::lowering) fn supports_direct_call_signature(function: &NirFunction) -> bool {
    direct_call_signature_kind(function)
        .is_some_and(|kind| kind != DirectCallScalarKind::OwnedExternalBuffer)
}

pub(super) fn direct_call_signature_kind(function: &NirFunction) -> Option<DirectCallScalarKind> {
    if function
        .return_type
        .as_ref()
        .is_some_and(NirTypeRef::is_mutex_permit_family)
    {
        return None;
    }
    let return_type = function.return_type.as_ref()?;
    let return_kind = if is_owned_external_buffer_type(return_type) {
        DirectCallScalarKind::OwnedExternalBuffer
    } else {
        direct_call_scalar_kind(return_type)?
    };
    if matches!(
        return_kind,
        DirectCallScalarKind::BorrowedBuffer | DirectCallScalarKind::TraversalPointer
    ) {
        return None;
    }
    for param in &function.params {
        direct_call_scalar_kind(&param.ty)?;
    }
    Some(return_kind)
}

pub(in crate::lowering) fn collect_owned_external_buffer_return_helpers(
    module: &NirModule,
) -> BTreeSet<String> {
    let eligible = module
        .functions
        .iter()
        .filter(|function| !function.is_async)
        .filter(|function| {
            direct_call_signature_kind(function) == Some(DirectCallScalarKind::OwnedExternalBuffer)
        })
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    let Some(main) = module
        .functions
        .iter()
        .find(|function| function.name == "main")
    else {
        return BTreeSet::new();
    };
    let roots = super::function_called_functions(main, &main.body, &eligible);
    let mut selected = roots.clone();
    for root in roots {
        let Some(function) = module
            .functions
            .iter()
            .find(|function| function.name == root)
        else {
            continue;
        };
        selected.extend(super::function_called_functions(
            function,
            &function.body,
            &eligible,
        ));
    }
    selected
}

pub(in crate::lowering) fn owned_external_buffer_helper_lowering_order(
    module: &NirModule,
    selected: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let eligible = selected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let dependencies = module
        .functions
        .iter()
        .filter(|function| selected.contains(&function.name))
        .map(|function| {
            (
                function.name.clone(),
                super::function_called_functions(function, &function.body, &eligible),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut pending = selected.clone();
    let mut order = Vec::with_capacity(selected.len());
    while !pending.is_empty() {
        let Some(next) = pending
            .iter()
            .find(|name| {
                dependencies
                    .get(name.as_str())
                    .is_none_or(|called| called.iter().all(|callee| !pending.contains(callee)))
            })
            .cloned()
        else {
            return Err(
                "recursive owned-buffer helper transfers remain closed during bootstrap lowering"
                    .to_owned(),
            );
        };
        pending.remove(&next);
        order.push(next);
    }
    Ok(order)
}

pub(super) fn owned_external_buffer_metadata_for_node(
    yir: &YirModule,
    node_name: &str,
) -> Result<Vec<String>, String> {
    let node = yir
        .nodes
        .iter()
        .find(|node| node.name == node_name)
        .ok_or_else(|| format!("owned external buffer return references missing `{node_name}`"))?;
    let (abi, destructor, destructor_hash) = match (
        node.op.module.as_str(),
        node.op.instruction.as_str(),
    ) {
        ("cpu", "extern_call_owned_buffer") => {
            let contract = yir_core::ffi::parse_owned_buffer_return_contract(&node.op.args)
                .map_err(|error| {
                    format!("owned external buffer helper has invalid authority: {error}")
                })?;
            (
                contract.abi,
                contract.destructor_symbol,
                contract.destructor_signature_hash,
            )
        }
        ("cpu", "call_owned_external_buffer") => {
            let contract = yir_core::ffi::parse_owned_buffer_function_transfer_contract(
                node.op.args.get(1..).unwrap_or_default(),
            )
            .map_err(|error| {
                format!("owned external buffer helper call has invalid authority: {error}")
            })?;
            (
                contract.abi,
                contract.destructor_symbol,
                contract.destructor_signature_hash,
            )
        }
        _ => {
            return Err(format!(
                "owned external buffer helper must return one registered producer or helper transfer, found `{}.{}`",
                node.op.module, node.op.instruction
            ));
        }
    };
    Ok(
        yir_core::ffi::owned_buffer_function_transfer_metadata(abi, destructor, destructor_hash)
            .into_iter()
            .collect(),
    )
}

pub(super) fn owned_external_buffer_metadata_for_helper(
    yir: &YirModule,
    function_name: &str,
) -> Result<Vec<String>, String> {
    let function = yir
        .functions
        .iter()
        .find(|function| function.name == function_name)
        .ok_or_else(|| format!("owned external buffer helper `{function_name}` is unavailable"))?;
    let result = function
        .result
        .as_ref()
        .ok_or_else(|| format!("owned external buffer helper `{function_name}` has no result"))?;
    let node = yir
        .nodes
        .iter()
        .find(|node| node.name == result.node)
        .ok_or_else(|| {
            format!("owned external buffer helper `{function_name}` result is missing")
        })?;
    if node.op.module != "cpu" || node.op.instruction != "return_owned_external_buffer" {
        return Err(format!(
            "owned external buffer helper `{function_name}` lacks its registered return transfer"
        ));
    }
    let contract = yir_core::ffi::parse_owned_buffer_function_transfer_contract(&node.op.args[1..])
        .map_err(|error| {
            format!("owned external buffer helper `{function_name}` has invalid transfer: {error}")
        })?;
    if !contract.inputs.is_empty() {
        return Err(format!(
            "owned external buffer helper `{function_name}` return transfer has trailing inputs"
        ));
    }
    Ok(node.op.args[1..].to_vec())
}

fn is_owned_external_buffer_type(ty: &NirTypeRef) -> bool {
    ty.is_ref && !ty.is_optional && ty.generic_args.is_empty() && ty.name == "Buffer"
}
