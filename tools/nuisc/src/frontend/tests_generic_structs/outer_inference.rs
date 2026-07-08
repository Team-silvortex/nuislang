use super::*;

#[test]
fn rejects_generic_alias_struct_literal_when_fields_do_not_fully_determine_type_args() {
    let error = parse_nuis_module(
        r#"
        mod cpu Main {
          type PhantomAlias<T, U> = Phantom<T, U>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          fn main() -> i64 {
            let phantom = PhantomAlias { value: 7, tag: 1 };
            return 0;
          }
        }
        "#,
    )
    .unwrap_err();

    assert!(
        error.contains(
            "generic alias constructor `PhantomAlias` could not infer generic parameter `U` for target `Phantom<T, U>`; add explicit type arguments or a stronger expected type"
        ),
        "{error}"
    );
}

#[test]
fn lowers_outer_generic_struct_literal_when_later_field_completes_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let outer = Outer {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Outer<i64, String>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Outer"
                    && matches!(type_args.as_slice(), [lhs, rhs] if lhs.render() == "i64" && rhs.render() == "String")
            ));
        }
        other => panic!("expected inferred outer generic struct let, found {other:?}"),
    }
}

#[test]
fn lowers_outer_generic_struct_literal_when_later_field_completes_inner_payload_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T, U> {
            value: T,
          }

          struct Outer<T, U> {
            inner: Just<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let outer = Outer {
              inner: Just(7),
              meta: "ok",
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Outer<i64, String>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Outer"
                    && matches!(type_args.as_slice(), [lhs, rhs] if lhs.render() == "i64" && rhs.render() == "String")
            ));
        }
        other => {
            panic!("expected inferred outer generic struct let from payload route, found {other:?}")
        }
    }
}

#[test]
fn lowers_transparent_alias_outer_literal_when_later_field_completes_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Outer<T, U>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          fn main() -> i64 {
            let outer = OuterAlias {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            };
            return outer.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Outer<i64, String>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Outer"
                    && matches!(type_args.as_slice(), [lhs, rhs] if lhs.render() == "i64" && rhs.render() == "String")
            ));
        }
        other => panic!("expected inferred transparent alias outer let, found {other:?}"),
    }
}

#[test]
fn lowers_non_transparent_alias_outer_literal_when_later_field_completes_inner_inference() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type OuterAlias<T, U> = Wrapper<Outer<T, U>>;

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          struct Outer<T, U> {
            inner: Phantom<T, U>,
            meta: U,
          }

          struct Wrapper<T> {
            inner: T,
            mark: i64,
          }

          fn main() -> i64 {
            let outer = OuterAlias {
              inner: Outer {
                inner: Phantom { value: 7, tag: 1 },
                meta: "ok",
              },
              mark: 1,
            };
            return outer.inner.inner.value;
          }
        }
        "#,
    )
    .unwrap();

    match &module.functions[0].body[0] {
        NirStmt::Let { name, ty, value } => {
            assert_eq!(name, "outer");
            assert_eq!(ty.as_ref().unwrap().render(), "Wrapper<Outer<i64, String>>");
            assert!(matches!(
                value,
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    ..
                } if type_name == "Wrapper"
                    && matches!(type_args.as_slice(), [inner] if inner.render() == "Outer<i64, String>")
            ));
        }
        other => panic!("expected inferred non-transparent alias outer let, found {other:?}"),
    }
}
