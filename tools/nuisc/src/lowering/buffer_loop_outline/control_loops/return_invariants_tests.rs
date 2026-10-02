use super::*;
use crate::frontend::parse_nuis_module;

const MIXED: &str = include_str!(
    "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
);

pub(super) fn promoted(source: &str) -> (NirFunction, Vec<NirStmt>, Vec<NirStmt>) {
    let module = parse_nuis_module(source).unwrap();
    let mut layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "step").unwrap();
    assert!(catalog.contains_key("step"), "{source}");
    let baseline = coalesced(function, &layouts, &catalog).unwrap();
    let (body, structs) = rewrite(
        function,
        &baseline.body,
        &baseline.signal,
        &layouts,
        &catalog,
        &BTreeSet::new(),
        true,
    )
    .unwrap();
    let plan = prepare(function, &layouts, &catalog, &BTreeSet::new())
        .unwrap()
        .unwrap();
    assert_eq!(plan.structs, structs);
    for definition in structs {
        layouts.insert(
            definition.name,
            definition
                .fields
                .into_iter()
                .map(|f| (f.name, f.ty))
                .collect(),
        );
    }
    assert!(scalar_helpers::validate_control_body(function, &body, &catalog, &layouts).is_some());
    assert_eq!(plan.body, body);
    (function.clone(), baseline.body, body)
}

fn all_bindings(body: &[NirStmt]) -> Vec<&str> {
    let mut blocks = vec![body];
    let mut result = Vec::new();
    while let Some(body) = blocks.pop() {
        for stmt in body {
            match stmt {
                NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => {
                    result.push(name.as_str())
                }
                NirStmt::While { body, .. } => blocks.push(body),
                NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } => blocks.extend([then_body.as_slice(), else_body.as_slice()]),
                _ => {}
            }
        }
    }
    result
}

#[test]
fn return_invariants_promote_proven_nested_leaves_and_revalidate_the_function() {
    let (function, baseline, body) = promoted(MIXED);
    assert_ne!(baseline, body);
    let outer = body
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::While { body, .. } => Some(body),
            _ => None,
        })
        .unwrap();
    assert!(!all_bindings(outer).contains(&"carry"));
    let inner = outer
        .iter()
        .find_map(|stmt| match stmt {
            NirStmt::While { body, .. } => Some(body),
            _ => None,
        })
        .unwrap();
    let writes = all_bindings(inner);
    assert!(!writes.contains(&"carry"));
    assert!(!writes.contains(&"selected"));
    assert_eq!(
        writes
            .iter()
            .filter(|n| n.starts_with("__nuis_loop_snapshot"))
            .count(),
        4
    );
    let fields = writes
        .iter()
        .filter(|n| n.starts_with("__nuis_loop_mutable"))
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(fields.len(), 2);
    assert_eq!(
        function.body,
        parse_nuis_module(MIXED)
            .unwrap()
            .functions
            .into_iter()
            .find(|f| f.name == "step")
            .unwrap()
            .body
    );
}

#[test]
fn return_invariants_keep_ordinary_breaks_and_unproved_fields_on_the_old_path() {
    let source = MIXED.replace("if j == 2 {", "if j == 1 { break; } if j == 2 {");
    let module = parse_nuis_module(&source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "step").unwrap();
    let baseline = coalesced(function, &layouts, &catalog).unwrap();
    let (candidate, _) = rewrite(
        function,
        &baseline.body,
        &baseline.signal,
        &layouts,
        &catalog,
        &BTreeSet::new(),
        true,
    )
    .unwrap();
    assert!(
        all_bindings(&candidate).contains(&"selected"),
        "child keeps its own ordinary break"
    );
    assert_ne!(candidate, baseline.body);
    assert_eq!(
        prepare(function, &layouts, &catalog, &BTreeSet::new())
            .unwrap()
            .unwrap()
            .body,
        baseline.body
    );
}

