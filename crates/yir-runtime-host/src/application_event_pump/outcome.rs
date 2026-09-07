use super::{ApplicationPumpOperation, ApplicationPumpPhase, ApplicationPumpReply};
use yir_core::ApplicationOutcome;

impl ApplicationPumpReply {
    /// A fault awaiting explicit cleanup is not a terminal outcome. A successful
    /// close reply is only delivered after the scoped provider Finish gate.
    pub fn outcome(&self) -> Result<Option<ApplicationOutcome>, String> {
        let outcome = match self.phase {
            ApplicationPumpPhase::Closed
                if self.operation == ApplicationPumpOperation::Close
                    && self.state.is_some()
                    && self.cleanup_completed
                    && !self.failure_kind.is_failure()
                    && self.trace.is_ok() =>
            {
                ApplicationOutcome::success()
            }
            ApplicationPumpPhase::Stopped
                if self.trace.is_err()
                    && self.failure_kind.is_failure()
                    && (!self.cleanup_completed
                        || (self.operation == ApplicationPumpOperation::Close
                            && self.state.is_some())) =>
            {
                ApplicationOutcome::failed(self.cleanup_completed, self.failure_kind)
            }
            ApplicationPumpPhase::Closed | ApplicationPumpPhase::Stopped => {
                return Err("application event pump terminal reply is inconsistent".to_owned());
            }
            _ => return Ok(None),
        };
        Ok(Some(outcome))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::{ApplicationFailureKind, Value};

    #[test]
    fn terminal_reply_matrix_requires_finish_and_retains_cleanup_failure_distinction() {
        for phase in [ApplicationPumpPhase::Closed, ApplicationPumpPhase::Stopped] {
            for operation in [
                ApplicationPumpOperation::Open,
                ApplicationPumpOperation::Event,
                ApplicationPumpOperation::Close,
            ] {
                for has_state in [false, true] {
                    for cleanup in [false, true] {
                        for successful_trace in [false, true] {
                            for kind in [
                                ApplicationFailureKind::None,
                                ApplicationFailureKind::ProviderFinalization,
                            ] {
                                let reply = ApplicationPumpReply {
                                    operation,
                                    phase,
                                    state: has_state.then_some(Value::Int(42)),
                                    cleanup_completed: cleanup,
                                    failure_kind: kind,
                                    trace: if successful_trace {
                                        Ok(Default::default())
                                    } else {
                                        Err("late failure".to_owned())
                                    },
                                };
                                let close_state =
                                    operation == ApplicationPumpOperation::Close && has_state;
                                let valid = match phase {
                                    ApplicationPumpPhase::Closed => {
                                        close_state
                                            && cleanup
                                            && successful_trace
                                            && !kind.is_failure()
                                    }
                                    _ => {
                                        !successful_trace
                                            && kind.is_failure()
                                            && (!cleanup || close_state)
                                    }
                                };
                                let outcome = reply.outcome();
                                assert_eq!(outcome.is_ok(), valid, "{reply:?}");
                                if let Ok(Some(outcome)) = outcome {
                                    assert_eq!(
                                        outcome.codes(),
                                        [
                                            if successful_trace { 1 } else { 2 },
                                            i64::from(cleanup),
                                            kind.code()
                                        ]
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn nonterminal_failure_cannot_claim_an_outcome() {
        for phase in [
            ApplicationPumpPhase::Opening,
            ApplicationPumpPhase::Open,
            ApplicationPumpPhase::Faulted,
        ] {
            let reply = ApplicationPumpReply {
                operation: ApplicationPumpOperation::Event,
                phase,
                state: None,
                cleanup_completed: false,
                failure_kind: ApplicationFailureKind::Callback,
                trace: Err("event failed".to_owned()),
            };
            assert_eq!(reply.outcome().unwrap(), None);
        }
    }
}
