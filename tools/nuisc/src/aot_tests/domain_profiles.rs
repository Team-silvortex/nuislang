use super::*;

fn sample_domain_unit(
    domain_family: &str,
    package_id: &str,
    backend_family: &str,
    vendor: &str,
    device_class: &str,
    selected_lowering_target: &str,
) -> BuildManifestDomainBuildUnit {
    BuildManifestDomainBuildUnit {
        package_id: package_id.to_owned(),
        domain_family: domain_family.to_owned(),
        abi: None,
        machine_arch: Some("arm64".to_owned()),
        machine_os: Some("darwin".to_owned()),
        backend_family: Some(backend_family.to_owned()),
        vendor: Some(vendor.to_owned()),
        device_class: Some(device_class.to_owned()),
        target_device: Some(device_class.to_owned()),
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some(selected_lowering_target.to_owned()),
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
        contract_family: format!("nustar.{domain_family}"),
        packaging_role: "hetero-contract".to_owned(),
    }
}

#[test]
fn resolve_cpu_build_target_for_known_abis() {
    let registry_root = registry_root();
    let apple =
        resolve_cpu_build_target_from_abi(&registry_root, "cpu.arm64.apple_aapcs64").unwrap();
    assert_eq!(apple.machine_arch, "arm64");
    assert_eq!(apple.machine_os, "darwin");
    assert_eq!(apple.clang_target, "aarch64-apple-darwin");
    assert_eq!(apple.isa_family, "aarch64");
    assert!(apple.isa_features.contains(&"neon".to_owned()));
    assert!(apple.isa_features.contains(&"lse".to_owned()));

    let apple_amd64 =
        resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.apple_sysv64").unwrap();
    assert_eq!(apple_amd64.machine_arch, "x86_64");
    assert_eq!(apple_amd64.machine_os, "darwin");
    assert_eq!(apple_amd64.object_format, "mach-o");
    assert_eq!(apple_amd64.calling_abi, "sysv64");
    assert_eq!(apple_amd64.clang_target, "x86_64-apple-darwin");
    assert_eq!(apple_amd64.isa_family, "x86_64");
    assert!(apple_amd64.isa_features.contains(&"sse2".to_owned()));
    assert!(apple_amd64.isa_features.contains(&"avx2".to_owned()));

    let linux = resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.sysv64").unwrap();
    assert_eq!(linux.machine_arch, "x86_64");
    assert_eq!(linux.machine_os, "linux");
    assert_eq!(linux.object_format, "elf");
    assert_eq!(linux.calling_abi, "sysv64");
    assert_eq!(linux.isa_family, "x86_64");
    assert!(linux.isa_features.contains(&"bmi2".to_owned()));

    let windows = resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.win64").unwrap();
    assert_eq!(windows.machine_os, "windows");
    assert_eq!(windows.clang_target, "x86_64-pc-windows-msvc");
    assert_eq!(windows.isa_family, "x86_64");
    assert!(windows.isa_features.contains(&"sse4.2".to_owned()));
    assert!(!windows.isa_features.contains(&"avx2".to_owned()));
}

