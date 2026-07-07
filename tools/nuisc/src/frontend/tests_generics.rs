use super::lower_ast_to_nir;
use super::parse_nuis_ast;
use super::parse_nuis_module;
use nuis_semantics::model::{NirExpr, NirStmt};

fn stmt_tree_contains_call<F>(body: &[NirStmt], predicate: &F) -> bool
where
    F: Fn(&str, &[NirExpr]) -> bool,
{
    body.iter().any(|stmt| stmt_contains_call(stmt, predicate))
}

fn stmt_contains_call<F>(stmt: &NirStmt, predicate: &F) -> bool
where
    F: Fn(&str, &[NirExpr]) -> bool,
{
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Expr(value)
        | NirStmt::Await(value)
        | NirStmt::Print(value) => expr_contains_call(value, predicate),
        NirStmt::Return(Some(value)) => expr_contains_call(value, predicate),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_call(condition, predicate)
                || stmt_tree_contains_call(then_body, predicate)
                || stmt_tree_contains_call(else_body, predicate)
        }
        NirStmt::While { condition, body } => {
            expr_contains_call(condition, predicate) || stmt_tree_contains_call(body, predicate)
        }
        _ => false,
    }
}

fn expr_contains_call<F>(expr: &NirExpr, predicate: &F) -> bool
where
    F: Fn(&str, &[NirExpr]) -> bool,
{
    match expr {
        NirExpr::Call { callee, args } => {
            predicate(callee, args) || args.iter().any(|arg| expr_contains_call(arg, predicate))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_call(value, predicate)),
        NirExpr::FieldAccess { base, .. }
        | NirExpr::Await(base)
        | NirExpr::Borrow(base)
        | NirExpr::BorrowEnd(base)
        | NirExpr::CpuJoin(base)
        | NirExpr::CpuThreadJoin(base)
        | NirExpr::DataReady(base)
        | NirExpr::DataMoved(base)
        | NirExpr::DataWindowed(base)
        | NirExpr::DataValue(base)
        | NirExpr::CpuThreadJoinResult(base)
        | NirExpr::CpuTaskCompleted(base)
        | NirExpr::CpuTaskTimedOut(base)
        | NirExpr::CpuTaskCancelled(base)
        | NirExpr::CpuTaskValue(base)
        | NirExpr::CpuMutexNew(base)
        | NirExpr::CpuMutexLock(base)
        | NirExpr::CpuMutexUnlock(base)
        | NirExpr::CpuMutexValue(base) => expr_contains_call(base, predicate),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_call(lhs, predicate) || expr_contains_call(rhs, predicate)
        }
        NirExpr::CpuExternCall { args, .. } => {
            args.iter().any(|arg| expr_contains_call(arg, predicate))
        }
        NirExpr::CpuSpawn { args, .. } | NirExpr::CpuThreadSpawn { args, .. } => {
            args.iter().any(|arg| expr_contains_call(arg, predicate))
        }
        _ => false,
    }
}

#[test]
fn monomorphizes_generic_function_call_into_concrete_nir_function() {
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

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs.add(rhs);
          }

          fn main() -> i64 {
            return sum_two(1, 2);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "sum_two__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "sum_two__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "impl.Addable.for.i64.add"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_call_used_as_method_receiver() {
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

          fn typed_zero<T: Addable>() -> T {
            return 0;
          }

          fn main() -> i64 {
            return typed_zero().add(1);
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
            if callee == "impl.Addable.for.i64.add"
                && matches!(
                    args.as_slice(),
                    [NirExpr::Call { callee: receiver_callee, args: receiver_args }, NirExpr::Int(1)]
                        if receiver_callee == "typed_zero__i64" && receiver_args.is_empty()
                )
    ));
}

#[test]
fn monomorphizes_generic_binary_add_with_addable_bound() {
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

          fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
            return lhs + rhs;
          }

          fn main() -> i64 {
            return sum_two(1, 2);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "sum_two__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "sum_two__i64")
        .unwrap();
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { op, .. }))) if *op == nuis_semantics::model::NirBinaryOp::Add
    ));
}

#[test]
fn monomorphizes_generic_function_with_parent_enum_parameter_from_variant_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn keep<T>(value: Option<T>) -> Option<T> {
            return value;
          }

          fn main() -> i64 {
            let value: Option<i64> = keep(Option.Some(7));
            match value {
              Option.Some(payload) => {
                return payload;
              }
              _ => {
                return 0;
              }
            }
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
        &main.body[0],
        NirStmt::Let {
            value: NirExpr::Call { callee, args },
            ..
        } if callee == "keep__i64"
            && matches!(
                args.as_slice(),
                [NirExpr::StructLiteral { type_name, type_args, .. }]
                    if type_name == "Option.Some"
                        && type_args.len() == 1
                        && type_args[0].render() == "i64"
            )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.params.as_slice(),
        [param] if param.ty.render() == "Option<i64>"
    ));
}

#[test]
fn monomorphizes_parent_enum_parameter_from_unit_variant_and_sibling_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn fallback<T>(value: Option<T>, fallback: T) -> T {
            match value {
              Option.Some(payload) => {
                return payload;
              }
              _ => {
                return fallback;
              }
            }
          }

          fn main() -> i64 {
            return fallback(Option.None, 7);
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
            if callee == "fallback__i64"
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { type_name, type_args, .. }, NirExpr::Int(7)]
                        if type_name == "Option.None"
                            && type_args.len() == 1
                            && type_args[0].render() == "i64"
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "fallback__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.params.as_slice(),
        [option_param, fallback_param]
            if option_param.ty.render() == "Option<i64>"
                && fallback_param.ty.render() == "i64"
    ));
}

#[test]
fn monomorphizes_parent_enum_parameter_from_unit_variant_and_expected_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          fn keep<T>(value: Option<T>) -> Option<T> {
            return value;
          }

          fn main() {
            let value: Option<i64> = keep(Option.None);
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
        &main.body[0],
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::Call { callee, args },
            ..
        } if ty.render() == "Option<i64>"
            && callee == "keep__i64"
            && matches!(
                args.as_slice(),
                [NirExpr::StructLiteral { type_name, type_args, .. }]
                    if type_name == "Option.None"
                        && type_args.len() == 1
                        && type_args[0].render() == "i64"
            )
    ));
}

#[test]
fn generic_bound_accepts_enum_variant_argument_via_parent_enum_impl() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option<T> {
            None,
            Some(T),
          }

          trait Showable {
            fn show(value: Self) -> i64;
          }

          impl Showable for Option<i64> {
            fn show(value: Option<i64>) -> i64 {
              match value {
                Option.Some(payload) => {
                  return payload;
                }
                Option.None => {
                  return 0;
                }
                _ => {
                  return -1;
                }
              }
            }
          }

          fn reveal<T: Showable>(value: T) -> i64 {
            return Showable.show(value);
          }

          fn main() -> i64 {
            return reveal(Option.Some(7));
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
    let specialized_name = match main.body.last() {
        Some(nuis_semantics::model::NirStmt::Return(Some(
            nuis_semantics::model::NirExpr::Call { callee, .. },
        ))) => callee.clone(),
        other => panic!("expected main to return specialized reveal call, found {other:?}"),
    };

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == specialized_name)
        .unwrap();
    assert!(matches!(
        specialized.body.first(),
        Some(nuis_semantics::model::NirStmt::Return(Some(
            nuis_semantics::model::NirExpr::Call { callee, .. }
        ))) if callee == "impl.Showable.for.Option_i64_.show"
    ));
}

