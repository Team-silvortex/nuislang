use super::*;

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Pair {{ x: i64, y: i64 }}
        struct State {{ a: Pair, b: Pair, unused: i64 }} {body}
    }}"
    ))
    .unwrap()
}

fn function<'a>(module: &'a NirModule, name: &str) -> &'a NirFunction {
    module.functions.iter().find(|f| f.name == name).unwrap()
}

fn project_scoped(module: &mut NirModule, names: &[&str]) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let names = names.iter().map(|name| (*name).to_owned()).collect();
    let changed = project(module, &names, &names, &layouts);
    crate::nir_verify::verify_nir_module(module).unwrap();
    changed
}

#[test]
fn scoped_projection_keeps_rebound_record_seeds_by_argument_identity() {
    for reverse in [false, true] {
        let mut module = module(
            "fn helper(input: State, old: Pair, index: i64) -> Pair {
            let saved = input;
            return Pair { x: old.x + saved.a.x + index, y: saved.b.y };
        }
        fn entry(state: State) -> Pair {
            let i = 0; let carry = state.a;
            while i < 2 { let carry = helper(state, carry, i); let i = i + 1; }
            return carry;
        }",
        );
        let entry = function(&module, "entry").clone();
        if reverse {
            module.functions.reverse();
        }
        assert!(project_scoped(&mut module, &["helper"]));
        let helper = function(&module, "helper");
        assert_eq!(
            helper
                .params
                .iter()
                .map(|p| p.ty.name.as_str())
                .collect::<Vec<_>>(),
            ["i64", "i64", "Pair", "i64"]
        );
        assert_eq!(helper.params[2].name, "old");
        assert_eq!(helper.params[3].name, "index");
        let NirStmt::While { body, .. } = &function(&module, "entry").body[2] else {
            panic!()
        };
        let NirStmt::Let {
            value: NirExpr::Call { args, .. },
            ..
        } = &body[0]
        else {
            panic!()
        };
        assert_eq!(
            &args[2..],
            &[NirExpr::Var("carry".into()), NirExpr::Var("i".into())]
        );
        assert_eq!(function(&module, "entry").params, entry.params);
        assert!(!project_scoped(&mut module, &["helper"]));
    }
}

