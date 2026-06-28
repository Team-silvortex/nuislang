pub(crate) struct BuildReportRuntimeAdapter;

impl nuis_runtime::DomainAdapter for BuildReportRuntimeAdapter {
    fn adapter_id(&self) -> &'static str {
        "nuis-build-report-runtime-adapter"
    }

    fn supports(&self, _unit: &nuis_artifact::BuildManifestDomainBuildUnit) -> bool {
        true
    }

    fn phase_outcome(
        &self,
        ctx: &nuis_runtime::ExecutionPhaseContext<'_>,
        action: &nuis_runtime::ExecutionPhaseAction,
    ) -> Option<nuis_runtime::ExecutionPhaseOutcome> {
        if runtime_context_is_kernel_host_reference(ctx) {
            return Some(nuis_runtime::ExecutionPhaseOutcome {
                status: "kernel-host-reference-dispatch-complete".to_owned(),
                produced_handles: action.output_handles.clone(),
                produced_slots: action
                    .output_handles
                    .iter()
                    .map(|key| nuis_runtime::ExecutionResourceBinding {
                        key: key.clone(),
                        kind: runtime_slot_resource_kind(key),
                        capability_label: Some(format!(
                            "cap.kernel_host_reference.{}.{}",
                            ctx.domain_family, key
                        )),
                        value: format!(
                            "kernel-host-reference://{}/{}/{}",
                            ctx.selected_lowering_target.unwrap_or("<none>"),
                            ctx.phase,
                            key
                        ),
                    })
                    .collect(),
                notes: vec![
                    "backend_mode=host-reference".to_owned(),
                    format!(
                        "ffi_bridge={}",
                        runtime_domain_ffi_bridge(ctx.domain_family)
                    ),
                    "ffi_policy=signature-whitelist-required".to_owned(),
                    format!("ffi_symbol={}", runtime_domain_ffi_symbol(ctx)),
                    format!(
                        "device_backend_requested={}",
                        ctx.selected_lowering_target.unwrap_or("<none>")
                    ),
                    format!("phase={}", ctx.phase),
                    "kernel payload reached runtime adapter dispatch boundary".to_owned(),
                ],
            });
        }
        if !runtime_context_is_cpu_fallback(ctx) {
            return None;
        }
        Some(nuis_runtime::ExecutionPhaseOutcome {
            status: "host-cpu-fallback-complete".to_owned(),
            produced_handles: action.output_handles.clone(),
            produced_slots: action
                .output_handles
                .iter()
                .map(|key| nuis_runtime::ExecutionResourceBinding {
                    key: key.clone(),
                    kind: runtime_slot_resource_kind(key),
                    capability_label: Some(format!(
                        "cap.host_cpu_fallback.{}.{}",
                        ctx.domain_family, key
                    )),
                    value: format!(
                        "host-cpu-fallback://{}/{}/{}",
                        ctx.domain_family, ctx.phase, key
                    ),
                })
                .collect(),
            notes: vec![
                format!("domain={}", ctx.domain_family),
                format!(
                    "lowering={}",
                    ctx.selected_lowering_target.unwrap_or("<none>")
                ),
                "executed by host CPU fallback adapter scaffold".to_owned(),
            ],
        })
    }
}

fn runtime_context_is_kernel_host_reference(ctx: &nuis_runtime::ExecutionPhaseContext<'_>) -> bool {
    ctx.domain_family == "kernel" && !runtime_context_is_cpu_fallback(ctx)
}

fn runtime_domain_ffi_bridge(domain_family: &str) -> String {
    format!(
        "cffi.{}.dispatch.v1",
        runtime_symbol_component(domain_family)
    )
}

fn runtime_domain_ffi_symbol(ctx: &nuis_runtime::ExecutionPhaseContext<'_>) -> String {
    format!(
        "nuis_{}_{}_dispatch_v1",
        runtime_symbol_component(ctx.domain_family),
        runtime_symbol_component(ctx.selected_lowering_target.unwrap_or("none"))
    )
}

fn runtime_symbol_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn runtime_context_is_cpu_fallback(ctx: &nuis_runtime::ExecutionPhaseContext<'_>) -> bool {
    let backend = ctx.backend_family.unwrap_or("");
    let target = ctx.selected_lowering_target.unwrap_or("");
    backend == "cpu-fallback"
        || backend == "cpu-host"
        || target == "cpu-fallback"
        || target == "cpu-host"
        || target.starts_with("cpu-fallback.")
        || target.ends_with(".cpu-host")
}

fn runtime_slot_resource_kind(key: &str) -> nuis_runtime::ExecutionResourceKind {
    if key.ends_with(".handle") {
        nuis_runtime::ExecutionResourceKind::Handle
    } else if key.ends_with(".packet") || key.contains("packet") {
        nuis_runtime::ExecutionResourceKind::Packet
    } else if key.ends_with(".buffer") || key.contains("buffer") {
        nuis_runtime::ExecutionResourceKind::Buffer
    } else if key.ends_with(".response") || key.contains("response") || key.ends_with(".target") {
        nuis_runtime::ExecutionResourceKind::Response
    } else if key.contains("scheduler") {
        nuis_runtime::ExecutionResourceKind::Scheduler
    } else if key.contains("bridge") {
        nuis_runtime::ExecutionResourceKind::Bridge
    } else {
        nuis_runtime::ExecutionResourceKind::Slot
    }
}
