use super::{
    coerce_to_i64, coerce_to_loop_scalar, emit_loop_numeric_op, fresh_reg, CpuLoopScalarKind,
    LlvmValueRef,
};

pub(crate) fn try_resolve_loop_carry_read_source(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    carry_kind: &str,
    raw_payloads: &[LlvmValueRef],
    payloads: &[String],
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<Option<(String, &'static str)>, String> {
    if matches!(carry_kind, "add_read_value_fixed" | "mul_read_value_fixed") {
        let ptr = expect_ptr_payload(
            raw_payloads,
            "fixed read pointer",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let source = emit_load_i64_source(
            body,
            next_reg,
            loop_scalar_kind,
            &ptr,
            None,
            "fixed read source",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, read_op(carry_kind))));
    }

    if matches!(
        carry_kind,
        "add_read_value_fixed_plus_invariant" | "mul_read_value_fixed_plus_invariant"
    ) {
        let ptr = expect_ptr_payload(
            raw_payloads,
            "fixed read pointer",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let offset = expect_payload(
            payloads,
            "invariant payload",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let read_source = emit_load_i64_source(
            body,
            next_reg,
            loop_scalar_kind,
            &ptr,
            None,
            "fixed read source",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let source = add_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            &read_source,
            offset,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, read_op(carry_kind))));
    }

    if matches!(carry_kind, "add_read_at_fixed" | "mul_read_at_fixed") {
        let ptr = expect_ptr_payload(
            raw_payloads,
            "fixed indexed-read buffer",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let index_value = expect_i64_payload(
            raw_payloads,
            1,
            body,
            next_reg,
            "fixed indexed-read index",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let source = emit_load_i64_source(
            body,
            next_reg,
            loop_scalar_kind,
            &ptr,
            Some(&index_value),
            "fixed indexed-read source",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, read_op(carry_kind))));
    }

    if matches!(
        carry_kind,
        "add_read_at_fixed_plus_invariant" | "mul_read_at_fixed_plus_invariant"
    ) {
        let ptr = expect_ptr_payload(
            raw_payloads,
            "fixed indexed-read buffer",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let index_value = expect_i64_payload(
            raw_payloads,
            1,
            body,
            next_reg,
            "fixed indexed-read index",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let offset = expect_payload(
            payloads,
            "invariant payload",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let read_source = emit_load_i64_source(
            body,
            next_reg,
            loop_scalar_kind,
            &ptr,
            Some(&index_value),
            "fixed indexed-read source",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        let source = add_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            &read_source,
            offset,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, read_op(carry_kind))));
    }

    if !is_dynamic_read_kind(carry_kind) {
        return Ok(None);
    }

    let buffer_ptr = expect_ptr_payload(
        raw_payloads,
        "dynamic read buffer",
        carry_kind,
        node_name,
        loop_instruction,
    )?;
    let dynamic_kind = carry_kind
        .strip_suffix("_plus_invariant")
        .unwrap_or(carry_kind);
    let index_value = dynamic_read_index(
        dynamic_kind,
        carry_kind,
        current,
        next_current,
        current_carries,
        next_carries,
        node_name,
        loop_instruction,
    )?;
    let read_source = emit_load_i64_source(
        body,
        next_reg,
        loop_scalar_kind,
        &buffer_ptr,
        Some(&index_value),
        "dynamic read source",
        carry_kind,
        node_name,
        loop_instruction,
    )?;
    let source = if carry_kind.ends_with("_plus_invariant") {
        let offset = expect_payload(
            payloads,
            "invariant payload",
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        add_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            &read_source,
            offset,
            node_name,
            loop_instruction,
        )?
    } else {
        read_source
    };
    Ok(Some((source, read_op(carry_kind))))
}

fn read_op(carry_kind: &str) -> &'static str {
    if carry_kind.starts_with("add_") {
        "add"
    } else {
        "mul"
    }
}

fn expect_ptr_payload(
    raw_payloads: &[LlvmValueRef],
    label: &str,
    carry_kind: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    match raw_payloads.first() {
        Some(LlvmValueRef::Ptr(ptr)) | Some(LlvmValueRef::BorrowedBuffer { ptr, .. }) => {
            Ok(ptr.clone())
        }
        _ => Err(format!(
            "cpu.{loop_instruction} `{node_name}` is missing {label} payload for `{carry_kind}` during LLVM lowering",
        )),
    }
}

