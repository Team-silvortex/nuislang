use super::{NustarContractInstruction, NustarContractStage, NustarContractTerminator};

pub(super) fn build_shader_ir_stage_contracts(wgsl_source: &str) -> Vec<NustarContractStage> {
    let mut stages = Vec::new();
    if let Some(vertex_src) = extract_shader_stage_source(wgsl_source, "@vertex", "@fragment") {
        if let Some(stage) = build_shader_ir_stage_contract("vertex", &vertex_src) {
            stages.push(stage);
        }
    }
    if let Some(fragment_src) = extract_shader_stage_source(wgsl_source, "@fragment", "") {
        if let Some(stage) = build_shader_ir_stage_contract("fragment", &fragment_src) {
            stages.push(stage);
        }
    }
    stages
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

    let return_expr = extract_fragment_return_expr_from_source(stage_src)?;
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
        terminator: NustarContractTerminator {
            op: "return".to_owned(),
            expr: return_expr,
        },
    })
}

fn extract_shader_stage_source(
    wgsl_source: &str,
    stage_marker: &str,
    next_marker: &str,
) -> Option<String> {
    let start = wgsl_source.find(stage_marker)?;
    let tail = &wgsl_source[start..];
    if next_marker.is_empty() {
        return Some(tail.to_owned());
    }
    let end = tail.find(next_marker)?;
    Some(tail[..end].to_owned())
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

fn extract_fragment_return_expr_from_source(fragment_src: &str) -> Option<String> {
    let return_pos = fragment_src.find("return")?;
    let after_return = &fragment_src[return_pos + "return".len()..];
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
