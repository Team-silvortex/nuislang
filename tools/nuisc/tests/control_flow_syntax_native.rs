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

fn temp_dir(project_name: &str) -> TempProject {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuisc_{project_name}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    TempProject(dir)
}

fn compile_and_run(project_name: &str, source: &str) -> std::process::ExitStatus {
    let project = temp_dir(project_name);
    let output_dir = project.0.join("out");
    fs::write(
        project.0.join("nuis.toml"),
        format!(
            "name = \"{project_name}\"\nversion = \"0.1.0\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n"
        ),
    )
    .unwrap();
    fs::write(project.0.join("main.ns"), source).unwrap();

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

    Command::new(output_dir.join(project_name))
        .status()
        .expect("run native binary")
}

#[test]
fn mixed_loop_terminal_tree_runs_as_a_native_binary() {
    let status = compile_and_run(
        "mixed_loop_terminal",
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
    );
    assert_eq!(status.code(), Some(10));
}

#[test]
fn state_carrying_unbounded_loop_runs_as_a_native_binary() {
    let status = compile_and_run(
        "state_carrying_unbounded_loop",
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            loop {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              if acc >= 6 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(6));
}

#[test]
fn invariant_while_let_payload_runs_as_a_native_binary() {
    let status = compile_and_run(
        "invariant_while_let_payload",
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn main() -> i64 {
            let selected: Option = Option.Some(2);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > payload {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(6));
}

#[test]
fn invariant_while_let_mismatch_skips_the_native_loop() {
    let status = compile_and_run(
        "invariant_while_let_mismatch",
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn main() -> i64 {
            let selected: Option = Option.None;
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > 2 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(0));
}

#[test]
fn invariant_while_let_accepts_runtime_enum_arguments() {
    let status = compile_and_run(
        "invariant_while_let_runtime_enum",
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn consume(selected: Option) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > payload {
                break;
              }
            }
            return acc;
          }

          fn main() -> i64 {
            return consume(Option.Some(2)) + consume(Option.None);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(6));
}

#[test]
fn terminal_while_let_variant_transition_runs_as_a_native_binary() {
    let status = compile_and_run(
        "terminal_while_let_variant_transition",
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn consume(selected: Phase) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              let selected: Phase = Phase.Done;
              if cursor > payload {
                break;
              }
            }
            match selected {
              Phase.Done => {
                return acc + 10;
              },
              Phase.Active(payload) => {
                return payload + 100;
              },
            }
          }

          fn main() -> i64 {
            return consume(Phase.Active(2)) + consume(Phase.Done);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(21));
}

#[test]
fn dynamic_while_let_variant_state_runs_across_native_backedges() {
    let status = compile_and_run(
        "dynamic_while_let_variant_state",
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn consume(selected: Phase) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              if payload > 1 {
                let selected: Phase = Phase.Active(payload - 1);
              } else {
                let selected: Phase = Phase.Done;
              }
              if cursor > 100 {
                break;
              }
            }
            match selected {
              Phase.Done => {
                return acc + 10;
              },
              Phase.Active(payload) => {
                return acc + payload + 100;
              },
            }
          }

          fn main() -> i64 {
            return consume(Phase.Active(3)) + consume(Phase.Done);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(26));
}

#[test]
fn dynamic_while_let_flow_control_reads_the_previous_payload() {
    let status = compile_and_run(
        "dynamic_while_let_previous_payload_flow",
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn consume(selected: Phase) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              if payload > 1 {
                let selected: Phase = Phase.Active(payload - 1);
              } else {
                let selected: Phase = Phase.Done;
              }
              if payload == 3 {
                continue;
              } else if payload == 2 {
                break;
              }
            }
            match selected {
              Phase.Done => {
                return acc + 40;
              },
              Phase.Active(payload) => {
                return acc + payload;
              },
            }
          }

          fn main() -> i64 {
            return consume(Phase.Active(4)) + consume(Phase.Done);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(50));
}

#[test]
fn dynamic_while_let_carries_ordered_multi_field_payloads() {
    let status = compile_and_run(
        "dynamic_while_let_multi_field_payloads",
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active {
              value: i64,
              step: i64,
            },
          }

          fn consume(selected: Phase) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active { value: payload, step: stride } = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              if payload < 6 {
                let selected: Phase = Phase.Active {
                  step: stride + 1,
                  value: payload + stride,
                };
              } else {
                let selected: Phase = Phase.Done;
              }
              if cursor > 100 {
                break;
              }
            }
            match selected {
              Phase.Done => {
                return acc + 10;
              }
              _ => {
                return -1;
              }
            }
          }

          fn main() -> i64 {
            return consume(Phase.Active { value: 1, step: 1 })
              + consume(Phase.Done);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(34));
}

#[test]
fn dynamic_while_let_preserves_bool_payloads_across_native_backedges() {
    let status = compile_and_run(
        "dynamic_while_let_bool_payload",
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active { ready: bool },
          }

          fn consume(selected: Phase) -> i64 {
            let cursor: i64 = 0;
            while let Phase.Active { ready: flag } = selected {
              let cursor: i64 = cursor + 1;
              let selected: Phase = Phase.Active { ready: flag };
              if cursor > 1 {
                break;
              }
            }
            match selected {
              Phase.Active { ready: flag } => {
                if flag {
                  return cursor + 10;
                }
                return cursor + 20;
              }
              Phase.Done => {
                return 30;
              }
            }
          }

          fn main() -> i64 {
            return consume(Phase.Active { ready: true })
              + consume(Phase.Active { ready: false })
              + consume(Phase.Done);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(64));
}

#[test]
fn dynamic_while_let_bool_payload_drives_native_replacement() {
    let status = compile_and_run(
        "dynamic_while_let_bool_replacement",
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active { ready: bool },
          }

          fn consume(selected: Phase) -> i64 {
            let cursor: i64 = 0;
            while let Phase.Active { ready: flag } = selected {
              let cursor: i64 = cursor + 1;
              if flag {
                let selected: Phase = Phase.Active { ready: false };
              } else {
                let selected: Phase = Phase.Done;
              }
              if cursor > 4 {
                break;
              }
            }
            match selected {
              Phase.Done => {
                return cursor + 10;
              }
              _ => {
                return 90;
              }
            }
          }

          fn main() -> i64 {
            return consume(Phase.Active { ready: true })
              + consume(Phase.Active { ready: false })
              + consume(Phase.Done);
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(33));
}
