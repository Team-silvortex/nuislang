use yir_core::FrameSurface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

impl RgbImage {
    pub fn to_ppm(&self) -> Vec<u8> {
        let mut output = format!("P6\n{} {}\n255\n", self.width, self.height).into_bytes();
        output.extend_from_slice(&self.pixels);
        output
    }
}

pub fn rasterize_frame(frame: &FrameSurface, scale: usize) -> RgbImage {
    let scale = scale.max(1);
    let width = frame.width * scale;
    let height = frame.height * scale;
    let mut pixels = Vec::with_capacity(width * height * 3);

    if let Some(rgba8) = &frame.rgba8 {
        rasterize_rgba8(frame, rgba8, scale, &mut pixels);
    } else {
        rasterize_glyphs(frame, scale, &mut pixels);
    }

    RgbImage {
        width,
        height,
        pixels,
    }
}

fn rasterize_rgba8(frame: &FrameSurface, rgba8: &[u8], scale: usize, pixels: &mut Vec<u8>) {
    for row in rgba8.chunks_exact(frame.width * 4) {
        for _ in 0..scale {
            for rgba in row.chunks_exact(4) {
                for _ in 0..scale {
                    pixels.extend_from_slice(&rgba[..3]);
                }
            }
        }
    }
}

fn rasterize_glyphs(frame: &FrameSurface, scale: usize, pixels: &mut Vec<u8>) {
    for row in &frame.rows {
        for _ in 0..scale {
            for glyph in row.chars() {
                let rgb = glyph_rgb(glyph);
                for _ in 0..scale {
                    pixels.extend_from_slice(&rgb);
                }
            }
        }
    }
}

fn glyph_rgb(glyph: char) -> [u8; 3] {
    match glyph {
        '.' => [12, 14, 20],
        ':' => [28, 34, 48],
        '-' => [52, 66, 92],
        '=' => [86, 112, 150],
        '+' => [132, 164, 205],
        '*' => [180, 206, 240],
        'o' => [120, 210, 255],
        'O' => [255, 200, 80],
        '@' => [255, 120, 160],
        _ => [220, 220, 220],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterizes_rgba8_without_falling_back_to_glyph_rows() {
        let frame = FrameSurface::from_rgba8(2, 1, vec![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

        let image = rasterize_frame(&frame, 2);

        assert_eq!(image.width, 4);
        assert_eq!(image.height, 2);
        assert_eq!(
            image.pixels,
            vec![1, 2, 3, 1, 2, 3, 5, 6, 7, 5, 6, 7, 1, 2, 3, 1, 2, 3, 5, 6, 7, 5, 6, 7,]
        );
    }
}
