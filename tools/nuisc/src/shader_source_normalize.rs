use super::*;

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
