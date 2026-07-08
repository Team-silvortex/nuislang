use super::*;

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
        main.body.first(),
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
