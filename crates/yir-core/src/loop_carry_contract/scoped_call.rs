use super::ScopedRecordInput;

/// A scoped helper without a backedge result. Owned captures retain their
/// existing explicit copy/move protocol; records contain scalar values only.
pub struct ScopedCall<'a> {
    pub callee: &'a str,
    pub operands: &'a [String],
}

impl ScopedCall<'_> {
    pub fn dependencies(&self) -> Result<Vec<String>, String> {
        dependencies(self.operands)
    }
}

pub fn parse_scoped_call(args: &[String]) -> Result<Option<ScopedCall<'_>>, String> {
    if args.get(6).map(String::as_str) != Some("scoped_call") {
        return Ok(None);
    }
    let invalid =
        || "invalid scoped_call payload: expected cpu action and named captures".to_owned();
    let arity = args.get(7).and_then(|arg| arg.parse::<usize>().ok());
    if args.get(5).map(String::as_str) != Some("cpu")
        || arity.is_none_or(|arity| arity == 0 || arity.checked_add(8) != Some(args.len()))
        || !named(&args[8])
    {
        return Err(invalid());
    }
    for operand in &args[9..] {
        if !readonly_record(operand)?
            && operand != "$current"
            && !named(
                operand
                    .strip_prefix("copy_owned:")
                    .or_else(|| operand.strip_prefix("move_owned:"))
                    .unwrap_or(operand),
            )
        {
            return Err(invalid());
        }
    }
    Ok(Some(ScopedCall {
        callee: &args[8],
        operands: &args[9..],
    }))
}

pub(super) fn named(value: &str) -> bool {
    !value.is_empty() && !value.contains(['$', ':', '|']) && !value.chars().any(char::is_whitespace)
}

pub(super) fn readonly_record(input: &str) -> Result<bool, String> {
    let Some(record) = ScopedRecordInput::parse(input)? else {
        return Ok(false);
    };
    // Induction is already typed i64 by the descriptor. Backedges require the
    // multi-carry seed map and cannot be smuggled through a read-only capture.
    if record
        .operands
        .iter()
        .any(|leaf| *leaf != "$current" && !named(leaf))
    {
        return Err("scoped read-only record cannot contain carry mappings".into());
    }
    Ok(true)
}

pub(super) fn dependencies(operands: &[String]) -> Result<Vec<String>, String> {
    let mut inputs = Vec::new();
    for operand in operands {
        for leaf in super::scoped_input_leaves(operand)? {
            if leaf != "$current" && leaf != "$carry" {
                inputs.push(
                    leaf.strip_prefix("copy_owned:")
                        .or_else(|| leaf.strip_prefix("move_owned:"))
                        .unwrap_or(leaf)
                        .to_owned(),
                );
            }
        }
    }
    Ok(inputs)
}
