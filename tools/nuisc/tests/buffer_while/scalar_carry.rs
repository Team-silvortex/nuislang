use super::*;
use scalar_helpers::{assert_traps, with_helpers};

fn source() -> String {
    with_helpers(&SOURCE
        .replace("let index: i64 = 0;", "let total: i64 = 1; let index: i64 = 0;")
        .replace("let index: i64 = index + 1;",
            "let total: i64 = accumulate(total, load_at(buffer, index), index); let index: i64 = index + 1;")
        .replace("let result: i64 =", "let result: i64 = total +"),
        r#"
        fn accumulate(total: i64, value: i64, index: i64) -> i64 {
            if index < 0 { return 1 / (index - index); }
            if index == 0 { return total + value; }
            let delta: i64 = value;
            if delta < 0 { return total - delta; }
            return total + delta;
        }
        "#)
}

#[test]
fn scalar_carry_buffer_loop_matches_native_zero_one_and_descending_trips() {
    let source = source();
    check_execution(&source, 154);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        9,
    );
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 7;"),
        59,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        138,
    );
    // A replacing update still preserves the incoming state if no iteration runs.
    let replaced = source.replace(
        "accumulate(total, load_at(buffer, index), index)",
        "load_at(buffer, index)",
    );
    check_execution(&replaced, 62);
    check_execution(
        &replaced.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        9,
    );
}

