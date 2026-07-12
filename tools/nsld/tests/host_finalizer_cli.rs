#![cfg(unix)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuisc::aot::{
    host_cpu_build_target, write_build_manifest, BuildManifestContext, CompileArtifacts,
};
use std::os::unix::fs::PermissionsExt;

#[test]
fn cli_emit_final_executable_invokes_allowed_host_finalizer_without_env_pollution() {
    let dir = unique_temp_dir("nsld-cli-host-finalizer");
    fs::create_dir_all(&dir).unwrap();
    let manifest = write_native_cpu_fixture(&dir);
    let fake_bin = dir.join("fake-bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_fake_clang(&fake_bin.join("clang"));

    let drive = run_nsld_args_with_host_finalizer_env(
        &[
            "drive",
            manifest.to_str().unwrap(),
            "--apply",
            "--until-clean",
            "--json",
        ],
        &fake_bin,
    );

    let output = run_nsld_with_host_finalizer_env("final-executable-output", &manifest, &fake_bin);
    let check = run_nsld_with_host_finalizer_env("check", &manifest, &fake_bin);
    let next_action = run_nsld_with_host_finalizer_env("check-next-action", &manifest, &fake_bin);
    let artifact_chain = run_nsld_with_host_finalizer_env("artifact-chain", &manifest, &fake_bin);
    let launcher_manifest = run_nsld_with_host_finalizer_env(
        "final-executable-launcher-manifest",
        &manifest,
        &fake_bin,
    );
    let launcher_dry_run =
        run_nsld_with_host_finalizer_env("final-executable-launcher-dry-run", &manifest, &fake_bin);
    let final_binary = dir.join("demo.bin");
    let invoked_marker = dir.join("demo.bin.invoked");
    let final_binary_bytes = fs::read(&final_binary).unwrap();
    let invoked = invoked_marker.exists();
    fs::remove_dir_all(dir).unwrap();

    assert_eq!(final_binary_bytes, b"host-native-executable\n");
    assert!(invoked);
    assert!(
        drive.contains("\"kind\":\"nsld_drive_until_clean\""),
        "{drive}"
    );
    assert!(drive.contains("\"completed\":true"), "{drive}");
    assert!(drive.contains("\"stop_reason\":\"clean\""), "{drive}");
    assert!(
        output.contains("\"output_kind\":\"host-native-executable\""),
        "{output}"
    );
    assert!(
        output.contains("\"output_validation_mode\":\"host-native-presence-and-invoke-plan\""),
        "{output}"
    );
    assert!(output.contains("\"present\":true"), "{output}");
    assert!(output.contains("\"runnable_candidate\":true"), "{output}");
    assert!(
        output.contains("\"output_image_header_required\":false"),
        "{output}"
    );
    assert!(!output.contains("final-executable-output:image-header-invalid"));
    assert!(
        check.contains("\"final_executable_output_kind\":\"host-native-executable\""),
        "{check}"
    );
    assert!(
        check.contains(
            "\"final_executable_output_validation_mode\":\"host-native-presence-and-invoke-plan\""
        ),
        "{check}"
    );
    assert!(
        check.contains("\"final_executable_output_runnable_candidate\":true"),
        "{check}"
    );
    assert!(
        check.contains("\"final_executable_output_image_header_required\":false"),
        "{check}"
    );
    assert!(next_action.contains("\"available\":false"), "{next_action}");
    assert!(next_action.contains("\"command_id\":null"), "{next_action}");
    assert!(
        artifact_chain.contains("\"next_action_available\":false"),
        "{artifact_chain}"
    );
    assert!(
        artifact_chain.contains("\"stage_id\":\"final-executable-output\"")
            && artifact_chain.contains("\"present\":true"),
        "{artifact_chain}"
    );
    assert!(
        launcher_manifest.contains("\"output_kind\":\"host-native-executable\""),
        "{launcher_manifest}"
    );
    assert!(
        launcher_manifest
            .contains("\"output_validation_mode\":\"host-native-presence-and-invoke-plan\""),
        "{launcher_manifest}"
    );
    assert!(
        launcher_manifest.contains("\"final_output_present\":true"),
        "{launcher_manifest}"
    );
    assert!(
        launcher_manifest.contains("\"image_header_required\":false"),
        "{launcher_manifest}"
    );
    assert!(
        launcher_manifest.contains("\"ready\":true"),
        "{launcher_manifest}"
    );
    assert!(
        launcher_dry_run.contains("\"final_output_readable\":true"),
        "{launcher_dry_run}"
    );
    assert!(
        launcher_dry_run.contains("\"final_output_hash_matches\":true"),
        "{launcher_dry_run}"
    );
    assert!(
        launcher_dry_run.contains("\"dry_run_ready\":true"),
        "{launcher_dry_run}"
    );
    assert!(
        launcher_dry_run.contains("\"verify-host-native-output-presence\""),
        "{launcher_dry_run}"
    );
}

fn run_nsld_with_host_finalizer_env(command: &str, manifest: &Path, fake_bin: &Path) -> String {
    run_nsld_args_with_host_finalizer_env(
        &[command, manifest.to_str().unwrap(), "--json"],
        fake_bin,
    )
}

fn run_nsld_args_with_host_finalizer_env(args: &[&str], fake_bin: &Path) -> String {
    let mut path = env::split_paths(&env::var_os("PATH").unwrap_or_default()).collect::<Vec<_>>();
    path.insert(0, fake_bin.to_path_buf());
    let path = env::join_paths(path).unwrap();
    let command_label = args.join(" ");
    let output = Command::new(env!("CARGO_BIN_EXE_nsld"))
        .args(args)
        .env("PATH", path)
        .env("NUIS_NSLD_HOST_FINALIZER_POLICY", "allow-host-invoke")
        .env("NUIS_NSLD_ALLOW_HOST_FINALIZER", "1")
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

fn write_native_cpu_fixture(dir: &Path) -> PathBuf {
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    let source = dir.join("demo.ns");
    fs::write(&source, "fn main() -> i64 { 0 }\n").unwrap();
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&bin, "host-native-executable\n").unwrap();

    let manifest = write_build_manifest(
        dir,
        &CompileArtifacts {
            ast_path: ast.display().to_string(),
            nir_path: nir.display().to_string(),
            yir_path: yir.display().to_string(),
            llvm_ir_path: ll.display().to_string(),
            binary_path: bin.display().to_string(),
            packaging_mode: "native-cpu-llvm".to_owned(),
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

fn write_fake_clang(path: &Path) {
    fs::write(
        path,
        "#!/bin/sh\nout=\"\"\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-o\" ]; then\n    shift\n    out=\"$1\"\n  fi\n  shift\ndone\nif [ -z \"$out\" ]; then\n  exit 64\nfi\nprintf 'host-native-executable\\n' > \"$out\"\nprintf 'invoked\\n' > \"$out.invoked\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
