use super::tests::{cases, plans};
use super::*;

fn lowered(source: &str, independent: bool) -> yir_core::YirModule {
    let (mut module, layouts, _) = plans(source, "work");
    if independent {
        for function in &mut module.functions {
            if let Some(body) = normalize_effects(function, &layouts).unwrap() {
                function.body = body;
            }
        }
    }
    let manifest =
        crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
            .unwrap();
    let yir = crate::lowering::lower_nir_to_yir(&module, &manifest, None).unwrap();
    yir_verify::verify_module(&yir).unwrap();
    yir
}

fn maximum_carry_width(module: &yir_core::YirModule) -> usize {
    module
        .nodes
        .iter()
        .filter_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args).unwrap()
        })
        .map(|call| call.seeds.len())
        .max()
        .unwrap()
}

fn invoked_source(call: cases::Call, kind: cases::SourceKind) -> String {
    let args = call
        .words()
        .into_iter()
        .enumerate()
        .map(|(slot, word)| match slot {
            0 | 4 => (word != 0).to_string(),
            2 => format!("{:?}", f32::from_bits(word as u32)),
            3 => format!("{:?}", f64::from_bits(word as u64)),
            _ => word.to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
    let source = kind.source(call.variant);
    source.replace(
        "fn main() -> i64 { return 0; }",
        &format!("fn main() -> i64 {{ let finished = start({args}); return finished.total; }}"),
    )
}

#[test]
fn effect_return_storage_removes_four_real_yir_carry_words_not_just_a_local_binding() {
    for variant in cases::variants() {
        let source = invoked_source(cases::base(variant), cases::SourceKind::Plain);
        let independent = lowered(&source, true);
        let shared = lowered(&source, false);
        let before = maximum_carry_width(&independent);
        let after = maximum_carry_width(&shared);
        assert_eq!(after + 4, before, "{variant:?}: {before} -> {after}");
    }
}

#[test]
fn effect_return_storage_matches_independent_reference_results_prints_and_selected_traps() {
    compare_sources(false);
}

#[test]
fn effect_return_storage_prefix_keeps_real_carry_reduction_and_ordered_effects() {
    compare_sources(true);
}

fn compare_sources(printed: bool) {
    compare_source_cases(if printed {
        cases::SourceKind::Prefix
    } else {
        cases::SourceKind::Plain
    });
}

#[test]
fn effect_return_storage_candidates_keep_real_reduction_order_and_faults() {
    compare_source_cases(cases::SourceKind::Candidates);
}

#[test]
fn effect_return_storage_reassignment_keeps_real_reduction_and_independent_values_effects() {
    compare_source_cases(cases::SourceKind::Reassigned);
}

fn compare_source_cases(kind: cases::SourceKind) {
    for variant in cases::variants() {
        let mut failure = cases::base(variant);
        failure.root = 0;
        for call in [
            cases::base(variant),
            cases::continued_then_break(variant),
            failure,
        ] {
            let source = invoked_source(call, kind);
            let independent = lowered(&source, true);
            let shared = lowered(&source, false);
            assert_eq!(
                maximum_carry_width(&shared) + 4,
                maximum_carry_width(&independent)
            );
            for reversed in [false, true] {
                let run = |mut yir: yir_core::YirModule| {
                    if reversed {
                        yir.nodes.reverse();
                        yir.functions.reverse();
                        for function in &mut yir.functions {
                            function.body_nodes.reverse();
                        }
                    }
                    let entry = yir
                        .functions
                        .iter()
                        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                        .unwrap();
                    let result = entry.result.as_ref().unwrap().node.clone();
                    yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    )
                    .map(|trace| {
                        let printed = trace
                            .events
                            .iter()
                            .filter(|e| {
                                e.contains("cpu.print")
                                    || (e.contains("cpu.guard_print")
                                        && e.contains("if true then print"))
                            })
                            .map(|e| e.rsplit_once(' ').unwrap().1.parse::<i64>().unwrap())
                            .collect::<Vec<_>>();
                        (trace.values[&result].clone(), printed)
                    })
                    .map_err(|_| ())
                };
                let actual = run(shared.clone());
                assert_eq!(actual, run(independent.clone()));
                match call.execute(false, None) {
                    Ok((state, trace)) => {
                        assert_eq!(
                            actual,
                            Ok((yir_core::Value::Int(state.total), kind.trace(trace)))
                        )
                    }
                    Err(_) => assert!(actual.is_err()),
                }
            }
        }
    }
}

#[test]
fn effect_return_storage_evaluates_every_record_field_before_overwriting_its_old_value() {
    for root in [0, 1] {
        let source = format!("mod cpu Main {{ struct State {{ left: i64, right: i64 }}
            fn work(seed: State, root: i64) -> State {{
                let mut carry = seed; let mut index = 0;
                while index < 2 {{ index = index + 1; carry = carry; print(7);
                    if index == 1 {{ return State {{ left: carry.right, right: carry.left + 1 / root }}; }}
                }} return carry;
            }} fn main() -> i64 {{ let finished = work(State {{ left: 41, right: 9 }}, {root}); return finished.left * 100 + finished.right; }} }}");
        let baseline = lowered(&source, true);
        let shared = lowered(&source, false);
        assert_eq!(
            maximum_carry_width(&shared) + 2,
            maximum_carry_width(&baseline)
        );
        let run = |module: &yir_core::YirModule| {
            let result = &module
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .node;
            yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(module),
                &yir_verify::default_registry(),
            )
            .map(|trace| trace.values[result].clone())
            .map_err(|_| ())
        };
        assert_eq!(run(&shared), run(&baseline));
        assert_eq!(
            run(&shared),
            if root == 0 {
                Err(())
            } else {
                Ok(yir_core::Value::Int(942))
            }
        );
    }
}
