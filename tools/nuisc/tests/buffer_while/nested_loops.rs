use super::*;
use scalar_helpers::assert_traps;

fn source(states: &str, body: &str, result: &str) -> String {
    SOURCE
        .replace(
            "let index: i64 = 0;",
            &format!("{states} let index: i64 = 0;"),
        )
        .replace("store_at(buffer, index, value);", body)
        .replace("load_at(buffer, 0) + load_at(buffer, 7) + index", result)
}

fn grid() -> String {
    source(
        "let total: i64 = 1; let checksum: i64 = 2;",
        r#"
        let column: i64 = 0;
        while column < 2 {
            let position: i64 = index * 2 + column;
            store_at(buffer, position, seed + position);
            let total: i64 = total + load_at(buffer, position);
            if column == 0 { let checksum: i64 = checksum + total; }
            else { let checksum: i64 = checksum + 1; }
            let column: i64 = column + 1;
        }
        let checksum: i64 = checksum + column;
        "#,
        "(total + checksum + load_at(buffer, 7)) % 200",
    )
    .replace("while index < 8", "while index < 4")
}

#[test]
fn nested_buffer_loops_execute_real_helpers_and_reset_iteration_locals() {
    let source = grid();
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.functions.iter().any(|function| {
        function.name.contains("__nuis_buffer_iteration_")
            && compiled.yir.nodes.iter().any(|node| {
                function.body_nodes.contains(&node.name)
                    && node.op.instruction == "loop_while_i64_effect"
                    && node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries")
            })
    }));
    // Eight distinct writes, with a fresh column counter on every row.
    check_execution(&source, 188);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 4;"),
        3,
    );
    check_execution(&source.replace("column < 2", "column < 0"), 3);
    check_execution(&source.replace("column < 2", "column < 1"), 99);
}

#[test]
fn nested_loops_preserve_persistent_counters_and_support_scalar_only_children() {
    let source = source(
        "let column: i64 = 0; let total: i64 = 1;",
        r#"
        while column < 2 {
            let total: i64 = total + seed;
            let column: i64 = column + 1;
        }
        store_at(buffer, index, total + column);
        "#,
        "total + column + load_at(buffer, 7)",
    );
    check_execution(&source, 22);
    check_execution(&source.replace("column < 2", "column < 0"), 2);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        1,
    );
    let no_carry = source.replace(
        "let total: i64 = total + seed;",
        "store_at(buffer, column, seed);",
    );
    check_execution(&no_carry, 6);
}

#[test]
fn nested_descending_loops_and_outer_dependent_bounds_keep_exact_state() {
    let source = source(
        "let total: i64 = 1;",
        r#"
        let column: i64 = index;
        while column > 0 {
            store_at(buffer, column, seed);
            let total: i64 = total + column;
            let column: i64 = column - 1;
        }
        let total: i64 = total + column;
        "#,
        "total + index",
    );
    check_execution(&source, 93);
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        85,
    );
    let ascending = source
        .replace("let column: i64 = index;", "let column: i64 = 0;")
        .replace("column > 0", "column < index")
        .replace("column - 1", "column + 1");
    check_execution(&ascending, 93);
}

#[test]
fn branch_nested_loops_return_visible_locals_without_speculating_bounds() {
    let source = source(
        "let total: i64 = 1;",
        r#"
        let column: i64 = 0;
        let local: i64 = 3;
        if index % 2 == 0 {
            while column < 2 {
                store_at(buffer, index, local);
                let local: i64 = local + seed;
                let column: i64 = column + 1;
            }
        } else {
            if seed < 0 {
                while column < 1 / (index - index) {
                    store_at(buffer, 8, seed);
                    let column: i64 = column + 1;
                }
            }
        }
        let total: i64 = total + local + column;
        "#,
        "total",
    );
    check_execution(&source, 65);
    assert_traps(
        &source.replace("return fill(4);", "return fill(0 - 1);"),
        "zero",
    );
}

