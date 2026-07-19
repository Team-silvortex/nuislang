use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_void_main_print_into_explicit_zero_entry_return() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            print(42);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let has_print = yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "print");
    let has_implicit_zero = yir.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "const_i64"
            && node.op.args == ["0".to_owned()]
            && node.name.starts_with("implicit_main_return_value_")
    });
    let has_implicit_return = yir.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "return_i64"
            && node
                .op
                .args
                .first()
                .is_some_and(|arg| arg.starts_with("implicit_main_return_value_"))
            && node.name.starts_with("implicit_main_return_")
    });

    assert!(
        has_print,
        "expected void main to preserve print side effect"
    );
    assert!(has_implicit_zero, "expected void main to exit with zero");
    assert!(
        has_implicit_return,
        "expected void main zero value to feed an explicit return_i64"
    );
}

#[test]
fn lowers_ordinary_self_recursive_function_into_helper_lane_and_call_i64() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn fact(current: i64) -> i64 {
            if current <= 1 {
              return 1;
            }
            return current * fact(current - 1);
          }

          fn main() -> i64 {
            return fact(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "call_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "param_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "return_i64"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:fact"));
}

#[test]
fn lowers_ordinary_self_recursive_function_into_helper_lane_and_call_bool() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn settle(flag: bool) -> bool {
            if flag == true {
              return false;
            }
            return settle(true);
          }

          fn main() -> i64 {
            let result: bool = settle(false);
            if result == false {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "call_bool"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "param_bool"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "return_bool"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:settle"));
}

#[test]
fn lowers_mutually_recursive_functions_into_helper_lanes_and_call_i64() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(value - 1);
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(value - 1);
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let call_i64_count = yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == "call_i64")
        .count();
    assert!(
        call_i64_count >= 2,
        "expected mutual recursion calls, found {call_i64_count}"
    );
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:odd"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:even"));
}

#[test]
fn lowers_recursive_component_reachable_scalar_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step(value: i64) -> i64 {
            return value - 1;
          }

          fn odd(value: i64) -> i64 {
            if value == 0 {
              return 0;
            }
            return even(step(value));
          }

          fn even(value: i64) -> i64 {
            if value == 0 {
              return 1;
            }
            return odd(step(value));
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    let step_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    assert!(
        step_calls >= 2,
        "expected calls into helper-lowered `step`, found {step_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_bool_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn flip(flag: bool) -> bool {
            if flag == true {
              return false;
            }
            return true;
          }

          fn odd(flag: bool) -> bool {
            if flag == true {
              return false;
            }
            return even(flip(flag));
          }

          fn even(flag: bool) -> bool {
            if flag == true {
              return true;
            }
            return odd(flip(flag));
          }

          fn main() -> i64 {
            if even(false) == true {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:flip"));
    let flip_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "flip")
        })
        .count();
    assert!(
        flip_calls >= 2,
        "expected calls into helper-lowered `flip`, found {flip_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_i32_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step(value: i32) -> i32 {
            return value - i32_from_i64(1);
          }

          fn odd(value: i32) -> i32 {
            if value == i32_from_i64(0) {
              return i32_from_i64(0);
            }
            return even(step(value));
          }

          fn even(value: i32) -> i32 {
            if value == i32_from_i64(0) {
              return i32_from_i64(1);
            }
            return odd(step(value));
          }

          fn main() -> i64 {
            let result: i32 = even(i32_from_i64(4));
            if result == i32_from_i64(1) {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    let step_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    assert!(
        step_calls >= 2,
        "expected calls into helper-lowered `step`, found {step_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_mixed_scalar_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn is_zero(value: i32) -> bool {
            if value == i32_from_i64(0) {
              return true;
            }
            return false;
          }

          fn step(value: i32) -> i32 {
            return value - i32_from_i64(1);
          }

          fn odd(value: i32) -> bool {
            if is_zero(value) == true {
              return false;
            }
            return even(step(value));
          }

          fn even(value: i32) -> bool {
            if is_zero(value) == true {
              return true;
            }
            return odd(step(value));
          }

          fn main() -> i64 {
            if even(i32_from_i64(4)) == true {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:is_zero"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    let bool_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "is_zero")
        })
        .count();
    let i32_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    assert!(
        bool_calls >= 2,
        "expected calls into helper-lowered `is_zero`, found {bool_calls}"
    );
    assert!(
        i32_calls >= 1,
        "expected calls into helper-lowered `step`, found {i32_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_tri_scalar_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step(value: i32) -> i32 {
            return value - i32_from_i64(1);
          }

          fn near_zero(value: i64) -> bool {
            let narrowed: i32 = i32_from_i64(value);
            if step(narrowed) == i32_from_i64(0) {
              return true;
            }
            return false;
          }

          fn odd(value: i64) -> i64 {
            if near_zero(value) == true {
              return 0;
            }
            return even(value - 1);
          }

          fn even(value: i64) -> i64 {
            if near_zero(value) == true {
              return 1;
            }
            return odd(value - 1);
          }

          fn main() -> i64 {
            return even(4);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:near_zero"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:odd"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:even"));
    let bool_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_bool"
                && node.op.args.first().is_some_and(|name| name == "near_zero")
        })
        .count();
    let i32_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    let i64_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_i64"
                && node
                    .op
                    .args
                    .first()
                    .is_some_and(|name| name == "odd" || name == "even")
        })
        .count();
    assert!(
        bool_calls >= 2,
        "expected calls into helper-lowered `near_zero`, found {bool_calls}"
    );
    assert!(
        i32_calls >= 1,
        "expected calls into helper-lowered `step`, found {i32_calls}"
    );
    assert!(
        i64_calls >= 2,
        "expected recursive i64 helper calls, found {i64_calls}"
    );
}

