use yir_core::{ExecutionState, Node, SamplerState, Texture2D, Value};

pub(crate) fn parse_texture_shape(node: &Node) -> Result<(usize, usize), String> {
    let width = node.op.args[1].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid width `{}`",
            node.name, node.op.args[1]
        )
    })?;
    let height = node.op.args[2].parse::<usize>().map_err(|_| {
        format!(
            "node `{}` has invalid height `{}`",
            node.name, node.op.args[2]
        )
    })?;
    if width == 0 || height == 0 {
        return Err(format!(
            "node `{}` texture shape must be non-zero",
            node.name
        ));
    }
    Ok((width, height))
}

pub(crate) fn parse_bool_flag(node: &Node, index: usize, label: &str) -> Result<bool, String> {
    let raw = node
        .op
        .args
        .get(index)
        .ok_or_else(|| format!("node `{}` missing {}", node.name, label))?;
    match raw.as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!(
            "node `{}` has invalid {} `{}`; expected 0 or 1",
            node.name, label, raw
        )),
    }
}

pub(crate) fn validate_texture_literal(node: &Node) -> Result<(), String> {
    let (width, height) = parse_texture_shape(node)?;
    let texels = parse_csv_ints(node, &node.op.args[3], "texture literal texel")?;
    if texels.len() != width * height {
        return Err(format!(
            "node `{}` expected {} texture texels, got {}",
            node.name,
            width * height,
            texels.len()
        ));
    }
    Ok(())
}

pub(crate) fn parse_texture_literal(node: &Node) -> Result<Texture2D, String> {
    let (width, height) = parse_texture_shape(node)?;
    let texels = parse_csv_ints(node, &node.op.args[3], "texture literal texel")?;
    if texels.len() != width * height {
        return Err(format!(
            "node `{}` expected {} texture texels, got {}",
            node.name,
            width * height,
            texels.len()
        ));
    }
    Ok(Texture2D {
        format: node.op.args[0].clone(),
        width,
        height,
        texels,
    })
}

pub(crate) fn parse_csv_ints(node: &Node, raw: &str, label: &str) -> Result<Vec<i64>, String> {
    raw.split(',')
        .map(|part| {
            let value = part.trim();
            value
                .parse::<i64>()
                .map_err(|_| format!("node `{}` has invalid {} `{value}`", node.name, label))
        })
        .collect()
}

pub(crate) fn parse_csv_indices(node: &Node, raw: &str) -> Result<Vec<usize>, String> {
    raw.split(',')
        .map(|part| {
            let value = part.trim();
            value
                .parse::<usize>()
                .map_err(|_| format!("node `{}` has invalid index literal `{value}`", node.name))
        })
        .collect()
}

pub(crate) fn sample_texture_nearest(
    texture: &Texture2D,
    sampler: &SamplerState,
    x: i64,
    y: i64,
) -> i64 {
    let address = sampler.address_mode.as_str();
    let ix = apply_address_mode(x, texture.width, address);
    let iy = apply_address_mode(y, texture.height, address);
    texture.texels[iy * texture.width + ix]
}

pub(crate) fn sample_texture_by_filter(
    texture: &Texture2D,
    sampler: &SamplerState,
    x: i64,
    y: i64,
) -> i64 {
    match sampler.filter.as_str() {
        "nearest" => sample_texture_nearest(texture, sampler, x, y),
        "linear" => {
            let u = texel_coord_to_normalized_1024(texture.width, x);
            let v = texel_coord_to_normalized_1024(texture.height, y);
            sample_texture_linear(texture, sampler, u, v)
        }
        _ => sample_texture_nearest(texture, sampler, x, y),
    }
}

pub(crate) fn sample_texture_uv_by_filter(
    texture: &Texture2D,
    sampler: &SamplerState,
    u_1024: i64,
    v_1024: i64,
) -> i64 {
    match sampler.filter.as_str() {
        "nearest" => {
            let (x, y) = normalized_uv_to_texel(texture, u_1024, v_1024);
            sample_texture_nearest(texture, sampler, x, y)
        }
        "linear" => sample_texture_linear(texture, sampler, u_1024, v_1024),
        _ => {
            let (x, y) = normalized_uv_to_texel(texture, u_1024, v_1024);
            sample_texture_nearest(texture, sampler, x, y)
        }
    }
}

