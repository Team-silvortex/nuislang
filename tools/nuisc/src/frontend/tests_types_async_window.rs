use super::parse_nuis_module;
use nuis_semantics::model::{NirAddressClass, NirExpr, NirStmt, NirTypeShape};
use std::collections::BTreeMap;

#[test]
fn rejects_non_numeric_binary_operands() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn join(lhs: String, rhs: String) -> String {
            let out: String = lhs + rhs;
            return out;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("numeric scalar operands"));
}

#[test]
fn rejects_bare_window_type_without_payload() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let packet: Window = data_profile_send_uplink("FabricPlane", 7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("Window"));
    assert!(error.contains("payload type argument"));
}

#[test]
fn rejects_nested_pipe_payload_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let pipe: Pipe<Pipe<i64>> = data_output_pipe(7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("Pipe<Pipe"));
}

#[test]
fn accepts_window_mut_type_annotation() {
    parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn keeps_window_annotation_compatible_with_copy_window_for_now() {
    parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let copy: Window<i64> = data_copy_window(7, 0, 1);
          }
        }
        "#,
    )
    .unwrap();
}

#[test]
fn infers_frozen_window_as_immutable_window_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let frozen: Window<i64> = data_freeze_window(data_copy_window(7, 0, 1));
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[0] else {
        panic!("expected typed let binding");
    };
    assert_eq!(ty.render(), "Window<i64>");
}

#[test]
fn infers_written_window_as_mutable_window_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
            let updated: WindowMut<i64> = data_write_window(copy, 0, 9);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
        panic!("expected typed let binding");
    };
    assert_eq!(ty.render(), "WindowMut<i64>");
}

#[test]
fn infers_buffer_backed_window_payload_as_i64() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let backing: ref Buffer = alloc_buffer(4, 0);
            let copy: WindowMut<i64> = data_copy_window(backing, 1, 2);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
        panic!("expected typed let binding");
    };
    assert_eq!(ty.render(), "WindowMut<i64>");
}

#[test]
fn infers_alloc_node_binding_as_ref_address_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let head: ref Node = alloc_node(7, null());
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[0] else {
        panic!("expected typed let binding");
    };
    assert_eq!(ty.render(), "ref Node");
    assert_eq!(ty.shape(), NirTypeShape::Ref);
    assert!(!ty.is_async_boundary_safe());
}

#[test]
fn infers_load_next_binding_as_ref_address_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let nil: ref Node? = null();
            let tail: ref Node = move(alloc_node(30, nil));
            let head: ref Node = alloc_node(10, tail);
            let next_ptr: ref Node = load_next(head);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[3] else {
        panic!("expected typed next-pointer binding");
    };
    assert_eq!(ty.render(), "ref Node");
    assert_eq!(ty.shape(), NirTypeShape::Ref);
}

#[test]
fn infers_alloc_buffer_binding_as_ref_buffer_address_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let scratch: ref Buffer = alloc_buffer(4, 0);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[0] else {
        panic!("expected typed let binding");
    };
    assert_eq!(ty.render(), "ref Buffer");
    assert_eq!(ty.shape(), NirTypeShape::Ref);
    assert_eq!(ty.container_kind(), None);
    assert!(!ty.is_async_boundary_safe());
}

#[test]
fn infers_borrow_expr_as_same_ref_address_surface_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let head: ref Node = alloc_node(7, null());
            let head_ref: ref Node = borrow(head);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
        panic!("expected typed borrow binding");
    };
    assert_eq!(ty.render(), "ref Node");
    assert_eq!(ty.shape(), NirTypeShape::Ref);
    assert!(ty.is_address_type());
    assert_eq!(ty.address_target_name(), Some("Node"));
}

#[test]
fn infers_borrow_end_expr_as_unit_not_new_address_surface() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let head: ref Node = alloc_node(7, null());
            let head_ref: ref Node = borrow(head);
            let closed: Unit = borrow_end(head_ref);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[2] else {
        panic!("expected typed borrow_end binding");
    };
    assert_eq!(ty.render(), "Unit");
    assert!(!ty.is_address_type());
}

#[test]
fn classifies_address_expression_authority_for_alloc_borrow_move_and_next_load() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let nil: ref Node? = null();
            let tail: ref Node = move(alloc_node(30, nil));
            let head: ref Node = alloc_node(10, tail);
            let head_ref: ref Node = borrow(head);
            let moved_head: ref Node = move(head);
            let next_ptr: ref Node = load_next(moved_head);
          }
        }
        "#,
    )
    .unwrap();

    let function = &module.functions[0];
    let mut type_bindings = BTreeMap::new();
    let mut address_classes = BTreeMap::new();
    let empty_signatures = BTreeMap::new();
    let empty_structs = BTreeMap::new();

    for stmt in &function.body {
        let NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        } = stmt
        else {
            continue;
        };
        type_bindings.insert(name.clone(), ty.clone());
        if let Some(class) = super::types::infer_nir_expr_address_class(
            value,
            &type_bindings,
            &address_classes,
            &empty_signatures,
            &empty_structs,
        ) {
            address_classes.insert(name.clone(), class);
        }
    }

    assert_eq!(address_classes.get("tail"), Some(&NirAddressClass::Owned));
    assert_eq!(address_classes.get("head"), Some(&NirAddressClass::Owned));
    assert_eq!(
        address_classes.get("head_ref"),
        Some(&NirAddressClass::Borrowed)
    );
    assert_eq!(
        address_classes.get("moved_head"),
        Some(&NirAddressClass::Owned)
    );
    assert_eq!(
        address_classes.get("next_ptr"),
        Some(&NirAddressClass::Owned)
    );
}

