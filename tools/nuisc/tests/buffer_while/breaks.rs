use super::*;
use scalar_helpers::{assert_traps, with_helpers};

fn source(states: &str, body: &str, result: &str) -> String {
    SOURCE
        .replace(
            "let index: i64 = 0;",
            &format!("{states} let index: i64 = 0;"),
        )
        .replace("store_at(buffer, index, value);", body)
        .replace("load_at(buffer, 0) + load_at(buffer, 7) + index", result)
}

fn carried() -> String {
    source(
        "let total: i64 = 1; let checksum: i64 = 2;",
        "store_at(buffer, index, value); let total: i64 = total + value; let checksum: i64 = checksum + total; if index == 2 { break; } let total: i64 = total + 1;",
        "total + checksum + index + load_at(buffer, 2)",
    )
}

#[test]
fn break_preserves_current_and_prefix_effects_without_user_scalar_carries() {
    let source = source(
        "",
        "store_at(buffer, index, value); if index == 2 { break; }",
        "load_at(buffer, 0) + load_at(buffer, 2) + index",
    );
    check_execution(&source, 16);
    check_execution(&source.replace("index == 2", "index == 0"), 4);
    check_execution(&source.replace("index == 2", "index == 7"), 21);
    check_execution(&source.replace("index == 2", "seed < 0"), 22);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        8,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        12,
    );
}

#[test]
fn break_commits_all_scalar_carries_and_does_not_run_the_remaining_iterations() {
    check_execution(&carried(), 80);
    check_execution(&carried().replace("index < 8", "index < 1000000"), 80);
    let compiled = nuisc::pipeline::compile_source(&carried()).unwrap();
    assert!(compiled.yir.nodes.iter().any(|node| {
        node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break")
    }));
    assert!(compiled.llvm_ir.contains("loop_while_i64_advance"));
    assert!(compiled.llvm_ir.contains("loop_break_control_invalid"));
}

#[test]
fn break_skips_suffix_traps_calls_and_child_bounds_but_not_prefix_effects() {
    for (suffix, diagnostic) in [
        ("store_at(buffer, 8, seed);", "index"),
        ("let total: i64 = 1 / (seed - seed); store_at(buffer, index, total);", "zero"),
        ("let column: i64 = 0; while column < 1 / (seed - seed) { store_at(buffer, 8, seed); let column: i64 = column + 1; }", "zero"),
        ("let total: i64 = identity(load_at(buffer, 8)); store_at(buffer, index, total);", "index"),
    ] {
        let source = with_helpers(&source(
            "let total: i64 = 1;",
            &format!("if seed > 0 {{ let total: i64 = total + 2; break; }} {suffix}"),
            "total + index",
        ), "fn identity(value: i64) -> i64 { return value; }");
        check_execution(&source, 3);
        assert_traps(&source.replace("return fill(4);", "return fill(0);"), diagnostic);
    }
    assert_traps(
        &source(
            "let total: i64 = 1;",
            "store_at(buffer, 8, seed); if seed > 0 { break; } let total: i64 = 1 / (seed - seed);",
            "total",
        ),
        "index",
    );
}

#[test]
fn nested_break_exits_only_its_loop_and_keeps_the_child_exit_counter() {
    let source = source(
        "let total: i64 = 1; let checksum: i64 = 2;",
        r#"
        let column: i64 = 0;
        while column < 3 {
            let position: i64 = index * 2 + column;
            store_at(buffer, position, seed + position);
            let total: i64 = total + 1;
            if column == 1 { break; }
            let column: i64 = column + 1;
        }
        let checksum: i64 = checksum + column;
        "#,
        "total + checksum + load_at(buffer, 7) + index",
    )
    .replace("index < 8", "index < 4");
    check_execution(&source, 30);
    check_execution(
        &source.replace(
            "let column: i64 = 0;",
            "if index == 2 { break; } let column: i64 = 0;",
        ),
        11,
    );
}

#[test]
fn break_and_explicit_step_continue_have_distinct_nested_guard_semantics() {
    let source = with_helpers(
        &source(
            "let total: i64 = 1;",
            r#"
        if index < 4 {
            if index % 2 == 0 {
                let total: i64 = bump(total);
                let index: i64 = index + 1;
                continue;
            } else { let total: i64 = total + 2; }
        } else {
            let total: i64 = bump(total);
            break;
        }
        store_at(buffer, index, total);
        let total: i64 = total + 3;
        "#,
            "total + load_at(buffer, 1) + load_at(buffer, 3) + index",
        ),
        "fn bump(value: i64) -> i64 { if value > 0 { return value + 1; } return 1; }",
    );
    check_execution(&source, 32);
}

