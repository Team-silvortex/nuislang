use nuis_artifact::{
    BridgeRegistryEntry, BuildManifestDomainBuildUnit, DomainBuildUnitPayloadBlob,
    DomainBuildUnitPayloadBlobSection, HostBridgePlanEntry,
};

use crate::{
    DomainAdapter, ExecutionPhaseAction, ExecutionPhaseContext, ExecutionPhaseOutcome,
    ExecutionResourceBinding, PreparedDomainExecution,
};

use super::super::{
    domain_resource_capability_label, slot_resource_capability_label, slot_resource_kind,
    ExecutionResourceKind,
};

pub(super) struct NetworkAdapter;
pub(super) struct PassiveAdapter;

impl DomainAdapter for NetworkAdapter {
    fn adapter_id(&self) -> &'static str {
        "network-test-adapter"
    }

    fn supports(&self, unit: &BuildManifestDomainBuildUnit) -> bool {
        unit.domain_family == "network"
    }

    fn phase_action(&self, ctx: &ExecutionPhaseContext<'_>) -> Option<ExecutionPhaseAction> {
        Some(ExecutionPhaseAction {
            kind: format!("network.{}", ctx.phase),
            input_handles: match ctx.phase {
                "bind" => vec!["authority.text".to_owned()],
                "submit" => vec!["session.handle".to_owned(), "request.packet".to_owned()],
                "wait" => vec!["task.handle".to_owned()],
                "finalize" => vec!["response.handle".to_owned()],
                _ => vec!["phase.input".to_owned()],
            },
            resolved_inputs: Vec::new(),
            output_handles: match ctx.phase {
                "bind" => vec!["session.handle".to_owned()],
                "submit" => vec!["task.handle".to_owned()],
                "wait" => vec!["response.handle".to_owned()],
                "finalize" => vec!["status.code".to_owned()],
                _ => vec!["phase.output".to_owned()],
            },
            resource_bindings: vec![
                ExecutionResourceBinding {
                    key: "bridge_surface".to_owned(),
                    kind: ExecutionResourceKind::Bridge,
                    capability_label: Some(domain_resource_capability_label(
                        ctx.domain_family,
                        ctx.selected_lowering_target,
                        "bridge_surface",
                        &ExecutionResourceKind::Bridge,
                    )),
                    value: ctx.bridge_surface.to_owned(),
                },
                ExecutionResourceBinding {
                    key: "scheduler_binding".to_owned(),
                    kind: ExecutionResourceKind::Scheduler,
                    capability_label: Some(domain_resource_capability_label(
                        ctx.domain_family,
                        ctx.selected_lowering_target,
                        "scheduler_binding",
                        &ExecutionResourceKind::Scheduler,
                    )),
                    value: ctx.scheduler_binding.to_owned(),
                },
                ExecutionResourceBinding {
                    key: "backend_summary".to_owned(),
                    kind: ExecutionResourceKind::Metadata,
                    capability_label: Some(domain_resource_capability_label(
                        ctx.domain_family,
                        ctx.selected_lowering_target,
                        "backend_summary",
                        &ExecutionResourceKind::Metadata,
                    )),
                    value: ctx.backend_summary.to_owned(),
                },
                ExecutionResourceBinding {
                    key: "active_session".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        ctx.domain_family,
                        ctx.selected_lowering_target,
                        "active_session",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "slot:session.handle".to_owned(),
                },
                ExecutionResourceBinding {
                    key: "active_task".to_owned(),
                    kind: ExecutionResourceKind::Handle,
                    capability_label: Some(domain_resource_capability_label(
                        ctx.domain_family,
                        ctx.selected_lowering_target,
                        "active_task",
                        &ExecutionResourceKind::Handle,
                    )),
                    value: "slot:task.handle".to_owned(),
                },
                ExecutionResourceBinding {
                    key: "active_response".to_owned(),
                    kind: ExecutionResourceKind::Response,
                    capability_label: Some(domain_resource_capability_label(
                        ctx.domain_family,
                        ctx.selected_lowering_target,
                        "active_response",
                        &ExecutionResourceKind::Response,
                    )),
                    value: "slot:response.handle".to_owned(),
                },
            ],
            resolved_resources: Vec::new(),
            scheduler_keys: vec![
                ctx.scheduler_binding.to_owned(),
                ctx.phase.to_owned(),
                "network".to_owned(),
            ],
            adapter_hint: Some(match ctx.phase {
                "bind" => "adapter.bind.session-open".to_owned(),
                "submit" => "adapter.submit.request-dispatch".to_owned(),
                "wait" => "adapter.wait.callback-poll".to_owned(),
                "finalize" => "adapter.finalize.response-commit".to_owned(),
                _ => "adapter.execute.generic".to_owned(),
            }),
        })
    }

    fn phase_outcome(
        &self,
        ctx: &ExecutionPhaseContext<'_>,
        action: &ExecutionPhaseAction,
    ) -> Option<ExecutionPhaseOutcome> {
        Some(ExecutionPhaseOutcome {
            status: format!("adapter-{}", ctx.phase),
            produced_handles: action.output_handles.clone(),
            produced_slots: action
                .output_handles
                .iter()
                .map(|key| ExecutionResourceBinding {
                    key: key.clone(),
                    kind: slot_resource_kind(key),
                    capability_label: Some(slot_resource_capability_label(key)),
                    value: format!("network://{}/{}", ctx.phase, key),
                })
                .collect(),
            notes: vec![
                format!("domain={}", ctx.domain_family),
                format!("kind={}", action.kind),
            ],
        })
    }
}

