use super::*;
use crate::frontend::parse_nuis_module;

#[test]
fn return_invariants_keep_admitted_inner_plan_when_outer_revalidation_fails() {
    let source = include_str!(
        "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    )
    .replace(
        "let enabled = selected.enabled;",
        "let enabled = selected.enabled;
            let k = 0; let child_limit = 1;
            while k < child_limit { let k = k + 1; let carry = carry; if k == 1 { break; } }",
    );
    let module = parse_nuis_module(&source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "step").unwrap();
    let baseline = coalesced(function, &layouts, &catalog).unwrap();
    let attempt = |nested| {
        rewrite(
            function,
            &baseline.body,
            &baseline.signal,
            &layouts,
            &catalog,
            &BTreeSet::new(),
            nested,
        )
        .unwrap()
    };
    let outer = attempt(true);
    let inner = attempt(false);
    assert_ne!(outer.0, inner.0);
    assert_ne!(inner.0, baseline.body);
    let valid = |(body, structs): &(Vec<NirStmt>, Vec<NirStructDef>)| {
        let mut module = module.clone();
        module.structs.extend(structs.clone());
        scalar_helpers::validate_control_body(
            function,
            body,
            &catalog,
            &control_values::layouts(&module),
        )
        .is_some()
    };
    assert!(!valid(&outer));
    assert!(valid(&inner));
    let prepared = prepare(function, &layouts, &catalog, &BTreeSet::new())
        .unwrap()
        .unwrap();
    assert_eq!(prepared.body, inner.0);
    assert_eq!(prepared.structs, inner.1);
    assert_eq!(execute(&module), Ok(19));
}

fn execute(module: &NirModule) -> Result<i64, ()> {
    let manifest =
        crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
            .unwrap();
    let mut yir = crate::lowering::lower_nir_to_yir(module, &manifest, None).unwrap();
    yir.nodes.reverse();
    yir.functions.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    yir_verify::verify_module(&yir).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&yir),
        &yir_verify::default_registry(),
    )
    .map_err(|_| ())?;
    let entry = yir
        .functions
        .iter()
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap();
    let yir_core::Value::Int(value) = trace.values[&entry.result.as_ref().unwrap().node] else {
        panic!("scalar result")
    };
    Ok(value)
}

#[test]
fn return_invariant_nested_loops_match_observed_indices_exits_and_arithmetic_oracle() {
    for leading in [true, false] {
        for limit in [0, 1, 3, 4] {
            for inner_limit in [0, 1, 3] {
                for exit in ["", "break;", "continue;"] {
                    for trap in [false, true] {
                        let step = "let i = i + 1;";
                        let stop = if trap { 4 } else { 2 };
                        let exit_body = if exit.is_empty() {
                            "let noop = j;"
                        } else {
                            exit
                        };
                        let source = format!("mod cpu Main {{
                            struct State {{ value: i64, tag: i64 }}
                            fn step(seed: State, limit: i64, inner_limit: i64) -> State {{
                                let carry = seed; let i = 0;
                                while i < limit {{
                                    {}
                                    let carry = carry; let selected = carry; let j = 0;
                                    while j < inner_limit {{
                                        let j = j + 1; let selected = selected; let old = selected;
                                        if j == 2 {{ {exit_body} }}
                                        let selected = State {{ value: old.value + j, tag: old.tag }};
                                        if i == {stop} {{ if j == 3 {{
                                            return State {{ value: selected.value + 4, tag: selected.tag }};
                                        }} }}
                                        if i == 3 {{ if j == 3 {{ let unused = {}; }} }}
                                    }}
                                    let carry = State {{ value: selected.value + 5, tag: selected.tag }};
                                    {}
                                }}
                                return State {{ value: carry.value, tag: carry.tag + i * 1000 }};
                            }}
                            fn main() -> i64 {{
                                let result = step(State {{ value: 10, tag: 7 }}, {limit}, {inner_limit});
                                return result.value * 100 + result.tag;
                            }}
                        }}", if leading { step } else { "" }, if trap { "1 / (i - 3)" } else { "i" }, if leading { "" } else { step });
                        if exit.is_empty() {
                            tests::promoted(&source);
                        }
                        let module = parse_nuis_module(&source).unwrap();
                        let layouts = control_values::layouts(&module);
                        let mut independent = module.clone();
                        for function in &mut independent.functions {
                            if let Some(body) = normalize(function, &layouts).unwrap() {
                                function.body = body;
                            }
                        }
                        let actual = execute(&module);
                        assert_eq!(actual, execute(&independent), "{source}");
                        let expected = (|| {
                            let mut value = 10;
                            for trip in 0..limit {
                                let i = trip + i32::from(leading);
                                for j in 1..=inner_limit {
                                    if j == 2 {
                                        if exit == "break;" {
                                            break;
                                        }
                                        if exit == "continue;" {
                                            continue;
                                        }
                                    }
                                    value += i64::from(j);
                                    if i == stop && j == 3 {
                                        return Ok((value + 4) * 100 + 7);
                                    }
                                    if trap && i == 3 && j == 3 {
                                        return Err(());
                                    }
                                }
                                value += 5;
                            }
                            Ok(value * 100 + 7 + i64::from(limit) * 1000)
                        })();
                        assert_eq!(actual, expected, "{source}");
                    }
                }
            }
        }
    }
}
