use super::*;

#[test]
fn execution_inspect_issues_flag_missing_target_and_phase_mismatch() {
    let overview = ExecutionInspectOverview {
        heterogeneous_domains: 2,
        domains: vec![
            ExecutionInspectDomainOverview {
                domain_family: "network".to_owned(),
                selected_lowering_target: None,
                phase_count: 0,
                event_count: 0,
                resource_keys: vec![],
                output_handles: vec![],
            },
            ExecutionInspectDomainOverview {
                domain_family: "shader".to_owned(),
                selected_lowering_target: Some("metal.apple-silicon-gpu".to_owned()),
                phase_count: 4,
                event_count: 3,
                resource_keys: vec!["shader_buffer".to_owned()],
                output_handles: vec![],
            },
        ],
    };

    let issues = execution_inspect_issues(&overview);

    assert_eq!(
        issues,
        vec![
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_target".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "zero_phases".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_network_request_packet".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_network_active_response".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "network".to_owned(),
                issue: "missing_network_response_handle".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "shader".to_owned(),
                issue: "phase_event_mismatch(4->3)".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "shader".to_owned(),
                issue: "missing_shader_frame_target".to_owned(),
            },
            ExecutionInspectIssue {
                domain_family: "shader".to_owned(),
                issue: "missing_shader_draw_handle".to_owned(),
            },
        ]
    );
}

#[test]
fn compile_command_writes_end_to_end_project_outputs() {
    let project_name = "compile_command_smoke";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "compile_command_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
            mod cpu Main {
              fn main() -> i64 {
                return 1;
              }
            }
            "#,
    );
    let output_dir = temp_dir("compile_command_outputs");
    let output_stem = project_name.to_owned();

    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    for path in [
        output_dir.join(format!("{output_stem}.ast.txt")),
        output_dir.join(format!("{output_stem}.nir.txt")),
        output_dir.join(format!("{output_stem}.yir")),
        output_dir.join(format!("{output_stem}.ll")),
        output_dir.join(&output_stem),
        output_dir.join("nuis.doc-index.json"),
        output_dir.join("nuis.build.manifest.toml"),
        output_dir.join("nuis.executable.envelope.toml"),
        output_dir.join("nuis.compiled.artifact"),
        output_dir.join("nuis.project.toml"),
        output_dir.join("nuis.project.plan.txt"),
        output_dir.join("nuis.project.organization.txt"),
        output_dir.join("nuis.project.exchange.txt"),
        output_dir.join("nuis.project.modules.txt"),
        output_dir.join("nuis.project.docs.txt"),
        output_dir.join("nuis.project.imports.txt"),
        output_dir.join("nuis.project.galaxy.txt"),
        output_dir.join("nuis.project.links.txt"),
        output_dir.join("nuis.project.packet.txt"),
        output_dir.join("nuis.project.host_ffi.txt"),
        output_dir.join("nuis.project.abi.txt"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_text.contains("manifest_schema = \"nuis-build-manifest-v1\""));
    assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
    assert!(manifest_text.contains("loaded_nustar = [\"official.cpu\"]"));
    assert!(manifest_text.contains("doc_index_path = "));
    assert!(manifest_text.contains("doc_index_module_count = 1"));
    assert!(manifest_text.contains("doc_index_documented_item_count = 0"));
    assert!(manifest_text.contains("[[domain_build_unit]]"));
    assert!(manifest_text.contains(&format!("name = \"{project_name}\"")));
    assert!(manifest_text.contains("manifest_copy = "));
    assert!(manifest_text.contains("plan_index = "));
    assert!(manifest_text.contains("organization_index = "));
    assert!(manifest_text.contains("exchange_index = "));
    assert!(manifest_text.contains("modules_index = "));
    assert!(manifest_text.contains("docs_index = "));
    assert!(manifest_text.contains("docs_module_count = 1"));
    assert!(manifest_text.contains("docs_documented_module_count = 0"));
    assert!(manifest_text.contains("docs_documented_item_count = 0"));
    assert!(manifest_text.contains("imports_index = "));
    assert!(manifest_text.contains("imports_library_count = 0"));
    assert!(manifest_text.contains("imports_visible_library_count = 0"));
    assert!(manifest_text.contains("imports_visible_module_count = 1"));
    assert!(manifest_text.contains("imports_documented_visible_module_count = 0"));
    assert!(manifest_text.contains("imports_documented_visible_item_count = 0"));
    assert!(manifest_text.contains("galaxy_index = "));
    assert!(manifest_text.contains("galaxy_count = 0"));
    assert!(manifest_text.contains("documented_galaxy_count = 0"));
    assert!(manifest_text.contains("documented_galaxy_library_module_count = 0"));
    assert!(manifest_text.contains("documented_galaxy_item_count = 0"));
    assert!(manifest_text.contains("links_index = "));
    assert!(manifest_text.contains("packet_index = "));
    assert!(manifest_text.contains("host_ffi_index = "));
    assert!(manifest_text.contains("abi_index = "));

    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert!(manifest_report
        .doc_index_path
        .as_deref()
        .is_some_and(|path| path.ends_with("nuis.doc-index.json")));
    assert_eq!(manifest_report.doc_index_module_count, 1);
    assert_eq!(manifest_report.doc_index_documented_item_count, 0);
    assert_eq!(manifest_report.doc_index_checked, 1);
    assert_eq!(manifest_report.project_docs_module_count, 1);
    assert_eq!(manifest_report.project_docs_documented_module_count, 0);
    assert_eq!(manifest_report.project_docs_documented_item_count, 0);
    assert_eq!(manifest_report.project_imports_library_count, 0);
    assert_eq!(manifest_report.project_imports_visible_library_count, 0);
    assert_eq!(manifest_report.project_imports_visible_module_count, 1);
    assert_eq!(
        manifest_report.project_imports_documented_visible_module_count,
        0
    );
    assert_eq!(
        manifest_report.project_imports_documented_visible_item_count,
        0
    );
    assert_eq!(manifest_report.project_galaxy_count, 0);
    assert_eq!(manifest_report.project_documented_galaxy_count, 0);
    assert_eq!(
        manifest_report.project_documented_galaxy_library_module_count,
        0
    );
    assert_eq!(manifest_report.project_documented_galaxy_item_count, 0);
    assert_eq!(
        manifest_report.envelope_schema,
        "nuis-executable-envelope-v1"
    );
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert_eq!(manifest_report.artifact_binary_name, output_stem);
    assert!(Path::new(&manifest_report.envelope_path).exists());
    assert!(Path::new(&manifest_report.artifact_path).exists());
    assert!(manifest_report.project_metadata_checked >= 2);

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);
}