#[test]
fn return_invariants_keep_all_stable_rhs_without_creating_empty_record_types() {
    let source = "mod cpu Main {
        struct State { value: i64, tag: i64 }
        fn step(seed: State, limit: i64) -> State {
            let state = seed; let i = 0;
            while i < limit { let i = i + 1; let state = state;
                if i == 2 { return state; }
            }
            return state;
        }
        fn main() -> i64 { let state = step(State { value: 37, tag: 5 }, 3); return state.value; }
    }";
    promoted(source);
    let module = parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let plan = prepare(&module.functions[0], &layouts, &catalog, &BTreeSet::new())
        .unwrap()
        .unwrap();
    assert!(plan.structs.is_empty());
    assert_eq!(
        all_bindings(&plan.body)
            .into_iter()
            .filter(|name| name.starts_with("__nuis_loop_snapshot"))
            .count(),
        2
    );
    crate::pipeline::compile_source(source).unwrap();
}

#[test]
fn return_invariants_fresh_names_and_failed_attempts_leave_input_untouched() {
    let source = MIXED.replace("let carry = state;", "let __nuis_loop_mutable_0 = state.count; let __nuis_loop_snapshot_0 = state.count; let carry = state;")
        .replace("struct Leaf", "struct __nuis_loop_fields_0 { kept: i64 } struct Leaf");
    let (_, baseline, body) = promoted(&source);
    for name in ["__nuis_loop_mutable_0", "__nuis_loop_snapshot_0"] {
        assert_eq!(
            all_bindings(&body)
                .into_iter()
                .filter(|n| *n == name)
                .count(),
            1
        );
    }
    let module = parse_nuis_module(&source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "step").unwrap();
    let plan = coalesced(function, &layouts, &catalog).unwrap();
    let mut pass = Pass {
        signal: &plan.signal,
        layouts: layouts.clone(),
        catalog: &catalog,
        names: BTreeSet::new(),
        budget: Budget(100),
        changed: false,
        structs: Vec::new(),
        nested: true,
    };
    let scope = function
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    assert!(pass.run(&plan.body, scope).is_none());
    assert_eq!(plan.body, baseline);
}

#[test]
fn return_invariants_match_independent_payloads_zero_trips_and_selected_traps() {
    let manifest =
        crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
            .unwrap();
    for leading in [true, false] {
        for limit in 0..=4 {
            for stop in [0, 2, 4] {
                for trap in [false, true] {
                    let step = "let i = i + 1;";
                    let source = format!("mod cpu Main {{
                        struct State {{ value: i64, tag: i64 }}
                        fn step(seed: State, limit: i64) -> State {{
                            let state = seed; let i = 0;
                            while i < limit {{
                                {}
                                let state = state;
                                let old = state;
                                if i == {stop} {{ return State {{ value: old.value + 2, tag: old.tag }}; }}
                                let unused = {};
                                if i > 1 {{ let state = State {{ tag: old.tag, value: old.value + 3 }}; }}
                                else {{ let state = State {{ value: old.value + 1, tag: old.tag }}; }}
                                {}
                            }}
                            return state;
                        }}
                        fn main() -> i64 {{ let result = step(State {{ value: 10, tag: 7 }}, {limit}); return result.value * 100 + result.tag; }}
                    }}", if leading { step } else { "" }, if trap { "1 / (i - 3)" } else { "i" }, if leading { "" } else { step });
                    promoted(&source);
                    let module = parse_nuis_module(&source).unwrap();
                    let layouts = control_values::layouts(&module);
                    let mut independent = module.clone();
                    for function in &mut independent.functions {
                        if let Some(body) = normalize(function, &layouts).unwrap() {
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
                        yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        )
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
                    let mut value = 10;
                    let mut expected = Ok(yir_core::Value::Int(1007));
                    for trip in 0..limit {
                        let i = trip + i32::from(leading);
                        if i == stop {
                            value += 2;
                            expected = Ok(yir_core::Value::Int(value * 100 + 7));
                            break;
                        }
                        if trap && i == 3 {
                            expected = Err(());
                            break;
                        }
                        value += if i > 1 { 3 } else { 1 };
                        expected = Ok(yir_core::Value::Int(value * 100 + 7));
                    }
                    assert_eq!(actual, expected, "{source}");
                }
            }
        }
    }
}