#[test]
fn shader_lowering_and_stub_include_profile_aware_fields() {
    let shader_unit = sample_domain_unit(
        "shader",
        "official.shader",
        "metal",
        "apple",
        "apple-silicon-gpu",
        "metal.apple-silicon-gpu",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
    let backend_stub = super::render_domain_build_unit_backend_stub(&shader_unit);
    let host_bridge_stub = super::render_domain_build_unit_host_bridge_stub(&shader_unit);

    assert!(lowering_plan.contains("lowering_profile = \"metal.apple-silicon-gpu\""));
    assert!(lowering_plan.contains("execution_route = \"unified-render-graph\""));
    assert!(lowering_plan.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(lowering_plan.contains("wake_adapter = \"metal-shared-event\""));
    assert!(lowering_plan.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]"));
    assert!(lowering_plan.contains("shader.profile.texture.v1"));
    assert!(lowering_plan.contains("shader.profile.sample-path.v1"));
    assert!(
        lowering_plan.contains("registered_lane_groups = [\"setup\", \"resource\", \"render\"]")
    );
    assert!(lowering_plan.contains("lowering_ir = \"msl2.4\""));
    assert!(lowering_plan.contains("shader_stage_model = \"metal-render-pipeline\""));
    assert!(lowering_plan.contains("stage_binding_model = \"argument-buffer-specialized\""));
    assert!(lowering_plan.contains("dispatch_encoding_model = \"tile-and-threadgroup\""));

    assert!(backend_stub.contains("backend_profile = \"metal.apple-silicon-gpu\""));
    assert!(backend_stub.contains("execution_route = \"unified-render-graph\""));
    assert!(backend_stub.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(backend_stub.contains("wake_adapter = \"metal-shared-event\""));
    assert!(backend_stub.contains("shader_ir = \"msl2.4\""));
    assert!(backend_stub.contains("shader_entry_model = \"metal-function-constant-specialized\""));
    assert!(backend_stub.contains("queue_binding_model = \"unified-command-queue\""));
    assert!(backend_stub.contains("resource_binding_model = \"argument-buffer-table\""));

    assert!(host_bridge_stub.contains("bridge_profile = \"metal.apple-silicon-gpu\""));
    assert!(host_bridge_stub.contains("execution_route = \"unified-render-graph\""));
    assert!(host_bridge_stub.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(host_bridge_stub.contains("wake_adapter = \"metal-shared-event\""));
    let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);
    assert!(sidecar.contains("ir_container = \"text.msl\""));
    assert!(sidecar.contains("shader.profile.bind-set.v1"));
    assert!(sidecar.contains("registered_lane_groups = [\"setup\", \"resource\", \"render\"]"));
    assert!(sidecar.contains("[lowering_capabilities]"));
    assert!(sidecar.contains("capability_owner = \"shader-nustar\""));
    assert!(sidecar.contains("native_ir = \"msl2.4\""));
    assert!(sidecar.contains("resource_lowering = \"argument-buffer-table\""));
    assert!(sidecar.contains("texture_lowering = \"texture2d-sampler-argument\""));
    assert!(sidecar.contains("shader.stage-interface"));
    assert!(sidecar.contains("entry_symbol = \"main0\""));
    assert!(sidecar.contains("stage_kind = \"fragment\""));
    assert!(sidecar.contains("resource_layout = \"argument-buffer\""));
    assert!(sidecar.contains("[pipeline_layout]"));
    assert!(sidecar.contains("color_targets = [\"rgba8unorm\"]"));
    assert!(sidecar.contains("threadgroup_topology = \"tile\""));
    assert!(sidecar.contains("[resource_bindings]"));
    assert!(sidecar.contains("binding_table = \"material.uniforms, frame.texture0\""));
    assert!(sidecar.contains("[entry_points]"));
    assert!(sidecar.contains("vertex = \"vs_main\""));
    assert!(sidecar.contains("fragment = \"main0\""));
    assert!(sidecar.contains("compute = \"cs_main\""));
    assert!(sidecar.contains("#include <metal_stdlib>"));
    assert!(sidecar.contains("vertex float4 vs_main"));
    assert!(sidecar.contains("fragment float4 main0"));
    assert!(sidecar.contains("kernel void cs_main"));
}

#[test]
fn shader_vulkan_lowering_plan_switches_to_spirv_pipeline_profile() {
    let shader_unit = sample_domain_unit(
        "shader",
        "official.shader",
        "vulkan",
        "cross-vendor",
        "discrete-or-integrated-gpu",
        "vulkan.discrete-or-integrated-gpu",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
    let backend_stub = super::render_domain_build_unit_backend_stub(&shader_unit);

    assert!(lowering_plan.contains("lowering_profile = \"vulkan.discrete-or-integrated-gpu\""));
    assert!(lowering_plan.contains("execution_route = \"spirv-render-queue\""));
    assert!(lowering_plan.contains("submission_adapter = \"vulkan-command-buffer\""));
    assert!(lowering_plan.contains("wake_adapter = \"vulkan-timeline-semaphore\""));
    assert!(lowering_plan.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]"));
    assert!(lowering_plan.contains("shader.profile.sampler.v1"));
    assert!(
        lowering_plan.contains("registered_lane_groups = [\"setup\", \"resource\", \"render\"]")
    );
    assert!(lowering_plan.contains("lowering_ir = \"spirv1.6\""));
    assert!(lowering_plan.contains("shader_stage_model = \"spirv-graphics-pipeline\""));
    assert!(lowering_plan.contains("stage_binding_model = \"descriptor-set-layout\""));
    assert!(lowering_plan.contains("dispatch_encoding_model = \"renderpass-command-buffer\""));

    assert!(backend_stub.contains("backend_profile = \"vulkan.discrete-or-integrated-gpu\""));
    assert!(backend_stub.contains("shader_ir = \"spirv1.6\""));
    assert!(backend_stub.contains("shader_entry_model = \"vulkan-pipeline\""));
    assert!(backend_stub.contains("queue_binding_model = \"explicit-device-queue\""));
    assert!(backend_stub.contains("resource_binding_model = \"descriptor-set-layout\""));
    let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);
    assert!(sidecar.contains("ir_container = \"text.spirv\""));
    assert!(sidecar.contains("pipeline_lowering = \"vulkan-graphics-pipeline\""));
    assert!(sidecar.contains("resource_lowering = \"descriptor-set-layout\""));
    assert!(sidecar.contains("texture_lowering = \"sampled-image-descriptor\""));
    assert!(sidecar.contains("spirv.interface-layout"));
    assert!(sidecar.contains("entry_symbol = \"main\""));
    assert!(sidecar.contains("stage_kind = \"fragment\""));
    assert!(sidecar.contains("resource_layout = \"descriptor-set\""));
    assert!(sidecar.contains("[pipeline_layout]"));
    assert!(sidecar.contains("threadgroup_topology = \"quad-fragment\""));
    assert!(sidecar.contains("[resource_bindings]"));
    assert!(sidecar.contains("binding_table = \"set0.binding0.texture, set0.binding1.sampler\""));
    assert!(sidecar.contains("[entry_points]"));
    assert!(sidecar.contains("vertex = \"vs_main\""));
    assert!(sidecar.contains("fragment = \"main\""));
    assert!(sidecar.contains("compute = \"cs_main\""));
    assert!(sidecar.contains("OpCapability Shader"));
    assert!(sidecar.contains("OpEntryPoint Vertex %vs_main"));
    assert!(sidecar.contains("OpEntryPoint Fragment %main"));
    assert!(sidecar.contains("OpEntryPoint GLCompute %cs_main"));
}

#[test]
fn shader_unknown_profile_falls_back_to_fragment_only_stage_set() {
    let shader_unit = sample_domain_unit(
        "shader",
        "official.shader",
        "experimental",
        "generic",
        "fragment-only-lab",
        "experimental.fragment-only-lab",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
    let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);

    assert!(lowering_plan.contains("supported_stages = [\"fragment\"]"));
    assert!(sidecar.contains("supported_stages = [\"fragment\"]"));
    assert!(sidecar.contains("entry_symbol = \"unimplemented\""));
    assert!(sidecar.contains("fragment = \"unimplemented\""));
    assert!(!sidecar.contains("vertex = "));
    assert!(!sidecar.contains("compute = "));
}

#[test]
fn kernel_coreml_profile_reports_dispatch_kinds() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "coreml",
        "apple",
        "apple-ane",
        "coreml.apple-ane",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&kernel_unit);
    let backend_stub = super::render_domain_build_unit_backend_stub(&kernel_unit);

    assert!(lowering_plan.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
    assert!(lowering_plan.contains("kernel.profile.tensor-reduce.v1"));
    assert!(lowering_plan.contains("kernel.profile.result-buffer.v1"));
    assert!(lowering_plan.contains(
        "registered_lane_groups = [\"setup\", \"memory\", \"compute\", \"shape\", \"reduce\", \"select\", \"debug\"]"
    ));
    assert!(backend_stub.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
}

#[test]
fn kernel_coreml_sidecar_emits_dispatch_templates() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "coreml",
        "apple",
        "apple-ane",
        "coreml.apple-ane",
    );
    let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

    assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(sidecar.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
    assert!(sidecar.contains("kernel.profile.tensor-selection.v1"));
    assert!(sidecar.contains(
        "registered_lane_groups = [\"setup\", \"memory\", \"compute\", \"shape\", \"reduce\", \"select\", \"debug\"]"
    ));
    assert!(sidecar.contains("[lowering_capabilities]"));
    assert!(sidecar.contains("capability_owner = \"kernel-nustar\""));
    assert!(sidecar.contains("native_ir = \"coreml-program\""));
    assert!(sidecar.contains("tensor_lowering = \"ranked-tensor-graph\""));
    assert!(sidecar.contains("dispatch_lowering = \"ane-graph-submit\""));
    assert!(sidecar.contains("kernel.shape-contract"));
    assert!(sidecar.contains("[dispatch_shapes]"));
    assert!(sidecar.contains("primary = \"graph\""));
    assert!(sidecar.contains("[entry_points]"));
    assert!(sidecar.contains("graph = \"infer_main\""));
    assert!(sidecar.contains("batch = \"infer_batch\""));
    assert!(sidecar.contains("graph_body = \"program infer_main"));
}

#[test]
fn kernel_vulkan_sidecar_emits_grid_and_indirect_dispatch_templates() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "vulkan",
        "cross-vendor",
        "discrete-or-integrated-gpu",
        "vulkan.discrete-or-integrated-gpu",
    );
    let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

    assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(sidecar.contains("supported_dispatch_kinds = [\"grid\", \"indirect\", \"batch\"]"));
    assert!(sidecar.contains("native_ir = \"spirv1.6\""));
    assert!(sidecar.contains("tensor_lowering = \"storage-buffer-tensor-view\""));
    assert!(sidecar.contains("dispatch_lowering = \"compute-grid-or-indirect\""));
    assert!(sidecar.contains("spirv.compute-layout"));
    assert!(sidecar.contains("primary = \"grid\""));
    assert!(sidecar.contains("fallback = \"indirect\""));
    assert!(sidecar.contains("binding_table = \"set0.buffer0, set0.buffer1\""));
    assert!(sidecar.contains("grid = \"main\""));
    assert!(sidecar.contains("indirect = \"main_indirect\""));
    assert!(sidecar.contains("OpEntryPoint GLCompute %main"));
}