#[test]
fn compile_command_reuses_cached_project_outputs_without_recompiling() {
    let project_name = "compile_command_cache_hit_smoke";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "compile_command_cache_hit_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
            mod cpu Main {
              fn main() -> i64 {
                return 7;
              }
            }
            "#,
    );
    let output_dir = temp_dir("compile_command_cache_hit_outputs");

    run(CommandKind::Compile {
        input: project_root.clone(),
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let first_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(first_report.compile_cache_status.as_deref(), Some("miss"));
    assert_eq!(first_report.loaded_nustar, vec!["official.cpu".to_owned()]);

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let second_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(second_report.compile_cache_status.as_deref(), Some("hit"));
    assert_eq!(second_report.loaded_nustar, vec!["official.cpu".to_owned()]);
    assert_eq!(second_report.packaging_mode, "native-cpu-llvm");
    assert!(Path::new(&second_report.artifact_path).exists());
}

#[test]
fn compile_command_writes_host_file_ffi_project_outputs() {
    let project_name = "compile_command_host_file_smoke";
    let project_root = write_temp_project_fixture(
        project_name,
        r#"
name = "compile_command_host_file_smoke"
entry = "main.ns"
modules = ["main.ns"]
abi = ["cpu=cpu.arm64.apple_aapcs64"]
"#
        .trim_start(),
        r#"
            mod cffi Main {
              extern "c" fn host_file_open(path_handle: i64, flags: i64) -> i64;
              extern "c" fn host_file_read(file_handle: i64, buffer_handle: i64, len: i64) -> i64;
              extern "c" fn host_file_write(file_handle: i64, text_handle: i64) -> i64;
              extern "c" fn host_file_close(file_handle: i64) -> i64;
              extern "c" fn host_path_copy(src_handle: i64, dst_handle: i64) -> i64;
              extern "c" fn host_fs_exists(path_handle: i64) -> i64;

              fn main() -> i64 {
                let handle: i64 = host_file_open(2103, 1);
                let backing: ref Buffer = alloc_buffer(8, 0);
                host_file_read(handle, host_buffer_handle(backing), 8);
                host_file_write(handle, 777);
                host_file_close(handle);
                host_path_copy(2103, 2109);
                host_fs_exists(2109);
                return 0;
              }
            }
            "#,
    );
    let output_dir = temp_dir("compile_command_host_file_outputs");
    let output_stem = project_name.to_owned();

    run(CommandKind::Compile {
        input: project_root.clone(),
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
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_text = fs::read_to_string(output_dir.join("nuis.build.manifest.toml")).unwrap();
    assert!(manifest_text.contains("host_ffi_index = "));

    let host_ffi_text = fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
    assert!(host_ffi_text.contains("host_file_open"));
    assert!(host_ffi_text.contains("host_file_read"));
    assert!(host_ffi_text.contains("host_file_write"));
    assert!(host_ffi_text.contains("host_file_close"));
    assert!(host_ffi_text.contains("host_path_copy"));
    assert!(host_ffi_text.contains("host_fs_exists"));

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);

    let status = Command::new(output_dir.join(&output_stem))
        .status()
        .expect("expected compiled binary to launch");
    assert!(
        status.success(),
        "expected compiled binary to exit successfully"
    );
}

#[test]
fn compile_command_writes_benchmark_report_file_tooling_outputs() {
    let project_root = PathBuf::from("../../examples/projects/tooling/benchmark_report_file_demo");
    let output_dir = temp_dir("compile_command_benchmark_report_file_outputs");
    let output_stem = "benchmark_report_file_demo".to_owned();

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
        output_dir.join("nuis.project.plan.txt"),
    ] {
        assert!(path.exists(), "expected output `{}`", path.display());
    }

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_text.contains("name = \"benchmark_report_file_demo\""));
    assert!(manifest_text.contains("packaging_mode = \"native-cpu-llvm\""));
    assert!(manifest_text.contains("host_ffi_index = "));

    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(manifest_report.artifact_binary_name, output_stem);
    assert_eq!(manifest_report.artifact_schema, "nuis-compiled-artifact-v1");
    assert!(manifest_report.project_metadata_checked >= 6);

    let host_ffi_text = fs::read_to_string(output_dir.join("nuis.project.host_ffi.txt")).unwrap();
    assert!(host_ffi_text.contains("host_monotonic_time_ns"));
    assert!(host_ffi_text.contains("host_sleep_ns"));
    assert!(host_ffi_text.contains("host_file_open"));
    assert!(host_ffi_text.contains("host_file_write"));
    assert!(host_ffi_text.contains("host_file_close"));
    assert!(host_ffi_text.contains("host_temp_file_handle"));

    let artifact_report =
        aot::verify_nuis_compiled_artifact(output_dir.join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(artifact_report.binary_name, output_stem);
    assert_eq!(artifact_report.packaging_mode, "native-cpu-llvm");
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);

    let status = Command::new(output_dir.join(&output_stem))
        .status()
        .expect("expected compiled benchmark report binary to launch");
    assert!(
        status.success(),
        "expected compiled benchmark report binary to exit successfully"
    );
}