#[test]
fn break_rejects_unmodeled_induction_mutations_and_nonterminal_exits() {
    for bad in [
        "let index: i64 = index + 1; break;",
        "break; store_at(buffer, index, seed);",
        "free(buffer); break;",
        "return seed;",
    ] {
        let source = source(
            "",
            &format!("if seed > 0 {{ {bad} }} store_at(buffer, index, value);"),
            "index",
        );
        assert!(nuisc::pipeline::compile_source(&source).is_err(), "{bad}");
    }
    for name in ["user_iteration", "__nuis_buffer_iteration_0"] {
        let source = format!(
            r#"
        mod cpu Main {{
            struct Control {{ carry0: i64 }}
            fn {name}(index: i64, buffer: ref Buffer, signal: i64) -> Control {{
                store_at(buffer, index, signal);
                return Control {{ carry0: 2 }};
            }}
            fn main() -> i64 {{
                let buffer: ref Buffer = alloc_buffer(8, 0);
                let index: i64 = 0;
                while index < 8 {{
                    let returned: Control = {name}(index, buffer, 0);
                    let signal: i64 = returned.carry0;
                    if signal == 1 {{ break; }}
                    let index: i64 = index + 1;
                }}
                free(buffer);
                return index;
            }}
        }}
        "#
        );
        assert!(
            nuisc::pipeline::compile_source(&source).is_err(),
            "ordinary i64 data is not a canonical break signal, even with a generated-looking name"
        );
    }
}

#[test]
fn break_keeps_effect_order_after_yir_declaration_reordering() {
    let source = carried();
    let mut compiled = nuisc::pipeline::compile_source(&source).unwrap();
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
    assert_eq!(trace.values[&result.node], yir_core::Value::Int(80));
    assert_eq!(native_run(&source, &compiled).status.code(), Some(80));
}

#[test]
fn break_uses_shared_session_fuel_and_never_exports_its_private_control_slot() {
    let project = Project::new(&carried().replace("index < 8", "index < 1000000000000"));
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, _) = yir_runtime_host::ApplicationSession::open_registered(
        checkpoint.yir(),
        &registry,
        "counter",
        vec![yir_core::Value::Int(4)],
    )
    .unwrap();
    session
        .event_budgeted(vec![yir_core::Value::Int(0)], 1000)
        .unwrap();
    let yir_core::Value::Struct(state) = session.state() else {
        panic!("state");
    };
    assert_eq!(
        state.fields,
        [("count".to_owned(), yir_core::Value::Int(80))]
    );
    let previous = session.state().clone();
    let error = session
        .event_budgeted(vec![yir_core::Value::Int(0)], 20)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(session.state(), &previous);
    assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
    session.close(vec![]).unwrap();
    assert_eq!(session.state(), &previous);
    assert!(session.completion_status().is_err());
}

#[test]
fn break_contract_keeps_glm_inputs_and_rejects_payload_or_zero_trip_seed_drift() {
    let source = carried().replace("let index: i64 = 0;", "let index: i64 = 8;");
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    let index = compiled
        .yir
        .nodes
        .iter()
        .position(|node| {
            node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break")
        })
        .unwrap();
    let node = &compiled.yir.nodes[index];
    let carries = yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
        .unwrap()
        .unwrap();
    let profile = yir_core::glm_profile_for_operation(&node.op);
    for seed in &carries.seeds {
        assert!(profile.accesses.iter().any(|access| access.input == *seed));
        assert!(compiled
            .yir
            .edges
            .iter()
            .any(|edge| edge.from == *seed && edge.to == node.name));
    }
    assert!(profile.accesses.iter().any(|access| {
        !carries.seeds.contains(&access.input.as_str()) && carries.operands.contains(&access.input)
    }));
    for (argument, value) in [(7, "0"), (9, "Wrong{carry0:bool}"), (10, "$unknown")] {
        let mut module = compiled.yir.clone();
        module.nodes[index].op.args[argument] = value.to_owned();
        assert!(yir_verify::verify_module(&module).is_err());
        assert!(yir_lower_llvm::emit_module(&module).is_err());
    }
    let control_seed = carries.seeds.last().unwrap().to_string();
    for invalid in ["1", "-1"] {
        let mut changed = nuisc::pipeline::compile_source(&source).unwrap();
        let producer = changed
            .yir
            .nodes
            .iter_mut()
            .find(|node| node.name == control_seed)
            .unwrap();
        assert_eq!(producer.op.instruction, "const_i64");
        producer.op.args[0] = invalid.to_owned();
        changed.llvm_ir = yir_lower_llvm::emit_module(&changed.yir).unwrap();
        let error = yir_runtime_host::execute_module_source_with_registry(
            &nuisc::render::render_yir(&changed.yir),
            &yir_verify::default_registry(),
        )
        .unwrap_err();
        assert!(error.contains("i64 zero seed"), "{error}");
        let output = native_run(&source, &changed);
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(output.status.signal(), Some(4 | 5)),
                "{}",
                output.status
            );
        }
        assert!(!output.status.success());
    }
}

