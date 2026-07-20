use super::{
    final_executable_output::{owned_package_summary_next_action, owned_package_summary_status},
    reports::{NsldFinalExecutableOutputReport, NsldProviderCompletionSummary},
};
use std::path::Path;

const NSDB_HANDOFF_PROTOCOL: &str = "nuis-nsdb-payload-execution-handoff-v1";
const NSDB_HANDOFF_FILE_NAME: &str = "nuis.nsdb.payload-execution-handoff.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NsldFinalOutputNsdbHandoffSummary {
    pub(crate) protocol: &'static str,
    pub(crate) persisted: bool,
    pub(crate) path: String,
    pub(crate) record_count: usize,
    pub(crate) ready_record_count: usize,
    pub(crate) first_trace_id: Option<String>,
    pub(crate) error: Option<String>,
}

pub(crate) fn attach_final_output_nsdb_handoff_summary(
    report: &mut NsldFinalExecutableOutputReport,
    summary: NsldFinalOutputNsdbHandoffSummary,
) {
    report.final_output_nsdb_handoff_protocol = summary.protocol.to_owned();
    report.final_output_nsdb_handoff_persisted = summary.persisted;
    report.final_output_nsdb_handoff_path = summary.path;
    report.final_output_nsdb_handoff_record_count = summary.record_count;
    report.final_output_nsdb_handoff_ready_record_count = summary.ready_record_count;
    report.final_output_nsdb_handoff_first_trace_id = summary.first_trace_id;
    report.final_output_nsdb_handoff_error = summary.error;
    let replay_summary = nsdb::payload_execution_replay_summary(output_dir_from_handoff_path(
        &report.final_output_nsdb_handoff_path,
    ));
    report.final_output_nsdb_replay_contract = replay_summary.contract.to_owned();
    report.final_output_nsdb_replay_ready = replay_summary.status == "replay-evidence-ready";
    report.final_output_nsdb_replay_status = replay_summary.status;
    report.final_output_nsdb_replay_checkpoint_count = replay_summary.checkpoint_count;
    report.final_output_nsdb_replayable_checkpoint_count =
        replay_summary.replayable_checkpoint_count;
    report.final_output_nsdb_provider_completion_count = replay_summary.provider_completion_count;
    report.final_output_nsdb_first_provider_family = replay_summary.first_provider_family;
    report.final_output_nsdb_first_provider_output_contract =
        replay_summary.first_provider_output_contract;
    report.final_output_nsdb_first_provider_output_evidence =
        replay_summary.first_provider_output_evidence;
    report.final_output_nsdb_provider_completion_claim_authority_contract =
        replay_summary.provider_completion_claim_authority_contract;
    report.final_output_nsdb_provider_completion_claim_authority =
        replay_summary.provider_completion_claim_authority;
    report.final_output_nsdb_provider_completion_claim_authority_status =
        replay_summary.provider_completion_claim_authority_status;
    report.final_output_nsdb_provider_completion_signature_contract =
        replay_summary.provider_completion_signature_contract;
    report.final_output_nsdb_provider_completion_signature_public_key_id =
        replay_summary.provider_completion_signature_public_key_id;
    report.final_output_nsdb_provider_completion_signature_status =
        replay_summary.provider_completion_signature_status;
    report.final_output_nsdb_provider_completion_digest_contract =
        replay_summary.provider_completion_digest_contract;
    report.final_output_nsdb_provider_completion_set_hash_claim =
        replay_summary.provider_completion_set_hash_claim;
    report.final_output_nsdb_provider_completion_set_hash =
        replay_summary.provider_completion_set_hash;
    report.final_output_nsdb_provider_completion_set_hash_validation_status =
        replay_summary.provider_completion_set_hash_validation_status;
    report.final_output_nsdb_provider_completions = replay_summary
        .provider_completions
        .into_iter()
        .map(|completion| NsldProviderCompletionSummary {
            trace_id: completion.trace_id,
            provider_family: completion.provider_family,
            output_contract: completion.output_contract,
            output_evidence: completion.output_evidence,
            record_hash: completion.record_hash,
        })
        .collect();
    report.final_output_nsdb_replay_command = report.final_output_nsdb_replay_ready.then(|| {
        format!(
            "nsdb replay {} --json",
            shell_quote_path(output_dir_from_handoff_path(
                &report.final_output_nsdb_handoff_path
            ))
        )
    });
    report.final_output_nsdb_replay_next_action = if report.final_output_nsdb_replay_ready {
        "replay-nsdb-payload-execution"
    } else {
        "resolve-final-output-nsdb-replay"
    }
    .to_owned();
    report.final_output_nsdb_replay_next_command =
        report.final_output_nsdb_replay_command.clone().or_else(|| {
            Some(format!(
                "nsld final-executable-output {} --json",
                shell_quote_path(Path::new(&report.manifest))
            ))
        });
    report.final_output_nsdb_replay_first_blocker = if report.final_output_nsdb_replay_ready {
        None
    } else {
        replay_summary
            .first_blocker
            .or_else(|| report.final_output_nsdb_handoff_error.clone())
            .or_else(|| Some("final-output-nsdb-replay-not-ready".to_owned()))
    };
    report.owned_package_summary_status =
        owned_package_summary_status(report.final_output_nsdb_replay_ready).to_owned();
    report.owned_package_summary_ready = report.final_output_nsdb_replay_ready;
    report.owned_package_summary_replay_status = report.final_output_nsdb_replay_status.clone();
    report.owned_package_summary_replay_ready = report.final_output_nsdb_replay_ready;
    report.owned_package_summary_next_action =
        owned_package_summary_next_action(report.final_output_nsdb_replay_ready).to_owned();
    report.owned_package_summary_next_command =
        report.final_output_nsdb_replay_next_command.clone();
    report.object_package_summary_status =
        owned_package_summary_status(report.final_output_nsdb_replay_ready).to_owned();
    report.object_package_summary_ready = report.final_output_nsdb_replay_ready;
    report.object_package_summary_replay_status = report.final_output_nsdb_replay_status.clone();
    report.object_package_summary_replay_ready = report.final_output_nsdb_replay_ready;
    report.object_package_summary_next_action =
        owned_package_summary_next_action(report.final_output_nsdb_replay_ready).to_owned();
    report.object_package_summary_next_command =
        report.final_output_nsdb_replay_next_command.clone();
}

