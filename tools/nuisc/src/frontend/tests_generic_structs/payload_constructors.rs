use super::*;

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
fn lowers_generic_alias_payload_constructor_from_alias_field_access() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;
          type PacketAlias<T> = Packet<T>;

          struct Just<T> {
            value: T,
          }

          struct Packet<T> {
            payload: T,
            tag: i64,
          }

          fn main() -> i64 {
            let packet: PacketAlias<i64> = PacketAlias { payload: 7, tag: 1 };
            let payload = JustAlias(packet.payload);
            return payload.value + packet.tag;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[1] {
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
                        [(field, NirExpr::FieldAccess { field: payload_field, .. })]
                            if field == "value" && payload_field == "payload"
                    )
            ));
        }
        other => panic!(
            "expected alias-field-access generic-alias payload constructor let, found {other:?}"
        ),
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
