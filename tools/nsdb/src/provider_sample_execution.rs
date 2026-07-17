use crate::provider_runner_registry::ProviderRunnerAdapter;

pub(crate) struct ProviderExecutionOutcome {
    pub(crate) contract: &'static str,
    pub(crate) status: &'static str,
    pub(crate) comparison_status: &'static str,
    pub(crate) evidence_status: &'static str,
    pub(crate) output_payload_contract: &'static str,
    pub(crate) output_payload_status: &'static str,
    pub(crate) output_payload_evidence_status: &'static str,
    pub(crate) output_payload_next_action: &'static str,
    pub(crate) output_payload_file_name: &'static str,
    pub(crate) next_action: &'static str,
    pub(crate) detail: &'static str,
}

pub(crate) fn provider_execution_outcome(
    adapter: &ProviderRunnerAdapter,
) -> ProviderExecutionOutcome {
    if adapter.real_device_capable {
        ProviderExecutionOutcome {
            contract: "nuis-provider-execution-comparison-v1",
            status: "real-device-runner-selected",
            comparison_status: "awaiting-real-device-output",
            evidence_status: "real-device-adapter-selected-no-output-yet",
            output_payload_contract: "nuis-provider-output-payload-handoff-v1",
            output_payload_status: "awaiting-provider-output-payload",
            output_payload_evidence_status: "provider-output-payload-missing",
            output_payload_next_action: "attach-provider-output-payload",
            output_payload_file_name: "not-materialized",
            next_action: "execute-real-device-provider-sample",
            detail: "selected real-device provider runner; output comparison waits for adapter execution",
        }
    } else {
        ProviderExecutionOutcome {
            contract: "nuis-provider-execution-comparison-v1",
            status: "host-fallback-runner-selected",
            comparison_status: "host-fallback-output-comparable",
            evidence_status: "host-simulated-output-anchor",
            output_payload_contract: "nuis-provider-output-payload-handoff-v1",
            output_payload_status: "host-fallback-output-payload-ready",
            output_payload_evidence_status: "deterministic-provider-output-anchor",
            output_payload_next_action: "compare-provider-output-payload",
            output_payload_file_name: "provider-output-payload-anchor",
            next_action: "compare-host-fallback-provider-sample",
            detail:
                "selected host-simulated provider runner; deterministic output anchor is comparable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::provider_execution_outcome;
    use crate::provider_runner_registry::ProviderRunnerAdapter;

    #[test]
    fn real_device_runner_waits_for_adapter_output() {
        let outcome = provider_execution_outcome(&ProviderRunnerAdapter {
            adapter_id: "metal.apple-silicon-gpu.real-device",
            capability_status: "registered-real-device",
            real_device_capable: true,
            kind: "metal-real-device-runner",
            execution_mode: "real-device-provider-runner",
        });

        assert_eq!(outcome.status, "real-device-runner-selected");
        assert_eq!(outcome.comparison_status, "awaiting-real-device-output");
        assert_eq!(
            outcome.output_payload_status,
            "awaiting-provider-output-payload"
        );
        assert_eq!(outcome.next_action, "execute-real-device-provider-sample");
    }

    #[test]
    fn host_fallback_runner_exposes_comparable_anchor() {
        let outcome = provider_execution_outcome(&ProviderRunnerAdapter {
            adapter_id: "generic.device.host-simulated",
            capability_status: "registered-host-simulated",
            real_device_capable: false,
            kind: "generic-host-simulated-runner",
            execution_mode: "host-simulated-provider-runner",
        });

        assert_eq!(outcome.status, "host-fallback-runner-selected");
        assert_eq!(outcome.comparison_status, "host-fallback-output-comparable");
        assert_eq!(
            outcome.output_payload_status,
            "host-fallback-output-payload-ready"
        );
        assert_eq!(outcome.next_action, "compare-host-fallback-provider-sample");
    }
}
