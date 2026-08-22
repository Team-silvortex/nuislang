#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineWgslSummary {
    pub(crate) schema: &'static str,
    pub(crate) stages: Vec<InlineWgslStageSummary>,
    pub(crate) bindings: Vec<InlineWgslBindingSummary>,
}

impl InlineWgslSummary {
    pub(crate) fn has_stage(&self, expected: &str) -> bool {
        self.stages.iter().any(|stage| stage.stage == expected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineWgslStageSummary {
    pub(crate) stage: String,
    pub(crate) entry: String,
    pub(crate) attributes: Vec<String>,
    pub(crate) workgroup_size: Option<String>,
    pub(crate) return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineWgslBindingSummary {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) address_space: Option<String>,
    pub(crate) ty: String,
}

pub(crate) fn summarize_inline_wgsl_source(source: &str) -> Result<InlineWgslSummary, String> {
    let normalized = super::normalize_inline_wgsl_source(source)?;
    summarize_normalized_wgsl_source(&normalized)
}

fn summarize_normalized_wgsl_source(source: &str) -> Result<InlineWgslSummary, String> {
    let scrubbed = strip_comments_preserving_shape(source);
    let mut stages = Vec::new();
    let mut bindings = Vec::new();
    let mut pending_attrs = Vec::<String>::new();
    let mut brace_depth = 0usize;

    for raw_line in scrubbed.lines() {
        let line_depth = brace_depth;
        let line = raw_line.trim();
        if line_depth == 0 && !line.is_empty() {
            let (attrs, rest) = collect_leading_attributes(line)?;
            if !attrs.is_empty() {
                pending_attrs.extend(attrs);
            }
            let rest = rest.trim_start();
            if rest.starts_with("fn ") {
                if let Some(stage) = stage_from_attrs(&pending_attrs) {
                    stages.push(parse_stage_summary(stage, rest, &pending_attrs)?);
                    pending_attrs.clear();
                }
            } else if rest.starts_with("var") {
                if let Some(binding) = parse_binding_summary(rest, &pending_attrs)? {
                    bindings.push(binding);
                    pending_attrs.clear();
                }
            } else if !rest.is_empty() {
                pending_attrs.clear();
            }
        }

        brace_depth = update_brace_depth(line, brace_depth);
    }

    Ok(InlineWgslSummary {
        schema: "nuis-inline-wgsl-summary-v1",
        stages,
        bindings,
    })
}

fn parse_stage_summary(
    stage: &str,
    rest: &str,
    pending_attrs: &[String],
) -> Result<InlineWgslStageSummary, String> {
    let entry = parse_function_name(rest)
        .ok_or_else(|| format!("wgsl {stage} stage must declare a function name"))?;
    let return_type = rest
        .split_once("->")
        .and_then(|(_, tail)| normalize_return_type(tail));
    let workgroup_size = pending_attrs
        .iter()
        .find_map(|attr| attr.strip_prefix("workgroup_size("))
        .and_then(|tail| tail.strip_suffix(')'))
        .map(str::to_owned);

    Ok(InlineWgslStageSummary {
        stage: stage.to_owned(),
        entry,
        attributes: pending_attrs.to_vec(),
        workgroup_size,
        return_type,
    })
}

fn parse_binding_summary(
    rest: &str,
    pending_attrs: &[String],
) -> Result<Option<InlineWgslBindingSummary>, String> {
    let Some(group) = attr_u32(pending_attrs, "group")? else {
        return Ok(None);
    };
    let Some(binding) = attr_u32(pending_attrs, "binding")? else {
        return Ok(None);
    };
    let Some(after_var) = rest.strip_prefix("var") else {
        return Ok(None);
    };
    let after_var = after_var.trim_start();
    let (address_space, after_header) = if let Some(after_open) = after_var.strip_prefix('<') {
        let close = after_open
            .find('>')
            .ok_or_else(|| "wgsl binding var<...> is missing `>`".to_owned())?;
        (
            Some(after_open[..close].trim().to_owned()),
            after_open[close + 1..].trim_start(),
        )
    } else {
        (None, after_var)
    };
    let (name, ty_tail) = after_header
        .split_once(':')
        .ok_or_else(|| "wgsl binding declaration must include `name: Type`".to_owned())?;
    let name = name.trim();
    if name.is_empty() {
        return Err("wgsl binding declaration has an empty name".to_owned());
    }
    let ty = ty_tail
        .split([';', '='])
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if ty.is_empty() {
        return Err(format!("wgsl binding `{name}` has an empty type"));
    }
    let kind = classify_binding_kind(address_space.as_deref(), &ty);

    Ok(Some(InlineWgslBindingSummary {
        group,
        binding,
        name: name.to_owned(),
        kind,
        address_space,
        ty,
    }))
}

fn classify_binding_kind(address_space: Option<&str>, ty: &str) -> String {
    if ty == "sampler" || ty.starts_with("sampler_") {
        "sampler".to_owned()
    } else if ty.starts_with("texture_") {
        "texture".to_owned()
    } else if matches!(address_space, Some("uniform")) {
        "uniform".to_owned()
    } else if matches!(
        address_space,
        Some("storage") | Some("storage, read") | Some("storage, read_write")
    ) {
        "storage".to_owned()
    } else {
        "var".to_owned()
    }
}

fn stage_from_attrs(attrs: &[String]) -> Option<&'static str> {
    if attrs.iter().any(|attr| attr == "vertex") {
        Some("vertex")
    } else if attrs.iter().any(|attr| attr == "fragment") {
        Some("fragment")
    } else if attrs.iter().any(|attr| attr == "compute") {
        Some("compute")
    } else {
        None
    }
}

fn attr_u32(attrs: &[String], name: &str) -> Result<Option<u32>, String> {
    let Some(value) = attrs
        .iter()
        .find_map(|attr| attr.strip_prefix(&format!("{name}(")))
        .and_then(|tail| tail.strip_suffix(')'))
    else {
        return Ok(None);
    };
    value
        .trim()
        .parse::<u32>()
        .map(Some)
        .map_err(|_| format!("wgsl attribute `{name}` value `{value}` must be an integer"))
}

fn collect_leading_attributes(line: &str) -> Result<(Vec<String>, &str), String> {
    let mut attrs = Vec::new();
    let mut rest = line.trim_start();
    while let Some(after_at) = rest.strip_prefix('@') {
        let name_len = after_at
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphabetic() || *ch == '_')
            .map(|(index, ch)| index + ch.len_utf8())
            .last()
            .unwrap_or(0);
        if name_len == 0 {
            return Err("wgsl attribute is missing a name".to_owned());
        }
        let name = &after_at[..name_len];
        let after_name = after_at[name_len..].trim_start();
        if let Some(after_open) = after_name.strip_prefix('(') {
            let close = matching_paren_close(after_open)
                .ok_or_else(|| format!("wgsl attribute `{name}` has unterminated arguments"))?;
            let args = after_open[..close].trim();
            attrs.push(format!("{name}({args})"));
            rest = after_open[close + 1..].trim_start();
        } else {
            attrs.push(name.to_owned());
            rest = after_name;
        }
    }
    Ok((attrs, rest))
}

fn matching_paren_close(source_after_open: &str) -> Option<usize> {
    let mut depth = 1usize;
    for (index, ch) in source_after_open.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_function_name(rest: &str) -> Option<String> {
    let after_fn = rest.strip_prefix("fn ")?.trim_start();
    let end = after_fn
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(|(index, ch)| index + ch.len_utf8())
        .last()?;
    Some(after_fn[..end].to_owned())
}

fn normalize_return_type(raw_tail: &str) -> Option<String> {
    let before_body = raw_tail
        .split('{')
        .next()
        .unwrap_or(raw_tail)
        .trim()
        .trim_end_matches(';')
        .trim();
    let mut ty = before_body;
    while let Ok((attrs, rest)) = collect_leading_attributes(ty) {
        if attrs.is_empty() {
            break;
        }
        ty = rest.trim_start();
    }
    (!ty.is_empty()).then(|| ty.to_owned())
}

fn update_brace_depth(line: &str, mut depth: usize) -> usize {
    for ch in line.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn strip_comments_preserving_shape(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(source.len());
    let mut index = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut block_depth = 0usize;

    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            out.push(ch);
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if block_depth > 0 {
            if ch == '/' && chars.get(index + 1).copied() == Some('*') {
                out.push(' ');
                out.push(' ');
                block_depth += 1;
                index += 2;
            } else if ch == '*' && chars.get(index + 1).copied() == Some('/') {
                out.push(' ');
                out.push(' ');
                block_depth = block_depth.saturating_sub(1);
                index += 2;
            } else {
                out.push(if ch == '\n' { '\n' } else { ' ' });
                index += 1;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            out.push(ch);
            index += 1;
        } else if ch == '/' && chars.get(index + 1).copied() == Some('/') {
            out.push(' ');
            out.push(' ');
            index += 2;
            while index < chars.len() && chars[index] != '\n' {
                out.push(' ');
                index += 1;
            }
        } else if ch == '/' && chars.get(index + 1).copied() == Some('*') {
            out.push(' ');
            out.push(' ');
            block_depth = 1;
            index += 2;
        } else {
            out.push(ch);
            index += 1;
        }
    }

    out
}