#[test]
fn monomorphizes_generic_binary_remainder_with_remainderable_bound() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          trait Remainderable {
            fn rem(lhs: Self, rhs: Self) -> Self;
          }

          impl Remainderable for i64 {
            fn rem(lhs: i64, rhs: i64) -> i64 {
              return lhs % rhs;
            }
          }

          fn reduce<T: Remainderable>(lhs: T, rhs: T) -> T {
            return lhs % rhs;
          }

          fn main() -> i64 {
            return reduce(9, 4);
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. }))) if callee == "reduce__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "reduce__i64")
        .unwrap();
    assert!(matches!(
        specialized.body.last(),
        Some(NirStmt::Return(Some(NirExpr::Binary { op, .. }))) if *op == nuis_semantics::model::NirBinaryOp::Rem
    ));
}

#[test]
fn monomorphizes_branch_local_payload_reconstruction_before_generic_call() {
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
              return takes_payload(payload);
            }
            let payload = JustAlias(typed_zero());
            return takes_payload(payload);
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
                        name,
                        ty: Some(ty),
                        value: NirExpr::StructLiteral { type_name, type_args, .. },
                    },
                    NirStmt::Return(Some(NirExpr::Call { callee, .. }))
                ] if name == "payload"
                    && ty.render() == "Just<i64>"
                    && type_name == "Just"
                    && matches!(type_args.as_slice(), [arg] if arg.render() == "i64")
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
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "takes_payload__i64"
    ));
}

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

#[test]
fn monomorphizes_zero_arg_generic_from_alias_payload_constructor_call_parameter_expectation() {
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

          fn takes_payload(value: Just<i64>) -> i64 {
            return value.value;
          }

          fn main() -> i64 {
            return takes_payload(JustAlias(typed_zero()));
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
            if callee == "takes_payload"
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

#[test]
fn monomorphizes_zero_arg_generic_from_if_branch_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            if 1 == 1 {
              return typed_zero();
            } else {
              return 9;
            }
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
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
                    if callee == "typed_zero__i64"
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_match_arm_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn typed_zero<T>() -> T {
            return 0;
          }

          fn main() -> i64 {
            let flag: i64 = 1;
            match flag {
              1 => {
                return typed_zero();
              }
              _ => {
                return 9;
              }
            }
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
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, .. }))]
                    if callee == "typed_zero__i64"
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_if_branch_alias_struct_literal_call_parameter_expectation() {
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
            if 1 == 1 {
              return takes_boxed(BoxAlias { value: typed_zero() });
            } else {
              return 9;
            }
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
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
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
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_from_match_arm_alias_payload_constructor_call_parameter_expectation(
) {
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

          fn takes_payload(value: Just<i64>) -> i64 {
            return value.value;
          }

          fn main() -> i64 {
            let flag: i64 = 1;
            match flag {
              1 => {
                return takes_payload(JustAlias(typed_zero()));
              }
              _ => {
                return 9;
              }
            }
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
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "takes_payload"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { fields, .. }]
                                if matches!(
                                    fields.as_slice(),
                                    [(field, NirExpr::Call { callee, .. })]
                                        if field == "value" && callee == "typed_zero__i64"
                                )
                        )
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Int(9)))]
            )
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_struct_literal_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Boxed<T> {
            value: T,
          }

          fn unwrap_box<T>(boxed: Boxed<T>) -> T {
            return boxed.value;
          }

          fn main() -> i64 {
            return unwrap_box(Boxed { value: 7 });
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
            if callee == "unwrap_box__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_alias_struct_literal_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type BoxAlias<T> = Boxed<T>;

          struct Boxed<T> {
            value: T,
          }

          fn unwrap_box<T>(boxed: Boxed<T>) -> T {
            return boxed.value;
          }

          fn main() -> i64 {
            return unwrap_box(BoxAlias { value: 7 });
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
            if callee == "unwrap_box__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_non_transparent_alias_struct_literal_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type WrappedStructAlias<T> = Wrapper<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Wrapper<T> {
            inner: T,
            tag: i64,
          }

          fn unwrap_wrapped<T>(wrapped: Wrapper<Boxed<T>>) -> T {
            return wrapped.inner.value;
          }

          fn main() -> i64 {
            return unwrap_wrapped(WrappedStructAlias {
              inner: Boxed { value: 7 },
              tag: 1,
            });
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
            if callee == "unwrap_wrapped__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_outer_struct_literal_with_deferred_inner_inference() {
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

          fn unwrap_outer<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(Outer {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            });
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
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_outer_struct_literal_with_deferred_inner_payload_inference()
{
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

          fn unwrap_outer<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(Outer {
              inner: Just(7),
              meta: "ok",
            });
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
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_payload_constructor_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Just<T> {
            value: T,
          }

          fn unwrap_just<T>(value: Just<T>) -> T {
            return value.value;
          }

          fn main() -> i64 {
            return unwrap_just(Just(7));
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
            if callee == "unwrap_just__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_inferred_alias_payload_constructor_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          fn unwrap_just<T>(value: Just<T>) -> T {
            return value.value;
          }

          fn main() -> i64 {
            return unwrap_just(JustAlias(7));
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
            if callee == "unwrap_just__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_transparent_alias_outer_literal_with_deferred_inner_inference(
) {
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

          fn unwrap_outer<T, U>(outer: Outer<T, U>) -> T {
            return outer.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(OuterAlias {
              inner: Phantom { value: 7, tag: 1 },
              meta: "ok",
            });
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
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_non_transparent_alias_outer_literal_with_deferred_inner_inference(
) {
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

          fn unwrap_outer<T, U>(wrapped: Wrapper<Outer<T, U>>) -> T {
            return wrapped.inner.inner.value;
          }

          fn main() -> i64 {
            return unwrap_outer(OuterAlias {
              inner: Outer {
                inner: Phantom { value: 7, tag: 1 },
                meta: "ok",
              },
              mark: 1,
            });
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
            if callee == "unwrap_outer__i64__String"
    ));
}

#[test]
fn monomorphizes_generic_function_from_pipe_shaped_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn roundtrip_pipe<T>(pipe: Pipe<T>) -> T {
            return data_input_pipe(pipe);
          }

          fn main() -> i64 {
            return roundtrip_pipe(data_output_pipe(7));
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
            if callee == "roundtrip_pipe__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "roundtrip_pipe__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("Pipe<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_window_shaped_argument_and_expected_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep_window<T>(window: Window<T>) -> Window<T> {
            return window;
          }

          fn main() -> i64 {
            let frozen: Window<i64> = keep_window(data_freeze_window(data_copy_window(7, 0, 1)));
            return data_read_window(frozen, 0);
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
            if callee == "keep_window__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_window__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("Window<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Window<i64>"
    ));
}

#[test]
fn monomorphizes_generic_function_from_task_shaped_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn ping() -> i64 {
            return 7;
          }

          fn keep_task<T>(task: Task<T>) -> Task<T> {
            return task;
          }

          fn main() -> i64 {
            let task: Task<i64> = keep_task(spawn(ping()));
            return join(task);
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
            if callee == "keep_task__i64"
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_task__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("Task<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Task<i64>"
    ));
}

#[test]
fn monomorphizes_generic_function_from_data_result_shaped_argument() {
    let ast = parse_nuis_ast(
        r#"
        mod cpu Main {
          fn keep_data<T>(result: DataResult<T>) -> DataResult<T> {
            return result;
          }

          fn main() -> i64 {
            let result: DataResult<i64> = keep_data(data_result(data_input_pipe(data_output_pipe(7))));
            return data_value(result);
          }
        }
        "#,
    )
    .unwrap();
    let module = lower_ast_to_nir(&ast).unwrap();

    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(main.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        } if name == "result" && callee == "keep_data__i64"
    )));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_data__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("DataResult<i64>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "DataResult<i64>"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_async_function_from_await_return_expectation() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn typed_zero<T>() -> T {
            return 0;
          }

          async fn main() -> i64 {
            return await typed_zero();
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
        Some(NirStmt::Return(Some(NirExpr::Await(value))))
            if matches!(
                value.as_ref(),
                NirExpr::Call { callee, .. } if callee == "typed_zero__i64"
            )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_async_function_through_await_into_alias_payload_call_parameter() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type JustAlias<T> = Just<T>;

          struct Just<T> {
            value: T,
          }

          async fn typed_zero<T>() -> T {
            return 0;
          }

          fn takes_payload(value: Just<i64>) -> i64 {
            return value.value;
          }

          async fn main() -> i64 {
            return takes_payload(JustAlias(await typed_zero()));
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
            if callee == "takes_payload"
                && matches!(
                    args.as_slice(),
                    [NirExpr::StructLiteral { fields, .. }]
                        if matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Await(value))]
                                if field == "value"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Call { callee, .. } if callee == "typed_zero__i64"
                                    )
                        )
                )
    ));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_zero__i64")
        .unwrap();
    assert!(specialized.is_async);
    assert!(specialized.generic_params.is_empty());
}