fn expect_payload<'a>(
    payloads: &'a [String],
    label: &str,
    carry_kind: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<&'a String, String> {
    payloads.last().ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` is missing {label} for `{carry_kind}` during LLVM lowering",
        )
    })
}

fn expect_i64_payload(
    raw_payloads: &[LlvmValueRef],
    index: usize,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    label: &str,
    carry_kind: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    raw_payloads
        .get(index)
        .and_then(|value| coerce_to_i64(value, body, next_reg))
        .ok_or_else(|| {
            format!(
                "cpu.{loop_instruction} `{node_name}` is missing {label} payload for `{carry_kind}` during LLVM lowering",
            )
        })
}

fn emit_load_i64_source(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    ptr: &str,
    index_value: Option<&str>,
    label: &str,
    carry_kind: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    let source_ptr = if let Some(index_value) = index_value {
        let slot = fresh_reg(next_reg);
        body.push(format!(
            "  {slot} = getelementptr inbounds i64, ptr {ptr}, i64 {index_value}"
        ));
        slot
    } else {
        ptr.to_owned()
    };
    let loaded = fresh_reg(next_reg);
    body.push(format!("  {loaded} = load i64, ptr {source_ptr}"));
    coerce_to_loop_scalar(
        &LlvmValueRef::I64(loaded),
        loop_scalar_kind,
        body,
        next_reg,
    )
    .ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` cannot coerce {label} `{carry_kind}` to the selected loop scalar kind during LLVM lowering",
        )
    })
}

fn add_invariant(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    source: &str,
    offset: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    emit_loop_numeric_op(body, next_reg, loop_scalar_kind, "add", source, offset).map_err(|error| {
        format!("cpu.{loop_instruction} `{node_name}` {error} during LLVM lowering")
    })
}

fn is_dynamic_read_kind(carry_kind: &str) -> bool {
    matches!(
        carry_kind,
        "add_read_at_dynamic_current"
            | "add_read_at_dynamic_prev_current"
            | "mul_read_at_dynamic_current"
            | "mul_read_at_dynamic_prev_current"
            | "add_read_at_dynamic_current_plus_invariant"
            | "add_read_at_dynamic_prev_current_plus_invariant"
            | "mul_read_at_dynamic_current_plus_invariant"
            | "mul_read_at_dynamic_prev_current_plus_invariant"
    ) || dynamic_carry_suffix(carry_kind).is_some()
}

fn dynamic_carry_suffix(carry_kind: &str) -> Option<&str> {
    [
        "add_read_at_dynamic_prev_carry",
        "mul_read_at_dynamic_prev_carry",
        "add_read_at_dynamic_carry",
        "mul_read_at_dynamic_carry",
    ]
    .iter()
    .find_map(|prefix| carry_kind.strip_prefix(prefix))
}

fn dynamic_read_index(
    dynamic_kind: &str,
    carry_kind: &str,
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    if dynamic_kind.ends_with("_prev_current") {
        return Ok(current.to_owned());
    }
    if dynamic_kind.ends_with("_current") {
        return Ok(next_current.to_owned());
    }
    resolve_dynamic_carry_index(
        dynamic_kind,
        "add_read_at_dynamic_prev_carry",
        current_carries,
        carry_kind,
        node_name,
        loop_instruction,
    )
    .or_else(|_| {
        resolve_dynamic_carry_index(
            dynamic_kind,
            "mul_read_at_dynamic_prev_carry",
            current_carries,
            carry_kind,
            node_name,
            loop_instruction,
        )
    })
    .or_else(|_| {
        resolve_dynamic_carry_index(
            dynamic_kind,
            "add_read_at_dynamic_carry",
            next_carries,
            carry_kind,
            node_name,
            loop_instruction,
        )
    })
    .or_else(|_| {
        resolve_dynamic_carry_index(
            dynamic_kind,
            "mul_read_at_dynamic_carry",
            next_carries,
            carry_kind,
            node_name,
            loop_instruction,
        )
    })
}

fn resolve_dynamic_carry_index(
    dynamic_kind: &str,
    prefix: &str,
    carries: &[String],
    carry_kind: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    let Some(rest) = dynamic_kind.strip_prefix(prefix) else {
        return Err(format!(
            "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
        ));
    };
    let source_index = rest.parse::<usize>().map_err(|_| {
        format!(
            "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
        )
    })?;
    carries.get(source_index).cloned().ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` references unavailable carry source `{carry_kind}` during LLVM lowering",
        )
    })
}
