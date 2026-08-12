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
    let check = run_nsld("check", &manifest);
    let artifact_chain = run_nsld("artifact-chain", &manifest);
    let launcher = run_nsld("final-executable-launcher-manifest", &manifest);
    let launcher_dry_run = run_nsld("final-executable-launcher-dry-run", &manifest);

    let actual_binary = fs::read(&final_binary).unwrap();
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
    fs::write(&program_object, minimal_arm64_object()).unwrap();
    fs::write(&runtime_object, minimal_arm64_object()).unwrap();
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

fn minimal_arm64_object() -> Vec<u8> {
    let mut bytes = vec![0u8; 104];
    bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
    bytes[4..8].copy_from_slice(&0x0100_000cu32.to_le_bytes());
    bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes[20..24].copy_from_slice(&72u32.to_le_bytes());
    bytes[32..36].copy_from_slice(&0x19u32.to_le_bytes());
    bytes[36..40].copy_from_slice(&72u32.to_le_bytes());
    bytes
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
