pub(crate) fn render_metadata_for_trace(
    values: &[String],
    domain_family: &str,
    trace_id: &str,
) -> String {
    let projection =
        nuisc::artifact_provider_metadata::project_artifact_provider_metadata_for_trace(
            values,
            domain_family,
            trace_id,
        );
    match projection {
        Ok(projected) => {
            let mut evidence = format!(
                "artifact_provider_metadata_contract=nuis-artifact-provider-metadata-v1;artifact_provider_metadata_scope_contract={};artifact_provider_metadata_scope_status=verified;artifact_provider_metadata_scope_domain={domain_family};artifact_provider_metadata_scope_trace={trace_id};artifact_provider_metadata_source_count={};artifact_provider_metadata_count={}",
                nuisc::artifact_provider_metadata::ARTIFACT_PROVIDER_METADATA_SCOPE_CONTRACT,
                values.len(),
                projected.len()
            );
            for (index, item) in projected.iter().enumerate() {
                evidence.push_str(&format!(";artifact_provider_metadata_{index}={item}"));
            }
            evidence
        }
        Err(error) => format!(
            "artifact_provider_metadata_contract=nuis-artifact-provider-metadata-v1;artifact_provider_metadata_scope_contract={};artifact_provider_metadata_scope_status=invalid;artifact_provider_metadata_scope_domain={domain_family};artifact_provider_metadata_scope_trace={trace_id};artifact_provider_metadata_source_count={};artifact_provider_metadata_count=0;artifact_provider_metadata_scope_error={}",
            nuisc::artifact_provider_metadata::ARTIFACT_PROVIDER_METADATA_SCOPE_CONTRACT,
            values.len(),
            error.replace([';', '\n', '\r'], "_")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_projection_keeps_global_metadata_and_selects_one_scoped_plan() {
        let values = vec![
            "@scope(trace=hetero-trace:shader:metal:first)|nuis.pixelmagic:filter-plan=first"
                .to_owned(),
            "nuis.other:key=global".to_owned(),
            "@scope(trace=hetero-trace:shader:metal:second)|nuis.pixelmagic:filter-plan=second"
                .to_owned(),
        ];
        let first = render_metadata_for_trace(&values, "shader", "hetero-trace:shader:metal:first");
        let second =
            render_metadata_for_trace(&values, "shader", "hetero-trace:shader:metal:second");

        assert!(first.contains("artifact_provider_metadata_source_count=3"));
        assert!(first.contains("artifact_provider_metadata_count=2"));
        assert!(first.contains("nuis.pixelmagic:filter-plan=first"));
        assert!(!first.contains("nuis.pixelmagic:filter-plan=second"));
        assert!(second.contains("nuis.pixelmagic:filter-plan=second"));
        assert!(!second.contains("nuis.pixelmagic:filter-plan=first"));
        assert!(first.contains("nuis.other:key=global"));
        assert!(second.contains("nuis.other:key=global"));
    }

    #[test]
    fn invalid_scopes_do_not_leak_provider_metadata() {
        let evidence = render_metadata_for_trace(
            &["@scope(domain=shader|nuis.pixelmagic:key=value".to_owned()],
            "shader",
            "hetero-trace:shader:metal",
        );

        assert!(evidence.contains("artifact_provider_metadata_scope_status=invalid"));
        assert!(evidence.contains("artifact_provider_metadata_count=0"));
        assert!(!evidence.contains("artifact_provider_metadata_0="));
    }
}
