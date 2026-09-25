use crate::{
    parse_loop_owned_struct_carry, parse_owned_struct_layout, OwnedStructFieldLayout,
    OwnedStructLayout, OwnedStructScalarLayout,
};

pub const SCOPED_I64_SEEDS_MARKER: &str = "$carry_seeds";

pub fn encode_scoped_i64_seeds(seeds: &[String]) -> Vec<String> {
    let mut encoded = vec![SCOPED_I64_SEEDS_MARKER.to_owned(), seeds.len().to_string()];
    encoded.extend_from_slice(seeds);
    encoded
}

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

impl ScopedI64Carries<'_> {
    pub fn dependencies(&self) -> Result<Vec<String>, String> {
        let mut inputs = Vec::new();
        for operand in self.operands {
            if operand != "$current" {
                inputs.push(
                    parse_loop_owned_struct_carry(operand)?
                        .map_or(operand.as_str(), |(_, seed)| seed)
                        .to_owned(),
                );
            }
        }
        for seed in &self.seeds {
            if !inputs.iter().any(|input| input == seed) {
                inputs.push((*seed).to_owned());
            }
        }
        Ok(inputs)
    }
}

pub fn parse_scoped_i64_carries(args: &[String]) -> Result<Option<ScopedI64Carries<'_>>, String> {
    let break_on_return = args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break");
    if !break_on_return && args.get(6).map(String::as_str) != Some("scoped_call_i64_carries") {
        return Ok(None);
    }
    let invalid = || {
        "invalid scoped_call_i64_carries payload: expected flat carryN:i64 layout with complete initial slots and unique, seed-matched carry operands".to_owned()
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
    let mut operands = &args[10..];
    let explicit = operands.first().map(String::as_str) == Some(SCOPED_I64_SEEDS_MARKER);
    let mut seeds = vec![None; layout.fields.len()];
    if explicit {
        let count = operands
            .get(1)
            .and_then(|count| count.parse::<usize>().ok())
            .ok_or_else(invalid)?;
        let end = count.checked_add(2).ok_or_else(invalid)?;
        if count != seeds.len() || end > operands.len() {
            return Err(invalid());
        }
        for (slot, input) in seeds.iter_mut().zip(&operands[2..end]) {
            if !named(input) {
                return Err(invalid());
            }
            *slot = Some(input.as_str());
        }
        operands = &operands[end..];
    }
    let mut mapped = vec![false; seeds.len()];
    for operand in operands {
        if let Some((index, input)) = parse_loop_owned_struct_carry(operand)? {
            let slot = seeds.get_mut(index).ok_or_else(invalid)?;
            if !named(input)
                || std::mem::replace(&mut mapped[index], true)
                || (explicit && *slot != Some(input))
            {
                return Err(invalid());
            }
            if !explicit {
                *slot = Some(input);
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

    #[test]
    fn explicit_seeds_are_independent_of_iteration_operands() {
        for action in ["scoped_call_i64_carries", "scoped_call_i64_carries_break"] {
            let mut args = format!("begin end step lt add cpu {action} 8 update State{{carry0:i64;carry1:i64}} $carry_seeds 2 first second $current $owned_struct_carry:0:first")
                .split_whitespace().map(str::to_owned).collect::<Vec<_>>();
            let parsed = parse_scoped_i64_carries(&args).unwrap().unwrap();
            assert_eq!(parsed.seeds, ["first", "second"]);
            assert_eq!(parsed.operands, ["$current", "$owned_struct_carry:0:first"]);
            assert_eq!(parsed.dependencies().unwrap(), ["first", "second"]);
            for (index, value) in [
                (11, "0"),
                (11, "1"),
                (11, "3"),
                (11, "18446744073709551615"),
                (12, "$current"),
                (13, "copy_owned:seed"),
                (15, "$owned_struct_carry:0:second"),
                (15, "$owned_struct_carry:2:first"),
                (15, "$carry_seeds"),
            ] {
                let mut invalid = args.clone();
                invalid[index] = value.into();
                assert!(parse_scoped_i64_carries(&invalid).is_err(), "{invalid:?}");
            }
            args.truncate(14);
            args[7] = "6".into();
            assert!(parse_scoped_i64_carries(&args)
                .unwrap()
                .unwrap()
                .operands
                .is_empty());
        }
        assert_eq!(
            encode_scoped_i64_seeds(&["a".into(), "b".into()]),
            ["$carry_seeds", "2", "a", "b"]
        );
    }

    #[test]
    fn explicit_seed_dependencies_and_glm_exclude_transport_metadata() {
        let args = "begin end step lt add cpu scoped_call_i64_carries 8 update State{carry0:i64;carry1:i64} $carry_seeds 2 first second $current $owned_struct_carry:0:first"
            .split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        let profile = crate::glm_profile_for_operation(
            &crate::Operation::parse("cpu.loop_while_i64_effect", args.clone()).unwrap(),
        );
        assert_eq!(
            profile
                .accesses
                .iter()
                .map(|a| a.input.as_str())
                .collect::<Vec<_>>(),
            ["begin", "end", "step", "first", "second"]
        );
        let mut duplicate = args.clone();
        duplicate.push(duplicate[15].clone());
        duplicate[7] = "9".into();
        assert!(parse_scoped_i64_carries(&duplicate).is_err());
        for len in 10..14 {
            let mut short = args[..len].to_vec();
            short[7] = (len - 8).to_string();
            assert!(parse_scoped_i64_carries(&short).is_err());
        }
    }
}
