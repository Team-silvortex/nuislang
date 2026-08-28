use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_nested_value_struct_parameters_through_flattened_helper_abi() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Pair {
            left: i64,
            right: i64
          }

          struct Envelope {
            ready: bool,
            pair: Pair
          }

          fn shift(value: Envelope, delta: i64) -> Envelope {
            return Envelope {
              ready: value.ready,
              pair: Pair {
                left: value.pair.left + delta,
                right: value.pair.right + delta
              }
            };
          }

          fn unused_shift(value: Envelope) -> Envelope {
            return value;
          }

          fn main() -> i64 {
            let source: Envelope = Envelope {
              ready: true,
              pair: Pair { left: 4, right: 7 }
            };
            let shifted: Envelope = shift(source, 3);
            if shifted.ready {
              return shifted.pair.left + shifted.pair.right;
            }
            return 0;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let helper = yir
        .functions
        .iter()
        .find(|function| function.name == "shift")
        .expect("aggregate helper boundary");
    assert_eq!(helper.parameters.len(), 4);
    assert_eq!(
        helper
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        [
            "value.ready",
            "value.pair.left",
            "value.pair.right",
            "delta"
        ]
    );
    assert!(yir.nodes.iter().any(|node| {
        node.op.instruction == "call_owned_struct"
            && node.op.args.first().is_some_and(|callee| callee == "shift")
            && node.op.args.len() == 6
    }));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:shift"));
    assert!(yir
        .functions
        .iter()
        .all(|function| function.name != "unused_shift"));
}

#[test]
fn lowers_result_of_owned_struct_through_variant_union_helper_abi() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Result<T, E> {
            Ok(T),
            Err(E)
          }

          struct Packet {
            value: i64,
            ready: bool
          }

          fn checked(packet: Packet, accept: bool) -> Result<Packet, i64> {
            if accept {
              return Result.Ok(packet);
            }
            return Result.Err(9);
          }

          fn main() -> i64 {
            let packet: Packet = Packet { value: 33, ready: true };
            match checked(packet, true) {
              Result.Ok(value) => {
                if value.ready { return value.value; }
                return 0;
              }
              Result.Err(error) => { return error; }
            }
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let returned = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.instruction == "return_owned_struct"
                && yir
                    .node_lanes
                    .get(&node.name)
                    .is_some_and(|lane| lane == "fn:checked")
        })
        .expect("owned Result helper return");
    assert_eq!(returned.op.args.len(), 2);
    assert!(returned.op.args[1].starts_with("__nuis_variant_union__Result{"));
    assert!(yir.nodes.iter().any(|node| {
        node.op.instruction == "call_owned_struct"
            && node
                .op
                .args
                .first()
                .is_some_and(|callee| callee == "checked")
            && node.op.args[1].starts_with("__nuis_variant_union__Result{")
    }));
    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned Result ABI LLVM lowering");
    assert!(llvm_ir.contains("define i64 @nuis_fn_checked("));
    assert!(llvm_ir.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.return_owned_struct"));
}

#[test]
fn lowers_bootstrap_scanner_unit_enum_guard_through_variant_union_abi() {
    let module = parse_nuis_module(include_str!(
        "../../../../tests/fixtures/bootstrap/accepted/compiler_scanner.ns"
    ))
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let guarded = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.instruction == "guard_return"
                && node
                    .op
                    .args
                    .get(2)
                    .is_some_and(|layout| layout.contains("__nuis_variant_union__ScanError{"))
        })
        .expect("guarded Result return with nested ScanError layout");
    assert_eq!(guarded.op.args.len(), 3);
    assert!(guarded.op.args[2].contains("__nuis_variant_union__ScanError{"));
    assert!(guarded.op.args[2].contains("ScanError.InvalidRange{}"));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("bootstrap scanner ABI lowering");
    assert!(llvm_ir.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.guard_return"));
}
