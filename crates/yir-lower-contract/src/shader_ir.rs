use super::{
    NustarContractInstruction, NustarContractStage, NustarContractTerminator,
    ShaderModuleBindingContract, ShaderModuleContract, ShaderModuleStageContract,
};

pub(super) fn build_inline_shader_module_contract(
    resource: &str,
    entry: &str,
    wgsl_source: &str,
) -> ShaderModuleContract {
    ShaderModuleContract {
        schema: "nuis-yir.shader.module-summary.v1".to_owned(),
        resource: resource.to_owned(),
        entry: entry.to_owned(),
        source_language: "wgsl".to_owned(),
        stages: build_shader_module_stage_contracts(wgsl_source),
        bindings: build_shader_module_binding_contracts(wgsl_source),
    }
}

pub(super) fn build_shader_ir_stage_contracts(wgsl_source: &str) -> Vec<NustarContractStage> {
    collect_shader_stage_sources(wgsl_source)
        .into_iter()
        .filter_map(|stage| build_shader_ir_stage_contract(stage.stage, &stage.source))
        .collect()
}

fn build_shader_module_stage_contracts(wgsl_source: &str) -> Vec<ShaderModuleStageContract> {
    collect_shader_stage_sources(wgsl_source)
        .into_iter()
        .filter_map(|stage_source| {
            let (attrs, fn_line) = collect_stage_attrs_and_fn_line(&stage_source.source)?;
            let entry = parse_function_name(fn_line)?;
            let return_type = fn_line
                .split_once("->")
                .and_then(|(_, tail)| normalize_return_type(tail));
            let workgroup_size = attrs
                .iter()
                .find_map(|attr| attr.strip_prefix("workgroup_size("))
                .and_then(|tail| tail.strip_suffix(')'))
                .map(str::to_owned);
            Some(ShaderModuleStageContract {
                stage: stage_source.stage.to_owned(),
                entry,
                attributes: attrs,
                workgroup_size,
                return_type,
            })
        })
        .collect()
}

fn build_shader_module_binding_contracts(wgsl_source: &str) -> Vec<ShaderModuleBindingContract> {
    let searchable = strip_comments_preserving_shape(wgsl_source);
    let mut bindings = Vec::new();
    let mut pending_attrs = Vec::new();
    let mut brace_depth = 0usize;

    for raw_line in searchable.lines() {
        let line_depth = brace_depth;
        let line = raw_line.trim();
        if line_depth == 0 && !line.is_empty() {
            let Some((attrs, rest)) = collect_leading_attributes(line) else {
                pending_attrs.clear();
                brace_depth = update_brace_depth(line, brace_depth);
                continue;
            };
            if !attrs.is_empty() {
                pending_attrs.extend(attrs);
            }
            let rest = rest.trim_start();
            if rest.starts_with("var") {
                if let Some(binding) = parse_module_binding(rest, &pending_attrs) {
                    bindings.push(binding);
                }
                pending_attrs.clear();
            } else if !rest.is_empty() {
                pending_attrs.clear();
            }
        }

        brace_depth = update_brace_depth(line, brace_depth);
    }

    bindings
}

