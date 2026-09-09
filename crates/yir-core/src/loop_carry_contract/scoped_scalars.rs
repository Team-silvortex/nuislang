use crate::{
    parse_loop_owned_struct_carry, parse_owned_struct_layout, OwnedStructFieldLayout,
    OwnedStructLayout, OwnedStructScalarLayout,
};

/// One aggregate helper call updates all scalar slots; layout order defines projection order.
#[derive(Debug)]
pub struct ScopedI64Carries<'a> {
    pub callee: &'a str,
    pub encoded_layout: &'a str,
    pub layout: OwnedStructLayout,
    pub seeds: Vec<&'a str>,
    pub operands: &'a [String],
    /// The last i64 slot is 0 to advance or 1 to break before the induction step.
    pub break_on_return: bool,
}

pub fn parse_scoped_i64_carries(args: &[String]) -> Result<Option<ScopedI64Carries<'_>>, String> {
    let break_on_return = args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break");
    if !break_on_return && args.get(6).map(String::as_str) != Some("scoped_call_i64_carries") {
        return Ok(None);
    }
    let invalid = || {
        "invalid scoped_call_i64_carries payload: expected flat carryN:i64 layout and each named seed exactly once".to_owned()
    };
    let arity = args.get(7).and_then(|value| value.parse::<usize>().ok());
    if args.get(5).map(String::as_str) != Some("cpu")
        || arity.is_none_or(|arity| arity < 3 || arity.checked_add(8) != Some(args.len()))
    {
        return Err(invalid());
    }
    let named = |value: &str| {
        !value.is_empty() && !value.contains(['$', ':']) && !value.chars().any(char::is_whitespace)
    };
    let layout = parse_owned_struct_layout(&args[9])?;
    if !named(&args[8])
        || !named(&layout.type_name)
        || layout.fields.is_empty()
        || layout
            .fields
            .iter()
            .enumerate()
            .any(|(index, (name, kind))| {
                name != &format!("carry{index}")
                    || kind != &OwnedStructFieldLayout::Scalar(OwnedStructScalarLayout::I64)
            })
    {
        return Err(invalid());
    }
    let operands = &args[10..];
    let mut seeds = vec![None; layout.fields.len()];
    for operand in operands {
        if let Some((index, input)) = parse_loop_owned_struct_carry(operand)? {
            let slot = seeds.get_mut(index).ok_or_else(invalid)?;
            if !named(input) || slot.replace(input).is_some() {
                return Err(invalid());
            }
        } else if operand != "$current" && !named(operand) {
            return Err(invalid());
        }
    }
    let seeds = seeds
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(invalid)?;
    Ok(Some(ScopedI64Carries {
        callee: &args[8],
        encoded_layout: &args[9],
        layout,
        seeds,
        operands,
        break_on_return,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_multi_scalar_layout_and_markers_are_checked_together() {
        for action in ["scoped_call_i64_carries", "scoped_call_i64_carries_break"] {
            let valid = format!("begin end step lt add cpu {action} 6 update State{{carry0:i64;carry1:i64}} $owned_struct_carry:1:second $current buffer $owned_struct_carry:0:first")
            .split_whitespace().map(str::to_owned).collect::<Vec<_>>();
            let parsed = parse_scoped_i64_carries(&valid).unwrap().unwrap();
            assert_eq!(parsed.seeds, ["first", "second"]);
            assert_eq!(parsed.break_on_return, action.ends_with("_break"));
            for length in 7..valid.len() {
                assert!(parse_scoped_i64_carries(&valid[..length]).is_err());
            }
            for (index, value) in [
                (5, "shader"),
                (7, "18446744073709551615"),
                (8, "$current"),
                (9, "State{}"),
                (9, "State{carry0:i64;carry1:bool}"),
                (9, "State{carry0:i64;carry0:i64}"),
                (9, "State{carry0:Inner{x:i64}}"),
                (10, "$owned_struct_carry:0:second"),
                (10, "$owned_struct_carry:2:second"),
                (10, "$owned_struct_carry:18446744073709551615:second"),
                (10, "$owned_struct_carry:1:$current"),
                (10, "second"),
                (12, "copy_owned:buffer"),
                (12, "move_owned:buffer"),
                (12, "$carry"),
            ] {
                let mut invalid = valid.clone();
                invalid[index] = value.to_owned();
                assert!(parse_scoped_i64_carries(&invalid).is_err(), "{invalid:?}");
            }
        }
        assert!(parse_scoped_i64_carries(&[]).unwrap().is_none());
    }

    #[test]
    fn break_control_can_be_the_only_carried_slot() {
        let args = "begin end step lt add cpu scoped_call_i64_carries_break 4 update Control{carry0:i64} $current $owned_struct_carry:0:zero"
            .split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        let parsed = parse_scoped_i64_carries(&args).unwrap().unwrap();
        assert!(parsed.break_on_return);
        assert_eq!(parsed.seeds, ["zero"]);
    }
}
