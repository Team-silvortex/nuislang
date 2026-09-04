use yir_core::{FrameSurface, SurfaceTarget};

pub(crate) fn clear_target_surface(target: &SurfaceTarget, fill: i64) -> FrameSurface {
    let width = target.width.max(1);
    let height = target.height.max(1);
    let glyph = match fill.rem_euclid(5) {
        0 => '.',
        1 => ':',
        2 => '-',
        3 => '=',
        _ => ' ',
    };
    let row = std::iter::repeat_n(glyph, width).collect::<String>();
    FrameSurface {
        width,
        height,
        rows: vec![row; height],
        rgba8: None,
    }
}

pub(crate) fn overlay_surfaces(
    base: &FrameSurface,
    top: &FrameSurface,
) -> Result<FrameSurface, String> {
    if base.width != top.width || base.height != top.height {
        return Err(format!(
            "shader.overlay expects matching frame dimensions, got {}x{} and {}x{}",
            base.width, base.height, top.width, top.height
        ));
    }
    if base.rgba8.is_some() || top.rgba8.is_some() {
        return Err("shader.overlay does not yet support RGBA8 frame surfaces".to_owned());
    }

    let rows = base
        .rows
        .iter()
        .zip(&top.rows)
        .map(|(base_row, top_row)| {
            base_row
                .chars()
                .zip(top_row.chars())
                .map(|(base_char, top_char)| if top_char != '.' { top_char } else { base_char })
                .collect::<String>()
        })
        .collect();

    Ok(FrameSurface {
        width: base.width,
        height: base.height,
        rows,
        rgba8: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_rejects_rgba8_frames_instead_of_returning_empty_rows() {
        let frame = FrameSurface::from_rgba8(1, 1, vec![1, 2, 3, 255]).unwrap();

        let error = overlay_surfaces(&frame, &frame).unwrap_err();

        assert_eq!(
            error,
            "shader.overlay does not yet support RGBA8 frame surfaces"
        );
    }
}
