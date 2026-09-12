use yir_core::Node;

/// Native subset of the existing scoped-call payload, not a new loop contract.
pub(crate) struct ScopedCall<'a> {
    pub callee: &'a str,
    pub operands: &'a [String],
    pub initial: Option<&'a str>,
}

pub(crate) fn parse(node: &Node) -> Result<Option<ScopedCall<'_>>, String> {
    if node.op.instruction != "loop_while_i64_effect" {
        return Ok(None);
    }
    let fail = || {
        format!(
            "native scalar loop `{}` does not admit this scoped action",
            node.name
        )
    };
    let args = &node.op.args;
    if args.len() < 9 || args[5] != "cpu" {
        return Err(fail());
    }
    if let Some(carry) = yir_core::loop_carry_contract::parse_scoped_i64_carry(args)? {
        return Ok(Some(ScopedCall {
            callee: carry.callee,
            operands: carry.operands,
            initial: Some(carry.initial),
        }));
    }
    if args[6] != "scoped_call"
        || args[7]
            .parse::<usize>()
            .ok()
            .is_none_or(|arity| arity == 0 || arity.checked_add(8) != Some(args.len()))
    {
        return Err(fail());
    }
    let named = |value: &str| {
        !value.is_empty() && !value.contains(['$', ':']) && !value.chars().any(char::is_whitespace)
    };
    if !named(&args[8]) || args[9..].iter().any(|arg| arg != "$current" && !named(arg)) {
        return Err(fail());
    }
    Ok(Some(ScopedCall {
        callee: &args[8],
        operands: &args[9..],
        initial: None,
    }))
}
