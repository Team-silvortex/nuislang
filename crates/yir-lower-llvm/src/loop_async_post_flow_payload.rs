pub(crate) fn async_post_flow_carry_source_payload_len(kind: &str) -> Option<usize> {
    let carry_state_fragment_is_valid = |fragment: &str| -> bool {
        match fragment {
            "current" | "prev_current" => true,
            other => other
                .strip_prefix("prev_carry")
                .or_else(|| other.strip_prefix("carry"))
                .is_some_and(|index| index.parse::<usize>().is_ok()),
        }
    };
    let add_state_list_payload_len = |kind: &str| -> Option<usize> {
        let (terms_part, payload_len) = if let Some(prefix) = kind.strip_prefix("add_scaled_by_") {
            if let Some((lhs_group, rest)) = prefix.split_once("_times_factor_group_") {
                let parse_group = |group: &str| -> Option<bool> {
                    if let Some(group) = group.strip_suffix("_plus_factor_invariant") {
                        let terms = group.split("_plus_").collect::<Vec<_>>();
                        if terms.is_empty()
                            || !terms.iter().all(|term| carry_state_fragment_is_valid(term))
                        {
                            return None;
                        }
                        Some(true)
                    } else {
                        let terms = group.split("_plus_").collect::<Vec<_>>();
                        if terms.is_empty()
                            || !terms.iter().all(|term| carry_state_fragment_is_valid(term))
                        {
                            return None;
                        }
                        Some(false)
                    }
                };
                let lhs_offset = parse_group(lhs_group)?;
                if let Some((rhs_group, rest)) =
                    rest.split_once("_times_factor_invariant_times_terms_")
                {
                    let rhs_offset = parse_group(rhs_group)?;
                    if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                        (
                            rest,
                            usize::from(lhs_offset) + usize::from(rhs_offset) + 2usize,
                        )
                    } else {
                        (
                            rest,
                            usize::from(lhs_offset) + usize::from(rhs_offset) + 1usize,
                        )
                    }
                } else {
                    let (rhs_group, rest) = rest.split_once("_times_terms_")?;
                    let rhs_offset = parse_group(rhs_group)?;
                    if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                        (
                            rest,
                            usize::from(lhs_offset) + usize::from(rhs_offset) + 1usize,
                        )
                    } else {
                        (rest, usize::from(lhs_offset) + usize::from(rhs_offset))
                    }
                }
            } else if let Some((factor_terms, rest)) =
                prefix.split_once("_plus_factor_invariant_times_factor_invariant_times_")
            {
                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                if factor_terms.is_empty()
                    || !factor_terms
                        .iter()
                        .all(|term| carry_state_fragment_is_valid(term))
                {
                    return None;
                }
                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                    (rest, 3usize)
                } else {
                    (rest, 2usize)
                }
            } else if let Some((factor_terms, rest)) =
                prefix.split_once("_times_factor_invariant_times_")
            {
                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                if factor_terms.len() < 2
                    || !factor_terms
                        .iter()
                        .all(|term| carry_state_fragment_is_valid(term))
                {
                    return None;
                }
                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                    (rest, 2usize)
                } else {
                    (rest, 1usize)
                }
            } else if let Some((factor_terms, rest)) =
                prefix.split_once("_plus_factor_invariant_times_")
            {
                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                if factor_terms.is_empty()
                    || !factor_terms
                        .iter()
                        .all(|term| carry_state_fragment_is_valid(term))
                {
                    return None;
                }
                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                    (rest, 2usize)
                } else {
                    (rest, 1usize)
                }
            } else if let Some((factor_terms, rest)) = prefix.split_once("_times_") {
                let factor_terms = factor_terms.split("_plus_").collect::<Vec<_>>();
                if factor_terms.len() < 2
                    || !factor_terms
                        .iter()
                        .all(|term| carry_state_fragment_is_valid(term))
                {
                    return None;
                }
                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                    (rest, 1usize)
                } else {
                    (rest, 0usize)
                }
            } else if let Some((factor, rest)) = prefix.split_once("_plus_factor_invariant_") {
                if !carry_state_fragment_is_valid(factor) {
                    return None;
                }
                if let Some(rest) = rest.strip_suffix("_plus_invariant") {
                    (rest, 2usize)
                } else {
                    (rest, 1usize)
                }
            } else if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                let (_, rest) = prefix.split_once('_')?;
                (rest, 1usize)
            } else {
                let (_, rest) = prefix.split_once('_')?;
                (rest, 0usize)
            }
        } else if let Some(prefix) = kind.strip_prefix("add_scaled_") {
            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                (prefix, 2usize)
            } else {
                (prefix, 1usize)
            }
        } else if let Some(prefix) = kind.strip_prefix("add_") {
            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                (prefix, 1usize)
            } else {
                (prefix, 0usize)
            }
        } else if let Some(prefix) = kind.strip_prefix("mul_") {
            if let Some(prefix) = prefix.strip_suffix("_plus_invariant") {
                (prefix, 1usize)
            } else {
                (prefix, 0usize)
            }
        } else {
            return None;
        };
        let terms = terms_part.split("_plus_").collect::<Vec<_>>();
        if terms.len() < 2 {
            return None;
        }
        if terms.iter().all(|term| carry_state_fragment_is_valid(term)) {
            Some(payload_len)
        } else {
            None
        }
    };
    let zero_payload_indexed_prefixes =
        ["add_prev_carry", "mul_prev_carry", "add_carry", "mul_carry"];
    let one_payload_zero_payload_indexed_prefixes =
        ["add_prev_carry", "add_carry", "mul_prev_carry", "mul_carry"];
    let one_payload_indexed_prefixes = [
        "add_read_at_dynamic_prev_carry",
        "mul_read_at_dynamic_prev_carry",
        "add_read_at_dynamic_carry",
        "mul_read_at_dynamic_carry",
    ];
    if matches!(
        kind,
        "keep"
            | "keep_prev_carry"
            | "add_current"
            | "add_prev_current"
            | "mul_current"
            | "mul_prev_current"
    ) || zero_payload_indexed_prefixes.iter().any(|prefix| {
        kind.strip_prefix(prefix)
            .is_some_and(|index| index.parse::<usize>().is_ok())
    }) {
        Some(0)
    } else if one_payload_indexed_prefixes.iter().any(|prefix| {
        kind.strip_prefix(prefix)
            .is_some_and(|index| index.parse::<usize>().is_ok())
    }) {
        Some(1)
    } else if one_payload_zero_payload_indexed_prefixes
        .iter()
        .any(|prefix| {
            kind.strip_prefix(prefix).is_some_and(|suffix| {
                suffix
                    .strip_suffix("_plus_invariant")
                    .is_some_and(|index| index.parse::<usize>().is_ok())
            })
        })
    {
        Some(1)
    } else if let Some(payload_len) = add_state_list_payload_len(kind) {
        Some(payload_len)
    } else if matches!(
        kind,
        "add_read_value_fixed"
            | "mul_read_value_fixed"
            | "add_read_value_fixed_plus_invariant"
            | "mul_read_value_fixed_plus_invariant"
            | "add_invariant"
            | "add_current_plus_invariant"
            | "add_prev_current_plus_invariant"
            | "mul_invariant"
            | "mul_current_plus_invariant"
            | "mul_prev_current_plus_invariant"
    ) {
        Some(1)
    } else if matches!(
        kind,
        "add_read_at_fixed"
            | "mul_read_at_fixed"
            | "add_read_at_fixed_plus_invariant"
            | "mul_read_at_fixed_plus_invariant"
    ) {
        Some(if kind.ends_with("_plus_invariant") {
            3
        } else {
            2
        })
    } else if matches!(
        kind,
        "add_read_at_dynamic_current_plus_invariant"
            | "add_read_at_dynamic_prev_current_plus_invariant"
            | "mul_read_at_dynamic_current_plus_invariant"
            | "mul_read_at_dynamic_prev_current_plus_invariant"
    ) {
        Some(2)
    } else if matches!(
        kind,
        "add_read_at_dynamic_current"
            | "add_read_at_dynamic_prev_current"
            | "mul_read_at_dynamic_current"
            | "mul_read_at_dynamic_prev_current"
            | "add_source_plus_invariant"
            | "mul_source_plus_invariant"
    ) {
        Some(1)
    } else if [
        "add_read_at_dynamic_prev_carry",
        "mul_read_at_dynamic_prev_carry",
        "add_read_at_dynamic_carry",
        "mul_read_at_dynamic_carry",
    ]
    .iter()
    .any(|prefix| {
        kind.strip_prefix(prefix)
            .is_some_and(|index| index.parse::<usize>().is_ok())
    }) {
        Some(1)
    } else if [
        "add_read_at_dynamic_prev_carry",
        "mul_read_at_dynamic_prev_carry",
        "add_read_at_dynamic_carry",
        "mul_read_at_dynamic_carry",
    ]
    .iter()
    .any(|prefix| {
        kind.strip_prefix(prefix).is_some_and(|suffix| {
            suffix
                .strip_suffix("_plus_invariant")
                .is_some_and(|index| index.parse::<usize>().is_ok())
        })
    }) {
        Some(2)
    } else {
        None
    }
}