#[test]
fn monomorphizes_generic_function_from_data_result_shaped_argument_in_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep_data<T>(result: DataResult<T>) -> DataResult<T> {
            return result;
          }

          fn produce() -> DataResult<i64> {
            return keep_data(data_result(data_input_pipe(data_output_pipe(7))));
          }

          fn main() -> i64 {
            return data_value(produce());
          }
        }
        "#,
    )
    .unwrap();

    let produce = module
        .functions
        .iter()
        .find(|function| function.name == "produce")
        .unwrap();
    assert!(matches!(
        produce.body.first(),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::DataResult { .. },
        }) if name == "__nuis_generic_return_arg_0" && ty.render() == "DataResult<i64>"
    ));
    assert!(matches!(
        produce.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "keep_data__i64"
    ));
}

#[test]
fn monomorphizes_generic_function_from_nested_alias_shaped_argument() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Frozen<T> = Window<T>;
          type Wrapped<T> = DataResult<Frozen<T>>;

          fn keep_wrapped<T>(wrapped: Wrapped<T>) -> Wrapped<T> {
            return wrapped;
          }

          fn main() -> i64 {
            let wrapped: Wrapped<i64> =
              keep_wrapped(data_result(data_freeze_window(data_copy_window(7, 0, 1))));
            return data_read_window(data_value(wrapped), 0);
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
    assert!(main.body.iter().any(|stmt| matches!(
        stmt,
        NirStmt::Let {
            name,
            value: NirExpr::Call { callee, .. },
            ..
        } if name == "wrapped" && callee == "keep_wrapped__i64"
    )));

    let specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_wrapped__i64")
        .unwrap();
    assert!(specialized.generic_params.is_empty());
    assert_eq!(
        specialized.params.first().map(|param| param.ty.render()),
        Some("DataResult<Window<i64>>".to_owned())
    );
    assert!(matches!(
        specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "DataResult<Window<i64>>"
    ));
}

#[test]
fn monomorphizes_zero_arg_generic_async_function_through_await_into_nested_alias_wrapper_argument()
{
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
          }

          async fn typed_box<T>() -> Boxed<T> {
            return Boxed(7);
          }

          fn keep_response<T>(response: Response<T>) -> Response<T> {
            return response;
          }

          async fn main() -> i64 {
            let response: Response<i64> = keep_response(Response(await typed_box()));
            return response.payload.value;
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
        }) if name == "response"
            && callee == "keep_response__i64"
            && matches!(
                args.as_slice(),
                [NirExpr::StructLiteral { type_name, type_args, fields }]
                    if type_name == "Envelope"
                        && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
                        && matches!(
                            fields.as_slice(),
                            [(field, NirExpr::Await(value))]
                                if field == "payload"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Call { callee, .. } if callee == "typed_box__i64"
                                    )
                        )
            )
    ));

    let box_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "typed_box__i64")
        .unwrap();
    assert!(box_specialized.is_async);
    assert!(box_specialized.generic_params.is_empty());
    assert!(matches!(
        box_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Boxed<i64>"
    ));

    let response_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_response__i64")
        .unwrap();
    assert!(response_specialized.generic_params.is_empty());
    assert_eq!(
        response_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("Envelope<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        response_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));
}

#[test]
fn monomorphizes_generic_nested_alias_task_join_through_if_branch() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response(Boxed(7));
          }

          fn keep_response<T>(response: Response<T>) -> Response<T> {
            return response;
          }

          fn choose(flag: bool, task: Task<Response<i64>>) -> Response<i64> {
            if flag {
              return keep_response(join(task));
            } else {
              return Response(Boxed(9));
            }
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let response: Response<i64> = choose(true, task);
            return response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let produce_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "produce_response__i64")
        .unwrap();
    assert!(produce_specialized.is_async);
    assert!(produce_specialized.generic_params.is_empty());
    assert!(matches!(
        produce_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));

    let keep_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "keep_response__i64")
        .unwrap();
    assert!(keep_specialized.generic_params.is_empty());
    assert_eq!(
        keep_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("Envelope<Boxed<i64>>".to_owned())
    );

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
        }) if name == "task" && ty.render() == "Task<Envelope<Boxed<i64>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "response"
            && ty.render() == "Envelope<Boxed<i64>>"
            && callee == "choose"
    ));

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
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "keep_response__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                    if type_name == "Envelope"
                        && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
            )
    ));
}

#[test]
fn monomorphizes_generic_response_unwrap_through_task_join_and_branch_constructors() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn unwrap_response<T>(response: Response<T>) -> T {
            match response {
              Response<T> { payload: { value: body }, ready: true } => {
                return body;
              }
              _ => {
                return response.payload.value;
              }
            }
          }

          fn consume(flag: bool, task: Task<Response<i64>>) -> i64 {
            if flag {
              return unwrap_response(join(task));
            } else {
              return unwrap_response(Response { payload: Boxed(9), ready: false });
            }
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            return consume(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let produced = module
        .functions
        .iter()
        .find(|function| function.name == "produce_response__i64")
        .unwrap();
    assert!(produced.is_async);
    assert!(produced.generic_params.is_empty());
    assert!(matches!(
        produced.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));

    let unwrapped = module
        .functions
        .iter()
        .find(|function| function.name == "unwrap_response__i64")
        .unwrap();
    assert!(unwrapped.generic_params.is_empty());
    assert_eq!(
        unwrapped.params.first().map(|param| param.ty.render()),
        Some("Envelope<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        unwrapped.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let consume = module
        .functions
        .iter()
        .find(|function| function.name == "consume")
        .unwrap();
    assert!(matches!(
        consume.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "unwrap_response__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "unwrap_response__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "Envelope"
                                    && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
                        )
            )
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
        }) if name == "task" && ty.render() == "Task<Envelope<Boxed<i64>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "consume"
    ));
}