#[test]
fn verify_build_manifest_rejects_drifted_host_ffi_signature_hash() {
    let project_root = write_temp_project_fixture(
        "drifted_host_ffi_signature_hash",
        r#"
name = "drifted_host_ffi_signature_hash"
entry = "main.ns"
modules = ["main.ns"]
"#
        .trim_start(),
        r#"
mod cffi Main {
  extern "c" fn host_monotonic_time_ns() -> i64;
  extern "c" fn host_sleep_ns(duration_ns: i64) -> i64;

  fn main() -> i64 {
    let started: i64 = host_monotonic_time_ns();
    let slept: i64 = host_sleep_ns(1);
    let ended: i64 = host_monotonic_time_ns();
    if slept < 0 {
      return 1;
    }
    if ended < started {
      return 1;
    }
    return 0;
  }
}
"#,
    );
    let output_dir = temp_dir("compile_command_hetero_proxy_benchmark_drift_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let host_ffi_path = output_dir.join("nuis.project.host_ffi.txt");
    let host_ffi_text = fs::read_to_string(&host_ffi_path).unwrap();
    let damaged = host_ffi_text.replacen("signature_hash=fnv1a64:", "signature_hash=fnv1a64:0", 1);
    fs::write(&host_ffi_path, damaged).unwrap();

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let error = match aot::verify_build_manifest(&manifest_path) {
        Ok(_) => panic!("expected drifted host ffi signature hash to be rejected"),
        Err(error) => error,
    };
    assert!(error.contains("project host_ffi index"));
    assert!(error.contains("signature hash mismatch"));
}

#[test]
fn compile_command_carries_borrowed_utf8_capability_into_nsld_plan() {
    let project_root = write_temp_project_fixture(
        "borrowed_utf8_host_ffi_capability",
        r#"
name = "borrowed_utf8_host_ffi_capability"
entry = "main.ns"
modules = ["main.ns"]
"#
        .trim_start(),
        r#"
mod cffi Main {
  extern "c" fn host_text_line_count(text: String) -> i64;

  fn main() -> i64 {
    let lines: i64 = host_text_line_count("alpha\nbeta\n");
    if lines == 2 {
      return 0;
    }
    return 1;
  }
}
"#,
    );
    let output_dir = temp_dir("borrowed_utf8_host_ffi_capability_outputs");

    run(CommandKind::Compile {
        input: project_root,
        output_dir: output_dir.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: None,
    })
    .unwrap();

    let host_ffi_path = output_dir.join("nuis.project.host_ffi.txt");
    let host_ffi_text = fs::read_to_string(&host_ffi_path).unwrap();
    assert!(host_ffi_text.contains("memory_capability_count=1"));
    assert!(host_ffi_text.contains("kind=borrowed_utf8"));
    assert!(host_ffi_text.contains("slot=arg:0"));
    assert!(host_ffi_text.contains("length=nul_terminated"));
    assert!(host_ffi_text.contains("mutability=read_only"));
    assert!(host_ffi_text.contains("lifetime=call"));
    assert!(host_ffi_text.contains("destructor=none"));

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let manifest_report = aot::verify_build_manifest(&manifest_path).unwrap();
    let artifact = load_nuis_compiled_artifact(&manifest_path).unwrap();
    let link_plan = linker::build_link_plan(&manifest_report, &artifact);
    let link_plan_json = linker::render_link_plan_json(&link_plan);
    assert_eq!(link_plan.host_ffi.memory_capability_count, 1);
    assert!(link_plan.host_ffi.validation.link_allowed);
    assert!(link_plan_json.contains("\"host_ffi_memory_capability_count\":1"));
    assert!(link_plan_json.contains("\"memory_capability_count\":1"));
    assert!(link_plan_json.contains("kind=borrowed_utf8"));

    let damaged = host_ffi_text.replace("lifetime=call", "lifetime=retained");
    fs::write(&host_ffi_path, damaged).unwrap();
    let error = match aot::verify_build_manifest(&manifest_path) {
        Ok(_) => panic!("expected borrowed UTF-8 lifetime drift to be rejected"),
        Err(error) => error,
    };
    assert!(error.contains("lifetime, length, mutability"));
    assert!(error.contains("policy drift"));
}

#[test]
fn benchmark_inventory_collects_declared_benchmarks() {
    let artifacts = pipeline::compile_source(
            r#"
            mod cpu Main {
              benchmark("sum_loop", warmup_iters=4, measure_iters=32, timeout_ms=25, clock_domain="global", clock_policy="bridge")
              async fn sum_loop() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

    let entries = collect_benchmark_inventory(&artifacts);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].symbol, "cpu::Main::sum_loop");
    assert_eq!(entries[0].label, "sum_loop");
    assert!(entries[0].is_async);
    assert_eq!(entries[0].return_type, "i64");
    assert_eq!(entries[0].warmup_iters, Some(4));
    assert_eq!(entries[0].measure_iters, Some(32));
    assert_eq!(entries[0].timeout_ms, Some(25));
    assert_eq!(entries[0].clock_domain.as_deref(), Some("global"));
    assert_eq!(entries[0].clock_policy.as_deref(), Some("bridge"));
}

#[test]
fn inspect_benchmarks_json_exposes_metadata() {
    let artifacts = pipeline::compile_source(
        r#"
            mod cpu Main {
              benchmark("sum_loop", measure_iters=32)
              fn sum_loop() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return sum_loop();
              }
            }
            "#,
    )
    .unwrap();

    let json = inspect_benchmarks_json(Path::new("main.ns"), &artifacts);
    assert!(json.contains("\"kind\":\"nuis_benchmark_inventory\""));
    assert!(json.contains("\"input\":\"main.ns\""));
    assert!(json.contains("\"benchmark_count\":1"));
    assert!(json.contains("\"symbol\":\"cpu::Main::sum_loop\""));
    assert!(json.contains("\"label\":\"sum_loop\""));
    assert!(json.contains("\"measure_iters\":32"));
}

