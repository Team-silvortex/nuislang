#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

#[allow(dead_code)]
#[path = "../src/final_executable_elf_test_fixture.rs"]
mod elf_fixture;

use nuisc::aot::{
    host_cpu_build_target, write_build_manifest, BuildManifestContext, BuildManifestProjectInfo,
    CompileArtifacts, CompileHostObject,
};
use std::{
    env, fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const SELECTION_EVIDENCE_FILE: &str = "nuis.nsld.final-output-selection-evidence.json";

#[test]
fn public_final_output_policy_persists_and_enforces_multi_dependency_elf_evidence() {
    let dir = unique_temp_dir("nsld-cli-linux-multi-dependency-selection");
    fs::create_dir_all(&dir).unwrap();
    let compatibility_executable = find_path_executable("true");
    let host_ffi_index = dir.join("nuis.project.host_ffi.txt");
    write_host_ffi_index(&host_ffi_index, "f64(f64)");
    let manifest = write_multi_dependency_fixture(&dir, &compatibility_executable, &host_ffi_index);
    let output_path = dir.join("demo.bin");
    let receipt_path = dir.join("nuis.nsld.registered-loader-probe-admission.toml");
    let evidence_path = dir.join(SELECTION_EVIDENCE_FILE);
    let compatibility_before = fs::read(&output_path).unwrap();

    let admission = run_nsld(&[
        "final-executable-private-image-loader-probe",
        manifest.to_str().unwrap(),
        "--apply",
        "--json",
    ]);
    assert!(admission.contains("\"status\":\"registered-loader-probe-admission-replay-verified\""));
    assert!(admission.contains("\"valid\":true"), "{admission}");
    assert!(admission.contains("\"current_private_image_matches\":true"));
    assert!(receipt_path.is_file());
    assert_eq!(
        fs::metadata(&receipt_path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let default_output = run_nsld(&[
        "final-executable-output",
        manifest.to_str().unwrap(),
        "--json",
    ]);
    assert!(default_output.contains("\"policy_id\":\"compatibility-default\""));
    assert!(default_output.contains("\"explicit_request\":false"));
    assert!(!evidence_path.exists());
    assert_eq!(fs::read(&output_path).unwrap(), compatibility_before);

    let selection_plan = run_nsld(&[
        "final-executable-output",
        manifest.to_str().unwrap(),
        "--output-policy",
        "admitted-private-image",
        "--json",
    ]);
    assert!(selection_plan.contains("\"status\":\"ready-private-image-selection-plan\""));
    assert!(selection_plan.contains("\"explicit_request\":true"));
    assert!(selection_plan.contains("\"apply_requested\":false"));
    assert!(selection_plan.contains("\"selection_ready\":true"));
    assert!(selection_plan.contains("\"selected\":false"));
    assert!(selection_plan.contains("\"admission_receipt_valid\":true"));
    let plan_ledger = json_string_value(&selection_plan, "selection_ledger_sha256");
    let planned_evidence = fs::read_to_string(&evidence_path).unwrap();
    assert!(planned_evidence
        .contains("\"contract\":\"nuis-nsld-final-output-selection-evidence-file-v1\""));
    assert!(planned_evidence.contains("\"policy_id\":\"admitted-private-image\""));
    assert!(planned_evidence.contains("\"selected\":false"));
    assert!(planned_evidence.contains(&plan_ledger));
    assert!(!planned_evidence.contains("selected_output_path"));
    assert!(!planned_evidence.contains(dir.to_str().unwrap()));
    assert_eq!(
        fs::metadata(&evidence_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(fs::read(&output_path).unwrap(), compatibility_before);

    let applied = run_nsld(&[
        "final-executable-output",
        manifest.to_str().unwrap(),
        "--output-policy",
        "admitted-private-image",
        "--apply",
        "--json",
    ]);
    assert!(applied.contains("\"status\":\"private-image-selected\""));
    assert!(applied.contains("\"publication_status\":\"private-image-published\""));
    assert!(applied.contains("\"installation_attempted\":true"));
    assert!(applied.contains("\"selected\":true"));
    assert!(applied.contains("\"selected_output_identity_matches\":true"));
    let apply_ledger = json_string_value(&applied, "selection_ledger_sha256");
    let applied_evidence = fs::read_to_string(&evidence_path).unwrap();
    assert_ne!(plan_ledger, apply_ledger);
    assert!(applied_evidence.contains(&apply_ledger));
    assert!(applied_evidence.contains("\"selected\":true"));
    let private_image = fs::read(&output_path).unwrap();
    assert_ne!(private_image, compatibility_before);
    assert_eq!(
        fs::metadata(&output_path).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let execution = Command::new(&output_path).output().unwrap();
    assert!(execution.status.success());
    assert!(execution.stdout.is_empty());
    assert!(execution.stderr.is_empty());

    fs::write(&output_path, &compatibility_before).unwrap();
    write_host_ffi_index(&host_ffi_index, "f32(f32)");
    let rejected = run_nsld_failure(&[
        "final-executable-output",
        manifest.to_str().unwrap(),
        "--output-policy",
        "admitted-private-image",
        "--apply",
        "--json",
    ]);
    let rejected_stdout = String::from_utf8(rejected.stdout).unwrap();
    assert!(rejected_stdout.contains("\"status\":\"blocked-private-image-selection\""));
    assert!(rejected_stdout.contains("\"admission_receipt_valid\":false"));
    assert!(rejected_stdout.contains("registered-loader-probe-admission-current-image-mismatch"));
    assert!(rejected_stdout.contains("\"installation_attempted\":false"));
    assert!(rejected_stdout.contains("\"selected\":false"));
    assert_eq!(fs::read(&output_path).unwrap(), compatibility_before);
    let rejected_ledger = json_string_value(&rejected_stdout, "selection_ledger_sha256");
    let rejected_evidence = fs::read_to_string(&evidence_path).unwrap();
    assert!(rejected_evidence.contains(&rejected_ledger));
    assert!(rejected_evidence.contains("\"status\":\"blocked-private-image-selection\""));
    assert!(rejected_evidence.contains("\"selected\":false"));
    assert_ne!(rejected_evidence, applied_evidence);

    fs::remove_dir_all(dir).unwrap();
}

fn write_multi_dependency_fixture(
    dir: &Path,
    compatibility_executable: &Path,
    host_ffi_index: &Path,
) -> PathBuf {
    let source = dir.join("demo.ns");
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let llvm_ir = dir.join("demo.ll");
    let binary = dir.join("demo.bin");
    let program_object = dir.join("demo.host-program.o");
    let runtime_object = dir.join("demo.host-runtime.o");
    fs::write(&source, "fn main() -> i64 { 0 }\n").unwrap();
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&llvm_ir, "llvm").unwrap();
    fs::write(
        &program_object,
        elf_fixture::elf_multi_dependency_program_object(),
    )
    .unwrap();
    fs::write(
        &runtime_object,
        elf_fixture::elf_linux_exit_runtime_object(),
    )
    .unwrap();
    fs::copy(compatibility_executable, &binary).unwrap();

    let cpu_target = host_cpu_build_target();
    assert_eq!(cpu_target.machine_arch, "x86_64");
    assert_eq!(cpu_target.machine_os, "linux");
    assert_eq!(cpu_target.object_format, "elf");
    assert_eq!(cpu_target.calling_abi, "sysv64");
    let manifest = write_build_manifest(
        dir,
        &CompileArtifacts {
            ast_path: ast.display().to_string(),
            nir_path: nir.display().to_string(),
            yir_path: yir.display().to_string(),
            llvm_ir_path: llvm_ir.display().to_string(),
            binary_path: binary.display().to_string(),
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
            loaded_nustar: vec!["official.cpu".to_owned(), "official.cffi".to_owned()],
            compile_cache: None,
            project: Some(project_info(host_ffi_index, &cpu_target.abi)),
            doc_index: None,
            cpu_target,
        },
    )
    .unwrap();
    PathBuf::from(manifest)
}

fn project_info(host_ffi_index: &Path, cpu_abi: &str) -> BuildManifestProjectInfo {
    BuildManifestProjectInfo {
        name: "multi-dependency-cli".to_owned(),
        abi_mode: "explicit".to_owned(),
        artifact_provider_metadata: Vec::new(),
        code_asset_requirements: Vec::new(),
        abi_graph_summary: None,
        abi_entries: vec![("cpu".to_owned(), cpu_abi.to_owned())],
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
        galaxy_resolution_lock_path: None,
        galaxy_resolution_sha256: None,
        galaxy_count: 0,
        galaxy_documented_count: 0,
        galaxy_documented_library_module_count: 0,
        galaxy_documented_item_count: 0,
        links_index_path: None,
        packet_index_path: None,
        host_ffi_index_path: Some(host_ffi_index.display().to_string()),
        abi_index_path: None,
    }
}

fn write_host_ffi_index(path: &Path, cos_signature: &str) {
    let entries = [
        ("libc", "getrandom", "isize(ref u8, usize, u32)"),
        ("libm", "cos", cos_signature),
        ("libc", "sched_yield", "i32()"),
    ];
    let mut source = String::new();
    for (abi, symbol, signature) in entries {
        let hash = yir_core::ffi::ffi_symbol_signature_hash(abi, symbol, signature);
        source.push_str(&format!(
            "abi={abi}\tsymbol={symbol}\tsignature_pattern={signature}\tsignature_hash={hash}\tpolicy=signature-whitelist-required\tmemory_capability_count=0\tmemory_capabilities=-\n"
        ));
    }
    fs::write(path, source).unwrap();
}

fn run_nsld(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_nsld"))
        .args(args)
        .env_remove("NUIS_NSLD_HOST_FINALIZER_POLICY")
        .env_remove("NUIS_NSLD_ALLOW_HOST_FINALIZER")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "nsld {} failed\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn run_nsld_failure(args: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_nsld"))
        .args(args)
        .env_remove("NUIS_NSLD_HOST_FINALIZER_POLICY")
        .env_remove("NUIS_NSLD_ALLOW_HOST_FINALIZER")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "nsld {} unexpectedly succeeded",
        args.join(" ")
    );
    output
}

fn json_string_value(source: &str, key: &str) -> String {
    let marker = format!("\"{key}\":\"");
    let rest = source
        .split_once(&marker)
        .unwrap_or_else(|| panic!("JSON field `{key}` is missing from {source}"))
        .1;
    rest.split_once('"')
        .unwrap_or_else(|| panic!("JSON field `{key}` is unterminated"))
        .0
        .to_owned()
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
