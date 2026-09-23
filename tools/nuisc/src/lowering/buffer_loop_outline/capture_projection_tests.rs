use super::*;

const RECORDS: &str = "struct Pair { x: i64, y: i64 }
    struct State { a: Pair, b: Pair, unused: i64 }";

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!("mod cpu Main {{ {RECORDS} {body} }}")).unwrap()
}

fn run(module: &mut NirModule, generated: &[&str]) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let generated = generated.iter().map(|s| (*s).to_owned()).collect();
    let changed = project(module, &generated, &layouts);
    crate::nir_verify::verify_nir_module(module).unwrap();
    changed
}

fn function<'a>(module: &'a NirModule, name: &str) -> &'a NirFunction {
    module.functions.iter().find(|f| f.name == name).unwrap()
}

#[test]
fn projection_flows_from_nested_helpers_to_callers_in_any_storage_order() {
    for reverse in [false, true] {
        let mut module = module(
            "fn outer(state: State, flag: bool) -> i64 {
                return inner(state, flag) + state.a.x;
            }
            fn inner(state: State, flag: bool) -> i64 {
                if flag { return state.a.x; } return state.b.y;
            }
            fn entry(state: State, flag: bool) -> i64 { return outer(state, flag); }",
        );
        if reverse {
            module.functions.reverse();
        }
        assert!(run(&mut module, &["outer", "inner"]));
        for name in ["outer", "inner"] {
            let helper = function(&module, name);
            assert_eq!(
                helper
                    .params
                    .iter()
                    .map(|p| p.ty.name.as_str())
                    .collect::<Vec<_>>(),
                ["i64", "i64", "bool"]
            );
            let mut roots = BTreeSet::new();
            walk::visit(&helper.body, |expr| {
                if let NirExpr::Var(name) = expr {
                    roots.insert(name.clone());
                }
                true
            });
            assert!(!roots.contains("state"));
        }
        assert_eq!(function(&module, "entry").params[0].ty.name, "State");
        assert!(!run(&mut module, &["outer", "inner"]));
    }
}

