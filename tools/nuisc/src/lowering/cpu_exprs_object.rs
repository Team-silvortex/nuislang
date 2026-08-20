use super::*;

pub(super) fn lower_cpu_extern_call_owned_object(
    abi: &str,
    callee: &str,
    signature: &str,
    args: &[NirExpr],
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if let Some(target_config) = &state.target_config {
        if !target_config.supports_host_ffi_abi(abi) {
            return Err(format!(
                "extern ABI `{abi}` is not supported by lowering target `{}`",
                target_config.abi
            ));
        }
    }
    let signature_hash = yir_core::ffi::ffi_symbol_signature_hash(abi, callee, signature);
    let registry = state.host_ffi_registry.as_ref().ok_or_else(|| {
        format!("owned extern object `{callee}` requires a loaded hash-bound host FFI registry")
    })?;
    let capability = registry
        .memory_capabilities(abi, callee, &signature_hash)
        .iter()
        .find(|capability| {
            capability.kind == HostFfiMemoryKind::OwnedReturnObject
                && capability.slot == HostFfiMemorySlot::Return
        })
        .ok_or_else(|| {
            format!(
                "owned extern object `{callee}` ABI `{abi}` signature `{signature}` hash `{signature_hash}` has no exact registered return capability"
            )
        })?;
    let HostFfiMemoryDestructor::Registered {
        symbol: destructor_symbol,
        signature_hash: destructor_signature_hash,
    } = &capability.destructor
    else {
        return Err(format!(
            "owned extern object `{callee}` capability does not name a registered destructor"
        ));
    };
    let capability_hash = capability.capability_hash.clone();
    let size_policy = capability.size.clone().unwrap_or_default();
    let read_policy = capability.read.clone().unwrap_or_default();
    let destructor_symbol = destructor_symbol.clone();
    let destructor_signature_hash = destructor_signature_hash.clone();
    let lowered_args = args
        .iter()
        .map(|arg| lower_expr(arg, state, bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let name = next_name(state, "cpu_extern_owned_object");
    let mut op_args = vec![
        yir_core::ffi::OWNED_OBJECT_RETURN_PROTOCOL.to_owned(),
        abi.to_owned(),
        callee.to_owned(),
        signature.to_owned(),
        signature_hash,
        capability_hash,
        size_policy,
        read_policy,
        destructor_symbol,
        destructor_signature_hash,
    ];
    op_args.extend(lowered_args.clone());
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "extern_call_owned_object".to_owned(),
            args: op_args,
        },
    });
    for arg in lowered_args {
        push_dep_edges(state, &arg, &name);
    }
    Ok(name)
}