#[test]
fn kernel_cpu_fallback_sidecar_emits_range_and_tile_dispatch_templates() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "cpu-fallback",
        "generic",
        "cpu-host",
        "cpu-fallback.cpu-host",
    );
    let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

    assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(sidecar.contains("supported_dispatch_kinds = [\"range\", \"tile\", \"batch\"]"));
    assert!(sidecar.contains("native_ir = \"host-simd\""));
    assert!(sidecar.contains("tensor_lowering = \"slice-backed-tensor-view\""));
    assert!(sidecar.contains("dispatch_lowering = \"threadpool-range-or-tile\""));
    assert!(sidecar.contains("host.slice-bounds"));
    assert!(sidecar.contains("primary = \"range\""));
    assert!(sidecar.contains("fallback = \"tile\""));
    assert!(sidecar.contains("binding_table = \"slice.input, slice.output\""));
    assert!(sidecar.contains("range = \"run_range\""));
    assert!(sidecar.contains("tile = \"run_tile\""));
    assert!(sidecar.contains("range_body = \"fn run_range"));
}

#[test]
fn network_urlsession_sidecar_emits_foundation_session_templates() {
    let network_unit = sample_domain_unit(
        "network",
        "official.network",
        "urlsession",
        "apple",
        "socket-io",
        "urlsession.socket-io",
    );
    let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

    assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(sidecar.contains("transport_ir = \"foundation-url-request\""));
    assert!(sidecar.contains("transport_binding_model = \"session-task-packet\""));
    assert!(sidecar.contains("[lowering_capabilities]"));
    assert!(sidecar.contains("capability_owner = \"network-nustar\""));
    assert!(sidecar.contains("frontend_ir = \"nuis-yir.network\""));
    assert!(sidecar.contains("native_ir = \"foundation-url-request\""));
    assert!(sidecar.contains("transport_lowering = \"session-task-packet\""));
    assert!(sidecar.contains("dispatch_lowering = \"urlsession-task-submit\""));
    assert!(sidecar.contains("network.session-shape"));
    assert!(sidecar.contains("[session_shapes]"));
    assert!(sidecar.contains("request = \"http-client-session\""));
    assert!(sidecar.contains("response = \"completion-callback\""));
    assert!(sidecar.contains("streaming = \"delegate-push-stream\""));
    assert!(sidecar.contains("binding_table = \"session.handle, request.packet, response.slot\""));
    assert!(sidecar.contains("connect = \"open_session\""));
    assert!(sidecar.contains("send = \"submit_request\""));
    assert!(sidecar.contains("recv = \"on_response\""));
    assert!(sidecar.contains("finalize = \"finish_exchange\""));
}

