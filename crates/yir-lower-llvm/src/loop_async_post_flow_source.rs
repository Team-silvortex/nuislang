use crate::emit_utils::fresh_reg;

fn resolve_state_term_for_async_post_flow(
    term: &str,
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    match term {
        "current" => Ok(next_current.to_owned()),
        "prev_current" => Ok(current.to_owned()),
        other if other.starts_with("prev_carry") => {
            let i = other[10..].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering"))?;
            current_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry term `{other}` during LLVM lowering"))
        }
        other if other.starts_with("carry") => {
            let i = other[5..].parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering"))?;
            next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry term `{other}` during LLVM lowering"))
        }
        other => Err(format!("cpu.{loop_instruction} `{node_name}` has unsupported carry term `{other}` during LLVM lowering")),
    }
}
pub(crate) fn resolve_source_for_async_post_flow(
    source_spec: &[String],
    current: &str,
    next_current: &str,
    current_carries: &[String],
    next_carries: &[String],
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    loop_instruction: &str,
) -> Result<String, String> {
    let Some(kind) = source_spec.first() else {
        return Err(format!(
            "cpu.{loop_instruction} `{node_name}` has empty carry source during LLVM lowering"
        ));
    };
    if matches!(kind.as_str(), "keep" | "keep_prev_carry") {
        return Ok(String::new());
    }
    if kind == "add_current" {
        return Ok(next_current.to_owned());
    }
    if let Some(rest) = kind.strip_prefix("add_carry") {
        let i = rest.parse::<usize>().map_err(|_| format!("cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{kind}` during LLVM lowering"))?;
        return next_carries.get(i).cloned().ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` references unavailable carry source `{kind}` during LLVM lowering"));
    }
    let parse_factor_group = |group: &str| -> Option<(Vec<String>, bool)> {
        if let Some(group) = group.strip_suffix("_plus_factor_invariant") {
            let terms = group.split("_plus_").map(str::to_owned).collect::<Vec<_>>();
            if terms.is_empty()
                || !terms.iter().all(|term| {
                    matches!(term.as_str(), "current" | "prev_current")
                        || term.starts_with("prev_carry")
                        || term.starts_with("carry")
                })
            {
                return None;
            }
            Some((terms, true))
        } else {
            let terms = group.split("_plus_").map(str::to_owned).collect::<Vec<_>>();
            if terms.is_empty()
                || !terms.iter().all(|term| {
                    matches!(term.as_str(), "current" | "prev_current")
                        || term.starts_with("prev_carry")
                        || term.starts_with("carry")
                })
            {
                return None;
            }
            Some((terms, false))
        }
    };
    if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
        if let Some((lhs_group, rest)) = prefix.split_once("_times_factor_group_") {
            let (lhs_terms, lhs_has_offset) = parse_factor_group(lhs_group)
                .ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` has unsupported factor group in `{kind}` during LLVM lowering"))?;
            let (rhs_group, terms_part, has_factor_scale) = if let Some((rhs_group, terms_part)) =
                rest.split_once("_times_factor_invariant_times_terms_")
            {
                (rhs_group, terms_part, true)
            } else if let Some((rhs_group, terms_part)) = rest.split_once("_times_terms_") {
                (rhs_group, terms_part, false)
            } else {
                return Err(format!(
                        "cpu.{loop_instruction} `{node_name}` has malformed factor-group carry kind `{kind}` during LLVM lowering"
                    ));
            };
            let (rhs_terms, rhs_has_offset) = parse_factor_group(rhs_group)
                .ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` has unsupported factor group in `{kind}` during LLVM lowering"))?;
            let (terms_part, has_invariant) =
                if let Some(terms_part) = terms_part.strip_suffix("_plus_invariant") {
                    (terms_part, true)
                } else {
                    (terms_part, false)
                };
            let terms = terms_part
                .split("_plus_")
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let mut payload_index = 1usize;
            let resolve_group = |group_terms: &[String],
                                 has_offset: bool,
                                 payload_index: &mut usize,
                                 body: &mut Vec<String>,
                                 next_reg: &mut usize|
             -> Result<String, String> {
                let mut acc = resolve_state_term_for_async_post_flow(
                    &group_terms[0],
                    current,
                    next_current,
                    current_carries,
                    next_carries,
                    node_name,
                    loop_instruction,
                )?;
                for term in group_terms.iter().skip(1) {
                    let rhs = resolve_state_term_for_async_post_flow(
                        term,
                        current,
                        next_current,
                        current_carries,
                        next_carries,
                        node_name,
                        loop_instruction,
                    )?;
                    let sum = fresh_reg(next_reg);
                    body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                    acc = sum;
                }
                if has_offset {
                    let offset_name = source_spec.get(*payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor-group offset payload for `{kind}` during LLVM lowering"))?;
                    *payload_index += 1;
                    let sum = fresh_reg(next_reg);
                    body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                    acc = sum;
                }
                Ok(acc)
            };
            let lhs = resolve_group(
                &lhs_terms,
                lhs_has_offset,
                &mut payload_index,
                body,
                next_reg,
            )?;
            let rhs = resolve_group(
                &rhs_terms,
                rhs_has_offset,
                &mut payload_index,
                body,
                next_reg,
            )?;
            let mut factor = fresh_reg(next_reg);
            body.push(format!("  {factor} = mul i64 {lhs}, {rhs}"));
            if has_factor_scale {
                let factor_scale_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor-scale payload for `{kind}` during LLVM lowering"))?;
                payload_index += 1;
                let scaled_factor = fresh_reg(next_reg);
                body.push(format!(
                    "  {scaled_factor} = mul i64 {factor}, {factor_scale_name}"
                ));
                factor = scaled_factor;
            }
            let mut acc = resolve_state_term_for_async_post_flow(
                &terms[0],
                current,
                next_current,
                current_carries,
                next_carries,
                node_name,
                loop_instruction,
            )?;
            for term in terms.iter().skip(1) {
                let rhs = resolve_state_term_for_async_post_flow(
                    term,
                    current,
                    next_current,
                    current_carries,
                    next_carries,
                    node_name,
                    loop_instruction,
                )?;
                let sum = fresh_reg(next_reg);
                body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
                acc = sum;
            }
            if has_invariant {
                let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
                let sum = fresh_reg(next_reg);
                body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
                acc = sum;
            }
            let scaled = fresh_reg(next_reg);
            body.push(format!("  {scaled} = mul i64 {acc}, {factor}"));
            return Ok(scaled);
        }
    }
    let parse_add_terms =
        |kind: &str| -> Option<(Option<Vec<String>>, bool, bool, bool, Vec<String>, bool)> {
            let carry_state_fragment_is_valid = |fragment: &str| -> bool {
                matches!(fragment, "current" | "prev_current")
                    || fragment.starts_with("prev_carry")
                    || fragment.starts_with("carry")
            };
            let (
                factor_term,
                scaled_by_payload,
                factor_invariant_payload,
                factor_scale_payload,
                terms_part,
                has_invariant,
            ) = if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
                let (prefix, has_invariant) =
                    if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                        (prefix, true)
                    } else {
                        (prefix, false)
                    };
                if let Some((factor_terms, terms_part)) =
                    prefix.split_once("_plus_factor_invariant_times_factor_invariant_times_")
                {
                    (
                        Some(
                            factor_terms
                                .split("_plus_")
                                .map(str::to_owned)
                                .collect::<Vec<_>>(),
                        ),
                        false,
                        true,
                        true,
                        terms_part,
                        has_invariant,
                    )
                } else if let Some((factor_terms, terms_part)) =
                    prefix.split_once("_times_factor_invariant_times_")
                {
                    (
                        Some(
                            factor_terms
                                .split("_plus_")
                                .map(str::to_owned)
                                .collect::<Vec<_>>(),
                        ),
                        false,
                        false,
                        true,
                        terms_part,
                        has_invariant,
                    )
                } else if let Some((factor_terms, terms_part)) =
                    prefix.split_once("_plus_factor_invariant_times_")
                {
                    (
                        Some(
                            factor_terms
                                .split("_plus_")
                                .map(str::to_owned)
                                .collect::<Vec<_>>(),
                        ),
                        false,
                        true,
                        false,
                        terms_part,
                        has_invariant,
                    )
                } else if let Some((factor_terms, terms_part)) = prefix.split_once("_times_") {
                    (
                        Some(
                            factor_terms
                                .split("_plus_")
                                .map(str::to_owned)
                                .collect::<Vec<_>>(),
                        ),
                        false,
                        false,
                        false,
                        terms_part,
                        has_invariant,
                    )
                } else {
                    let (factor_term, factor_invariant_payload, terms_part) =
                        if let Some((factor_term, terms_part)) =
                            prefix.split_once("_plus_factor_invariant_")
                        {
                            (Some(vec![factor_term.to_owned()]), true, terms_part)
                        } else {
                            let (factor_term, terms_part) = prefix.split_once('_')?;
                            (Some(vec![factor_term.to_owned()]), false, terms_part)
                        };
                    (
                        factor_term,
                        false,
                        factor_invariant_payload,
                        false,
                        terms_part,
                        has_invariant,
                    )
                }
            } else if let Some(prefix) = kind.strip_prefix("add_scaled_") {
                if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                    (None, true, false, false, prefix, true)
                } else {
                    (None, true, false, false, prefix, false)
                }
            } else if let Some(prefix) = kind.strip_prefix("add_") {
                if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                    (None, false, false, false, prefix, true)
                } else {
                    (None, false, false, false, prefix, false)
                }
            } else {
                return None;
            };
            let terms = terms_part
                .split("_plus_")
                .map(|term| term.to_owned())
                .collect::<Vec<_>>();
            if terms.iter().all(|term| {
                matches!(term.as_str(), "current" | "prev_current")
                    || term.starts_with("prev_carry")
                    || term.starts_with("carry")
            }) {
                if let Some(factor_terms) = factor_term.as_ref() {
                    if factor_terms.is_empty()
                        || !factor_terms
                            .iter()
                            .all(|term| carry_state_fragment_is_valid(term))
                    {
                        return None;
                    }
                }
                Some((
                    factor_term,
                    scaled_by_payload,
                    factor_invariant_payload,
                    factor_scale_payload,
                    terms,
                    has_invariant,
                ))
            } else {
                None
            }
        };
    if let Some((
        factor_term,
        scaled_by_payload,
        factor_invariant_payload,
        factor_scale_payload,
        terms,
        has_invariant,
    )) = parse_add_terms(kind)
    {
        let mut payload_index = 1usize;
        let factor = if let Some(factor_terms) = factor_term {
            let mut factor = resolve_state_term_for_async_post_flow(
                &factor_terms[0],
                current,
                next_current,
                current_carries,
                next_carries,
                node_name,
                loop_instruction,
            )?;
            for factor_term in factor_terms.iter().skip(1) {
                let rhs = resolve_state_term_for_async_post_flow(
                    factor_term,
                    current,
                    next_current,
                    current_carries,
                    next_carries,
                    node_name,
                    loop_instruction,
                )?;
                let sum = fresh_reg(next_reg);
                body.push(format!("  {sum} = add i64 {factor}, {rhs}"));
                factor = sum;
            }
            if factor_scale_payload {
                let factor_scale_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor scale payload for `{kind}` during LLVM lowering"))?;
                payload_index += 1;
                let scaled = fresh_reg(next_reg);
                body.push(format!(
                    "  {scaled} = mul i64 {factor}, {factor_scale_name}"
                ));
                factor = scaled;
            }
            if factor_invariant_payload {
                let factor_offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing factor invariant payload for `{kind}` during LLVM lowering"))?;
                payload_index += 1;
                let sum = fresh_reg(next_reg);
                body.push(format!("  {sum} = add i64 {factor}, {factor_offset_name}"));
                factor = sum;
            }
            Some(factor)
        } else if scaled_by_payload {
            let factor_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing scaled carry factor for `{kind}` during LLVM lowering"))?;
            payload_index += 1;
            Some(factor_name.clone())
        } else {
            None
        };
        let mut acc = resolve_state_term_for_async_post_flow(
            &terms[0],
            current,
            next_current,
            current_carries,
            next_carries,
            node_name,
            loop_instruction,
        )?;
        for term in terms.iter().skip(1) {
            let rhs = resolve_state_term_for_async_post_flow(
                term,
                current,
                next_current,
                current_carries,
                next_carries,
                node_name,
                loop_instruction,
            )?;
            let sum = fresh_reg(next_reg);
            body.push(format!("  {sum} = add i64 {acc}, {rhs}"));
            acc = sum;
        }
        if factor.is_some() && has_invariant {
            let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
            payload_index += 1;
            let sum = fresh_reg(next_reg);
            body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
            acc = sum;
        }
        if let Some(factor) = factor {
            let scaled_reg = fresh_reg(next_reg);
            body.push(format!("  {scaled_reg} = mul i64 {acc}, {factor}"));
            acc = scaled_reg;
        } else if has_invariant {
            let offset_name = source_spec.get(payload_index).ok_or_else(|| format!("cpu.{loop_instruction} `{node_name}` is missing invariant payload for `{kind}` during LLVM lowering"))?;
            let sum = fresh_reg(next_reg);
            body.push(format!("  {sum} = add i64 {acc}, {offset_name}"));
            acc = sum;
        }
        return Ok(acc);
    }
    Err(format!("cpu.{loop_instruction} `{node_name}` has unsupported carry kind `{kind}` during LLVM lowering"))
}
