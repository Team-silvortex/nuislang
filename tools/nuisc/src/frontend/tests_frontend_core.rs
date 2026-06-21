use super::lower_type_ref;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{
    AstBinaryOp, AstDestructureBinding, AstDestructureField, AstExpr, AstStmt, AstVisibility,
    NirBinaryOp, NirExpr, NirStmt,
};
use std::fs;
use std::path::PathBuf;

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

#[test]
fn lowers_text_handle_builtin_as_host_text_handle() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let lifted: i64 = text_handle("hello");
            return lifted;
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
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "lifted"
            && ty.render() == "i64"
            && callee == "host_text_handle"
            && args.len() == 1
    ));
}

#[test]
fn rewrites_serialize_then_deserialize_text_helper_into_text_handle() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn helper(value: String) -> i64 {
            let buffer: ref Buffer = alloc_buffer(128, 0);
            let len: i64 = serialize_text_into(value, buffer, 0);
            return deserialize_text_from(buffer, 0, len);
          }

          fn main() -> i64 {
            return helper("hello");
          }
        }
        "#,
    )
    .unwrap();

    let helper = module
        .functions
        .iter()
        .find(|function| function.name == "helper")
        .unwrap();
    assert!(matches!(
        helper.body.first(),
        Some(NirStmt::Return(Some(NirExpr::CpuExternCall { callee, args, .. })))
            if callee == "host_text_handle" && args.len() == 1
    ));
    assert!(helper
        .annotations
        .iter()
        .any(|annotation| annotation.name == "__nuisc_text_handle_rewrite"));
}

#[test]
fn rewrites_local_i64_text_handle_pattern_when_buffer_is_not_reused() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(128, 0);
            let len: i64 = serialize_text_into("hello", buffer, 0);
            let handle: i64 = deserialize_text_from(buffer, 0, len);
            return handle;
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
        main.body.get(0),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "handle"
            && ty.render() == "i64"
            && callee == "host_text_handle"
            && args.len() == 1
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Var(name)))) if name == "handle"
    ));
    assert!(main
        .annotations
        .iter()
        .any(|annotation| annotation.name == "__nuisc_text_handle_rewrite"));
}

#[test]
fn lowers_fill_copy_compare_byte_builtins_as_host_calls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type ByteSlice = Slice<i64>;

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(8, 0);
            let rhs_buffer: ref Buffer = alloc_buffer(8, 0);
            let lhs: ByteSlice = bytes(lhs_buffer, 0, 4);
            let rhs: ByteSlice = bytes(rhs_buffer, 2, 4);
            let filled: i64 = fillbytes(lhs, 65);
            let copied: i64 = copybytes(rhs, lhs);
            let cmp: i64 = comparebytes(lhs, rhs);
            free(rhs_buffer);
            free(lhs_buffer);
            return filled + copied + cmp;
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
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "filled"
            && ty.render() == "i64"
            && callee == "host_fill_bytes"
            && args.len() == 4
    ));
    assert!(matches!(
        main.body.get(5),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "copied"
            && ty.render() == "i64"
            && callee == "host_copy_bytes"
            && args.len() == 6
    ));
    assert!(matches!(
        main.body.get(6),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "cmp"
            && ty.render() == "i64"
            && callee == "host_compare_bytes"
            && args.len() == 6
    ));
}

#[test]
fn lowers_bytes_prefixed_helpers_as_standardized_byte_api() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type ByteSlice = Slice<i64>;

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(8, 0);
            let rhs_buffer: ref Buffer = alloc_buffer(8, 0);
            let lhs: ByteSlice = bytes(lhs_buffer, 0, 4);
            let rhs: ByteSlice = bytes(rhs_buffer, 1, 3);
            let filled: i64 = bytes_fill(lhs, 65);
            let copied: i64 = bytes_copy_from(lhs, rhs);
            let cmp: i64 = bytes_compare(lhs, rhs);
            let eq: bool = bytes_eq(lhs, rhs);
            let starts: bool = bytes_starts_with(lhs, rhs);
            let ends: bool = bytes_ends_with(lhs, rhs);
            free(rhs_buffer);
            free(lhs_buffer);
            return filled
              + copied
              + cmp
              + if eq { 1 } else { 0 }
              + if starts { 1 } else { 0 }
              + if ends { 1 } else { 0 };
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
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, .. },
        }) if name == "filled" && ty.render() == "i64" && callee == "host_fill_bytes"
    ));
    assert!(matches!(
        main.body.get(5),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, .. },
        }) if name == "copied" && ty.render() == "i64" && callee == "host_copy_bytes"
    ));
    assert!(matches!(
        main.body.get(6),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, .. },
        }) if name == "cmp" && ty.render() == "i64" && callee == "host_compare_bytes"
    ));
    assert!(matches!(
        main.body.get(7),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Binary { op, .. },
        }) if name == "eq" && ty.render() == "bool" && *op == NirBinaryOp::Eq
    ));
    assert!(matches!(
        main.body.get(8),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Binary { op, .. },
        }) if name == "starts" && ty.render() == "bool" && *op == NirBinaryOp::And
    ));
    assert!(matches!(
        main.body.get(9),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Binary { op, .. },
        }) if name == "ends" && ty.render() == "bool" && *op == NirBinaryOp::And
    ));
}

