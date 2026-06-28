use nuis_artifact::{NuisExecutableEnvelope, NuisLifecycleContract};

pub(crate) struct NuisEnvelopeDomainSummary {
    pub domain_family: String,
    pub contract_family: String,
    pub function_kind: String,
    pub graph_kind: String,
    pub default_time_mode: String,
}

pub(crate) fn build_nuis_envelope(
    domains: &[NuisEnvelopeDomainSummary],
    packaging_mode: &str,
) -> NuisExecutableEnvelope {
    let mut domain_families = domains
        .iter()
        .map(|item| item.domain_family.clone())
        .collect::<Vec<_>>();
    domain_families.sort();
    domain_families.dedup();

    let mut contract_families = domains
        .iter()
        .map(|item| item.contract_family.clone())
        .collect::<Vec<_>>();
    contract_families.sort();
    contract_families.dedup();

    let function_kind = domains
        .first()
        .map(|item| item.function_kind.clone())
        .unwrap_or_else(|| "function-node".to_owned());
    let graph_kind = domains
        .first()
        .map(|item| item.graph_kind.clone())
        .unwrap_or_else(|| "function-graph".to_owned());
    let default_time_mode = domains
        .first()
        .map(|item| item.default_time_mode.clone())
        .unwrap_or_else(|| "logical".to_owned());

    NuisExecutableEnvelope {
        schema: "nuis-executable-envelope-v1".to_owned(),
        executable_kind: packaging_mode.to_owned(),
        package_count: domains.len(),
        domain_families,
        contract_families,
        function_kind,
        graph_kind,
        default_time_mode,
    }
}

pub(crate) fn build_nuis_lifecycle_contract(
    envelope: &NuisExecutableEnvelope,
    packaging_mode: &str,
) -> NuisLifecycleContract {
    let mut hook_surface = vec![
        "on_bridge_bind".to_owned(),
        "on_scheduler_tick".to_owned(),
        "on_task_poll".to_owned(),
        "on_result_commit".to_owned(),
        "on_summary_flush".to_owned(),
        "on_managed_rpc".to_owned(),
        "on_shutdown_prepare".to_owned(),
    ];
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.network")
    {
        hook_surface.push("on_network_bridge_progress".to_owned());
    }
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.shader" || family == "nustar.kernel")
    {
        hook_surface.push("on_hetero_submission_progress".to_owned());
    }
    let mut export_surface = vec![
        "nuis_lifecycle_bootstrap_export_v1".to_owned(),
        "nuis_lifecycle_tick_export_v1".to_owned(),
        "nuis_lifecycle_shutdown_export_v1".to_owned(),
        "nuis_lifecycle_yalivia_rpc_export_v1".to_owned(),
    ];
    let mut runtime_capability_flags = vec![
        "runtime.bootstrap".to_owned(),
        "runtime.tick".to_owned(),
        "runtime.shutdown".to_owned(),
        "runtime.rpc.yalivia".to_owned(),
    ];
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.network")
    {
        export_surface.push("nuis_lifecycle_network_bridge_progress_export_v1".to_owned());
        runtime_capability_flags.push("runtime.progress.network".to_owned());
    }
    if envelope
        .contract_families
        .iter()
        .any(|family| family == "nustar.shader" || family == "nustar.kernel")
    {
        export_surface.push("nuis_lifecycle_hetero_submission_progress_export_v1".to_owned());
        runtime_capability_flags.push("runtime.progress.hetero".to_owned());
    }
    NuisLifecycleContract {
        schema: "nuis-lifecycle-contract-v1".to_owned(),
        bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
        tick_policy: if packaging_mode == "native-cpu-llvm" {
            "owned-pump.active-wait-drain".to_owned()
        } else {
            "owned-pump.bootstrap-adaptive".to_owned()
        },
        shutdown_policy: "flush-summaries-then-release-bridges".to_owned(),
        yalivia_rpc: "optional.lifecycle-hook-rpc.v1".to_owned(),
        hook_surface,
        export_surface,
        runtime_capability_flags,
    }
}