#[test]
fn scalar_carry_feeds_writes_and_order_is_independent_of_yir_declarations() {
    let source = source()
        .replace("seed + index * 3", "total + index")
        .replace(
            "let result: i64 = total + load_at(buffer, 0) + load_at(buffer, 7) + index;",
            "let result: i64 = total % 100 + load_at(buffer, 0) + index;",
        );
    // total evolves as 2 * total + index: 1 -> 2 -> 5 -> ... -> 503.
    check_execution(&source, 12);
    let mut compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled
        .yir
        .nodes
        .iter()
        .any(|node| node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carry")));
    for field in ["current", "carry0"] {
        assert!(compiled
            .yir
            .nodes
            .iter()
            .any(|node| node.op.instruction == "field"
                && node.op.args.get(1).map(String::as_str) == Some(field)));
    }
    compiled.yir.nodes.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let reordered = nuisc::render::render_yir(&compiled.yir);
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &reordered,
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
    assert_eq!(trace.values[&result.node], yir_core::Value::Int(12));
    assert_eq!(native_run(&source, &compiled).status.code(), Some(12));
}

#[test]
fn scalar_carry_projects_into_subsequent_loops_and_helper_calls() {
    let source = source().replace("let result: i64 = total +", concat!(
        "let index: i64 = 0; while index < 8 { ",
        "store_at(buffer, index, total); let total: i64 = total + 1; let index: i64 = index + 1; } ",
        "let result: i64 = total % 100 +"));
    // First loop leaves total=117. Second writes 117..124 and leaves total=125.
    let source = source.replace(
        "total % 100 + load_at(buffer, 0) + load_at(buffer, 7) + index",
        "accumulate(total, load_at(buffer, 0), index) % 200 + index",
    );
    check_execution(&source, 50);
}

#[test]
fn scalar_carry_update_traps_after_prior_buffer_effects_in_source_order() {
    let source = source();
    for (update, diagnostic) in [
        (
            "accumulate(total, load_at(buffer, 8), 1 / (index - index))",
            "index",
        ),
        (
            "accumulate(total, 1 / (index - index), load_at(buffer, 8))",
            "zero",
        ),
        ("accumulate(total, value, 0 - 1)", "zero"),
        ("(0 - 9223372036854775807 - 1) / (0 - 1)", "overflow"),
    ] {
        assert_traps(
            &source.replace("accumulate(total, load_at(buffer, index), index)", update),
            diagnostic,
        );
    }
    assert_traps(
        &source
            .replace(
                "store_at(buffer, index, value)",
                "store_at(buffer, 8, value)",
            )
            .replace(
                "accumulate(total, load_at(buffer, index), index)",
                "1 / (index - index)",
            ),
        "index",
    );
}

#[test]
fn scalar_carry_callbacks_share_fuel_and_never_commit_failed_state() {
    let source = source().replace("if index < 0", "if index < 0 || value < 0");
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
        for (input, expected) in [(0, 154), (1, 1664)] {
            session.event(vec![yir_core::Value::Int(input)]).unwrap();
            let yir_core::Value::Struct(state) = session.state() else {
                panic!("missing state");
            };
            assert_eq!(state.fields[0].1, yir_core::Value::Int(expected));
        }
        let last = session.state().clone();
        let error = if fuel_failure {
            session
                .event_budgeted(vec![yir_core::Value::Int(0)], 80)
                .unwrap_err()
        } else {
            session
                .event(vec![yir_core::Value::Int(-2000)])
                .unwrap_err()
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
fn scalar_carry_keeps_unsupported_mutations_and_mutable_headers_fail_closed() {
    let source = source();
    for invalid in [
        source.replace("index < 8", "index < total"),
        source.replace(
            "let total: i64 = accumulate",
            "let total: bool = accumulate",
        ),
        source.replace(
            "let total: i64 = accumulate(total, load_at(buffer, index), index);",
            "if index > 0 { let total: i64 = total + value; }",
        ),
        source.replace(
            "let total: i64 = accumulate",
            "let total: i64 = total + 1; let total: i64 = accumulate",
        ),
        source.replace(
            "store_at(buffer, index, value);",
            "let total: i64 = total + value; store_at(buffer, index, value);",
        ),
    ] {
        assert!(
            nuisc::pipeline::compile_source(&invalid).is_err(),
            "must reject\n{invalid}"
        );
    }
}

#[test]
fn scalar_carry_malformed_yir_is_rejected_by_registry_and_llvm() {
    let compiled = nuisc::pipeline::compile_source(&source()).unwrap();
    let index = compiled
        .yir
        .nodes
        .iter()
        .position(|node| node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carry"))
        .unwrap();
    let carry_index = compiled.yir.nodes[index]
        .op
        .args
        .iter()
        .position(|arg| arg == "$carry")
        .unwrap();
    for (operand, value) in [(7, "0"), (9, "$carry"), (carry_index, "$unknown")] {
        let mut module = compiled.yir.clone();
        module.nodes[index].op.args[operand] = value.to_owned();
        assert!(yir_verify::verify_module(&module).is_err());
        assert!(yir_lower_llvm::emit_module(&module).is_err());
    }
    let mut module = compiled.yir.clone();
    module.nodes[index].op.args[9] = "missing_seed".to_owned();
    assert!(yir_verify::verify_module(&module).is_err());
    assert!(yir_lower_llvm::emit_module(&module).is_err());
}

#[test]
fn scalar_carry_seed_and_glm_dependencies_remain_explicit() {
    let compiled = nuisc::pipeline::compile_source(&source()).unwrap();
    let node = compiled
        .yir
        .nodes
        .iter()
        .find(|node| node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carry"))
        .unwrap();
    let seed = &node.op.args[9];
    let profile = yir_core::glm_profile_for_operation(&node.op);
    assert!(profile.accesses.iter().any(|access| &access.input == seed));
    assert!(profile
        .accesses
        .iter()
        .all(|access| !access.input.starts_with('$')));
    assert!(compiled
        .yir
        .edges
        .iter()
        .any(|edge| &edge.from == seed && edge.to == node.name));
    for (instruction, value) in [("const_i32", "1"), ("const_bool", "true")] {
        let mut module = compiled.yir.clone();
        let seed = module
            .nodes
            .iter_mut()
            .find(|node| &node.name == seed)
            .unwrap();
        seed.op.instruction = instruction.to_owned();
        seed.op.args = vec![value.to_owned()];
        assert!(yir_lower_llvm::emit_module(&module).is_err());
        assert!(yir_runtime_host::execute_module_source_with_registry(
            &nuisc::render::render_yir(&module),
            &yir_verify::default_registry()
        )
        .is_err());
    }
}