#[test]
fn lowers_bytes_search_helpers_as_slice_window_host_calls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type ByteSlice = Slice<i64>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let view: ByteSlice = bytes(buffer, 5, 12);
            let marker: i64 = bytes_find_byte(view, 58);
            let value: i64 = bytes_find_text(view, "value");
            let line_end: i64 = bytes_find_line_end(view);
            let trimmed: i64 = bytes_trim_line_end(view);
            free(buffer);
            return marker + value + line_end + trimmed;
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
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "marker"
            && ty.render() == "i64"
            && callee == "host_buffer_find_byte"
            && args.len() == 4
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "value"
            && ty.render() == "i64"
            && callee == "host_buffer_find_text"
            && args.len() == 4
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "line_end"
            && ty.render() == "i64"
            && callee == "host_buffer_find_line_end"
            && args.len() == 3
    ));
    assert!(matches!(
        main.body.get(5),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuExternCall { callee, args, .. },
        }) if name == "trimmed"
            && ty.render() == "i64"
            && callee == "host_buffer_trim_line_end"
            && args.len() == 3
    ));
}

#[test]
fn lowers_bytes_contains_helpers_as_find_not_missing_checks() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type ByteSlice = Slice<i64>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let view: ByteSlice = bytes(buffer, 5, 12);
            let has_colon: bool = bytes_contains_byte(view, 58);
            let has_value: bool = bytes_contains_text(view, "value");
            free(buffer);
            return if has_colon && has_value { 1 } else { 0 };
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
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Binary { op, lhs, rhs },
        }) if name == "has_colon"
            && ty.render() == "bool"
            && *op == NirBinaryOp::Ne
            && matches!(lhs.as_ref(), NirExpr::CpuExternCall { callee, .. } if callee == "host_buffer_find_byte")
            && matches!(rhs.as_ref(), NirExpr::Int(-1))
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Binary { op, lhs, rhs },
        }) if name == "has_value"
            && ty.render() == "bool"
            && *op == NirBinaryOp::Ne
            && matches!(lhs.as_ref(), NirExpr::CpuExternCall { callee, .. } if callee == "host_buffer_find_text")
            && matches!(rhs.as_ref(), NirExpr::Int(-1))
    ));
}

#[test]
fn lowers_bytes_slice_before_and_after_with_absolute_indices() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type ByteSlice = Slice<i64>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let view: ByteSlice = bytes(buffer, 5, 12);
            let marker: i64 = bytes_find_byte(view, 58);
            let head: ByteSlice = bytes_slice_before(view, marker);
            let tail: ByteSlice = bytes_slice_after(view, marker);
            free(buffer);
            return head.len + tail.len;
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
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_args, fields, .. },
        }) if name == "head"
            && ty.render() == "Slice<i64>"
            && type_args[0].render() == "i64"
            && matches!(fields.as_slice(),
                [(buffer_field, NirExpr::FieldAccess { field: buffer_access, .. }),
                 (start_field, NirExpr::FieldAccess { field: start_access, .. }),
                 (len_field, NirExpr::Binary { op, lhs, rhs })]
                if buffer_field == "buffer"
                    && buffer_access == "buffer"
                    && start_field == "start"
                    && start_access == "start"
                    && len_field == "len"
                    && *op == NirBinaryOp::Sub
                    && matches!(lhs.as_ref(), NirExpr::Var(name) if name == "marker")
                    && matches!(rhs.as_ref(), NirExpr::FieldAccess { field, .. } if field == "start")
            )
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_args, fields, .. },
        }) if name == "tail"
            && ty.render() == "Slice<i64>"
            && type_args[0].render() == "i64"
            && matches!(fields.as_slice(),
                [(buffer_field, NirExpr::FieldAccess { field: buffer_access, .. }),
                 (start_field, NirExpr::Binary { op: start_op, .. }),
                 (len_field, NirExpr::Binary { op: len_op, .. })]
                if buffer_field == "buffer"
                    && buffer_access == "buffer"
                    && start_field == "start"
                    && *start_op == NirBinaryOp::Add
                    && len_field == "len"
                    && *len_op == NirBinaryOp::Sub
            )
    ));
}

#[test]
fn lowers_bytes_split_once_helpers_as_byte_split_structs() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type ByteSlice = Slice<i64>;

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(32, 0);
            let view: ByteSlice = bytes(buffer, 5, 12);
            let split = bytes_split_once_byte(view, 58);
            let text_split = bytes_split_once_text(view, "value");
            free(buffer);
            return split.index
              + text_split.index
              + split.before.len
              + text_split.after.len
              + if split.found { 1 } else { 0 };
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
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, fields, .. },
        }) if name == "split"
            && ty.render() == "ByteSplit"
            && type_name == "ByteSplit"
            && fields.iter().any(|(field, value)| field == "found" && matches!(value, NirExpr::Binary { op, .. } if *op == NirBinaryOp::Ne))
            && fields.iter().any(|(field, value)| field == "index" && matches!(value, NirExpr::CpuExternCall { callee, .. } if callee == "host_buffer_find_byte"))
            && fields.iter().any(|(field, value)| field == "before" && matches!(value, NirExpr::StructLiteral { type_name, .. } if type_name == "Slice"))
            && fields.iter().any(|(field, value)| field == "after" && matches!(value, NirExpr::StructLiteral { type_name, .. } if type_name == "Slice"))
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, fields, .. },
        }) if name == "text_split"
            && ty.render() == "ByteSplit"
            && type_name == "ByteSplit"
            && fields.iter().any(|(field, value)| field == "index" && matches!(value, NirExpr::CpuExternCall { callee, .. } if callee == "host_buffer_find_text"))
    ));
}

