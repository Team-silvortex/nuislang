use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuisc_mixed_loop_terminal_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn mixed_loop_terminal_tree_runs_as_a_native_binary() {
    let project = temp_dir();
    let output_dir = project.join("out");
    fs::write(
        project.join("nuis.toml"),
        "name = \"mixed_loop_terminal\"\nversion = \"0.1.0\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n",
    )
    .unwrap();
    fs::write(
        project.join("main.ns"),
        r#"
        mod cpu Main {
          fn classify(value: i64) -> i64 {
            loop {
              if value < 0 {
                continue;
              } else if value == 0 {
                break;
              } else {
                return 7;
              }
            }
            return 3;
          }

          fn main() -> i64 {
            return classify(0) + classify(1);
          }
        }
        "#,
    )
    .unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "compile",
            &project.display().to_string(),
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

    let binary = output_dir.join("mixed_loop_terminal");
    let run = Command::new(&binary).output().expect("run native binary");
    assert_eq!(run.status.code(), Some(10));
    fs::remove_dir_all(project).unwrap();
}