#[test]
fn projection_retains_exact_nominal_subrecords_and_deduplicates_prefix_uses() {
    let mut module = module(
        "fn consume(pair: Pair) -> i64 { return pair.x; }
        fn helper(state: State) -> i64 { return consume(state.a) + state.a.y; }
        fn entry(state: State) -> i64 { return helper(state); }",
    );
    assert!(run(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(helper.params.len(), 1);
    assert_eq!(helper.params[0].ty.name, "Pair");
    assert_eq!(function(&module, "consume").params[0].name, "pair");
}

#[test]
fn projection_keeps_whole_aliases_returns_and_rebound_parameters() {
    for body in [
        "let alias = state; return alias.a.x;",
        "let state = State { a: state.b, b: state.a, unused: 0 }; return state.a.x;",
        "if flag { let state = State { a: state.b, b: state.a, unused: 0 }; } return state.a.x;",
        "return consume(state);",
    ] {
        let mut module = module(&format!(
            "fn consume(state: State) -> i64 {{ return state.a.x; }}
            fn helper(state: State, flag: bool) -> i64 {{ {body} }}
            fn entry(state: State, flag: bool) -> i64 {{ return helper(state, flag); }}"
        ));
        assert!(!run(&mut module, &["helper"]));
        assert_eq!(function(&module, "helper").params[0].ty.name, "State");
    }
}

#[test]
fn projection_does_not_duplicate_or_discard_computed_record_arguments() {
    for body in ["return state.a.x;", "return 7;"] {
        let mut module = module(&format!(
            "fn computed(state: State) -> State {{ print(123); return state; }}
            fn helper(state: State) -> i64 {{ {body} }}
            fn entry(state: State) -> i64 {{ return helper(computed(state)); }}"
        ));
        assert!(!run(&mut module, &["helper"]));
    }
}

#[test]
fn projection_keeps_scalar_arguments_and_uses_hygienic_names() {
    let mut module = module(
        "fn helper(state: State, unused: i64, __nuis_capture_field_0: i64) -> i64 {
            let __nuis_capture_field_1 = 3;
            return state.a.x + __nuis_capture_field_0 + __nuis_capture_field_1;
        }
        fn entry(state: State, divisor: i64) -> i64 { return helper(state, 1 / divisor, 8); }",
    );
    assert!(run(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(helper.params[0].name, "__nuis_capture_field_2");
    assert_eq!(helper.params[1].name, "unused");
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &function(&module, "entry").body[0]
    else {
        panic!()
    };
    assert!(matches!(
        args[1],
        NirExpr::Binary {
            op: NirBinaryOp::Div,
            ..
        }
    ));
}

#[test]
fn projection_drops_only_unused_ready_records() {
    let mut module = module(
        "fn helper(state: State, flag: bool) -> bool { return flag; }
        fn entry(state: State, flag: bool) -> bool { return helper(state, flag); }",
    );
    assert!(run(&mut module, &["helper"]));
    assert_eq!(function(&module, "helper").params.len(), 1);
    assert_eq!(function(&module, "helper").params[0].name, "flag");
}

#[test]
fn projection_excludes_scoped_targets_and_generated_call_cycles() {
    let mut module = module(
        "fn helper(state: State) -> i64 { return state.a.x; }
        fn entry(state: State) -> i64 {
            let i = 0; while i < 2 { let value = helper(state); let i = i + 1; } return 0;
        }",
    );
    assert!(!run(&mut module, &["helper"]));
    let mut module = self::module(
        "fn first(state: State) -> i64 { return second(state); }
        fn second(state: State) -> i64 { return first(state) + state.a.x; }
        fn outer(state: State) -> i64 { return first(state); }",
    );
    assert!(!run(&mut module, &["first", "second", "outer"]));
}

#[test]
fn projection_vetoes_calls_hidden_in_unsupported_expressions() {
    for wrap in [NirExpr::Move, NirExpr::KernelRelu, NirExpr::CastI64ToF64] {
        let mut module = module(
            "fn helper(state: State) -> i64 { return state.a.x; }
        fn entry(state: State) -> i64 { return helper(state); }",
        );
        let entry = module
            .functions
            .iter_mut()
            .find(|f| f.name == "entry")
            .unwrap();
        let NirStmt::Return(Some(expr)) = &mut entry.body[0] else {
            panic!()
        };
        *expr = wrap(Box::new(expr.clone()));
        let layouts = control_values::TypedLayouts::collect(&module);
        assert!(!project(
            &mut module,
            &BTreeSet::from(["helper".into()]),
            &layouts
        ));
        assert_eq!(function(&module, "helper").params[0].ty.name, "State");
    }
}

#[test]
fn projection_tracks_field_demand_beneath_integer_conversions() {
    let mut module = module(
        "fn helper(state: State) -> i64 { return state.a.x; }
        fn entry(state: State) -> i64 { return helper(state); }",
    );
    let helper = module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap();
    let NirStmt::Return(Some(expr)) = &mut helper.body[0] else {
        panic!()
    };
    *expr = NirExpr::CastI32ToI64(Box::new(NirExpr::CastI64ToI32(Box::new(expr.clone()))));
    assert!(run(&mut module, &["helper"]));
    assert_eq!(function(&module, "helper").params.len(), 1);
    assert_eq!(function(&module, "helper").params[0].ty.name, "i64");
}

#[test]
fn projected_local_choices_preserve_effectful_predicate_once_and_old_snapshots() {
    let source = "mod cpu Main {
        struct State { left: i64, right: i64, unused: i64 }
        fn predicate(flag: i64) -> bool { print(flag); return flag > 0; }
        fn relay(value: i64) -> i64 { return value; }
        @noinline fn event(flag: i64, divisor: i64) -> i64 {
            let state = State { left: 12, right: 9, unused: 80 };
            let saved = state;
            let selected: i64 = if predicate(flag) { relay(state.left) } else { state.right / divisor };
            let state = State { left: selected, right: 0, unused: 0 };
            print(saved.left);
            return state.left;
        }
        fn main() -> i64 { print(event(1, 0)); print(event(0, 2)); return 0; }
    }";
    let yir = crate::pipeline::compile_source(source).unwrap().yir;
    let selection = yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_conditional_value"))
        .unwrap();
    assert_eq!(selection.parameters.len(), 4);
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
    for (line, value) in prints.iter().zip([1, 12, 12, 0, 12, 4]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    yir_lower_llvm::emit_module(&yir).unwrap();
}
