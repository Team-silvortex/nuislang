use crate::ApplicationFailureKind;

pub const APPLICATION_OUTCOME_CONTRACT: &str = "nuis-yir-application-outcome-v1";

/// Immutable terminal observation, separate from application state and provider
/// receipts. Decoding this value grants no completion, resource or retry authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationOutcome {
    cleanup_completed: bool,
    failure_kind: ApplicationFailureKind,
}

impl ApplicationOutcome {
    pub fn success() -> Self {
        Self {
            cleanup_completed: true,
            failure_kind: ApplicationFailureKind::None,
        }
    }

    /// A known failure without a classified cause stays failed, never success.
    pub fn failed(cleanup_completed: bool, kind: ApplicationFailureKind) -> Self {
        Self {
            cleanup_completed,
            failure_kind: if kind.is_failure() {
                kind
            } else {
                ApplicationFailureKind::Unclassified
            },
        }
    }

    pub fn from_codes(status: i64, cleanup: i64, failure: i64) -> Result<Self, String> {
        let kind = ApplicationFailureKind::from_code(failure)?;
        match (status, cleanup, kind.is_failure()) {
            (1, 1, false) => Ok(Self::success()),
            (2, 0 | 1, true) => Ok(Self::failed(cleanup == 1, kind)),
            _ => Err("application outcome has contradictory or noncanonical fields".to_owned()),
        }
    }

    /// Stable scalar order: status (succeeded=1, failed=2), cleanup (0/1), failure.
    pub fn codes(self) -> [i64; 3] {
        [
            if self.is_success() { 1 } else { 2 },
            i64::from(self.cleanup_completed),
            self.failure_kind.code(),
        ]
    }

    pub fn is_success(self) -> bool {
        !self.failure_kind.is_failure()
    }

    pub fn cleanup_completed(self) -> bool {
        self.cleanup_completed
    }

    pub fn failure_kind(self) -> ApplicationFailureKind {
        self.failure_kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_codes_reject_unknown_and_contradictory_reports() {
        for status in [i64::MIN, -1, 0, 1, 2, 3, i64::MAX] {
            for cleanup in [-1, 0, 1, 2, i64::MAX] {
                for failure in -1..=12 {
                    let valid = (status == 1 && cleanup == 1 && failure == 0)
                        || (status == 2
                            && (0..=1).contains(&cleanup)
                            && (1..=11).contains(&failure));
                    let outcome = ApplicationOutcome::from_codes(status, cleanup, failure);
                    assert_eq!(outcome.is_ok(), valid, "{status}/{cleanup}/{failure}");
                    if let Ok(outcome) = outcome {
                        assert_eq!(outcome.codes(), [status, cleanup, failure]);
                        assert_eq!(outcome.is_success(), status == 1);
                        assert_eq!(outcome.cleanup_completed(), cleanup == 1);
                        assert_eq!(outcome.failure_kind().code(), failure);
                    }
                }
            }
        }
    }

    #[test]
    fn unclassified_failure_never_becomes_success() {
        for cleanup in [false, true] {
            let outcome = ApplicationOutcome::failed(cleanup, ApplicationFailureKind::None);
            assert_eq!(outcome.codes(), [2, i64::from(cleanup), 8]);
        }
    }
}
