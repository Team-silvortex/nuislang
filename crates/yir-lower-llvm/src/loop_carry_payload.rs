pub(crate) fn loop_carry_payload_len(kind: &str) -> usize {
    let carry_state_fragment_is_valid = |fragment: &str| -> bool {
        fragment == "current"
            || fragment == "prev_current"
            || fragment
                .strip_prefix("prev_carry")
                .is_some_and(|index| index.parse::<usize>().is_ok())
            || fragment
                .strip_prefix("carry")
                .is_some_and(|index| index.parse::<usize>().is_ok())
    };
    let add_state_list_payload_len = |kind: &str| -> Option<usize> {
        let (terms_part, payload_len) = if let Some(prefix) = kind.strip_prefix("add_") {
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
    if kind.contains("_plus_factor_invariant_") && kind.starts_with("mul_scaled_by_") {
        1 + usize::from(kind.ends_with("_plus_invariant"))
    } else if kind.starts_with("mul_scaled_by_") {
        usize::from(kind.ends_with("_plus_invariant"))
    } else if kind.starts_with("mul_scaled_") {
        1 + usize::from(kind.ends_with("_plus_invariant"))
    } else if kind.ends_with("_plus_invariant")
        && kind.starts_with("mul_")
        && !matches!(
            kind,
            "mul_current_plus_invariant"
                | "mul_prev_current_plus_invariant"
                | "mul_invariant"
                | "mul_source_plus_invariant"
        )
    {
        1
    } else if kind.starts_with("mul_")
        && kind.contains("_plus_")
        && !matches!(
            kind,
            "mul_current"
                | "mul_prev_current"
                | "mul_read_value_fixed"
                | "mul_read_at_fixed"
                | "mul_read_at_dynamic_current"
                | "mul_read_at_dynamic_prev_current"
        )
    {
        usize::from(kind.ends_with("_plus_invariant"))
    } else if matches!(
        kind,
        "mul_current_plus_invariant"
            | "mul_prev_current_plus_invariant"
            | "mul_invariant"
            | "mul_source_plus_invariant"
    ) {
        1
    } else if matches!(
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
        0
    } else if one_payload_indexed_prefixes.iter().any(|prefix| {
        kind.strip_prefix(prefix)
            .is_some_and(|index| index.parse::<usize>().is_ok())
    }) {
        1
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
        1
    } else if let Some(payload_len) = add_state_list_payload_len(kind) {
        payload_len
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
        1
    } else if matches!(
        kind,
        "add_read_at_fixed"
            | "mul_read_at_fixed"
            | "add_read_at_fixed_plus_invariant"
            | "mul_read_at_fixed_plus_invariant"
    ) {
        if kind.ends_with("_plus_invariant") {
            3
        } else {
            2
        }
    } else if matches!(
        kind,
        "add_read_at_dynamic_current_plus_invariant"
            | "add_read_at_dynamic_prev_current_plus_invariant"
            | "mul_read_at_dynamic_current_plus_invariant"
            | "mul_read_at_dynamic_prev_current_plus_invariant"
    ) {
        2
    } else if matches!(
        kind,
        "add_read_at_dynamic_current"
            | "add_read_at_dynamic_prev_current"
            | "mul_read_at_dynamic_current"
            | "mul_read_at_dynamic_prev_current"
            | "add_source_plus_invariant"
            | "mul_source_plus_invariant"
    ) {
        1
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
        1
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
        2
    } else {
        0
    }
}
