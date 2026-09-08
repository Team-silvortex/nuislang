use yir_core::provider_runtime_ipc::{Message, ProviderRuntimeSessionOutcome, SessionDrain};

/// Application intent, not a backend/OS selector. Both the reader adapter and
/// final observer use the same policy; neither can infer drain from process exit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProviderLaunchPolicy {
    CompletionOnly,
    ExplicitDrain,
}

impl ProviderLaunchPolicy {
    pub(crate) fn admit_request(self, message: &Message) -> Result<(), String> {
        if self == Self::ExplicitDrain && matches!(message, Message::Finish(_)) {
            return Err(
                "explicit provider drain launch rejects Finish before publication".to_owned(),
            );
        }
        Ok(())
    }

    pub(crate) fn admit_exit(
        self,
        provider: &ProviderLaunchOutcome,
        code: Option<i32>,
    ) -> Result<(), String> {
        match (self, provider) {
            (Self::CompletionOnly, ProviderLaunchOutcome::Finished { .. }) => Ok(()),
            (Self::ExplicitDrain, ProviderLaunchOutcome::Drained(_))
                if code == Some(yir_runtime_host::APPLICATION_CANCELLED_EXIT_CODE) =>
            {
                Ok(())
            }
            (Self::ExplicitDrain, ProviderLaunchOutcome::Drained(_)) => Err(format!(
                "provider drained but child exit {code:?} did not confirm the cancellation policy"
            )),
            _ => Err("provider lifecycle does not match the admitted launch policy".to_owned()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProviderLaunchOutcome {
    Finished { sessions: usize, invocations: usize },
    Drained(SessionDrain),
}

impl ProviderLaunchOutcome {
    pub(crate) fn into_finished_count(self) -> Result<usize, String> {
        match self {
            Self::Finished { invocations, .. } => Ok(invocations),
            Self::Drained(_) => {
                Err("runtime provider drained without application completion".to_owned())
            }
        }
    }
}

/// Bounded terminal accounting independent of sockets, processes, windows and
/// provider implementations. A policy error permanently poisons this observer.
pub(crate) struct ProviderLaunchLifecycle {
    policy: ProviderLaunchPolicy,
    outcome: Option<ProviderLaunchOutcome>,
    failure: Option<String>,
}

impl ProviderLaunchLifecycle {
    pub(crate) fn new(policy: ProviderLaunchPolicy) -> Self {
        Self {
            policy,
            outcome: None,
            failure: None,
        }
    }

    /// True means a drain is terminal: the adapter must not accept more sessions.
    pub(crate) fn observe(
        &mut self,
        outcome: ProviderRuntimeSessionOutcome,
    ) -> Result<bool, String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let result = self.observe_checked(outcome);
        if let Err(error) = &result {
            self.failure = Some(error.clone());
        }
        result
    }

    fn observe_checked(&mut self, outcome: ProviderRuntimeSessionOutcome) -> Result<bool, String> {
        if matches!(self.outcome, Some(ProviderLaunchOutcome::Drained(_))) {
            return Err(
                "runtime provider drain is terminal; another session is not admitted".to_owned(),
            );
        }
        match (self.policy, outcome) {
            (
                ProviderLaunchPolicy::CompletionOnly,
                ProviderRuntimeSessionOutcome::Finished(count),
            ) => {
                let (sessions, invocations) = match self.outcome {
                    Some(ProviderLaunchOutcome::Finished {
                        sessions,
                        invocations,
                    }) => (sessions, invocations),
                    None => (0usize, 0usize),
                    _ => unreachable!("terminal drain checked above"),
                };
                self.outcome = Some(ProviderLaunchOutcome::Finished {
                    sessions: sessions
                        .checked_add(1)
                        .ok_or("runtime IPC session count overflow")?,
                    invocations: invocations
                        .checked_add(count)
                        .ok_or("runtime IPC invocation count overflow")?,
                });
                Ok(false)
            }
            (
                ProviderLaunchPolicy::ExplicitDrain,
                ProviderRuntimeSessionOutcome::Drained(receipt),
            ) => {
                receipt.admit(&receipt.target, receipt.sequence)?;
                self.outcome = Some(ProviderLaunchOutcome::Drained(receipt));
                Ok(true)
            }
            (ProviderLaunchPolicy::CompletionOnly, ProviderRuntimeSessionOutcome::Drained(_)) => {
                Err("runtime provider drained without application completion".to_owned())
            }
            (ProviderLaunchPolicy::ExplicitDrain, ProviderRuntimeSessionOutcome::Finished(_)) => {
                Err("explicit provider drain launch cannot consume Finished".to_owned())
            }
        }
    }

    pub(crate) fn finish(self) -> Result<ProviderLaunchOutcome, String> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.outcome
            .ok_or_else(|| "runtime child did not complete a provider lifecycle".to_owned())
    }
}

#[cfg(test)]
#[path = "artifact_runtime_provider_lifecycle_tests.rs"]
mod tests;
