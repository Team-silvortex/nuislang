use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

fn stmt_drops_owned_name(stmt: &nuis_semantics::model::NirStmt, expected: &str) -> bool {
    matches!(
        stmt,
        nuis_semantics::model::NirStmt::Expr(nuis_semantics::model::NirExpr::DropBytes(inner))
            if matches!(inner.as_ref(), nuis_semantics::model::NirExpr::Var(name) if name == expected)
    )
}

#[test]
fn nested_loop_cleanup_keeps_outer_owner_until_outer_break() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            while true {
              let outer_iteration: Bytes = copy_bytes(buffer);
              while true {
                let inner_iteration: Bytes = copy_bytes(buffer);
                continue;
              }
              break;
            }
            free(buffer);
          }
        }
        "#,
    )
    .unwrap();
    assert!(crate::owned_cleanup::insert_owned_bytes_cleanup(
        &mut module
    ));
    let outer = module.functions[0]
        .body
        .iter()
        .find_map(|stmt| match stmt {
            nuis_semantics::model::NirStmt::While { body, .. } => Some(body),
            _ => None,
        })
        .expect("expected outer while");
    let inner = outer
        .iter()
        .find_map(|stmt| match stmt {
            nuis_semantics::model::NirStmt::While { body, .. } => Some(body),
            _ => None,
        })
        .expect("expected inner while");
    assert!(inner
        .get(1)
        .is_some_and(|stmt| stmt_drops_owned_name(stmt, "inner_iteration")));
    assert!(matches!(
        inner.get(2),
        Some(nuis_semantics::model::NirStmt::Continue)
    ));
    assert!(outer
        .get(2)
        .is_some_and(|stmt| stmt_drops_owned_name(stmt, "outer_iteration")));
    assert!(matches!(
        outer.get(3),
        Some(nuis_semantics::model::NirStmt::Break)
    ));
    crate::nir_verify::verify_nir_module(&module)
        .expect("nested loop cleanup should preserve GLM ownership state");
}

#[test]
fn lowers_scoped_helper_with_inner_owned_loop_into_outer_loop_call() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn scoped_work(buffer: ref Buffer, seed: i64) -> i64 {
            let inner: i64 = 0;
            while inner < 2 {
              let iteration: Bytes = copy_bytes(buffer);
              drop_bytes(iteration);
              let inner: i64 = inner + 1;
            }
            return seed + inner;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let outer: i64 = 0;
            while outer < 3 {
              scoped_work(buffer, outer);
              let outer: i64 = outer + 1;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let outer = yir
        .nodes
        .iter()
        .find(|node| node.name.starts_with("loop_while_i64_scoped_call"))
        .expect("expected outer scoped-call loop");
    assert_eq!(
        &outer.op.args[5..9],
        ["cpu", "scoped_call", "3", "scoped_work"]
    );
    assert_eq!(outer.op.args[10], "$current");
    let captured_buffer = &outer.op.args[9];
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == yir_core::EdgeKind::Lifetime
            && edge.from == *captured_buffer
            && edge.to == outer.name
    }));
    let inner = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.instruction == "loop_while_i64_effect"
                && yir.node_lanes.get(&node.name).map(String::as_str) == Some("fn:scoped_work")
        })
        .expect("expected owned inner loop in scoped helper lane");
    assert_eq!(&inner.op.args[5..8], ["cpu", "owned_bytes_copy_drop", "1"]);

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("scoped nested loop should lower");
    assert!(
        llvm_ir.contains("define i64 @nuis_fn_scoped_work(ptr %arg0, i64 %arg0_len, i64 %arg1)")
    );
    assert!(llvm_ir.contains("call i64 @nuis_fn_scoped_work(ptr %"));
    assert!(llvm_ir.contains("call ptr @nuis_scheduler_owned_blob_copy_v1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect"));
}

