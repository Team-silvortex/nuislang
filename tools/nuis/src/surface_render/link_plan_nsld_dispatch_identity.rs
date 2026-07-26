use crate::workflow::NsldFinalExecutableOutputBoundarySummary;
use std::fmt;

pub(super) fn write_provider_dispatch_identity_capability<W: fmt::Write>(
    out: &mut W,
    summary: Option<&NsldFinalExecutableOutputBoundarySummary>,
) -> fmt::Result {
    let capability = summary.map(|summary| &summary.provider_dispatch_identity_capability);
    for prefix in [
        "nsld_final_executable_output_object_package",
        "nsld_final_executable_output_debugger_api",
    ] {
        writeln!(
            out,
            "  {prefix}_provider_dispatch_identity_capability_contract: {}",
            capability
                .map(|capability| capability.contract)
                .unwrap_or("<unavailable>")
        )?;
        writeln!(
            out,
            "  {prefix}_provider_dispatch_identity_ready: {}",
            capability
                .map(|capability| crate::yes_no(capability.ready))
                .unwrap_or("<unavailable>")
        )?;
        writeln!(
            out,
            "  {prefix}_provider_dispatch_identity_status: {}",
            capability
                .map(|capability| capability.status)
                .unwrap_or("<unavailable>")
        )?;
        writeln!(
            out,
            "  {prefix}_provider_dispatch_identity_source: {} status={}",
            capability
                .map(|capability| capability.source_contract.as_str())
                .unwrap_or("<unavailable>"),
            capability
                .map(|capability| capability.source_status.as_str())
                .unwrap_or("<unavailable>")
        )?;
        writeln!(
            out,
            "  {prefix}_provider_dispatch_identity_hash: {}",
            capability
                .and_then(|capability| capability.identity_hash.as_deref())
                .unwrap_or("<none>")
        )?;
        writeln!(
            out,
            "  {prefix}_provider_dispatch_identity_first_blocker: {}",
            capability
                .and_then(|capability| capability.first_blocker.as_deref())
                .unwrap_or("<none>")
        )?;
    }
    writeln!(
        out,
        "  nsld_final_executable_output_provider_dispatch_identity_projection_source: debugger_cursor_lineage_provider_dispatch_identity_hash"
    )
}