fn output_dir_from_handoff_path(path: &str) -> &Path {
    Path::new(path).parent().unwrap_or_else(|| Path::new(path))
}

fn shell_quote_path(path: &Path) -> String {
    let text = path.display().to_string();
    if text
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':'))
    {
        text
    } else {
        format!("'{}'", text.replace('\'', "'\\''"))
    }
}

pub(crate) fn persist_final_output_nsdb_handoff(
    output_dir: &Path,
    report: &NsldFinalExecutableOutputReport,
) -> NsldFinalOutputNsdbHandoffSummary {
    let path = output_dir.join(NSDB_HANDOFF_FILE_NAME);
    let path_text = path.display().to_string();
    let Some(record) = final_output_payload_trace_record(report) else {
        return NsldFinalOutputNsdbHandoffSummary {
            protocol: NSDB_HANDOFF_PROTOCOL,
            persisted: false,
            path: path_text,
            record_count: 0,
            ready_record_count: 0,
            first_trace_id: None,
            error: Some("payload-execution-trace-unavailable".to_owned()),
        };
    };
    let handoff_record = nsdb::PayloadExecutionHandoffRecord {
        trace_id: record.trace_id.clone(),
        status: record.status.clone(),
        execution_phase: "container-loader-handoff".to_owned(),
        target: record.target.clone(),
        entry_symbol: record.entry_symbol.clone().unwrap_or_default(),
        entry_kind: record.entry_kind.clone().unwrap_or_default(),
        entry_section_id: record.entry_section_id.clone().unwrap_or_default(),
        provider_family: String::new(),
        output_contract: String::new(),
        output_evidence: String::new(),
        first_blocker: record.first_blocker.clone().unwrap_or_default(),
        next_action: record.next_action.clone(),
    };
    match nsdb::persist_payload_execution_handoff_record(
        output_dir,
        "nsld-final-executable-output",
        handoff_record,
    ) {
        Ok(summary) => NsldFinalOutputNsdbHandoffSummary {
            protocol: NSDB_HANDOFF_PROTOCOL,
            persisted: true,
            path: path_text,
            record_count: summary.record_count,
            ready_record_count: summary.ready_record_count,
            first_trace_id: summary.first_trace_id,
            error: None,
        },
        Err(error) => NsldFinalOutputNsdbHandoffSummary {
            protocol: NSDB_HANDOFF_PROTOCOL,
            persisted: false,
            path: path_text,
            record_count: 1,
            ready_record_count: usize::from(record.status == "ready"),
            first_trace_id: Some(record.trace_id),
            error: Some(error),
        },
    }
}

struct FinalOutputPayloadTraceRecord {
    trace_id: String,
    status: String,
    target: String,
    entry_symbol: Option<String>,
    entry_kind: Option<String>,
    entry_section_id: Option<String>,
    first_blocker: Option<String>,
    next_action: String,
}

fn final_output_payload_trace_record(
    report: &NsldFinalExecutableOutputReport,
) -> Option<FinalOutputPayloadTraceRecord> {
    if report.first_payload_execution_target != "container-loader" {
        return None;
    }
    let symbol = report
        .first_payload_execution_entry_symbol
        .clone()
        .unwrap_or_else(|| "unknown-symbol".to_owned());
    Some(FinalOutputPayloadTraceRecord {
        trace_id: format!(
            "payload-trace:{}:{}",
            report.first_payload_execution_target, symbol
        ),
        status: report.first_payload_execution_status.clone(),
        target: report.first_payload_execution_target.clone(),
        entry_symbol: report.first_payload_execution_entry_symbol.clone(),
        entry_kind: report.first_payload_execution_entry_kind.clone(),
        entry_section_id: report.first_payload_execution_entry_section_id.clone(),
        first_blocker: report.first_payload_execution_first_blocker.clone(),
        next_action: if report.first_payload_execution_ready {
            "handoff-payload-trace-to-nsdb".to_owned()
        } else {
            "resolve-payload-execution-blocker".to_owned()
        },
    })
}
