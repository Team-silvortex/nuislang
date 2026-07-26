use super::reports::NsldFinalExecutableOutputReport;

pub(super) fn payload_execution_trace_protocol() -> &'static str {
    "nsdb-yir-payload-execution-trace-v1"
}

pub(super) fn payload_execution_trace_available(report: &NsldFinalExecutableOutputReport) -> bool {
    report.first_payload_execution_target == "container-loader"
}

pub(super) fn payload_execution_trace_record_count(
    report: &NsldFinalExecutableOutputReport,
) -> usize {
    usize::from(payload_execution_trace_available(report))
}

pub(super) fn payload_execution_trace_ready_record_count(
    report: &NsldFinalExecutableOutputReport,
) -> usize {
    usize::from(payload_execution_trace_available(report) && report.first_payload_execution_ready)
}

pub(super) fn payload_execution_trace_id(report: &NsldFinalExecutableOutputReport) -> String {
    let symbol = report
        .first_payload_execution_entry_symbol
        .as_deref()
        .unwrap_or("unknown-symbol");
    format!(
        "payload-trace:{}:{}",
        report.first_payload_execution_target, symbol
    )
}
