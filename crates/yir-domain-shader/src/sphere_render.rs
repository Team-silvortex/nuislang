use super::parse_ball_packet;
use super::surface_primitives::sphere_palette;
use yir_core::{FrameSurface, Value};

pub(crate) fn draw_ball_surface(value: &Value) -> Result<FrameSurface, String> {
    let packet = parse_ball_packet(value, "shader.draw_ball")?;

    let width = 16usize;
    let height = 9usize;
    let speed = packet.speed;
    let center_x = (((speed).round() as i64).rem_euclid(width as i64)) as usize;
    let center_y = ((((speed / 2.0).round()) as i64).rem_euclid(height as i64)) as usize;
    let glyph = match packet.color_key.rem_euclid(3) {
        0 => 'o',
        1 => 'O',
        _ => '@',
    };

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        for x in 0..width {
            let dx = x.abs_diff(center_x);
            let dy = y.abs_diff(center_y);
            if dx <= 1 && dy <= 1 {
                row.push(glyph);
            } else {
                row.push('.');
            }
        }
        rows.push(row);
    }

    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}

pub(crate) fn draw_ball_surface_with_size(
    value: &Value,
    width: usize,
    height: usize,
) -> Result<FrameSurface, String> {
    let packet = parse_ball_packet(value, "shader.draw_ball")?;

    let width = width.max(8);
    let height = height.max(8);
    let radius = (0.72f32 * packet.radius_scale).clamp(0.18, 0.95);
    let offset_x = (packet.speed * 0.03).sin() * 0.22;
    let offset_y = (packet.speed * 0.02).cos() * 0.16;
    let palette = sphere_palette(packet.color_key);

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        let ny = ((y as f32 / (height - 1) as f32) * 2.0 - 1.0) - offset_y;
        for x in 0..width {
            let nx = ((x as f32 / (width - 1) as f32) * 2.0 - 1.0) - offset_x;
            let r2 = nx * nx + ny * ny;
            if r2 > radius * radius {
                row.push('.');
                continue;
            }

            let nz = (radius * radius - r2).sqrt();
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
            let lx = -0.45f32;
            let ly = -0.35f32;
            let lz = 0.82f32;
            let ll = (lx * lx + ly * ly + lz * lz).sqrt();
            let light =
                ((nx / len) * (lx / ll) + (ny / len) * (ly / ll) + (nz / len) * (lz / ll)).max(0.0);
            let rim = (1.0 - (nz / radius)).powf(1.6) * 0.35;
            let shade = (light * 0.85 + rim).clamp(0.0, 1.0);
            let index =
                ((shade * (palette.len() - 1) as f32).round() as usize).min(palette.len() - 1);
            row.push(palette[index]);
        }
        rows.push(row);
    }

    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}

pub(crate) fn draw_sphere_surface_with_size(
    value: &Value,
    width: usize,
    height: usize,
) -> Result<FrameSurface, String> {
    let width = width.max(8);
    let height = height.max(8);
    let packet = parse_ball_packet(value, "shader.draw_sphere")?;

    let radius = (0.72f32 * packet.radius_scale).clamp(0.18, 0.95);
    let offset_x = (packet.speed * 0.03).sin() * 0.22;
    let offset_y = (packet.speed * 0.02).cos() * 0.16;
    let palette = sphere_palette(packet.color_key);

    let mut rows = Vec::with_capacity(height);
    for y in 0..height {
        let mut row = String::with_capacity(width);
        let ny = ((y as f32 / (height - 1) as f32) * 2.0 - 1.0) - offset_y;
        for x in 0..width {
            let nx = ((x as f32 / (width - 1) as f32) * 2.0 - 1.0) - offset_x;
            let r2 = nx * nx + ny * ny;
            if r2 > radius * radius {
                row.push('.');
                continue;
            }

            let nz = (radius * radius - r2).sqrt();
            let len = (nx * nx + ny * ny + nz * nz).sqrt().max(0.0001);
            let lx = -0.45f32;
            let ly = -0.35f32;
            let lz = 0.82f32;
            let ll = (lx * lx + ly * ly + lz * lz).sqrt();
            let light =
                ((nx / len) * (lx / ll) + (ny / len) * (ly / ll) + (nz / len) * (lz / ll)).max(0.0);
            let rim = (1.0 - (nz / radius)).powf(1.6) * 0.35;
            let shade = (light * 0.85 + rim).clamp(0.0, 1.0);
            let index =
                ((shade * (palette.len() - 1) as f32).round() as usize).min(palette.len() - 1);
            row.push(palette[index]);
        }
        rows.push(row);
    }

    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}
