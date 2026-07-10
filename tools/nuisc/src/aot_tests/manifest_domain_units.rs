use super::*;

#[test]
fn build_manifest_tracks_heterogeneous_domain_build_units() {
    let dir = temp_dir("build_manifest_heterogeneous_units");
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&bin, "bin").unwrap();

    let written = CompileArtifacts {
        ast_path: ast.display().to_string(),
        nir_path: nir.display().to_string(),
        yir_path: yir.display().to_string(),
        llvm_ir_path: ll.display().to_string(),
        binary_path: bin.display().to_string(),
        packaging_mode: "native-cpu-llvm".to_owned(),
    };
    let cpu_target = CpuBuildTarget {
        abi: "cpu.arm64.apple_aapcs64".to_owned(),
        machine_arch: "arm64".to_owned(),
        machine_os: "darwin".to_owned(),
        object_format: "macho".to_owned(),
        calling_abi: "apple_aapcs64".to_owned(),
        clang_target: "arm64-apple-darwin".to_owned(),
        isa_family: "aarch64".to_owned(),
        isa_features: vec!["a64".to_owned(), "neon".to_owned()],
        cross_compile: false,
    };
    let manifest = super::write_build_manifest(
        &dir,
        &written,
        &BuildManifestContext {
            input_path: "/tmp/hetero.ns".to_owned(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec![
                "official.cpu".to_owned(),
                "official.kernel".to_owned(),
                "official.network".to_owned(),
            ],
            compile_cache: None,
            project: Some(BuildManifestProjectInfo {
                name: "hetero".to_owned(),
                abi_mode: "explicit".to_owned(),
                abi_graph_summary: None,
                abi_entries: vec![
                    ("cpu".to_owned(), cpu_target.abi.clone()),
                    ("kernel".to_owned(), "kernel.apple_ane.coreml.v1".to_owned()),
                    (
                        "network".to_owned(),
                        "network.socket.macos.arm64.v1".to_owned(),
                    ),
                ],
                plan_summary: None,
                effective_input: None,
                text_handle_rewrite_helper_hits: 0,
                text_handle_rewrite_local_hits: 0,
                manifest_copy_path: None,
                plan_index_path: None,
                organization_index_path: None,
                exchange_index_path: None,
                modules_index_path: None,
                docs_index_path: None,
                docs_module_count: 0,
                docs_documented_module_count: 0,
                docs_documented_item_count: 0,
                imports_index_path: None,
                imports_library_count: 0,
                imports_visible_library_count: 0,
                imports_visible_module_count: 0,
                imports_documented_visible_module_count: 0,
                imports_documented_visible_item_count: 0,
                galaxy_index_path: None,
                galaxy_count: 0,
                galaxy_documented_count: 0,
                galaxy_documented_library_module_count: 0,
                galaxy_documented_item_count: 0,
                links_index_path: None,
                packet_index_path: None,
                host_ffi_index_path: None,
                abi_index_path: None,
            }),
            doc_index: None,
            cpu_target,
        },
    )
    .unwrap();

    let report = verify_build_manifest(PathBuf::from(&manifest).as_path()).unwrap();
    assert_eq!(report.envelope_package_count, 3);
    assert_eq!(report.execution_contracts_checked, 3);
    assert_eq!(report.domain_build_unit_count, 3);
    assert_eq!(report.heterogeneous_domain_count, 2);
    assert_eq!(report.domain_payload_blobs_checked, 2);
    assert_eq!(report.domain_payload_blob_sections_checked, 10);
    assert_eq!(report.domain_payload_contract_sections_checked, 2);
    assert_eq!(report.domain_payload_lowering_plans_checked, 2);
    assert_eq!(report.domain_payload_backend_stubs_checked, 2);
    assert_eq!(report.domain_payload_bridge_plans_checked, 2);
    assert_eq!(report.domain_bridge_stubs_checked, 2);
    assert_eq!(report.bridge_registry_units, 2);
    assert_eq!(report.bridge_registry_checked, 1);
    assert_eq!(report.bridge_registry_entries_checked, 2);
    assert_eq!(report.host_bridge_plan_units, 2);
    assert_eq!(report.host_bridge_plan_checked, 1);
    assert_eq!(report.host_bridge_plan_entries_checked, 2);
    assert_eq!(report.lowering_plan_units, 2);
    assert_eq!(report.lowering_plan_index_checked, 1);
    assert_eq!(report.lowering_plan_entries_checked, 2);
    assert_eq!(report.clock_protocol_domains, 3);
    assert_eq!(report.clock_protocol_checked, 1);
    assert_eq!(report.clock_protocol_entries_checked, 10);
    assert_eq!(report.hetero_calculate_plan_units, 2);
    assert_eq!(report.hetero_calculate_plan_checked, 1);
    assert_eq!(report.hetero_calculate_plan_entries_checked, 2);
    let kernel_payload = dir.join("nuis.domain.kernel.payload.toml");
    let kernel_bridge_stub = dir.join("nuis.domain.kernel.bridge.stub.txt");
    let kernel_payload_blob = dir.join("nuis.domain.kernel.payload.bin");
    let network_payload = dir.join("nuis.domain.network.payload.toml");
    let network_bridge_stub = dir.join("nuis.domain.network.bridge.stub.txt");
    let network_payload_blob = dir.join("nuis.domain.network.payload.bin");
    let bridge_registry = dir.join("nuis.bridge.registry.toml");
    let host_bridge_plan_index = dir.join("nuis.host-bridge.plan-index.toml");
    let lowering_plan_index = dir.join("nuis.lowering.plan-index.toml");
    let clock_protocol = dir.join("nuis.clock-protocol.toml");
    let hetero_calculate_plan = dir.join("nuis.hetero-calculate.plan.toml");
    assert!(kernel_payload.exists());
    assert!(kernel_bridge_stub.exists());
    assert!(kernel_payload_blob.exists());
    assert!(network_payload.exists());
    assert!(network_bridge_stub.exists());
    assert!(network_payload_blob.exists());
    assert!(bridge_registry.exists());
    assert!(host_bridge_plan_index.exists());
    assert!(lowering_plan_index.exists());
    assert!(clock_protocol.exists());
    assert!(hetero_calculate_plan.exists());
    let kernel_payload_text = fs::read_to_string(&kernel_payload).unwrap();
    let kernel_bridge_stub_text = fs::read_to_string(&kernel_bridge_stub).unwrap();
    let network_payload_text = fs::read_to_string(&network_payload).unwrap();
    let network_bridge_stub_text = fs::read_to_string(&network_bridge_stub).unwrap();
    let bridge_registry_text = fs::read_to_string(&bridge_registry).unwrap();
    let host_bridge_plan_index_text = fs::read_to_string(&host_bridge_plan_index).unwrap();
    let lowering_plan_index_text = fs::read_to_string(&lowering_plan_index).unwrap();
    let clock_protocol_text = fs::read_to_string(&clock_protocol).unwrap();
    let hetero_calculate_plan_text = fs::read_to_string(&hetero_calculate_plan).unwrap();
    let bridge_registry_path_text = bridge_registry.display().to_string();
    let host_bridge_plan_index_path_text = host_bridge_plan_index.display().to_string();
    let lowering_plan_index_path_text = lowering_plan_index.display().to_string();
    let clock_protocol_path_text = clock_protocol.display().to_string();
    let hetero_calculate_plan_path_text = hetero_calculate_plan.display().to_string();
    assert_eq!(
        report.bridge_registry_path.as_deref(),
        Some(bridge_registry_path_text.as_str())
    );
    assert_eq!(
        report.host_bridge_plan_index_path.as_deref(),
        Some(host_bridge_plan_index_path_text.as_str())
    );
    assert_eq!(
        report.lowering_plan_index_path.as_deref(),
        Some(lowering_plan_index_path_text.as_str())
    );
    assert_eq!(
        report.clock_protocol_path.as_deref(),
        Some(clock_protocol_path_text.as_str())
    );
    assert_eq!(
        report.hetero_calculate_plan_path.as_deref(),
        Some(hetero_calculate_plan_path_text.as_str())
    );
    assert!(bridge_registry_text.contains("schema = \"nuis-bridge-registry-v1\""));
    assert!(bridge_registry_text.contains("bridge_count = 2"));
    assert!(bridge_registry_text.contains("[[bridge]]"));
    assert!(bridge_registry_text.contains("domain_family = \"kernel\""));
    assert!(bridge_registry_text.contains("domain_family = \"network\""));
    assert!(bridge_registry_text.contains("backend_family = \"coreml\""));
    assert!(bridge_registry_text.contains("vendor = \"apple\""));
    assert!(bridge_registry_text.contains("device_class = \"apple-ane\""));
    assert!(bridge_registry_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(bridge_registry_text.contains("backend_family = \"urlsession\""));
    assert!(bridge_registry_text.contains("device_class = \"socket-io\""));
    assert!(bridge_registry_text.contains("selected_lowering_target = \"urlsession.socket-io\""));
    assert!(bridge_registry_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
    assert!(bridge_registry_text.contains("host_ffi_bridge = \"cffi.network.dispatch.v1\""));
    assert!(bridge_registry_text.contains("host_ffi_policy = \"signature-whitelist-required\""));
    assert!(bridge_registry_text
        .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
    assert!(bridge_registry_text
        .contains("host_ffi_symbol = \"nuis_network_urlsession_socket_io_dispatch_v1\""));
    assert!(bridge_registry_text.contains("host_ffi_signature_hash = \"0x"));
    assert!(bridge_registry_text.contains("bridge_stub_path = "));
    assert!(host_bridge_plan_index_text.contains("schema = \"nuis-host-bridge-plan-index-v1\""));
    assert!(host_bridge_plan_index_text.contains("plan_count = 2"));
    assert!(host_bridge_plan_index_text.contains("[[plan]]"));
    assert!(host_bridge_plan_index_text.contains("domain_family = \"kernel\""));
    assert!(host_bridge_plan_index_text.contains("domain_family = \"network\""));
    assert!(host_bridge_plan_index_text.contains("backend_family = \"coreml\""));
    assert!(host_bridge_plan_index_text.contains("vendor = \"apple\""));
    assert!(host_bridge_plan_index_text.contains("device_class = \"apple-ane\""));
    assert!(host_bridge_plan_index_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(host_bridge_plan_index_text.contains("backend_family = \"urlsession\""));
    assert!(host_bridge_plan_index_text.contains("device_class = \"socket-io\""));
    assert!(
        host_bridge_plan_index_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
    );
    assert!(host_bridge_plan_index_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
    assert!(host_bridge_plan_index_text.contains("host_ffi_bridge = \"cffi.network.dispatch.v1\""));
    assert!(
        host_bridge_plan_index_text.contains("host_ffi_policy = \"signature-whitelist-required\"")
    );
    assert!(host_bridge_plan_index_text
        .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
    assert!(host_bridge_plan_index_text
        .contains("host_ffi_symbol = \"nuis_network_urlsession_socket_io_dispatch_v1\""));
    assert!(host_bridge_plan_index_text.contains("host_ffi_signature_hash = \"0x"));
    assert!(host_bridge_plan_index_text
        .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
    assert!(lowering_plan_index_text.contains("schema = \"nuis-domain-lowering-plan-index-v1\""));
    assert!(lowering_plan_index_text.contains("plan_count = 2"));
    assert!(lowering_plan_index_text.contains("[[lowering_plan]]"));
    assert!(lowering_plan_index_text.contains("domain_family = \"kernel\""));
    assert!(lowering_plan_index_text.contains("domain_family = \"network\""));
    assert!(lowering_plan_index_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(
        lowering_plan_index_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
    );
    assert!(lowering_plan_index_text.contains("execution_route = \"ane-graph-execution\""));
    assert!(lowering_plan_index_text.contains("execution_route = \"foundation-session-reactor\""));
    assert!(lowering_plan_index_text.contains("kernel.profile.tensor-reduce.v1"));
    assert!(lowering_plan_index_text.contains("kernel.profile.result-buffer.v1"));
    assert!(lowering_plan_index_text.contains(
        "registered_lane_groups = [\"setup\", \"memory\", \"compute\", \"shape\", \"reduce\", \"select\", \"debug\"]"
    ));
    assert!(lowering_plan_index_text
        .contains("symbol_namespace = \"nuis::domain::kernel::coreml_apple_ane\""));
    assert!(
        lowering_plan_index_text.contains("debug_anchor = \"nuis.debug.kernel.coreml_apple_ane\"")
    );
    assert!(
        lowering_plan_index_text.contains("linkage_anchor = \"nuis.link.kernel.coreml_apple_ane\"")
    );
    assert!(lowering_plan_index_text.contains(
        "source_map_scope = \"domain:kernel/package:official.kernel/target:coreml.apple-ane\""
    ));
    assert!(lowering_plan_index_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
    assert!(lowering_plan_index_text.contains("host_ffi_policy = \"signature-whitelist-required\""));
    assert!(lowering_plan_index_text
        .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
    assert!(lowering_plan_index_text.contains(
        "host_ffi_signature = \"fn(payload: ptr, payload_len: usize, bridge_state: ptr) -> i64\""
    ));
    assert!(lowering_plan_index_text.contains("host_ffi_signature_hash = \"0x"));
    assert!(lowering_plan_index_text
        .contains("symbol_namespace = \"nuis::domain::network::urlsession_socket_io\""));
    assert!(lowering_plan_index_text
        .contains("debug_anchor = \"nuis.debug.network.urlsession_socket_io\""));
    assert!(lowering_plan_index_text.contains("ir_sidecar_path = "));
    assert!(lowering_plan_index_text.contains("payload_blob_path = "));
    assert!(lowering_plan_index_text.contains("bridge_stub_path = "));
    let manifest_text = fs::read_to_string(&manifest).unwrap();
    assert!(manifest_text.contains("[hetero_calculate_plan]"));
    assert!(manifest_text.contains("hetero_calculate_plan_path = "));
    assert!(manifest_text
        .contains("hetero_calculate_plan_schema = \"nuis-hetero-calculate-link-plan-v1\""));
    assert!(manifest_text.contains("hetero_calculate_plan_units = 2"));
    assert!(manifest_text.contains("hetero_calculate_plan_inline = "));
    assert!(clock_protocol_text.contains("schema = \"nuis-clock-protocol-v1\""));
    assert!(clock_protocol_text.contains("mode = \"heterogeneous-lifecycle-clock\""));
    assert!(clock_protocol_text.contains("relation = \"data-segment-commit\""));
    assert!(clock_protocol_text.contains("from = \"t0001.kernel.complete\""));
    assert!(clock_protocol_text.contains("to = \"t0001.kernel.data_commit\""));
    assert!(clock_protocol_text.contains("from = \"t0002.network.complete\""));
    assert!(clock_protocol_text.contains("to = \"t0002.network.data_commit\""));
    assert!(hetero_calculate_plan_text.contains("schema = \"nuis-hetero-calculate-link-plan-v1\""));
    assert!(hetero_calculate_plan_text.contains("mode = \"heterogeneous-static-lifecycle\""));
    assert!(hetero_calculate_plan_text.contains("static_link = true"));
    assert!(hetero_calculate_plan_text.contains("lifecycle_driven = true"));
    assert!(hetero_calculate_plan_text.contains("[validation]"));
    assert!(hetero_calculate_plan_text.contains("valid = true"));
    assert!(hetero_calculate_plan_text.contains("[[node]]"));
    assert!(hetero_calculate_plan_text.contains("timestamp = \"t0001.kernel\""));
    assert!(hetero_calculate_plan_text.contains("timestamp = \"t0002.network\""));
    assert!(hetero_calculate_plan_text.contains("wait_on = [\"t0001.kernel\"]"));
    assert!(hetero_calculate_plan_text.contains("[[data_segment]]"));
    assert!(hetero_calculate_plan_text.contains("order_key = \"data:0001:kernel\""));
    assert!(hetero_calculate_plan_text.contains("order_key = \"data:0002:network\""));
    let kernel_blob =
        super::decode_domain_build_unit_payload_blob(&fs::read(&kernel_payload_blob).unwrap())
            .unwrap();
    let network_blob =
        super::decode_domain_build_unit_payload_blob(&fs::read(&network_payload_blob).unwrap())
            .unwrap();
    let kernel_lowering_plan = super::render_domain_build_unit_lowering_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let kernel_backend_stub = super::render_domain_build_unit_backend_stub(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let kernel_ir_sidecar = super::render_domain_build_unit_kernel_ir_sidecar(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let kernel_bridge_plan = super::render_domain_build_unit_bridge_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let network_lowering_plan = super::render_domain_build_unit_lowering_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    let network_backend_stub = super::render_domain_build_unit_backend_stub(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    let network_ir_sidecar = super::render_domain_build_unit_network_ir_sidecar(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    let network_bridge_plan = super::render_domain_build_unit_bridge_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    assert!(kernel_payload_text.contains("schema = \"nuis-domain-build-payload-v1\""));
    assert!(kernel_payload_text.contains("support_surface = ["));
    assert!(kernel_payload_text.contains("default_lanes = ["));
    assert!(kernel_payload_text.contains("resource_families = ["));
    assert!(kernel_payload_text.contains("lowering_targets = ["));
    assert!(kernel_payload_text.contains("ops = ["));
    assert!(network_payload_text.contains("schema = \"nuis-domain-build-payload-v1\""));
    assert!(network_payload_text.contains("host_ffi_surface = ["));
    assert!(network_payload_text.contains("clock_bridge_default = "));
    assert_eq!(kernel_blob.domain_family, "kernel");
    assert_eq!(kernel_blob.package_id, "official.kernel");
    assert_eq!(kernel_blob.backend_family.as_deref(), Some("coreml"));
    assert_eq!(
        kernel_blob.selected_lowering_target.as_deref(),
        Some("coreml.apple-ane")
    );
    assert_eq!(kernel_blob.contract_family, "nustar.kernel");
    assert_eq!(kernel_blob.packaging_role, "hetero-contract");
    assert_eq!(kernel_blob.payload_kind, "contract-sidecar");
    assert_eq!(kernel_blob.payload_format, "toml");
    assert_eq!(kernel_blob.sections.len(), 5);
    assert_eq!(kernel_blob.sections[0].name, "contract_toml");
    assert_eq!(
        kernel_blob.sections[0].bytes,
        kernel_payload_text.as_bytes()
    );
    assert_eq!(kernel_blob.sections[1].name, "lowering_plan");
    assert_eq!(
        kernel_blob.sections[1].bytes,
        kernel_lowering_plan.as_bytes()
    );
    assert_eq!(kernel_blob.sections[2].name, "backend_stub");
    assert_eq!(
        kernel_blob.sections[2].bytes,
        kernel_backend_stub.as_bytes()
    );
    assert_eq!(kernel_blob.sections[3].name, "bridge_plan");
    assert_eq!(kernel_blob.sections[3].bytes, kernel_bridge_plan.as_bytes());
    assert_eq!(kernel_blob.sections[4].name, "kernel_ir_sidecar");
    assert_eq!(kernel_blob.sections[4].bytes, kernel_ir_sidecar.as_bytes());
    let kernel_backend_text = std::str::from_utf8(&kernel_blob.sections[2].bytes).unwrap();
    let kernel_bridge_text = std::str::from_utf8(&kernel_blob.sections[3].bytes).unwrap();
    let kernel_sidecar_text = std::str::from_utf8(&kernel_blob.sections[4].bytes).unwrap();
    assert!(kernel_bridge_stub_text.contains("schema = \"nuis-host-bridge-spec-v1\""));
    assert!(kernel_bridge_stub_text.contains("vendor = \"apple\""));
    assert!(kernel_bridge_stub_text.contains("device_class = \"apple-ane\""));
    assert!(kernel_bridge_stub_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(kernel_bridge_stub_text
        .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
    assert!(kernel_bridge_stub_text.contains("host_ffi_surface = \"buffer,queue,fence\""));
    assert!(kernel_bridge_stub_text.contains("handle_family = \"kernel.buffer,kernel.dispatch\""));
    assert!(kernel_bridge_stub_text.contains(
        "phase_submit_inputs = [\"dispatch.handle\", \"bound.buffer.table\", \"queue.slot\"]"
    ));
    assert!(kernel_bridge_stub_text.contains("phase_wait_wake = \"completion-fence\""));
    assert!(kernel_bridge_stub_text.contains("bridge_plan_begin = true"));
    assert!(kernel_bridge_stub_text.contains("bridge_plan_end = true"));
    assert!(kernel_bridge_stub_text.contains("phase_submit = \"queue-dispatch-submit\""));
    assert!(kernel_backend_text.contains("stub_kind = \"kernel-dispatch\""));
    assert!(kernel_backend_text.contains("dispatch_shape = \"grid-launch\""));
    assert!(kernel_backend_text.contains("memory_binding = \"buffer-table\""));
    assert!(kernel_backend_text.contains("completion_model = \"device-fence\""));
    assert!(kernel_backend_text.contains("scheduler_binding = \"hetero-submit-bridge\""));
    assert!(kernel_backend_text.contains("backend_profile = \"coreml.apple-ane\""));
    assert!(kernel_backend_text.contains("execution_route = \"ane-graph-execution\""));
    assert!(kernel_backend_text.contains("submission_adapter = \"coreml-graph-submit\""));
    assert!(kernel_backend_text.contains("wake_adapter = \"coreml-completion-callback\""));
    assert!(
        kernel_backend_text.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]")
    );
    assert!(kernel_backend_text.contains("kernel_ir = \"coreml-program\""));
    assert!(kernel_backend_text.contains("kernel_entry_model = \"mlmodelc-function\""));
    assert!(kernel_backend_text.contains("queue_binding_model = \"ane-submission-service\""));
    assert!(kernel_backend_text.contains("resource_binding_model = \"tensor-argument-table\""));
    assert!(kernel_sidecar_text.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(
        kernel_sidecar_text.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]")
    );
    assert!(kernel_sidecar_text.contains("graph = \"infer_main\""));
    assert!(kernel_backend_text.contains("bind_phase = \"buffer-and-argument-bind\""));
    assert!(kernel_backend_text.contains("launch_phase = \"queue-dispatch-submit\""));
    assert!(kernel_backend_text.contains("wait_phase = \"fence-await-or-poll\""));
    assert!(kernel_backend_text.contains("finalize_phase = \"result-commit-and-release\""));
    assert!(kernel_bridge_text.contains("bridge_kind = \"managed-lifecycle-bridge\""));
    assert!(kernel_bridge_text.contains("phase_bind = \"buffer-and-argument-bind\""));
    assert!(kernel_bridge_text.contains("phase_submit = \"queue-dispatch-submit\""));
    assert!(kernel_bridge_text.contains("phase_wait = \"fence-await-or-poll\""));
    assert!(kernel_bridge_text.contains("phase_finalize = \"result-commit-and-release\""));
    assert_eq!(network_blob.domain_family, "network");
    assert_eq!(network_blob.package_id, "official.network");
    assert_eq!(network_blob.backend_family.as_deref(), Some("urlsession"));
    assert_eq!(
        network_blob.selected_lowering_target.as_deref(),
        Some("urlsession.socket-io")
    );
    assert_eq!(network_blob.contract_family, "nustar.network");
    assert_eq!(network_blob.packaging_role, "hetero-contract");
    assert_eq!(network_blob.payload_kind, "contract-sidecar");
    assert_eq!(network_blob.payload_format, "toml");
    assert_eq!(network_blob.sections.len(), 5);
    assert_eq!(network_blob.sections[0].name, "contract_toml");
    assert_eq!(
        network_blob.sections[0].bytes,
        network_payload_text.as_bytes()
    );
    assert_eq!(network_blob.sections[1].name, "lowering_plan");
    assert_eq!(
        network_blob.sections[1].bytes,
        network_lowering_plan.as_bytes()
    );
    assert_eq!(network_blob.sections[2].name, "backend_stub");
    assert_eq!(
        network_blob.sections[2].bytes,
        network_backend_stub.as_bytes()
    );
    assert_eq!(network_blob.sections[3].name, "bridge_plan");
    assert_eq!(
        network_blob.sections[3].bytes,
        network_bridge_plan.as_bytes()
    );
    assert_eq!(network_blob.sections[4].name, "network_ir_sidecar");
    assert_eq!(
        network_blob.sections[4].bytes,
        network_ir_sidecar.as_bytes()
    );
    let network_backend_text = std::str::from_utf8(&network_blob.sections[2].bytes).unwrap();
    let network_bridge_text = std::str::from_utf8(&network_blob.sections[3].bytes).unwrap();
    let network_sidecar_text = std::str::from_utf8(&network_blob.sections[4].bytes).unwrap();
    assert!(network_bridge_stub_text.contains("schema = \"nuis-host-bridge-spec-v1\""));
    assert!(network_bridge_stub_text.contains("vendor = \"apple\""));
    assert!(network_bridge_stub_text.contains("device_class = \"socket-io\""));
    assert!(
        network_bridge_stub_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
    );
    assert!(network_bridge_stub_text
        .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
    assert!(network_bridge_stub_text.contains("host_ffi_surface = \"socket,urlsession\""));
    assert!(
        network_bridge_stub_text.contains("handle_family = \"network.request,network.response\"")
    );
    assert!(network_bridge_stub_text.contains(
        "phase_submit_inputs = [\"session.handle\", \"request.handle\", \"request.packet\"]"
    ));
    assert!(network_bridge_stub_text.contains("phase_wait_wake = \"io-ready\""));
    assert!(network_bridge_stub_text.contains("bridge_plan_begin = true"));
    assert!(network_bridge_stub_text.contains("bridge_plan_end = true"));
    assert!(network_bridge_stub_text.contains("phase_submit = \"packet-write-dispatch\""));
    assert!(network_backend_text.contains("stub_kind = \"network-host-bridge\""));
    assert!(network_backend_text.contains("transport_model = \"client-session\""));
    assert!(network_backend_text.contains("request_shape = \"packetized-exchange\""));
    assert!(network_backend_text.contains("response_shape = \"completion-callback\""));
    assert!(network_backend_text.contains("scheduler_binding = \"network-poll-bridge\""));
    assert!(network_backend_text.contains("backend_profile = \"urlsession.socket-io\""));
    assert!(network_backend_text.contains("execution_route = \"foundation-session-reactor\""));
    assert!(network_backend_text.contains("submission_adapter = \"urlsession-task-submit\""));
    assert!(network_backend_text.contains("wake_adapter = \"urlsession-callback\""));
    assert!(network_backend_text.contains("transport_ir = \"foundation-url-request\""));
    assert!(network_backend_text.contains("transport_entry_model = \"urlsession-task\""));
    assert!(network_backend_text.contains("socket_binding_model = \"session-owned-socket\""));
    assert!(network_backend_text.contains("completion_binding_model = \"delegate-callback\""));
    assert!(network_backend_text.contains("connect_phase = \"socket-bind-or-session-open\""));
    assert!(network_backend_text.contains("send_phase = \"packet-write-dispatch\""));
    assert!(network_backend_text.contains("recv_phase = \"callback-or-read-ready\""));
    assert!(network_backend_text.contains("finalize_phase = \"response-commit-and-wake\""));
    assert!(network_bridge_text.contains("bridge_kind = \"managed-lifecycle-bridge\""));
    assert!(network_bridge_text.contains("phase_bind = \"socket-bind-or-session-open\""));
    assert!(network_bridge_text.contains("phase_submit = \"packet-write-dispatch\""));
    assert!(network_bridge_text.contains("phase_wait = \"callback-or-read-ready\""));
    assert!(network_bridge_text.contains("phase_finalize = \"response-commit-and-wake\""));
    assert!(network_sidecar_text.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(network_sidecar_text.contains("request = \"http-client-session\""));
    assert!(network_sidecar_text.contains("response = \"completion-callback\""));
    assert!(network_sidecar_text.contains("streaming = \"delegate-push-stream\""));
    assert!(network_sidecar_text.contains("connect = \"open_session\""));
    assert!(network_sidecar_text.contains("finalize = \"finish_exchange\""));
    assert!(report
        .domain_build_units
        .iter()
        .any(|unit| unit.domain_family == "cpu"
            && unit.packaging_role == "host-binary"
            && unit.artifact_stub_path.is_none()
            && unit.selected_lowering_target.as_deref() == Some("llvm")));
    assert!(report
        .domain_build_units
        .iter()
        .any(|unit| unit.domain_family == "kernel"
            && unit
                .artifact_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.artifact.toml"))
            && unit
                .artifact_payload_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.payload.toml"))
            && unit
                .artifact_bridge_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.bridge.stub.txt"))
            && unit
                .artifact_ir_sidecar_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.lowering.ir.txt"))
            && unit
                .artifact_payload_blob_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.payload.bin"))
            && unit
                .artifact_payload_blob_bytes
                .is_some_and(|bytes| bytes > 0)
            && unit.artifact_payload_format.as_deref() == Some("ndpb-v2")
            && unit.backend_family.as_deref() == Some("coreml")
            && unit.selected_lowering_target.as_deref() == Some("coreml.apple-ane")));
    assert!(report
        .domain_build_units
        .iter()
        .any(|unit| unit.domain_family == "network"
            && unit
                .artifact_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.artifact.toml"))
            && unit
                .artifact_payload_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.payload.toml"))
            && unit
                .artifact_bridge_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.bridge.stub.txt"))
            && unit
                .artifact_ir_sidecar_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.lowering.ir.txt"))
            && unit
                .artifact_payload_blob_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.payload.bin"))
            && unit
                .artifact_payload_blob_bytes
                .is_some_and(|bytes| bytes > 0)
            && unit.artifact_payload_format.as_deref() == Some("ndpb-v2")
            && unit.backend_family.as_deref() == Some("urlsession")
            && unit.selected_lowering_target.as_deref() == Some("urlsession.socket-io")));
}
