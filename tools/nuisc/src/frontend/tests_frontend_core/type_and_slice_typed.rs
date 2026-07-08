use super::*;

#[test]
fn lowers_slice_i32_read_and_write_via_slot_casts() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i32 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i32> = slice<i32>(buffer, 1, 2);
            let value: i32 = i32_from_i64(7);
            view[0] = value;
            let replay: i32 = view[0];
            free(buffer);
            return replay;
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
            value: NirExpr::StructLiteral { type_args, .. },
        }) if name == "view" && ty.render() == "Slice<i32>" && type_args[0].render() == "i32"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Expr(NirExpr::StoreAt { value, .. }))
            if matches!(value.as_ref(), NirExpr::CastI32ToI64(inner)
                if matches!(inner.as_ref(), NirExpr::Var(name) if name == "value"))
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CastI64ToI32(inner),
        }) if name == "replay" && ty.render() == "i32"
            && matches!(inner.as_ref(), NirExpr::LoadAt { .. })
    ));
}

#[test]
fn lowers_slice_bool_read_and_write_via_slot_casts() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> bool {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<bool> = slice<bool>(buffer, 1, 2);
            let value: bool = true;
            view[0] = value;
            let replay: bool = view[0];
            free(buffer);
            return replay;
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
            value: NirExpr::StructLiteral { type_args, .. },
        }) if name == "view" && ty.render() == "Slice<bool>" && type_args[0].render() == "bool"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Expr(NirExpr::StoreAt { value, .. }))
            if matches!(value.as_ref(), NirExpr::CastBoolToI64(inner)
                if matches!(inner.as_ref(), NirExpr::Var(name) if name == "value"))
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CastI64ToBool(inner),
        }) if name == "replay" && ty.render() == "bool"
            && matches!(inner.as_ref(), NirExpr::LoadAt { .. })
    ));
}

#[test]
fn lowers_slice_f32_read_and_write_via_slot_casts() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> f32 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<f32> = slice<f32>(buffer, 1, 2);
            let value: f32 = 1.5;
            view[0] = value;
            let replay: f32 = view[0];
            free(buffer);
            return replay;
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
            value: NirExpr::StructLiteral { type_args, .. },
        }) if name == "view" && ty.render() == "Slice<f32>" && type_args[0].render() == "f32"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Expr(NirExpr::StoreAt { value, .. }))
            if matches!(value.as_ref(), NirExpr::CastF32ToI64(inner)
                if matches!(inner.as_ref(), NirExpr::Var(name) if name == "value"))
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CastI64ToF32(inner),
        }) if name == "replay" && ty.render() == "f32"
            && matches!(inner.as_ref(), NirExpr::LoadAt { .. })
    ));
}

#[test]
fn lowers_slice_f64_read_and_write_via_slot_casts() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> f64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<f64> = slice<f64>(buffer, 1, 2);
            let value: f64 = 1.5;
            view[0] = value;
            let replay: f64 = view[0];
            free(buffer);
            return replay;
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
            value: NirExpr::StructLiteral { type_args, .. },
        }) if name == "view" && ty.render() == "Slice<f64>" && type_args[0].render() == "f64"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Expr(NirExpr::StoreAt { value, .. }))
            if matches!(value.as_ref(), NirExpr::CastF64ToI64(inner)
                if matches!(inner.as_ref(), NirExpr::Var(name) if name == "value"))
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CastI64ToF64(inner),
        }) if name == "replay" && ty.render() == "f64"
            && matches!(inner.as_ref(), NirExpr::LoadAt { .. })
    ));
}

#[test]
fn lowers_slice_alias_payload_byte_as_i64_view() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Byte = i64;
          type ByteSlice = Slice<Byte>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: ByteSlice = slice<Byte>(buffer, 1, 2);
            view[0] = 72;
            let replay: Byte = view[0];
            free(buffer);
            return replay;
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
            value: NirExpr::StructLiteral { type_args, .. },
        }) if name == "view" && ty.render() == "Slice<i64>" && type_args[0].render() == "i64"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Expr(NirExpr::StoreAt { value, .. }))
            if matches!(value.as_ref(), NirExpr::Int(72))
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::LoadAt { .. },
        }) if name == "replay" && ty.render() == "i64"
    ));
}

#[test]
fn lowers_bytes_and_subbytes_builtins_as_i64_byte_views() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Byte = i64;
          type ByteSlice = Slice<Byte>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: ByteSlice = bytes(buffer, 1, 4);
            let inner: ByteSlice = subbytes(view, 1, 2);
            inner[0] = 72;
            let replay: Byte = inner[0];
            free(buffer);
            return replay + inner.len;
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
            value: NirExpr::StructLiteral { type_args, .. },
        }) if name == "view" && ty.render() == "Slice<i64>" && type_args[0].render() == "i64"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_args, fields, .. },
        }) if name == "inner"
            && ty.render() == "Slice<i64>"
            && type_args[0].render() == "i64"
            && matches!(fields.as_slice(),
                [(buffer_field, NirExpr::FieldAccess { field: buffer_access, .. }),
                 (start_field, NirExpr::Binary { op, lhs, rhs }),
                 (len_field, NirExpr::Int(2))]
                if buffer_field == "buffer"
                    && buffer_access == "buffer"
                    && start_field == "start"
                    && *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::FieldAccess { field, .. } if field == "start")
                    && matches!(rhs.as_ref(), NirExpr::Int(1))
                    && len_field == "len"
            )
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Expr(NirExpr::StoreAt { value, .. }))
            if matches!(value.as_ref(), NirExpr::Int(72))
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::LoadAt { .. },
        }) if name == "replay" && ty.render() == "i64"
    ));
}
