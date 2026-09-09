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

fn conditional_count() -> String {
    source(
        "let total: i64 = 1;",
        r#"
        store_at(buffer, index, value);
        if index % 2 == 0 { let total: i64 = total + value; }
    "#,
        "total",
    )
}

#[test]
fn branch_carries_preserve_untaken_and_zero_trip_seeds() {
    let source = conditional_count();
    check_execution(&source, 53);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        1,
    );
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 7;"),
        1,
    );
    check_execution(
        &source
            .replace("let index: i64 = 0;", "let index: i64 = 7;")
            .replace("index < 8", "index > 0")
            .replace("index + 1", "index - 1"),
        49,
    );
    check_execution(
        &source.replace("if index % 2 == 0 {", "if index % 2 != 0 {} else {"),
        53,
    );
    check_execution(&source.replace("total + value", "value"), 22);
}

#[test]
fn nested_branch_carries_merge_all_slots_and_preserve_sequential_rebindings() {
    let source = source(
        "let total: i64 = 1; let checksum: i64 = 2;",
        r#"
        store_at(buffer, index, value);
        if index % 2 == 0 {
            let total: i64 = total + value;
            if total > 12 { let checksum: i64 = checksum + total; }
            else { let checksum: i64 = checksum + 3; }
            let total: i64 = total + 1;
        } else {
            let checksum: i64 = checksum + value;
            let total: i64 = total + checksum;
        }
        store_at(buffer, index, total + checksum);
        if total % 3 == 0 { let checksum: i64 = checksum + 1; }
        let checksum: i64 = checksum + total;
    "#,
        "(total + checksum + load_at(buffer, 7)) % 200",
    );
    let (mut total, mut checksum, mut pixel) = (1, 2, 0);
    for index in 0..8 {
        let value = 4 + index * 3;
        if index % 2 == 0 {
            total += value;
            checksum += if total > 12 { total } else { 3 };
            total += 1;
        } else {
            checksum += value;
            total += checksum;
        }
        pixel = total + checksum;
        if total % 3 == 0 {
            checksum += 1;
        }
        checksum += total;
    }
    check_execution(&source, (total + checksum + pixel) % 200);
    check_execution(
        &source.replace("let index: i64 = 0;", "let index: i64 = 8;"),
        3,
    );
}

#[test]
fn branch_carry_predicate_is_snapshotted_before_state_and_buffer_updates() {
    let source = source(
        "let total: i64 = 1; let checksum: i64 = 2;",
        r#"
        store_at(buffer, index, value);
        if total == 1 || load_at(buffer, index) == value {
            let total: i64 = total + 1;
            store_at(buffer, index, value + 1);
            let checksum: i64 = checksum + total;
        } else {
            let total: i64 = 1 / (index - index);
            let checksum: i64 = load_at(buffer, 8);
        }
    "#,
        "total + checksum",
    );
    check_execution(&source, 55);
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
    assert_eq!(trace.values[&result.node], yir_core::Value::Int(55));
    assert_eq!(native_run(&source, &compiled).status.code(), Some(55));
    assert_traps(
        &source.replace("total == 1 || load_at(buffer, index) == value", "seed < 0"),
        "zero",
    );
}

#[test]
fn branch_carry_traps_keep_source_order_before_and_after_writes() {
    for (body, diagnostic) in [
        ("store_at(buffer, 8, value); let total: i64 = 1 / (index - index);", "index"),
        ("let total: i64 = 1 / (index - index); store_at(buffer, 8, value);", "zero"),
        ("let total: i64 = load_at(buffer, 8); let checksum: i64 = 1 / (index - index); store_at(buffer, index, value);", "index"),
        ("let total: i64 = 1 / (index - index); let checksum: i64 = load_at(buffer, 8); store_at(buffer, index, value);", "zero"),
    ] {
        let source = source("let total: i64 = 1; let checksum: i64 = 2;",
            &format!("if seed > 0 {{ {body} }}"), "total + checksum");
        assert_traps(&source, diagnostic);
        check_execution(&source.replace("return fill(4);", "return fill(0);"), 3);
    }
}

