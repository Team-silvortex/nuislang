use super::NativeResourceSession;
use std::path::PathBuf;
use yir_core::native_resource_session::{
    MixedResourceLayout, FRAME_EXPORT_CONTRACT, MAX_BUFFER_WORDS, MIXED_FRAME_EXPORT_CONTRACT,
};

/// Explicit presentation view over resource State, not a new callback ABI.
/// One i64 cell contains one unsigned 0xAABBGGRR pixel; PPM omits alpha.
#[derive(Debug)]
pub struct ResourceFrameExport {
    width: usize,
    height: usize,
    output: PathBuf,
    fields: Option<[String; 2]>,
}

impl ResourceFrameExport {
    pub fn parse(dimensions: &str, output: &str) -> Result<Self, String> {
        let (width, height) = dimensions
            .split_once('x')
            .ok_or("resource frame requires WIDTHxHEIGHT")?;
        let dimension = |value: &str| -> Result<usize, String> {
            let parsed = value
                .parse::<usize>()
                .map_err(|_| "invalid resource frame dimension")?;
            if parsed == 0 || parsed.to_string() != value {
                return Err("resource frame dimension must be canonical and positive".to_owned());
            }
            Ok(parsed)
        };
        let frame = Self {
            width: dimension(width)?,
            height: dimension(height)?,
            output: output.into(),
            fields: None,
        };
        if frame
            .width
            .checked_mul(frame.height)
            .is_none_or(|n| n > MAX_BUFFER_WORDS)
            || output.is_empty()
        {
            return Err(
                "resource frame exceeds its fixed Buffer bound or has no output path".to_owned(),
            );
        }
        Ok(frame)
    }

    fn validate_pixels(&self, pixels: &[i64]) -> Result<(), String> {
        if pixels.len() != self.width * self.height {
            return Err("resource frame dimensions do not match the complete Buffer".to_owned());
        }
        if pixels.iter().any(|&pixel| u32::try_from(pixel).is_err()) {
            return Err("resource frame requires unsigned packed RGBA8 cells".to_owned());
        }
        Ok(())
    }

    pub fn preflight(&self, initial: &[i64]) -> Result<(), String> {
        if self.fields.is_some() {
            return Err("mixed frame requires a registered State layout".to_owned());
        }
        self.validate_pixels(initial)?;
        crate::frame_export::require_new_frame_path(&self.output)
    }

    pub fn from_state_fields(fields: &str, output: &str) -> Result<Self, String> {
        let (width, height) = fields
            .split_once(',')
            .ok_or("mixed frame requires WIDTH_FIELD,HEIGHT_FIELD")?;
        yir_core::YirApplicationSession::validate_identifier(width)?;
        yir_core::YirApplicationSession::validate_identifier(height)?;
        if width == height || output.is_empty() {
            return Err("mixed frame requires distinct fields and an output path".to_owned());
        }
        Ok(Self {
            width: 0,
            height: 0,
            output: output.into(),
            fields: Some([width.to_owned(), height.to_owned()]),
        })
    }

    pub fn preflight_state(
        &self,
        initial: &[i64],
        layout: Option<&MixedResourceLayout>,
    ) -> Result<(), String> {
        match (&self.fields, layout) {
            (None, None) => self.preflight(initial),
            (Some(fields), Some(layout)) => {
                for field in fields {
                    layout.scalar_index(field)?;
                }
                if initial.is_empty()
                    || initial.len() > MAX_BUFFER_WORDS
                    || initial.iter().any(|&pixel| u32::try_from(pixel).is_err())
                {
                    return Err(
                        "mixed frame requires bounded unsigned packed RGBA8 cells".to_owned()
                    );
                }
                crate::frame_export::require_new_frame_path(&self.output)
            }
            _ => Err("frame extent source must match the resource State contract".to_owned()),
        }
    }

    fn ppm(&self, pixels: &[i64]) -> Result<Vec<u8>, String> {
        self.validate_pixels(pixels)?;
        let rgba = pixels
            .iter()
            .flat_map(|&pixel| (pixel as u32).to_le_bytes())
            .collect();
        let frame = yir_core::FrameSurface::from_rgba8(self.width, self.height, rgba)?;
        Ok(yir_host_render::rasterize_frame(&frame, 1).to_ppm())
    }

    pub fn write_closed(&self, session: &NativeResourceSession<'_>) -> Result<(), String> {
        session.completion_status()?;
        self.preflight_state(session.state(), session.mixed_layout())?;
        let resolved;
        let frame = if let Some(fields) = &self.fields {
            let layout = session.mixed_layout().ok_or("missing mixed State layout")?;
            let read = |name: &str| -> Result<i64, String> {
                session
                    .scalar_state()
                    .get(layout.scalar_index(name)?)
                    .copied()
                    .ok_or("incomplete mixed State".to_owned())
            };
            let dimensions = format!("{}x{}", read(&fields[0])?, read(&fields[1])?);
            resolved = Self::parse(
                &dimensions,
                self.output.to_str().ok_or("non-UTF-8 frame path")?,
            )?;
            &resolved
        } else {
            self
        };
        crate::frame_export::write_new_frame(&self.output, || frame.ppm(session.state()))?;
        let contract = if self.fields.is_some() {
            MIXED_FRAME_EXPORT_CONTRACT
        } else {
            FRAME_EXPORT_CONTRACT
        };
        eprintln!("nuis: native_resource_frame_contract={contract}");
        eprintln!("nuis: native_resource_frame_execution=native-cpu-resource-session");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_rgba_view_has_exact_extent_and_does_not_clamp_or_repeat_pixels() {
        let frame = ResourceFrameExport::parse("2x1", "unused.ppm").unwrap();
        assert_eq!(
            frame.ppm(&[0x04030201, 0xff060504]).unwrap(),
            b"P6\n2 1\n255\n\x01\x02\x03\x04\x05\x06"
        );
        for bad in [
            vec![0],
            vec![0, 0, 0],
            vec![-1, 0],
            vec![u32::MAX as i64 + 1, 0],
        ] {
            assert!(frame.ppm(&bad).is_err());
        }
        for dimensions in [
            "0x1",
            "1x0",
            "01x2",
            "+1x2",
            "4097x1",
            "2x3x4",
            "18446744073709551615x2",
        ] {
            assert!(ResourceFrameExport::parse(dimensions, "unused.ppm").is_err());
        }
        assert!(ResourceFrameExport::parse("64x64", "unused.ppm").is_ok());
        assert!(ResourceFrameExport::parse("1x1", "").is_err());
    }
}
