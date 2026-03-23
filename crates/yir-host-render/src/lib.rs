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

    RgbImage {
        width,
        height,
        pixels,
    }
}

fn glyph_rgb(glyph: char) -> [u8; 3] {
    match glyph {
        '.' => [12, 14, 20],
        'o' => [120, 210, 255],
        'O' => [255, 200, 80],
        '@' => [255, 120, 160],
        _ => [220, 220, 220],
    }
}
