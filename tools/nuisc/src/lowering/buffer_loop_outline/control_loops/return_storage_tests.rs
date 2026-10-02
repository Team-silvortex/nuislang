use super::*;
use crate::frontend::parse_nuis_module;

const MIXED: &str = include_str!(
    "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
);

fn optimized(source: &str, function: &str) -> (bool, Option<Vec<NirStmt>>) {
    let module = parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let function = module
        .functions
        .iter()
        .find(|f| f.name == function)
        .unwrap();
    (
        catalog.contains_key(&function.name),
        returns::coalesce(function, &layouts, &catalog),
    )
}

#[test]
fn return_storage_reuses_exact_initialized_records_without_name_conventions() {
    for name in ["carry", "working_copy", "__nuis_return_value_0"] {
        let source = MIXED.replace("carry", name);
        let mut module = parse_nuis_module(&source).unwrap();
        for _ in 0..2 {
            let layouts = control_values::layouts(&module);
            let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
            let function = module.functions.iter().find(|f| f.name == "step").unwrap();
            let before = function.clone();
            let baseline = returns::normalize(function, &layouts).unwrap().unwrap();
            let body = returns::coalesce(function, &layouts, &catalog).unwrap();
            assert_eq!(body.len() + 1, baseline.len());
            assert_eq!(&body[1..4], &function.body[..3]);
            assert_eq!(function, &before);
            let NirStmt::If { then_body, .. } = &body[5] else {
                panic!("return propagation")
            };
            assert_eq!(
                then_body,
                &[NirStmt::Return(Some(NirExpr::Var(name.into())))]
            );
            module.functions.reverse();
        }
    }
}

#[test]
fn return_storage_revalidates_new_inner_writes_without_weakening_source_order() {
    // The original inner loop only reads carry. Reusing it for the payload
    // would create a new write set and an unavailable forward sibling read.
    let source = MIXED.replace(
        "let j = j + 1;",
        "let j = j + 1; let snapshot = carry.right;",
    );
    let (admitted, body) = optimized(&source, "step");
    assert!(admitted);
    assert!(body.is_none());
    let yir = crate::pipeline::compile_source(&source).unwrap().yir;
    let max = yir
        .nodes
        .iter()
        .filter_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args).unwrap()
        })
        .map(|call| call.seeds.len())
        .max();
    // The independent payload remains required, but the source carry's outer
    // invariant tag/count can leave the backedge without granting the invalid read.
    assert_eq!(max, Some(17));
}

#[test]
fn return_storage_never_grants_original_invalid_reads_or_return_types() {
    for source in [
        MIXED.replace("let carry = carry; let selected", "let selected"),
        MIXED.replace(
            "return State { left: selected, right: carry.right, count: limit\n                    };",
            "return selected;",
        ),
        MIXED.replace("let carry = state;", "const carry: State = state;"),
    ] {
        let (admitted, body) = optimized(&source, "step");
        assert!(!admitted, "{source}");
        assert!(body.is_none());
    }
}

fn wrapper(body: &str) -> String {
    format!(
        "mod cpu Main {{
            struct Pair {{ value: i64, index: i64 }}
            struct Other {{ value: i64, index: i64 }}
            fn work(seed: Pair, limit: i64) -> Pair {{ {body} }}
            fn main() -> i64 {{ return 0; }}
        }}"
    )
}

#[test]
fn return_storage_keeps_parameters_constants_late_and_scoped_bindings_independent() {
    let iteration = "let index = 0; while index < limit {
        let index = index + 1;
        if index == 2 { return Pair { value: index, index: index }; }
    }";
    for body in [
        format!("{iteration} return seed;"),
        format!("const state: Pair = seed; {iteration} return state;"),
        format!("let state = seed; {iteration} return state;"),
        format!("{iteration} let state = seed; return state;"),
        format!("if limit > 0 {{ let state = seed; {iteration} return state; }} return seed;"),
        format!("let other = Other {{ value: 0, index: 0 }}; {iteration} return seed;").replace(
            "let index = index + 1;",
            "let index = index + 1; let other = other;",
        ),
    ] {
        let (admitted, body) = optimized(&wrapper(&body), "work");
        assert!(admitted);
        assert!(body.is_none());
    }
}

#[test]
fn return_storage_preserves_leading_trailing_and_sibling_exit_payloads() {
    for leading in [true, false] {
        let body = "let state = seed; let index = 0;
            while index < limit {
                STEP
                let state = state;
                if index == 2 { return Pair { value: state.index, index: state.value }; }
                if index == 3 { return Pair { value: 1 / (index - 3), index: index }; }
                let state = Pair { value: state.value + 1, index: index };
                TAIL
            }
            return state;"
            .replace(
                "STEP",
                if leading {
                    "let index = index + 1;"
                } else {
                    ""
                },
            )
            .replace(
                "TAIL",
                if leading {
                    ""
                } else {
                    "let index = index + 1;"
                },
            );
        let (admitted, body) = optimized(&wrapper(&body), "work");
        assert!(admitted);
        assert!(body.is_some());
    }
}

#[test]
fn return_storage_matches_independent_payload_execution_and_selected_traps() {
    let manifest =
        crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
            .unwrap();
    for leading in [true, false] {
        for limit in 0..=4 {
            for stop in [1, 2, 4] {
                let body = format!(
                    "let state = seed; let index = 0;
                    while index < limit {{
                        STEP
                        let state = state;
                        if index == {stop} {{ return Pair {{ value: state.index, index: state.value }}; }}
                        if index == 3 {{ return Pair {{ value: 1 / (index - 3), index: index }}; }}
                        let state = Pair {{ value: state.value + 1, index: index }};
                        TAIL
                    }}
                    return state;"
                )
                .replace("STEP", if leading { "let index = index + 1;" } else { "" })
                .replace("TAIL", if leading { "" } else { "let index = index + 1;" });
                let source = wrapper(&body).replace(
                    "return 0;",
                    &format!("let result = work(Pair {{ value: 41, index: 9 }}, {limit}); return result.value * 100 + result.index;"),
                );
                assert!(optimized(&source, "work").1.is_some());
                let module = parse_nuis_module(&source).unwrap();
                let layouts = control_values::layouts(&module);
                let mut independent = module.clone();
                for function in &mut independent.functions {
                    if let Some(body) = returns::normalize(function, &layouts).unwrap() {
                        function.body = body;
                    }
                }
                let execute = |module: &NirModule| {
                    let mut yir =
                        crate::lowering::lower_nir_to_yir(module, &manifest, None).unwrap();
                    yir.nodes.reverse();
                    yir.functions.reverse();
                    for function in &mut yir.functions {
                        function.body_nodes.reverse();
                    }
                    yir_verify::verify_module(&yir).unwrap();
                    let trace = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    );
                    trace
                        .map(|trace| {
                            let entry = yir
                                .functions
                                .iter()
                                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                                .unwrap();
                            trace.values[&entry.result.as_ref().unwrap().node].clone()
                        })
                        .map_err(|_| ())
                };
                let actual = execute(&module);
                assert_eq!(actual, execute(&independent), "{source}");
                assert_eq!(
                    actual.is_err(),
                    stop == 4 && if leading { limit >= 3 } else { limit >= 4 }
                );
                if limit == 0 {
                    assert_eq!(actual, Ok(yir_core::Value::Int(4109)));
                }
            }
        }
    }
}