#[test]
fn monomorphizes_network_shaped_generic_task_exchange_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Request<T> = HttpRequest<Boxed<T>>;
          type Response<T> = HttpResponse<Boxed<T>>;
          type HttpResult<T> = ResultEnvelope<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry: bool,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            ok: bool,
          }

          fn keep_request<T>(request: Request<T>) -> Request<T> {
            return request;
          }

          async fn exchange<T>(request: Request<T>) -> HttpResult<T> {
            return HttpResult {
              response: Response { body: request.body, status: 200 },
              ok: true,
            };
          }

          fn read_body<T>(result: HttpResult<T>) -> T {
            match result {
              HttpResult<T> { response: { body: { value: payload }, status: 200 }, ok: true } => {
                return payload;
              }
              _ => {
                return result.response.body.value;
              }
            }
          }

          fn serve(flag: bool, task: Task<HttpResult<i64>>) -> i64 {
            if flag {
              return read_body(join(task));
            } else {
              return read_body(HttpResult {
                response: Response { body: Boxed(9), status: 503 },
                ok: false,
              });
            }
          }

          fn main() -> i64 {
            let request: Request<i64> = keep_request(Request { body: Boxed(7), retry: false });
            let task: Task<HttpResult<i64>> = spawn(exchange(request));
            return serve(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let keep_request = module
        .functions
        .iter()
        .find(|function| function.name == "keep_request__i64")
        .unwrap();
    assert!(keep_request.generic_params.is_empty());
    assert_eq!(
        keep_request.params.first().map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        keep_request.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "HttpRequest<Boxed<i64>>"
    ));

    let exchange = module
        .functions
        .iter()
        .find(|function| function.name == "exchange__i64")
        .unwrap();
    assert!(exchange.is_async);
    assert!(exchange.generic_params.is_empty());
    assert_eq!(
        exchange.params.first().map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "ResultEnvelope<HttpResponse<Boxed<i64>>>"
    ));

    let read_body = module
        .functions
        .iter()
        .find(|function| function.name == "read_body__i64")
        .unwrap();
    assert!(read_body.generic_params.is_empty());
    assert_eq!(
        read_body.params.first().map(|param| param.ty.render()),
        Some("ResultEnvelope<HttpResponse<Boxed<i64>>>".to_owned())
    );
    assert!(matches!(
        read_body.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let serve = module
        .functions
        .iter()
        .find(|function| function.name == "serve")
        .unwrap();
    assert!(matches!(
        serve.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "read_body__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "read_body__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "ResultEnvelope"
                                    && matches!(type_args.as_slice(), [ty] if ty.render() == "HttpResponse<Boxed<i64>>")
                        )
            )
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
            value: NirExpr::Call { callee, .. },
        }) if name == "request"
            && ty.render() == "HttpRequest<Boxed<i64>>"
            && callee == "keep_request__i64"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render() == "Task<ResultEnvelope<HttpResponse<Boxed<i64>>>>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "serve"
    ));
}

#[test]
fn monomorphizes_std_net_facade_shaped_http_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          fn net_http_request<T>(request: NetHttpRequest<T>) -> NetHttpRequest<T> {
            return request;
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchange<T> {
            return NetHttpClientExchange {
              result: NetResult {
                response: NetHttpResponse {
                  body: request.body,
                  status: 200,
                },
                recv_ready: true,
              },
              attempts: request.retry_budget,
            };
          }

          async fn net_session<T>(request: NetHttpRequest<T>) -> NetSession<T> {
            return NetSession {
              exchange: await net_http_client_exchange(request),
              open: true,
            };
          }

          fn net_http_response_value<T>(session: NetSession<T>) -> T {
            match session {
              NetSession<T> {
                exchange: {
                  result: {
                    response: { body: { value: payload }, status: 200 },
                    recv_ready: true,
                  },
                  attempts: 2,
                },
                open: true,
              } => {
                return payload;
              }
              _ => {
                return session.exchange.result.response.body.value;
              }
            }
          }

          fn serve(flag: bool, task: Task<NetSession<i64>>) -> i64 {
            if flag {
              return net_http_response_value(join(task));
            } else {
              return net_http_response_value(NetSession {
                exchange: NetHttpClientExchange {
                  result: NetResult {
                    response: NetHttpResponse { body: Boxed(9), status: 503 },
                    recv_ready: false,
                  },
                  attempts: 1,
                },
                open: false,
              });
            }
          }

          fn main() -> i64 {
            let request: NetHttpRequest<i64> = net_http_request(NetHttpRequest {
              body: Boxed(7),
              retry_budget: 2,
            });
            let task: Task<NetSession<i64>> = spawn(net_session(request));
            return serve(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let request_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_request__i64")
        .unwrap();
    assert!(request_specialized.generic_params.is_empty());
    assert_eq!(
        request_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        request_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "HttpRequest<Boxed<i64>>"
    ));

    let exchange_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_client_exchange__i64")
        .unwrap();
    assert!(exchange_specialized.is_async);
    assert!(exchange_specialized.generic_params.is_empty());
    assert_eq!(
        exchange_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange_specialized
            .return_type
            .as_ref()
            .map(|ty| ty.render()),
        Some(rendered) if rendered == "ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>"
    ));

    let session_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_session__i64")
        .unwrap();
    assert!(session_specialized.is_async);
    assert!(session_specialized.generic_params.is_empty());
    assert_eq!(
        session_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        session_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>"
    ));

    let value_specialized = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_response_value__i64")
        .unwrap();
    assert!(value_specialized.generic_params.is_empty());
    assert_eq!(
        value_specialized
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>".to_owned())
    );
    assert!(matches!(
        value_specialized.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let serve = module
        .functions
        .iter()
        .find(|function| function.name == "serve")
        .unwrap();
    assert!(matches!(
        serve.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "net_http_response_value__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "net_http_response_value__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "SessionLane"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty]
                                            if ty.render()
                                                == "ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>"
                                    )
                        )
            )
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
            value: NirExpr::Call { callee, .. },
        }) if name == "request"
            && ty.render() == "HttpRequest<Boxed<i64>>"
            && callee == "net_http_request__i64"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render()
                == "Task<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
            if callee == "serve"
    ));
}

#[test]
fn monomorphizes_std_net_demo_shaped_summary_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;
          type NetHttpClientExchangeSummary<T> = ExchangeSummary<NetSession<T>>;
          type NetSessionSummary<T> = SessionSummary<NetHttpClientExchangeSummary<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          struct ExchangeSummary<T> {
            session: T,
            exchange_value: i64,
          }

          struct SessionSummary<T> {
            summary: T,
            session_value: i64,
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchange<T> {
            return NetHttpClientExchange {
              result: NetResult {
                response: NetHttpResponse {
                  body: request.body,
                  status: 200,
                },
                recv_ready: true,
              },
              attempts: request.retry_budget,
            };
          }

          async fn net_session<T>(request: NetHttpRequest<T>) -> NetSession<T> {
            return NetSession {
              exchange: await net_http_client_exchange(request),
              open: true,
            };
          }

          async fn capture_net_http_client_exchange_summary<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return NetHttpClientExchangeSummary {
              session: await net_session(request),
              exchange_value: 41,
            };
          }

          async fn capture_net_session_summary<T>(
            request: NetHttpRequest<T>
          ) -> NetSessionSummary<T> {
            return SessionSummary {
              summary: await capture_net_http_client_exchange_summary(request),
              session_value: 99,
            };
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            match summary {
              NetSessionSummary<T> {
                summary: {
                  session: {
                    exchange: {
                      result: {
                        response: { body: { value: payload }, status: 200 },
                        recv_ready: true,
                      },
                      attempts: 2,
                    },
                    open: true,
                  },
                  exchange_value: 41,
                },
                session_value: 99,
              } => {
                return payload;
              }
              _ => {
                return summary.summary.session.exchange.result.response.body.value;
              }
            }
          }

          fn serve(flag: bool, task: Task<NetSessionSummary<i64>>) -> i64 {
            if flag {
              return summarize_net_session(join(task));
            } else {
              return summarize_net_session(SessionSummary {
                summary: ExchangeSummary {
                  session: SessionLane {
                    exchange: ExchangeLane {
                      result: ResultEnvelope {
                        response: HttpResponse { body: Boxed(9), status: 503 },
                        recv_ready: false,
                      },
                      attempts: 1,
                    },
                    open: false,
                  },
                  exchange_value: 40,
                },
                session_value: 98,
              });
            }
          }

          fn main() -> i64 {
            let request: NetHttpRequest<i64> = NetHttpRequest {
              body: Boxed(7),
              retry_budget: 2,
            };
            let task: Task<NetSessionSummary<i64>> = spawn(capture_net_session_summary(request));
            return serve(true, task);
          }
        }
        "#,
    )
    .unwrap();

    let exchange_summary = module
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_client_exchange_summary__i64")
        .unwrap();
    assert!(exchange_summary.is_async);
    assert!(exchange_summary.generic_params.is_empty());
    assert_eq!(
        exchange_summary
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange_summary.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
    ));

    let session_summary = module
        .functions
        .iter()
        .find(|function| function.name == "capture_net_session_summary__i64")
        .unwrap();
    assert!(session_summary.is_async);
    assert!(session_summary.generic_params.is_empty());
    assert_eq!(
        session_summary
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        session_summary.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
    ));

    let summarize = module
        .functions
        .iter()
        .find(|function| function.name == "summarize_net_session__i64")
        .unwrap();
    assert!(summarize.generic_params.is_empty());
    assert_eq!(
        summarize.params.first().map(|param| param.ty.render()),
        Some(
            "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
                .to_owned()
        )
    );
    assert!(matches!(
        summarize.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "i64"
    ));

    let serve = module
        .functions
        .iter()
        .find(|function| function.name == "serve")
        .unwrap();
    assert!(matches!(
        serve.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "summarize_net_session__i64"
                        && matches!(args.as_slice(), [NirExpr::CpuJoin(_)])
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::Call { callee, args }))]
                    if callee == "summarize_net_session__i64"
                        && matches!(
                            args.as_slice(),
                            [NirExpr::StructLiteral { type_name, type_args, .. }]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty]
                                            if ty.render()
                                                == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                                    )
                        )
            )
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
            value: NirExpr::StructLiteral { type_name, type_args, .. },
        }) if name == "request"
            && ty.render() == "HttpRequest<Boxed<i64>>"
            && type_name == "HttpRequest"
            && matches!(type_args.as_slice(), [arg] if arg.render() == "Boxed<i64>")
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render()
                == "Task<SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>>"
    ));
}

