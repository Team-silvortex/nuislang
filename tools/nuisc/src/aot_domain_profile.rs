use nuis_artifact::BuildManifestDomainBuildUnit;

pub(crate) struct DerivedLoweringProfile<'a> {
    pub(crate) profile_key: &'a str,
    pub(crate) execution_route: &'static str,
    pub(crate) submission_adapter: &'static str,
    pub(crate) wake_adapter: &'static str,
}

pub(crate) fn derived_lowering_profile_for_unit<'a>(
    unit: &'a BuildManifestDomainBuildUnit,
) -> DerivedLoweringProfile<'a> {
    let profile_key = unit
        .selected_lowering_target
        .as_deref()
        .or(unit.backend_family.as_deref())
        .unwrap_or("none");
    let (execution_route, submission_adapter, wake_adapter) =
        match (unit.domain_family.as_str(), profile_key) {
            ("shader", "metal.apple-silicon-gpu") => (
                "unified-render-graph",
                "metal-command-encoder",
                "metal-shared-event",
            ),
            ("shader", "metal.mac-discrete-or-integrated-gpu") => (
                "render-graph-device-queue",
                "metal-command-buffer",
                "metal-frame-fence",
            ),
            ("shader", "vulkan.discrete-or-integrated-gpu") => (
                "spirv-render-queue",
                "vulkan-command-buffer",
                "vulkan-timeline-semaphore",
            ),
            ("shader", "directx.discrete-or-integrated-gpu") => {
                ("dxil-render-queue", "directx-command-list", "directx-fence")
            }
            ("shader", "opengl.discrete-or-integrated-gpu") => (
                "driver-managed-render-pipeline",
                "opengl-driver-submit",
                "gl-sync-object",
            ),
            ("shader", "cpu-fallback.cpu-host") => (
                "host-render-fallback",
                "cpu-raster-dispatch",
                "host-frame-complete",
            ),
            ("kernel", "coreml.apple-ane") => (
                "ane-graph-execution",
                "coreml-graph-submit",
                "coreml-completion-callback",
            ),
            ("kernel", "vulkan.discrete-or-integrated-gpu") => (
                "spirv-compute-queue",
                "vulkan-compute-submit",
                "vulkan-timeline-semaphore",
            ),
            ("kernel", "cpu-fallback.cpu-host") => (
                "host-kernel-fallback",
                "cpu-threadpool-dispatch",
                "host-join-wake",
            ),
            ("network", "urlsession.socket-io") => (
                "foundation-session-reactor",
                "urlsession-task-submit",
                "urlsession-callback",
            ),
            ("network", "winsock.socket-io") => (
                "windows-socket-reactor",
                "winsock-overlapped-submit",
                "iocp-ready",
            ),
            ("network", "socket-abi.socket-io") => (
                "posix-socket-reactor",
                "socket-send-recv-submit",
                "poll-ready",
            ),
            ("shader", _) => (
                "generic-render-dispatch",
                "render-submit-bridge",
                "frame-present",
            ),
            ("kernel", _) => (
                "generic-accelerator-dispatch",
                "hetero-submit-bridge",
                "completion-fence",
            ),
            ("network", _) => ("generic-io-reactor", "network-poll-bridge", "io-ready"),
            ("cpu", _) | ("host", _) => ("host-inline-call", "direct-call", "immediate"),
            _ => ("generic-dispatch", "generic-submit", "generic-wake"),
        };
    DerivedLoweringProfile {
        profile_key,
        execution_route,
        submission_adapter,
        wake_adapter,
    }
}

