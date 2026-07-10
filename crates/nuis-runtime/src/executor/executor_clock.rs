use std::collections::BTreeSet;

use crate::{PreparedDomainExecution, RuntimeError};

use super::ExecutionTraceEvent;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionClockGate {
    pub wait_on: Vec<String>,
    pub emits: Vec<String>,
}

impl ExecutionClockGate {
    pub fn is_empty(&self) -> bool {
        self.wait_on.is_empty() && self.emits.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionClockValidation {
    pub initial_timestamps: Vec<String>,
    pub observed_emits: Vec<String>,
    pub final_timestamps: Vec<String>,
}

pub(super) fn execution_clock_summary(prepared: &PreparedDomainExecution<'_>) -> Option<String> {
    let domain = prepared.clock_domain?;
    let mut fields = vec![
        format!("clock_domain={}", domain.clock_domain_id),
        format!("kind={}", domain.clock_kind),
        format!("epoch={}", domain.clock_epoch_kind),
        format!("resolution={}", domain.clock_resolution),
        format!("bridge={}", domain.clock_bridge_default),
        format!("hook={}", domain.lifecycle_hook),
    ];
    if !prepared.clock_edges.is_empty() {
        fields.push(format!(
            "happens_before={}",
            prepared
                .clock_edges
                .iter()
                .map(|edge| format!("{}->{}", edge.from, edge.to))
                .collect::<Vec<_>>()
                .join("|")
        ));
    }
    Some(fields.join(" "))
}

pub(super) fn execution_clock_gate(prepared: &PreparedDomainExecution<'_>) -> ExecutionClockGate {
    let mut wait_on = Vec::new();
    let mut emits = Vec::new();
    let mut seen_wait = BTreeSet::new();
    let mut seen_emit = BTreeSet::new();
    for edge in &prepared.clock_edges {
        if edge.relation == "data-segment-commit" {
            for emitted in [&edge.from, &edge.to] {
                if !emitted.is_empty() && seen_emit.insert(emitted.clone()) {
                    emits.push(emitted.clone());
                }
            }
        } else {
            for wait in edge
                .from
                .split('|')
                .map(str::trim)
                .filter(|wait| !wait.is_empty())
            {
                if seen_wait.insert(wait.to_owned()) {
                    wait_on.push(wait.to_owned());
                }
            }
            if !edge.to.is_empty() && seen_emit.insert(edge.to.clone()) {
                emits.push(edge.to.clone());
            }
        }
    }
    ExecutionClockGate { wait_on, emits }
}

pub(super) fn validate_clock_gate_sequence(
    events: &[ExecutionTraceEvent],
    initial_timestamps: &[String],
) -> Result<ExecutionClockValidation, RuntimeError> {
    let mut satisfied = BTreeSet::new();
    let mut final_timestamps = Vec::new();
    for timestamp in initial_timestamps {
        if satisfied.insert(timestamp.clone()) {
            final_timestamps.push(timestamp.clone());
        }
    }

    let mut observed_emits = Vec::new();
    let mut seen_emits = BTreeSet::new();
    for event in events {
        for required in &event.clock_gate.wait_on {
            if !satisfied.contains(required) {
                return Err(RuntimeError::new(format!(
                    "clock gate violation in phase `{}`: missing timestamp `{}`",
                    event.phase, required
                )));
            }
        }
        for emitted in &event.clock_gate.emits {
            if satisfied.insert(emitted.clone()) {
                final_timestamps.push(emitted.clone());
            }
            if seen_emits.insert(emitted.clone()) {
                observed_emits.push(emitted.clone());
            }
        }
    }

    Ok(ExecutionClockValidation {
        initial_timestamps: initial_timestamps.to_vec(),
        observed_emits,
        final_timestamps,
    })
}
