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
        r#"
        if index % 2 == 0 {
            store_at(buffer, index, seed);
            let total: i64 = total + index;
            let checksum: i64 = checksum + total;
            let index: i64 = index + 1;
            continue;
        }
        store_at(buffer, index, value);
        let total: i64 = total + load_at(buffer, index);
        let checksum: i64 = checksum + total;
        "#,
        "(total + checksum + load_at(buffer, 0) + load_at(buffer, 7) + index) % 200",
    )
}

#[test]
fn guarded_continue_preserves_prefix_state_and_steps_once() {
    let no_carry = source(
        "",
        "if index % 2 == 0 { store_at(buffer, index, seed); let index: i64 = index + 1; continue; } store_at(buffer, index, value);",
        "load_at(buffer, 0) + load_at(buffer, 7) + index",
    );
    check_execution(&no_carry, 37);
    check_execution(&no_carry.replace("index % 2 == 0", "seed > 0"), 16);
    let source = carried();
    check_execution(&source, 160);
    check_execution(&source.replace("index % 2 == 0", "seed > 0"), 139);
    check_execution(&source.replace("index % 2 == 0", "seed < 0"), 160);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        11,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        83,
    );
}

#[test]
fn continue_unit_steps_do_not_wrap_at_integer_boundaries() {
    let source = source(
        "let total: i64 = 0;",
        "if seed > 0 { store_at(buffer, 0, seed); let total: i64 = total + 1; let index: i64 = index + 1; continue; } store_at(buffer, 0, 99);",
        "total + load_at(buffer, 0) + (9223372036854775807 - index)",
    )
    .replace("seed + index * 3", "seed")
    .replace("let index: i64 = 0;", "let index: i64 = 9223372036854775805;")
    .replace("index < 8", "index < 9223372036854775807");
    check_execution(&source, 6);
    let descending = source
        .replace("9223372036854775805", "(0 - 9223372036854775807 + 1)")
        .replace(
            "index < 9223372036854775807",
            "index > (0 - 9223372036854775807 - 1)",
        )
        .replace("index + 1", "index - 1")
        .replace(
            "9223372036854775807 - index",
            "index - (0 - 9223372036854775807 - 1)",
        );
    check_execution(&descending, 6);
}

#[test]
fn nested_and_sequential_continue_guards_snapshot_each_predicate_once() {
    let source = source(
        "let total: i64 = 1;",
        r#"
        if index < 4 {
            if index % 2 == 0 {
                let total: i64 = total + 1;
                let index: i64 = index + 1;
                continue;
            } else { let total: i64 = total + 2; }
            let total: i64 = total + 3;
        } else {
            if seed < 0 { let index: i64 = index + 1; continue; }
        }
        if index % 3 == 0 {
            store_at(buffer, index, 9);
            let total: i64 = total + 5;
            let index: i64 = index + 1;
            continue;
        }
        store_at(buffer, index, seed);
        let total: i64 = total + 10;
        "#,
        "total + load_at(buffer, 7) + index",
    );
    check_execution(&source, 75);
    check_execution(
        &source.replace("return fill(4);", "return fill(0 - 1);"),
        36,
    );
    let snapshot = self::source(
        "let total: i64 = 1;",
        "store_at(buffer, index, 1); if load_at(buffer, index) == 1 { store_at(buffer, index, 0); let total: i64 = total + 1; let index: i64 = index + 1; continue; } else { let total: i64 = total + 100; } store_at(buffer, index, 99);",
        "total + load_at(buffer, 7) + index",
    );
    check_execution(&snapshot, 17);
}

#[test]
fn continue_skips_suffix_reads_math_calls_and_nested_loop_bounds() {
    for (suffix, diagnostic) in [
        ("store_at(buffer, 8, seed);", "index"),
        ("let total: i64 = 1 / (seed - seed); store_at(buffer, index, total);", "zero"),
        ("let column: i64 = 0; while column < 1 / (seed - seed) { store_at(buffer, 8, seed); let column: i64 = column + 1; }", "zero"),
        ("let total: i64 = identity(load_at(buffer, 8)); store_at(buffer, index, total);", "index"),
    ] {
        let source = with_helpers(
            &source(
                "let total: i64 = 1;",
                &format!("if seed > 0 {{ let total: i64 = total + 1; let index: i64 = index + 1; continue; }} {suffix}"),
                "total + index",
            ),
            "fn identity(value: i64) -> i64 { return value; }",
        );
        check_execution(&source, 17);
        assert_traps(&source.replace("return fill(4);", "return fill(0);"), diagnostic);
    }
    let prefix = source(
        "let total: i64 = 1;",
        "store_at(buffer, 8, seed); if seed > 0 { let index: i64 = index + 1; continue; } let total: i64 = 1 / (seed - seed);",
        "total",
    );
    assert_traps(&prefix, "index");
}