pub(crate) fn render_target_specific_backend_fields(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> String {
    let mut out = String::new();
    match (unit.domain_family.as_str(), profile.profile_key) {
        ("shader", "metal.apple-silicon-gpu") => {
            out.push_str("shader_ir = \"msl2.4\"\n");
            out.push_str("shader_entry_model = \"metal-function-constant-specialized\"\n");
            out.push_str("queue_binding_model = \"unified-command-queue\"\n");
            out.push_str("resource_binding_model = \"argument-buffer-table\"\n");
        }
        ("shader", "metal.mac-discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"msl2.4\"\n");
            out.push_str("shader_entry_model = \"metal-pipeline-state\"\n");
            out.push_str("queue_binding_model = \"device-command-queue\"\n");
            out.push_str("resource_binding_model = \"descriptor-table-emulated\"\n");
        }
        ("shader", "vulkan.discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"spirv1.6\"\n");
            out.push_str("shader_entry_model = \"vulkan-pipeline\"\n");
            out.push_str("queue_binding_model = \"explicit-device-queue\"\n");
            out.push_str("resource_binding_model = \"descriptor-set-layout\"\n");
        }
        ("shader", "directx.discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"dxil6.8\"\n");
            out.push_str("shader_entry_model = \"directx-pipeline-state\"\n");
            out.push_str("queue_binding_model = \"command-queue\"\n");
            out.push_str("resource_binding_model = \"root-signature\"\n");
        }
        ("shader", "opengl.discrete-or-integrated-gpu") => {
            out.push_str("shader_ir = \"glsl460\"\n");
            out.push_str("shader_entry_model = \"driver-linked-program\"\n");
            out.push_str("queue_binding_model = \"driver-managed\"\n");
            out.push_str("resource_binding_model = \"uniform-slot-table\"\n");
        }
        ("shader", "cpu-fallback.cpu-host") => {
            out.push_str("shader_ir = \"host-simd\"\n");
            out.push_str("shader_entry_model = \"cpu-raster-kernel\"\n");
            out.push_str("queue_binding_model = \"threadpool-work-queue\"\n");
            out.push_str("resource_binding_model = \"host-buffer-slices\"\n");
        }
        ("kernel", "coreml.apple-ane") => {
            out.push_str("kernel_ir = \"coreml-program\"\n");
            out.push_str("kernel_entry_model = \"mlmodelc-function\"\n");
            out.push_str("queue_binding_model = \"ane-submission-service\"\n");
            out.push_str("resource_binding_model = \"tensor-argument-table\"\n");
        }
        ("kernel", "vulkan.discrete-or-integrated-gpu") => {
            out.push_str("kernel_ir = \"spirv1.6\"\n");
            out.push_str("kernel_entry_model = \"compute-pipeline\"\n");
            out.push_str("queue_binding_model = \"compute-queue\"\n");
            out.push_str("resource_binding_model = \"descriptor-set-layout\"\n");
        }
        ("kernel", "cpu-fallback.cpu-host") => {
            out.push_str("kernel_ir = \"host-simd\"\n");
            out.push_str("kernel_entry_model = \"threadpool-kernel\"\n");
            out.push_str("queue_binding_model = \"host-work-queue\"\n");
            out.push_str("resource_binding_model = \"host-buffer-slices\"\n");
        }
        ("network", "urlsession.socket-io") => {
            out.push_str("transport_ir = \"foundation-url-request\"\n");
            out.push_str("transport_entry_model = \"urlsession-task\"\n");
            out.push_str("socket_binding_model = \"session-owned-socket\"\n");
            out.push_str("completion_binding_model = \"delegate-callback\"\n");
        }
        ("network", "winsock.socket-io") => {
            out.push_str("transport_ir = \"winsock-overlapped\"\n");
            out.push_str("transport_entry_model = \"iocp-request\"\n");
            out.push_str("socket_binding_model = \"overlapped-socket-handle\"\n");
            out.push_str("completion_binding_model = \"iocp-completion-port\"\n");
        }
        ("network", "socket-abi.socket-io") => {
            out.push_str("transport_ir = \"posix-socket\"\n");
            out.push_str("transport_entry_model = \"poll-reactor-request\"\n");
            out.push_str("socket_binding_model = \"fd-owned-session\"\n");
            out.push_str("completion_binding_model = \"poll-ready-token\"\n");
        }
        _ => {}
    }
    out
}