#[test]
fn nested_loop_traps_keep_source_order_across_loop_boundaries() {
    for (body, diagnostic) in [
        ("store_at(buffer, 8, seed); let column: i64 = 0; while column < 1 / (index - index) { store_at(buffer, index, seed); let column: i64 = column + 1; }", "index"),
        ("let column: i64 = 0; while column < 1 / (index - index) { store_at(buffer, 8, seed); let column: i64 = column + 1; }", "zero"),
        ("let column: i64 = 0; while column < 2 { store_at(buffer, 8, seed); let column: i64 = column + 1; } let total: i64 = 1 / (index - index);", "index"),
        ("let column: i64 = 0; while column < 2 { let total: i64 = 1 / (index - index); store_at(buffer, 8, seed); let column: i64 = column + 1; }", "zero"),
    ] {
        assert_traps(&source("let total: i64 = 1;", body, "total"), diagnostic);
    }
    let source = grid();
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
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap()
        .result
        .as_ref()
        .unwrap();
    assert_eq!(trace.values[&result.node], yir_core::Value::Int(188));
    assert_eq!(native_run(&source, &compiled).status.code(), Some(188));
}

#[test]
fn nested_loops_reject_ancestor_header_mutation_and_unsupported_control() {
    for bad in [
        "let index: i64 = index + 1;",
        "let limit: i64 = limit + 1;",
        "let total: bool = true;",
        "free(buffer);",
        "break;",
        "continue;",
        "return 0;",
        "while index < 8 { let index: i64 = index + 1; }",
        "while limit < 10 { let limit: i64 = limit + 1; }",
    ] {
        let source = source(
            "let total: i64 = 1; let limit: i64 = 8;",
            &format!("let column: i64 = 0; while column < 2 {{ {bad} store_at(buffer, index, seed); let column: i64 = column + 1; }}"),
            "total",
        ).replace("index < 8", "index < limit");
        assert!(
            nuisc::pipeline::compile_source(&source).is_err(),
            "{source}"
        );
    }
    for (from, to) in [
        ("column < 2", "column <= 2"),
        ("column + 1", "column + 2"),
        ("column < 2", "column < total"),
    ] {
        assert!(
            nuisc::pipeline::compile_source(&grid().replace(from, to)).is_err(),
            "{to}"
        );
    }
}

#[test]
fn nested_loops_share_session_fuel_and_never_commit_failed_callback_state() {
    let project = Project::new(&grid());
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
    assert_eq!(state.fields[0].1, yir_core::Value::Int(188));
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

#[test]
fn recursive_loops_retain_scalar_callees_and_linear_helper_growth() {
    let depth = 8;
    let mut body = "store_at(buffer, index, seed); let total: i64 = bump(total);".to_owned();
    for level in (0..depth).rev() {
        body = format!("let child{level}: i64 = 0; while child{level} < 1 {{ {body} let child{level}: i64 = child{level} + 1; }}");
    }
    let source = source("let total: i64 = 1; let __nuis_loop_state_0: i64 = 7;", &body, "total + __nuis_loop_state_0")
        .replace("fn main()", "fn bump(value: i64) -> i64 { if value > 0 { return value + 1; } return 1; } fn __nuis_buffer_iteration_0() -> i64 { return 99; } fn main()");
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.functions.len() < depth * 3 + 10);
    assert!(compiled
        .yir
        .functions
        .iter()
        .any(|f| f.name.ends_with("bump")));
    check_execution(&source, 16);
    let siblings = source.replace(&body, &body.repeat(4));
    // Each sibling needs its own lexical counter names, not a rebind of an iteration local.
    let bodies = (0..4)
        .map(|sibling| {
            (0..depth).fold(body.clone(), |body, level| {
                body.replace(&format!("child{level}"), &format!("s{sibling}child{level}"))
            })
        })
        .collect::<String>();
    let siblings = siblings.replace(&body.repeat(4), &bodies);
    let compiled = nuisc::pipeline::compile_source(&siblings).unwrap();
    assert!(compiled.yir.functions.len() < depth * 4 * 3 + 10);
    check_execution(&siblings, 40);
}
