/// One i64 helper result carried between iterations, with named or record captures.
/// The enclosing loop returns `LoopState { current: i64, carry0: i64 }`.
#[derive(Debug)]
pub struct ScopedI64Carry<'a> {
    pub callee: &'a str,
    pub initial: &'a str,
    pub operands: &'a [String],
}

impl ScopedI64Carry<'_> {
    pub fn dependencies(&self) -> Result<Vec<String>, String> {
        let mut inputs = vec![self.initial.to_owned()];
        inputs.extend(super::scoped_call::dependencies(self.operands)?);
        Ok(inputs)
    }
}

pub fn parse_scoped_i64_carry(args: &[String]) -> Result<Option<ScopedI64Carry<'_>>, String> {
    if args.get(6).map(String::as_str) != Some("scoped_call_i64_carry") {
        return Ok(None);
    }
    let invalid = || {
        "invalid scoped_call_i64_carry payload: expected cpu action, callee, named i64 seed and exactly one $carry operand".to_owned()
    };
    let arity = args.get(7).and_then(|value| value.parse::<usize>().ok());
    if args.get(5).map(String::as_str) != Some("cpu")
        || arity.is_none_or(|arity| arity < 3 || arity.checked_add(8) != Some(args.len()))
    {
        return Err(invalid());
    }
    use super::scoped_call::{named, readonly_record};
    let operands = &args[10..];
    if !named(&args[8])
        || !named(&args[9])
        || operands
            .iter()
            .filter(|arg| arg.as_str() == "$carry")
            .count()
            != 1
    {
        return Err(invalid());
    }
    for arg in operands {
        if !readonly_record(arg)? && arg != "$current" && arg != "$carry" && !named(arg) {
            return Err(invalid());
        }
    }
    Ok(Some(ScopedI64Carry {
        callee: &args[8],
        initial: &args[9],
        operands,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> Vec<String> {
        "begin end step lt add cpu scoped_call_i64_carry 5 update seed $current $carry buffer"
            .split_whitespace()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn scoped_scalar_contract_is_explicit_and_checked() {
        let valid = args();
        let carry = parse_scoped_i64_carry(&valid).unwrap().unwrap();
        assert_eq!(carry.callee, "update");
        assert_eq!(carry.initial, "seed");
        assert_eq!(carry.operands, ["$current", "$carry", "buffer"]);
        for length in 7..valid.len() {
            assert!(parse_scoped_i64_carry(&valid[..length]).is_err());
        }
        for (index, value) in [
            (5, "shader"),
            (7, "0"),
            (7, "18446744073709551615"),
            (8, ""),
            (9, "$carry"),
            (9, "copy_owned:seed"),
            (11, "seed"),
            (12, "$carry"),
            (12, "$unknown"),
            (12, "copy_owned:buffer"),
            (12, "move_owned:buffer"),
            (12, "carry:0:seed"),
        ] {
            let mut invalid = valid.clone();
            invalid[index] = value.to_owned();
            assert!(parse_scoped_i64_carry(&invalid).is_err(), "{invalid:?}");
        }
        assert!(parse_scoped_i64_carry(&[]).unwrap().is_none());
    }
}
