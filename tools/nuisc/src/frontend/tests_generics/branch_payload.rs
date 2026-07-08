use super::*;

#[test]
fn monomorphizes_branch_local_payload_reconstruction_through_forwarded_local() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_payload<T>(value: JustAlias<T>) -> T {
            return value.value;
          }

          fn choose(flag: bool) -> i64 {
            if flag {
              let payload = JustAlias(typed_zero());
              let selected = payload;
              return takes_payload(selected);
            }
            let payload = JustAlias(typed_zero());
            let selected = payload;
            return takes_payload(selected);
          }

          fn main() -> i64 {
            return choose(true);
          }
        }
        "#,
    )
    .unwrap();

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .unwrap();
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let {
                        name: payload_name,
                        ty: Some(payload_ty),
                        value: NirExpr::StructLiteral { type_name, type_args, .. },
                    },
                    NirStmt::Let {
                        name: selected_name,
                        ty: Some(selected_ty),
                        value: NirExpr::Var(source_name),
                    },
                    NirStmt::Return(Some(NirExpr::Call { callee, .. }))
                ] if payload_name == "payload"
                    && payload_ty.render() == "Just<i64>"
                    && type_name == "Just"
                    && matches!(type_args.as_slice(), [arg] if arg.render() == "i64")
                    && selected_name == "selected"
                    && selected_ty.render() == "Just<i64>"
                    && source_name == "payload"
                    && callee == "takes_payload__i64"
            )
                && else_body.is_empty()
    ));
    assert!(matches!(
        choose.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, type_args, .. },
        }) if name == "payload"
            && ty.render() == "Just<i64>"
            && type_name == "Just"
            && matches!(type_args.as_slice(), [arg] if arg.render() == "i64")
    ));
    assert!(matches!(
        choose.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Var(source),
        }) if name == "selected" && ty.render() == "Just<i64>" && source == "payload"
    ));
    assert!(matches!(
        choose.body.get(3),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "takes_payload__i64"
    ));
}

#[test]
fn monomorphizes_branch_local_payload_reconstruction_through_forwarded_helper_call() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn typed_zero<T>() -> T {
            return 0;
          }

          fn forward<T>(value: JustAlias<T>) -> JustAlias<T> {
            return value;
          }

          fn takes_payload<T>(value: JustAlias<T>) -> T {
            return value.value;
          }

          fn choose(flag: bool) -> i64 {
            if flag {
              let payload = JustAlias(typed_zero());
              let selected = payload;
              let echoed = forward(selected);
              return takes_payload(echoed);
            }
            let payload = JustAlias(typed_zero());
            let selected = payload;
            let echoed = forward(selected);
            return takes_payload(echoed);
          }

          fn main() -> i64 {
            return choose(true);
          }
        }
        "#,
    )
    .unwrap();

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .unwrap();
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [
                    NirStmt::Let {
                        name: payload_name,
                        ty: Some(payload_ty),
                        value: NirExpr::StructLiteral { type_name, type_args, .. },
                    },
                    NirStmt::Let {
                        name: selected_name,
                        ty: Some(selected_ty),
                        value: NirExpr::Var(source_name),
                    },
                    NirStmt::Let {
                        name: echoed_name,
                        ty: Some(echoed_ty),
                        value: NirExpr::Call { callee: echoed_callee, .. },
                    },
                    NirStmt::Return(Some(NirExpr::Call { callee, .. }))
                ] if payload_name == "payload"
                    && payload_ty.render() == "Just<i64>"
                    && type_name == "Just"
                    && matches!(type_args.as_slice(), [arg] if arg.render() == "i64")
                    && selected_name == "selected"
                    && selected_ty.render() == "Just<i64>"
                    && source_name == "payload"
                    && echoed_name == "echoed"
                    && echoed_ty.render() == "Just<i64>"
                    && echoed_callee == "forward__i64"
                    && callee == "takes_payload__i64"
            )
                && else_body.is_empty()
    ));
}

#[test]
fn monomorphizes_explicit_zero_arg_generic_function_call_into_concrete_nir_function() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            return typed_zero<i64>();
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
            if callee == "typed_zero__i64" && args.is_empty()
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
}

