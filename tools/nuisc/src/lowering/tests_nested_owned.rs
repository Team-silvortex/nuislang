use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

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

#[test]
fn normalizes_owned_return_match_into_existing_survivor_tree() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn retain(bytes: Bytes, delta: i64) -> Bytes {
            return move(bytes);
          }

          fn choose(lhs: Bytes, rhs: Bytes, selector: i64) -> Bytes {
            match selector {
              1 => {
                return retain(move(lhs), 3);
              }
              2 => {
                return move(rhs);
              }
              _ => {
                return retain(move(lhs), 7);
              }
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(2, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(3, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let selector: i64 = cpu_input_i64("selector", 2, 1, 3, 1);
            let selected: Bytes = choose(move(lhs), move(rhs), selector);
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("owned match should lower");
    let nodes = yir
        .nodes
        .iter()
        .filter(|node| node.op.instruction == "select_owned_bytes_tree")
        .collect::<Vec<_>>();
    assert_eq!(nodes.len(), 1);
    assert!(!yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "branch_call_owned_bytes"));
    let args = yir_core::parse_owned_select_tree_args(&nodes[0].op.args).expect("tree protocol");
    let mut conditions = Vec::new();
    yir_core::owned_select_tree_conditions(&args.tree, &mut conditions);
    assert_eq!(
        conditions.len(),
        2,
        "three match arms require two decisions"
    );
    assert_eq!(args.owners.len(), 2);
    assert_eq!(
        nodes[0]
            .op
            .args
            .windows(2)
            .filter(|entry| *entry == ["call", "retain"])
            .count(),
        2
    );

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned match LLVM lowering");
    assert!(llvm_ir.contains("select_owned_tree_then"));
    assert!(llvm_ir.contains("select_owned_tree_else"));
    assert_eq!(llvm_ir.matches("call ptr @nuis_fn_retain(ptr").count(), 2);
    assert!(llvm_ir.contains(" = phi ptr ["));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        4
    );
}

#[test]
fn lowers_used_enum_payload_as_selected_leaf_projection() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Route {
            Left(i64),
            Right(i64),
          }

          fn retain(bytes: Bytes, delta: i64) -> Bytes {
            return move(bytes);
          }

          fn choose(lhs: Bytes, rhs: Bytes, selector: i64) -> Bytes {
            let route: Route = if selector == 1 {
              Route.Left(selector)
            } else {
              Route.Right(selector)
            };
            match route {
              Route.Left(payload) => {
                return retain(move(lhs), payload);
              }
              Route.Right(payload) => {
                return move(rhs);
              }
              _ => {
                return retain(move(lhs), 7);
              }
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(2, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(3, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let selector: i64 = cpu_input_i64("selector", 1, 1, 2, 1);
            let selected: Bytes = choose(move(lhs), move(rhs), selector);
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("payload match should lower");
    let tree = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_tree")
        .expect("owned payload match tree");
    let args = yir_core::parse_owned_select_tree_args(&tree.op.args).expect("tree protocol");
    let mut conditions = Vec::new();
    yir_core::owned_select_tree_conditions(&args.tree, &mut conditions);
    assert_eq!(conditions.len(), 2);
    assert!(tree.op.args.iter().any(|arg| arg == "variant_field"));
    assert!(!yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "variant_field"));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("payload match LLVM lowering");
    assert_eq!(llvm_ir.matches("call ptr @nuis_fn_retain(ptr").count(), 2);
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
}

#[test]
fn lowers_nested_payload_field_as_selected_leaf_projection() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Payload {
            score: i64,
          }

          enum Route {
            Left(Payload),
            Right(Payload),
          }

          fn retain(bytes: Bytes, delta: i64) -> Bytes {
            return move(bytes);
          }

          fn choose(lhs: Bytes, rhs: Bytes, selector: i64) -> Bytes {
            let payload: Payload = Payload { score: selector };
            let route: Route = if selector == 1 {
              Route.Left(payload)
            } else {
              Route.Right(payload)
            };
            match route {
              Route.Left(value) => {
                return retain(move(lhs), value.score);
              }
              Route.Right(value) => {
                return move(rhs);
              }
              _ => {
                return retain(move(lhs), 7);
              }
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(2, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(3, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let selector: i64 = cpu_input_i64("selector", 1, 1, 2, 1);
            let selected: Bytes = choose(move(lhs), move(rhs), selector);
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("nested payload should lower");
    let tree = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_tree")
        .expect("nested payload match tree");
    assert!(tree.op.args.iter().any(|arg| arg == "struct_field"));
    assert!(tree.op.args.iter().any(|arg| arg == "variant_field"));
    assert!(!yir.nodes.iter().any(|node| node.op.instruction == "field"));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("nested payload LLVM lowering");
    assert_eq!(llvm_ir.matches("call ptr @nuis_fn_retain(ptr").count(), 2);
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
}

#[test]
fn lowers_nested_payload_cast_inside_selected_leaf() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Payload {
            score: i64,
          }

          enum Route {
            Left(Payload),
            Right(Payload),
          }

          fn retain(bytes: Bytes, delta: i32) -> Bytes {
            return move(bytes);
          }

          fn choose(lhs: Bytes, rhs: Bytes, selector: i64) -> Bytes {
            let payload: Payload = Payload { score: selector };
            let route: Route = if selector == 1 {
              Route.Left(payload)
            } else {
              Route.Right(payload)
            };
            match route {
              Route.Left(value) => {
                return retain(move(lhs), i32_from_i64(value.score));
              }
              Route.Right(value) => {
                return move(rhs);
              }
              _ => {
                return retain(move(lhs), i32_from_i64(7));
              }
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(2, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(3, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let selector: i64 = cpu_input_i64("selector", 1, 1, 2, 1);
            let selected: Bytes = choose(move(lhs), move(rhs), selector);
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

    let yir = lower_nir_to_yir_builtin_cpu(&module).expect("nested cast should lower");
    let tree = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_tree")
        .expect("nested cast match tree");
    assert!(tree
        .op
        .args
        .windows(2)
        .any(|args| args == ["cast", "i64_to_i32"]));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("nested cast LLVM lowering");
    assert!(llvm_ir.contains(" = trunc i64 "));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_tree"));
}

#[test]
fn keeps_referenced_payload_prelude_for_selected_leaf_lowering() {
    let stmts = vec![
        NirStmt::Let {
            name: "payload".to_owned(),
            ty: None,
            value: NirExpr::VariantFieldAccess {
                base: Box::new(NirExpr::Var("route".to_owned())),
                variant: "Route.Left".to_owned(),
                field: "value".to_owned(),
            },
        },
        NirStmt::Return(Some(NirExpr::Call {
            callee: "retain".to_owned(),
            args: vec![
                NirExpr::Move(Box::new(NirExpr::Var("bytes".to_owned()))),
                NirExpr::Var("payload".to_owned()),
            ],
        })),
    ];

    assert!(
        super::nested_owned_returns::strip_unused_pure_leaf_prelude(&stmts).is_none(),
        "a referenced payload must remain branch-local until selected-leaf projection exists"
    );
}