#[test]
fn lowers_recursive_component_reachable_f32_helpers_into_helper_lanes() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn step(value: f32) -> f32 {
            return value - 1.0;
          }

          fn cool(value: f32) -> f32 {
            if value <= 1.0 {
              return 0.5;
            }
            return warm(step(value));
          }

          fn warm(value: f32) -> f32 {
            if value <= 1.0 {
              return 1.5;
            }
            return cool(step(value));
          }

          fn main() -> i64 {
            let result: f32 = warm(4.0);
            if result > 1.0 {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:step"));
    let f32_calls = yir
        .nodes
        .iter()
        .filter(|node| {
            node.op.module == "cpu"
                && node.op.instruction == "call_f32"
                && node.op.args.first().is_some_and(|name| name == "step")
        })
        .count();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "param_f32"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "return_f32"));
    assert!(
        f32_calls >= 2,
        "expected calls into helper-lowered `step`, found {f32_calls}"
    );
}

#[test]
fn lowers_ordinary_self_recursive_function_into_helper_lane_and_call_f64() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn decay(current: f64) -> f64 {
            if current <= 1.0 {
              return 1.0;
            }
            return current * decay(current - 1.0);
          }

          fn main() -> i64 {
            let result: f64 = decay(4.0);
            if result > 1.0 {
              return 7;
            }
            return 9;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "call_f64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "param_f64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "return_f64"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:decay"));
}

#[test]
fn lowers_spawned_nested_struct_async_helper_into_owned_call_lane() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          struct Metrics {
            weight: f32,
            score: i64,
            label: String
          }

          struct Packet {
            code: i64,
            ready: bool,
            metrics: Metrics
          }

          async fn make_packet(code: i64, ready: bool) -> Packet {
            return Packet {
              code: code,
              ready: ready,
              metrics: Metrics { weight: 2.5, score: 7, label: "packet" }
            };
          }

          fn main() -> i64 {
            let packet: Packet = join(spawn(make_packet(31, true)));
            return packet.code;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let call = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "call_owned_struct")
        .expect("spawned nested struct helper should retain an owned call node");
    assert_eq!(call.op.args[0], "make_packet");
    assert_eq!(
        call.op.args[1],
        "Packet{code:i64;ready:bool;metrics:Metrics{weight:f32;score:i64;label:String}}"
    );
    let returned = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "return_owned_struct")
        .expect("nested struct helper should retain an owned return node");
    assert_eq!(
        yir.node_lanes.get(&returned.name).map(String::as_str),
        Some("fn:make_packet")
    );
}

#[test]
fn lowers_owned_bytes_move_through_direct_helper_without_copying() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn consume(bytes: Bytes, seed: i64) -> i64 {
            let len: i64 = bytes_len(bytes);
            drop_bytes(bytes);
            return seed + len;
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let result: i64 = consume(move(bytes), 3);
            free(buffer);
            return result;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "move_owned_bytes"));
    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned move direct call should lower");
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        1,
        "moving an owned blob must not allocate another copy"
    );
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        1
    );
    assert!(!llvm_ir.contains("deferred lowering for cpu.move_owned_bytes"));
}

#[test]
fn returns_owned_bytes_from_direct_helper_to_new_owner() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn relay(bytes: Bytes) -> Bytes {
            return relay(move(bytes));
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(2, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let returned: Bytes = relay(move(bytes));
            let len: i64 = bytes_len(returned);
            drop_bytes(returned);
            free(buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "return_owned_bytes"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "call_owned_bytes"));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned Bytes return should lower");
    assert!(llvm_ir.contains("define ptr @nuis_fn_relay(ptr %arg0)"));
    assert!(llvm_ir.contains("call ptr @nuis_fn_relay(ptr %"));
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        1
    );
    assert!(!llvm_ir.contains("deferred lowering for cpu.call_owned_bytes"));
}