#[test]
fn scoped_projection_unions_writes_across_callers_and_nested_scopes() {
    for write in [
        "let current = state;",
        "if i > 0 { let current = state; }",
        "let j = 0; while j < 1 { let current = state; let j = j + 1; }",
    ] {
        let mut module = module(&format!(
            "fn helper(input: State) -> i64 {{ return input.a.x; }}
        fn first(state: State) -> i64 {{
            let i = 0; while i < 2 {{ let value = helper(state); let i = i + 1; }} return 0;
        }}
        fn second(state: State) -> i64 {{
            let current = state; let i = 0;
            while i < 2 {{ let value = helper(current); {write} let i = i + 1; }} return 0;
        }}"
        ));
        let before = module.clone();
        assert!(!project_scoped(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn scoped_projection_keeps_computed_and_whole_uses_transactional() {
    for argument in ["computed(state)", "state"] {
        let result = if argument == "state" {
            "consume(input)"
        } else {
            "input.a.x"
        };
        let mut module = module(&format!(
            "fn computed(state: State) -> State {{ print(9); return state; }}
        fn consume(state: State) -> i64 {{ return state.a.x; }}
        fn helper(input: State) -> i64 {{ let saved = input; return {result}; }}
        fn entry(state: State) -> i64 {{
            let i = 0; while i < 2 {{ let value = helper({argument}); let i = i + 1; }} return 0;
        }}"
        ));
        let before = module.clone();
        assert!(!project_scoped(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

fn execute(source: &str) -> (yir_core::YirModule, Vec<String>) {
    let compiled = crate::pipeline::compile_source(source).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let prints = trace
        .events
        .into_iter()
        .filter(|e| e.contains("cpu.print"))
        .collect();
    (compiled.yir, prints)
}

#[test]
fn scoped_field_inputs_preserve_zero_trips_and_induction_carry_maps() {
    for trailing in [false, true] {
        let step = "let i = i + 1;";
        let source = format!(
            "mod cpu Main {{
            struct Input {{ numerator: i64, denominator: i64, unused: i64 }}
            fn relay(value: i64) -> i64 {{ return value; }}
            @noinline fn walk(input: Input, limit: i64) -> i64 {{
                let i = 0; let total = 1;
                while i < limit {{ {} let saved = input;
                    let total = total + relay(saved.numerator) / saved.denominator; {} }}
                return total + i * 100;
            }}
            fn main() -> i64 {{
                print(walk(Input {{ numerator: 6, denominator: 0, unused: 9 }}, 0));
                print(walk(Input {{ numerator: 6, denominator: 2, unused: 9 }}, 3));
                return 0;
            }}
        }}",
            if trailing { "" } else { step },
            if trailing { step } else { "" }
        );
        let (yir, prints) = execute(&source);
        assert_eq!(prints.len(), 2);
        for (line, value) in prints.iter().zip([1, 310]) {
            assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
        }
        let iteration = yir
            .functions
            .iter()
            .find(|f| f.name.starts_with("__nuis_scalar_iteration_"))
            .unwrap();
        assert_eq!(iteration.parameters.len(), 4);
        let driver = yir
            .nodes
            .iter()
            .find(|n| n.op.args.iter().any(|a| a == "scoped_call_i64_carry"))
            .unwrap();
        assert_eq!(
            driver
                .op
                .args
                .iter()
                .filter(|a| a.as_str() == "$carry")
                .count(),
            1
        );
        assert_eq!(
            driver
                .op
                .args
                .iter()
                .filter(|a| a.as_str() == "$current")
                .count(),
            1
        );
    }
}

#[test]
fn scoped_field_inputs_preserve_bool_record_and_break_carry_slots() {
    let (yir, prints) = execute(
        "mod cpu Main {
        struct Input { delta: i64, stop: i64, unused: i64 }
        struct Pair { x: i64, y: i64 }
        @noinline fn walk(input: Input, limit: i64) -> i64 {
            let i = 0; let pair = Pair { x: 1, y: 2 }; let flag = false; let total = 0;
            while i < limit {
                let i = i + 1; let saved = input;
                let pair = Pair { x: pair.x + saved.delta, y: pair.y + 1 };
                let flag = !flag; let total = total + pair.x;
                if i == saved.stop { break; }
            }
            if flag { return pair.x + pair.y * 10 + total * 100 + i * 1000; }
            return pair.x + pair.y * 10 + total * 100 + i * 1000 + 10000;
        }
        fn main() -> i64 {
            print(walk(Input { delta: 3, stop: 2, unused: 9 }, 0));
            print(walk(Input { delta: 3, stop: 2, unused: 9 }, 4));
            print(walk(Input { delta: 3, stop: 9, unused: 9 }, 3));
            return 0;
        }
    }",
    );
    assert_eq!(prints.len(), 3);
    for (line, value) in prints.iter().zip([10021, 13147, 5160]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    assert!(yir.nodes.iter().any(|n| n
        .op
        .args
        .iter()
        .any(|a| a == "scoped_call_i64_carries_break")));
    let iteration = yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_scalar_iteration_"))
        .unwrap();
    assert!(!iteration
        .parameters
        .iter()
        .any(|p| p.name.contains("unused")));
}

#[test]
fn scoped_field_inputs_keep_nested_break_controls_local() {
    let (yir, prints) = execute(
        "mod cpu Main {
        struct Input { delta: i64, stop: i64, unused: i64 }
        @noinline fn walk(input: Input, limit: i64) -> i64 {
            let i = 0; let total = 0;
            while i < limit {
                let i = i + 1; let j = 0;
                while j < 2 {
                    let j = j + 1; let saved = input;
                    let total = total + saved.delta;
                    if j == 1 { break; }
                }
                if i == input.stop { break; }
            }
            return total + i * 100;
        }
        fn main() -> i64 {
            print(walk(Input { delta: 3, stop: 2, unused: 9 }, 0));
            print(walk(Input { delta: 3, stop: 2, unused: 9 }, 4));
            return 0;
        }
    }",
    );
    assert_eq!(prints.len(), 2);
    for (line, value) in prints.iter().zip([0, 206]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    assert_eq!(
        yir.nodes
            .iter()
            .filter(|n| n
                .op
                .args
                .iter()
                .any(|a| a == "scoped_call_i64_carries_break"))
            .count(),
        2
    );
}