pub(crate) fn render_target_specific_lowering_fields(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> String {
    let mut out = String::new();
    match (unit.domain_family.as_str(), profile.profile_key) {
        ("shader", "metal.apple-silicon-gpu") => {
            out.push_str("lowering_ir = \"msl2.4\"\n");
            out.push_str("shader_stage_model = \"metal-render-pipeline\"\n");
            out.push_str("stage_binding_model = \"argument-buffer-specialized\"\n");
            out.push_str("dispatch_encoding_model = \"tile-and-threadgroup\"\n");
        }
        ("shader", "metal.mac-discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"msl2.4\"\n");
            out.push_str("shader_stage_model = \"metal-render-pipeline\"\n");
            out.push_str("stage_binding_model = \"descriptor-table-emulated\"\n");
            out.push_str("dispatch_encoding_model = \"device-queue-encoder\"\n");
        }
        ("shader", "vulkan.discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"spirv1.6\"\n");
            out.push_str("shader_stage_model = \"spirv-graphics-pipeline\"\n");
            out.push_str("stage_binding_model = \"descriptor-set-layout\"\n");
            out.push_str("dispatch_encoding_model = \"renderpass-command-buffer\"\n");
        }
        ("shader", "directx.discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"dxil6.8\"\n");
            out.push_str("shader_stage_model = \"dxil-pso\"\n");
            out.push_str("stage_binding_model = \"root-signature\"\n");
            out.push_str("dispatch_encoding_model = \"command-list-recording\"\n");
        }
        ("shader", "opengl.discrete-or-integrated-gpu") => {
            out.push_str("lowering_ir = \"glsl460\"\n");
            out.push_str("shader_stage_model = \"linked-program-pipeline\"\n");
            out.push_str("stage_binding_model = \"uniform-slot-table\"\n");
            out.push_str("dispatch_encoding_model = \"driver-issued-draw\"\n");
        }
        ("shader", "cpu-fallback.cpu-host") => {
            out.push_str("lowering_ir = \"host-simd\"\n");
            out.push_str("shader_stage_model = \"cpu-raster-pipeline\"\n");
            out.push_str("stage_binding_model = \"host-buffer-slices\"\n");
            out.push_str("dispatch_encoding_model = \"threadpool-tile-dispatch\"\n");
        }
        ("network", "urlsession.socket-io") => {
            out.push_str("lowering_ir = \"foundation-url-request\"\n");
            out.push_str("transport_binding_model = \"session-task-packet\"\n");
            out.push_str("completion_encoding_model = \"delegate-callback-lifecycle\"\n");
        }
        ("network", "winsock.socket-io") => {
            out.push_str("lowering_ir = \"winsock-overlapped\"\n");
            out.push_str("transport_binding_model = \"overlapped-packet-reactor\"\n");
            out.push_str("completion_encoding_model = \"iocp-completion-lifecycle\"\n");
        }
        ("network", "socket-abi.socket-io") => {
            out.push_str("lowering_ir = \"posix-socket\"\n");
            out.push_str("transport_binding_model = \"packet-poll-reactor\"\n");
            out.push_str("completion_encoding_model = \"poll-ready-lifecycle\"\n");
        }
        _ => {}
    }
    out
}

pub(crate) fn render_schedule_contract_fields(profile: &DerivedLoweringProfile<'_>) -> String {
    let mut out = String::new();
    out.push_str("[schedule_contract]\n");
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        profile.execution_route
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        profile.submission_adapter
    ));
    out.push_str(&format!("wake_adapter = \"{}\"\n", profile.wake_adapter));
    out.push_str("clock_contract = \"global-time-partial-order\"\n");
    out.push_str("completion_contract = \"lifecycle-hook-fence\"\n");
    out.push_str("data_order_contract = \"deterministic-segment-order\"\n");
    out
}

pub(crate) fn shader_supported_stages_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match (unit.domain_family.as_str(), profile.profile_key) {
        (
            "shader",
            "metal.apple-silicon-gpu"
            | "metal.mac-discrete-or-integrated-gpu"
            | "vulkan.discrete-or-integrated-gpu"
            | "directx.discrete-or-integrated-gpu"
            | "opengl.discrete-or-integrated-gpu"
            | "cpu-fallback.cpu-host",
        ) => Some(&["vertex", "fragment", "compute"]),
        ("shader", _) => Some(&["fragment"]),
        _ => None,
    }
}

pub(crate) fn shader_registered_feature_surfaces_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    _profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match unit.domain_family.as_str() {
        "shader" => Some(&[
            "shader.profile.target.v1",
            "shader.profile.viewport.v1",
            "shader.profile.pipeline.v1",
            "shader.profile.texture.v1",
            "shader.profile.sampler.v1",
            "shader.profile.bind-set.v1",
            "shader.profile.attachment.v1",
            "shader.profile.sample-path.v1",
        ]),
        _ => None,
    }
}

pub(crate) fn shader_registered_lane_groups_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    _profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match unit.domain_family.as_str() {
        "shader" => Some(&["setup", "resource", "render"]),
        _ => None,
    }
}

