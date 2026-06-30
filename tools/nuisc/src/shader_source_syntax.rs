pub(super) fn starts_with_keyword(chars: &[char], index: usize, keyword: &str) -> bool {
    let keyword_chars = keyword.chars().collect::<Vec<_>>();
    if chars.get(index..index + keyword_chars.len()) != Some(keyword_chars.as_slice()) {
        return false;
    }
    let prev_ok =
        index == 0 || !chars[index - 1].is_ascii_alphanumeric() && chars[index - 1] != '_';
    let next = chars.get(index + keyword_chars.len()).copied();
    let next_ok = !next.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_');
    prev_ok && next_ok
}

pub(super) fn parse_stage_metadata(
    chars: &[char],
    start: usize,
    stage_kind: &str,
) -> Result<(Vec<String>, usize), String> {
    let mut index = start;
    if chars.get(index).copied() != Some('(') {
        return Ok((Vec::new(), start));
    }
    index += 1;
    while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
        index += 1;
    }

    let mut attributes = Vec::new();
    loop {
        while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
            index += 1;
        }
        let keyword_start = index;
        while chars
            .get(index)
            .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
        {
            index += 1;
        }
        if keyword_start == index {
            return Err(format!(
                "wgsl stage block `{stage_kind}` metadata is missing an attribute name"
            ));
        }
        let attribute_name = chars[keyword_start..index].iter().collect::<String>();
        let attribute = match attribute_name.as_str() {
            "workgroup_size" => {
                if stage_kind != "compute" {
                    return Err(format!(
                        "wgsl stage block `{stage_kind}` does not support workgroup_size(...) metadata"
                    ));
                }
                while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
                    index += 1;
                }
                if chars.get(index).copied() != Some('(') {
                    return Err(
                        "wgsl workgroup_size metadata must use `workgroup_size(x, y, z)`"
                            .to_owned(),
                    );
                }
                let (args, next_index) = parse_parenthesized_args(chars, index)?;
                index = next_index;
                format!("workgroup_size({args})")
            }
            "early_depth_test" => {
                if stage_kind != "fragment" {
                    return Err(format!(
                        "wgsl stage block `{stage_kind}` does not support early_depth_test metadata"
                    ));
                }
                "early_depth_test".to_owned()
            }
            _ => {
                return Err(format!(
                    "wgsl stage block `{stage_kind}` only supports workgroup_size(...) and early_depth_test metadata today"
                ));
            }
        };
        attributes.push(attribute);

        while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
            index += 1;
        }
        match chars.get(index).copied() {
            Some(',') => {
                index += 1;
            }
            Some(')') => {
                index += 1;
                return Ok((attributes, index));
            }
            _ => {
                return Err(format!(
                    "wgsl stage block `{stage_kind}` metadata must separate entries with `,` and end with `)`"
                ));
            }
        }
    }
}

pub(super) fn parse_parenthesized_args(
    chars: &[char],
    start: usize,
) -> Result<(String, usize), String> {
    let mut index = start;
    if chars.get(index).copied() != Some('(') {
        return Err("expected `(`".to_owned());
    }
    let args_start = index + 1;
    index += 1;
    let mut arg_depth = 1usize;
    while index < chars.len() {
        match chars[index] {
            '(' => {
                arg_depth += 1;
                index += 1;
            }
            ')' => {
                arg_depth = arg_depth.saturating_sub(1);
                if arg_depth == 0 {
                    let args = chars[args_start..index]
                        .iter()
                        .collect::<String>()
                        .trim()
                        .to_owned();
                    index += 1;
                    return Ok((args, index));
                }
                index += 1;
            }
            _ => index += 1,
        }
    }

    Err("unterminated wgsl metadata arguments".to_owned())
}

pub(super) fn parse_binding_metadata(
    chars: &[char],
    start: usize,
) -> Result<((String, String), usize), String> {
    let (args, next_index) = parse_parenthesized_args(chars, start)?;
    let mut parts = args
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty());
    let group = parts
        .next()
        .ok_or_else(|| "wgsl binding declaration must provide a group index".to_owned())?;
    let slot = parts
        .next()
        .ok_or_else(|| "wgsl binding declaration must provide a binding slot".to_owned())?;
    if parts.next().is_some() {
        return Err(
            "wgsl binding declaration only supports `binding(group, slot)` today".to_owned(),
        );
    }
    if !group.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!(
            "wgsl binding declaration group `{group}` must be an integer literal"
        ));
    }
    if !slot.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(format!(
            "wgsl binding declaration slot `{slot}` must be an integer literal"
        ));
    }
    Ok(((group.to_owned(), slot.to_owned()), next_index))
}

pub(super) fn starts_with_bare_attribute_keyword(chars: &[char], index: usize) -> bool {
    ["builtin", "location", "interpolate"]
        .iter()
        .any(|keyword| {
            starts_with_keyword(chars, index, keyword)
                && !previous_significant_char(chars, index).is_some_and(|ch| ch == '@')
        })
}

pub(super) fn is_attribute_position(chars: &[char], index: usize) -> bool {
    match previous_non_whitespace_index(chars, index) {
        None => true,
        Some(prev) => match chars[prev] {
            '{' | '(' | ',' => true,
            ')' => previous_parenthesized_attribute_name(chars, prev)
                .as_deref()
                .is_some_and(is_supported_bare_attribute_keyword),
            '>' => previous_non_whitespace_index(chars, prev)
                .is_some_and(|before_arrow| chars[before_arrow] == '-'),
            ch if ch.is_ascii_alphabetic() || ch == '_' => previous_identifier(chars, prev)
                .as_deref()
                .is_some_and(is_supported_bare_attribute_keyword),
            _ => false,
        },
    }
}

fn is_supported_bare_attribute_keyword(keyword: &str) -> bool {
    matches!(
        keyword,
        "builtin" | "location" | "interpolate" | "invariant"
    )
}

fn previous_significant_char(chars: &[char], index: usize) -> Option<char> {
    previous_non_whitespace_index(chars, index).map(|pos| chars[pos])
}

fn previous_non_whitespace_index(chars: &[char], index: usize) -> Option<usize> {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        if !chars[cursor].is_whitespace() {
            return Some(cursor);
        }
    }
    None
}

fn previous_identifier(chars: &[char], end: usize) -> Option<String> {
    if !chars
        .get(end)
        .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
    {
        return None;
    }
    let mut start = end;
    while start > 0 && (chars[start - 1].is_ascii_alphabetic() || chars[start - 1] == '_') {
        start -= 1;
    }
    Some(chars[start..=end].iter().collect::<String>())
}

fn previous_parenthesized_attribute_name(chars: &[char], close_paren: usize) -> Option<String> {
    if chars.get(close_paren).copied() != Some(')') {
        return None;
    }
    let mut depth = 1usize;
    let mut index = close_paren;
    while index > 0 {
        index -= 1;
        match chars[index] {
            ')' => depth += 1,
            '(' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let name_end = previous_non_whitespace_index(chars, index)?;
                    return previous_identifier(chars, name_end);
                }
            }
            _ => {}
        }
    }
    None
}
