use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_nested_owned_survivor_tree_with_unique_owner_consumption() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose(lhs: Bytes, rhs: Bytes, outer: bool, inner: bool) -> Bytes {
            if outer {
              if inner {
                return move(lhs);
              } else {
                return move(rhs);
              }
            } else {
              return move(lhs);
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(2, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(3, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let outer: i64 = cpu_input_i64("outer", 1, 0, 1, 1);
            let inner: i64 = cpu_input_i64("inner", 0, 0, 1, 1);
            let selected: Bytes = choose(move(lhs), move(rhs), outer == 1, inner == 1);
            let len: i64 = bytes_len(selected);
            drop_bytes(selected);
            free(rhs_buffer);
            free(lhs_buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("nested owner tree should lower");
    let node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_tree")
        .expect("nested owned select tree node");
    let args =
        yir_core::parse_owned_select_tree_args(&node.op.args).expect("owned select tree protocol");
    assert_eq!(args.owners.len(), 2);
    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert_eq!(
        profile
            .accesses
            .iter()
            .filter(|access| access.mode == yir_core::GlmUseMode::Own)
            .count(),
        2,
        "each unique owner must be consumed once regardless of leaf aliases"
    );

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("nested owner tree LLVM lowering");
    assert!(llvm_ir.contains("select_owned_tree_then"));
    assert!(llvm_ir.contains("select_owned_tree_else"));
    assert!(llvm_ir.contains("select_owned_tree_merge"));
    assert!(llvm_ir.contains(" = phi ptr ["));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        2
    );
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        4
    );
}

#[test]
fn lowers_nested_owned_tree_with_static_helper_call_leaves() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep_left(bytes: Bytes, delta: i64) -> Bytes {
            return move(bytes);
          }

          fn keep_right(bytes: Bytes, factor: i64) -> Bytes {
            return move(bytes);
          }

          fn choose(lhs: Bytes, rhs: Bytes, outer: bool, inner: bool) -> Bytes {
            if outer {
              if inner {
                return keep_left(move(lhs), 3);
              } else {
                return move(rhs);
              }
            } else {
              return keep_right(move(lhs), 7);
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(2, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(3, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let outer: i64 = cpu_input_i64("outer", 1, 0, 1, 1);
            let inner: i64 = cpu_input_i64("inner", 0, 0, 1, 1);
            let selected: Bytes = choose(move(lhs), move(rhs), outer == 1, inner == 1);
            let len: i64 = bytes_len(selected);
            drop_bytes(selected);
            free(rhs_buffer);
            free(lhs_buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("nested helper tree should lower");
    let node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_tree")
        .expect("nested owned helper tree node");
    let args = yir_core::parse_owned_select_tree_args(&node.op.args).expect("tree protocol");
    assert_eq!(args.owners.len(), 2);
    assert!(node
        .op
        .args
        .windows(2)
        .any(|entry| entry == ["call", "keep_left"]));
    assert!(node
        .op
        .args
        .windows(2)
        .any(|entry| entry == ["call", "keep_right"]));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:keep_left"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:keep_right"));
    let scalar_dependencies = yir
        .edges
        .iter()
        .filter(|edge| edge.to == node.name && edge.kind == yir_core::EdgeKind::Dep)
        .filter_map(|edge| {
            yir.nodes
                .iter()
                .find(|source| source.name == edge.from)
                .filter(|source| source.op.instruction == "const_i64")
        })
        .count();
    assert_eq!(scalar_dependencies, 2);

    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert_eq!(
        profile
            .accesses
            .iter()
            .filter(|access| access.mode == yir_core::GlmUseMode::Own)
            .count(),
        2
    );

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("nested helper LLVM lowering");
    assert!(llvm_ir.contains("call ptr @nuis_fn_keep_left(ptr"));
    assert!(llvm_ir.contains("call ptr @nuis_fn_keep_right(ptr"));
    assert!(llvm_ir.contains(" = phi ptr ["));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        4
    );
}
