use yir_core::{ApplicationSessionSignature, Value, YirFunctionParameter, YirModule};

use crate::{
    ApplicationEventPump, ApplicationProviderSource, ApplicationPumpOperation, ApplicationPumpPhase,
};

mod ffi;
pub use ffi::*;

pub const WINDOW_SESSION_CONTRACT: &str = "nuis-yir-window-session-v1";

/// Window-specific host adapter, not a YIR application or rendering policy.
/// The registered Nuis helpers own configuration, image transitions and clocks.
pub struct WindowSession {
    pump: ApplicationEventPump,
}

pub struct WindowSessionReply {
    pub operation: ApplicationPumpOperation,
    pub phase: ApplicationPumpPhase,
    pub frame: Result<Option<Vec<u8>>, String>,
}

impl WindowSession {
    pub fn spawn(
        source: String,
        provider: ApplicationProviderSource<'_>,
        id: String,
        width: i64,
        height: i64,
    ) -> Result<Self, String> {
        if !(1..=16384).contains(&width) || !(1..=16384).contains(&height) {
            return Err("window dimensions must be in 1..=16384".to_owned());
        }
        Ok(Self {
            pump: ApplicationEventPump::spawn_checked(
                source,
                provider,
                id,
                vec![Value::Int(width), Value::Int(height)],
                crate::application_event_pump::ApplicationPumpChecks {
                    preflight: validate_window_session,
                    trace: validate_window_trace,
                },
            )?,
        })
    }

    pub fn phase(&self) -> ApplicationPumpPhase {
        self.pump.phase()
    }

    pub fn pending(&self) -> bool {
        self.pump.pending().is_some()
    }

    pub fn event(&mut self, kind: i64, code: i64) -> Result<(), String> {
        match kind {
            0 if code == 0 => {}
            1 if u32::try_from(code).ok().and_then(char::from_u32).is_some() => {}
            _ => {
                return Err("window event requires redraw(0,0) or key(1,Unicode scalar)".to_owned())
            }
        }
        self.pump.event(vec![Value::Int(kind), Value::Int(code)])
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.pump.close(vec![])
    }

    pub fn poll(&mut self) -> Result<Option<WindowSessionReply>, String> {
        let Some(reply) = self.pump.poll()? else {
            return Ok(None);
        };
        let frame = reply.trace.and_then(|trace| {
            if trace.presented_frames.is_empty() {
                Ok(None)
            } else {
                crate::render_trace_to_ppm_bytes(&trace, 1).map(Some)
            }
        });
        // A host presentation-contract failure must never admit more work or a
        // successful close. Callback failures already carry the pump's fault phase.
        if frame.is_err()
            && matches!(
                reply.phase,
                ApplicationPumpPhase::Open | ApplicationPumpPhase::Closed
            )
        {
            self.pump.abort();
        }
        Ok(Some(WindowSessionReply {
            operation: reply.operation,
            phase: self.phase(),
            frame,
        }))
    }
}

fn validate_window_trace(
    operation: ApplicationPumpOperation,
    trace: &yir_exec::ExecutionTrace,
) -> Result<(), String> {
    if trace.presented_frames.len() > 1 {
        return Err("window callback presented more than one frame".to_owned());
    }
    if operation == ApplicationPumpOperation::Close && !trace.presented_frames.is_empty() {
        return Err("window close callback must not present a frame".to_owned());
    }
    for frame in &trace.presented_frames {
        let rgba8 = frame
            .rgba8
            .as_ref()
            .ok_or("window frame requires RGBA8 pixels, not a glyph surface")?;
        let bytes = frame
            .width
            .checked_mul(frame.height)
            .and_then(|size| size.checked_mul(4));
        if frame.width == 0
            || frame.height == 0
            || bytes.is_none_or(|bytes| bytes > 16777216)
            || Some(rgba8.len()) != bytes
        {
            return Err("window frame exceeds the valid 16MiB RGBA8 extent".to_owned());
        }
    }
    Ok(())
}

pub fn validate_window_session(module: &YirModule, id: &str) -> Result<(), String> {
    let registration = yir_core::registered_application_session(module, id)?;
    let signature = ApplicationSessionSignature::bind(module, registration.entries())?;
    let state_count = signature.state_parameters.len();
    for (parameters, names) in [
        (
            signature.open.parameters.as_slice(),
            &["width", "height"][..],
        ),
        (
            &signature.event.parameters[state_count..],
            &["kind", "code"][..],
        ),
        (&signature.close.parameters[state_count..], &[][..]),
    ] {
        validate_parameters(parameters, names)?;
    }
    Ok(())
}

fn validate_parameters(parameters: &[YirFunctionParameter], names: &[&str]) -> Result<(), String> {
    if parameters.len() != names.len()
        || parameters
            .iter()
            .zip(names)
            .any(|(parameter, name)| parameter.name != *name || parameter.ty != "i64")
    {
        return Err(format!(
            "{WINDOW_SESSION_CONTRACT} requires scalar i64 parameters {names:?}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::FrameSurface;
    use yir_exec::ExecutionTrace;

    #[test]
    fn window_trace_requires_exact_bounded_rgba8_without_glyph_fallback() {
        let valid = FrameSurface::from_rgba8(1, 1, vec![1, 2, 3, 255]).unwrap();
        let mut trace = ExecutionTrace {
            presented_frames: vec![valid],
            ..Default::default()
        };
        validate_window_trace(ApplicationPumpOperation::Event, &trace).unwrap();
        for (width, height, rgba8) in [
            (1, 1, None),
            (1, 1, Some(vec![1, 2, 3])),
            (0, 1, Some(vec![])),
            (1, 0, Some(vec![])),
            (usize::MAX, 2, Some(vec![])),
            (4096, 4096, Some(vec![])),
        ] {
            trace.presented_frames[0] = FrameSurface {
                width,
                height,
                rows: vec!["@".to_owned()],
                rgba8,
            };
            assert!(validate_window_trace(ApplicationPumpOperation::Event, &trace).is_err());
        }
    }
}
