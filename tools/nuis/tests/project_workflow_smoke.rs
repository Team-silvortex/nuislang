use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {:?}: {error}", args))
}

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context} missing `{needle}`\nfull output:\n{haystack}"
    );
}

fn copied_example_project(path: &str, label: &str) -> PathBuf {
    let source = Path::new(path);
    let root = temp_dir(label);
    let dst = root.join(source.file_name().unwrap());
    copy_dir_recursive(source, &dst);
    dst
}

fn assert_project_workflow_smoke(project_path: &str, label: &str, expected_locked_keys: &[&str]) {
    let project_root = copied_example_project(project_path, label);
    let project_root_text = project_root.display().to_string();

    let status = run_nuis(&["project-status", &project_root_text]);
    assert_success(&status, "nuis project-status");
    let status_stdout = String::from_utf8_lossy(&status.stdout);
    assert_contains(
        &status_stdout,
        "abi_mode: auto-recommended",
        "project-status output",
    );
    assert_contains(
        &status_stdout,
        "recommended_next_step: project_lock_abi",
        "project-status output",
    );

    let lock = run_nuis(&["project-lock-abi", &project_root_text]);
    assert_success(&lock, "nuis project-lock-abi");
    let lock_stdout = String::from_utf8_lossy(&lock.stdout);
    assert_contains(
        &lock_stdout,
        "mode: auto -> explicit",
        "project-lock-abi output",
    );

    let manifest_path = project_root.join("nuis.toml");
    let manifest_text = fs::read_to_string(&manifest_path).unwrap();
    assert_contains(&manifest_text, "abi = [", "locked manifest");
    for key in expected_locked_keys {
        assert_contains(&manifest_text, key, "locked manifest");
    }

    let output_dir = temp_dir(&format!("{label}_release_check"));
    let release_check = run_nuis(&[
        "release-check",
        &project_root_text,
        &output_dir.display().to_string(),
    ]);
    assert_success(&release_check, "nuis release-check");
    let release_stdout = String::from_utf8_lossy(&release_check.stdout);
    assert_contains(&release_stdout, "release-check: ok", "release-check output");
    assert!(
        output_dir.join("nuis.build.manifest.toml").exists(),
        "expected build manifest in {}",
        output_dir.display()
    );
}

#[test]
fn auto_abi_projects_lock_and_release_check() {
    for (path, label, expected_locked_keys) in [
        (
            "../../examples/projects/tooling/cli_runtime_demo",
            "tooling_project_workflow_smoke",
            &["\"cpu="][..],
        ),
        (
            "../../examples/projects/domains/network_profile_demo",
            "network_project_workflow_smoke",
            &["\"cpu=", "\"network="][..],
        ),
        (
            "../../examples/projects/domains/shader_render_profile_demo",
            "shader_project_workflow_smoke",
            &["\"cpu=", "\"shader=", "\"data="][..],
        ),
        (
            "../../examples/projects/task/task_runtime_demo",
            "task_project_workflow_smoke",
            &["\"cpu="][..],
        ),
        (
            "../../examples/projects/state/counted_while_demo",
            "state_project_workflow_smoke",
            &["\"cpu="][..],
        ),
    ] {
        assert_project_workflow_smoke(path, label, expected_locked_keys);
    }
}
