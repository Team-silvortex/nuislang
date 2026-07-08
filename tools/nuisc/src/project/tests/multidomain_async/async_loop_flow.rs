use super::*;

#[test]
fn compiles_async_loop_flow_chain_project() {
    let root = write_temp_project(
        "async_loop_flow_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 2 {
                break;
              }
              let acc: i64 = acc + value;
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_cond_chain_project() {
    let root = write_temp_project(
        "async_loop_cond_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_cond_chain_project() {
    let root = write_temp_project(
        "async_loop_flow_cond_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = await step(value);
              if value > 3 {
                continue;
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_cond_chain_compound_control_project() {
    let root = write_temp_project(
        "async_loop_flow_cond_chain_compound_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 1 {
                if value > 4 {
                  break;
                } else {
                }
              } else {
              }
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_flow_cond_chain_recursive_control_project() {
    let root = write_temp_project(
        "async_loop_flow_cond_chain_recursive_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 1 && value > 3 && value < 6 {
                break;
              } else {
              }
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_post_flow_cond_chain_project() {
    let root = write_temp_project(
        "async_loop_post_flow_cond_chain",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 5 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_post_flow_cond_chain_compound_control_project() {
    let root = write_temp_project(
        "async_loop_post_flow_cond_chain_compound_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 6 {
              let value: i64 = await step(value);
              if value > 2 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              match acc {
                5 => { continue; },
                _ => {
                  if acc < 6 {
                    continue;
                  } else {
                  }
                }
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}

#[test]
fn compiles_async_loop_post_flow_cond_chain_recursive_control_project() {
    let root = write_temp_project(
        "async_loop_post_flow_cond_chain_recursive_control",
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              if value > 4 {
                let acc: i64 = acc + value;
              } else {
                let acc: i64 = acc + 0;
              }
              if acc > 1 && acc > 3 && acc < 10 {
                continue;
              } else {
              }
            }
            return acc;
          }
        }
        "#,
        multidomain_support_modules(),
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu"
            && node.op.instruction == "loop_while_scalar_async_post_flow_cond_chain"));
    assert!(artifacts.llvm_ir.contains("@nuis_fn_step"));
}
