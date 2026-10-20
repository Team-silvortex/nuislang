use crate::shader_codegen::syntax::{
    collect_function_calls, find_matching_paren, is_identifier_char,
};
use crate::shader_codegen::ShaderTargetFlavor;

pub(super) fn hoist_repeated_texture_samples(
    expr: &str,
    flavor: ShaderTargetFlavor,
) -> (Vec<String>, String) {
    let sample_prefix = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "texture(u_texture0, ",
        ShaderTargetFlavor::Metal => "tex.sample(samp, ",
        ShaderTargetFlavor::Hlsl => "shaderTexture.Sample(shaderSampler, ",
    };
    let sample_type = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
    };

    let mut rewritten = expr.to_owned();
    let mut prelude = Vec::new();
    let mut slot = 0usize;
    loop {
        let calls = collect_function_calls(&rewritten, sample_prefix);
        let Some(first) = calls.first() else {
            break;
        };
        let target = first.text.clone();
        let count = calls.iter().filter(|call| call.text == target).count();
        if count <= 1 {
            break;
        }
        let temp_name = format!("nuis_sample_{slot}");
        slot += 1;
        prelude.push(format!("    {sample_type} {temp_name} = {target};"));
        rewritten = rewritten.replace(&target, &temp_name);
    }
    (prelude, rewritten)
}

pub(super) fn hoist_repeated_math_subexpressions(
    mut prelude: Vec<String>,
    expr: String,
    flavor: ShaderTargetFlavor,
) -> (Vec<String>, String) {
    let vec2_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
    };

    let mut rewritten = expr;
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if let Some(call) =
        canonical_uv_clamp_expr(flavor).filter(|call| joined.matches(call).count() >= 2)
    {
        let temp = "nuis_uv_0";
        prelude = prelude
            .into_iter()
            .map(|line| {
                line.replace(&format!("({call})"), temp)
                    .replace(&call, temp)
            })
            .collect::<Vec<_>>();
        prelude.insert(0, format!("    {vec2_ty} {temp} = {};", call));
        prelude[0] = format!("    {vec2_ty} {temp} = {};", call);
        rewritten = rewritten.replace(&call, temp);
    }

    if let Some(call) =
        canonical_wave_expr(flavor).filter(|call| rewritten.matches(call).count() >= 2)
    {
        let temp = "nuis_wave_0";
        prelude.push(format!("    {vec2_ty} {temp} = {};", call));
        rewritten = rewritten.replace(&call, temp);
    }

    (prelude, rewritten)
}

fn canonical_uv_clamp_expr(flavor: ShaderTargetFlavor) -> Option<String> {
    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            "clamp(v_uv, vec2(0.0, 0.0), vec2(1.0, 1.0))".to_owned()
        }
        ShaderTargetFlavor::Metal => "clamp(in.uv, float2(0.0, 0.0), float2(1.0, 1.0))".to_owned(),
        ShaderTargetFlavor::Hlsl => {
            "clamp(input.uv, float2(0.0, 0.0), float2(1.0, 1.0))".to_owned()
        }
    })
}

fn canonical_wave_expr(flavor: ShaderTargetFlavor) -> Option<String> {
    let uv_temp = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "nuis_uv_0",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "nuis_uv_0",
    };
    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            format!("fract({uv_temp} * 3.0)")
        }
        ShaderTargetFlavor::Metal => format!("fract({uv_temp} * 3.0)"),
        ShaderTargetFlavor::Hlsl => format!("frac({uv_temp} * 3.0)"),
    })
}

pub(super) fn hoist_normalized_math_subexpressions(
    prelude: Vec<String>,
    expr: String,
    flavor: ShaderTargetFlavor,
) -> (Vec<String>, String) {
    let vec2_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
    };
    let vec3_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
    };
    let scalar_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "float",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float",
    };

    let mut prelude = prelude;
    let mut rewritten = expr;
    let wave_expr = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            "fract(nuis_uv_0 * 3.0)".to_owned()
        }
        ShaderTargetFlavor::Metal => "fract(nuis_uv_0 * 3.0)".to_owned(),
        ShaderTargetFlavor::Hlsl => "frac(nuis_uv_0 * 3.0)".to_owned(),
    };
    let wave_xy_expr = format!("({wave_expr}).xy");
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if joined.matches(&wave_xy_expr).count() >= 2 {
        let temp = "nuis_wave_0";
        prelude = prelude
            .into_iter()
            .map(|line| line.replace(&wave_xy_expr, &format!("{temp}.xy")))
            .collect::<Vec<_>>();
        rewritten = rewritten.replace(&wave_xy_expr, &format!("{temp}.xy"));
        prelude.push(format!("    {vec2_ty} {temp} = {wave_expr};"));
    }

    let light_expr = canonical_light_expr(flavor);
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if joined.matches(&light_expr).count() >= 1 {
        let temp = "nuis_light_0";
        prelude = prelude
            .into_iter()
            .map(|line| line.replace(&light_expr, temp))
            .collect::<Vec<_>>();
        rewritten = rewritten.replace(&light_expr, temp);
        prelude.push(format!("    {scalar_ty} {temp} = {light_expr};"));
    }

    let tint_expr = canonical_tint_expr(flavor);
    let tint_xyz_expr = format!("({tint_expr}).xyz");
    let joined = prelude.join("\n") + "\n" + &rewritten;
    if joined.matches(&tint_xyz_expr).count() >= 1 {
        let temp = "nuis_tint_0";
        prelude = prelude
            .into_iter()
            .map(|line| line.replace(&tint_xyz_expr, &format!("{temp}.xyz")))
            .collect::<Vec<_>>();
        rewritten = rewritten.replace(&tint_xyz_expr, &format!("{temp}.xyz"));
        prelude.push(format!("    {vec3_ty} {temp} = {tint_expr};"));
    }

    (prelude, rewritten)
}

