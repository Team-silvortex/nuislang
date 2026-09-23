use super::*;

#[test]
fn typed_literal_narrowing_emits_wrapped_i32_constants_without_cast_instructions() {
    use nuis_semantics::model::{NirExpr, NirStmt};
    for value in [
        0,
        -1,
        i32::MAX as i64,
        i32::MAX as i64 + 1,
        i32::MIN as i64 - 1,
        i64::MIN,
        i64::MAX,
    ] {
        let mut module =
            parse_nuis_module("mod cpu Main { fn main() -> i64 { return 0; } }").unwrap();
        module.functions[0].body = vec![NirStmt::Return(Some(NirExpr::CastI32ToI64(Box::new(
            NirExpr::CastI64ToI32(Box::new(NirExpr::Int(value))),
        ))))];
        let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
        assert!(yir
            .nodes
            .iter()
            .any(|n| n.op.instruction == "const_i32" && n.op.args == [(value as i32).to_string()]));
        assert!(!yir
            .nodes
            .iter()
            .any(|n| n.op.instruction == "cast_i64_to_i32"));
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}

#[test]
fn mixed_nested_local_selection_keeps_snapshots_and_effectful_predicate_once() {
    let source = "mod cpu Main {
        struct Leaf { flag: bool, value: i64 }
        struct Packet { payload: Leaf }
        fn predicate(value: i64) -> bool { print(value); return value > 0; }
        fn relay(value: Packet) -> Packet { return value; }
        fn checked(value: Packet, divisor: i64) -> Packet {
            return Packet { payload: Leaf { flag: value.payload.flag, value: value.payload.value / divisor } };
        }
        @noinline fn event(a: i64, b: i64) -> i64 {
            let saved = Packet { payload: Leaf { flag: true, value: 12 } };
            let before = saved;
            let saved: Packet = if predicate(a) {
                if b == 0 { relay(saved) } else { checked(saved, b) }
            } else { checked(saved, b) };
            print(before.payload.value); print(saved.payload.value);
            return before.payload.value + saved.payload.value;
        }
        fn main() -> i64 { return event(1, 0) + event(1, 2); }
    }";
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let prints = trace
        .events
        .iter()
        .filter(|e| e.contains("cpu.print"))
        .collect::<Vec<_>>();
    assert_eq!(prints.len(), 6, "{prints:?}");
    for (line, value) in prints.iter().zip([1, 12, 12, 1, 12, 6]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    yir_lower_llvm::emit_module(&yir).unwrap();
}

#[test]
fn fallible_local_value_in_effectful_function_is_selected_before_evaluation() {
    let source = r#"mod cpu Main {
        struct State { value: i64, divisor: i64 }
        @noinline fn event(state: State, kind: i64) -> i64 {
            if kind < 0 { return state.value; }
            let selected: i64 = if kind == 0 { 7 } else { state.value % state.divisor };
            print(selected);
            return selected;
        }
        fn main() -> i64 {
            let ignored = event(State { value: 9, divisor: 0 }, -1);
            let skipped = event(State { value: 9, divisor: 0 }, 0);
            let reached = event(State { value: 9, divisor: 2 }, 1);
            return ignored + skipped + reached;
        }
    }"#;
    let module = parse_nuis_module(source).unwrap();
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    yir_verify::verify_module(&yir).unwrap();
    for reversed in [false, true] {
        let mut yir = yir.clone();
        if reversed {
            yir.nodes.reverse();
            for function in &mut yir.functions {
                function.body_nodes.reverse();
            }
        }
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .unwrap();
        let prints = trace
            .events
            .iter()
            .filter(|event| event.contains("cpu.print"))
            .collect::<Vec<_>>();
        assert_eq!(prints.len(), 2, "{prints:?}");
        assert!(prints[0].ends_with("7"), "{prints:?}");
        assert!(prints[1].ends_with("1"), "{prints:?}");
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}

#[test]
fn selected_local_values_keep_checked_failures_even_when_unused() {
    for op in ["/", "%"] {
        for operands in ["9, 0", "-9223372036854775807 - 1, -1"] {
            let source = format!(
                r#"mod cpu Main {{
                fn leaf(a: i64, b: i64) -> i64 {{ return a {op} b; }}
                fn ignore(value: i64) -> i64 {{ return 3; }}
                @noinline fn event(enabled: bool, a: i64, b: i64) -> i64 {{
                    let unused: i64 = if enabled {{ ignore(leaf(a, b)) }} else {{ 0 }};
                    print(99); return 0;
                }}
                fn main() -> i64 {{ return event(true, {operands}); }}
            }}"#
            );
            let yir = crate::pipeline::compile_source(&source).unwrap().yir;
            let error = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            )
            .unwrap_err();
            assert!(
                error.contains("zero") || error.contains("overflow"),
                "{error}"
            );
        }
    }
}

