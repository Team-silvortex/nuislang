pub(crate) fn normalize_inline_wgsl_source(source: &str) -> Result<String, String> {
    let chars = source.chars().collect::<Vec<_>>();
    let mut out = String::new();
    let mut index = 0usize;
    let mut transformed = false;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0usize;

    while index < chars.len() {
        let ch = chars[index];

        if in_line_comment {
            out.push(ch);
            if ch == '\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }

        if block_comment_depth > 0 {
            out.push(ch);
            match ch {
                '/' if chars.get(index + 1).copied() == Some('*') => {
                    out.push('*');
                    index += 2;
                    block_comment_depth += 1;
                    continue;
                }
                '*' if chars.get(index + 1).copied() == Some('/') => {
                    out.push('/');
                    index += 2;
                    block_comment_depth = block_comment_depth.saturating_sub(1);
                    continue;
                }
                _ => {
                    index += 1;
                    continue;
                }
            }
        }

        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else {
                match ch {
                    '\\' => escape = true,
                    '"' => in_string = false,
                    _ => {}
                }
            }
            index += 1;
            continue;
        }

        if depth == 0 && starts_with_keyword(&chars, index, "stage") {
            let original = index;
            index += "stage".chars().count();
            while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
                index += 1;
            }

            let stage_start = index;
            while chars
                .get(index)
                .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
            {
                index += 1;
            }
            if stage_start == index {
                return Err("wgsl stage block is missing stage kind".to_owned());
            }
            let stage_kind = chars[stage_start..index].iter().collect::<String>();
            if !matches!(stage_kind.as_str(), "vertex" | "fragment" | "compute") {
                return Err(format!(
                    "wgsl stage block `{stage_kind}` is unsupported; expected `vertex`, `fragment`, or `compute`"
                ));
            }

            let mut stage_attributes = Vec::new();
            while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
                index += 1;
            }
            if chars.get(index).copied() == Some('(') {
                let (attributes, next_index) = parse_stage_metadata(&chars, index, &stage_kind)?;
                stage_attributes = attributes;
                index = next_index;
                while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
                    index += 1;
                }
            }
            if chars.get(index).copied() != Some('{') {
                return Err(format!(
                    "wgsl stage block `{stage_kind}` must start with `{{`"
                ));
            }
            index += 1;
            let body_start = index;
            let mut local_depth = 1usize;
            let mut local_in_string = false;
            let mut local_escape = false;
            let mut local_in_line_comment = false;
            let mut local_block_comment_depth = 0usize;
            while index < chars.len() {
                let current = chars[index];
                if local_in_line_comment {
                    if current == '\n' {
                        local_in_line_comment = false;
                    }
                    index += 1;
                    continue;
                }
                if local_block_comment_depth > 0 {
                    match current {
                        '/' if chars.get(index + 1).copied() == Some('*') => {
                            local_block_comment_depth += 1;
                            index += 2;
                            continue;
                        }
                        '*' if chars.get(index + 1).copied() == Some('/') => {
                            local_block_comment_depth = local_block_comment_depth.saturating_sub(1);
                            index += 2;
                            continue;
                        }
                        _ => {
                            index += 1;
                            continue;
                        }
                    }
                }
                if local_in_string {
                    if local_escape {
                        local_escape = false;
                    } else {
                        match current {
                            '\\' => local_escape = true,
                            '"' => local_in_string = false,
                            _ => {}
                        }
                    }
                    index += 1;
                    continue;
                }
                match current {
                    '"' => {
                        local_in_string = true;
                        index += 1;
                    }
                    '/' if chars.get(index + 1).copied() == Some('/') => {
                        local_in_line_comment = true;
                        index += 2;
                    }
                    '/' if chars.get(index + 1).copied() == Some('*') => {
                        local_block_comment_depth = 1;
                        index += 2;
                    }
                    '{' => {
                        local_depth += 1;
                        index += 1;
                    }
                    '}' => {
                        local_depth = local_depth.saturating_sub(1);
                        if local_depth == 0 {
                            let body = chars[body_start..index]
                                .iter()
                                .collect::<String>()
                                .trim()
                                .to_owned();
                            let normalized_body = normalize_inline_wgsl_source(&body)?;
                            let needs_separator = !out.is_empty()
                                && !out.ends_with('\n')
                                && !out.ends_with('{')
                                && !out.ends_with('}');
                            if needs_separator {
                                out.push('\n');
                            }
                            out.push('@');
                            out.push_str(&stage_kind);
                            out.push('\n');
                            for attribute in &stage_attributes {
                                out.push('@');
                                out.push_str(attribute);
                                out.push('\n');
                            }
                            out.push_str(&normalized_body);
                            out.push('\n');
                            index += 1;
                            transformed = true;
                            break;
                        }
                        index += 1;
                    }
                    _ => index += 1,
                }
            }
            if local_depth != 0 {
                return Err(format!("wgsl stage block `{stage_kind}` is unterminated"));
            }
            if index == original {
                index += 1;
            }
            continue;
        }

        if depth == 0 && starts_with_keyword(&chars, index, "binding") {
            let original = index;
            index += "binding".chars().count();
            while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
                index += 1;
            }
            if chars.get(index).copied() != Some('(') {
                return Err("wgsl binding declaration must use `binding(group, slot)`".to_owned());
            }
            let ((group, slot), next_index) = parse_binding_metadata(&chars, index)?;
            index = next_index;
            while chars.get(index).is_some_and(|ch| ch.is_whitespace()) {
                index += 1;
            }
            let declaration_start = index;
            let mut local_in_string = false;
            let mut local_escape = false;
            let mut local_in_line_comment = false;
            let mut local_block_comment_depth = 0usize;
            let mut generic_depth = 0usize;
            while index < chars.len() {
                let current = chars[index];
                if local_in_line_comment {
                    if current == '\n' {
                        local_in_line_comment = false;
                    }
                    index += 1;
                    continue;
                }
                if local_block_comment_depth > 0 {
                    match current {
                        '/' if chars.get(index + 1).copied() == Some('*') => {
                            local_block_comment_depth += 1;
                            index += 2;
                            continue;
                        }
                        '*' if chars.get(index + 1).copied() == Some('/') => {
                            local_block_comment_depth = local_block_comment_depth.saturating_sub(1);
                            index += 2;
                            continue;
                        }
                        _ => {
                            index += 1;
                            continue;
                        }
                    }
                }
                if local_in_string {
                    if local_escape {
                        local_escape = false;
                    } else {
                        match current {
                            '\\' => local_escape = true,
                            '"' => local_in_string = false,
                            _ => {}
                        }
                    }
                    index += 1;
                    continue;
                }

                match current {
                    '"' => {
                        local_in_string = true;
                        index += 1;
                    }
                    '/' if chars.get(index + 1).copied() == Some('/') => {
                        local_in_line_comment = true;
                        index += 2;
                    }
                    '/' if chars.get(index + 1).copied() == Some('*') => {
                        local_block_comment_depth = 1;
                        index += 2;
                    }
                    '<' => {
                        generic_depth += 1;
                        index += 1;
                    }
                    '>' => {
                        generic_depth = generic_depth.saturating_sub(1);
                        index += 1;
                    }
                    ';' if generic_depth == 0 => {
                        let declaration = chars[declaration_start..=index]
                            .iter()
                            .collect::<String>()
                            .trim()
                            .to_owned();
                        let needs_separator = !out.is_empty()
                            && !out.ends_with('\n')
                            && !out.ends_with('{')
                            && !out.ends_with('}');
                        if needs_separator {
                            out.push('\n');
                        }
                        out.push_str("@group(");
                        out.push_str(&group);
                        out.push_str(")\n");
                        out.push_str("@binding(");
                        out.push_str(&slot);
                        out.push_str(")\n");
                        out.push_str(&declaration);
                        out.push('\n');
                        index += 1;
                        transformed = true;
                        break;
                    }
                    _ => index += 1,
                }
            }
            if index == original {
                index += 1;
            }
            if index >= chars.len() && declaration_start < chars.len() {
                return Err("wgsl binding declaration is unterminated; expected `;`".to_owned());
            }
            continue;
        }

        if starts_with_keyword(&chars, index, "invariant") && is_attribute_position(&chars, index) {
            let needs_separator = out
                .chars()
                .last()
                .is_some_and(|ch| !ch.is_whitespace() && ch != '@' && ch != '(');
            if needs_separator {
                out.push(' ');
            }
            out.push_str("@invariant");
            index += "invariant".chars().count();
            transformed = true;
            continue;
        }

        if starts_with_bare_attribute_keyword(&chars, index) && is_attribute_position(&chars, index)
        {
            let attribute_start = index;
            while chars
                .get(index)
                .is_some_and(|ch| ch.is_ascii_alphabetic() || *ch == '_')
            {
                index += 1;
            }
            let attribute_name = chars[attribute_start..index].iter().collect::<String>();
            if chars.get(index).copied() != Some('(') {
                return Err(format!(
                    "wgsl attribute `{attribute_name}` must use `{attribute_name}(...)`"
                ));
            }
            let (args, next_index) = parse_parenthesized_args(&chars, index)?;
            let needs_separator = out
                .chars()
                .last()
                .is_some_and(|ch| !ch.is_whitespace() && ch != '@' && ch != '(');
            if needs_separator {
                out.push(' ');
            }
            out.push('@');
            out.push_str(&attribute_name);
            out.push('(');
            out.push_str(&args);
            out.push(')');
            index = next_index;
            transformed = true;
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
                out.push(ch);
            }
            '/' if chars.get(index + 1).copied() == Some('/') => {
                out.push(ch);
                out.push('/');
                in_line_comment = true;
                index += 2;
                continue;
            }
            '/' if chars.get(index + 1).copied() == Some('*') => {
                out.push(ch);
                out.push('*');
                block_comment_depth = 1;
                index += 2;
                continue;
            }
            '{' => {
                depth += 1;
                out.push(ch);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                out.push(ch);
            }
            _ => out.push(ch),
        }
        index += 1;
    }

    if transformed {
        Ok(out.trim().to_owned())
    } else {
        Ok(source.trim().to_owned())
    }
}