#[test]
fn lowers_subslice_i64_with_rebased_start() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(8, 0);
            let view: Slice<i64> = slice<i64>(buffer, 2, 5);
            let inner: Slice<i64> = subslice<i64>(view, 1, 2);
            inner[0] = 7;
            let value: i64 = inner[0];
            free(buffer);
            return value + inner.len;
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
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, type_args, fields },
        }) if name == "inner"
            && ty.render() == "Slice<i64>"
            && type_name == "Slice"
            && type_args.len() == 1
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
        Some(NirStmt::Expr(NirExpr::StoreAt { index, value, .. }))
            if matches!(index.as_ref(),
                NirExpr::Binary { op, lhs, rhs }
                if *op == NirBinaryOp::Add
                    && matches!(lhs.as_ref(), NirExpr::FieldAccess { field, .. } if field == "start")
                    && matches!(rhs.as_ref(), NirExpr::Int(0))
            )
            && matches!(value.as_ref(), NirExpr::Int(7))
    ));
}

#[test]
fn rejects_reassignment_of_immutable_local() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 1;
            value = 2;
            return value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("cannot assign to immutable local `value`"),
        "{error}"
    );
    assert!(error.contains("let mut"), "{error}");
}

#[test]
fn rejects_reassignment_of_unknown_local() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            value = 2;
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("cannot assign to unknown local `value`"),
        "{error}"
    );
}

#[test]
fn lowers_float_literals_with_expected_scalar_context() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn add32() -> f32 {
            let sum: f32 = 1.5 + 2.25;
            return sum;
          }

          fn add64() -> f64 {
            return 1.5 + 2.25;
          }
        }
        "#,
    )
    .unwrap();

    let add32 = module
        .functions
        .iter()
        .find(|function| function.name == "add32")
        .unwrap();
    let sum_ty = add32
        .body
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::Let { name, ty, value } if name == "sum" => {
                assert!(matches!(
                    value,
                    NirExpr::Binary {
                        lhs,
                        rhs,
                        ..
                    } if matches!(lhs.as_ref(), NirExpr::F32(value) if value == "1.5")
                        && matches!(rhs.as_ref(), NirExpr::F32(value) if value == "2.25")
                ));
                ty.as_ref()
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(sum_ty.render(), "f32");

    let add64 = module
        .functions
        .iter()
        .find(|function| function.name == "add64")
        .unwrap();
    assert!(matches!(
        add64.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Binary { lhs, rhs, .. })))
            if matches!(lhs.as_ref(), NirExpr::F64(value) if value == "1.5")
                && matches!(rhs.as_ref(), NirExpr::F64(value) if value == "2.25")
    ));
}

#[test]
fn lowers_project_local_cpu_helper_calls_with_qualified_callees() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_completed(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          pub fn encode_completed(value: i64) -> i64 {
            return value + 1;
          }

          pub fn task_policy_completed(value: i64) -> i64 {
            return encode_completed(value);
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
    let helper_function = module
        .functions
        .iter()
        .find(|function| function.name == "TaskHelpers.task_policy_completed")
        .unwrap();
    assert!(matches!(
        helper_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "TaskHelpers.encode_completed"
    ));

    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "TaskHelpers.task_policy_completed"
    ));
}

#[test]
fn lowers_payload_style_single_field_struct_constructor_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just {
            value: i64,
          }

          fn main() -> i64 {
            let payload: Just = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, value, .. } => {
            assert_eq!(name, "payload");
            assert!(matches!(
                value,
                NirExpr::StructLiteral { type_name, fields, .. }
                    if type_name == "Just"
                        && matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Int(7))] if field == "value"
                        )
            ));
        }
        other => panic!("expected lowered payload constructor let, found {other:?}"),
    }
}

#[test]
fn parses_compound_buffer_assignment_into_store_at_sugar() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main(scratch: ref Buffer, slot: i64, step: i64) -> i64 {
            scratch[slot] += step;
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let main = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(AstStmt::Expr(AstExpr::Call { callee, generic_args, args }))
            if callee == "store_at"
                && generic_args.is_empty()
                && matches!(
                    args.as_slice(),
                    [
                        AstExpr::Var(buffer),
                        AstExpr::Var(slot),
                        AstExpr::Binary {
                            op,
                            lhs,
                            rhs,
                        }
                    ] if buffer == "scratch"
                        && slot == "slot"
                        && *op == AstBinaryOp::Add
                        && matches!(
                            lhs.as_ref(),
                            AstExpr::Call { callee: load_callee, generic_args: load_generics, args: load_args }
                                if load_callee == "load_at"
                                    && load_generics.is_empty()
                                    && matches!(
                                        load_args.as_slice(),
                                        [AstExpr::Var(load_buffer), AstExpr::Var(load_slot)]
                                            if load_buffer == "scratch" && load_slot == "slot"
                                    )
                        )
                        && matches!(rhs.as_ref(), AstExpr::Var(step) if step == "step")
                )
    ));
}

#[test]
fn lowers_compound_pointer_value_assignment_into_store_value_sugar() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(head: ref Node) -> i64 {
            head.value %= 2;
            return head.value;
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
        Some(NirStmt::Expr(NirExpr::StoreValue { target, value }))
            if matches!(target.as_ref(), NirExpr::Var(name) if name == "head")
                && matches!(
                    value.as_ref(),
                    NirExpr::Binary {
                        op,
                        lhs,
                        rhs,
                    } if *op == NirBinaryOp::Rem
                        && matches!(lhs.as_ref(), NirExpr::LoadValue(inner) if matches!(inner.as_ref(), NirExpr::Var(name) if name == "head"))
                        && matches!(rhs.as_ref(), NirExpr::Int(2))
                )
    ));
}

#[test]
fn rejects_compound_pointer_next_assignment() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main(head: ref Node, next: ref Node) -> i64 {
            head.next += next;
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("compound assignment target `.next` is not supported yet"),
        "{error}"
    );
}

#[test]
fn rejects_private_local_cpu_helper_calls_across_modules() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_completed(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("unknown function `task_policy_completed`"),
        "unexpected error: {error}"
    );
}

