use super::*;
use scalar_helpers::{assert_traps, with_helpers};

fn source() -> String {
    with_helpers(&SOURCE
        .replace("let index: i64 = 0;", "let total: i64 = 1; let checksum: i64 = 2; let index: i64 = 0;")
        .replace("let index: i64 = index + 1;", "let total: i64 = accumulate(total, load_at(buffer, index)); let checksum: i64 = checksum + total; let index: i64 = index + 1;")
        .replace("let result: i64 = load_at(buffer, 0) + load_at(buffer, 7) + index;", "let result: i64 = (total + checksum) % 200;"),
        "fn accumulate(total: i64, value: i64) -> i64 { if value < 0 { return 1 / (value - value); } return total + value; }")
}

#[test]
fn multiple_carries_preserve_sequential_updates_zero_one_and_descending_trips() {
    let source = source();
    // total=117; checksum=2+5+12+22+35+51+70+92+117=406.
    check_execution(&source, 123);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        3,
    );
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 7;"),
        54,
    );
    // Descending values 25,22,19,16,13,10,7: total=113, checksum=541.
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        54,
    );
}

#[test]
fn multiple_carries_feed_writes_and_three_slots_are_not_precombined() {
    let source = source().replace("seed + index * 3", "total + checksum + index");
    let mut total = 1;
    let mut checksum = 2;
    for index in 0..8 {
        total += total + checksum + index;
        checksum += total;
    }
    check_execution(&source, (total + checksum) % 200);
    let source = source
        .replace(
            "let index: i64 = 0;",
            "let history: i64 = 3; let index: i64 = 0;",
        )
        .replace(
            "let index: i64 = index + 1;",
            "let history: i64 = history + checksum + total; let index: i64 = index + 1;",
        )
        .replace(
            "(total + checksum) % 200",
            "(total + checksum + history) % 200",
        );
    let (mut total, mut checksum, mut history) = (1, 2, 3);
    for index in 0..8 {
        total += total + checksum + index;
        checksum += total;
        history += checksum + total;
    }
    check_execution(&source, (total + checksum + history) % 200);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        6,
    );
}

#[test]
fn multiple_carries_replacing_updates_and_subsequent_loops_use_projected_values() {
    let replaced = source().replace("checksum + total", "value + total");
    check_execution(&replaced, 59);
    check_execution(
        &replaced.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        3,
    );
    let subsequent = source().replace(
        "let result: i64 =",
        concat!(
            "let index: i64 = 0; while index < 8 { ",
            "store_at(buffer, index, checksum); let checksum: i64 = checksum + total; ",
            "let total: i64 = total + 1; let index: i64 = index + 1; } let result: i64 ="
        ),
    );
    // Second loop: checksum += 117..124, total=125.
    check_execution(&subsequent, 95);
}

#[test]
fn multiple_carry_traps_preserve_source_order_across_writes_and_updates() {
    for (first, second, diagnostic) in [
        ("load_at(buffer, 8)", "1 / (index - index)", "index"),
        ("1 / (index - index)", "load_at(buffer, 8)", "zero"),
        (
            "value",
            "(0 - 9223372036854775807 - 1) / (0 - 1)",
            "overflow",
        ),
    ] {
        assert_traps(
            &source()
                .replace("accumulate(total, load_at(buffer, index))", first)
                .replace("checksum + total", second),
            diagnostic,
        );
    }
    assert_traps(
        &source()
            .replace(
                "store_at(buffer, index, value)",
                "store_at(buffer, 8, value)",
            )
            .replace("checksum + total", "1 / (index - index)"),
        "index",
    );
}