#[test]
fn lowers_scoped_owned_bytes_copy_capture_with_helper_cleanup() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_copy(bytes: Bytes, seed: i64) -> i64 {
            let len: i64 = bytes_len(bytes);
            return seed + len;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let outer: i64 = 0;
            while outer < 3 {
              consume_copy(copy_bytes(buffer), outer);
              let outer: i64 = outer + 1;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();
    assert!(crate::owned_cleanup::insert_owned_bytes_cleanup(
        &mut module
    ));
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let outer = yir
        .nodes
        .iter()
        .find(|node| node.name.starts_with("loop_while_i64_scoped_call"))
        .expect("expected scoped-call loop with copied Bytes capture");
    assert_eq!(
        &outer.op.args[5..9],
        ["cpu", "scoped_call", "3", "consume_copy"]
    );
    let copied = outer.op.args[9]
        .strip_prefix("copy_owned:")
        .expect("owned Bytes capture should declare copy semantics");
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == yir_core::EdgeKind::Dep && edge.from == copied && edge.to == outer.name
    }));
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == yir_core::EdgeKind::Lifetime && edge.from == copied && edge.to == outer.name
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.op.instruction == "param_owned_bytes"
            && yir.node_lanes.get(&node.name).map(String::as_str) == Some("fn:consume_copy")
    }));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned copy capture should lower");
    assert!(llvm_ir.contains("define i64 @nuis_fn_consume_copy(ptr %arg0, i64 %arg1)"));
    assert!(llvm_ir.contains("call ptr @nuis_scheduler_owned_blob_copy_v1"));
    assert!(llvm_ir.contains("call i64 @nuis_fn_consume_copy(ptr %"));
    let helper = llvm_ir
        .split("define i64 @nuis_fn_consume_copy")
        .nth(1)
        .and_then(|tail| tail.split("\n}").next())
        .expect("expected consume_copy helper body");
    assert_eq!(
        helper
            .matches("call void @nuis_scheduler_owned_blob_drop_v1(ptr %arg0)")
            .count(),
        1
    );
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect"));
}

#[test]
fn rejects_implicit_owned_bytes_reuse_in_scoped_loop() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_move(bytes: Bytes, seed: i64) -> i64 {
            drop_bytes(bytes);
            return seed;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let outer: i64 = 0;
            while outer < 2 {
              consume_move(bytes, outer);
              let outer: i64 = outer + 1;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let error = yir_lower_llvm::emit_module(&yir)
        .expect_err("repeated owned Bytes capture must not imply copy or move reuse");
    assert!(
        error.contains("requires explicit copy or move capture"),
        "{error}"
    );
}

#[test]
fn lowers_single_iteration_scoped_owned_bytes_move_capture() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_move(bytes: Bytes, seed: i64) -> i64 {
            let len: i64 = bytes_len(bytes);
            drop_bytes(bytes);
            return seed + len;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let outer: i64 = 0;
            while outer < 1 {
              consume_move(move(bytes), outer);
              let outer: i64 = outer + 1;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let outer = yir
        .nodes
        .iter()
        .find(|node| node.name.starts_with("loop_while_i64_scoped_call"))
        .expect("expected single-shot scoped move loop");
    let moved = outer.op.args[9]
        .strip_prefix("move_owned:")
        .expect("owned Bytes capture should declare move semantics");
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == yir_core::EdgeKind::Dep && edge.from == moved && edge.to == outer.name
    }));
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == yir_core::EdgeKind::Lifetime && edge.from == moved && edge.to == outer.name
    }));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("single-shot move should lower");
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        1,
        "move capture must transfer the existing blob without another copy"
    );
    assert!(llvm_ir.contains("call i64 @nuis_fn_consume_move(ptr %"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect"));
}

#[test]
fn lowers_statically_folded_single_iteration_owned_move_capture() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_move(bytes: Bytes, seed: i64) -> i64 {
            drop_bytes(bytes);
            return seed;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let base: i64 = 0;
            let start: i64 = base + 0;
            let bound: i64 = base + 1;
            let stride: i64 = 1;
            let outer: i64 = start;
            while outer < bound {
              consume_move(move(bytes), outer);
              let outer: i64 = outer + stride;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module)
        .expect("foldable local facts should prove one owned move iteration");
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.name.starts_with("loop_while_i64_scoped_call"))
        .expect("expected statically proven scoped loop");
    assert!(loop_node
        .op
        .args
        .iter()
        .any(|arg| arg.starts_with("move_owned:")));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("folded exact-one loop should lower");
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect"));
}

#[test]
fn rejects_scoped_owned_bytes_move_across_repeating_backedge() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_move(bytes: Bytes, seed: i64) -> i64 {
            drop_bytes(bytes);
            return seed;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let outer: i64 = 0;
            while outer < 2 {
              consume_move(move(bytes), outer);
              let outer: i64 = outer + 1;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let error = lower_nir_to_yir_builtin_cpu(&module)
        .expect_err("owned Bytes move must not cross a repeating backedge");
    assert!(error.contains("can only move owned Bytes"), "{error}");
    assert!(error.contains("execute exactly once"), "{error}");
}

#[test]
fn rejects_scoped_owned_bytes_move_when_loop_may_not_execute() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_move(bytes: Bytes, seed: i64) -> i64 {
            drop_bytes(bytes);
            return seed;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let outer: i64 = 0;
            while outer < 0 {
              consume_move(move(bytes), outer);
              let outer: i64 = outer + 1;
            }
            free(buffer);
            return outer;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module)
        .expect_err("owned Bytes move must have a proven receiver");
    assert!(error.contains("execute exactly once"), "{error}");
}

