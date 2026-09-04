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