#[test]
fn classifies_load_next_from_borrowed_source_as_borrowed_address() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let nil: ref Node? = null();
            let tail: ref Node = move(alloc_node(30, nil));
            let head: ref Node = alloc_node(10, tail);
            let head_ref: ref Node = borrow(head);
            let next_ptr: ref Node = load_next(head_ref);
          }
        }
        "#,
    )
    .unwrap();

    let function = &module.functions[0];
    let mut type_bindings = BTreeMap::new();
    let mut address_classes = BTreeMap::new();
    let empty_signatures = BTreeMap::new();
    let empty_structs = BTreeMap::new();

    for stmt in &function.body {
        let NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        } = stmt
        else {
            continue;
        };
        type_bindings.insert(name.clone(), ty.clone());
        if let Some(class) = super::types::infer_nir_expr_address_class(
            value,
            &type_bindings,
            &address_classes,
            &empty_signatures,
            &empty_structs,
        ) {
            address_classes.insert(name.clone(), class);
        }
    }

    assert_eq!(
        address_classes.get("head_ref"),
        Some(&NirAddressClass::Borrowed)
    );
    assert_eq!(
        address_classes.get("next_ptr"),
        Some(&NirAddressClass::Borrowed)
    );
}

#[test]
fn lowers_unary_pointer_null_check_and_node_deref() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let nil: ref Node? = null();
            let head: ref Node = alloc_node(7, nil);
            let empty: bool = !head;
            let value: i64 = *head;
            if empty {
              return 0;
            }
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let function = &module.functions[0];
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::IsNull(inner),
            ..
        }) if name == "empty" && matches!(inner.as_ref(), NirExpr::Var(var) if var == "head")
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            name,
            value: NirExpr::LoadValue(inner),
            ..
        }) if name == "value" && matches!(inner.as_ref(), NirExpr::Var(var) if var == "head")
    ));
}

#[test]
fn rejects_unary_deref_for_non_node_address_types() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let backing: ref Buffer = alloc_buffer(4, 0);
            let value: i64 = *backing;
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("unary `*` currently expects `ref Node` operand"),
        "{error}"
    );
    assert!(error.contains("ref Buffer"), "{error}");
}

#[test]
fn lowers_node_pointer_field_access_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let nil: ref Node? = null();
            let tail: ref Node = move(alloc_node(30, nil));
            let head: ref Node = alloc_node(10, tail);
            let value: i64 = head.value;
            let next_ptr: ref Node = head.next;
            return value + load_value(next_ptr);
          }
        }
        "#,
    )
    .unwrap();

    let function = &module.functions[0];
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            name,
            value: NirExpr::LoadValue(inner),
            ..
        }) if name == "value" && matches!(inner.as_ref(), NirExpr::Var(var) if var == "head")
    ));
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            name,
            value: NirExpr::LoadNext(inner),
            ..
        }) if name == "next_ptr" && matches!(inner.as_ref(), NirExpr::Var(var) if var == "head")
    ));
}

#[test]
fn rejects_unknown_node_pointer_field_access_sugar() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let nil: ref Node? = null();
            let head: ref Node = alloc_node(10, nil);
            return head.missing;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("pointer field sugar currently supports only `value` and `next`"),
        "{error}"
    );
    assert!(error.contains("ref Node"), "{error}");
}

#[test]
fn lowers_buffer_index_and_len_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let backing: ref Buffer = alloc_buffer(4, 9);
            let len: i64 = backing.len;
            let first: i64 = backing[0];
            return len + first;
          }
        }
        "#,
    )
    .unwrap();

    let function = &module.functions[0];
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::BufferLen(inner),
            ..
        }) if name == "len" && matches!(inner.as_ref(), NirExpr::Var(var) if var == "backing")
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::LoadAt { buffer, index },
            ..
        }) if name == "first"
            && matches!(buffer.as_ref(), NirExpr::Var(var) if var == "backing")
            && matches!(
                index.as_ref(),
                NirExpr::Binary { lhs, rhs, .. }
                    if matches!(lhs.as_ref(), NirExpr::Int(0))
                        && matches!(rhs.as_ref(), NirExpr::Int(0))
            )
    ));
}

