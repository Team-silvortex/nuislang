use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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

    let mut child = Command::new(output_dir.join(project_name))
        .spawn()
        .expect("run native binary");
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("poll native binary") {
            return status;
        }
        if started.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("{project_name}: native binary exceeded its test deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn invariant_add_and_multiply_carries_run_as_a_native_binary() {
    let status = compile_and_run(
        "invariant_carries",
        "mod cpu Main {
      @noinline fn walk(limit: i64) -> i64 {
        let index: i64 = 0; let total: i64 = 2; let product: i64 = 2;
        while index < limit {
          let index: i64 = index + 1;
          let total: i64 = total + 7;
          let product: i64 = product * 2;
        }
        return total + product;
      }
      fn main() -> i64 { return walk(3) + walk(0); }
    }",
    );
    assert_eq!(status.code(), Some(43));
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
fn aggregate_state_runs_across_a_scoped_call_loop_backedge() {
    let status = compile_and_run(
        "aggregate_loop_backedge",
        r#"
        mod cpu Main {
          struct Pair {
            left: i64,
            right: i64
          }

          struct State {
            ready: bool,
            count: i64,
            pair: Pair
          }

          fn advance(state: State, iteration: i64) -> State {
            return State {
              ready: state.ready,
              count: state.count + 1,
              pair: Pair {
                left: state.pair.left + iteration + 1,
                right: state.pair.right + 1
              }
            };
          }

          fn main() -> i64 {
            let state: State = State {
              ready: true,
              count: 0,
              pair: Pair { left: 1, right: 2 }
            };
            let iteration: i64 = 0;
            while iteration < 3 {
              let state: State = advance(state, iteration);
              let iteration: i64 = iteration + 1;
            }
            return state.count + state.pair.left + state.pair.right;
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(15));
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

#[test]
fn nested_carry_scoped_iteration_preserves_native_entry_result() {
    let status = compile_and_run(
        "nested_carry_entry",
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value > 3 {
                let acc: i64 = acc + value;
              } else {
                if value > 1 {
                  let acc: i64 = acc + value;
                } else {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(14));
}

#[test]
fn callable_main_wrapper_avoids_user_helper_collisions_natively() {
    let source = r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value > 3 {
                let acc: i64 = acc + value;
              } else {
                if value > 1 {
                  let acc: i64 = acc + value;
                } else {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#
    .replace(
        "fn main()",
        "@noinline fn __nuis_entry_main_0() -> i64 { return 3; }
@noinline fn __nuis_entry_main_1() -> i64 { return 5; }
fn main()",
    )
    .replace(
        "return acc;",
        "return acc + __nuis_entry_main_0() + __nuis_entry_main_1();",
    )
    .replace(
        "let acc: i64 = acc + value;",
        "let acc: i64 = acc + value; let acc: i64 = acc + 1;",
    );
    let status = compile_and_run("sequence_main_collision", &source);
    assert_eq!(status.code(), Some(26));
}

#[test]
fn iteration_temporaries_preserve_boolean_snapshots_in_native_entry() {
    let status = compile_and_run(
        "temporary_snapshot_entry",
        r#"mod cpu Main {
          @noinline fn calculate(limit: i64) -> i64 {
            let index: i64 = 0;
            let total: i64 = 0;
            while index < limit {
              let index: i64 = index + 1;
              let delta = index * 2;
              let selected = delta < 5 || index == 4;
              let delta: i64 = delta + 100;
              if selected {
                let local = delta - 100;
                let total: i64 = total + local;
              } else {
                let local = 1;
                let total: i64 = total + local;
              }
            }
            return total;
          }
          fn main() -> i64 { return calculate(4); }
        }"#,
    );
    assert_eq!(status.code(), Some(15));
}

#[test]
fn temporary_only_iteration_has_no_extra_native_carry() {
    let status = compile_and_run(
        "temporary_only_entry",
        r#"mod cpu Main {
          @noinline fn calculate(limit: i64) -> i64 {
            let index: i64 = 0;
            while index < limit {
              let index: i64 = index + 1;
              let scratch = index * 2;
              let selected = scratch < limit || index == limit;
              if selected { let local = scratch + 3; }
            }
            return index;
          }
          fn main() -> i64 { return calculate(4); }
        }"#,
    );
    assert_eq!(status.code(), Some(4));
}

#[test]
fn inline_bool_match_temporary_preserves_native_entry_result() {
    let status = compile_and_run(
        "inline_match_temporary",
        r#"
        mod cpu Main {
          @inline
          fn hot(value: i64) -> bool {
            return value > 2;
          }

          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              match hot(value) {
                true => {
                  let acc: i64 = acc + value;
                },
                _ => {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#,
    );
    assert_eq!(status.code(), Some(12));
}

#[test]
fn checked_iteration_division_and_remainder_run_through_default_native_entry() {
    let status = compile_and_run(
        "checked_iteration_entry",
        r#"mod cpu Main {
          @noinline fn calculate(limit: i64, divisor: i64) -> i64 {
            let index: i64 = 0;
            let total: i64 = 0;
            while index < limit {
              let index: i64 = index + 1;
              let valid = divisor != 0 && index / divisor >= 0;
              if valid {
                let quotient = index / divisor;
                let saved: i64 = quotient;
                let quotient: i64 = quotient + 1;
                let remainder = index % divisor;
                let total: i64 = total + saved * divisor + remainder;
              } else { let total: i64 = total + 1; }
            }
            return total;
          }
          fn main() -> i64 { return calculate(4, 2) + calculate(4, 0) + calculate(0, 0); }
        }"#,
    );
    assert_eq!(status.code(), Some(14));
}

#[test]
fn unused_checked_iteration_still_traps_without_any_carried_result() {
    for op in ["/", "%"] {
        let source = format!(
            "mod cpu Main {{
          @noinline fn calculate(limit: i64, divisor: i64) -> i64 {{
            let index: i64 = 0;
            while index < limit {{
              let index: i64 = index + 1;
              let unused = index {op} divisor;
            }}
            return index;
          }}
          fn main() -> i64 {{ return calculate(2, 0); }}
        }}"
        );
        let status = compile_and_run("unused_checked_iteration", &source);
        assert!(!status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}

#[test]
fn iteration_helper_calls_keep_boolean_arguments_lazy_in_native_entry() {
    let status = compile_and_run(
        "iteration_calls_entry",
        r#"mod cpu Main {
      @noinline fn quotient(value: i64, divisor: i64) -> i64 { return value / divisor; }
      @noinline fn flag(value: bool) -> bool { return value; }
      @noinline fn add(left: i64, right: i64) -> i64 { return left + right; }
      @noinline fn choose(enabled: bool, yes: i64, no: i64) -> i64 { if enabled { return yes; } return no; }
      @noinline fn calculate(limit: i64, divisor: i64) -> i64 {
        let index: i64 = 0;
        let total: i64 = 0;
        while index < limit {
          let index: i64 = index + 1;
          let ready = flag(divisor != 0);
          let weight = choose(ready && quotient(index, divisor) > 0, index, 1);
          let selected = flag(ready) == true;
          if selected || divisor == 0 { let total: i64 = add(total, weight); }
        }
        return total;
      }
      fn main() -> i64 { return calculate(4, 2) + calculate(4, 0) + calculate(0, 0); }
    }"#,
    );
    assert_eq!(status.code(), Some(14));
}

#[test]
fn ignored_iteration_call_arguments_are_evaluated_before_the_call() {
    for op in ["/", "%"] {
        let source = format!(
            "mod cpu Main {{
          @noinline fn checked(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }}
          @noinline fn ignore(value: i64) -> i64 {{ return 0; }}
          @noinline fn calculate(limit: i64, divisor: i64) -> i64 {{
            let index: i64 = 0;
            while index < limit {{
              let index: i64 = index + 1;
              let unused = ignore(checked(index, divisor));
            }}
            return index;
          }}
          fn main() -> i64 {{ return calculate(2, 0); }}
        }}"
        );
        let status = compile_and_run("ignored_iteration_argument", &source);
        assert!(!status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}

#[test]
fn deep_source_calls_report_a_parser_error_instead_of_aborting_the_compiler() {
    let project = temp_dir("deep_source_calls");
    fs::write(project.0.join("nuis.toml"), "name = \"deep_source_calls\"\nversion = \"0.1.0\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n").unwrap();
    fs::write(project.0.join("main.ns"), format!("mod cpu Main {{ fn identity(value: i64) -> i64 {{ return value; }} fn main() -> i64 {{ return {}1{}; }} }}", "identity(".repeat(4096), ")".repeat(4096))).unwrap();
    let compile = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .arg("compile")
        .arg(&project.0)
        .arg(project.0.join("out"))
        .output()
        .unwrap();
    assert!(!compile.status.success());
    assert!(compile.status.code().is_some(), "{:?}", compile.status);
    assert!(
        String::from_utf8_lossy(&compile.stderr)
            .contains("source expression nesting exceeds parser limit"),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

#[test]
fn flat_iteration_values_compose_with_default_native_entry() {
    let status = compile_and_run(
        "flat_iteration_values",
        r#"mod cpu Main {
      struct Parts { quotient: i64, remainder: i64 }
      @noinline fn split(value: i64, divisor: i64) -> Parts {
        return Parts { remainder: value % divisor, quotient: value / divisor };
      }
      @noinline fn restore(parts: Parts, divisor: i64) -> i64 {
        return parts.quotient * divisor + parts.remainder;
      }
      @noinline fn calculate(limit: i64, divisor: i64) -> i64 {
        let index: i64 = 0;
        let total: i64 = 0;
        while index < limit {
          let index: i64 = index + 1;
          let valid = divisor != 0 && split(index, divisor).quotient >= 0;
          if valid {
            let parts = split(index, divisor);
            let snapshot: Parts = parts;
            let total: i64 = total + restore(snapshot, divisor);
          } else { let total: i64 = total + 1; }
        }
        return total;
      }
      fn main() -> i64 { return calculate(4, 2) + calculate(4, 0) + calculate(0, 0); }
    }"#,
    );
    assert_eq!(status.code(), Some(14));
}

#[test]
fn discarded_flat_iteration_result_keeps_native_arithmetic_failure() {
    for op in ["/", "%"] {
        let source = format!(
            "mod cpu Main {{
          struct Packet {{ value: i64 }}
          @noinline fn packet(value: i64, divisor: i64) -> Packet {{
            return Packet {{ value: value {op} divisor }};
          }}
          @noinline fn calculate(limit: i64, divisor: i64) -> i64 {{
            let index: i64 = 0;
            while index < limit {{
              let index: i64 = index + 1;
              let unused = packet(index, divisor);
            }}
            return index;
          }}
          fn main() -> i64 {{ return calculate(2, 0); }}
        }}"
        );
        let status = compile_and_run("discarded_flat_iteration", &source);
        assert!(!status.success());
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}

const LOOP_CALL_SOURCE: &str = r#"mod cpu Main {
  struct Packet { value: i64 }
  @noinline fn packet(value: i64, stride: i64) -> Packet {
    let index: i64 = 0;
    let total: i64 = value;
    while index < 2 {
      let index: i64 = index + stride;
      let total: i64 = total + index;
    }
    return Packet { value: total };
  }
  @noinline fn calculate(limit: i64, stride: i64, enabled: bool) -> i64 {
    let index: i64 = 0;
    let total: i64 = 0;
    while index < limit {
      let index: i64 = index + 1;
      if enabled {
        let value = packet(index, stride);
        let saved: Packet = value;
        let total: i64 = total + saved.value;
      } else { let total: i64 = total + 1; }
    }
    return total;
  }
  fn main() -> i64 {
    return calculate(4, 1, true) + calculate(2, 0, false) + calculate(0, 0, true);
  }
}"#;

#[test]
fn loop_bearing_iteration_helpers_run_through_default_native_entry() {
    let status = compile_and_run("loop_bearing_iteration_helpers", LOOP_CALL_SOURCE);
    assert_eq!(status.code(), Some(24));
}

#[test]
fn discarded_loop_helper_keeps_its_checked_body_failure() {
    // Ordinary native entry allows general loops; bounded induction preflight
    // is the native-session profile's contract, not the CLI's loop semantics.
    let source = LOOP_CALL_SOURCE
        .replace(
            "let total: i64 = total + index;",
            "let total: i64 = total + index / (2 - index);",
        )
        .replace(
            "let total: i64 = total + saved.value;",
            "let total: i64 = total + 1;",
        );
    let status = compile_and_run("discarded_loop_helper", &source);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}
#[path = "control_flow_syntax_native/aggregate_carries.rs"]
mod aggregate_carries;
#[path = "control_flow_syntax_native/aggregate_rebinding.rs"]
mod aggregate_rebinding;
#[path = "control_flow_syntax_native/bool_rebinding.rs"]
mod bool_rebinding;
