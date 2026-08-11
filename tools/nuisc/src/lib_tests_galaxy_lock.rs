use super::*;

#[test]
fn compile_command_writes_and_verifies_galaxy_resolution_lock() {
    let source =
        fs::read_to_string("../../examples/projects/tooling/hetero_proxy_benchmark_demo/main.ns")
            .unwrap();
    let project_root = write_temp_project_fixture(
        "hetero_proxy_benchmark_demo",
        r#"
name = "hetero_proxy_benchmark_demo"
entry = "main.ns"
modules = ["main.ns"]
galaxy = ["std=workspace"]
"#
        .trim_start(),
        &source,
    );
    let output_dir = temp_dir("compile_command_galaxy_resolution_lock_outputs");
    let output_stem = "hetero_proxy_benchmark_demo".to_owned();

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    for path in [
        output_dir.join(format!("{output_stem}.ll")),
        output_dir.join(&output_stem),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.compiled.artifact"),
        output_dir.join("nuis.project.host_ffi.txt"),
        output_dir.join("nuis.project.galaxy.lock"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_text.contains("name = \"hetero_proxy_benchmark_demo\""));
    assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
    assert!(manifest_text.contains("host_ffi_index = "));
    assert!(manifest_text.contains("galaxy_resolution_lock = "));
    assert!(manifest_text.contains("galaxy_resolution_sha256 = \"sha256:"));

    let galaxy_lock_path = output_dir.join("nuis.project.galaxy.lock");
    let galaxy_lock_source = fs::read_to_string(&galaxy_lock_path).unwrap();
    let galaxy_lock_summary = crate::project::verify_project_galaxy_resolution_lock_source(
        &galaxy_lock_source,
        &galaxy_lock_path,
    )
    .unwrap();
    assert_eq!(galaxy_lock_summary.dependencies, 2);
    assert_eq!(
        galaxy_lock_summary.library_modules,
        galaxy_lock_summary.selected_library_modules
    );

    let host_ffi_text = fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
    assert!(host_ffi_text.contains("host_monotonic_time_ns"));
    assert!(host_ffi_text.contains("host_sleep_ns"));
    assert!(host_ffi_text.contains("signature_pattern=i64()"));
    assert!(host_ffi_text.contains("signature_pattern=i64(i64)"));
    assert!(host_ffi_text.contains("signature_hash=fnv1a64:"));
    assert!(host_ffi_text.contains("policy=signature-whitelist-required"));

    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(manifest_report.artifact_binary_name, output_stem);
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert!(manifest_report.project_metadata_checked >= 7);
    let verify_manifest_json = verify_build_manifest_json(&manifest_path, &manifest_report);
    assert!(verify_manifest_json.contains("\"project_host_ffi_index\":"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_symbol_count\":2"));
    assert!(verify_manifest_json.contains("\"project_host_ffi_policy_count\":2"));
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let link_plan = linker::build_link_plan(&manifest_report, &artifact);
    let link_plan_json = linker::render_link_plan_json(&link_plan);
    assert!(link_plan_json.contains("\"host_ffi_symbol_count\":2"));
    assert!(link_plan_json.contains("\"host_ffi_policy_count\":2"));
    assert!(link_plan_json.contains("\"host_ffi_policy\":\"signature-whitelist-required\""));
    assert!(link_plan_json.contains("\"host_ffi_validation_checked\":2"));
    assert!(link_plan_json.contains("\"host_ffi_validation_valid\":true"));
    assert!(link_plan_json.contains("\"host_ffi_link_allowed\":true"));
    assert!(link_plan_json.contains("\"host_ffi_validation_issues\":[]"));
    assert!(link_plan_json.contains("\"host_ffi_validation_notes\":[]"));
    assert!(link_plan_json.contains("\"host_ffi_abi_groups\":[{"));
    assert!(link_plan_json.contains("\"symbols\":[\"host_monotonic_time_ns:i64()\""));
    assert!(link_plan_json.contains("\"symbol\":\"host_sleep_ns\""));

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);

    let status = Command::new(output_dir.join(&output_stem))
        .status()
        .expect("expected compiled hetero proxy benchmark binary to launch");
    assert!(status.success());

    let damaged = galaxy_lock_source.replacen("package_id = ", "package_id = \"drift\" # ", 1);
    fs::write(&galaxy_lock_path, damaged).unwrap();
    let error = match aot::verify_build_manifest(&manifest_path) {
        Ok(_) => panic!("drifted Galaxy resolution lock must be rejected"),
        Err(error) => error,
    };
    assert!(error.contains("payload hash mismatch"));
}