#[test]
fn monomorphizes_explicit_multi_arg_generic_function_call_through_nested_alias_wrappers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type CellAlias<T> = Cell<T>;
          type PacketAlias<T> = Packet<T>;

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

          fn wrap_envelope<T>(packet: T, ready: bool) -> Envelope<T> {
            return Envelope {
              packet: packet,
              ready: ready,
            };
          }

          fn main() -> i64 {
            let cell: CellAlias<i64> = wrap_cell<i64>(7);
            let packet: PacketAlias<CellAlias<i64>> =
              wrap_packet<CellAlias<i64>>(cell, 9);
            let envelope: Envelope<PacketAlias<CellAlias<i64>>> =
              wrap_envelope<PacketAlias<CellAlias<i64>>>(packet, true);
            return envelope.packet.payload.value + envelope.packet.tag;
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
            value: NirExpr::Call { callee, args },
        }) if name == "cell"
            && ty.render() == "Cell<i64>"
            && callee.starts_with("wrap_cell__")
            && args.len() == 1
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, args },
        }) if name == "packet"
            && ty.render() == "Packet<Cell<i64>>"
            && callee.starts_with("wrap_packet__")
            && args.len() == 2
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, args },
        }) if name == "envelope"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee.starts_with("wrap_envelope__")
            && args.len() == 2
    ));

    let packet_specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("wrap_packet__"))
        .unwrap();
    assert!(packet_specialized.generic_params.is_empty());
    assert_eq!(
        packet_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("Cell<i64>".to_owned())
    );
    assert_eq!(
        packet_specialized
            .params
            .get(1)
            .map(|param| param.ty.render()),
        Some("i64".to_owned())
    );
    assert!(matches!(
        packet_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Packet<Cell<i64>>"
    ));

    let envelope_specialized = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("wrap_envelope__"))
        .unwrap();
    assert!(envelope_specialized.generic_params.is_empty());
    assert_eq!(
        envelope_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("Packet<Cell<i64>>".to_owned())
    );
    assert_eq!(
        envelope_specialized
            .params
            .get(1)
            .map(|param| param.ty.render()),
        Some("bool".to_owned())
    );
    assert!(matches!(
        envelope_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
}

#[test]
fn monomorphizes_explicit_generic_wrappers_through_async_if_and_match_control_flow() {
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

          async fn produce_cell<T>() -> CellAlias<T> {
            return CellAlias { value: 7 };
          }

          fn choose(
            flag: bool,
            task: Task<CellAlias<i64>>
          ) -> EnvelopeAlias<PacketAlias<CellAlias<i64>>> {
            if flag {
              return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                wrap_packet<CellAlias<i64>>(join(task), 9),
                true
              );
            }
            let seed: CellAlias<i64> = CellAlias { value: 5 };
            match wrap_envelope<PacketAlias<CellAlias<i64>>>(
              wrap_packet<CellAlias<i64>>(seed, 4),
              false
            ) {
              EnvelopeAlias<PacketAlias<CellAlias<i64>>> {
                packet: { payload: cell, tag: tag },
                ready: false
              } => {
                return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                  wrap_packet<CellAlias<i64>>(cell, tag + 1),
                  true
                );
              }
              _ => {
                let zero: CellAlias<i64> = CellAlias { value: 0 };
                return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                  wrap_packet<CellAlias<i64>>(zero, 0),
                  false
                );
              }
            }
          }

          fn main() -> i64 {
            let task: Task<CellAlias<i64>> = spawn(produce_cell<i64>());
            let envelope: EnvelopeAlias<PacketAlias<CellAlias<i64>>> = choose(true, task);
            return envelope.packet.payload.value + envelope.packet.tag;
          }
        }
        "#,
    )
    .unwrap();

    let produce = module
        .functions
        .iter()
        .find(|function| function.name == "produce_cell__i64")
        .unwrap();
    assert!(produce.is_async);
    assert!(produce.generic_params.is_empty());
    assert!(matches!(
        produce.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Cell<i64>"
    ));

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
        }) if name == "task" && ty.render() == "Task<Cell<i64>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "envelope"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee == "choose"
    ));

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose")
        .unwrap();
    assert!(choose
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::If { .. })));
    assert!(stmt_tree_contains_call(&choose.body, &|callee, args| {
        callee.starts_with("wrap_packet__")
            && matches!(args, [NirExpr::CpuJoin(_), NirExpr::Int(9)])
    }));
    assert!(stmt_tree_contains_call(&choose.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));
    assert!(module.functions.iter().any(|function| {
        function.name.starts_with("wrap_packet__") && function.generic_params.is_empty()
    }));
    assert!(module.functions.iter().any(|function| {
        function.name.starts_with("wrap_envelope__") && function.generic_params.is_empty()
    }));
}
