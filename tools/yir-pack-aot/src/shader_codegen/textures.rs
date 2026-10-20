use crate::shader_codegen::syntax::split_top_level_args;
use crate::shader_codegen::ShaderTargetFlavor;

pub(super) struct TextureSampleCall {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) args: Vec<String>,
}

pub(super) fn find_texture_sample_call(expr: &str) -> Option<TextureSampleCall> {
    let start = expr.find("textureSample(")?;
    let open = start + "textureSample".len();
    let mut depth = 0usize;
    let mut end = None;
    for (idx, ch) in expr[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(open + idx + ch.len_utf8());
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    let inner = &expr[open + 1..end - 1];
    Some(TextureSampleCall {
        start,
        end,
        args: split_top_level_args(inner),
    })
}

pub(super) fn render_backend_texture_sample_call(
    args: &[String],
    flavor: ShaderTargetFlavor,
) -> Option<String> {
    if args.len() != 3 {
        return None;
    }
    let coord = args[2].trim();
    Some(match flavor {
        ShaderTargetFlavor::OpenGl | ShaderTargetFlavor::VulkanGlsl => {
            format!("texture(u_texture0, {coord})")
        }
        ShaderTargetFlavor::Metal => format!("tex.sample(samp, {coord})"),
        ShaderTargetFlavor::Hlsl => format!("shaderTexture.Sample(shaderSampler, {coord})"),
    })
}