#[test]
fn nested_continue_targets_only_its_own_loop_and_resets_each_iteration() {
    let source = source(
        "let total: i64 = 1; let checksum: i64 = 2;",
        r#"
        if index == 1 {
            let total: i64 = total + 7;
            let index: i64 = index + 1;
            continue;
        }
        let column: i64 = 0;
        while column < 2 {
            if column == 0 {
                let total: i64 = total + index;
                let column: i64 = column + 1;
                continue;
            }
            let position: i64 = index * 2 + column;
            store_at(buffer, position, seed + position);
            let checksum: i64 = checksum + load_at(buffer, position);
            let column: i64 = column + 1;
        }
        let total: i64 = total + column;
        "#,
        "(total + checksum + load_at(buffer, 7) + index) % 200",
    )
    .replace("index < 8", "index < 4");
    check_execution(&source, 61);
}

#[test]
fn continue_rejects_missing_or_mismatched_steps_and_unsupported_exits() {
    for bad in [
        "continue;",
        "let index: i64 = index + 2; continue;",
        "let index: i64 = index - 1; continue;",
        "let total: i64 = total + 1; continue;",
        "let index: bool = true; continue;",
        "let index: i64 = index + 1; store_at(buffer, index, seed); continue;",
        "let index: i64 = index + 1; continue; store_at(buffer, index, seed);",
        "let index: i64 = index + 1; break;",
        "free(buffer); let index: i64 = index + 1; continue;",
    ] {
        let source = source(
            "let total: i64 = 1;",
            &format!("if seed > 0 {{ {bad} }} store_at(buffer, index, total);"),
            "total",
        );
        assert!(nuisc::pipeline::compile_source(&source).is_err(), "{bad}");
    }
    let wrong_scope = source(
        "",
        "let column: i64 = 0; while column < 2 { if seed > 0 { let index: i64 = index + 1; continue; } store_at(buffer, column, seed); let column: i64 = column + 1; }",
        "index",
    );
    assert!(nuisc::pipeline::compile_source(&wrong_scope).is_err());
}

#[test]
fn continue_helpers_grow_linearly_and_keep_private_names_and_real_scalar_calls() {
    let controls = 32;
    let body = (0..controls).map(|index| format!(
        "if index == {index} {{ let total: i64 = bump(total); let checksum: i64 = checksum + total; let index: i64 = index + 1; continue; }}"
    )).collect::<String>() + "store_at(buffer, index, seed);";
    let source = with_helpers(
        &source(
            "let total: i64 = 1; let checksum: i64 = 2; let __nuis_buffer_continue_0: i64 = 7;",
            &body,
            "total + checksum + __nuis_buffer_continue_0 + index",
        ),
        "fn bump(value: i64) -> i64 { if value > 0 { return value + 1; } return 1; } fn __nuis_buffer_branch_0() -> i64 { return 99; }",
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.functions.len() < controls * 5 + 20);
    assert!(compiled.yir.nodes.iter().any(|node| {
        node.op.instruction == "call_i64" && node.op.args.first().is_some_and(|name| name == "bump")
    }));
    check_execution(&source, 70);
    assert_eq!(
        nuisc::render::render_yir(&compiled.yir),
        nuisc::render::render_yir(&nuisc::pipeline::compile_source(&source).unwrap().yir)
    );
}

#[test]
fn continue_effect_order_survives_yir_declaration_reordering() {
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
    assert_eq!(trace.values[&result.node], yir_core::Value::Int(160));
    assert_eq!(native_run(&source, &compiled).status.code(), Some(160));
}

#[test]
fn continue_shares_session_fuel_without_leaking_private_or_failed_state() {
    let project = Project::new(&carried());
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
    session.event(vec![yir_core::Value::Int(0)]).unwrap();
    let yir_core::Value::Struct(state) = session.state() else {
        panic!("state");
    };
    assert_eq!(state.fields.len(), 1);
    assert_eq!(state.fields[0].1, yir_core::Value::Int(160));
    let last = session.state().clone();
    let error = session
        .event_budgeted(vec![yir_core::Value::Int(0)], 80)
        .unwrap_err();
    assert!(error.contains("step budget exhausted"), "{error}");
    assert_eq!(session.state(), &last);
    assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
    session.close(vec![]).unwrap();
    assert_eq!(session.state(), &last);
    assert!(session.completion_status().is_err());
}