#[test]
fn inspect_docs_json_exposes_documented_items() {
    let ast = frontend::parse_nuis_ast(
        r#"
            /// module docs
            mod cpu Docs {
              /// function docs
              fn answer() -> i32 {
                42
              }
            }
            "#,
    )
    .unwrap();

    let indexes = vec![frontend::extract_ast_doc_index(&ast)];
    let json = inspect_docs_json(Path::new("main.ns"), &indexes);
    assert!(json.contains("\"kind\":\"nuis_doc_index\""));
    assert!(json.contains("\"input\":\"main.ns\""));
    assert!(json.contains("\"module_count\":1"));
    assert!(json.contains("\"documented_item_count\":2"));
    assert!(json.contains("\"module_path\":\"cpu.Docs\""));
    assert!(json.contains("\"kind\":\"module\""));
    assert!(json.contains("\"path\":\"cpu.Docs\""));
    assert!(json.contains("\"docs\":[\"module docs\"]"));
    assert!(json.contains("\"signature\":\"mod cpu Docs\""));
    assert!(json.contains("\"kind\":\"function\""));
    assert!(json.contains("\"path\":\"cpu.Docs::answer\""));
    assert!(json.contains("\"docs\":[\"function docs\"]"));
    assert!(json.contains("\"signature\":\"fn answer() -> i32\""));
}