#[test]
fn nested_local_values_capture_rebindings_and_evaluate_effectful_predicate_once() {
    let source = r#"mod cpu Main {
        struct Pair { left: i64, right: i64 }
        fn predicate(value: i64) -> bool { print(value); return value > 0; }
        @noinline fn event(a: i64, b: i64, __nuis_value_condition_0: i64) -> i64 {
            let value: i64 = 12;
            let value: i64 = if predicate(a) {
                if b == 0 { value } else { value / b }
            } else { __nuis_value_condition_0 };
            let flag: bool = if a > 0 { value % 5 == 2 } else { true };
            let pair: Pair = if flag {
                Pair { right: value % 5, left: value }
            } else { Pair { left: 0, right: 0 } };
            print(pair.left); print(pair.right);
            return pair.left + pair.right;
        }
        fn main() -> i64 { return event(1, 0, 99); }
    }"#;
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let prints = trace
        .events
        .iter()
        .filter(|event| event.contains("cpu.print"))
        .collect::<Vec<_>>();
    assert_eq!(prints.len(), 3, "{prints:?}");
    for (line, value) in prints.iter().zip([1, 12, 2]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    yir_lower_llvm::emit_module(&yir).unwrap();
}

#[test]
fn fallible_local_selection_does_not_admit_effectful_arms() {
    let source = r#"mod cpu Main {
        fn effect(a: i64, b: i64) -> i64 { print(a); return a / b; }
        @noinline fn event(enabled: bool) -> i64 {
            let selected: i64 = if enabled { effect(1, 0) } else { 0 };
            print(selected); return selected;
        }
        fn main() -> i64 { return event(false); }
    }"#;
    let error = crate::pipeline::compile_source(source)
        .err()
        .expect("effectful arm must remain rejected");
    assert!(
        error.contains("conditional fallible return requires guarded helper lowering"),
        "{error}"
    );
}

#[test]
fn one_sided_checked_rebinding_keeps_the_existing_outer_value() {
    for body in [
        "if enabled { let saved: i64 = value / divisor; }",
        "if !enabled {} else { let saved: i64 = value / divisor; }",
    ] {
        let source = format!(
            r#"mod cpu Main {{
            @noinline fn event(enabled: bool, value: i64, divisor: i64) -> i64 {{
                let saved: i64 = value;
                {body}
                print(saved); return saved;
            }}
            fn main() -> i64 {{
                let first = event(false, 12, 0);
                let second = event(true, 12, 2);
                return first + second;
            }}
        }}"#
        );
        let yir = crate::pipeline::compile_source(&source).unwrap().yir;
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .unwrap();
        let prints = trace
            .events
            .iter()
            .filter(|event| event.contains("cpu.print"))
            .collect::<Vec<_>>();
        assert_eq!(prints.len(), 2, "{prints:?}");
        assert!(prints[0].ends_with(": 12"), "{prints:?}");
        assert!(prints[1].ends_with(": 6"), "{prints:?}");
    }
}