#[test]
fn multiple_carries_reject_non_scalar_updates_and_mutable_headers() {
    for source in [
        source().replace("index < 8", "index < checksum"),
        source().replace(
            "let checksum: i64 = checksum + total;",
            "if index > 0 { let checksum: bool = true; }",
        ),
        source().replace(
            "let checksum: i64 = checksum + total;",
            "let checksum: i64 = checksum + total; let buffer: ref Buffer = alloc_buffer(8, 0);",
        ),
        source().replace(
            "store_at(buffer, index, value);",
            "free(buffer); store_at(buffer, index, value);",
        ),
    ] {
        assert!(
            nuisc::pipeline::compile_source(&source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn multiple_carries_callbacks_share_fuel_without_committing_failed_state() {
    let source = source();
    let project = Project::new(&source);
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let registry = yir_verify::default_registry();
    for fuel_failure in [false, true] {
        let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
            checkpoint.yir(),
            &registry,
            "counter",
            vec![yir_core::Value::Int(4)],
        )
        .unwrap();
        session.event(vec![yir_core::Value::Int(0)]).unwrap();
        let yir_core::Value::Struct(state) = session.state() else {
            panic!("state");
        };
        assert_eq!(state.fields[0].1, yir_core::Value::Int(123));
        let last = session.state().clone();
        let error = if fuel_failure {
            session
                .event_budgeted(vec![yir_core::Value::Int(0)], 80)
                .unwrap_err()
        } else {
            session.event(vec![yir_core::Value::Int(-200)]).unwrap_err()
        };
        assert!(
            error.contains(if fuel_failure {
                "step budget exhausted"
            } else {
                "zero"
            }),
            "{error}"
        );
        assert_eq!(session.state(), &last);
        assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
        session.close(vec![]).unwrap();
        assert!(session.completion_status().is_err());
    }
}

#[test]
fn multiple_carries_keep_glm_seeds_and_run_independent_of_declaration_order() {
    let source = source();
    let mut compiled = nuisc::pipeline::compile_source(&source).unwrap();
    let node = compiled
        .yir
        .nodes
        .iter()
        .find(|node| node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries"))
        .unwrap();
    let carries = yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
        .unwrap()
        .unwrap();
    assert_eq!(carries.seeds.len(), 2);
    let profile = yir_core::glm_profile_for_operation(&node.op);
    for seed in carries.seeds {
        assert!(profile.accesses.iter().any(|access| access.input == seed));
        assert!(compiled
            .yir
            .edges
            .iter()
            .any(|edge| edge.from == seed && edge.to == node.name));
    }
    assert!(profile
        .accesses
        .iter()
        .all(|access| !access.input.starts_with('$')));
    compiled.yir.nodes.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    compiled.llvm_ir = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let result = compiled
        .yir
        .functions
        .iter()
        .find(|function| function.role == yir_core::YirFunctionRole::Entry)
        .unwrap()
        .result
        .as_ref()
        .unwrap();
    assert_eq!(trace.values[&result.node], yir_core::Value::Int(123));
    assert_eq!(native_run(&source, &compiled).status.code(), Some(123));
}

#[test]
fn multiple_carries_fail_closed_on_layout_signature_and_seed_drift() {
    let compiled = nuisc::pipeline::compile_source(&source()).unwrap();
    let index = compiled
        .yir
        .nodes
        .iter()
        .position(|node| node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries"))
        .unwrap();
    for (argument, value) in [
        (7, "0"),
        (9, "Wrong{carry0:i64;carry1:bool}"),
        (10, "$unknown"),
    ] {
        let mut module = compiled.yir.clone();
        module.nodes[index].op.args[argument] = value.to_owned();
        assert!(yir_verify::verify_module(&module).is_err());
        assert!(yir_lower_llvm::emit_module(&module).is_err());
    }
    // A syntactically valid but different callee layout must fail before a native read.
    for layout in [
        "Wrong{carry0:i64;carry1:i64}",
        "Wrong{carry0:i64;carry1:i64;carry2:i64}",
    ] {
        let mut module = compiled.yir.clone();
        let callee = module.nodes[index].op.args[8].clone();
        let returned = module
            .nodes
            .iter_mut()
            .find(|node| node.name == format!("__fn_{callee}_return"))
            .unwrap();
        returned.op.args[1] = layout.to_owned();
        assert!(yir_lower_llvm::emit_module(&module).is_err());
    }
    let seeds =
        yir_core::loop_carry_contract::parse_scoped_i64_carries(&compiled.yir.nodes[index].op.args)
            .unwrap()
            .unwrap()
            .seeds
            .iter()
            .map(|seed| seed.to_string())
            .collect::<Vec<_>>();
    for seed in seeds {
        for instruction in ["const_i32", "const_bool"] {
            let mut module = compiled.yir.clone();
            let producer = module
                .nodes
                .iter_mut()
                .find(|node| node.name == seed)
                .unwrap();
            producer.op.instruction = instruction.to_owned();
            producer.op.args[0] = if instruction == "const_bool" {
                "true"
            } else {
                "1"
            }
            .to_owned();
            assert!(yir_lower_llvm::emit_module(&module).is_err());
            assert!(yir_runtime_host::execute_module_source_with_registry(
                &nuisc::render::render_yir(&module),
                &yir_verify::default_registry()
            )
            .is_err());
        }
    }
}

#[test]
fn twelve_scalar_slots_and_generated_names_have_no_fixed_precombination() {
    let declarations = (0..12)
        .map(|index| format!("let state{index}: i64 = {index};"))
        .collect::<String>();
    let updates = (0..12)
        .map(|index| format!("let state{index}: i64 = state{index} + 1;"))
        .collect::<String>();
    let sum = (0..12)
        .map(|index| format!("state{index}"))
        .collect::<Vec<_>>()
        .join(" + ");
    let source = SOURCE
        .replace(
            "let index: i64 = 0;",
            &format!("{declarations} let __nuis_loop_state_0: i64 = 7; let index: i64 = 0;"),
        )
        .replace(
            "let index: i64 = index + 1;",
            &format!("{updates} let index: i64 = index + 1;"),
        )
        .replace(
            "load_at(buffer, 0) + load_at(buffer, 7) + index",
            &format!("{sum} + __nuis_loop_state_0"),
        )
        .replace(
            "struct Counter",
            "struct __nuis_scalar_carries_0 { value: i64 } struct Counter",
        );
    check_execution(&source, 169);
}