#[test]
fn suggests_similar_local_function_name_for_unknown_call() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }

          fn main() -> i64 {
            return task_policy_complted(7);
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("unknown function `task_policy_complted`"),
        "{error}"
    );
    assert!(
        error.contains("did you mean `task_policy_completed`?"),
        "{error}"
    );
}

#[test]
fn suggests_similar_imported_helper_function_name_for_unknown_call() {
    let entry = parse_nuis_ast(
        r#"
        use cpu TaskHelpers;

        mod cpu Main {
          fn main() -> i64 {
            return task_policy_complted(7);
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu TaskHelpers {
          pub fn task_policy_completed(value: i64) -> i64 {
            return value + 1;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("unknown function `task_policy_complted`"),
        "{error}"
    );
    assert!(
        error.contains("did you mean `task_policy_completed`?"),
        "{error}"
    );
}

#[test]
fn lowers_non_numeric_binary_add_via_addable_impl() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for Pair {
            fn add(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value + rhs.value };
            }
          }

          fn main() -> i64 {
            let sum: Pair = Pair { value: 1 } + Pair { value: 2 };
            return sum.value;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "sum" && callee == "impl.Addable.for.Pair.add"
    ));
}

#[test]
fn lowers_non_numeric_binary_sub_mul_div_via_trait_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Subtractable {
            fn sub(lhs: Self, rhs: Self) -> Self;
          }

          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          trait Dividable {
            fn div(lhs: Self, rhs: Self) -> Self;
          }

          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Subtractable for Pair {
            fn sub(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value - rhs.value };
            }
          }

          impl Multipliable for Pair {
            fn mul(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value * rhs.value };
            }
          }

          impl Dividable for Pair {
            fn div(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value / rhs.value };
            }
          }

          impl Remainderable for Pair {
            fn rem(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value % rhs.value };
            }
          }

          fn main() -> i64 {
            let diff: Pair = Pair { value: 6 } - Pair { value: 2 };
            let prod: Pair = Pair { value: 3 } * Pair { value: 4 };
            let quot: Pair = Pair { value: 8 } / Pair { value: 2 };
            let rest: Pair = Pair { value: 9 } % Pair { value: 4 };
            return diff.value + prod.value + quot.value + rest.value;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "diff" && callee == "impl.Subtractable.for.Pair.sub"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "prod" && callee == "impl.Multipliable.for.Pair.mul"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "quot" && callee == "impl.Dividable.for.Pair.div"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "rest" && callee == "impl.Remainderable.for.Pair.rem"
    ));
}

#[test]
fn parses_binary_operator_precedence_in_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            return 1 + 2 * 3;
          }
        }
        "#,
    )
    .unwrap();

    let main = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(AstStmt::Return(Some(AstExpr::Binary { op, lhs, rhs })))
            if *op == AstBinaryOp::Add
                && matches!(lhs.as_ref(), AstExpr::Int(1))
                && matches!(
                    rhs.as_ref(),
                    AstExpr::Binary {
                        op: inner_op,
                        lhs: inner_lhs,
                        rhs: inner_rhs,
                    } if *inner_op == AstBinaryOp::Mul
                        && matches!(inner_lhs.as_ref(), AstExpr::Int(2))
                        && matches!(inner_rhs.as_ref(), AstExpr::Int(3))
                )
    ));
}

#[test]
fn parses_logical_and_comparison_precedence_in_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> bool {
            return 1 == 1 || 2 == 3 && 4 < 5;
          }
        }
        "#,
    )
    .unwrap();

    let main = ast
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(AstStmt::Return(Some(AstExpr::Binary { op, lhs, rhs })))
            if *op == AstBinaryOp::Or
                && matches!(
                    lhs.as_ref(),
                    AstExpr::Binary {
                        op: lhs_op,
                        lhs: lhs_lhs,
                        rhs: lhs_rhs,
                    } if *lhs_op == AstBinaryOp::Eq
                        && matches!(lhs_lhs.as_ref(), AstExpr::Int(1))
                        && matches!(lhs_rhs.as_ref(), AstExpr::Int(1))
                )
                && matches!(
                    rhs.as_ref(),
                    AstExpr::Binary {
                        op: rhs_op,
                        lhs: rhs_lhs,
                        rhs: rhs_rhs,
                    } if *rhs_op == AstBinaryOp::And
                        && matches!(
                            rhs_lhs.as_ref(),
                            AstExpr::Binary {
                                op: inner_eq_op,
                                lhs: inner_eq_lhs,
                                rhs: inner_eq_rhs,
                            } if *inner_eq_op == AstBinaryOp::Eq
                                && matches!(inner_eq_lhs.as_ref(), AstExpr::Int(2))
                                && matches!(inner_eq_rhs.as_ref(), AstExpr::Int(3))
                        )
                        && matches!(
                            rhs_rhs.as_ref(),
                            AstExpr::Binary {
                                op: inner_lt_op,
                                lhs: inner_lt_lhs,
                                rhs: inner_lt_rhs,
                            } if *inner_lt_op == AstBinaryOp::Lt
                                && matches!(inner_lt_lhs.as_ref(), AstExpr::Int(4))
                                && matches!(inner_lt_rhs.as_ref(), AstExpr::Int(5))
                        )
                )
    ));
}

