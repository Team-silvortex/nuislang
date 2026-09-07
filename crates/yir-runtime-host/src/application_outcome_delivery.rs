use yir_core::{ApplicationOutcome, Value};
use yir_exec::ExecutionTrace;

use crate::ApplicationSession;

#[cfg(test)]
mod tests;

pub const APPLICATION_OUTCOME_DELIVERY_CONTRACT: &str = "nuis-yir-application-outcome-delivery-v1";

/// One owned delivery attempt, not a device completion or resource capability.
/// There is no public constructor or Clone: a terminal source issues this once.
/// Dropping it abandons delivery without executing application code.
#[derive(Debug)]
#[must_use = "dropping the delivery abandons it without invoking the parent"]
pub struct ApplicationOutcomeDelivery {
    outcome: ApplicationOutcome,
}

impl ApplicationOutcomeDelivery {
    pub(crate) fn new(outcome: ApplicationOutcome) -> Self {
        Self { outcome }
    }

    pub fn outcome(&self) -> ApplicationOutcome {
        self.outcome
    }

    /// Consume the attempt even on signature, phase or execution failure. The
    /// caller explicitly selects an already open parent, whose registered event
    /// takes its carried state followed by status/cleanup/failure i64 arguments.
    /// Only these scalars cross: no child context, registry, handles or budget.
    /// Parent effects use its existing authority; this is not a pure observer.
    /// Execution is synchronous; scoped fuel is not a provider preemption or
    /// GUI deadline guarantee.
    pub fn deliver(
        self,
        parent: &mut ApplicationSession<'_>,
        max_steps: usize,
    ) -> Result<ExecutionTrace, String> {
        parent.event_budgeted(
            self.outcome.codes().into_iter().map(Value::Int).collect(),
            max_steps,
        )
    }
}