#[test]
fn monomorphizes_match_arm_std_net_summary_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;
          type NetHttpClientExchangeSummary<T> = ExchangeSummary<NetSession<T>>;
          type NetSessionSummary<T> = SessionSummary<NetHttpClientExchangeSummary<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          struct ExchangeSummary<T> {
            session: T,
            exchange_value: i64,
          }

          struct SessionSummary<T> {
            summary: T,
            session_value: i64,
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchange<T> {
            return NetHttpClientExchange {
              result: NetResult {
                response: NetHttpResponse {
                  body: request.body,
                  status: 200,
                },
                recv_ready: true,
              },
              attempts: request.retry_budget,
            };
          }

          async fn net_session<T>(request: NetHttpRequest<T>) -> NetSession<T> {
            return NetSession {
              exchange: await net_http_client_exchange(request),
              open: true,
            };
          }

          async fn capture_net_http_client_exchange_summary<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return NetHttpClientExchangeSummary {
              session: await net_session(request),
              exchange_value: 41,
            };
          }

          async fn capture_net_session_summary<T>(
            request: NetHttpRequest<T>
          ) -> NetSessionSummary<T> {
            return SessionSummary {
              summary: await capture_net_http_client_exchange_summary(request),
              session_value: 99,
            };
          }

          fn choose_summary(
            mode: i64,
            task: Task<NetSessionSummary<i64>>,
            summary_task: Task<NetHttpClientExchangeSummary<i64>>
          ) -> NetSessionSummary<i64> {
            match mode {
              0 => {
                return join(task);
              }
              1 => {
                return SessionSummary {
                  summary: join(summary_task),
                  session_value: 99,
                };
              }
              _ => {
                return SessionSummary {
                  summary: ExchangeSummary {
                    session: SessionLane {
                      exchange: ExchangeLane {
                        result: ResultEnvelope {
                          response: HttpResponse { body: Boxed(9), status: 503 },
                          recv_ready: false,
                        },
                        attempts: 1,
                      },
                      open: false,
                    },
                    exchange_value: 40,
                  },
                  session_value: 98,
                };
              }
            }
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            return summary.summary.session.exchange.result.response.body.value;
          }

          fn main() -> i64 {
            let summary_task: Task<NetHttpClientExchangeSummary<i64>> =
              spawn(capture_net_http_client_exchange_summary(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let task: Task<NetSessionSummary<i64>> =
              spawn(capture_net_session_summary(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let summary: NetSessionSummary<i64> = choose_summary(1, task, summary_task);
            return summarize_net_session(summary);
          }
        }
        "#,
    )
    .unwrap();

    let exchange_summary = module
        .functions
        .iter()
        .find(|function| function.name == "capture_net_http_client_exchange_summary__i64")
        .unwrap();
    assert!(exchange_summary.is_async);
    assert!(exchange_summary.generic_params.is_empty());
    assert_eq!(
        exchange_summary
            .params
            .first()
            .map(|param| param.ty.render()),
        Some("HttpRequest<Boxed<i64>>".to_owned())
    );
    assert!(matches!(
        exchange_summary.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
    ));

    let choose = module
        .functions
        .iter()
        .find(|function| function.name == "choose_summary")
        .unwrap();
    assert!(matches!(
        choose.body.first(),
        Some(NirStmt::If { then_body, else_body, .. })
            if matches!(
                then_body.as_slice(),
                [NirStmt::Return(Some(NirExpr::CpuJoin(_)))]
            ) && matches!(
                else_body.as_slice(),
                [NirStmt::If {
                    then_body: nested_then,
                    else_body: nested_else,
                    ..
                }] if matches!(
                    nested_then.as_slice(),
                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                        if type_name == "SessionSummary"
                            && matches!(
                                type_args.as_slice(),
                                [ty]
                                    if ty.render()
                                        == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                            )
                            && matches!(
                                fields.as_slice(),
                                [
                                    (summary_field, NirExpr::CpuJoin(value)),
                                    (session_value_field, NirExpr::Int(99))
                                ]
                                    if summary_field == "summary"
                                        && session_value_field == "session_value"
                                        && matches!(
                                            value.as_ref(),
                                            NirExpr::Var(name) if name == "summary_task"
                                        )
                            )
                ) && matches!(
                    nested_else.as_slice(),
                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                        if type_name == "SessionSummary"
                            && matches!(
                                type_args.as_slice(),
                                [ty]
                                    if ty.render()
                                        == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                            )
                )
            )
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
        }) if name == "summary_task"
            && ty.render()
                == "Task<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task"
            && ty.render()
                == "Task<SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>>"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "summary"
            && ty.render()
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
            && callee == "choose_summary"
    ));
}