#[test]
fn lowers_logical_and_comparison_precedence_in_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let ready: bool = 1 == 1 || 2 == 3 && 4 < 5;
            if ready {
              return 1;
            }
            return 0;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "ready"
            && *op == NirBinaryOp::Or
            && matches!(
                lhs.as_ref(),
                NirExpr::Binary {
                    op: lhs_op,
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                } if *lhs_op == NirBinaryOp::Eq
                    && matches!(lhs_lhs.as_ref(), NirExpr::Int(1))
                    && matches!(lhs_rhs.as_ref(), NirExpr::Int(1))
            )
            && matches!(
                rhs.as_ref(),
                NirExpr::Binary {
                    op: rhs_op,
                    lhs: rhs_lhs,
                    rhs: rhs_rhs,
                } if *rhs_op == NirBinaryOp::And
                    && matches!(
                        rhs_lhs.as_ref(),
                        NirExpr::Binary {
                            op: inner_eq_op,
                            lhs: inner_eq_lhs,
                            rhs: inner_eq_rhs,
                        } if *inner_eq_op == NirBinaryOp::Eq
                            && matches!(inner_eq_lhs.as_ref(), NirExpr::Int(2))
                            && matches!(inner_eq_rhs.as_ref(), NirExpr::Int(3))
                    )
                    && matches!(
                        rhs_rhs.as_ref(),
                        NirExpr::Binary {
                            op: inner_lt_op,
                            lhs: inner_lt_lhs,
                            rhs: inner_lt_rhs,
                        } if *inner_lt_op == NirBinaryOp::Lt
                            && matches!(inner_lt_lhs.as_ref(), NirExpr::Int(4))
                            && matches!(inner_lt_rhs.as_ref(), NirExpr::Int(5))
                    )
            )
    ));
}

#[test]
fn lowers_parenthesized_logical_precedence_in_nir() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let grouped: bool = (1 == 1 || 2 == 3) && 4 < 5;
            if grouped {
              return 1;
            }
            return 0;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "grouped"
            && *op == NirBinaryOp::And
            && matches!(
                lhs.as_ref(),
                NirExpr::Binary {
                    op: lhs_op,
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                } if *lhs_op == NirBinaryOp::Or
                    && matches!(
                        lhs_lhs.as_ref(),
                        NirExpr::Binary { op: inner_op, .. } if *inner_op == NirBinaryOp::Eq
                    )
                    && matches!(
                        lhs_rhs.as_ref(),
                        NirExpr::Binary { op: inner_op, .. } if *inner_op == NirBinaryOp::Eq
                    )
            )
            && matches!(
                rhs.as_ref(),
                NirExpr::Binary { op: rhs_op, .. } if *rhs_op == NirBinaryOp::Lt
            )
    ));
}

#[test]
fn lowers_overloaded_binary_precedence_without_parentheses() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for Pair {
            fn add(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value + rhs.value };
            }
          }

          impl Multipliable for Pair {
            fn mul(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value * rhs.value };
            }
          }

          fn main() -> i64 {
            let mixed: Pair = Pair { value: 1 } + Pair { value: 2 } * Pair { value: 3 };
            return mixed.value;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "mixed"
            && callee == "impl.Addable.for.Pair.add"
            && matches!(
                args.as_slice(),
                [
                    NirExpr::StructLiteral { fields: lhs_fields, .. },
                    NirExpr::Call { callee: rhs_callee, args: rhs_args }
                ] if matches!(lhs_fields.as_slice(), [(field, NirExpr::Int(1))] if field == "value")
                    && rhs_callee == "impl.Multipliable.for.Pair.mul"
                    && matches!(
                        rhs_args.as_slice(),
                        [
                            NirExpr::StructLiteral { fields: mul_lhs_fields, .. },
                            NirExpr::StructLiteral { fields: mul_rhs_fields, .. }
                        ] if matches!(mul_lhs_fields.as_slice(), [(field, NirExpr::Int(2))] if field == "value")
                            && matches!(mul_rhs_fields.as_slice(), [(field, NirExpr::Int(3))] if field == "value")
                    )
            )
    ));
}

#[test]
fn lowers_builtin_remainder_with_multiplicative_precedence() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 9 % 4 * 2;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary { op, lhs, rhs },
            ..
        }) if name == "value"
            && *op == NirBinaryOp::Mul
            && matches!(
                lhs.as_ref(),
                NirExpr::Binary {
                    op: lhs_op,
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                } if *lhs_op == NirBinaryOp::Rem
                    && matches!(lhs_lhs.as_ref(), NirExpr::Int(9))
                    && matches!(lhs_rhs.as_ref(), NirExpr::Int(4))
            )
            && matches!(rhs.as_ref(), NirExpr::Int(2))
    ));
}

#[test]
fn lowers_overloaded_binary_remainder_via_trait_impl() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Remainderable for Pair {
            fn rem(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value % rhs.value };
            }
          }

          fn main() -> i64 {
            let rest: Pair = Pair { value: 9 } % Pair { value: 4 };
            return rest.value;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "rest" && callee == "impl.Remainderable.for.Pair.rem"
    ));
}

