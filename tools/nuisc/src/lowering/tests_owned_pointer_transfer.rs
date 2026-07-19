use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_exact_one_owned_pointer_transfer_across_selected_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume_left(bytes: Bytes, head: ref Node) -> Bytes {
            let observed: i64 = load_value(head);
            free(head);
            return move(bytes);
          }

          fn consume_right(bytes: Bytes, head: ref Node) -> Bytes {
            let observed: i64 = load_value(head);
            free(head);
            return move(bytes);
          }

          fn choose(bytes: Bytes, selector: i64) -> Bytes {
            let nil: ref Node? = null();
            let head: ref Node = alloc_node(17, nil);
            if selector == 1 {
              return consume_left(move(bytes), move(head));
            } else {
              return consume_right(move(bytes), move(head));
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(3, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let selector: i64 = cpu_input_i64("selector", 1, 1, 2, 1);
            let selected: Bytes = choose(move(bytes), selector);
            let len: i64 = bytes_len(selected);
            drop_bytes(selected);
            free(buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("exact-one transfer should lower");
    let tree = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_tree")
        .expect("selected owned transfer tree");
    assert_eq!(
        tree.op
            .args
            .iter()
            .filter(|arg| arg.as_str() == "owned_transfer")
            .count(),
        2
    );
    let profile = yir_core::glm_profile_for_operation(&tree.op);
    assert_eq!(
        profile
            .accesses
            .iter()
            .filter(|access| access.mode == yir_core::GlmUseMode::Own)
            .count(),
        2,
        "one Bytes owner and one Node transfer must be owned"
    );

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned transfer LLVM lowering");
    assert!(llvm_ir.contains("call ptr @nuis_fn_consume_left(ptr"));
    assert!(llvm_ir.contains("call ptr @nuis_fn_consume_right(ptr"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
}

#[test]
fn rejects_asymmetric_owned_pointer_transfer_across_selected_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume(bytes: Bytes, head: ref Node) -> Bytes {
            free(head);
            return move(bytes);
          }

          fn inspect(bytes: Bytes, head: ref Node) -> Bytes {
            let observed: i64 = load_value(head);
            return move(bytes);
          }

          fn choose(bytes: Bytes, selector: bool) -> Bytes {
            let head: ref Node = alloc_node(17, null());
            if selector {
              return consume(move(bytes), move(head));
            } else {
              return inspect(move(bytes), borrow(head));
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(1, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let selected: Bytes = choose(move(bytes), true);
            drop_bytes(selected);
            free(buffer);
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("same moved Node set on every reachable leaf"),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn rejects_owned_pointer_transfer_to_helper_that_does_not_consume_it() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn leak(bytes: Bytes, head: ref Node, release: bool) -> Bytes {
            if release {
              free(head);
            } else {
              let observed: i64 = load_value(head);
            }
            return move(bytes);
          }

          fn choose(bytes: Bytes, selector: bool) -> Bytes {
            let head: ref Node = alloc_node(17, null());
            if selector {
              return leak(move(bytes), move(head), true);
            } else {
              return leak(move(bytes), move(head), false);
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(1, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let selected: Bytes = choose(move(bytes), true);
            drop_bytes(selected);
            free(buffer);
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("exactly one free(...) on every exit path"),
        "unexpected diagnostic: {error}"
    );
}

#[test]
fn accepts_branch_complete_owned_pointer_consumption_in_selected_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume(bytes: Bytes, head: ref Node, release_left: bool) -> Bytes {
            if release_left {
              free(head);
            } else {
              free(head);
            }
            return move(bytes);
          }

          fn choose(bytes: Bytes, selector: bool) -> Bytes {
            let head: ref Node = alloc_node(17, null());
            if selector {
              return consume(move(bytes), move(head), true);
            } else {
              return consume(move(bytes), move(head), false);
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(1, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let selected: Bytes = choose(move(bytes), true);
            drop_bytes(selected);
            free(buffer);
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module)
        .expect("branch-complete owned pointer consumption should lower");
    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("branch consumer LLVM lowering");
    assert!(llvm_ir.contains("define ptr @nuis_fn_consume(ptr %arg0, ptr %arg1, i1 %arg2)"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
}
