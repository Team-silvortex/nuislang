#![allow(dead_code)]

mod handoff;
mod model;
mod provider_runner_metal;
mod provider_runner_registry;
mod provider_sample;
mod provider_sample_execute;
mod provider_sample_execution;
mod provider_sample_materialize;
#[cfg(test)]
mod provider_sample_materialize_tests;
mod provider_sample_payload;
mod provider_sample_runner;

pub use provider_sample_execute::{execute_provider_samples, ProviderSampleExecuteReport};
pub use provider_sample_materialize::{
    materialize_provider_samples, ProviderSampleMaterializeReport,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadExecutionReplaySummary {
    pub contract: &'static str,
    pub status: String,
    pub checkpoint_count: usize,
    pub replayable_checkpoint_count: usize,
    pub first_blocker: Option<String>,
}

pub fn payload_execution_replay_summary(
    output_dir: &std::path::Path,
) -> PayloadExecutionReplaySummary {
    let handoff = handoff::read_payload_execution_handoff(output_dir);
    let checkpoint_count = handoff.events.len();
    let replayable_checkpoint_count = handoff
        .events
        .iter()
        .filter(|event| event.status == "ready")
        .count();
    let first_blocker = if !handoff.available {
        Some("payload-execution-handoff-missing".to_owned())
    } else if handoff.status != "ready" {
        Some(format!("payload-execution-handoff:{}", handoff.status))
    } else if handoff.hetero_execution_closure_status != "none"
        && (handoff.hetero_execution_closure_status != "closed"
            || handoff.hetero_execution_closure_ready != "true")
    {
        Some(
            if handoff.hetero_execution_closure_first_blocker != "none" {
                format!(
                    "hetero-execution-closure:{}",
                    handoff.hetero_execution_closure_first_blocker
                )
            } else {
                format!(
                    "hetero-execution-closure:{}",
                    handoff.hetero_execution_closure_status
                )
            },
        )
    } else if checkpoint_count == 0 {
        Some("payload-execution-replay:no-checkpoints".to_owned())
    } else if replayable_checkpoint_count != checkpoint_count {
        Some("payload-execution-replay:blocked-checkpoint".to_owned())
    } else {
        None
    };
    PayloadExecutionReplaySummary {
        contract: "nsdb-payload-execution-replay-plan-v1",
        status: if first_blocker.is_none() {
            "replay-evidence-ready".to_owned()
        } else {
            "blocked".to_owned()
        },
        checkpoint_count,
        replayable_checkpoint_count,
        first_blocker,
    }
}

#[cfg(test)]
mod tests {
    use super::payload_execution_replay_summary;
    use std::{fs, path::Path};

    #[test]
    fn payload_execution_replay_summary_consumes_ready_handoff() {
        let dir =
            std::env::temp_dir().join(format!("nsdb-lib-replay-summary-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.payload-execution-handoff.toml"),
            r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
source = "nsld-final-executable-output"
record_count = 1
ready_record_count = 1
first_trace_id = "payload-trace:container-loader:main"
first_status = "ready"
first_next_action = "handoff-payload-trace-to-nsdb"

[[records]]
trace_id = "payload-trace:container-loader:main"
status = "ready"
execution_phase = "container-loader-handoff"
target = "container-loader"
entry_symbol = "main"
entry_kind = "lifecycle-bootstrap"
entry_section_id = "sec0000.compiled-artifact"
first_blocker = ""
next_action = "handoff-payload-trace-to-nsdb"
"#,
        )
        .unwrap();

        let summary = payload_execution_replay_summary(Path::new(&dir));
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(summary.contract, "nsdb-payload-execution-replay-plan-v1");
        assert_eq!(summary.status, "replay-evidence-ready");
        assert_eq!(summary.checkpoint_count, 1);
        assert_eq!(summary.replayable_checkpoint_count, 1);
        assert_eq!(summary.first_blocker, None);
    }

    #[test]
    fn payload_execution_replay_summary_blocks_pending_hetero_closure() {
        let dir = std::env::temp_dir().join(format!(
            "nsdb-lib-replay-summary-closure-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("nuis.nsdb.payload-execution-handoff.toml"),
            r#"
protocol = "nuis-nsdb-payload-execution-handoff-v1"
debugger_contract = "nsdb-yir-payload-execution-trace-v1"
record_count = 1
ready_record_count = 1
hetero_execution_closure_protocol = "nuis-hetero-execution-closure-v1"
hetero_execution_closure_status = "host-runner-pending"
hetero_execution_closure_ready = "false"
hetero_execution_closure_first_blocker = "host-runner-backend-artifact-payload:not-observed"
hetero_execution_closure_next_action = "run-host-runner-payload-probe"

[[records]]
trace_id = "payload-trace:container-loader:main"
status = "ready"
execution_phase = "container-loader-handoff"
entry_symbol = "main"
next_action = "handoff-payload-trace-to-nsdb"
"#,
        )
        .unwrap();

        let summary = payload_execution_replay_summary(Path::new(&dir));
        fs::remove_dir_all(dir).unwrap();

        assert_eq!(summary.status, "blocked");
        assert_eq!(summary.checkpoint_count, 1);
        assert_eq!(summary.replayable_checkpoint_count, 1);
        assert_eq!(
            summary.first_blocker.as_deref(),
            Some("hetero-execution-closure:host-runner-backend-artifact-payload:not-observed")
        );
    }
}
