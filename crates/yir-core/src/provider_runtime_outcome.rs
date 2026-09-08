use super::SessionDrain;

/// Transport-independent provider-owned terminal evidence. This is neither a
/// child-process exit status nor a successful application outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderRuntimeSessionOutcome {
    Finished(usize),
    Drained(SessionDrain),
}

impl ProviderRuntimeSessionOutcome {
    /// Success-only consumers must reject, not count, a drained session.
    pub fn into_finished_count(self) -> Result<usize, String> {
        match self {
            Self::Finished(count) => Ok(count),
            Self::Drained(_) => {
                Err("runtime provider drained without application completion".to_owned())
            }
        }
    }
}