fn starts_with_keyword(chars: &[char], index: usize, keyword: &str) -> bool {
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

fn parse_stage_metadata(
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

fn parse_parenthesized_args(chars: &[char], start: usize) -> Result<(String, usize), String> {
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

fn parse_binding_metadata(
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

fn starts_with_bare_attribute_keyword(chars: &[char], index: usize) -> bool {
    ["builtin", "location", "interpolate"]
        .iter()
        .any(|keyword| {
            starts_with_keyword(chars, index, keyword)
                && !previous_significant_char(chars, index).is_some_and(|ch| ch == '@')
        })
}

fn is_attribute_position(chars: &[char], index: usize) -> bool {
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

#[cfg(test)]
mod tests {
    use super::normalize_inline_wgsl_source;

    #[test]
    fn normalizes_top_level_stage_blocks_into_standard_wgsl_attributes() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

stage vertex {
  fn vs_main() -> VsOut {
    var out: VsOut;
    return out;
  }
}

stage fragment {
  fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}
"#,
        )
        .expect("stage blocks normalize");

        assert!(normalized.contains("@vertex"), "{normalized}");
        assert!(normalized.contains("@fragment"), "{normalized}");
        assert!(!normalized.contains("stage vertex"), "{normalized}");
        assert!(!normalized.contains("stage fragment"), "{normalized}");
    }

    #[test]
    fn normalizes_compute_stage_workgroup_size_metadata() {
        let normalized = normalize_inline_wgsl_source(
            r#"
stage compute(workgroup_size(8, 4, 1)) {
  fn cs_main() {
  }
}
"#,
        )
        .expect("compute stage metadata normalizes");

        assert!(normalized.contains("@compute"), "{normalized}");
        assert!(
            normalized.contains("@workgroup_size(8, 4, 1)"),
            "{normalized}"
        );
        assert!(!normalized.contains("stage compute"), "{normalized}");
    }

    #[test]
    fn normalizes_fragment_stage_metadata_lists() {
        let normalized = normalize_inline_wgsl_source(
            r#"
stage fragment(early_depth_test) {
  fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}
"#,
        )
        .expect("fragment stage metadata normalizes");

        assert!(normalized.contains("@fragment"), "{normalized}");
        assert!(normalized.contains("@early_depth_test"), "{normalized}");
        assert!(!normalized.contains("stage fragment"), "{normalized}");
    }

    #[test]
    fn normalizes_multiple_stage_metadata_entries() {
        let normalized = normalize_inline_wgsl_source(
            r#"
stage compute(workgroup_size(8, 4, 1)) {
  fn cs_main() {
  }
}

stage fragment(early_depth_test) {
  fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}
"#,
        )
        .expect("multiple stage metadata entries normalize");

        assert!(
            normalized.contains("@workgroup_size(8, 4, 1)"),
            "{normalized}"
        );
        assert!(normalized.contains("@early_depth_test"), "{normalized}");
    }

    #[test]
    fn normalizes_top_level_binding_declarations() {
        let normalized = normalize_inline_wgsl_source(
            r#"
binding(0, 0) var color_sampler: sampler;
binding(0, 1) var color_tex: texture_2d<f32>;
"#,
        )
        .expect("binding declarations normalize");

        assert!(normalized.contains("@group(0)\n@binding(0)\nvar color_sampler: sampler;"));
        assert!(normalized.contains("@group(0)\n@binding(1)\nvar color_tex: texture_2d<f32>;"));
        assert!(!normalized.contains("binding(0, 0)"), "{normalized}");
    }

    #[test]
    fn normalizes_binding_declarations_with_uniform_generics() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct Globals {
  exposure: f32,
};

binding(0, 2) var<uniform> globals: Globals;
"#,
        )
        .expect("uniform binding declaration normalizes");

        assert!(normalized.contains("@group(0)\n@binding(2)\nvar<uniform> globals: Globals;"));
    }

    #[test]
    fn normalizes_bare_builtin_and_location_attributes() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct VsOut {
  builtin(position) pos: vec4<f32>,
  location(0) uv: vec2<f32>,
};

stage vertex {
  fn vs_main(builtin(vertex_index) vid: u32) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    out.uv = vec2<f32>(f32(vid), 0.0);
    return out;
  }
}

stage fragment {
  fn fs_main(location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
    return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
  }
}
"#,
        )
        .expect("builtin/location attributes normalize");

        assert!(
            normalized.contains("@builtin(position) pos: vec4<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains("@location(0) uv: vec2<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains("fn vs_main(@builtin(vertex_index) vid: u32)"),
            "{normalized}"
        );
        assert!(
            normalized.contains("fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32>"),
            "{normalized}"
        );
        assert!(
            !normalized.contains("\n  builtin(position)"),
            "{normalized}"
        );
        assert!(!normalized.contains("\n  location(0)"), "{normalized}");
    }

    #[test]
    fn normalizes_bare_interpolate_and_invariant_attributes() {
        let normalized = normalize_inline_wgsl_source(
            r#"
struct VsOut {
  invariant builtin(position) pos: vec4<f32>,
  interpolate(flat) location(0) uv: vec2<f32>,
};

stage fragment {
  fn fs_main(interpolate(flat) location(0) uv: vec2<f32>) -> location(0) vec4<f32> {
    return vec4<f32>(uv.x, uv.y, 1.0, 1.0);
  }
}
"#,
        )
        .expect("interpolate/invariant attributes normalize");

        assert!(
            normalized.contains("@invariant @builtin(position) pos: vec4<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains("@interpolate(flat) @location(0) uv: vec2<f32>,"),
            "{normalized}"
        );
        assert!(
            normalized.contains(
                "fn fs_main(@interpolate(flat) @location(0) uv: vec2<f32>) -> @location(0) vec4<f32>"
            ),
            "{normalized}"
        );
        assert!(
            !normalized.contains("\n  invariant builtin(position)"),
            "{normalized}"
        );
        assert!(
            !normalized.contains("\n  interpolate(flat) location(0)"),
            "{normalized}"
        );
    }
}