#[test]
fn network_socket_abi_sidecar_emits_poll_reactor_templates() {
    let network_unit = sample_domain_unit(
        "network",
        "official.network",
        "socket-abi",
        "cross-vendor",
        "socket-io",
        "socket-abi.socket-io",
    );
    let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

    assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(sidecar.contains("transport_ir = \"posix-socket\""));
    assert!(sidecar.contains("transport_binding_model = \"packet-poll-reactor\""));
    assert!(sidecar.contains("capability_owner = \"network-nustar\""));
    assert!(sidecar.contains("native_ir = \"posix-socket\""));
    assert!(sidecar.contains("transport_lowering = \"packet-poll-reactor\""));
    assert!(sidecar.contains("dispatch_lowering = \"poll-send-recv-submit\""));
    assert!(sidecar.contains("network.packet-shape"));
    assert!(sidecar.contains("request = \"socket-reactor-session\""));
    assert!(sidecar.contains("response = \"poll-ready-response\""));
    assert!(sidecar.contains("streaming = \"fd-edge-stream\""));
    assert!(sidecar.contains("binding_table = \"fd.handle, packet.buffer, ready.token\""));
    assert!(sidecar.contains("connect = \"open_fd_session\""));
    assert!(sidecar.contains("recv = \"poll_ready_response\""));
    assert!(sidecar.contains("finalize = \"finish_poll_exchange\""));
}