#[test]
fn lowers_overloaded_binary_parentheses_and_left_associativity() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          trait Subtractable {
            fn sub(lhs: Self, rhs: Self) -> Self;
          }

          trait Multipliable {
            fn mul(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for Pair {
            fn add(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value + rhs.value };
            }
          }

          impl Subtractable for Pair {
            fn sub(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value - rhs.value };
            }
          }

          impl Multipliable for Pair {
            fn mul(lhs: Pair, rhs: Pair) -> Pair {
              return Pair { value: lhs.value * rhs.value };
            }
          }

          fn main() -> i64 {
            let grouped: Pair = (Pair { value: 1 } + Pair { value: 2 }) * Pair { value: 3 };
            let folded: Pair = Pair { value: 9 } - Pair { value: 3 } - Pair { value: 1 };
            return grouped.value + folded.value;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "grouped"
            && callee == "impl.Multipliable.for.Pair.mul"
            && matches!(
                args.as_slice(),
                [
                    NirExpr::Call { callee: lhs_callee, .. },
                    NirExpr::StructLiteral { fields: rhs_fields, .. }
                ] if lhs_callee == "impl.Addable.for.Pair.add"
                    && matches!(rhs_fields.as_slice(), [(field, NirExpr::Int(3))] if field == "value")
            )
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, args },
            ..
        }) if name == "folded"
            && callee == "impl.Subtractable.for.Pair.sub"
            && matches!(
                args.as_slice(),
                [
                    NirExpr::Call { callee: lhs_callee, args: lhs_args },
                    NirExpr::StructLiteral { fields: rhs_fields, .. }
                ] if lhs_callee == "impl.Subtractable.for.Pair.sub"
                    && matches!(
                        lhs_args.as_slice(),
                        [
                            NirExpr::StructLiteral { fields: first_fields, .. },
                            NirExpr::StructLiteral { fields: second_fields, .. }
                        ] if matches!(first_fields.as_slice(), [(field, NirExpr::Int(9))] if field == "value")
                            && matches!(second_fields.as_slice(), [(field, NirExpr::Int(3))] if field == "value")
                    )
                    && matches!(rhs_fields.as_slice(), [(field, NirExpr::Int(1))] if field == "value")
            )
    ));
}

#[test]
fn lowers_non_numeric_binary_comparisons_via_trait_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Equatable {
            fn eq(lhs: Self, rhs: Self) -> bool;
          }

          trait Orderable {
            fn lt(lhs: Self, rhs: Self) -> bool;
            fn le(lhs: Self, rhs: Self) -> bool;
            fn gt(lhs: Self, rhs: Self) -> bool;
            fn ge(lhs: Self, rhs: Self) -> bool;
          }

          impl Equatable for Pair {
            fn eq(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value == rhs.value;
            }
          }

          impl Orderable for Pair {
            fn lt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value < rhs.value;
            }

            fn le(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value <= rhs.value;
            }

            fn gt(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value > rhs.value;
            }

            fn ge(lhs: Pair, rhs: Pair) -> bool {
              return lhs.value >= rhs.value;
            }
          }

          fn main() -> i64 {
            let same: bool = Pair { value: 1 } == Pair { value: 1 };
            let different: bool = Pair { value: 1 } != Pair { value: 2 };
            let less: bool = Pair { value: 1 } < Pair { value: 2 };
            let less_eq: bool = Pair { value: 1 } <= Pair { value: 2 };
            let greater: bool = Pair { value: 3 } > Pair { value: 2 };
            let greater_eq: bool = Pair { value: 3 } >= Pair { value: 2 };
            if same && different && less && less_eq && greater && greater_eq {
              return 1;
            }
            return 0;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "same" && callee == "impl.Equatable.for.Pair.eq"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs,
                rhs,
            },
            ..
        }) if name == "different"
            && matches!(lhs.as_ref(), NirExpr::Call { callee, .. } if callee == "impl.Equatable.for.Pair.eq")
            && matches!(rhs.as_ref(), NirExpr::Bool(false))
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "less" && callee == "impl.Orderable.for.Pair.lt"
    ));
    assert!(matches!(
        main.body.get(3),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "less_eq" && callee == "impl.Orderable.for.Pair.le"
    ));
    assert!(matches!(
        main.body.get(4),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "greater" && callee == "impl.Orderable.for.Pair.gt"
    ));
    assert!(matches!(
        main.body.get(5),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "greater_eq" && callee == "impl.Orderable.for.Pair.ge"
    ));
}

#[test]
fn lowers_builtin_unary_not_and_neg() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let toggled: bool = !false;
            let negated: i64 = -7;
            if toggled {
              return negated;
            }
            return 0;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs,
                rhs,
            },
            ..
        }) if name == "toggled"
            && matches!(lhs.as_ref(), NirExpr::Bool(false))
            && matches!(rhs.as_ref(), NirExpr::Bool(false))
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Binary {
                op: NirBinaryOp::Sub,
                lhs,
                rhs,
            },
            ..
        }) if name == "negated"
            && matches!(lhs.as_ref(), NirExpr::Int(0))
            && matches!(rhs.as_ref(), NirExpr::Int(7))
    ));
}

#[test]
fn lowers_non_builtin_unary_not_and_neg_via_trait_impls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            value: i64,
          }

          trait Notable {
            fn not(value: Self) -> bool;
          }

          trait Negatable {
            fn neg(value: Self) -> Self;
          }

          impl Notable for Pair {
            fn not(value: Pair) -> bool {
              return value.value == 0;
            }
          }

          impl Negatable for Pair {
            fn neg(value: Pair) -> Pair {
              return Pair { value: 0 - value.value };
            }
          }

          fn main() -> i64 {
            let empty: bool = !Pair { value: 0 };
            let flipped: Pair = -Pair { value: 7 };
            if empty {
              return flipped.value;
            }
            return 0;
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
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "empty" && callee == "impl.Notable.for.Pair.not"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        }) if name == "flipped" && callee == "impl.Negatable.for.Pair.neg"
    ));
}

