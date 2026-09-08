use yir_core::ApplicationFailureKind;

/// Descriptive evidence for one explicitly abandoned provider session. Even a
/// confirmed drain is not application success or permission for resource reuse.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProviderDrainObservation {
    #[default]
    NotRequested,
    /// The scope ended before a provider drain observation could be collected.
    NotObserved,
    /// Replay is not a live provider and cannot produce a retirement receipt.
    ReplayOnly,
    /// An earlier exchange left no validated frontier; no drain was sent.
    Unavailable,
    Failed(ApplicationFailureKind),
    /// Exact target and completed-frame count checked on the admitted connection.
    Drained {
        completed_dispatches: usize,
    },
}

impl ProviderDrainObservation {
    pub fn code(self) -> i32 {
        match self {
            Self::NotRequested => 0,
            Self::NotObserved => 1,
            Self::ReplayOnly => 2,
            Self::Unavailable => 3,
            Self::Failed(_) => 4,
            Self::Drained { .. } => 5,
        }
    }

    /// Independent drain failure, not the application's first-failure latch.
    pub fn failure_kind(self) -> ApplicationFailureKind {
        match self {
            Self::Failed(kind) => kind,
            _ => ApplicationFailureKind::None,
        }
    }

    pub fn completed_dispatches(self) -> Option<usize> {
        match self {
            Self::Drained {
                completed_dispatches,
            } => Some(completed_dispatches),
            _ => None,
        }
    }
}
