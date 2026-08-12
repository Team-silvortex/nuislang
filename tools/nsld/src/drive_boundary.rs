pub(crate) fn final_output_boundary_stop_reason(reason: Option<&str>) -> &'static str {
    match reason {
        Some(reason)
            if reason.contains("repair provider output payload diagnostics")
                || reason.contains("provider-sample-blocked") =>
        {
            "provider-output-payload-repair-required"
        }
        Some(reason) if reason.contains("device-provider-sample:") => {
            "provider-sample-materialization-required"
        }
        Some(reason) if reason.contains("final-executable-output:not-nsld-owned") => {
            "host-finalizer-policy-required"
        }
        Some(reason) if reason.contains("final-executable-output:missing") => {
            "final-output-missing"
        }
        Some(reason) if reason.contains("final-executable-output:image-header-invalid") => {
            "final-output-invalid"
        }
        Some(reason) if reason.contains("final-executable-output:hash-mismatch") => {
            "final-output-invalid"
        }
        Some(reason) if reason.contains("final-executable-output:size-mismatch") => {
            "final-output-invalid"
        }
        _ => "blocked-boundary",
    }
}
