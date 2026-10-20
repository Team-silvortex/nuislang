use crate::shader_codegen::normalize::{
    hoist_normalized_math_subexpressions, hoist_repeated_math_subexpressions,
    hoist_repeated_texture_samples, normalize_backend_expr,
};
use crate::shader_codegen::syntax::replace_identifier_token;
use crate::shader_codegen::textures::{
    find_texture_sample_call, render_backend_texture_sample_call,
};
use crate::shader_codegen::ShaderTargetFlavor;

pub(crate) fn compile_backend_expr(
    expr: &str,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    let normalized_expr =
        map_wgsl_expr_identifiers(&apply_backend_builtin_mapping(expr, flavor), flavor);
    let mapped_expr = map_wgsl_texture_sample_expr(&normalized_expr, flavor, uses_texture)?;
    Some(normalize_backend_expr(&mapped_expr))
}

pub(super) fn map_wgsl_type_to_backend(ty: &str, flavor: ShaderTargetFlavor) -> String {
    ty.replace(
        "vec4<f32>",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
        },
    )
    .replace(
        "vec3<f32>",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
        },
    )
    .replace(
        "vec2<f32>",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
        },
    )
    .replace("f32", "float")
    .replace("u32", "uint")
}

pub(super) fn infer_backend_type(expr: &str, flavor: ShaderTargetFlavor) -> String {
    if expr.contains("vec4") || expr.contains("textureSample(") {
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4".to_owned(),
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4".to_owned(),
        }
    } else if expr.contains("vec3") || expr.contains(".xyz") {
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3".to_owned(),
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3".to_owned(),
        }
    } else if expr.contains("vec2") || expr.contains(".xy") {
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2".to_owned(),
            ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2".to_owned(),
        }
    } else {
        "float".to_owned()
    }
}

pub(crate) fn map_wgsl_expr_to_backend(
    expr: &str,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    let normalized_expr =
        map_wgsl_expr_identifiers(&apply_backend_builtin_mapping(expr, flavor), flavor);
    let mapped_expr = map_wgsl_texture_sample_expr(&normalized_expr, flavor, uses_texture)?;
    let (prelude_lines, final_expr) = hoist_repeated_texture_samples(&mapped_expr, flavor);
    let (prelude_lines, final_expr) =
        hoist_repeated_math_subexpressions(prelude_lines, final_expr, flavor);
    let prelude_lines = prelude_lines
        .into_iter()
        .map(|line| normalize_backend_expr(&line))
        .collect::<Vec<_>>();
    let final_expr = normalize_backend_expr(&final_expr);
    let (prelude_lines, final_expr) =
        hoist_normalized_math_subexpressions(prelude_lines, final_expr, flavor);

    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            if prelude_lines.is_empty() {
                format!("    outColor = {};", final_expr)
            } else {
                format!(
                    "{}\n    outColor = {};",
                    prelude_lines.join("\n"),
                    final_expr
                )
            }
        }
        ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => {
            if prelude_lines.is_empty() {
                format!("    return {};", final_expr)
            } else {
                format!("{}\n    return {};", prelude_lines.join("\n"), final_expr)
            }
        }
    })
}

pub(super) fn apply_backend_builtin_mapping(expr: &str, flavor: ShaderTargetFlavor) -> String {
    let mapped = expr
        .replace(
            "vec4<f32>",
            match flavor {
                ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec4",
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float4",
            },
        )
        .replace(
            "vec3<f32>",
            match flavor {
                ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec3",
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float3",
            },
        )
        .replace(
            "vec2<f32>",
            match flavor {
                ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "vec2",
                ShaderTargetFlavor::Metal | ShaderTargetFlavor::Hlsl => "float2",
            },
        )
        .replace("@location(0)", "");

    let mix_name = match flavor {
        ShaderTargetFlavor::Hlsl => "lerp",
        _ => "mix",
    };
    let fract_name = match flavor {
        ShaderTargetFlavor::Hlsl => "frac",
        _ => "fract",
    };

    let mapped = replace_identifier_token(&mapped, "mix", mix_name);
    replace_identifier_token(&mapped, "fract", fract_name)
}

pub(super) fn map_wgsl_expr_identifiers(expr: &str, flavor: ShaderTargetFlavor) -> String {
    replace_identifier_token(
        expr,
        "uv",
        match flavor {
            ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => "v_uv",
            ShaderTargetFlavor::Metal => "in.uv",
            ShaderTargetFlavor::Hlsl => "input.uv",
        },
    )
}

fn map_wgsl_texture_sample_expr(
    expr: &str,
    flavor: ShaderTargetFlavor,
    uses_texture: bool,
) -> Option<String> {
    if expr.contains("textureSample(") && !uses_texture {
        return None;
    }
    let mut mapped = expr.to_owned();
    while let Some(call) = find_texture_sample_call(&mapped) {
        let replacement = render_backend_texture_sample_call(&call.args, flavor)?;
        mapped.replace_range(call.start..call.end, &replacement);
    }
    Some(mapped)
}
