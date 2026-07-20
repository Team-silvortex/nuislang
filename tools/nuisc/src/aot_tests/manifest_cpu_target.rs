use super::*;

#[test]
fn build_manifest_round_trips_cpu_target_metadata() {
    let dir = temp_dir("build_manifest_cpu_target");
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
        abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
        isa_family: "x86_64".to_owned(),
        isa_features: vec!["x86-64".to_owned(), "sse2".to_owned()],
        cross_compile: true,
    };
    let manifest = super::write_build_manifest(
        &dir,
        &written,
        &BuildManifestContext {
            input_path: dir.join("demo.ns").to_string_lossy().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: Some(BuildManifestCacheInfo {
                status: "miss".to_owned(),
                key: "abc".to_owned(),
                root: dir.join("cache").to_string_lossy().to_string(),
            }),
                project: Some(BuildManifestProjectInfo {
                    name: "demo".to_owned(),
                    abi_mode: "explicit".to_owned(),
                    abi_graph_summary: Some(
                        "graph\tmode=explicit\tdomains=cpu\tcpu_summary=present\tdata_summary=absent\tkernel_target=absent\tshader_target=absent\tnetwork_target=absent"
                            .to_owned(),
                    ),
                    abi_entries: vec![("cpu".to_owned(), cpu_target.abi.clone())],
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
                cpu_target: cpu_target.clone(),
            },
        )
        .unwrap();
    let manifest_text = std::fs::read_to_string(&manifest).unwrap();
    assert!(manifest_text.contains("[nuis_envelope]"));
    assert!(manifest_text.contains("path = "));
    assert!(manifest_text.contains("schema = \"nuis-executable-envelope-v1\""));
    assert!(manifest_text.contains("[nuis_artifact]"));
    assert!(manifest_text.contains("artifact_schema = \"nuis-compiled-artifact-v1\""));
    assert!(manifest_text.contains("domain_families = [\"cpu\"]"));
    assert!(manifest_text.contains("abi_graph = "));
    assert!(manifest_text.contains("graph\tmode=explicit"));
    assert!(manifest_text.contains("[[execution_contract]]"));
    assert!(manifest_text.contains("[[domain_build_unit]]"));
    assert!(manifest_text.contains("package_id = \"official.cpu\""));
    assert!(manifest_text.contains("contract_family = \"nustar.cpu\""));
    assert!(manifest_text.contains("packaging_role = \"host-binary\""));
    let envelope = parse_nuis_executable_envelope(PathBuf::from(&manifest).as_path()).unwrap();
    assert_eq!(envelope.schema, "nuis-executable-envelope-v1");
    assert_eq!(envelope.executable_kind, "native-cpu-llvm");
    assert_eq!(envelope.package_count, 1);
    assert_eq!(envelope.domain_families, vec!["cpu".to_owned()]);
    assert_eq!(envelope.contract_families, vec!["nustar.cpu".to_owned()]);
    let rendered_envelope = render_nuis_executable_envelope(&envelope);
    assert!(rendered_envelope.contains("envelope_schema = \"nuis-executable-envelope-v1\""));
    assert!(rendered_envelope.contains("executable_kind = \"native-cpu-llvm\""));
    let encoded_envelope = encode_nuis_executable_envelope_binary(&envelope).unwrap();
    let decoded_envelope = decode_nuis_executable_envelope_binary(&encoded_envelope).unwrap();
    assert_eq!(decoded_envelope, envelope);
    let compiled_artifact =
        parse_nuis_compiled_artifact(PathBuf::from(&dir).join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(compiled_artifact.schema, "nuis-compiled-artifact-v1");
    assert_eq!(compiled_artifact.packaging_mode, "native-cpu-llvm");
    assert_eq!(compiled_artifact.binary_name, "demo.bin");
    assert_eq!(compiled_artifact.binary_blob, b"bin".to_vec());
    assert_eq!(compiled_artifact.build_manifest_source, manifest_text);
    assert_eq!(compiled_artifact.build_manifest_bytes, manifest_text.len());
    assert_eq!(compiled_artifact.envelope, envelope);
    assert_eq!(
        compiled_artifact.lifecycle.schema,
        "nuis-lifecycle-contract-v1"
    );
    assert_eq!(
        compiled_artifact.lifecycle.bootstrap_entry,
        "nuis.bootstrap.lifecycle.v1"
    );
    assert_eq!(compiled_artifact.lifecycle.export_surface.len(), 4);
    assert_eq!(
        compiled_artifact.lifecycle.runtime_capability_flags.len(),
        4
    );
    assert!(compiled_artifact
        .lifecycle
        .export_surface
        .contains(&"nuis_lifecycle_tick_export_v1".to_owned()));
    assert!(compiled_artifact
        .lifecycle
        .runtime_capability_flags
        .contains(&"runtime.tick".to_owned()));
    assert!(manifest_text.contains("[nuis_lifecycle]"));
    assert!(manifest_text.contains("lifecycle_schema = \"nuis-lifecycle-contract-v1\""));
    assert!(manifest_text.contains("lifecycle_export_surface = ["));
    let unpacked_dir = dir.join("unpacked");
    fs::create_dir_all(&unpacked_dir).unwrap();
    let unpacked_envelope = unpacked_dir.join("nuis.executable.envelope.toml");
    let unpacked_artifact = unpacked_dir.join("nuis.compiled.artifact");
    let unpacked_binary = unpacked_dir.join("demo.bin");
    fs::write(&unpacked_binary, &compiled_artifact.binary_blob).unwrap();
    super::write_nuis_executable_envelope(&unpacked_envelope, &compiled_artifact.envelope).unwrap();
    let relocated_manifest = super::render_relocated_unpacked_build_manifest(
        &compiled_artifact,
        &unpacked_dir,
        &unpacked_envelope,
        &unpacked_artifact,
        &unpacked_binary,
    )
    .unwrap();
    assert!(relocated_manifest.contains(&format!("output_dir = \"{}\"", unpacked_dir.display())));
    assert!(relocated_manifest.contains(&format!(
        "artifact_path = \"{}\"",
        unpacked_artifact.display()
    )));
    assert!(!relocated_manifest.contains("plan_index = "));
    let encoded_artifact = encode_nuis_compiled_artifact_binary(&compiled_artifact).unwrap();
    let decoded_artifact = decode_nuis_compiled_artifact_binary(&encoded_artifact).unwrap();
    assert_eq!(decoded_artifact, compiled_artifact);
    let artifact_verify_report =
        verify_nuis_compiled_artifact(PathBuf::from(&dir).join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert!(artifact_verify_report.lifecycle_contract_consistent);
    assert!(artifact_verify_report.lifecycle_runtime_capability_flags_consistent);
    let report = verify_build_manifest(PathBuf::from(&manifest).as_path()).unwrap();
    assert!(std::path::Path::new(&report.envelope_path).exists());
    assert!(std::path::Path::new(&report.artifact_path).exists());
    assert_eq!(report.envelope_schema, "nuis-executable-envelope-v1");
    assert_eq!(report.envelope_package_count, 1);
    assert_eq!(report.artifact_schema, "nuis-compiled-artifact-v1");
    assert_eq!(report.artifact_binary_name, "demo.bin");
    assert_eq!(report.artifact_binary_bytes, 3);
    assert_eq!(report.lifecycle_schema, "nuis-lifecycle-contract-v1");
    assert_eq!(
        report.lifecycle_bootstrap_entry,
        "nuis.bootstrap.lifecycle.v1"
    );
    assert!(report.lifecycle_hook_count >= 7);
    assert!(report
        .lifecycle_hook_surface
        .contains(&"on_scheduler_tick".to_owned()));
    assert_eq!(report.lifecycle_export_count, 4);
    assert!(report
        .lifecycle_export_surface
        .contains(&"nuis_lifecycle_shutdown_export_v1".to_owned()));
    assert!(report
        .lifecycle_runtime_capability_flags
        .contains(&"runtime.shutdown".to_owned()));
    assert_eq!(report.execution_contracts_checked, 1);
    assert_eq!(report.domain_build_unit_count, 1);
    assert_eq!(report.heterogeneous_domain_count, 0);
    assert_eq!(report.domain_payload_blobs_checked, 0);
    assert_eq!(report.bridge_registry_path, None);
    assert_eq!(report.bridge_registry_units, 0);
    assert_eq!(report.bridge_registry_checked, 0);
    assert_eq!(report.host_bridge_plan_index_path, None);
    assert_eq!(report.host_bridge_plan_units, 0);
    assert_eq!(report.host_bridge_plan_checked, 0);
    assert_eq!(report.lowering_plan_index_path, None);
    assert_eq!(report.lowering_plan_units, 0);
    assert_eq!(report.lowering_plan_index_checked, 0);
    assert_eq!(report.doc_index_path, None);
    assert_eq!(report.doc_index_module_count, 0);
    assert_eq!(report.doc_index_documented_item_count, 0);
    assert_eq!(report.doc_index_checked, 0);
    assert_eq!(report.domain_build_units.len(), 1);
    assert_eq!(report.domain_build_units[0].domain_family, "cpu");
    assert_eq!(report.domain_build_units[0].artifact_stub_path, None);
    assert_eq!(report.domain_build_units[0].artifact_payload_path, None);
    assert_eq!(report.domain_build_units[0].artifact_bridge_stub_path, None);
    assert_eq!(
        report.domain_build_units[0].artifact_payload_blob_path,
        None
    );
    assert_eq!(
        report.domain_build_units[0].artifact_payload_blob_bytes,
        None
    );
    assert_eq!(report.domain_build_units[0].artifact_payload_format, None);
    assert_eq!(
        report.domain_build_units[0]
            .selected_lowering_target
            .as_deref(),
        Some("llvm")
    );
    assert_eq!(report.cpu_target_abi, cpu_target.abi);
    assert_eq!(report.cpu_target_machine_arch, cpu_target.machine_arch);
    assert_eq!(report.cpu_target_machine_os, cpu_target.machine_os);
    assert_eq!(report.cpu_target_object_format, cpu_target.object_format);
    assert_eq!(report.cpu_target_calling_abi, cpu_target.calling_abi);
    assert_eq!(report.cpu_target_clang, cpu_target.clang_target);
    assert!(report.cpu_target_cross);
    assert_eq!(report.project_metadata_checked, 0);
}
