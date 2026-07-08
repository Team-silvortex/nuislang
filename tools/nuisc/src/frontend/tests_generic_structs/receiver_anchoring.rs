use super::*;

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_struct_literal() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Phantom<T, U> {
            value: T,
            tag: i64,
          }

          impl<T, U> Showable for Phantom<T, U> {
            fn show(value: Phantom<T, U>) -> i64 {
              return value.tag;
            }
          }

          fn main() -> i64 {
            return Phantom { value: 7, tag: 1 }.show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Phantom")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }]
                        if type_name == "Phantom"
                            && matches!(type_args.as_slice(), [first, second] if first.render() == "i64" && second.render() == "bool")
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_payload_constructor() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn main() -> i64 {
            return Carrier(7).show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }]
                        if type_name == "Carrier"
                            && matches!(type_args.as_slice(), [first, second] if first.render() == "i64" && second.render() == "bool")
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_anchoring_payload_alias_constructor() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          type CarrierAlias<T, U> = Carrier<T, U>;

          struct Carrier<T, U> {
            value: T,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn main() -> i64 {
            return CarrierAlias(7).show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }]
                        if type_name == "Carrier"
                            && matches!(type_args.as_slice(), [first, second] if first.render() == "i64" && second.render() == "bool")
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_generic_helper_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn make_carrier<T, U>(value: T) -> Carrier<T, U> {
            return Carrier(value);
          }

          fn main() -> i64 {
            return make_carrier(7).show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::Call { callee: helper, .. }]
                        if helper == "make_carrier__i64__bool"
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_helper_field_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          struct Wrapper<T, U> {
            inner: Carrier<T, U>,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn make_carrier<T, U>(value: T) -> Carrier<T, U> {
            return Carrier(value);
          }

          fn wrap<T, U>(inner: Carrier<T, U>) -> Wrapper<T, U> {
            return Wrapper { inner: inner };
          }

          fn main() -> i64 {
            return wrap(make_carrier(7)).inner.show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::FieldAccess { base, field }]
                        if field == "inner"
                            && matches!(
                                base.as_ref(),
                                NirExpr::Call { callee: helper, .. }
                                    if helper == "wrap__i64__bool"
                            )
                )
    ));
}

#[test]
fn lowers_receiver_method_call_with_explicit_generic_args_through_nested_helper_field_chain() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Showable {
            fn show(value: Self) -> i64;
          }

          struct Carrier<T, U> {
            value: T,
          }

          struct Wrapper<T, U> {
            inner: Carrier<T, U>,
          }

          struct Nest<T, U> {
            outer: Wrapper<T, U>,
          }

          impl<T, U> Showable for Carrier<T, U> {
            fn show(value: Carrier<T, U>) -> i64 {
              return value.value;
            }
          }

          fn make_carrier<T, U>(value: T) -> Carrier<T, U> {
            return Carrier(value);
          }

          fn wrap<T, U>(inner: Carrier<T, U>) -> Wrapper<T, U> {
            return Wrapper { inner: inner };
          }

          fn nest<T, U>(outer: Wrapper<T, U>) -> Nest<T, U> {
            return Nest { outer: outer };
          }

          fn main() -> i64 {
            return nest(wrap(make_carrier(7))).outer.inner.show<i64, bool>();
          }
        }
        "#,
    )
    .unwrap();

    assert!(matches!(
        module.functions[0].body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee.starts_with("impl.Showable.for.Carrier")
                && callee.ends_with(".show__i64__bool")
                && matches!(
                    args.as_slice(),
                    [NirExpr::FieldAccess { base, field }]
                        if field == "inner"
                            && matches!(
                                base.as_ref(),
                                NirExpr::FieldAccess { base: outer_base, field: outer_field }
                                    if outer_field == "outer"
                                        && matches!(
                                            outer_base.as_ref(),
                                            NirExpr::Call { callee: helper, .. }
                                                if helper == "nest__i64__bool"
                                        )
                            )
                )
    ));
}