#[test]
fn collect_doc_indexes_reads_single_source_input() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nuis_doc_index_{nonce}.ns"));
    std::fs::write(
        &path,
        r#"
            /// module docs
            mod cpu Docs {
              /// value docs
              const ANSWER: i32 = 42;
            }
            "#,
    )
    .unwrap();

    let indexes = collect_doc_indexes(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    assert_eq!(indexes.len(), 1);
    assert_eq!(indexes[0].module_path, "cpu.Docs");
    assert_eq!(indexes[0].items.len(), 2);
    assert_eq!(indexes[0].items[0].path, "cpu.Docs");
    assert_eq!(indexes[0].items[1].path, "cpu.Docs::ANSWER");
}

#[test]
fn write_json_output_persists_payload() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nuis_doc_index_output_{nonce}.json"));
    write_json_output(&path, "{\"kind\":\"nuis_doc_index\"}").unwrap();
    let written = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    assert_eq!(written, "{\"kind\":\"nuis_doc_index\"}");
}

#[test]
fn inspect_galaxy_docs_json_reports_documented_library_modules() {
    let summary = inspect_galaxy_doc_summary("pixelmagic").unwrap();
    let json = inspect_galaxy_docs_json(&summary);

    assert!(json.contains("\"kind\":\"nuis_galaxy_doc_index\""));
    assert!(json.contains("\"galaxy\":\"pixelmagic\""));
    assert!(json.contains("\"package_id\":\"nuis.pixelmagic\""));
    assert!(json.contains("\"documented_library_module_count\":"));
    assert!(json.contains("\"documented_item_count\":"));
    assert!(json.contains("\"library_module\":\"lib/image_contracts.ns\""));
    assert!(json.contains("\"module_path\":\"cpu.PixelMagicContracts\""));
}

#[test]
fn inspect_stdlib_docs_json_reports_all_official_galaxies() {
    let summary = inspect_stdlib_doc_summary().unwrap();
    let json = inspect_stdlib_docs_json(&summary);

    assert!(json.contains("\"kind\":\"nuis_stdlib_doc_index\""));
    assert!(json.contains("\"galaxy_count\":5"));
    assert!(json.contains("\"galaxy\":\"core\""));
    assert!(json.contains("\"galaxy\":\"std\""));
    assert!(json.contains("\"galaxy\":\"pixelmagic\""));
    assert!(json.contains("\"galaxy\":\"witsage\""));
    assert!(json.contains("\"galaxy\":\"ns-nova\""));
}
