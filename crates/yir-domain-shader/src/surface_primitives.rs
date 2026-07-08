use crate::geometry_overlay::stamp_line;

pub(crate) fn sphere_palette(color: i64) -> &'static [char] {
    match color.rem_euclid(3) {
        0 => &[':', '-', '=', '+', '*', 'o'],
        1 => &[':', '-', '=', '+', '*', 'O'],
        _ => &[':', '-', '=', '+', '*', '@'],
    }
}

pub(crate) fn control_panel_accent(color: i64, contrast: i64) -> char {
    match (color + contrast).rem_euclid(5) {
        0 => '#',
        1 => '*',
        2 => '@',
        3 => '+',
        _ => '%',
    }
}

pub(crate) fn draw_slider(
    rows: &mut [Vec<char>],
    left: usize,
    y: usize,
    width: usize,
    value: usize,
    accent: char,
) {
    if y >= rows.len() || left >= rows[y].len() || width < 4 {
        return;
    }
    let right = left.saturating_add(width.saturating_sub(1));
    if right >= rows[y].len() {
        return;
    }
    rows[y][left] = '[';
    rows[y][right] = ']';
    let row = &mut rows[y];
    for cell in row.iter_mut().take(right).skip(left + 1) {
        *cell = '-';
    }
    let inner = right.saturating_sub(left + 1).max(1);
    let fill = (value.min(127) * inner) / 127;
    for cell in row.iter_mut().skip(left + 1).take(fill.min(inner)) {
        *cell = '=';
    }
    let knob_x = left + 1 + fill.min(inner.saturating_sub(1));
    row[knob_x] = accent;
}

pub(crate) fn draw_knob(
    rows: &mut [Vec<char>],
    cx: usize,
    cy: usize,
    radius: usize,
    value: usize,
    accent: char,
) {
    if rows.is_empty() || radius == 0 {
        return;
    }
    let angle =
        (value.min(127) as f32 / 127.0) * std::f32::consts::PI * 1.5 + std::f32::consts::PI * 0.75;
    let needle_x = cx as f32 + angle.cos() * radius as f32 * 0.7;
    let needle_y = cy as f32 + angle.sin() * radius as f32 * 0.7;
    for y in cy.saturating_sub(radius)..=(cy + radius).min(rows.len().saturating_sub(1)) {
        for x in cx.saturating_sub(radius)..=(cx + radius).min(rows[y].len().saturating_sub(1)) {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;
            let dist2 = dx * dx + dy * dy;
            let r2 = (radius as isize) * (radius as isize);
            if dist2 <= r2 && dist2 >= r2.saturating_sub(radius as isize * 2) {
                rows[y][x] = 'o';
            }
        }
    }
    if cy < rows.len() && cx < rows[cy].len() {
        rows[cy][cx] = accent;
    }
    let nx = needle_x
        .round()
        .clamp(0.0, (rows[0].len().saturating_sub(1)) as f32) as usize;
    let ny = needle_y
        .round()
        .clamp(0.0, (rows.len().saturating_sub(1)) as f32) as usize;
    stamp_line(rows, cx, cy, nx, ny, accent);
    if ny < rows.len() && nx < rows[ny].len() {
        rows[ny][nx] = accent;
    }
}

pub(crate) fn fill_panel_background(rows: &mut [Vec<char>], surface: i64, contrast: i64) {
    let palette = match (surface + contrast).rem_euclid(5) {
        0 => [' ', '.', '.', ':'],
        1 => [' ', '.', ':', '*'],
        2 => [' ', '.', '`', '+'],
        3 => [' ', '.', '.', '='],
        _ => [' ', '·', '.', ':'],
    };
    for (y, row) in rows.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let band = ((x / 7) + (y / 3)) % palette.len();
            *cell = palette[band];
        }
    }
}

pub(crate) fn draw_card(
    rows: &mut [Vec<char>],
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    accent: char,
    fill: char,
) {
    if right <= left + 1 || bottom <= top + 1 {
        return;
    }
    draw_box(
        rows,
        left,
        top,
        right,
        bottom,
        BoxGlyphs::new('.', '.', '\'', '\'', '-', '|'),
    );
    fill_rect(
        rows,
        left + 1,
        top + 1,
        right.saturating_sub(1),
        bottom.saturating_sub(1),
        fill,
    );
    if top < rows.len() && left + 2 < rows[top].len() {
        rows[top][left + 2] = accent;
    }
    if top < rows.len() && right >= 2 && right - 2 < rows[top].len() {
        rows[top][right - 2] = accent;
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BoxGlyphs {
    tl: char,
    tr: char,
    br: char,
    bl: char,
    horizontal: char,
    vertical: char,
}

impl BoxGlyphs {
    pub(crate) fn new(
        tl: char,
        tr: char,
        br: char,
        bl: char,
        horizontal: char,
        vertical: char,
    ) -> Self {
        Self {
            tl,
            tr,
            br,
            bl,
            horizontal,
            vertical,
        }
    }
}

pub(crate) fn draw_box(
    rows: &mut [Vec<char>],
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    glyphs: BoxGlyphs,
) {
    if rows.is_empty() || top >= rows.len() || bottom >= rows.len() || left >= right {
        return;
    }
    for x in left..=right.min(rows[top].len().saturating_sub(1)) {
        rows[top][x] = glyphs.horizontal;
        rows[bottom][x] = glyphs.horizontal;
    }
    for row in rows.iter_mut().take(bottom + 1).skip(top) {
        if left < row.len() {
            row[left] = glyphs.vertical;
        }
        if right < row.len() {
            row[right] = glyphs.vertical;
        }
    }
    if left < rows[top].len() {
        rows[top][left] = glyphs.tl;
    }
    if right < rows[top].len() {
        rows[top][right] = glyphs.tr;
    }
    if right < rows[bottom].len() {
        rows[bottom][right] = glyphs.br;
    }
    if left < rows[bottom].len() {
        rows[bottom][left] = glyphs.bl;
    }
}

pub(crate) fn fill_rect(
    rows: &mut [Vec<char>],
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    fill: char,
) {
    if rows.is_empty() {
        return;
    }
    for y in top..=bottom.min(rows.len().saturating_sub(1)) {
        for x in left..=right.min(rows[y].len().saturating_sub(1)) {
            rows[y][x] = fill;
        }
    }
}

pub(crate) fn put_text(rows: &mut [Vec<char>], left: usize, y: usize, text: &str) {
    if y >= rows.len() {
        return;
    }
    for (offset, ch) in text.chars().enumerate() {
        let x = left + offset;
        if x >= rows[y].len() {
            break;
        }
        rows[y][x] = ch;
    }
}