#[test]
fn lowers_project_local_cpu_helper_calls_with_shader_and_data_modules_present() {
    let entry = parse_nuis_ast(
        r#"
        use cpu ShaderTaskAsyncShapes;
        use data FabricPlane;
        use shader SurfaceShader;

        mod cpu Main {
          fn main(primary_result: TaskResult<i64>, secondary_result: TaskResult<i64>) -> i64 {
            return ShaderTaskAsyncShapes.async_policy_summary_completed(
              primary_result,
              secondary_result
            );
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu ShaderTaskAsyncShapes {
          pub fn encode_completed(result: TaskResult<i64>) -> i64 {
            if task_completed(result) {
              return 1;
            }
            return 0;
          }

          pub fn async_policy_summary_completed(
            primary_result: TaskResult<i64>,
            secondary_result: TaskResult<i64>
          ) -> i64 {
            return encode_completed(primary_result) + encode_completed(secondary_result);
          }
        }
        "#,
    )
    .unwrap();
    let data_module = parse_nuis_ast(
        r#"
        mod data FabricPlane {
          struct SurfaceShaderPacket {
            color: i64,
          }
        }
        "#,
    )
    .unwrap();
    let shader_module = parse_nuis_ast(
        r#"
        mod shader SurfaceShader {
          struct SurfaceShaderPacket {
            color: i64,
          }
        }
        "#,
    )
    .unwrap();

    let module =
        super::lower_project_ast_to_nir(&entry, &[helper, data_module, shader_module]).unwrap();
    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main_function.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "ShaderTaskAsyncShapes.async_policy_summary_completed"
    ));
}

#[test]
fn lowers_real_shader_project_helper_calls_from_disk() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/projects/domains/shader_async_policy_profile_demo");
    let shared_root = root.join("../shared");
    let entry = parse_nuis_ast(&fs::read_to_string(root.join("main.ns")).unwrap()).unwrap();
    let shader_module =
        parse_nuis_ast(&fs::read_to_string(root.join("surface_shader.ns")).unwrap()).unwrap();
    let data_module =
        parse_nuis_ast(&fs::read_to_string(root.join("fabric_plane.ns")).unwrap()).unwrap();
    let helper = parse_nuis_ast(
        &fs::read_to_string(shared_root.join("shader_task_async_shapes.ns")).unwrap(),
    )
    .unwrap();

    let module =
        super::lower_project_ast_to_nir(&entry, &[shader_module, data_module, helper]).unwrap();
    let main_function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(main_function.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::Call { callee, .. },
                ..
            } if callee == "ShaderTaskAsyncShapes.async_policy_summary_completed"
        )
    }));
}

#[test]
fn rejects_private_helper_field_access_across_modules() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Shapes;

        mod cpu Main {
          fn main() -> i64 {
            let cfg: Config = Shapes.make();
            return cfg.secret;
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu Shapes {
          pub struct Config {
            pub visible: i64,
            secret: i64
          }

          pub fn make() -> Config {
            return Config {
              visible: 1,
              secret: 2
            };
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("type `Config` has no field `secret`"),
        "unexpected error: {error}"
    );
}

#[test]
fn suggests_similar_visible_field_name_for_field_access() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Config {
            count: i64,
            label: String,
          }

          fn main() -> i64 {
            let cfg: Config = Config { count: 7, label: "ok" };
            return cfg.cout;
          }
        }
        "#,
    );

    let error = module.unwrap_err();
    assert!(
        error.contains("type `Config` has no field `cout`"),
        "{error}"
    );
    assert!(error.contains("did you mean `count`?"), "{error}");
}

#[test]
fn rejects_struct_literals_for_imported_structs_with_hidden_private_fields() {
    let entry = parse_nuis_ast(
        r#"
        use cpu Shapes;

        mod cpu Main {
          fn main() -> i64 {
            let cfg: Config = Config {
              visible: 1
            };
            return cfg.visible;
          }
        }
        "#,
    )
    .unwrap();
    let helper = parse_nuis_ast(
        r#"
        mod cpu Shapes {
          pub struct Config {
            pub visible: i64,
            secret: i64
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
    assert!(
        error.contains("struct literal `Config` cannot be constructed outside its defining module because it hides 1 private field"),
        "unexpected error: {error}"
    );
}

#[test]
fn parses_pub_const_items_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          pub const LIMIT: i64 = 7;

          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(ast.consts.len(), 1);
    assert!(matches!(ast.consts[0].visibility, AstVisibility::Public));
    assert_eq!(ast.consts[0].name, "LIMIT");
    assert_eq!(
        ast.consts[0]
            .ty
            .as_ref()
            .map(|ty| lower_type_ref(ty).render())
            .as_deref(),
        Some("i64")
    );
}

#[test]
fn parses_top_level_const_items_without_explicit_type() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          const LIMIT = 7;
        }
        "#,
    )
    .unwrap();
    assert_eq!(ast.consts.len(), 1);
    assert!(ast.consts[0].ty.is_none());
}

#[test]
fn lowers_top_level_const_reads_by_inlining_values() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          const LIMIT: i64 = 7;

          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(module.consts.len(), 1);
    assert!(matches!(
        module.functions[0].body.first(),
        Some(NirStmt::Return(Some(NirExpr::Int(7))))
    ));
}

#[test]
fn infers_top_level_const_item_types_from_values() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          const LIMIT = 7;

          fn main() -> i64 {
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    assert_eq!(module.consts.len(), 1);
    assert_eq!(module.consts[0].ty.render(), "i64");
    assert!(matches!(
        module.functions[0].body.first(),
        Some(NirStmt::Return(Some(NirExpr::Int(7))))
    ));
}

#[test]
fn parses_local_const_without_explicit_type() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            const LIMIT = 7;
            return LIMIT;
          }
        }
        "#,
    )
    .unwrap();
    match &ast.functions[0].body[0] {
        AstStmt::Const { ty, .. } => assert!(ty.is_none()),
        other => panic!("expected local const statement, found {other:?}"),
    }
}

