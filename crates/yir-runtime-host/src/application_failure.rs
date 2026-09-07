use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};
use yir_core::ApplicationFailureKind;

/// Scope-owned evidence across the executor's existing String error boundary.
/// Only the first producer failure is retained; reading never drains or resets it.
#[derive(Clone, Default)]
pub(crate) struct FailureState(Arc<AtomicI64>);

impl FailureState {
    pub(crate) fn kind(&self) -> ApplicationFailureKind {
        ApplicationFailureKind::from_code(self.0.load(Ordering::Acquire))
            .unwrap_or(ApplicationFailureKind::Unclassified)
    }

    pub(crate) fn record(&self, kind: ApplicationFailureKind) {
        if kind.is_failure() {
            let _ = self
                .0
                .compare_exchange(0, kind.code(), Ordering::AcqRel, Ordering::Acquire);
        }
    }

    pub(crate) fn report(&self, failure: ProviderFailure) -> String {
        self.record(failure.kind);
        failure.detail
    }
}

#[derive(Debug)]
pub(crate) struct ProviderFailure {
    pub(crate) kind: ApplicationFailureKind,
    pub(crate) detail: String,
}

impl ProviderFailure {
    pub(crate) fn new(kind: ApplicationFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub(crate) fn exchange(detail: String) -> Self {
        Self::new(ApplicationFailureKind::ProviderExchange, detail)
    }
}

// Local descriptor/identity validators retain their diagnostic API. Transport
// and remote rejection sites must choose their more specific category explicitly.
impl From<String> for ProviderFailure {
    fn from(detail: String) -> Self {
        Self::new(ApplicationFailureKind::ProviderContract, detail)
    }
}

impl std::fmt::Display for ProviderFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_state_is_first_wins_and_isolated_between_sessions() {
        let first = FailureState::default();
        let observer = first.clone();
        let second = FailureState::default();
        first.record(ApplicationFailureKind::DispatchLimit);
        observer.record(ApplicationFailureKind::None);
        observer.record(ApplicationFailureKind::Host);
        assert_eq!(first.kind(), ApplicationFailureKind::DispatchLimit);
        assert_eq!(observer.kind(), first.kind());
        assert_eq!(second.kind(), ApplicationFailureKind::None);
    }
}