pub(crate) fn kernel_supported_dispatch_kinds_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match (unit.domain_family.as_str(), profile.profile_key) {
        ("kernel", "coreml.apple-ane") => Some(&["graph", "batch", "tile"]),
        ("kernel", "vulkan.discrete-or-integrated-gpu") => Some(&["grid", "indirect", "batch"]),
        ("kernel", "cpu-fallback.cpu-host") => Some(&["range", "tile", "batch"]),
        ("kernel", _) => Some(&["graph"]),
        _ => None,
    }
}

pub(crate) fn kernel_registered_feature_surfaces_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    _profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match unit.domain_family.as_str() {
        "kernel" => Some(&[
            "kernel.profile.bind-core.v1",
            "kernel.profile.queue.v1",
            "kernel.profile.batch-lanes.v1",
            "kernel.profile.entry.v1",
            "kernel.profile.tensor-shape.v1",
            "kernel.profile.tensor-broadcast.v1",
            "kernel.profile.tensor-reduce.v1",
            "kernel.profile.tensor-selection.v1",
            "kernel.profile.tensor-sort.v1",
            "kernel.profile.tensor-topk.v1",
            "kernel.profile.dispatch-grid.v1",
            "kernel.profile.result-buffer.v1",
        ]),
        _ => None,
    }
}

pub(crate) fn kernel_registered_lane_groups_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    _profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    match unit.domain_family.as_str() {
        "kernel" => Some(&[
            "setup", "memory", "compute", "shape", "reduce", "select", "debug",
        ]),
        _ => None,
    }
}

pub(crate) fn registered_feature_surfaces_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    shader_registered_feature_surfaces_for_profile(unit, profile)
        .or_else(|| kernel_registered_feature_surfaces_for_profile(unit, profile))
}

pub(crate) fn registered_lane_groups_for_profile(
    unit: &BuildManifestDomainBuildUnit,
    profile: &DerivedLoweringProfile<'_>,
) -> Option<&'static [&'static str]> {
    shader_registered_lane_groups_for_profile(unit, profile)
        .or_else(|| kernel_registered_lane_groups_for_profile(unit, profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain_unit(domain: &str, target: &str) -> BuildManifestDomainBuildUnit {
        BuildManifestDomainBuildUnit {
            package_id: format!("official.{domain}"),
            domain_family: domain.to_owned(),
            abi: None,
            machine_arch: None,
            machine_os: None,
            backend_family: None,
            vendor: None,
            device_class: None,
            target_device: None,
            ir_format: None,
            dispatch_abi: None,
            backend_priority: None,
            verification: None,
            selected_lowering_target: Some(target.to_owned()),
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
            contract_family: format!("nustar.{domain}"),
            packaging_role: "hetero-contract".to_owned(),
        }
    }

    #[test]
    fn coreml_profile_keeps_ane_route_and_dispatch_kinds() {
        let unit = domain_unit("kernel", "coreml.apple-ane");
        let profile = derived_lowering_profile_for_unit(&unit);

        assert_eq!(profile.execution_route, "ane-graph-execution");
        assert_eq!(profile.submission_adapter, "coreml-graph-submit");
        assert_eq!(profile.wake_adapter, "coreml-completion-callback");
        assert_eq!(
            kernel_supported_dispatch_kinds_for_profile(&unit, &profile),
            Some(&["graph", "batch", "tile"][..])
        );
    }

    #[test]
    fn metal_shader_profile_keeps_stage_contract() {
        let unit = domain_unit("shader", "metal.apple-silicon-gpu");
        let profile = derived_lowering_profile_for_unit(&unit);

        assert_eq!(profile.execution_route, "unified-render-graph");
        assert_eq!(
            shader_supported_stages_for_profile(&unit, &profile),
            Some(&["vertex", "fragment", "compute"][..])
        );
    }

    #[test]
    fn schedule_contract_renders_profile_owned_adapters() {
        let unit = domain_unit("kernel", "vulkan.discrete-or-integrated-gpu");
        let profile = derived_lowering_profile_for_unit(&unit);
        let rendered = render_schedule_contract_fields(&profile);

        assert!(rendered.contains("[schedule_contract]"));
        assert!(rendered.contains("execution_route = \"spirv-compute-queue\""));
        assert!(rendered.contains("submission_adapter = \"vulkan-compute-submit\""));
        assert!(rendered.contains("wake_adapter = \"vulkan-timeline-semaphore\""));
        assert!(rendered.contains("clock_contract = \"global-time-partial-order\""));
    }
}