#[test]
fn monomorphizes_while_body_std_net_summary_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;
          type NetHttpClientExchangeSummary<T> = ExchangeSummary<NetSession<T>>;
          type NetSessionSummary<T> = SessionSummary<NetHttpClientExchangeSummary<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          struct ExchangeSummary<T> {
            session: T,
            exchange_value: i64,
          }

          struct SessionSummary<T> {
            summary: T,
            session_value: i64,
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return ExchangeSummary {
              session: SessionLane {
                exchange: ExchangeLane {
                  result: ResultEnvelope {
                    response: HttpResponse {
                      body: request.body,
                      status: 200,
                    },
                    recv_ready: true,
                  },
                  attempts: request.retry_budget,
                },
                open: true,
              },
              exchange_value: 41,
            };
          }

          fn loop_summary(
            seed: i64,
            summary_task: Task<NetHttpClientExchangeSummary<i64>>
          ) -> NetSessionSummary<i64> {
            while seed > 0 {
              return SessionSummary {
                summary: join(summary_task),
                session_value: 99,
              };
            }
            return SessionSummary {
              summary: ExchangeSummary {
                session: SessionLane {
                  exchange: ExchangeLane {
                    result: ResultEnvelope {
                      response: HttpResponse { body: Boxed(9), status: 503 },
                      recv_ready: false,
                    },
                    attempts: 1,
                  },
                  open: false,
                },
                exchange_value: 40,
              },
              session_value: 98,
            };
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            return summary.summary.session.exchange.result.response.body.value;
          }

          fn main() -> i64 {
            let summary_task: Task<NetHttpClientExchangeSummary<i64>> =
              spawn(net_http_client_exchange(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let summary: NetSessionSummary<i64> = loop_summary(1, summary_task);
            return summarize_net_session(summary);
          }
        }
        "#,
    )
    .unwrap();

    let exchange = module
        .functions
        .iter()
        .find(|function| function.name == "net_http_client_exchange__i64")
        .unwrap();
    assert!(exchange.is_async);
    assert!(exchange.generic_params.is_empty());
    assert!(matches!(
        exchange.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered)
            if rendered
                == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
    ));

    let loop_summary = module
        .functions
        .iter()
        .find(|function| function.name == "loop_summary")
        .unwrap();
    assert!(matches!(
        loop_summary.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                    if type_name == "SessionSummary"
                        && matches!(
                            type_args.as_slice(),
                            [ty]
                                if ty.render()
                                    == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                        )
                        && matches!(
                            fields.as_slice(),
                            [
                                (summary_field, NirExpr::CpuJoin(value)),
                                (session_value_field, NirExpr::Int(99))
                            ]
                                if summary_field == "summary"
                                    && session_value_field == "session_value"
                                    && matches!(
                                        value.as_ref(),
                                        NirExpr::Var(name) if name == "summary_task"
                                    )
                        )
            )
    ));
    assert!(matches!(
        loop_summary.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. })))
            if type_name == "SessionSummary"
                && matches!(
                    type_args.as_slice(),
                    [ty]
                        if ty.render()
                            == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                )
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
        }) if name == "summary_task"
            && ty.render()
                == "Task<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "summary"
            && ty.render()
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
            && callee == "loop_summary"
    ));
}

#[test]
fn monomorphizes_nested_while_match_std_net_summary_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;
          type NetHttpClientExchangeSummary<T> = ExchangeSummary<NetSession<T>>;
          type NetSessionSummary<T> = SessionSummary<NetHttpClientExchangeSummary<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          struct ExchangeSummary<T> {
            session: T,
            exchange_value: i64,
          }

          struct SessionSummary<T> {
            summary: T,
            session_value: i64,
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return ExchangeSummary {
              session: SessionLane {
                exchange: ExchangeLane {
                  result: ResultEnvelope {
                    response: HttpResponse {
                      body: request.body,
                      status: 200,
                    },
                    recv_ready: true,
                  },
                  attempts: request.retry_budget,
                },
                open: true,
              },
              exchange_value: 41,
            };
          }

          fn nested_loop_summary(
            seed: i64,
            mode: i64,
            summary_task: Task<NetHttpClientExchangeSummary<i64>>
          ) -> NetSessionSummary<i64> {
            while seed > 0 {
              match mode {
                1 => {
                  return SessionSummary {
                    summary: join(summary_task),
                    session_value: 99,
                  };
                }
                _ => {
                  return SessionSummary {
                    summary: ExchangeSummary {
                      session: SessionLane {
                        exchange: ExchangeLane {
                          result: ResultEnvelope {
                            response: HttpResponse { body: Boxed(9), status: 503 },
                            recv_ready: false,
                          },
                          attempts: 1,
                        },
                        open: false,
                      },
                      exchange_value: 40,
                    },
                    session_value: 98,
                  };
                }
              }
            }
            return SessionSummary {
              summary: ExchangeSummary {
                session: SessionLane {
                  exchange: ExchangeLane {
                    result: ResultEnvelope {
                      response: HttpResponse { body: Boxed(8), status: 204 },
                      recv_ready: true,
                    },
                    attempts: 0,
                  },
                  open: true,
                },
                exchange_value: 39,
              },
              session_value: 97,
            };
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            return summary.summary.session.exchange.result.response.body.value;
          }

          fn main() -> i64 {
            let summary_task: Task<NetHttpClientExchangeSummary<i64>> =
              spawn(net_http_client_exchange(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let summary: NetSessionSummary<i64> = nested_loop_summary(1, 1, summary_task);
            return summarize_net_session(summary);
          }
        }
        "#,
    )
    .unwrap();

    let nested = module
        .functions
        .iter()
        .find(|function| function.name == "nested_loop_summary")
        .unwrap();
    assert!(matches!(
        nested.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If {
                    then_body,
                    else_body,
                    ..
                }] if matches!(
                    then_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                        if type_name == "SessionSummary"
                            && matches!(
                                type_args.as_slice(),
                                [ty]
                                    if ty.render()
                                        == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                            )
                            && matches!(
                                fields.as_slice(),
                                [
                                    (summary_field, NirExpr::CpuJoin(value)),
                                    (session_value_field, NirExpr::Int(99))
                                ]
                                    if summary_field == "summary"
                                        && session_value_field == "session_value"
                                        && matches!(
                                            value.as_ref(),
                                            NirExpr::Var(name) if name == "summary_task"
                                        )
                            )
                ) && matches!(
                    else_body.as_slice(),
                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                        if type_name == "SessionSummary"
                            && matches!(
                                type_args.as_slice(),
                                [ty]
                                    if ty.render()
                                        == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                            )
                )
            )
    ));
    assert!(matches!(
        nested.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. })))
            if type_name == "SessionSummary"
                && matches!(
                    type_args.as_slice(),
                    [ty]
                        if ty.render()
                            == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                )
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
        }) if name == "summary_task"
            && ty.render()
                == "Task<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "summary"
            && ty.render()
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
            && callee == "nested_loop_summary"
    ));
}

