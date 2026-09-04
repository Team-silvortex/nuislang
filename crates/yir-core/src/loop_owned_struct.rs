pub const LOOP_OWNED_STRUCT_CARRY_PREFIX: &str = "$owned_struct_carry:";

pub fn encode_loop_owned_struct_carry(index: usize, input: &str) -> String {
    format!("{LOOP_OWNED_STRUCT_CARRY_PREFIX}{index}:{input}")
}

pub fn parse_loop_owned_struct_carry(encoded: &str) -> Result<Option<(usize, &str)>, String> {
    let Some(payload) = encoded.strip_prefix(LOOP_OWNED_STRUCT_CARRY_PREFIX) else {
        return Ok(None);
    };
    let (index, input) = payload.split_once(':').ok_or_else(|| {
        format!("owned struct loop carry `{encoded}` is missing its input binding")
    })?;
    let index = index.parse::<usize>().map_err(|_| {
        format!("owned struct loop carry `{encoded}` has invalid leaf index `{index}`")
    })?;
    if input.is_empty() {
        return Err(format!(
            "owned struct loop carry `{encoded}` has an empty input binding"
        ));
    }
    Ok(Some((index, input)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_owned_struct_loop_carry_operands() {
        let encoded = encode_loop_owned_struct_carry(3, "state_field_7");
        assert_eq!(
            parse_loop_owned_struct_carry(&encoded).unwrap(),
            Some((3, "state_field_7"))
        );
        assert_eq!(parse_loop_owned_struct_carry("plain").unwrap(), None);
    }

    #[test]
    fn rejects_malformed_owned_struct_loop_carry_operands() {
        assert!(parse_loop_owned_struct_carry("$owned_struct_carry:x:value").is_err());
        assert!(parse_loop_owned_struct_carry("$owned_struct_carry:0:").is_err());
    }
}