fn canonical_light_expr(flavor: ShaderTargetFlavor) -> String {
    let vec3_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
    };
    format!(
        "smoothstep(0.15, 0.95, dot((normalize({vec3_ty}(nuis_wave_0.xy, 0.5))), (normalize({vec3_ty}(0.4, 0.6, 0.7)))))"
    )
}

fn canonical_tint_expr(flavor: ShaderTargetFlavor) -> String {
    let vec3_ty = match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
    };
    format!(
        "clamp(nuis_sample_0.xyz * nuis_light_0 + {vec3_ty}(nuis_wave_0.xy, 1.0 - nuis_uv_0.x) * 0.25, {vec3_ty}(0.0, 0.0, 0.0), {vec3_ty}(1.0, 1.0, 1.0))"
    )
}

pub(super) fn normalize_backend_expr(expr: &str) -> String {
    let mut normalized = expr.to_owned();
    for _ in 0..8 {
        let next = normalize_backend_expr_once(&normalized);
        if next == normalized {
            break;
        }
        normalized = next;
    }
    normalized
}

fn normalize_backend_expr_once(expr: &str) -> String {
    let mut out = expr.to_owned();
    out = strip_parens_around_member_bases(&out);
    out = strip_parens_around_simple_atoms(&out);
    out = remove_redundant_vec_ctor_swizzles(&out);
    out = remove_wrapped_ctor_swizzles(&out);
    out = remove_redundant_fn_result_swizzles(&out, "mix");
    out = remove_redundant_fn_result_swizzles(&out, "lerp");
    out = remove_wrapped_fn_result_swizzles(&out, "mix");
    out = remove_wrapped_fn_result_swizzles(&out, "lerp");
    out
}

fn strip_parens_around_member_bases(expr: &str) -> String {
    let chars = expr.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(expr.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '(' {
            let mut j = i + 1;
            let mut seen = false;
            while j < chars.len() && is_simple_member_char(chars[j]) {
                seen = true;
                j += 1;
            }
            if seen
                && j < chars.len()
                && chars[j] == ')'
                && j + 1 < chars.len()
                && chars[j + 1] == '.'
            {
                for ch in &chars[i + 1..j] {
                    out.push(*ch);
                }
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn is_simple_member_char(ch: char) -> bool {
    is_identifier_char(ch) || ch == '.'
}

fn strip_parens_around_simple_atoms(expr: &str) -> String {
    let chars = expr.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(expr.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '(' {
            let mut j = i + 1;
            let mut seen = false;
            while j < chars.len() && is_simple_member_char(chars[j]) {
                seen = true;
                j += 1;
            }
            if seen && j < chars.len() && chars[j] == ')' {
                for ch in &chars[i + 1..j] {
                    out.push(*ch);
                }
                i = j + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn remove_redundant_vec_ctor_swizzles(expr: &str) -> String {
    let mut out = expr.to_owned();
    for ctor in ["vec2", "vec3", "vec4", "float2", "float3", "float4"] {
        let full = format!("{ctor}(");
        while let Some(start) = out.find(&full) {
            let open = start + ctor.len();
            let Some(end) = find_matching_paren(&out, open) else {
                break;
            };
            let tail = &out[end + 1..];
            let redundant = match ctor {
                "vec2" | "float2" => tail.starts_with(".xy"),
                "vec3" | "float3" => tail.starts_with(".xyz"),
                "vec4" | "float4" => tail.starts_with(".xyzw"),
                _ => false,
            };
            if redundant {
                out.replace_range(end + 1..end + 1 + redundant_swizzle_len(ctor), "");
                continue;
            }
            break;
        }
    }
    out
}

fn remove_wrapped_ctor_swizzles(expr: &str) -> String {
    let mut out = expr.to_owned();
    for ctor in ["vec2", "vec3", "vec4", "float2", "float3", "float4"] {
        out = remove_wrapped_call_swizzle(&out, ctor);
    }
    out
}

fn remove_redundant_fn_result_swizzles(expr: &str, fn_name: &str) -> String {
    let mut out = expr.to_owned();
    let needle = format!("{fn_name}(");
    while let Some(start) = out.find(&needle) {
        let open = start + fn_name.len();
        let Some(end) = find_matching_paren(&out, open) else {
            break;
        };
        if out[end + 1..].starts_with(".xyz") {
            out.replace_range(end + 1..end + 5, "");
            continue;
        }
        break;
    }
    out
}

fn remove_wrapped_fn_result_swizzles(expr: &str, fn_name: &str) -> String {
    remove_wrapped_call_swizzle(expr, fn_name)
}

fn remove_wrapped_call_swizzle(expr: &str, name: &str) -> String {
    let mut out = expr.to_owned();
    let needle = format!("({name}(");
    while let Some(start) = out.find(&needle) {
        let open = start + 1 + name.len();
        let Some(end) = find_matching_paren(&out, open) else {
            break;
        };
        let swizzle = if out[end + 1..].starts_with(").xyzw") {
            Some((".xyzw", 6usize))
        } else if out[end + 1..].starts_with(").xyz") {
            Some((".xyz", 5usize))
        } else if out[end + 1..].starts_with(").xy") {
            Some((".xy", 4usize))
        } else {
            None
        };
        let Some((_, remove_len)) = swizzle else {
            break;
        };
        let replacement = out[start + 1..end + 1].to_owned();
        out.replace_range(start..end + 1 + remove_len, &replacement);
    }
    out
}

fn redundant_swizzle_len(ctor: &str) -> usize {
    match ctor {
        "vec2" | "float2" => 3,
        "vec3" | "float3" => 4,
        "vec4" | "float4" => 5,
        _ => 0,
    }
}
