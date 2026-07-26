pub(super) fn dispatch_identity_json_fields(
    summary: Option<&crate::workflow::NsldFinalExecutableOutputBoundarySummary>,
) -> Vec<String> {
    vec![crate::json_optional_string_field(
        "nsld_final_executable_output_debugger_cursor_lineage_provider_dispatch_identity_hash",
        summary.and_then(|summary| {
            summary
                .debugger_cursor_lineage_provider_dispatch_identity_hash
                .as_deref()
        }),
    )]
}
