use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct TempProject(PathBuf);

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn clocked_kernel_result_receipt_runs_as_native_binary() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let project =
        TempProject(std::env::temp_dir().join(format!("nuis_provider_completion_receipt_{nonce}")));
    let output_dir = project.0.join("out");
    fs::create_dir_all(&project.0).unwrap();
    fs::write(
        project.0.join("nuis.toml"),
        "name = \"provider_completion_receipt\"\nversion = \"0.1.0\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n",
    )
    .unwrap();
    fs::write(
        project.0.join("main.ns"),
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let input = kernel_tensor(1, 1, "7");
            let result: KernelResult<i64> =
              kernel_result(kernel_reduce_sum(input), 17);
            let token: i64 = kernel_completion_token(result);
            let clock: i64 = kernel_completion_clock(result);
            let root: i64 = kernel_completion_root(result);
            if token > 0 && clock == 17 && root > 0 {
              return 0;
            }
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "compile",
            &project.0.display().to_string(),
            &output_dir.display().to_string(),
        ])
        .output()
        .expect("run nuisc compile");
    assert!(
        compile.status.success(),
        "compile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let status = Command::new(output_dir.join("provider_completion_receipt"))
        .status()
        .expect("run provider receipt binary");
    assert_eq!(status.code(), Some(0));
}

#[test]
fn ns_nova_rejects_mismatched_provider_receipt_fields_natively() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let project =
        TempProject(std::env::temp_dir().join(format!("nuis_ns_nova_receipt_validation_{nonce}")));
    let output_dir = project.0.join("out");
    fs::create_dir_all(&project.0).unwrap();
    fs::write(
        project.0.join("nuis.toml"),
        concat!(
            "name = \"ns_nova_receipt_validation\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main.ns\"\n",
            "modules = [\"main.ns\"]\n",
            "galaxy = [\"ns-nova=workspace\"]\n",
            "galaxy_imports = [\"ns-nova:lib/app_runtime.ns\"]\n",
        ),
    )
    .unwrap();
    fs::write(
        project.0.join("main.ns"),
        r#"
        use cpu NovaAppRuntime;

        mod cpu Main {
          fn main() -> i64 {
            let state: NovaAppState = NovaAppRuntime.open(64, 64, 60, 0);
            let frame: NovaFrameTransaction = NovaAppRuntime.begin_frame(state, 1);
            let root: i64 = 41;
            let clock: i64 = 1;
            let token: i64 = NovaAppRuntime.completion_receipt_token(root, clock);
            let valid: NovaFrameResultHandle =
              NovaAppRuntime.capture_frame_result(frame, 1, token, clock, root);
            let bad_token: NovaFrameResultHandle =
              NovaAppRuntime.capture_frame_result(frame, 1, token + 2, clock, root);
            let bad_clock: NovaFrameResultHandle =
              NovaAppRuntime.capture_frame_result(frame, 1, token, clock + 1, root);
            let bad_root: NovaFrameResultHandle =
              NovaAppRuntime.capture_frame_result(frame, 1, token, clock, root + 2);
            let valid_submit: NovaFrameTransaction =
              NovaAppRuntime.submit_frame(frame, valid);
            let token_submit: NovaFrameTransaction =
              NovaAppRuntime.submit_frame(frame, bad_token);
            let clock_submit: NovaFrameTransaction =
              NovaAppRuntime.submit_frame(frame, bad_clock);
            let root_submit: NovaFrameTransaction =
              NovaAppRuntime.submit_frame(frame, bad_root);
            let valid_phase: i64 = valid_submit.phase;
            let committed: NovaAppState = NovaAppRuntime.commit_frame(valid_submit);
            let next_frame: NovaFrameTransaction = NovaAppRuntime.begin_frame(committed, 2);
            let rebound_root: i64 = root + 2;
            let rebound_token: i64 =
              NovaAppRuntime.completion_receipt_token(rebound_root, 2);
            let rebound: NovaFrameResultHandle = NovaAppRuntime.capture_frame_result(
              next_frame,
              1,
              rebound_token,
              2,
              rebound_root
            );
            let rebound_submit: NovaFrameTransaction =
              NovaAppRuntime.submit_frame(next_frame, rebound);
            if valid_phase == NovaAppRuntime.phase_submitted() &&
              token_submit.phase == NovaAppRuntime.phase_rejected() &&
              clock_submit.phase == NovaAppRuntime.phase_rejected() &&
              root_submit.phase == NovaAppRuntime.phase_rejected() &&
              rebound_submit.phase == NovaAppRuntime.phase_rejected() {
              return 0;
            }
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "compile",
            &project.0.display().to_string(),
            &output_dir.display().to_string(),
        ])
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .output()
        .expect("run nuisc compile");
    assert!(
        compile.status.success(),
        "compile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let status = Command::new(output_dir.join("ns_nova_receipt_validation"))
        .status()
        .expect("run NS Nova receipt validation binary");
    assert_eq!(status.code(), Some(0));
}
