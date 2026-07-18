use super::{emit_loop_numeric_op, resolve_loop_carry_term, CpuLoopScalarKind};

pub(crate) fn try_resolve_loop_carry_scaled_source(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    carry_kind: &str,
    payloads: &[String],
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<Option<(String, &'static str)>, String> {
    if let Some((factor_term, terms_part)) = carry_kind
        .strip_prefix("mul_scaled_by_")
        .and_then(|prefix| prefix.split_once("_plus_factor_invariant_"))
    {
        let (terms_part, has_invariant) = split_invariant_suffix(terms_part);
        let factor_offset = expect_payload(payloads, 0, carry_kind, node_name, loop_instruction)?;
        let factor = resolve_term(
            factor_term,
            carry_kind,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        let factor = emit_add(
            body,
            next_reg,
            loop_scalar_kind,
            &factor,
            factor_offset,
            node_name,
            loop_instruction,
        )?;
        let source = emit_terms_with_optional_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            terms_part,
            has_invariant.then_some(1),
            carry_kind,
            payloads,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        let source = emit_mul(
            body,
            next_reg,
            loop_scalar_kind,
            &source,
            &factor,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, "mul")));
    }

    if let Some(prefix) = carry_kind.strip_prefix("mul_scaled_by_") {
        let (factor_term, terms_part) = prefix.split_once('_').ok_or_else(|| {
            format!(
                "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
            )
        })?;
        let (terms_part, has_invariant) = split_invariant_suffix(terms_part);
        let factor = resolve_term(
            factor_term,
            carry_kind,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        let source = emit_terms_with_optional_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            terms_part,
            has_invariant.then_some(0),
            carry_kind,
            payloads,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        let source = emit_mul(
            body,
            next_reg,
            loop_scalar_kind,
            &source,
            &factor,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, "mul")));
    }

    if let Some(terms_part) = carry_kind.strip_prefix("mul_scaled_") {
        let (terms_part, has_invariant) = split_invariant_suffix(terms_part);
        let factor = expect_payload(payloads, 0, carry_kind, node_name, loop_instruction)?;
        let source = emit_terms_with_optional_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            terms_part,
            None,
            carry_kind,
            payloads,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        let mut source = emit_mul(
            body,
            next_reg,
            loop_scalar_kind,
            &source,
            factor,
            node_name,
            loop_instruction,
        )?;
        if has_invariant {
            let offset = expect_payload(payloads, 1, carry_kind, node_name, loop_instruction)?;
            source = emit_add(
                body,
                next_reg,
                loop_scalar_kind,
                &source,
                offset,
                node_name,
                loop_instruction,
            )?;
        }
        return Ok(Some((source, "mul")));
    }

    if let Some(terms_part) = carry_kind.strip_prefix("mul_") {
        let (terms_part, has_invariant) = split_invariant_suffix(terms_part);
        let source = emit_terms_with_optional_invariant(
            body,
            next_reg,
            loop_scalar_kind,
            terms_part,
            has_invariant.then_some(0),
            carry_kind,
            payloads,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        return Ok(Some((source, "mul")));
    }

    Ok(None)
}

fn split_invariant_suffix(terms_part: &str) -> (&str, bool) {
    if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
        (terms_part, true)
    } else {
        (terms_part, false)
    }
}

fn emit_terms_with_optional_invariant(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    terms_part: &str,
    invariant_index: Option<usize>,
    carry_kind: &str,
    payloads: &[String],
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    let mut terms = terms_part.split("_plus_");
    let first = terms.next().ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{carry_kind}` during LLVM lowering",
        )
    })?;
    let mut source = resolve_term(
        first,
        carry_kind,
        current,
        next_current,
        current_carries,
        next_carries,
        node_name,
        loop_instruction,
    )?;
    for term in terms {
        let rhs = resolve_term(
            term,
            carry_kind,
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        source = emit_add(
            body,
            next_reg,
            loop_scalar_kind,
            &source,
            &rhs,
            node_name,
            loop_instruction,
        )?;
    }
    if let Some(invariant_index) = invariant_index {
        let offset = expect_payload(
            payloads,
            invariant_index,
            carry_kind,
            node_name,
            loop_instruction,
        )?;
        source = emit_add(
            body,
            next_reg,
            loop_scalar_kind,
            &source,
            offset,
            node_name,
            loop_instruction,
        )?;
    }
    Ok(source)
}

fn resolve_term(
    term: &str,
    carry_kind: &str,
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    resolve_loop_carry_term(
        term,
        carry_kind,
        current,
        next_current,
        current_carries,
        next_carries,
        node_name,
        loop_instruction,
    )
}

fn expect_payload<'a>(
    payloads: &'a [String],
    index: usize,
    carry_kind: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<&'a String, String> {
    payloads.get(index).ok_or_else(|| {
        format!(
            "cpu.{loop_instruction} `{node_name}` is missing carry payload for `{carry_kind}` during LLVM lowering",
        )
    })
}

fn emit_add(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    lhs: &str,
    rhs: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    emit_loop_numeric_op(body, next_reg, loop_scalar_kind, "add", lhs, rhs).map_err(|error| {
        format!("cpu.{loop_instruction} `{node_name}` {error} during LLVM lowering")
    })
}

fn emit_mul(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    loop_scalar_kind: CpuLoopScalarKind,
    lhs: &str,
    rhs: &str,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    emit_loop_numeric_op(body, next_reg, loop_scalar_kind, "mul", lhs, rhs).map_err(|error| {
        format!("cpu.{loop_instruction} `{node_name}` {error} during LLVM lowering")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_scaled_source_uses_earlier_same_edge_carry_value() {
        let mut body = Vec::new();
        let mut next_reg = 0;
        let resolved = try_resolve_loop_carry_scaled_source(
            &mut body,
            &mut next_reg,
            CpuLoopScalarKind::I64,
            "mul_scaled_by_carry0_current_plus_invariant",
            &["%offset".to_owned()],
            "%current",
            "%next_current",
            &["%old_score".to_owned()],
            &["%new_score".to_owned()],
            "loop_node",
            "loop_while_i64_effect_flow",
        )
        .expect("scaled source should lower")
        .expect("scaled source should be recognized");

        assert_eq!(resolved.1, "mul");
        assert!(body.iter().any(|line| line.contains("%new_score")));
        assert!(!body.iter().any(|line| line.contains("%old_score")));
    }
}