#[test]
fn branch_carries_reject_mutable_headers_non_i64_state_and_ownership_control() {
    for bad in [
        "let index: i64 = index + 1;",
        "let limit: i64 = limit + 1;",
        "let total: bool = true;",
        "let flag: bool = false;",
        "let local: i64 = seed + index; if index < 2 { let local: i64 = local + 1; } store_at(buffer, index, local);",
        "free(buffer);",
        "let index: i64 = index + 1; break;",
        "continue;",
        "return 0;",
        "while index < 8 { let index: i64 = index + 1; }",
    ] {
        let source = source(
            "let total: i64 = 1; let limit: i64 = 8; let flag: bool = seed > 0;",
            &format!("if index < 4 {{ {bad} }} if flag {{ store_at(buffer, index, value); }}"),
            "total",
        )
        .replace("while index < 8", "while index < limit");
        assert!(
            nuisc::pipeline::compile_source(&source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn branch_carries_share_callback_fuel_without_committing_failed_state() {
    let source = conditional_count().replace(
        "store_at(buffer, index, value);",
        "if seed < 0 { let total: i64 = 1 / (index - index); } store_at(buffer, index, value);",
    );
    let project = Project::new(&source);
    let checkpoint = nuisc::pipeline::resolve_compile_input(&project.0)
        .unwrap()
        .compile_to_verified_yir(&Default::default())
        .unwrap();
    let registry = yir_verify::default_registry();
    for exhausted in [false, true] {
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
        assert_eq!(state.fields[0].1, yir_core::Value::Int(53));
        let last = session.state().clone();
        let error = if exhausted {
            session.event_budgeted(vec![yir_core::Value::Int(0)], 80)
        } else {
            session.event(vec![yir_core::Value::Int(-100)])
        }
        .unwrap_err();
        assert!(
            error.contains(if exhausted {
                "step budget exhausted"
            } else {
                "zero"
            }),
            "{error}"
        );
        assert_eq!(session.state(), &last);
        assert!(session.event(vec![yir_core::Value::Int(0)]).is_err());
        session.close(vec![]).unwrap();
        assert_eq!(session.state(), &last);
        assert!(session.completion_status().is_err());
    }
}

#[test]
fn branch_carries_keep_linear_helper_growth_and_generated_name_isolation() {
    let branches = 32;
    let body = format!(
        "store_at(buffer, index, value); {}",
        "if index < 4 { let total: i64 = total + 1; let checksum: i64 = checksum + total; }"
            .repeat(branches)
    );
    let source = source(
        "let total: i64 = 1; let checksum: i64 = 2; let __nuis_branch_state_0: i64 = 7;",
        &body,
        "(total + checksum + __nuis_branch_state_0) % 200",
    )
    .replace(
        "struct Counter",
        "struct __nuis_scalar_carries_0 { value: i64 } struct Counter",
    )
    .replace(
        "fn main()",
        "fn __nuis_buffer_branch_0() -> i64 { return 99; } fn main()",
    );
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(compiled.yir.functions.len() <= branches * 2 + 8);
    assert!(compiled
        .yir
        .functions
        .iter()
        .any(|function| function.name == "__nuis_buffer_branch_1"));
    let (mut total, mut checksum) = (1, 2);
    for _ in 0..4 * branches {
        total += 1;
        checksum += total;
    }
    check_execution(&source, ((total + checksum + 7) % 200) as i32);
}

#[test]
fn branch_carries_feed_subsequent_loops() {
    let source = conditional_count().replace(
        "let result: i64 = total;",
        concat!(
            "let index: i64 = 0; while index < 8 { ",
            "if index % 2 == 0 { let total: i64 = total + 1; } ",
            "store_at(buffer, index, total); let index: i64 = index + 1; } ",
            "let result: i64 = total + load_at(buffer, 7);"
        ),
    );
    check_execution(&source, 114);
}