pub(crate) fn sample_texture_linear(
    texture: &Texture2D,
    sampler: &SamplerState,
    u_1024: i64,
    v_1024: i64,
) -> i64 {
    let (base_x, frac_x) = normalized_uv_to_linear_coord(texture.width, u_1024);
    let (base_y, frac_y) = normalized_uv_to_linear_coord(texture.height, v_1024);
    let address = sampler.address_mode.as_str();

    let x0 = apply_address_mode(base_x, texture.width, address);
    let x1 = apply_address_mode(base_x + 1, texture.width, address);
    let y0 = apply_address_mode(base_y, texture.height, address);
    let y1 = apply_address_mode(base_y + 1, texture.height, address);

    let t00 = texture.texels[y0 * texture.width + x0];
    let t10 = texture.texels[y0 * texture.width + x1];
    let t01 = texture.texels[y1 * texture.width + x0];
    let t11 = texture.texels[y1 * texture.width + x1];

    let top = lerp_fixed(t00, t10, frac_x);
    let bottom = lerp_fixed(t01, t11, frac_x);
    lerp_fixed(top, bottom, frac_y)
}

pub(crate) fn texel_coord_to_normalized_1024(extent: usize, coord: i64) -> i64 {
    if extent <= 1 {
        return 0;
    }
    let max_index = extent.saturating_sub(1) as i64;
    let clamped = coord.clamp(0, max_index);
    ((clamped * 1024) + (max_index / 2)) / max_index.max(1)
}

pub(crate) fn apply_address_mode(coord: i64, extent: usize, address_mode: &str) -> usize {
    if extent == 0 {
        return 0;
    }
    match address_mode {
        "repeat" | "wrap" => coord.rem_euclid(extent as i64) as usize,
        _ => coord.clamp(0, extent.saturating_sub(1) as i64) as usize,
    }
}

pub(crate) fn expect_texture_value(
    state: &ExecutionState,
    name: &str,
    op: &str,
) -> Result<Texture2D, String> {
    match state.expect_value(name)?.clone() {
        Value::Texture(texture) => Ok(texture),
        other => Err(format!("{op} expects texture value, got {}", other)),
    }
}

pub(crate) fn expect_sampler_value(
    state: &ExecutionState,
    name: &str,
    op: &str,
) -> Result<SamplerState, String> {
    match state.expect_value(name)?.clone() {
        Value::Sampler(sampler) => Ok(sampler),
        other => Err(format!("{op} expects sampler value, got {}", other)),
    }
}

pub(crate) fn expect_uv_value(
    state: &ExecutionState,
    name: &str,
    op: &str,
) -> Result<(i64, i64), String> {
    match state.expect_value(name)?.clone() {
        Value::Tuple(values) if values.len() == 2 => match (&values[0], &values[1]) {
            (Value::Int(u), Value::Int(v)) => Ok((*u, *v)),
            _ => Err(format!("{op} expects uv tuple `(int, int)`")),
        },
        other => Err(format!("{op} expects uv tuple, got {}", other)),
    }
}

pub(crate) fn normalized_uv_to_texel(texture: &Texture2D, u_1024: i64, v_1024: i64) -> (i64, i64) {
    (
        normalized_component_to_texel(texture.width, u_1024),
        normalized_component_to_texel(texture.height, v_1024),
    )
}

pub(crate) fn normalized_component_to_texel(extent: usize, value_1024: i64) -> i64 {
    if extent <= 1 {
        return 0;
    }
    ((value_1024 * (extent as i64 - 1)) + 512) / 1024
}

pub(crate) fn normalized_uv_to_linear_coord(extent: usize, value_1024: i64) -> (i64, i64) {
    if extent <= 1 {
        return (0, 0);
    }
    let scaled = value_1024 * (extent as i64 - 1);
    let base = scaled.div_euclid(1024);
    let frac = scaled.rem_euclid(1024);
    (base, frac)
}

pub(crate) fn lerp_fixed(a: i64, b: i64, t_1024: i64) -> i64 {
    ((a * (1024 - t_1024)) + (b * t_1024) + 512) / 1024
}
