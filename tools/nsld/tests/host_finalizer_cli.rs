#![cfg(all(target_os = "macos", target_arch = "aarch64"))]

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuisc::aot::{
    host_cpu_build_target, write_build_manifest, BuildManifestContext, CompileArtifacts,
    CompileHostObject,
};

#[test]
fn cli_materializes_and_runs_registered_internal_macho_artifact_image() {
    let dir = unique_temp_dir("nsld-cli-internal-macho-finalizer");
    fs::create_dir_all(&dir).unwrap();
    let source_executable = find_path_executable("true");
    let expected_binary = fs::read(&source_executable).unwrap();
    let manifest = write_native_cpu_fixture(&dir, &source_executable);
    let final_binary = dir.join("demo.bin");

    let mut non_executable = fs::metadata(&final_binary).unwrap().permissions();
    non_executable.set_mode(0o644);
    fs::set_permissions(&final_binary, non_executable).unwrap();

    let drive = run_nsld_args(&[
        "drive",
        manifest.to_str().unwrap(),
        "--apply",
        "--until-clean",
        "--json",
    ]);
    let output = run_nsld("final-executable-output", &manifest);
    let invoke_plan = run_nsld("final-executable-host-invoke-plan", &manifest);
    let invoke_plan_text = run_nsld_args(&[
        "final-executable-host-invoke-plan",
        manifest.to_str().unwrap(),
    ]);
    let check = run_nsld("check", &manifest);
    let artifact_chain = run_nsld("artifact-chain", &manifest);
    let launcher = run_nsld("final-executable-launcher-manifest", &manifest);
    let launcher_dry_run = run_nsld("final-executable-launcher-dry-run", &manifest);

    let actual_binary = fs::read(&final_binary).unwrap();
    let persisted_invoke_plan =
        fs::read_to_string(dir.join("nuis.nsld.final-executable-host-invoke-plan.toml")).unwrap();
    let executable_mode = fs::metadata(&final_binary).unwrap().permissions().mode();
    let launched = Command::new(&final_binary).output().unwrap();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(actual_binary, expected_binary);
    assert_ne!(executable_mode & 0o111, 0);
    assert!(launched.status.success());
    assert!(invoke_plan
        .contains("\"finalizer_contract\":\"nuis-nsld-executable-finalizer-registry-v1\""));
    assert!(invoke_plan
        .contains("\"finalizer_provider_id\":\"nsld.finalizer.mach-o.arm64.artifact-image-v1\""));
    assert!(invoke_plan
        .contains("\"finalizer_execution_kind\":\"registered-nsld-artifact-image-writer\""));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-host-object-linkage-v1\""));
    assert!(invoke_plan.contains("\"relocation_count\":2"));
    assert!(invoke_plan.contains("\"internally_resolved_symbol_count\":1"));
    assert!(invoke_plan.contains("\"unresolved_external_symbols\":[\"_puts\"]"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-placement-binding-v1\""));
    assert!(
        invoke_plan.contains("\"status\":\"placement-ready-with-external-compatibility-boundary\"")
    );
    assert!(invoke_plan.contains("\"merged_section_count\":1"));
    assert!(invoke_plan.contains("\"section_placement_count\":2"));
    assert!(invoke_plan.contains("\"symbol_binding_count\":2"));
    assert!(invoke_plan.contains("\"internally_bound_symbol_count\":1"));
    assert!(invoke_plan.contains("\"external_compatibility_symbol_count\":1"));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-relocation-application-v1\"")
    );
    assert!(invoke_plan.contains("\"status\":\"planned-with-platform-structure-boundary\""));
    assert!(invoke_plan.contains("\"relocation_count\":2"));
    assert!(invoke_plan.contains("\"registered_kind_count\":1"));
    assert!(invoke_plan.contains("\"ready_application_count\":1"));
    assert!(invoke_plan.contains("\"platform_structure_count\":1"));
    assert!(invoke_plan.contains("\"relocation_kind\":\"arm64-branch26\""));
    assert!(invoke_plan.contains("\"action_kind\":\"rewrite-branch26\""));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-materialization-preview-v1\"")
    );
    assert!(invoke_plan.contains("\"status\":\"preview-ready-with-platform-structure-boundary\""));
    assert!(invoke_plan.contains("\"image_span_bytes\":16"));
    assert!(invoke_plan.contains("\"previewed_patch_count\":1"));
    assert!(invoke_plan.contains("\"deferred_patch_count\":1"));
    assert!(invoke_plan.contains("\"source_bytes_hex\":\"00000094\""));
    assert!(invoke_plan.contains("\"encoded_bytes_hex\":\"02000094\""));
    assert!(invoke_plan.contains("\"source_output_offset\":0"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-patch-application-v1\""));
    assert!(invoke_plan
        .contains("\"status\":\"direct-patches-applied-with-platform-structure-boundary\""));
    assert!(invoke_plan.contains("\"expected_patch_count\":1"));
    assert!(invoke_plan.contains("\"applied_patch_count\":1"));
    assert!(invoke_plan.contains("\"write_once_span_count\":1"));
    assert!(invoke_plan.contains("\"post_write_bytes_hash\":"));
    assert!(invoke_plan.contains("\"application_ledger_hash\":"));
    assert!(
        invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-platform-structure-plan-v1\"")
    );
    assert!(invoke_plan.contains("\"status\":\"allocated-ready-for-platform-patching\""));
    assert!(invoke_plan.contains("\"deferred_relocation_count\":1"));
    assert!(invoke_plan.contains("\"target_count\":1"));
    assert!(invoke_plan.contains("\"base_image_span_bytes\":16"));
    assert!(invoke_plan.contains("\"planned_image_span_bytes\":40"));
    assert!(invoke_plan.contains("\"stub_region_offset\":16"));
    assert!(invoke_plan.contains("\"stub_entry_count\":1"));
    assert!(invoke_plan.contains("\"got_region_offset\":32"));
    assert!(invoke_plan.contains("\"got_entry_count\":1"));
    assert!(invoke_plan.contains("\"target_symbol\":\"_puts\""));
    assert!(invoke_plan.contains("\"got_output_offset\":32"));
    assert!(invoke_plan.contains("\"stub_output_offset\":16"));
    assert!(invoke_plan.contains("\"patch_target_kind\":\"branch-stub\""));
    assert!(invoke_plan.contains("\"patch_target_output_offset\":16"));
    assert!(invoke_plan
        .contains("\"contract\":\"nuis-nsld-macho-arm64-platform-patch-application-v1\""));
    assert!(invoke_plan.contains("\"status\":\"platform-patches-applied-with-unresolved-binds\""));
    assert!(invoke_plan.contains("\"platform_image_span_bytes\":40"));
    assert!(invoke_plan.contains("\"expected_deferred_patch_count\":1"));
    assert!(invoke_plan.contains("\"applied_deferred_patch_count\":1"));
    assert!(invoke_plan.contains("\"stub_write_count\":1"));
    assert!(invoke_plan.contains("\"got_write_count\":1"));
    assert!(invoke_plan.contains("\"unresolved_bind_count\":1"));
    assert!(invoke_plan.contains("\"write_once_span_count\":3"));
    assert!(invoke_plan.contains("\"write_kind\":\"arm64-branch-stub\""));
    assert!(invoke_plan.contains("\"encoded_bytes_hex\":\"10000090101240f900021fd6\""));
    assert!(invoke_plan.contains("\"source_output_offset\":8"));
    assert!(invoke_plan.contains("\"patch_target_output_offset\":16"));
    assert!(invoke_plan.contains("\"status\":\"unresolved-external\""));
    assert!(invoke_plan.contains("\"got_output_offset\":32"));
    assert!(invoke_plan.contains("\"shell_layout_plan\":{"));
    assert!(invoke_plan.contains("\"contract\":\"nuis-nsld-macho-arm64-shell-layout-plan-v1\""));
    assert!(invoke_plan.contains("\"status\":\"layout-planned-with-code-signature-boundary\""));
    assert!(invoke_plan.contains("\"entry_symbol\":\"_nuis_entry\""));
    assert!(invoke_plan.contains("\"segment_name\":\"__LINKEDIT\""));
    assert!(invoke_plan.contains("\"section_name\":\"__stubs\""));
    assert!(invoke_plan.contains("\"section_name\":\"__got\""));
    assert!(invoke_plan.contains("\"record_kind\":\"external-undefined\""));
    assert!(invoke_plan.contains("\"target_symbol\":\"_puts\",\"dylib_ordinal\":1"));
    assert!(invoke_plan.contains("\"command_kind\":\"code-signature\""));
    assert!(invoke_plan.contains("\"code_signature_status\":\"required-payload-pending\""));
    assert!(invoke_plan.contains("\"target_symbol\":\"_nuis_runtime\""));
    assert!(invoke_plan
        .contains("\"symbol\":\"_nuis_runtime\",\"reference_object_id\":\"host.program-llvm\""));
    assert!(invoke_plan.contains("\"target_object_id\":\"host.runtime-shim\""));
    assert!(
        invoke_plan.contains("\"symbol\":\"_puts\",\"reference_object_id\":\"host.runtime-shim\"")
    );
    assert!(invoke_plan.contains("\"status\":\"external-compatibility\""));
    assert!(invoke_plan_text
        .contains("finalizer_input_placement_contract: nuis-nsld-macho-placement-binding-v1"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_relocation_application_contract: nuis-nsld-macho-arm64-relocation-application-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_relocation_application_count: 2"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_materialization: contract=nuis-nsld-macho-arm64-materialization-preview-v1"
    ));
    assert!(invoke_plan_text.contains(
        "finalizer_input_patch_application: contract=nuis-nsld-macho-arm64-patch-application-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_applied_patch: id="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_platform_structure: contract=nuis-nsld-macho-arm64-platform-structure-plan-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_platform_target: id="));
    assert!(invoke_plan_text.contains("symbol=_puts"));
    assert!(invoke_plan_text.contains("finalizer_input_platform_binding: relocation="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_platform_patch_application: contract=nuis-nsld-macho-arm64-platform-patch-application-v1"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_platform_write: id="));
    assert!(invoke_plan_text.contains("kind=arm64-branch-stub"));
    assert!(invoke_plan_text.contains("finalizer_input_platform_patch: relocation="));
    assert!(invoke_plan_text.contains("finalizer_input_platform_bind: id="));
    assert!(invoke_plan_text.contains("status=unresolved-external"));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_layout: contract=nuis-nsld-macho-arm64-shell-layout-plan-v1"
    ));
    assert!(invoke_plan_text.contains(
        "finalizer_input_shell_entry: rule=arm64.macho.program-entry.v1 symbol=_nuis_entry"
    ));
    assert!(invoke_plan_text.contains("finalizer_input_shell_segment: id="));
    assert!(invoke_plan_text.contains("name=__LINKEDIT"));
    assert!(invoke_plan_text.contains("finalizer_input_shell_bind: id="));
    assert!(invoke_plan_text.contains("finalizer_input_shell_command: id="));
    assert!(invoke_plan_text.contains(
        "finalizer_input_symbol_binding: symbol=_nuis_runtime reference=host.program-llvm:1 status=internal"
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_placement_contract = \"nuis-nsld-macho-placement-binding-v1\""));
    assert!(persisted_invoke_plan.contains("finalizer_input_merged_section_count = 1"));
    assert!(persisted_invoke_plan.contains("finalizer_input_section_placement_count = 2"));
    assert!(persisted_invoke_plan.contains("finalizer_input_symbol_binding_count = 2"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_relocation_application_contract = \"nuis-nsld-macho-arm64-relocation-application-v1\""
    ));
    assert!(persisted_invoke_plan.contains("finalizer_input_relocation_application_count = 2"));
    assert!(persisted_invoke_plan.contains("arm64-branch26|rewrite-branch26"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_materialization_contract = \"nuis-nsld-macho-arm64-materialization-preview-v1\""
    ));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_materialization_previewed_patch_count = 1")
    );
    assert!(persisted_invoke_plan.contains("00000094|02000094"));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_patch_application_contract = \"nuis-nsld-macho-arm64-patch-application-v1\""
    ));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_patch_application_applied_patch_count = 1")
    );
    assert!(persisted_invoke_plan.contains("finalizer_input_patch_application_patches = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_platform_structure_contract = \"nuis-nsld-macho-arm64-platform-structure-plan-v1\""
    ));
    assert!(persisted_invoke_plan.contains("finalizer_input_platform_structure_target_count = 1"));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_platform_structure_stub_entry_count = 1")
    );
    assert!(
        persisted_invoke_plan.contains("finalizer_input_platform_structure_got_entry_count = 1")
    );
    assert!(persisted_invoke_plan.contains("finalizer_input_platform_structure_targets = ["));
    assert!(persisted_invoke_plan.contains("finalizer_input_platform_structure_bindings = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_platform_patch_application_contract = \"nuis-nsld-macho-arm64-platform-patch-application-v1\""
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_platform_image_span_bytes = 40"));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_applied_deferred_patch_count = 1"));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_unresolved_bind_count = 1"));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_structure_writes = ["));
    assert!(
        persisted_invoke_plan.contains("finalizer_input_platform_patch_application_patches = [")
    );
    assert!(persisted_invoke_plan
        .contains("finalizer_input_platform_patch_application_bind_records = ["));
    assert!(persisted_invoke_plan.contains(
        "finalizer_input_shell_layout_contract = \"nuis-nsld-macho-arm64-shell-layout-plan-v1\""
    ));
    assert!(persisted_invoke_plan
        .contains("finalizer_input_shell_layout_entry_symbol = \"_nuis_entry\""));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_segments = ["));
    assert!(persisted_invoke_plan.contains("|__LINKEDIT|"));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_sections = ["));
    assert!(persisted_invoke_plan.contains("|__stubs|"));
    assert!(persisted_invoke_plan.contains("|__got|"));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_binds = ["));
    assert!(persisted_invoke_plan.contains("finalizer_input_shell_layout_load_commands = ["));
    assert!(invoke_plan.contains("\"invocation_kind\":\"registered-internal-finalizer\""));
    assert!(invoke_plan.contains("\"invocation_policy\":\"registered-internal\""));
    assert!(invoke_plan.contains("\"requires_explicit_allow\":false"));
    assert!(invoke_plan.contains("\"explicit_allow_present\":false"));
    assert!(invoke_plan.contains("\"would_invoke\":true"));
    assert!(drive.contains("\"kind\":\"nsld_drive_until_clean\""));
    assert!(drive.contains("\"completed\":true"), "{drive}");
    assert!(drive.contains("\"stop_reason\":\"clean\""), "{drive}");
    assert!(output.contains("\"output_kind\":\"host-native-executable\""));
    assert!(output.contains("\"present\":true"), "{output}");
    assert!(output.contains("\"nsld_owned_output\":true"), "{output}");
    assert!(output.contains("\"runnable_candidate\":true"), "{output}");
    assert!(check.contains("\"final_executable_output_present\":true"));
    assert!(check.contains("\"final_executable_output_runnable_candidate\":true"));
    assert!(artifact_chain.contains("\"stage_id\":\"final-executable-output\""));
    assert!(artifact_chain.contains("\"present\":true"));
    assert!(launcher.contains("\"final_output_present\":true"));
    assert!(launcher.contains("\"ready\":true"));
    assert!(launcher_dry_run.contains("\"final_output_readable\":true"));
    assert!(launcher_dry_run.contains("\"dry_run_ready\":true"));
}

fn run_nsld(command: &str, manifest: &Path) -> String {
    run_nsld_args(&[command, manifest.to_str().unwrap(), "--json"])
}

fn run_nsld_args(args: &[&str]) -> String {
    let command_label = args.join(" ");
    let output = Command::new(env!("CARGO_BIN_EXE_nsld"))
        .args(args)
        .env_remove("NUIS_NSLD_HOST_FINALIZER_POLICY")
        .env_remove("NUIS_NSLD_ALLOW_HOST_FINALIZER")
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsld {command_label}: {error}"));
    if !output.status.success() {
        panic!(
            "nsld {command_label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).unwrap()
}

fn write_native_cpu_fixture(dir: &Path, source_executable: &Path) -> PathBuf {
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    let source = dir.join("demo.ns");
    let program_object = dir.join("demo.host-program.o");
    let runtime_object = dir.join("demo.host-runtime.o");
    fs::write(&source, "fn main() -> i64 { 0 }\n").unwrap();
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(
        &program_object,
        arm64_object("_nuis_entry", "_nuis_runtime"),
    )
    .unwrap();
    fs::write(&runtime_object, arm64_object("_nuis_runtime", "_puts")).unwrap();
    fs::copy(source_executable, &bin).unwrap();

    let manifest = write_build_manifest(
        dir,
        &CompileArtifacts {
            ast_path: ast.display().to_string(),
            nir_path: nir.display().to_string(),
            yir_path: yir.display().to_string(),
            llvm_ir_path: ll.display().to_string(),
            binary_path: bin.display().to_string(),
            packaging_mode: "native-cpu-llvm".to_owned(),
            host_objects: vec![
                CompileHostObject {
                    object_id: "host.program-llvm".to_owned(),
                    role: "program-llvm".to_owned(),
                    path: program_object.display().to_string(),
                },
                CompileHostObject {
                    object_id: "host.runtime-shim".to_owned(),
                    role: "runtime-shim".to_owned(),
                    path: runtime_object.display().to_string(),
                },
            ],
        },
        &BuildManifestContext {
            input_path: source.display().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target: host_cpu_build_target(),
        },
    )
    .unwrap();
    PathBuf::from(manifest)
}

fn arm64_object(defined: &str, undefined: &str) -> Vec<u8> {
    const SEGMENT_OFFSET: usize = 32;
    const SECTION_OFFSET: usize = 104;
    const SYMTAB_OFFSET: usize = 184;
    const PAYLOAD_OFFSET: usize = 208;
    const RELOCATION_OFFSET: usize = 216;
    const SYMBOL_OFFSET: usize = 224;
    const STRING_OFFSET: usize = 256;
    let mut strings = vec![0];
    let defined_index = strings.len() as u32;
    strings.extend_from_slice(defined.as_bytes());
    strings.push(0);
    let undefined_index = strings.len() as u32;
    strings.extend_from_slice(undefined.as_bytes());
    strings.push(0);
    let mut bytes = vec![0u8; STRING_OFFSET + strings.len()];
    bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
    bytes[4..8].copy_from_slice(&0x0100_000cu32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    write_u32(&mut bytes, 16, 2);
    write_u32(&mut bytes, 20, 176);
    write_u32(&mut bytes, SEGMENT_OFFSET, 0x19);
    write_u32(&mut bytes, SEGMENT_OFFSET + 4, 152);
    write_u32(&mut bytes, SEGMENT_OFFSET + 64, 1);
    bytes[SECTION_OFFSET..SECTION_OFFSET + 6].copy_from_slice(b"__text");
    bytes[SECTION_OFFSET + 16..SECTION_OFFSET + 22].copy_from_slice(b"__TEXT");
    write_u64(&mut bytes, SECTION_OFFSET + 40, 8);
    write_u32(&mut bytes, SECTION_OFFSET + 48, PAYLOAD_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 56, RELOCATION_OFFSET as u32);
    write_u32(&mut bytes, SECTION_OFFSET + 60, 1);
    write_u32(&mut bytes, PAYLOAD_OFFSET, 0x9400_0000);
    write_u32(&mut bytes, SYMTAB_OFFSET, 0x2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 4, 24);
    write_u32(&mut bytes, SYMTAB_OFFSET + 8, SYMBOL_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 12, 2);
    write_u32(&mut bytes, SYMTAB_OFFSET + 16, STRING_OFFSET as u32);
    write_u32(&mut bytes, SYMTAB_OFFSET + 20, strings.len() as u32);
    write_u32(&mut bytes, RELOCATION_OFFSET, 0);
    write_u32(
        &mut bytes,
        RELOCATION_OFFSET + 4,
        1 | (1 << 24) | (2 << 25) | (1 << 27) | (2 << 28),
    );
    write_u32(&mut bytes, SYMBOL_OFFSET, defined_index);
    bytes[SYMBOL_OFFSET + 4] = 0x0f;
    bytes[SYMBOL_OFFSET + 5] = 1;
    write_u32(&mut bytes, SYMBOL_OFFSET + 16, undefined_index);
    bytes[SYMBOL_OFFSET + 20] = 0x01;
    bytes[STRING_OFFSET..].copy_from_slice(&strings);
    bytes
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}

fn find_path_executable(name: &str) -> PathBuf {
    env::var_os("PATH")
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("required test executable `{name}` was not found on PATH"))
}
