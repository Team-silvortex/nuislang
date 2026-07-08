use super::*;

#[test]
fn infers_struct_field_type_from_shared_type_helper() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Packet {
            count: i32,
            label: String,
          }

          fn pick(packet: Packet) -> i32 {
            return packet.count;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "pick")
        .unwrap();
    let return_type = function.return_type.as_ref().unwrap();
    assert_eq!(return_type.render(), "i32");
}

#[test]
fn infers_binary_result_from_operand_scalar_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add(lhs: i32, rhs: i32) -> i32 {
            let sum: i32 = lhs + rhs;
            return sum;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "add")
        .unwrap();
    let sum_stmt = function
        .body
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::Let { name, ty, .. } if name == "sum" => ty.as_ref(),
            _ => None,
        })
        .unwrap();
    assert_eq!(sum_stmt.render(), "i32");
}

#[test]
fn lowers_mutable_local_reassignment_into_rebound_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let mut value: i64 = 1;
            value = value + 2;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Let { name, value, .. })
            if name == "value" && matches!(value, NirExpr::Int(1))
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "value"
            && *op == NirBinaryOp::Add
            && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
            && matches!(rhs.as_ref(), NirExpr::Int(2))
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "value"
    ));
}

#[test]
fn lowers_mutable_local_compound_assignment() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let mut value: i64 = 9;
            value %= 4;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "value"
            && *op == NirBinaryOp::Rem
            && matches!(lhs.as_ref(), NirExpr::Var(lhs_name) if lhs_name == "value")
            && matches!(rhs.as_ref(), NirExpr::Int(4))
    ));
}

#[test]
fn lowers_slice_i64_construction_and_index_access() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i64> = slice(buffer, 2, 3);
            view[1] = 9;
            let value: i64 = view[1];
            let size: i64 = view.len;
            free(buffer);
            return value + size;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, type_args, fields },
        }) if name == "view"
            && ty.render() == "Slice<i64>"
            && type_name == "Slice"
            && type_args.len() == 1
            && type_args[0].render() == "i64"
            && fields.iter().map(|(field, _)| field.as_str()).collect::<Vec<_>>() == vec!["buffer", "start", "len"]
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Expr(NirExpr::StoreAt { buffer, index, value }))
            if matches!(buffer.as_ref(), NirExpr::FieldAccess { field, .. } if field == "buffer")
                && matches!(index.as_ref(),
                    NirExpr::Binary { op, lhs, rhs }
                    if *op == NirBinaryOp::Add
                        && matches!(lhs.as_ref(), NirExpr::FieldAccess { field, .. } if field == "start")
                        && matches!(rhs.as_ref(), NirExpr::Int(1))
                )
                && matches!(value.as_ref(), NirExpr::Int(9))
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::LoadAt { buffer, index },
        }) if name == "value"
            && ty.render() == "i64"
            && matches!(buffer.as_ref(), NirExpr::FieldAccess { field, .. } if field == "buffer")
            && matches!(index.as_ref(),
                NirExpr::Binary { op, lhs, rhs }
                if *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::FieldAccess { field, .. } if field == "start")
                    && matches!(rhs.as_ref(), NirExpr::Int(1))
            )
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
        }) if name == "size" && ty.render() == "i64" && field == "len"
    ));
}

#[test]
fn lowers_explicit_slice_builtins() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i64> = slice<i64>(buffer, 2, 3);
            let raw: ref Buffer = slice_buffer(view);
            let start: i64 = slice_start(view);
            let size: i64 = slice_len(view);
            free(buffer);
            return buffer_len(raw) + start + size;
          }
        }
        "#,
    )
    .unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, type_args, .. },
        }) if name == "view"
            && ty.render() == "Slice<i64>"
            && type_name == "Slice"
            && type_args.len() == 1
            && type_args[0].render() == "i64"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
        }) if name == "raw" && ty.render() == "ref Buffer" && field == "buffer"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
        }) if name == "start" && ty.render() == "i64" && field == "start"
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
        }) if name == "size" && ty.render() == "i64" && field == "len"
    ));
}

#[test]
fn rejects_non_i64_explicit_slice_payload_for_buffer_view() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<String> = slice<String>(buffer, 2, 3);
            free(buffer);
            return view.len;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "slice<...>(...) currently supports only `Slice<i64>`, `Slice<i32>`, `Slice<bool>`, `Slice<f32>`, and `Slice<f64>`"
        ),
        "{error}"
    );
}