#[test]
fn rejects_unknown_buffer_field_access_sugar() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let backing: ref Buffer = alloc_buffer(4, 0);
            return backing.value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("buffer field sugar currently supports only `len`"),
        "{error}"
    );
    assert!(error.contains("ref Buffer"), "{error}");
}

#[test]
fn lowers_buffer_index_assignment_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let backing: ref Buffer = alloc_buffer(4, 0);
            backing[1] = 9;
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.get(1),
        Some(NirStmt::Expr(NirExpr::StoreAt { buffer, index, value }))
            if matches!(buffer.as_ref(), NirExpr::Var(name) if name == "backing")
                && matches!(
                    index.as_ref(),
                    NirExpr::Binary { lhs, rhs, .. }
                        if matches!(lhs.as_ref(), NirExpr::Int(0))
                            && matches!(rhs.as_ref(), NirExpr::Int(1))
                )
                && matches!(value.as_ref(), NirExpr::Int(9))
    ));
}

#[test]
fn lowers_node_pointer_field_assignment_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let nil: ref Node? = null();
            let tail: ref Node = move(alloc_node(30, nil));
            let head: ref Node = alloc_node(10, tail);
            head.value = 77;
            head.next = tail;
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.get(3),
        Some(NirStmt::Expr(NirExpr::StoreValue { target, value }))
            if matches!(target.as_ref(), NirExpr::Var(name) if name == "head")
                && matches!(value.as_ref(), NirExpr::Int(77))
    ));
    assert!(matches!(
        module.functions[0].body.get(4),
        Some(NirStmt::Expr(NirExpr::StoreNext { target, next }))
            if matches!(target.as_ref(), NirExpr::Var(name) if name == "head")
                && matches!(next.as_ref(), NirExpr::Var(name) if name == "tail")
    ));
}

#[test]
fn rejects_read_only_buffer_len_assignment() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let backing: ref Buffer = alloc_buffer(4, 0);
            backing.len = 3;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("`.len` is read-only"), "{error}");
}

#[test]
fn infers_read_window_payload_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
            let value: i64 = data_read_window(copy, 0);
          }
        }
        "#,
    )
    .unwrap();

    let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
        panic!("expected typed let binding");
    };
    assert_eq!(ty.render(), "i64");
}

#[test]
fn rejects_instance_of_scalar_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let wrong: Instance<i64> = instantiate shader SurfaceShader;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("nominal unit type"));
}

#[test]
fn accepts_typed_marker_and_handle_table_annotations() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let handles: HandleTable<FabricBindings> =
              data_profile_handle_table("FabricPlane");
            let ready: Marker<CpuToShader> =
              data_profile_marker("FabricPlane", "cpu_to_shader");
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    let declared_types = function
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            NirStmt::Let { ty: Some(ty), .. } => Some(ty.render()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(declared_types.contains(&"HandleTable<FabricBindings>".to_owned()));
    assert!(declared_types.contains(&"Marker<CpuToShader>".to_owned()));
}

#[test]
fn rejects_marker_with_non_nominal_tag_type() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let ready: Marker<i64> = data_marker("cpu_to_shader");
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(error.contains("nominal tag type"));
}

#[test]
fn lowers_async_fn_and_await_stmt_into_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          async fn main() {
            await ping();
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(function.is_async);
    assert!(matches!(function.body.first(), Some(NirStmt::Await(_))));
}

#[test]
fn lowers_await_expression_in_let_and_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          async fn main() -> i64 {
            let value: i64 = await ping();
            return await ping();
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            value: NirExpr::Await(_),
            ..
        })
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Await(_))))
    ));
}

#[test]
fn lowers_await_expression_inside_call_args_and_binary_expr() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn add_one(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = add_one(await ping());
            return await ping() + value;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            value: NirExpr::Call { args, .. },
            ..
        }) if matches!(args.first(), Some(NirExpr::Await(_)))
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Binary { lhs, .. })))
            if matches!(lhs.as_ref(), NirExpr::Await(_))
    ));
}

#[test]
fn lowers_while_into_nir_statement() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            while value < 3 {
              print(value);
              continue;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::While { condition, body })
            if matches!(condition, NirExpr::Binary { .. })
                && matches!(body.as_slice(), [NirStmt::Print(_), NirStmt::Continue])
    ));
}

#[test]
fn lowers_break_into_nir_statement() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            while true {
              break;
            }
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::While { body, .. }) if matches!(body.as_slice(), [NirStmt::Break])
    ));
}

#[test]
fn lowers_continue_into_nir_statement() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            while true {
              continue;
            }
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::While { body, .. }) if matches!(body.as_slice(), [NirStmt::Continue])
    ));
}

#[test]
fn lowers_explicit_spawn_join_and_cancel() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(ping());
            cancel(task);
            return join(task);
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
            ..
        }) if ty.render() == "Task<i64>"
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Expr(NirExpr::CpuCancel(_)))
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::CpuJoin(_))))
    ));
}