#[test]
fn break_helpers_have_linear_growth_private_names_and_one_time_predicates() {
    let controls = 32;
    let body = (0..controls)
        .map(|index| format!("if index == {index} {{ let total: i64 = bump(total); break; }}"))
        .collect::<String>()
        + "store_at(buffer, index, total);";
    let source = with_helpers(
        &source(
            "let total: i64 = 1; let __nuis_buffer_break_0: i64 = 7;",
            &body,
            "total + __nuis_buffer_break_0 + index",
        ),
        "fn bump(value: i64) -> i64 { if value > 0 { return value + 1; } return 1; }",
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.functions.len() < controls * 5 + 20);
    assert!(compiled
        .yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "call_i64"
            && node.op.args.first().map(String::as_str) == Some("bump")));
    assert_eq!(
        nuisc::render::render_yir(&compiled.yir),
        nuisc::render::render_yir(&nuisc::pipeline::compile_source(&source).unwrap().yir)
    );
    check_execution(&source, 9);
    check_execution(&self::source(
        "let total: i64 = 1;",
        "store_at(buffer, index, 1); if load_at(buffer, index) == 1 { store_at(buffer, index, 0); let total: i64 = total + 1; break; } else { let total: i64 = total + 100; } store_at(buffer, index, 99);",
        "total + load_at(buffer, 0) + index",
    ), 2);
}

#[test]
fn malformed_return_control_traps_in_native_and_reference_execution() {
    let source = carried();
    for (invalid, bind_lane) in [("-1", true), ("2", true), ("2", false)] {
        let mut compiled = nuisc::pipeline::compile_source(&source).unwrap();
        let callee = compiled
            .yir
            .nodes
            .iter()
            .find(|node| {
                node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break")
            })
            .unwrap()
            .op
            .args[8]
            .clone();
        let function = compiled
            .yir
            .functions
            .iter_mut()
            .find(|function| function.name == callee)
            .unwrap();
        let returned = compiled
            .yir
            .nodes
            .iter()
            .find(|node| node.name == function.result.as_ref().unwrap().node)
            .unwrap();
        assert_eq!(returned.op.instruction, "return_owned_struct");
        let packed = returned.op.args[0].clone();
        let packed_node = compiled
            .yir
            .nodes
            .iter_mut()
            .find(|node| node.name == packed)
            .unwrap();
        assert_eq!(packed_node.op.instruction, "struct");
        let (field, _) = packed_node.op.args.last().unwrap().split_once('=').unwrap();
        let name = "__test_invalid_break_control".to_owned();
        *packed_node.op.args.last_mut().unwrap() = format!("{field}={name}");
        let mut literal = packed_node.clone();
        literal.name = name.clone();
        literal.op.instruction = "const_i64".to_owned();
        literal.op.args = vec![invalid.to_owned()];
        function.body_nodes.push(name.clone());
        if bind_lane {
            compiled
                .yir
                .node_lanes
                .insert(name.clone(), format!("fn:{callee}"));
        }
        compiled.yir.nodes.push(literal);
        compiled.yir.edges.push(yir_core::Edge {
            kind: yir_core::EdgeKind::Dep,
            from: name,
            to: packed,
        });
        yir_verify::verify_module(&compiled.yir).unwrap();
        compiled.llvm_ir = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        if !bind_lane {
            assert!(compiled
                .llvm_ir
                .contains("deferred lowering for declared CPU function result"));
        }
        let error = yir_runtime_host::execute_module_source_with_registry(
            &nuisc::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        )
        .unwrap_err();
        assert!(error.contains("break flag must be i64 0 or 1"), "{error}");
        let output = native_run(&source, &compiled);
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(output.status.signal(), Some(4 | 5)),
                "{}",
                output.status
            );
        }
        assert!(!output.status.success());
    }
}
