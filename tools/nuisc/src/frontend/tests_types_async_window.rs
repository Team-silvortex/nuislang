use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

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