#[test]
fn infers_local_const_item_types_inside_branches() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            if true {
              const LIMIT = 7;
              return LIMIT;
            } else {
              match 1 {
                1 => {
                  const LIMIT = 8;
                  return LIMIT;
                }
                _ => {
                  return 9;
                }
              }
            }
          }
        }
        "#,
    )
    .unwrap();
    match &module.functions[0].body[0] {
        NirStmt::If {
            then_body,
            else_body,
            ..
        } => {
            match &then_body[0] {
                NirStmt::Const { ty, .. } => assert_eq!(ty.render(), "i64"),
                other => panic!("expected inferred const in then branch, found {other:?}"),
            }
            match &else_body[0] {
                NirStmt::If { then_body, .. } => match &then_body[0] {
                    NirStmt::Const { ty, .. } => assert_eq!(ty.render(), "i64"),
                    other => {
                        panic!("expected inferred const in match arm branch, found {other:?}")
                    }
                },
                other => panic!("expected lowered match branch if, found {other:?}"),
            }
        }
        other => panic!("expected if statement, found {other:?}"),
    }
}

#[test]
fn parses_struct_destructuring_let_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Packet {
            kind: i64,
            ready: bool,
          }

          fn main() -> i64 {
            let packet: Packet = Packet { kind: 7, ready: true };
            let Packet { kind, ready } = packet;
            if ready {
              return kind;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    match &ast.functions[0].body[1] {
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => {
            assert_eq!(type_ref.as_ref().unwrap().name, "Packet");
            assert_eq!(
                fields,
                &vec![
                    AstDestructureField {
                        field: "kind".to_owned(),
                        binding: AstDestructureBinding::Bind("kind".to_owned())
                    },
                    AstDestructureField {
                        field: "ready".to_owned(),
                        binding: AstDestructureBinding::Bind("ready".to_owned())
                    }
                ]
            );
            assert!(matches!(value, nuis_semantics::model::AstExpr::Var(name) if name == "packet"));
        }
        other => panic!("expected destructuring let statement, found {other:?}"),
    }
}

#[test]
fn lowers_explicit_trait_qualified_call_to_impl_symbol() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.add(7, 8);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Int(7), NirExpr::Int(8)])
    ));
}

#[test]
fn lowers_explicit_trait_qualified_call_with_public_helper_trait() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.add(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Int(7), NirExpr::Int(8)])
    ));
}

#[test]
fn lowers_explicit_trait_qualified_call_to_default_impl_symbol() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn zero(seed: Self) -> Self {
              return Addable.add(seed, 0);
            }
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.zero(7);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.zero"
                && matches!(args.as_slice(), [NirExpr::Int(7)])
    ));
}

#[test]
fn lowers_synthesized_default_impl_body_via_concrete_trait_impl_calls() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;

            fn zero(seed: Self) -> Self {
              return Addable.add(seed, 0);
            }
          }

          impl Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Addable.zero(7);
          }
        }
        "#,
    )
    .unwrap();

    let synthesized = module
        .functions
        .iter()
        .find(|function| function.name == "impl.Addable.for.i64.zero")
        .unwrap();
    assert!(matches!(
        synthesized.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Var(seed), NirExpr::Int(0)] if seed == "seed")
    ));
}

#[test]
fn lowers_zero_arg_explicit_trait_qualified_call_via_explicit_self_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn zero() -> Self {
              return 0;
            }
          }

          impl Addable for i64 {
          }

          fn main() -> i64 {
            return Addable.zero<i64>();
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Addable.for.i64.zero" && args.is_empty()
    ));
}

#[test]
fn lowers_zero_arg_explicit_trait_qualified_call_via_expected_self_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn zero() -> Self {
              return 0;
            }
          }

          impl Addable for i64 {
          }

          fn main() -> i64 {
            let value: i64 = Addable.zero();
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
            if name == "value"
                && matches!(value, NirExpr::Call { callee, args } if callee == "impl.Addable.for.i64.zero" && args.is_empty())
    ));
}

#[test]
fn rejects_zero_arg_explicit_trait_qualified_call_without_self_anchor() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Addable {
            fn zero() -> Self {
              return 0;
            }
          }

          impl Addable for i64 {
          }

          fn main() -> i64 {
            Addable.zero();
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("trait method `Addable.zero` without receiver argument cannot infer `Self`"),
        "{error}"
    );
    assert!(error.contains("Addable.zero<Type>()"), "{error}");
    assert!(error.contains("expected return type"), "{error}");
}

#[test]
fn lowers_fully_qualified_helper_trait_call_to_impl_symbol() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Helper.Addable.add(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let module = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap();
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        main.body.first(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "impl.Helper.Addable.for.i64.add"
                && matches!(args.as_slice(), [NirExpr::Int(7), NirExpr::Int(8)])
    ));
}

#[test]
fn reports_missing_impl_for_explicit_qualified_trait_call_on_concrete_type() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          fn main() -> i64 {
            return Helper.Addable.add(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(
        error.contains("trait `Helper.Addable` has no impl for `i64`"),
        "{error}"
    );
    assert!(error.contains("Helper.Addable.add"), "{error}");
}

#[test]
fn suggests_trait_method_name_for_explicit_qualified_trait_call() {
    let main_ast = parse_nuis_ast(
        r#"
        use cpu Helper;

        mod cpu Main {
          impl Helper.Addable for i64 {
            fn add(lhs: i64, rhs: i64) -> i64 {
              return lhs + rhs;
            }
          }

          fn main() -> i64 {
            return Helper.Addable.ad(7, 8);
          }
        }
        "#,
    )
    .unwrap();
    let helper_ast = parse_nuis_ast(
        r#"
        mod cpu Helper {
          pub trait Addable {
            fn add(lhs: Self, rhs: Self) -> Self;
          }
        }
        "#,
    )
    .unwrap();

    let error = super::lower_project_ast_to_nir(&main_ast, &[helper_ast]).unwrap_err();
    assert!(error.contains("does not define method `ad`"), "{error}");
    assert!(
        error.contains("did you mean `Helper.Addable.add`?"),
        "{error}"
    );
}
