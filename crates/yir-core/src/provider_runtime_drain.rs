use super::{number, DispatchTarget, MAX_DISPATCHES};

pub const SESSION_DRAIN_CONTRACT: &str = "nuis-yir-provider-session-drain-v1";

/// A terminal request/receipt bound to one admitted connection's target and
/// completed dispatch count. Not an application outcome or transferable lease.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionDrain {
    pub sequence: usize,
    pub target: DispatchTarget,
}

impl SessionDrain {
    pub fn admit(&self, target: &DispatchTarget, sequence: usize) -> Result<(), String> {
        self.validate()?;
        if self.target != *target || self.sequence != sequence {
            return Err("runtime IPC drain target or sequence mismatch".to_owned());
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), String> {
        DispatchTarget::parse(&self.target.fields())?;
        if self.sequence > MAX_DISPATCHES {
            return Err("runtime IPC drain sequence exceeds budget".to_owned());
        }
        Ok(())
    }

    pub(super) fn fields(&self) -> Result<Vec<String>, String> {
        self.validate()?;
        let mut fields = vec![SESSION_DRAIN_CONTRACT.to_owned(), self.sequence.to_string()];
        fields.extend(self.target.fields().map(str::to_owned));
        Ok(fields)
    }

    pub(super) fn parse(fields: &[&str]) -> Result<Self, String> {
        if fields.len() != 7 || fields[0] != SESSION_DRAIN_CONTRACT {
            return Err("runtime IPC drain contract or shape is invalid".to_owned());
        }
        let drain = Self {
            sequence: number(fields[1])?,
            target: DispatchTarget::parse(&fields[2..])?,
        };
        drain.validate()?;
        Ok(drain)
    }
}

#[cfg(test)]
#[path = "provider_runtime_drain_tests.rs"]
mod tests;