#[test]
fn monomorphizes_higher_order_nested_while_match_std_net_summary_session_flow() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type NetHttpRequest<T> = HttpRequest<Boxed<T>>;
          type NetHttpResponse<T> = HttpResponse<Boxed<T>>;
          type NetResult<T> = ResultEnvelope<NetHttpResponse<T>>;
          type NetHttpClientExchange<T> = ExchangeLane<NetResult<T>>;
          type NetSession<T> = SessionLane<NetHttpClientExchange<T>>;
          type NetHttpClientExchangeSummary<T> = ExchangeSummary<NetSession<T>>;
          type NetSessionSummary<T> = SessionSummary<NetHttpClientExchangeSummary<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct HttpRequest<T> {
            body: T,
            retry_budget: i64,
          }

          struct HttpResponse<T> {
            body: T,
            status: i64,
          }

          struct ResultEnvelope<T> {
            response: T,
            recv_ready: bool,
          }

          struct ExchangeLane<T> {
            result: T,
            attempts: i64,
          }

          struct SessionLane<T> {
            exchange: T,
            open: bool,
          }

          struct ExchangeSummary<T> {
            session: T,
            exchange_value: i64,
          }

          struct SessionSummary<T> {
            summary: T,
            session_value: i64,
          }

          fn apply(x: i64, f: Fn1<i64, i64>) -> i64 {
            return f(x);
          }

          async fn net_http_client_exchange<T>(
            request: NetHttpRequest<T>
          ) -> NetHttpClientExchangeSummary<T> {
            return ExchangeSummary {
              session: SessionLane {
                exchange: ExchangeLane {
                  result: ResultEnvelope {
                    response: HttpResponse {
                      body: request.body,
                      status: 200,
                    },
                    recv_ready: true,
                  },
                  attempts: request.retry_budget,
                },
                open: true,
              },
              exchange_value: 41,
            };
          }

          fn nested_loop_summary(
            seed: i64,
            mode: i64,
            summary_task: Task<NetHttpClientExchangeSummary<i64>>
          ) -> NetSessionSummary<i64> {
            while seed > 0 {
              match apply(mode, |x: i64| -> i64 { return x + 1; }) {
                2 => {
                  return SessionSummary {
                    summary: join(summary_task),
                    session_value: 99,
                  };
                }
                _ => {
                  return SessionSummary {
                    summary: ExchangeSummary {
                      session: SessionLane {
                        exchange: ExchangeLane {
                          result: ResultEnvelope {
                            response: HttpResponse { body: Boxed(9), status: 503 },
                            recv_ready: false,
                          },
                          attempts: 1,
                        },
                        open: false,
                      },
                      exchange_value: 40,
                    },
                    session_value: 98,
                  };
                }
              }
            }
            return SessionSummary {
              summary: ExchangeSummary {
                session: SessionLane {
                  exchange: ExchangeLane {
                    result: ResultEnvelope {
                      response: HttpResponse { body: Boxed(8), status: 204 },
                      recv_ready: true,
                    },
                    attempts: 0,
                  },
                  open: true,
                },
                exchange_value: 39,
              },
              session_value: 97,
            };
          }

          fn summarize_net_session<T>(summary: NetSessionSummary<T>) -> T {
            return summary.summary.session.exchange.result.response.body.value;
          }

          fn main() -> i64 {
            let summary_task: Task<NetHttpClientExchangeSummary<i64>> =
              spawn(net_http_client_exchange(NetHttpRequest {
                body: Boxed(7),
                retry_budget: 2,
              }));
            let summary: NetSessionSummary<i64> = nested_loop_summary(1, 1, summary_task);
            return summarize_net_session(summary);
          }
        }
        "#,
    )
    .unwrap();

    let nested = module
        .functions
        .iter()
        .find(|function| function.name == "nested_loop_summary")
        .unwrap();
    assert!(matches!(
        nested.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [
                    NirStmt::Let { name, value: NirExpr::Call { .. }, .. },
                    NirStmt::If { then_body, else_body, .. }
                ] if name.starts_with("__match_scrutinee_")
                    && matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                            if type_name == "SessionSummary"
                                && matches!(
                                    type_args.as_slice(),
                                    [ty]
                                        if ty.render()
                                            == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                                )
                                && matches!(
                                    fields.as_slice(),
                                    [
                                        (summary_field, NirExpr::CpuJoin(value)),
                                        (session_value_field, NirExpr::Int(99))
                                    ]
                                        if summary_field == "summary"
                                            && session_value_field == "session_value"
                                            && matches!(
                                                value.as_ref(),
                                                NirExpr::Var(task_name) if task_name == "summary_task"
                                            )
                                )
                    ) && matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                            if type_name == "SessionSummary"
                                && matches!(
                                    type_args.as_slice(),
                                    [ty]
                                        if ty.render()
                                            == "ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>"
                                )
                    )
            )
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
        }) if name == "summary_task"
            && ty.render()
                == "Task<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "summary"
            && ty.render()
                == "SessionSummary<ExchangeSummary<SessionLane<ExchangeLane<ResultEnvelope<HttpResponse<Boxed<i64>>>>>>>"
            && callee == "nested_loop_summary"
    ));
}

#[test]
fn monomorphizes_higher_order_generic_mapper_with_explicit_helper_chain() {
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

          fn apply_packetized<T>(
            value: T,
            mapper: Fn1<T, EnvelopeAlias<PacketAlias<CellAlias<T>>>>
          ) -> EnvelopeAlias<PacketAlias<CellAlias<T>>> {
            return mapper(value);
          }

          async fn produce_seed() -> i64 {
            return 7;
          }

          fn main() -> i64 {
            let task: Task<i64> = spawn(produce_seed());
            let seed: i64 = join(task);
            let selected: EnvelopeAlias<PacketAlias<CellAlias<i64>>> =
              apply_packetized(seed, |x: i64| -> EnvelopeAlias<PacketAlias<CellAlias<i64>>> {
                let cell: CellAlias<i64> = CellAlias { value: x };
                if x > 0 {
                  let packet: PacketAlias<CellAlias<i64>> =
                    wrap_packet<CellAlias<i64>>(cell, 6);
                  return wrap_envelope<PacketAlias<CellAlias<i64>>>(packet, true);
                }
                let packet: PacketAlias<CellAlias<i64>> =
                  wrap_packet<CellAlias<i64>>(cell, 1);
                return wrap_envelope<PacketAlias<CellAlias<i64>>>(packet, false);
              });
            return selected.packet.payload.value + selected.packet.tag;
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
            value: NirExpr::CpuSpawn { .. },
        }) if name == "task" && ty.render() == "Task<i64>"
    ));
    assert!(matches!(
        main.body.get(1),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::CpuJoin(_),
        }) if name == "seed" && ty.render() == "i64"
    ));
    assert!(matches!(
        main.body.get(2),
        Some(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::Call { callee, .. },
        }) if name == "selected"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee.starts_with("__hof_apply_packetized")
    ));

    let specialized_hof = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__hof_apply_packetized"))
        .unwrap();
    assert!(specialized_hof.generic_params.is_empty());
    assert!(matches!(
        specialized_hof.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .unwrap();
    assert!(lambda.generic_params.is_empty());
    assert!(matches!(
        lambda.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Packet<Cell<i64>>>"
    ));
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_packet__")
    }));
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));
}

#[test]
fn monomorphizes_higher_order_generic_mapper_from_field_access_arguments_without_typed_locals() {
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

          fn apply_packetized<T>(
            payload: T,
            tag: i64,
            mapper: Fn2<T, i64, EnvelopeAlias<PacketAlias<T>>>
          ) -> EnvelopeAlias<PacketAlias<T>> {
            return mapper(payload, tag);
          }

          fn main() -> i64 {
            let packet: PacketAlias<CellAlias<i64>> =
              wrap_packet<CellAlias<i64>>(wrap_cell<i64>(7), 9);
            let selected: EnvelopeAlias<PacketAlias<CellAlias<i64>>> =
              apply_packetized(
                packet.payload,
                packet.tag,
                |payload: CellAlias<i64>, tag: i64| -> EnvelopeAlias<PacketAlias<CellAlias<i64>>> {
                  if tag > 0 {
                    return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                      wrap_packet<CellAlias<i64>>(payload, tag),
                      true
                    );
                  }
                  return wrap_envelope<PacketAlias<CellAlias<i64>>>(
                    wrap_packet<CellAlias<i64>>(payload, 0),
                    false
                  );
                }
              );
            return selected.packet.payload.value + selected.packet.tag;
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
            value: NirExpr::Call { callee, .. },
        }) if name == "selected"
            && ty.render() == "Envelope<Packet<Cell<i64>>>"
            && callee.starts_with("__hof_apply_packetized")
    ));

    let lambda = module
        .functions
        .iter()
        .find(|function| function.name.starts_with("__lambda_main_"))
        .unwrap();
    assert!(lambda.generic_params.is_empty());
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_packet__")
    }));
    assert!(stmt_tree_contains_call(&lambda.body, &|callee, _| {
        callee.starts_with("wrap_envelope__")
    }));
}

#[test]
fn monomorphizes_continue_branch_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(flag: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              if flag {
                continue;
              } else {
                return Summary {
                  response: join(task),
                  code: 7,
                };
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let produced = module
        .functions
        .iter()
        .find(|function| function.name == "produce_response__i64")
        .unwrap();
    assert!(produced.is_async);
    assert!(produced.generic_params.is_empty());
    assert!(matches!(
        produced.return_type.as_ref().map(|ty| ty.render()),
        Some(rendered) if rendered == "Envelope<Boxed<i64>>"
    ));

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(then_body.as_slice(), [NirStmt::Continue])
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
            )
    ));
    assert!(matches!(
        route.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. })))
            if type_name == "SessionSummary"
                && matches!(type_args.as_slice(), [ty] if ty.render() == "Envelope<Boxed<i64>>")
    ));
}

