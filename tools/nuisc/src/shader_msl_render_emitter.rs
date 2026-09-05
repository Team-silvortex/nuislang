const MSL_TARGETS: &[&str] = &[
    "metal.apple-silicon-gpu",
    "metal.mac-discrete-or-integrated-gpu",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredMslRenderModule {
    pub source: String,
    pub vertex_entry: String,
    pub fragment_entry: String,
}

pub fn lower_canonical_inline_wgsl_render_for_profile(
    source: &str,
    profile_entry: &str,
    profile_lowering_target: &str,
) -> Result<LoweredMslRenderModule, String> {
    if !MSL_TARGETS.contains(&profile_lowering_target) {
        return Err(format!(
            "canonical inline WGSL render target `{profile_lowering_target}` is unsupported"
        ));
    }
    if !valid_ident(profile_entry) {
        return Err("canonical inline WGSL render profile entry must be an identifier".to_owned());
    }

    let source = crate::shader_source::normalize_inline_wgsl_source(source)?;
    let source = crate::shader_source::strip_comments_preserving_shape(&source);
    let summary = crate::shader_source::summarize_inline_wgsl_source(&source)?;
    let uniform = fragment_uniform(&source, &summary)?;
    let vertex_entry = unique_stage_entry(&summary, "vertex")?;
    let fragment_entry = unique_stage_entry(&summary, "fragment")?;
    if summary.stages.len() != 2 {
        return Err(
            "canonical inline WGSL render module must contain exactly vertex and fragment stages"
                .to_owned(),
        );
    }

    let vertex_body = function_body(&source, &vertex_entry)?;
    validate_fullscreen_vertex_signature(&source, &vertex_entry)?;
    validate_fullscreen_vertex_body(vertex_body)?;
    let fragment_body = function_body(&source, &fragment_entry)?;
    let fragment_statements = lower_fragment_body(fragment_body)?;
    let source = render_msl(
        profile_entry,
        profile_lowering_target,
        &vertex_entry,
        &fragment_entry,
        &fragment_statements,
        uniform,
    );
    Ok(LoweredMslRenderModule {
        source,
        vertex_entry,
        fragment_entry,
    })
}

fn fragment_uniform<'a>(
    source: &str,
    summary: &'a crate::shader_source::InlineWgslSummary,
) -> Result<Option<(u32, &'a str)>, String> {
    let binding = match summary.bindings.as_slice() {
        [] => return Ok(None),
        [binding] => binding,
        _ => return Err("canonical render supports at most one fragment uniform".to_owned()),
    };
    let compact_type = binding
        .ty
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    if binding.group != 0
        || binding.binding > 30
        || binding.kind != "uniform"
        || binding.address_space.as_deref() != Some("uniform")
        || compact_type != "vec4<f32>"
        || !valid_ident(&binding.name)
        || matches!(binding.name.as_str(), "uv" | "input")
    {
        return Err("canonical render requires group-zero fragment vec4<f32> uniform".to_owned());
    }
    let compact = source
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>();
    let declaration = format!(
        "@group(0)@binding({})var<uniform>{}:vec4<f32>;",
        binding.binding, binding.name
    );
    if compact.matches(&declaration).count() != 1 {
        return Err("unsupported fragment uniform declaration or initializer".to_owned());
    }
    Ok(Some((binding.binding, &binding.name)))
}

fn unique_stage_entry(
    summary: &crate::shader_source::InlineWgslSummary,
    stage: &str,
) -> Result<String, String> {
    let entries = summary
        .stages
        .iter()
        .filter(|item| item.stage == stage)
        .map(|item| item.entry.clone())
        .collect::<Vec<_>>();
    match entries.as_slice() {
        [entry] => Ok(entry.clone()),
        [] => Err(format!(
            "canonical inline WGSL render module is missing its {stage} stage"
        )),
        _ => Err(format!(
            "canonical inline WGSL render module has multiple {stage} stages"
        )),
    }
}

fn function_body<'a>(source: &'a str, entry: &str) -> Result<&'a str, String> {
    let marker = format!("fn {entry}");
    let function_start = source
        .find(&marker)
        .ok_or_else(|| format!("WGSL stage entry `{entry}` is missing"))?;
    let body_start = source[function_start..]
        .find('{')
        .map(|offset| function_start + offset)
        .ok_or_else(|| format!("WGSL stage entry `{entry}` has no body"))?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, ch) in source[body_start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(&source[body_start + 1..body_start + offset]);
                }
            }
            _ => {}
        }
    }
    Err(format!(
        "WGSL stage entry `{entry}` has an unterminated body"
    ))
}

fn validate_fullscreen_vertex_signature(source: &str, entry: &str) -> Result<(), String> {
    let marker = format!("fn {entry}");
    let start = source
        .find(&marker)
        .ok_or("canonical vertex function is missing")?;
    let header = source[start..].split('{').next().unwrap_or_default();
    let compact = header
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    if compact != format!("fn{entry}(@builtin(vertex_index)vid:u32)->VsOut") {
        return Err(
            "unsupported canonical vertex signature; expected vertex_index input".to_owned(),
        );
    }
    Ok(())
}

fn validate_fullscreen_vertex_body(body: &str) -> Result<(), String> {
    let canonical = concat!(
        "varout:VsOut;",
        "letx:f32=f32((vid<<1u)&2u);",
        "lety:f32=f32(vid&2u);",
        "out.pos=vec4<f32>(x*2.0-1.0,y*-2.0+1.0,0.0,1.0);",
        "out.uv=vec2<f32>(x,y);returnout;"
    );
    let observed = body
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    if observed != canonical {
        return Err(
            "unsupported noncanonical WGSL vertex body; refusing to substitute fullscreen geometry"
                .to_owned(),
        );
    }
    Ok(())
}

