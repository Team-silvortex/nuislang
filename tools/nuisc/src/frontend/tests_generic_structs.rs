use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

#[test]
fn parses_generic_struct_definition_into_ast() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }
        }
        "#,
    )
    .unwrap();

    let definition = &ast.structs[0];
    assert_eq!(definition.name, "Boxed");
    assert_eq!(definition.generic_params.len(), 1);
    assert_eq!(definition.generic_params[0].name, "T");
    assert!(definition.generic_params[0].bound.is_none());
    assert_eq!(definition.fields[0].ty.name, "T");
}

#[test]
fn lowers_generic_struct_literal_with_expected_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_explicit_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed<i64> { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected explicit generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = Boxed { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic struct let, found {other:?}"),
    }
}

#[test]
fn rejects_generic_struct_literal_with_conflicting_inferred_type_args() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair<T> {
            left: T,
            right: T,
          }

          fn main() -> i64 {
            let pair = Pair { left: 7, right: "hi" };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "struct literal `Pair` inferred conflicting types `i64` and `String` for generic parameter `T`"
        ),
        "{error}"
    );
}

#[test]
fn lowers_nested_generic_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          struct Wrapper<T> {
            inner: Boxed<T>,
            tag: i64,
          }

          fn main() -> i64 {
            let wrapped = Wrapper {
              inner: Boxed { value: 7 },
              tag: 1,
            };
            return wrapped.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "wrapped");
            assert_eq!(ty.as_ref().unwrap().render(), "Wrapper<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Wrapper"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
            ));
        }
        other => panic!("expected inferred nested generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_multi_field_generic_struct_literal_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair<T, U> {
            left: T,
            right: U,
          }

          fn main() -> i64 {
            let pair = Pair { left: 7, right: 9 };
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { ty: Some(ty), .. } => {
            assert_eq!(ty.render(), "Pair<i64, i64>");
        }
        other => panic!("expected inferred pair generic struct let, found {other:?}"),
    }
}

#[test]
fn rejects_generic_struct_literal_when_fields_do_not_fully_determine_type_args() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          fn main() -> i64 {
            let phantom = Phantom { value: 7, tag: 1 };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "cannot infer generic arguments for struct literal `Phantom` in the current frontend; add an explicit expected type"
        ),
        "{error}"
    );
}

#[test]
fn lowers_generic_struct_destructuring_let() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: 7 };
            let Boxed<i64> { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let_with_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxI64 = Boxed<i64>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: BoxI64 = Boxed<i64> { value: 7 };
            let BoxI64 { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered aliased generic destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_destructuring_let_with_generic_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed: BoxAlias<i64> = Boxed<i64> { value: 7 };
            let BoxAlias<i64> { value } = boxed;
            return value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "value");
            assert_eq!(ty.as_ref().unwrap().render(), "i64");
            assert!(matches!(
                value,
                NirExpr::FieldAccess { field, .. } if field == "value"
            ));
        }
        other => panic!("expected lowered generic-aliased destructured field let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_struct_literal_with_inferred_generic_alias_type_head() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = BoxAlias { value: 7 };
            return boxed.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "boxed");
            assert_eq!(ty.as_ref().unwrap().render(), "Boxed<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Boxed"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic-alias struct let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_expected_type() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload: Just<i64> = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected lowered generic payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_explicit_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = Just<i64>(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected explicit generic payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_inferred_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = Just(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_generic_alias_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = JustAlias<i64>(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected lowered generic-alias payload constructor let, found {other:?}"),
    }
}

#[test]
fn lowers_generic_payload_struct_constructor_with_inferred_generic_alias_type_args() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = JustAlias(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<i64>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "i64")
                    && matches!(
                        fields.as_slice(),
                        [(field, NirExpr::Int(7))] if field == "value"
                    )
            ));
        }
        other => panic!("expected inferred generic-alias payload constructor let, found {other:?}"),
    }
}

#[test]
fn rejects_payload_style_constructor_with_wrong_generic_alias_arity() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = JustAlias<i64, bool>(7);
            return payload.value;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains("type alias `JustAlias` expects 1 generic argument(s), found 2"),
        "{error}"
    );
}

#[test]
fn lowers_payload_style_constructor_for_non_transparent_generic_alias_when_inferable() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedAlias<T> = Just<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = WrappedAlias(Boxed { value: 7 });
            return payload.value.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "payload");
            assert_eq!(ty.as_ref().unwrap().render(), "Just<Boxed<i64>>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Just"
                    && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
            ));
        }
        other => panic!(
            "expected inferred non-transparent generic-alias payload constructor let, found {other:?}"
        ),
    }
}

#[test]
fn reports_non_transparent_alias_payload_constructor_shape_mismatch_clearly() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedAlias<T> = Just<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Just<T> {
            value: T,
          }

          fn main() -> i64 {
            let payload = WrappedAlias(7);
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "generic alias constructor `WrappedAlias` could not match target field shape `Boxed<T>` with concrete type `i64`"
        ),
        "{error}"
    );
}

#[test]
fn reports_alias_constructor_missing_generic_parameter_clearly() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type PhantomAlias<T, U> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn main() -> i64 {
            let boxed = PhantomAlias { value: 7 };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "generic alias constructor `PhantomAlias` could not infer generic parameter `U` for target `Boxed<T>`"
        ),
        "{error}"
    );
}
