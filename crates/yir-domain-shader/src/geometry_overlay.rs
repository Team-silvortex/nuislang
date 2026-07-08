use yir_core::{FrameSurface, IndexBuffer, ShaderBindingSet, Value, VertexBuffer, VertexLayout};

pub(crate) struct GeometryInputs {
    pub(crate) vertex_layout: VertexLayout,
    pub(crate) vertex_buffer: VertexBuffer,
    pub(crate) index_buffer: Option<IndexBuffer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VertexAttributeKind {
    Pos2,
    Color2,
    Uv2,
    Unknown,
}

pub(crate) fn resolve_geometry_inputs(
    bindings: &ShaderBindingSet,
) -> Result<GeometryInputs, String> {
    let vertex_layout = bindings
        .bindings
        .iter()
        .find(|binding| binding.kind == "vertex_layout_binding")
        .ok_or_else(|| "shader.draw_instanced bind_set is missing vertex_layout_binding".to_owned())
        .and_then(|binding| match binding.value.as_ref() {
            Value::VertexLayout(layout) => Ok(layout.clone()),
            other => Err(format!(
                "shader.draw_instanced expected vertex_layout binding, got {}",
                other
            )),
        })?;
    let vertex_buffer = bindings
        .bindings
        .iter()
        .find(|binding| binding.kind == "vertex_binding")
        .ok_or_else(|| "shader.draw_instanced bind_set is missing vertex_binding".to_owned())
        .and_then(|binding| match binding.value.as_ref() {
            Value::VertexBuffer(buffer) => Ok(buffer.clone()),
            other => Err(format!(
                "shader.draw_instanced expected vertex_buffer binding, got {}",
                other
            )),
        })?;

    let index_buffer = bindings
        .bindings
        .iter()
        .find(|binding| binding.kind == "index_binding")
        .map(|binding| match binding.value.as_ref() {
            Value::IndexBuffer(buffer) => Ok(buffer.clone()),
            other => Err(format!(
                "shader.draw_instanced expected index_buffer binding, got {}",
                other
            )),
        })
        .transpose()?;

    Ok(GeometryInputs {
        vertex_layout,
        vertex_buffer,
        index_buffer,
    })
}

pub(crate) fn render_geometry_overlay(
    frame: &mut FrameSurface,
    geometry: &GeometryInputs,
    vertex_count: usize,
    topology: &str,
) {
    if frame.rows.is_empty() || frame.width == 0 {
        return;
    }

    let attributes = geometry
        .vertex_layout
        .attributes
        .iter()
        .map(|attr| parse_vertex_attribute_kind(attr))
        .collect::<Vec<_>>();
    let referenced_vertices = referenced_vertex_indices(geometry, vertex_count);
    let mut rows = frame
        .rows
        .iter()
        .map(|row| row.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut samples = Vec::new();
    for vertex_index in referenced_vertices {
        if let Some(sample) = interpret_vertex(geometry, &attributes, vertex_index) {
            let x = sample_to_frame_coord(sample.x, frame.width);
            let y = sample_to_frame_coord(-sample.y, frame.height);
            stamp_vertex_marker(&mut rows, x, y, sample.glyph);
            samples.push((x, y, sample.glyph));
        }
    }
    draw_topology_edges(&mut rows, &samples, topology);

    for (x, y, glyph) in samples {
        stamp_vertex_marker(&mut rows, x, y, glyph);
    }

    for (row, chars) in frame.rows.iter_mut().zip(rows) {
        *row = chars.into_iter().collect();
    }
}

fn draw_topology_edges(rows: &mut [Vec<char>], samples: &[(usize, usize, char)], topology: &str) {
    match topology {
        "triangle_strip" => {
            for window in samples.windows(3) {
                let [(ax, ay, _), (bx, by, _), (cx, cy, _)] = [window[0], window[1], window[2]];
                stamp_triangle_fill(
                    rows,
                    TrianglePoints {
                        a: (ax, ay),
                        b: (bx, by),
                        c: (cx, cy),
                    },
                    ',',
                );
            }
            for window in samples.windows(2) {
                let [(ax, ay, _), (bx, by, _)] = [window[0], window[1]];
                stamp_line(rows, ax, ay, bx, by, '+');
            }
            for window in samples.windows(3) {
                let [(ax, ay, _), (_, _, _), (cx, cy, _)] = [window[0], window[1], window[2]];
                stamp_line(rows, ax, ay, cx, cy, '+');
            }
        }
        "triangle" => {
            for chunk in samples.chunks(3) {
                if chunk.len() == 3 {
                    let (ax, ay, _) = chunk[0];
                    let (bx, by, _) = chunk[1];
                    let (cx, cy, _) = chunk[2];
                    stamp_triangle_fill(
                        rows,
                        TrianglePoints {
                            a: (ax, ay),
                            b: (bx, by),
                            c: (cx, cy),
                        },
                        ',',
                    );
                    stamp_line(rows, ax, ay, bx, by, '+');
                    stamp_line(rows, bx, by, cx, cy, '+');
                    stamp_line(rows, cx, cy, ax, ay, '+');
                }
            }
        }
        _ => {
            for window in samples.windows(2) {
                let [(ax, ay, _), (bx, by, _)] = [window[0], window[1]];
                stamp_line(rows, ax, ay, bx, by, '+');
            }
        }
    }
}

fn referenced_vertex_indices(geometry: &GeometryInputs, vertex_count: usize) -> Vec<usize> {
    if let Some(index_buffer) = &geometry.index_buffer {
        index_buffer
            .indices
            .iter()
            .copied()
            .take(vertex_count)
            .collect()
    } else {
        (0..vertex_count.min(geometry.vertex_buffer.vertex_count)).collect()
    }
}

struct VertexSample {
    x: f32,
    y: f32,
    glyph: char,
}

fn interpret_vertex(
    geometry: &GeometryInputs,
    attributes: &[VertexAttributeKind],
    vertex_index: usize,
) -> Option<VertexSample> {
    if vertex_index >= geometry.vertex_buffer.vertex_count {
        return None;
    }
    let stride = geometry.vertex_layout.stride;
    let base = vertex_index.checked_mul(stride)?;
    if geometry.vertex_buffer.elements.len() < base + stride {
        return None;
    }
    let slice = &geometry.vertex_buffer.elements[base..base + stride];

    let mut cursor = 0usize;
    let mut pos = None;
    let mut color = None;
    let mut uv = None;
    for attr in attributes {
        match attr {
            VertexAttributeKind::Pos2 => {
                if cursor + 2 <= slice.len() {
                    pos = Some((slice[cursor] as f32, slice[cursor + 1] as f32));
                }
                cursor += 2;
            }
            VertexAttributeKind::Color2 => {
                if cursor + 2 <= slice.len() {
                    color = Some((slice[cursor] as f32, slice[cursor + 1] as f32));
                }
                cursor += 2;
            }
            VertexAttributeKind::Uv2 => {
                if cursor + 2 <= slice.len() {
                    uv = Some((slice[cursor] as f32, slice[cursor + 1] as f32));
                }
                cursor += 2;
            }
            VertexAttributeKind::Unknown => {
                cursor += 1;
            }
        }
    }

    let (x, y) = pos?;
    let glyph = if let Some((u, v)) = uv {
        if u + v >= 1.5 {
            'u'
        } else {
            'v'
        }
    } else if let Some((r, g)) = color {
        match ((r + g) * 0.5).round() as i64 {
            value if value <= 0 => '#',
            1 => '%',
            _ => '@',
        }
    } else {
        '#'
    };

    Some(VertexSample { x, y, glyph })
}

fn parse_vertex_attribute_kind(raw: &str) -> VertexAttributeKind {
    match raw.trim() {
        "pos2f" => VertexAttributeKind::Pos2,
        "color2f" => VertexAttributeKind::Color2,
        "uv2f" => VertexAttributeKind::Uv2,
        _ => VertexAttributeKind::Unknown,
    }
}

fn sample_to_frame_coord(value: f32, extent: usize) -> usize {
    if extent <= 1 {
        return 0;
    }
    let normalized = ((value.clamp(-1.0, 1.0) + 1.0) * 0.5) * (extent as f32 - 1.0);
    normalized.round() as usize
}

fn stamp_vertex_marker(rows: &mut [Vec<char>], x: usize, y: usize, glyph: char) {
    if rows.is_empty() {
        return;
    }
    let height = rows.len();
    let width = rows[0].len();
    let positions = [
        (x, y),
        (x.saturating_sub(1), y),
        ((x + 1).min(width.saturating_sub(1)), y),
        (x, y.saturating_sub(1)),
        (x, (y + 1).min(height.saturating_sub(1))),
    ];
    for (px, py) in positions {
        if let Some(row) = rows.get_mut(py) {
            if let Some(cell) = row.get_mut(px) {
                *cell = glyph;
            }
        }
    }
}

pub(crate) fn stamp_line(
    rows: &mut [Vec<char>],
    ax: usize,
    ay: usize,
    bx: usize,
    by: usize,
    glyph: char,
) {
    if rows.is_empty() {
        return;
    }

    let mut x0 = ax as isize;
    let mut y0 = ay as isize;
    let x1 = bx as isize;
    let y1 = by as isize;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if let Some(row) = rows.get_mut(y0.max(0) as usize) {
            if let Some(cell) = row.get_mut(x0.max(0) as usize) {
                if *cell == '.' {
                    *cell = glyph;
                }
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[derive(Clone, Copy)]
struct TrianglePoints {
    a: (usize, usize),
    b: (usize, usize),
    c: (usize, usize),
}

fn stamp_triangle_fill(rows: &mut [Vec<char>], points: TrianglePoints, glyph: char) {
    if rows.is_empty() || rows[0].is_empty() {
        return;
    }

    let (ax, ay) = points.a;
    let (bx, by) = points.b;
    let (cx, cy) = points.c;
    let min_x = ax.min(bx).min(cx);
    let max_x = ax.max(bx).max(cx).min(rows[0].len().saturating_sub(1));
    let min_y = ay.min(by).min(cy);
    let max_y = ay.max(by).max(cy).min(rows.len().saturating_sub(1));

    let axf = ax as f32;
    let ayf = ay as f32;
    let bxf = bx as f32;
    let byf = by as f32;
    let cxf = cx as f32;
    let cyf = cy as f32;

    let area = edge_function(axf, ayf, bxf, byf, cxf, cyf);
    if area.abs() < f32::EPSILON {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pxf = x as f32 + 0.5;
            let pyf = y as f32 + 0.5;
            let w0 = edge_function(bxf, byf, cxf, cyf, pxf, pyf);
            let w1 = edge_function(cxf, cyf, axf, ayf, pxf, pyf);
            let w2 = edge_function(axf, ayf, bxf, byf, pxf, pyf);
            let all_positive = w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0;
            let all_negative = w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0;
            if all_positive || all_negative {
                if let Some(row) = rows.get_mut(y) {
                    if let Some(cell) = row.get_mut(x) {
                        if *cell == '.' {
                            *cell = glyph;
                        }
                    }
                }
            }
        }
    }
}

fn edge_function(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}