impl DomainAdapter for PassiveAdapter {
    fn adapter_id(&self) -> &'static str {
        "passive-adapter"
    }

    fn supports(&self, _unit: &BuildManifestDomainBuildUnit) -> bool {
        true
    }
}

pub(super) fn prepared_network_execution<'a>(
    adapter: &'a dyn DomainAdapter,
    payload_blob: &'a DomainBuildUnitPayloadBlob,
    host_plan: &'a HostBridgePlanEntry,
    bridge_registry: &'a BridgeRegistryEntry,
    unit: &'a BuildManifestDomainBuildUnit,
) -> PreparedDomainExecution<'a> {
    PreparedDomainExecution {
        unit,
        payload_blob: Some(payload_blob),
        adapter,
        bridge_registry_entry: Some(bridge_registry),
        host_bridge_plan_entry: Some(host_plan),
        clock_domain: None,
        clock_edges: Vec::new(),
    }
}

pub(super) fn sample_network_unit() -> BuildManifestDomainBuildUnit {
    BuildManifestDomainBuildUnit {
        package_id: "official.network".to_owned(),
        domain_family: "network".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("urlsession".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("socket-io".to_owned()),
        target_device: Some("urlsession-stack".to_owned()),
        ir_format: Some("host-ffi-plan".to_owned()),
        dispatch_abi: Some("nuis-host-call".to_owned()),
        backend_priority: Some(700),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("urlsession.socket-io".to_owned()),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
        contract_family: "nustar.network".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
    }
}

pub(super) fn sample_network_payload() -> DomainBuildUnitPayloadBlob {
    DomainBuildUnitPayloadBlob {
        domain_family: "network".to_owned(),
        package_id: "official.network".to_owned(),
        backend_family: Some("urlsession".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("socket-io".to_owned()),
        target_device: Some("urlsession-stack".to_owned()),
        ir_format: Some("host-ffi-plan".to_owned()),
        dispatch_abi: Some("nuis-host-call".to_owned()),
        backend_priority: Some(700),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("urlsession.socket-io".to_owned()),
        contract_family: "nustar.network".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        payload_kind: "contract-sidecar".to_owned(),
        payload_format: "toml".to_owned(),
        sections: vec![
            DomainBuildUnitPayloadBlobSection {
                name: "lowering_plan".to_owned(),
                bytes: b"execution_route = \"foundation-session-reactor\"\nphase_bind = \"socket-bind-or-session-open\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "backend_stub".to_owned(),
                bytes: b"transport_ir = \"foundation-url-request\"\ntransport_entry_model = \"urlsession-task\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "bridge_plan".to_owned(),
                bytes: b"phase_submit = \"packet-write-dispatch\"\nphase_wait = \"callback-or-read-ready\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "network_ir_sidecar".to_owned(),
                bytes: b"schema = \"nuis-network-ir-sidecar-v1\"\nrequest = \"http-client-session\"".to_vec(),
            },
        ],
    }
}

pub(super) fn sample_network_host_plan() -> HostBridgePlanEntry {
    let bridge_stub_path = std::env::temp_dir().join("network.bridge.stub.txt");
    HostBridgePlanEntry {
        domain_family: "network".to_owned(),
        package_id: "official.network".to_owned(),
        bridge_stub_path: bridge_stub_path.display().to_string(),
        bridge_surface: "host-ffi.bridge.network".to_owned(),
        scheduler_binding: "network-poll-bridge".to_owned(),
        phase_order: vec![
            "bind".to_owned(),
            "submit".to_owned(),
            "wait".to_owned(),
            "finalize".to_owned(),
        ],
        plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
    }
}

pub(super) fn sample_network_bridge_registry() -> BridgeRegistryEntry {
    let bridge_stub_path = std::env::temp_dir().join("network.bridge.stub.txt");
    let payload_blob_path = std::env::temp_dir().join("network.payload.bin");

    BridgeRegistryEntry {
        domain_family: "network".to_owned(),
        package_id: "official.network".to_owned(),
        backend_family: "urlsession".to_owned(),
        selected_lowering_target: "urlsession.socket-io".to_owned(),
        bridge_stub_path: bridge_stub_path.display().to_string(),
        payload_blob_path: payload_blob_path.display().to_string(),
        plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
    }
}

pub(super) fn sample_kernel_unit() -> BuildManifestDomainBuildUnit {
    BuildManifestDomainBuildUnit {
        package_id: "official.kernel".to_owned(),
        domain_family: "kernel".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("vulkan".to_owned()),
        vendor: Some("cross-vendor".to_owned()),
        device_class: Some("discrete-or-integrated-gpu".to_owned()),
        target_device: Some("vulkan-device".to_owned()),
        ir_format: Some("spirv".to_owned()),
        dispatch_abi: Some("vulkan-compute-pipeline".to_owned()),
        backend_priority: Some(30),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("vulkan.discrete-or-integrated-gpu".to_owned()),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
        contract_family: "nustar.kernel".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
    }
}

pub(super) fn sample_kernel_payload() -> DomainBuildUnitPayloadBlob {
    DomainBuildUnitPayloadBlob {
        domain_family: "kernel".to_owned(),
        package_id: "official.kernel".to_owned(),
        backend_family: Some("vulkan".to_owned()),
        vendor: Some("cross-vendor".to_owned()),
        device_class: Some("discrete-or-integrated-gpu".to_owned()),
        target_device: Some("vulkan-device".to_owned()),
        ir_format: Some("spirv".to_owned()),
        dispatch_abi: Some("vulkan-compute-pipeline".to_owned()),
        backend_priority: Some(30),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("vulkan.discrete-or-integrated-gpu".to_owned()),
        contract_family: "nustar.kernel".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        payload_kind: "contract-sidecar".to_owned(),
        payload_format: "toml".to_owned(),
        sections: vec![
            DomainBuildUnitPayloadBlobSection {
                name: "lowering_plan".to_owned(),
                bytes: b"dispatch_shape = \"grid-launch\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "backend_stub".to_owned(),
                bytes: b"kernel_ir = \"spirv1.6\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "bridge_plan".to_owned(),
                bytes: b"phase_submit = \"queue-dispatch-submit\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "kernel_ir_sidecar".to_owned(),
                bytes: b"schema = \"nuis-kernel-ir-sidecar-v1\"\n[lowering_capabilities]\ncapability_owner = \"kernel-nustar\"\nnative_ir = \"spirv1.6\"\ntensor_lowering = \"storage-buffer-tensor-view\"\ndispatch_lowering = \"compute-grid-or-indirect\"\nresult_lowering = \"storage-buffer-result\"".to_vec(),
            },
        ],
    }
}

pub(super) fn sample_shader_unit() -> BuildManifestDomainBuildUnit {
    BuildManifestDomainBuildUnit {
        package_id: "official.shader".to_owned(),
        domain_family: "shader".to_owned(),
        abi: None,
        machine_arch: None,
        machine_os: None,
        backend_family: Some("metal".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("apple-silicon-gpu".to_owned()),
        target_device: Some("apple-gpu".to_owned()),
        ir_format: Some("msl".to_owned()),
        dispatch_abi: Some("metal-render-pipeline".to_owned()),
        backend_priority: Some(10),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
    }
}

pub(super) fn sample_shader_payload() -> DomainBuildUnitPayloadBlob {
    DomainBuildUnitPayloadBlob {
        domain_family: "shader".to_owned(),
        package_id: "official.shader".to_owned(),
        backend_family: Some("metal".to_owned()),
        vendor: Some("apple".to_owned()),
        device_class: Some("apple-silicon-gpu".to_owned()),
        target_device: Some("apple-gpu".to_owned()),
        ir_format: Some("msl".to_owned()),
        dispatch_abi: Some("metal-render-pipeline".to_owned()),
        backend_priority: Some(10),
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
        contract_family: "nustar.shader".to_owned(),
        packaging_role: "hetero-contract".to_owned(),
        payload_kind: "contract-sidecar".to_owned(),
        payload_format: "toml".to_owned(),
        sections: vec![
            DomainBuildUnitPayloadBlobSection {
                name: "lowering_plan".to_owned(),
                bytes: b"dispatch_encoding_model = \"tile-and-threadgroup\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "backend_stub".to_owned(),
                bytes: b"shader_ir = \"msl2.4\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "bridge_plan".to_owned(),
                bytes: b"phase_submit = \"render-submit-bridge\"".to_vec(),
            },
            DomainBuildUnitPayloadBlobSection {
                name: "shader_ir_sidecar".to_owned(),
                bytes: b"schema = \"nuis-shader-ir-sidecar-v1\"\n[lowering_capabilities]\ncapability_owner = \"shader-nustar\"\nnative_ir = \"msl2.4\"\npipeline_lowering = \"metal-render-pipeline-state\"\nresource_lowering = \"argument-buffer-table\"\ntexture_lowering = \"texture2d-sampler-argument\"".to_vec(),
            },
        ],
    }
}

pub(super) fn sample_host_plan(
    domain_family: &str,
    package_id: &str,
    scheduler: &str,
) -> HostBridgePlanEntry {
    let bridge_stub_path = std::env::temp_dir()
        .join(format!("nuis_runtime_host_plan_{domain_family}"))
        .join(format!("{domain_family}.bridge.stub.txt"));
    HostBridgePlanEntry {
        domain_family: domain_family.to_owned(),
        package_id: package_id.to_owned(),
        bridge_stub_path: bridge_stub_path.display().to_string(),
        bridge_surface: format!("host-ffi.bridge.{domain_family}"),
        scheduler_binding: scheduler.to_owned(),
        phase_order: vec![
            "bind".to_owned(),
            "submit".to_owned(),
            "wait".to_owned(),
            "finalize".to_owned(),
        ],
        plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
    }
}

pub(super) fn sample_bridge_registry(
    domain_family: &str,
    package_id: &str,
    backend: &str,
    target: &str,
) -> BridgeRegistryEntry {
    let base_dir =
        std::env::temp_dir().join(format!("nuis_runtime_bridge_registry_{domain_family}"));
    let bridge_stub_path = base_dir.join(format!("{domain_family}.bridge.stub.txt"));
    let payload_blob_path = base_dir.join(format!("{domain_family}.payload.bin"));

    BridgeRegistryEntry {
        domain_family: domain_family.to_owned(),
        package_id: package_id.to_owned(),
        backend_family: backend.to_owned(),
        selected_lowering_target: target.to_owned(),
        bridge_stub_path: bridge_stub_path.display().to_string(),
        payload_blob_path: payload_blob_path.display().to_string(),
        plan_inline: "bridge_kind = \"managed-lifecycle-bridge\"".to_owned(),
    }
}
