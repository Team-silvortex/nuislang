use super::*;

pub(super) fn lower(
    node: &Node,
    body: &mut Vec<String>,
    registers: &BTreeMap<String, LlvmValueRef>,
    lengths: &BTreeMap<String, String>,
    signatures: &BTreeMap<String, CpuHelperSignature>,
    return_kind: CpuCallScalarKind,
    return_layout: Option<&yir_core::OwnedStructLayout>,
    next_reg: &mut usize,
    next_block: &mut usize,
) -> Result<bool, String> {
    let tail = node.op.instruction == "tail_call_owned_struct_return";
    if node.op.module != "cpu" || (!tail && node.op.instruction != "guard_call_owned_struct_return")
    {
        return Ok(false);
    }
    let call = if tail {
        yir_core::guarded_owned_call::parse_tail(&node.op.args)?
    } else {
        yir_core::guarded_owned_call::parse(&node.op.args)?
    };
    let callee = call.callee;
    let layout = yir_core::parse_owned_struct_layout(call.layout)?;
    let signature = signatures
        .get(callee)
        .ok_or("guarded owned helper is unavailable")?;
    if return_kind != CpuCallScalarKind::I64
        || return_layout != Some(&layout)
        || signature.ret != CpuCallScalarKind::I64
        || signature.owned_struct_layout.as_ref() != Some(&layout)
        || !signature.implicit_parameters.is_empty()
        || signature.mutex_permit_params.len() != signature.params.len()
        || signature.params.len() != call.inputs.len() + call.transfers.len()
        || signature.params.first().and_then(|p| p.scalar())
            != Some(CpuCallScalarKind::BorrowedBuffer)
        || signature.params[1..call.inputs.len()].iter().any(|p| {
            !matches!(
                p.scalar(),
                Some(CpuCallScalarKind::I64 | CpuCallScalarKind::Bool)
            )
        })
        || signature.params[call.inputs.len()..]
            .iter()
            .any(|p| p.scalar() != Some(CpuCallScalarKind::OwnedBytes))
        || signature.mutex_permit_params.iter().any(Option::is_some)
    {
        return Err("guarded owned helper signature or return layout drift".into());
    }
    let condition = if tail {
        None
    } else {
        let Some(LlvmValueRef::Bool { i1, .. }) = registers.get(call.condition) else {
            return Err("guarded owned call requires an available bool condition".into());
        };
        Some(i1)
    };
    let mut inputs = call_lowering::lower_branch_scalar_args(
        registers,
        lengths,
        call.inputs,
        &signature.params[..call.inputs.len()],
        &signature.mutex_permit_params[..call.inputs.len()],
    )
    .ok_or("guarded owned call has unavailable captures")?;
    let owners = call
        .transfers
        .iter()
        .chain(call.cleanup)
        .map(|name| match registers.get(name) {
            Some(LlvmValueRef::OwnedBytes { blob }) => Ok(blob.clone()),
            _ => Err("guarded ownership requires an available Bytes owner".to_owned()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if owners.iter().collect::<BTreeSet<_>>().len() != owners.len() {
        return Err("guarded ownership aliases one Bytes allocation".into());
    }
    inputs.extend(
        owners[..call.transfers.len()]
            .iter()
            .map(|blob| format!("ptr {blob}")),
    );
    let inputs = inputs.join(", ");
    let continuation = condition.map(|condition| {
        let selected = fresh_block(next_block, "guard_owned_call_then");
        let continuation = fresh_block(next_block, "guard_owned_call_cont");
        body.push(format!(
            "  br i1 {condition}, label %{selected}, label %{continuation}"
        ));
        body.push(format!("{selected}:"));
        continuation
    });
    let result = fresh_reg(next_reg);
    for blob in &owners[call.transfers.len()..] {
        body.push(format!(
            "  call void @nuis_scheduler_owned_blob_drop_v1(ptr {blob})"
        ));
    }
    body.push(format!("  {result} = call i64 @nuis_fn_{callee}({inputs})"));
    // The helper already packed and transferred the entire owned aggregate.
    // Forward failure (zero) unchanged; do not decode, copy or pack it again.
    body.push(format!("  ret i64 {result}"));
    if let Some(continuation) = continuation {
        body.push(format!("{continuation}:"));
    }
    Ok(true)
}

#[cfg(test)]
#[path = "guarded_owned_call_tests.rs"]
mod tests;