fn lower_fragment_body(body: &str) -> Result<Vec<String>, String> {
    if body.contains('{') || body.contains('}') {
        return Err("canonical WGSL fragment body must not contain nested blocks".to_owned());
    }
    let mut lowered = Vec::new();
    let mut returned = false;
    for raw in body.split(';') {
        let statement = raw.trim();
        if statement.is_empty() {
            continue;
        }
        if let Some(binding) = statement.strip_prefix("let ") {
            lowered.push(lower_float_binding(binding)?);
        } else if let Some(value) = statement.strip_prefix("return ") {
            if returned {
                return Err("canonical WGSL fragment body has multiple returns".to_owned());
            }
            let value = lower_fragment_return(value)?;
            lowered.push(format!("return {value};"));
            returned = true;
        } else {
            return Err(format!(
                "unsupported canonical WGSL fragment statement `{statement}`"
            ));
        }
    }
    if !returned {
        return Err("canonical WGSL fragment body must return one vec4<f32>".to_owned());
    }
    Ok(lowered)
}

fn lower_float_binding(binding: &str) -> Result<String, String> {
    let (declaration, expression) = binding
        .split_once('=')
        .ok_or_else(|| "canonical WGSL fragment let binding must contain `=`".to_owned())?;
    let (name, ty) = declaration
        .split_once(':')
        .ok_or_else(|| "canonical WGSL fragment let binding must declare a type".to_owned())?;
    let name = name.trim();
    if !valid_ident(name) || ty.trim() != "f32" {
        return Err(format!(
            "canonical WGSL fragment binding `{}` must be a named f32",
            declaration.trim()
        ));
    }
    let expression = lower_scalar_expression(expression.trim())?;
    Ok(format!("float {name} = {expression};"))
}

fn lower_fragment_return(value: &str) -> Result<String, String> {
    let arguments = value
        .trim()
        .strip_prefix("vec4<f32>(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| "canonical WGSL fragment return must be vec4<f32>".to_owned())?;
    let components = split_top_level(arguments, ',')?;
    if components.len() != 4 {
        return Err("canonical WGSL fragment return must have four components".to_owned());
    }
    let components = components
        .into_iter()
        .map(lower_scalar_expression)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("float4({})", components.join(", ")))
}

fn lower_scalar_expression(expression: &str) -> Result<String, String> {
    if expression.is_empty()
        || expression.chars().any(|ch| {
            !(ch.is_ascii_alphanumeric()
                || matches!(ch, '_' | '.' | '+' | '-' | '*' | '/' | '(' | ')' | ' '))
        })
    {
        return Err(format!(
            "unsupported canonical WGSL scalar expression `{expression}`"
        ));
    }
    Ok(expression.replace("f32(", "float("))
}

fn split_top_level(source: &str, separator: char) -> Result<Vec<&str>, String> {
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut values = Vec::new();
    for (index, ch) in source.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| "canonical WGSL expression has unmatched `)`".to_owned())?;
            }
            _ if ch == separator && depth == 0 => {
                values.push(source[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("canonical WGSL expression has unmatched `(`".to_owned());
    }
    values.push(source[start..].trim());
    Ok(values)
}

fn valid_ident(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn render_msl(
    profile_entry: &str,
    target: &str,
    vertex_entry: &str,
    fragment_entry: &str,
    fragment_statements: &[String],
    uniform: Option<(u32, &str)>,
) -> String {
    let (uniform_metadata, uniform_parameter) = match uniform {
        Some((slot, name)) => (
            format!(
                "{}{slot}\n",
                yir_domain_shader::FRAGMENT_UNIFORM_CAPABILITY_MARKER
            ),
            format!(", constant float4& {name} [[buffer({slot})]]"),
        ),
        None => (String::new(), String::new()),
    };
    let fragment_body = fragment_statements
        .iter()
        .map(|statement| format!("    {statement}\n"))
        .collect::<String>();
    format!(
        "// nuis-module-lowering-plan contract=nuis-yir.shader.backend-lowering-plan.v1\n\
         // nuis-module-profile-entry {profile_entry}\n\
         // nuis-module-profile-lowering-target {target}\n\
         // nuis-module-lowering-target msl:metal-gpu\n\
         // nuis-module-native-ir msl2.4\n\
         {uniform_metadata}#include <metal_stdlib>\n\
         using namespace metal;\n\
         \n\
         struct NuisRasterOut {{\n\
             float4 position [[position]];\n\
             float2 uv [[user(locn0)]];\n\
         }};\n\
         \n\
         vertex NuisRasterOut {vertex_entry}(uint vid [[vertex_id]]) {{\n\
             NuisRasterOut out;\n\
             float x = float((vid << 1u) & 2u);\n\
             float y = float(vid & 2u);\n\
             out.position = float4(x * 2.0 - 1.0, y * -2.0 + 1.0, 0.0, 1.0);\n\
             out.uv = float2(x, y);\n\
             return out;\n\
         }}\n\
         \n\
         fragment float4 {fragment_entry}(NuisRasterOut input [[stage_in]]{uniform_parameter}) {{\n\
             float2 uv = input.uv;\n\
{fragment_body}\
         }}\n"
    )
}

#[cfg(test)]
#[path = "shader_msl_render_emitter_tests.rs"]
mod tests;