fn build_shader_ir_stage_contract(
    stage_name: &str,
    stage_src: &str,
) -> Option<NustarContractStage> {
    let mut instructions = Vec::new();
    for raw_line in stage_src.lines() {
        let line = raw_line.trim();
        if line.starts_with("let ") {
            let Some(eq_pos) = line.find('=') else {
                continue;
            };
            let lhs = line["let ".len()..eq_pos].trim();
            let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
            if rhs.is_empty() {
                continue;
            }
            let (result, ty) = if let Some(colon_pos) = lhs.find(':') {
                (
                    lhs[..colon_pos].trim().to_owned(),
                    Some(lhs[colon_pos + 1..].trim().to_owned()),
                )
            } else {
                (lhs.to_owned(), None)
            };
            if result.is_empty() {
                continue;
            }
            instructions.push(NustarContractInstruction {
                result,
                ty,
                op: classify_shader_ir_op(rhs),
                args: collect_shader_ir_args(rhs),
                expr: rhs.to_owned(),
            });
        } else if line.contains('=') && line.ends_with(';') && !line.starts_with("return ") {
            let eq_pos = line.find('=').expect("checked contains =");
            let lhs = line[..eq_pos].trim();
            let rhs = line[eq_pos + 1..].trim().trim_end_matches(';').trim();
            if lhs.is_empty() || rhs.is_empty() {
                continue;
            }
            instructions.push(NustarContractInstruction {
                result: lhs.to_owned(),
                ty: None,
                op: "assign".to_owned(),
                args: collect_shader_ir_args(rhs),
                expr: rhs.to_owned(),
            });
        }
    }

    let terminator = extract_return_expr_from_source(stage_src)
        .map(|return_expr| NustarContractTerminator {
            op: "return".to_owned(),
            expr: return_expr,
        })
        .unwrap_or_else(|| NustarContractTerminator {
            op: "end".to_owned(),
            expr: "void".to_owned(),
        });

    Some(NustarContractStage {
        stage: stage_name.to_owned(),
        function: format!("shader.{stage_name}"),
        node_kind: "function-node".to_owned(),
        execution_domain: "shader".to_owned(),
        time_mode: "logical".to_owned(),
        contract_family: "nustar.shader".to_owned(),
        time_domain: format!("shader.stage.{stage_name}"),
        glm_scope: format!("shader::{stage_name}"),
        instructions,
        terminator,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShaderStageSource {
    stage: &'static str,
    source: String,
}

fn collect_shader_stage_sources(wgsl_source: &str) -> Vec<ShaderStageSource> {
    let searchable = strip_comments_preserving_shape(wgsl_source);
    let mut markers = Vec::new();
    for (marker, stage) in [
        ("@vertex", "vertex"),
        ("@fragment", "fragment"),
        ("@compute", "compute"),
    ] {
        let mut offset = 0usize;
        while let Some(relative) = searchable[offset..].find(marker) {
            let start = offset + relative;
            if has_stage_marker_boundary(&searchable, start + marker.len()) {
                markers.push((start, stage));
            }
            offset = start + marker.len();
        }
    }
    markers.sort_by_key(|(start, _)| *start);

    markers
        .iter()
        .enumerate()
        .map(|(index, (start, stage))| {
            let end = markers
                .get(index + 1)
                .map(|(next_start, _)| *next_start)
                .unwrap_or(searchable.len());
            ShaderStageSource {
                stage,
                source: searchable[*start..end].to_owned(),
            }
        })
        .collect()
}

fn has_stage_marker_boundary(source: &str, marker_end: usize) -> bool {
    source[marker_end..]
        .chars()
        .next()
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
}

fn collect_stage_attrs_and_fn_line(stage_source: &str) -> Option<(Vec<String>, &str)> {
    let mut attrs = Vec::new();
    for raw_line in stage_source.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let (line_attrs, rest) = collect_leading_attributes(line)?;
        attrs.extend(line_attrs);
        let rest = rest.trim_start();
        if rest.starts_with("fn ") {
            return Some((attrs, rest));
        }
    }
    None
}

fn parse_module_binding(rest: &str, attrs: &[String]) -> Option<ShaderModuleBindingContract> {
    let group = attr_u32(attrs, "group")?;
    let binding = attr_u32(attrs, "binding")?;
    let after_var = rest.strip_prefix("var")?.trim_start();
    let (address_space, after_header) = if let Some(after_open) = after_var.strip_prefix('<') {
        let close = after_open.find('>')?;
        (
            Some(after_open[..close].trim().to_owned()),
            after_open[close + 1..].trim_start(),
        )
    } else {
        (None, after_var)
    };
    let (name, ty_tail) = after_header.split_once(':')?;
    let name = name.trim();
    let ty = ty_tail.split([';', '=']).next().unwrap_or_default().trim();
    if name.is_empty() || ty.is_empty() {
        return None;
    }
    Some(ShaderModuleBindingContract {
        group,
        binding,
        name: name.to_owned(),
        kind: classify_binding_kind(address_space.as_deref(), ty),
        address_space,
        ty: ty.to_owned(),
    })
}

fn collect_leading_attributes(line: &str) -> Option<(Vec<String>, &str)> {
    let mut attrs = Vec::new();
    let mut rest = line.trim_start();
    while let Some(after_at) = rest.strip_prefix('@') {
        let name_len = after_at
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphabetic() || *ch == '_')
            .map(|(index, ch)| index + ch.len_utf8())
            .last()?;
        let name = &after_at[..name_len];
        let after_name = after_at[name_len..].trim_start();
        if let Some(after_open) = after_name.strip_prefix('(') {
            let close = matching_paren_close(after_open)?;
            let args = after_open[..close].trim();
            attrs.push(format!("{name}({args})"));
            rest = after_open[close + 1..].trim_start();
        } else {
            attrs.push(name.to_owned());
            rest = after_name;
        }
    }
    Some((attrs, rest))
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
    loop {
        let Some((attrs, rest)) = collect_leading_attributes(ty) else {
            break;
        };
        if attrs.is_empty() {
            break;
        }
        ty = rest.trim_start();
    }
    (!ty.is_empty()).then(|| ty.to_owned())
}

fn attr_u32(attrs: &[String], name: &str) -> Option<u32> {
    attrs
        .iter()
        .find_map(|attr| attr.strip_prefix(&format!("{name}(")))
        .and_then(|tail| tail.strip_suffix(')'))
        .and_then(|value| value.trim().parse::<u32>().ok())
}

fn classify_binding_kind(address_space: Option<&str>, ty: &str) -> String {
    if ty == "sampler" || ty.starts_with("sampler_") {
        "sampler".to_owned()
    } else if ty.starts_with("texture_") {
        "texture".to_owned()
    } else if matches!(address_space, Some("uniform")) {
        "uniform".to_owned()
    } else if matches!(address_space, Some("storage") | Some("storage, read")) {
        "storage".to_owned()
    } else {
        "var".to_owned()
    }
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

fn classify_shader_ir_op(expr: &str) -> String {
    if expr.contains("textureSample(") {
        "sample_texture".to_owned()
    } else if expr.contains("smoothstep(") {
        "smoothstep".to_owned()
    } else if expr.contains("normalize(") {
        "normalize".to_owned()
    } else if expr.contains("dot(") {
        "dot".to_owned()
    } else if expr.contains("clamp(") {
        "clamp".to_owned()
    } else if expr.contains("fract(") {
        "fract".to_owned()
    } else if expr.contains("mix(") {
        "mix".to_owned()
    } else if expr.contains("vec4") || expr.contains("vec3") || expr.contains("vec2") {
        "construct".to_owned()
    } else {
        "expr".to_owned()
    }
}

fn collect_shader_ir_args(expr: &str) -> Vec<String> {
    if let Some(open) = expr.find('(') {
        if let Some(close) = expr.rfind(')') {
            if close > open {
                return expr[open + 1..close]
                    .split(',')
                    .map(str::trim)
                    .filter(|arg| !arg.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
            }
        }
    }
    Vec::new()
}

fn extract_return_expr_from_source(stage_src: &str) -> Option<String> {
    let return_pos = stage_src.find("return")?;
    let after_return = &stage_src[return_pos + "return".len()..];
    let semicolon_pos = after_return.find(';')?;
    Some(after_return[..semicolon_pos].trim().to_owned())
}

pub(super) fn decode_inline_shader_source(raw: &str) -> String {
    fn decode_once(raw: &str) -> String {
        let mut out = String::new();
        let mut chars = raw.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        }
        out
    }

    let mut current = raw.to_owned();
    for _ in 0..2 {
        let decoded = decode_once(&current);
        if decoded == current {
            break;
        }
        current = decoded;
    }
    current
}