#[test]
fn monomorphizes_break_branch_before_generic_summary_fallback_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(flag: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              if flag {
                break;
              } else {
                return Summary {
                  response: join(task),
                  code: 7,
                };
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(then_body.as_slice(), [NirStmt::Break])
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
            )
    ));
    assert!(matches!(
        route.body.get(1),
        Some(NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields })))
            if type_name == "SessionSummary"
                && matches!(type_args.as_slice(), [ty] if ty.render() == "Envelope<Boxed<i64>>")
                && matches!(
                    fields.as_slice(),
                    [
                        (response_field, NirExpr::StructLiteral { type_name, type_args, .. }),
                        (code_field, NirExpr::Int(8))
                    ]
                        if response_field == "response"
                            && code_field == "code"
                            && type_name == "Envelope"
                            && matches!(type_args.as_slice(), [ty] if ty.render() == "Boxed<i64>")
                )
    ));
}

#[test]
fn monomorphizes_guarded_match_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(mode: i64, ready: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              match mode {
                2 if ready => {
                  return Summary {
                    response: join(task),
                    code: 7,
                  };
                }
                _ => {
                  return Summary {
                    response: Response { payload: Boxed(9), ready: false },
                    code: 8,
                  };
                }
              }
            }
            return Summary {
              response: Response { payload: Boxed(10), ready: true },
              code: 9,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(2, true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { condition, then_body, else_body }]
                    if matches!(condition, NirExpr::Binary { .. })
                        && matches!(
                            then_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                        )
            )
    ));
}

#[test]
fn monomorphizes_nested_match_continue_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(mode: i64, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              match mode {
                0 => {
                  continue;
                }
                _ => {
                  return Summary {
                    response: join(task),
                    code: 7,
                  };
                }
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(1, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(then_body.as_slice(), [NirStmt::Continue])
                        && matches!(
                            else_body.as_slice(),
                            [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                if type_name == "SessionSummary"
                                    && matches!(
                                        type_args.as_slice(),
                                        [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                    )
                                    && matches!(
                                        fields.as_slice(),
                                        [
                                            (response_field, NirExpr::CpuJoin(value)),
                                            (code_field, NirExpr::Int(7))
                                        ]
                                            if response_field == "response"
                                                && code_field == "code"
                                                && matches!(
                                                    value.as_ref(),
                                                    NirExpr::Var(task_name) if task_name == "task"
                                                )
                                    )
                        )
            )
    ));
}

#[test]
fn monomorphizes_nested_if_break_before_generic_summary_join_in_while() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          type Response<T> = Envelope<Boxed<T>>;
          type Summary<T> = SessionSummary<Response<T>>;

          struct Boxed<T> {
            value: T,
          }

          struct Envelope<T> {
            payload: T,
            ready: bool,
          }

          struct SessionSummary<T> {
            response: T,
            code: i64,
          }

          async fn produce_response<T>() -> Response<T> {
            return Response { payload: Boxed(7), ready: true };
          }

          fn route(flag: bool, seed: i64, task: Task<Response<i64>>) -> Summary<i64> {
            while seed > 0 {
              if flag {
                if seed > 1 {
                  break;
                } else {
                  return Summary {
                    response: join(task),
                    code: 7,
                  };
                }
              } else {
                return Summary {
                  response: Response { payload: Boxed(8), ready: false },
                  code: 6,
                };
              }
            }
            return Summary {
              response: Response { payload: Boxed(9), ready: false },
              code: 8,
            };
          }

          fn main() -> i64 {
            let task: Task<Response<i64>> = spawn(produce_response());
            let summary: Summary<i64> = route(true, 1, task);
            return summary.response.payload.value;
          }
        }
        "#,
    )
    .unwrap();

    let route = module
        .functions
        .iter()
        .find(|function| function.name == "route")
        .unwrap();
    assert!(matches!(
        route.body.first(),
        Some(NirStmt::While { body, .. })
            if matches!(
                body.as_slice(),
                [NirStmt::If { then_body, else_body, .. }]
                    if matches!(
                        then_body.as_slice(),
                        [NirStmt::If { then_body, else_body, .. }]
                            if matches!(then_body.as_slice(), [NirStmt::Break])
                                && matches!(
                                    else_body.as_slice(),
                                    [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, fields }))]
                                        if type_name == "SessionSummary"
                                            && matches!(
                                                type_args.as_slice(),
                                                [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                            )
                                            && matches!(
                                                fields.as_slice(),
                                                [
                                                    (response_field, NirExpr::CpuJoin(value)),
                                                    (code_field, NirExpr::Int(7))
                                                ]
                                                    if response_field == "response"
                                                        && code_field == "code"
                                                        && matches!(
                                                            value.as_ref(),
                                                            NirExpr::Var(task_name) if task_name == "task"
                                                        )
                                            )
                                )
                    ) && matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::StructLiteral { type_name, type_args, .. }))]
                            if type_name == "SessionSummary"
                                && matches!(
                                    type_args.as_slice(),
                                    [ty] if ty.render() == "Envelope<Boxed<i64>>"
                                )
                    )
            )
    ));
}

#[test]
fn monomorphizes_generic_thread_and_mutex_helpers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct MutexSnapshot<T> {
            value: T,
            lock: Mutex<T>,
          }

          async fn ping(seed: i64) -> i64 {
            return seed + 9;
          }

          fn mutex_snapshot<T>(lock: Mutex<T>) -> MutexSnapshot<T> {
            let guard: MutexGuard<T> = mutex_lock(lock);
            let value: T = mutex_value(guard);
            let reopened: Mutex<T> = mutex_unlock(guard);
            return MutexSnapshot {
              value: value,
              lock: reopened,
            };
          }

          fn join_thread_result<T>(worker: Thread<T>) -> TaskResult<T> {
            return thread_join_result(worker);
          }

          fn main() -> i64 {
            let snapshot: MutexSnapshot<i64> = mutex_snapshot(mutex_new(7));
            let joined: TaskResult<i64> =
              join_thread_result(thread_spawn(ping(snapshot.value)));
            if task_completed(joined) {
              return task_value(joined);
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
    assert!(stmt_tree_contains_call(&main.body, &|callee, _| callee == "mutex_snapshot__i64"));
    assert!(stmt_tree_contains_call(&main.body, &|callee, _| {
        callee == "join_thread_result__i64"
    }));

    let snapshot = module
        .functions
        .iter()
        .find(|function| function.name == "mutex_snapshot__i64")
        .expect("expected specialized mutex snapshot helper");
    assert!(snapshot.generic_params.is_empty());
    assert!(snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexLock(_),
            } if name == "guard" && ty.render() == "MutexGuard<i64>"
        )
    }));
    assert!(snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexValue(_),
            } if name == "value" && ty.render() == "i64"
        )
    }));
    assert!(snapshot.body.iter().any(|stmt| {
        matches!(
            stmt,
            NirStmt::Let {
                name,
                ty: Some(ty),
                value: NirExpr::CpuMutexUnlock(_),
            } if name == "reopened" && ty.render() == "Mutex<i64>"
        )
    }));

    let joiner = module
        .functions
        .iter()
        .find(|function| function.name == "join_thread_result__i64")
        .expect("expected specialized thread join helper");
    assert!(joiner.generic_params.is_empty());
    assert!(matches!(
        joiner.body.last(),
        Some(NirStmt::Return(Some(NirExpr::CpuThreadJoinResult(value))))
            if matches!(value.as_ref(), NirExpr::Var(name) if name == "worker")
    ));
}