#[test]
fn rebinds_scoped_owned_bytes_return_across_repeating_backedge() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn relay(bytes: Bytes) -> Bytes {
            return move(bytes);
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let outer: i64 = 0;
            while outer < 3 {
              let bytes: Bytes = relay(move(bytes));
              let outer: i64 = outer + 1;
            }
            let len: i64 = bytes_len(bytes);
            drop_bytes(bytes);
            free(buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    crate::owned_cleanup::insert_owned_bytes_cleanup(&mut module);
    crate::nir_verify::verify_nir_module(&module)
        .expect("owned helper return should re-establish the loop owner");

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let outer = yir
        .nodes
        .iter()
        .find(|node| node.name.starts_with("loop_while_i64_scoped_call"))
        .expect("expected repeating scoped owned-return loop");
    assert_eq!(outer.op.args[6], "scoped_call_owned_return");
    let result_name = &outer.op.args[9];
    assert!(yir.nodes.iter().any(|node| {
        node.name == *result_name
            && node.op.instruction == "loop_owned_result"
            && node.op.args == [outer.name.clone()]
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.op.instruction == "owned_bytes_len" && node.op.args == [result_name.clone()]
    }));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("scoped owned return should lower");
    assert!(llvm_ir.contains("define ptr @nuis_fn_relay(ptr %arg0)"));
    assert!(llvm_ir.contains("call ptr @nuis_fn_relay(ptr %"));
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        1,
        "repeating ownership rebinding must not copy the blob"
    );
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_owned_result"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect"));
}

#[test]
fn lowers_owned_bytes_counted_while_with_continue_as_registered_effect() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            while value < 2 {
              let iteration: Bytes = copy_bytes(buffer);
              drop_bytes(iteration);
              let value: i64 = value + 1;
              continue;
            }
            free(buffer);
            return value;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect")
        .expect("expected resource-aware counted loop");
    assert_eq!(
        &loop_node.op.args[3..8],
        ["lt", "add", "cpu", "owned_bytes_copy_drop", "1"]
    );
}

#[test]
fn lowers_owned_bytes_guarded_break_as_registered_effect_flow() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value >= 2 {
                drop_bytes(iteration);
                break;
              }
              drop_bytes(iteration);
            }
            free(buffer);
            return value;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware flow loop");
    assert_eq!(loop_node.op.args[3], "lt");
    assert_eq!(loop_node.op.args[4], "add");
    assert_eq!(loop_node.op.args[5], "3");
    assert_eq!(loop_node.op.args[6], "current_ge");
    assert_eq!(loop_node.op.args[8], "break");
    assert_eq!(loop_node.op.args[9], "0");
    assert_eq!(
        &loop_node.op.args[10..13],
        ["cpu", "owned_bytes_copy_drop", "1"]
    );
}

#[test]
fn lowers_owned_bytes_compound_continue_with_ordered_carries() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let acc: i64 = 0;
            let weighted: i64 = 0;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value < 2 || value == 2 {
                drop_bytes(iteration);
                continue;
              }
              let acc: i64 = acc + value;
              let weighted: i64 = weighted + (value + acc);
              drop_bytes(iteration);
            }
            free(buffer);
            return weighted;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware compound flow loop");
    assert_eq!(loop_node.op.args[5], "6");
    assert_eq!(loop_node.op.args[6], "or");
    assert_eq!(loop_node.op.args[7], "current_lt");
    assert_eq!(loop_node.op.args[9], "current_eq");
    assert_eq!(loop_node.op.args[11], "continue");
    assert_eq!(loop_node.op.args[12], "2");
    assert_eq!(loop_node.op.args[14], "add_current");
    assert_eq!(loop_node.op.args[16], "add_current_plus_carry0");
}

