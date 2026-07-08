use super::*;

#[test]
fn monomorphizes_generic_function_body_internal_explicit_helpers_through_branch_specialization() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type CellAlias<T> = Cell<T>;
          type PacketAlias<T> = Packet<T>;
          type EnvelopeAlias<T> = Envelope<T>;

          struct Cell<T> {
            value: T,
          }

          struct Packet<T> {
            payload: T,
            tag: i64,
          }

          struct Envelope<T> {
            packet: T,
            ready: bool,
          }

          fn wrap_cell<T>(value: T) -> CellAlias<T> {
            return CellAlias { value: value };
          }

          fn wrap_packet<T>(payload: T, tag: i64) -> PacketAlias<T> {
            return PacketAlias {
              payload: payload,
              tag: tag,
            };
          }

          fn wrap_envelope<T>(packet: T, ready: bool) -> EnvelopeAlias<T> {
            return EnvelopeAlias {
              packet: packet,
              ready: ready,
            };
          }

          async fn produce_packetized<T>(value: T) -> EnvelopeAlias<PacketAlias<CellAlias<T>>> {
            let cell: CellAlias<T> = CellAlias { value: value };
            let packet: PacketAlias<CellAlias<T>> =
              wrap_packet<CellAlias<T>>(cell, 3);
            return wrap_envelope<PacketAlias<CellAlias<T>>>(packet, true);
          }

          fn choose_packetized<T>(
            flag: bool,
            value: T,
            fallback: T
          ) -> EnvelopeAlias<PacketAlias<CellAlias<T>>> {
            if flag {
              let cell: CellAlias<T> = CellAlias { value: value };
              let packet: PacketAlias<CellAlias<T>> =
                wrap_packet<CellAlias<T>>(cell, 8);
              return wrap_envelope<PacketAlias<CellAlias<T>>>(packet, true);
            }
            let zero: CellAlias<T> = CellAlias { value: fallback };
            let packet: PacketAlias<CellAlias<T>> =
              wrap_packet<CellAlias<T>>(zero, 1);
            return wrap_envelope<PacketAlias<CellAlias<T>>>(packet, false);
          }

          fn main() -> i64 {
            let task: Task<EnvelopeAlias<PacketAlias<CellAlias<i64>>>> =
              spawn(produce_packetized<i64>(7));
            let joined: EnvelopeAlias<PacketAlias<CellAlias<i64>>> = join(task);
            let selected: EnvelopeAlias<PacketAlias<CellAlias<i64>>> =
              choose_packetized<i64>(joined.ready, joined.packet.payload.value, 0);
            return selected.packet.payload.value + selected.packet.tag;
          }
        }
        "#,
    )
    .unwrap();

    let produce = module
        .functions
        .iter()
        .find(|function| function.name == "produce_packetized__i64")
        .unwrap();
    assert!(produce.is_async);
    assert!(produce.generic_params.is_empty());
    assert!(matches!(
        produce.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
    assert!(stmt_tree_contains_call(&produce.body, &|callee, _| {
        callee.starts_with("wrap_packet__")
    }));
    assert!(stmt_tree_contains_call(&produce.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose_packetized__i64")
        .unwrap();
    assert!(choose.generic_params.is_empty());
    assert!(matches!(
        choose.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
    assert!(choose
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::If { .. })));
    assert!(stmt_tree_contains_call(&choose.body, &|callee, _| {
        callee.starts_with("wrap_packet__")
    }));
    assert!(stmt_tree_contains_call(&choose.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));

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
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task" && ty.render() == "Task<Envelope<Packet<Cell<i64>>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuJoin(_),
        }) if name == "joined" && ty.render() == "Envelope<Packet<Cell<i64>>>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "selected"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee == "choose_packetized__i64"
    ));
}

#[test]
fn monomorphizes_multi_generic_function_call_into_concrete_nir_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Keepable {
            fn keep(lhs: Self, rhs: Self) -> Self;
          }

          impl Keepable for i64 {
            fn keep(lhs: i64, rhs: i64) -> i64 {
              return lhs;
            }
          }

          impl Keepable for bool {
            fn keep(lhs: bool, rhs: bool) -> bool {
              return rhs;
            }
          }

          fn choose_second<A: Keepable, B: Keepable>(a0: A, a1: A, b0: B, b1: B) -> B {
            return b0.keep(b1);
          }

          fn main() -> bool {
            return choose_second(1, 2, true, false);
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
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "choose_second__i64__bool"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "choose_second__i64__bool")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.name.as_str()),
        Some("bool")
    ));
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "impl.Keepable.for.bool.keep"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_local_type_annotation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let value: i64 = typed_zero();
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
        Some(NirStmt::Let { value: NirExpr::Call { callee, .. }, .. })
            if callee == "typed_zero__i64"
    ));
    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.name.as_str()),
        Some("i64")
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            return typed_zero();
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
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "typed_zero__i64"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_nested_call_parameter_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_i64(value: i64) -> i64 {
            return value;
          }

          fn main() -> i64 {
            return takes_i64(typed_zero());
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
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "takes_i64"
                && matches!(args.as_slice(), [NirExpr::Call { callee, .. }] if callee == "typed_zero__i64")
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_struct_field_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let boxed: Boxed<i64> = Boxed { value: typed_zero() };
            return boxed.value;
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
            value: NirExpr::StructLiteral { fields, .. },
            ..
        }) if matches!(
            fields.as_slice(),
            [(field, NirExpr::Call { callee, .. })] if field == "value" && callee == "typed_zero__i64"
        )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_alias_struct_literal_call_parameter_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_boxed(value: Boxed<i64>) -> i64 {
            return value.value;
          }

          fn main() -> i64 {
            return takes_boxed(BoxAlias { value: typed_zero() });
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
        main.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, args })))
            if callee == "takes_boxed"
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { fields, .. }]
                        if matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Call { callee, .. })]
                                if field == "value" && callee == "typed_zero__i64"
                        )
                )
    ));
}