#[test]
fn network_winsock_sidecar_emits_iocp_templates() {
    let network_unit = sample_domain_unit(
        "network",
        "official.network",
        "winsock",
        "microsoft",
        "socket-io",
        "winsock.socket-io",
    );
    let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

    assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(sidecar.contains("transport_ir = \"winsock-overlapped\""));
    assert!(sidecar.contains("transport_binding_model = \"overlapped-packet-reactor\""));
    assert!(sidecar.contains("capability_owner = \"network-nustar\""));
    assert!(sidecar.contains("native_ir = \"winsock-overlapped\""));
    assert!(sidecar.contains("transport_lowering = \"overlapped-packet-reactor\""));
    assert!(sidecar.contains("dispatch_lowering = \"winsock-overlapped-submit\""));
    assert!(sidecar.contains("network.overlapped-shape"));
    assert!(sidecar.contains("request = \"overlapped-client-session\""));
    assert!(sidecar.contains("response = \"iocp-completion\""));
    assert!(sidecar.contains("streaming = \"completion-port-stream\""));
    assert!(
        sidecar.contains("binding_table = \"socket.handle, overlapped.packet, completion.port\"")
    );
    assert!(sidecar.contains("connect = \"connect_overlapped\""));
    assert!(sidecar.contains("recv = \"await_iocp_completion\""));
    assert!(sidecar.contains("finalize = \"finish_iocp_exchange\""));
}