#[test]
fn returns_same_owned_bytes_owner_through_dynamic_if() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose(bytes: Bytes, enabled: bool) -> Bytes {
            if enabled {
              return move(bytes);
            } else {
              return move(bytes);
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(3, 7);
            let bytes: Bytes = copy_bytes(buffer);
            let choice: i64 = cpu_input_i64("choice", 1, 0, 1, 1);
            let returned: Bytes = choose(move(bytes), choice == 1);
            let len: i64 = bytes_len(returned);
            drop_bytes(returned);
            free(buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let selected = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes")
        .expect("conditional owned return should use the owned select contract");
    assert_eq!(selected.op.args[1], selected.op.args[2]);

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("owned select should lower");
    assert!(llvm_ir.contains("select i1"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes"));
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        1
    );
}

#[test]
fn lowers_dynamic_if_selecting_distinct_owned_bytes_with_branch_cleanup() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn choose(lhs: Bytes, rhs: Bytes, enabled: bool) -> Bytes {
            if enabled {
              return move(lhs);
            } else {
              return move(rhs);
            }
          }

          fn main() -> i64 {
            let lhs_buffer: ref Buffer = alloc_buffer(1, 3);
            let rhs_buffer: ref Buffer = alloc_buffer(1, 7);
            let lhs: Bytes = copy_bytes(lhs_buffer);
            let rhs: Bytes = copy_bytes(rhs_buffer);
            let choice: i64 = cpu_input_i64("choice", 1, 0, 1, 1);
            let returned: Bytes = choose(move(lhs), move(rhs), choice == 1);
            let len: i64 = bytes_len(returned);
            drop_bytes(returned);
            free(rhs_buffer);
            free(lhs_buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let select = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "select_owned_bytes_drop_unselected")
        .expect("distinct owned select node");
    assert_ne!(select.op.args[1], select.op.args[2]);

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("distinct owned select should lower");
    assert!(llvm_ir.contains("select_owned_cleanup_then"));
    assert!(llvm_ir.contains("select_owned_cleanup_else"));
    assert!(llvm_ir.contains("select_owned_cleanup_merge"));
    assert!(llvm_ir.contains(" = phi ptr [ "));
    assert!(!llvm_ir.contains("deferred lowering for cpu.select_owned_bytes_drop_unselected"));
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        2
    );
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        3
    );
}

#[test]
fn lowers_branch_local_owned_helpers_into_real_llvm_call_blocks() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn keep_left(bytes: Bytes, delta: i64) -> Bytes {
            return move(bytes);
          }

          fn keep_right(bytes: Bytes, factor: i64) -> Bytes {
            return move(bytes);
          }

          fn choose(bytes: Bytes, enabled: bool) -> Bytes {
            if enabled {
              return keep_left(move(bytes), 3);
            } else {
              return keep_right(move(bytes), 7);
            }
          }

          fn main() -> i64 {
            let buffer: ref Buffer = alloc_buffer(4, 9);
            let bytes: Bytes = copy_bytes(buffer);
            let choice: i64 = cpu_input_i64("choice", 1, 0, 1, 1);
            let returned: Bytes = choose(move(bytes), choice == 1);
            let len: i64 = bytes_len(returned);
            drop_bytes(returned);
            free(buffer);
            return len;
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let branch_call = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "branch_call_owned_bytes")
        .expect("conditional owned helper calls should retain one branch call node");
    assert_eq!(branch_call.op.args[1], "keep_left");
    assert_eq!(branch_call.op.args[2], "keep_right");
    assert_eq!(branch_call.op.args[4], "1");
    assert_eq!(branch_call.op.args[6], "1");
    let then_scalar = yir
        .nodes
        .iter()
        .find(|node| node.name == branch_call.op.args[5])
        .expect("then scalar argument source");
    let else_scalar = yir
        .nodes
        .iter()
        .find(|node| node.name == branch_call.op.args[7])
        .expect("else scalar argument source");
    assert_eq!(then_scalar.op.args, ["3"]);
    assert_eq!(else_scalar.op.args, ["7"]);
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:keep_left"));
    assert!(yir.node_lanes.values().any(|lane| lane == "fn:keep_right"));

    let llvm_ir = yir_lower_llvm::emit_module(&yir).expect("branch-local owned calls should lower");
    assert!(llvm_ir.contains("branch_owned_call_then"));
    assert!(llvm_ir.contains("branch_owned_call_else"));
    assert!(llvm_ir.contains("branch_owned_call_merge"));
    let then_call = llvm_ir
        .lines()
        .find(|line| line.contains("call ptr @nuis_fn_keep_left(ptr"))
        .expect("then branch helper call");
    let else_call = llvm_ir
        .lines()
        .find(|line| line.contains("call ptr @nuis_fn_keep_right(ptr"))
        .expect("else branch helper call");
    assert!(then_call.contains(", i64 "));
    assert!(else_call.contains(", i64 "));
    assert!(llvm_ir.contains(" = phi ptr ["));
    assert!(!llvm_ir.contains("deferred lowering for cpu.branch_call_owned_bytes"));
    assert_eq!(
        llvm_ir
            .matches("call ptr @nuis_scheduler_owned_blob_copy_v1")
            .count(),
        1
    );
    assert_eq!(
        llvm_ir
            .matches("call void @nuis_scheduler_owned_blob_drop_v1")
            .count(),
        1
    );
}