#[test]
fn lowers_owned_bytes_mixed_break_continue_flow_tree() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let score: i64 = 0;
            while value < 5 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value > 4 {
                drop_bytes(iteration);
                break;
              } else {
                if value < 3 {
                  drop_bytes(iteration);
                  continue;
                }
              }
              let score: i64 = score + value;
              drop_bytes(iteration);
            }
            free(buffer);
            return score;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected mixed-action resource-aware flow loop");
    assert!(loop_node.op.args.iter().any(|arg| arg == "flow_break"));
    assert!(loop_node.op.args.iter().any(|arg| arg == "flow_continue"));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("mixed-action YIR should lower to LLVM");
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect_flow"));
    assert!(llvm_ir.matches("loop_effect_flow_action.").count() >= 2);
}

#[test]
fn lowers_owned_bytes_flow_with_affine_multiplicative_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let scaled: i64 = 1;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value < 3 {
                drop_bytes(iteration);
                continue;
              }
              let scaled: i64 = scaled * (value + 1);
              drop_bytes(iteration);
            }
            free(buffer);
            return scaled;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware flow loop with affine multiplicative carry");
    assert_eq!(loop_node.op.args[9], "1");
    assert_eq!(loop_node.op.args[11], "mul_current_plus_invariant");
    assert_eq!(loop_node.op.args[13], "cpu");
    assert_eq!(loop_node.op.args[14], "owned_bytes_copy_drop");
}

#[test]
fn lowers_owned_bytes_flow_with_scaled_factor_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let scaled: i64 = 1;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value < 3 {
                drop_bytes(iteration);
                continue;
              }
              let scaled: i64 = scaled * ((value + 1) * 2);
              drop_bytes(iteration);
            }
            free(buffer);
            return scaled;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware flow loop with scaled carry");
    assert_eq!(loop_node.op.args[9], "1");
    assert_eq!(loop_node.op.args[11], "mul_scaled_current_plus_invariant");
    assert_eq!(loop_node.op.args[14], "cpu");
    assert_eq!(loop_node.op.args[15], "owned_bytes_copy_drop");
}

#[test]
fn lowers_owned_bytes_flow_with_updated_carry_as_scaled_factor() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let score: i64 = 0;
            let scaled: i64 = 1;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value < 3 {
                drop_bytes(iteration);
                continue;
              }
              let score: i64 = score + value;
              let scaled: i64 = scaled * ((value + 1) * score);
              drop_bytes(iteration);
            }
            free(buffer);
            return scaled;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware flow loop with state-scaled carry");
    assert_eq!(loop_node.op.args[9], "2");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(
        loop_node.op.args[13],
        "mul_scaled_by_carry0_current_plus_invariant"
    );
    assert_eq!(loop_node.op.args[15], "cpu");
    assert_eq!(loop_node.op.args[16], "owned_bytes_copy_drop");
}

#[test]
fn lowers_owned_bytes_flow_with_factor_group_delta_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let score: i64 = 0;
            let grouped: i64 = 0;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value < 3 {
                drop_bytes(iteration);
                continue;
              }
              let score: i64 = score + value;
              let grouped: i64 = grouped + ((value + score) * ((value + 1) * (score + 1)));
              drop_bytes(iteration);
            }
            free(buffer);
            return grouped;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware flow loop with factor-group carry");
    assert_eq!(loop_node.op.args[9], "2");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(
        loop_node.op.args[13],
        "add_scaled_by_current_plus_factor_invariant_times_factor_group_carry0_plus_factor_invariant_times_terms_current_plus_carry0"
    );
    assert_eq!(loop_node.op.args[16], "cpu");
    assert_eq!(loop_node.op.args[17], "owned_bytes_copy_drop");

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("factor-group YIR should lower to LLVM");
    assert!(!llvm_ir.contains("deferred lowering for cpu.loop_while_i64_effect_flow"));
}

#[test]
fn lowers_owned_bytes_mid_body_continue_with_scalar_carry() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let value: i64 = 0;
            let acc: i64 = 0;
            let weighted: i64 = 0;
            while value < 4 {
              let iteration: Bytes = copy_bytes(buffer);
              let value: i64 = value + 1;
              if value < 3 {
                drop_bytes(iteration);
                continue;
              }
              let acc: i64 = acc + value;
              let weighted: i64 = weighted + acc;
              drop_bytes(iteration);
            }
            free(buffer);
            return weighted;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_i64_effect_flow")
        .expect("expected resource-aware flow loop with carry");
    assert_eq!(loop_node.op.args[8], "continue");
    assert_eq!(loop_node.op.args[9], "2");
    assert_eq!(loop_node.op.args[11], "add_current");
    assert_eq!(loop_node.op.args[13], "add_carry0");
    assert_eq!(
        &loop_node.op.args[14..17],
        ["cpu", "owned_bytes_copy_drop", "1"]
    );
}
