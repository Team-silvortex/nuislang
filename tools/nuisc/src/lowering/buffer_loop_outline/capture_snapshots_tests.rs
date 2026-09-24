use super::*;

fn module(body: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Pair {{ x: i64, y: i64 }}
        struct State {{ a: Pair, b: Pair, unused: i64 }}
        {body}
    }}"
    ))
    .unwrap()
}

fn function<'a>(module: &'a NirModule, name: &str) -> &'a NirFunction {
    module.functions.iter().find(|f| f.name == name).unwrap()
}

fn project(module: &mut NirModule, names: &[&str]) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let changed = super::super::project(
        module,
        &names.iter().map(|name| (*name).to_owned()).collect(),
        &layouts,
    );
    crate::nir_verify::verify_nir_module(module).unwrap();
    changed
}

#[test]
fn straight_line_origin_and_alias_rebindings_expose_old_field_demand() {
    for (name, returned) in [
        ("state", "saved.a.x + state.b.y"),
        ("saved", "old.a.x + saved.b.y"),
    ] {
        let mut module = module(&format!(
            "fn helper(state: State) -> i64 {{
            let saved = state; let old = saved;
            let {name} = State {{ a: Pair {{ x: 30, y: 0 }}, b: Pair {{ x: 0, y: 7 }}, unused: 0 }};
            return {returned};
        }} fn entry(state: State) -> i64 {{ return helper(state); }}"
        ));
        assert!(project(&mut module, &["helper"]));
        let helper = function(&module, "helper");
        assert_eq!(helper.params.len(), 1);
        assert_eq!(helper.params[0].ty.name, "i64");
        assert_eq!(helper.body.len(), 2);
        assert!(
            matches!(&helper.body[0], NirStmt::Let { name, .. } if name.starts_with("__nuis_capture_snapshot_"))
        );
        assert_eq!(function(&module, "entry").params[0].ty.name, "State");
        assert!(!project(&mut module, &["helper"]));
    }
}

#[test]
fn self_rebinding_reads_previous_version_and_reserves_future_names() {
    let mut module = module(
        "fn helper(state: State, __nuis_capture_snapshot_0: i64) -> i64 {
        let saved = state;
        let state = State { a: state.b, b: state.a, unused: 0 };
        let state = State { a: state.b, b: state.a, unused: 0 };
        let __nuis_capture_snapshot_1 = 9;
        return saved.a.x + state.b.y + __nuis_capture_snapshot_0 + __nuis_capture_snapshot_1;
    }",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let helper = &mut module.functions[0];
    normalize(helper, &layouts);
    let first = &helper.body[1];
    let second = &helper.body[2];
    assert!(matches!(first, NirStmt::Let { name, .. } if name == "__nuis_capture_snapshot_2"));
    assert!(matches!(second, NirStmt::Let { name, .. } if name == "__nuis_capture_snapshot_3"));
    let reads = |stmt: &NirStmt| {
        let mut reads = BTreeSet::new();
        walk::visit(std::slice::from_ref(stmt), |expr| {
            if let NirExpr::Var(name) = expr {
                reads.insert(name.clone());
            }
            true
        });
        reads
    };
    assert_eq!(reads(first), BTreeSet::from(["state".to_owned()]));
    assert_eq!(
        reads(second),
        BTreeSet::from(["__nuis_capture_snapshot_2".to_owned()])
    );
    crate::nir_verify::verify_nir_module(&module).unwrap();
    let before = module.clone();
    normalize(&mut module.functions[0], &layouts);
    assert_eq!(module, before);
}

#[test]
fn snapshot_projection_keeps_nominal_subrecords_in_private_call_chains() {
    for reverse in [false, true] {
        let mut module = module(
            "fn inner(state: State) -> Pair {
            let old = state;
            let state = State { a: Pair { x: 30, y: 0 }, b: Pair { x: 0, y: 0 }, unused: 0 };
            return old.a;
        }
        fn outer(state: State) -> Pair {
            let old = state;
            let state = State { a: state.b, b: state.b, unused: 0 };
            return inner(old);
        }
        fn entry(state: State) -> Pair { return outer(state); }",
        );
        if reverse {
            module.functions.reverse();
        }
        assert!(project(&mut module, &["inner", "outer"]));
        assert_eq!(function(&module, "inner").params[0].ty.name, "Pair");
        let outer = function(&module, "outer");
        assert_eq!(outer.params.len(), 2);
        assert!(outer.params.iter().all(|p| p.ty.name == "Pair"));
        assert_eq!(outer.return_type.as_ref().unwrap().name, "Pair");
    }
}

#[test]
fn whole_old_records_and_computed_callers_keep_original_snapshots() {
    for (body, call) in [
        ("let old = state; let state = State { a: state.b, b: state.a, unused: 0 }; return old;", "helper(state)"),
        ("let old = state; let state = State { a: state.b, b: state.a, unused: 0 }; return old.a;", "helper(relay(state))"),
    ] {
        let returned = if body.ends_with("return old;") { "State" } else { "Pair" };
        let mut module = module(&format!("fn relay(state: State) -> State {{ return state; }}
            fn helper(state: State) -> {returned} {{ {body} }}
            fn entry(state: State) -> {returned} {{ return {call}; }}"));
        let before = module.clone();
        assert!(!project(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn control_boundary_writes_prevent_all_versions_of_that_name() {
    for control in [
        "if flag { let state = State { a: state.b, b: state.a, unused: 0 }; }",
        "while flag { let state = State { a: state.b, b: state.a, unused: 0 }; break; }",
    ] {
        let mut module = module(&format!(
            "fn helper(state: State, flag: bool) -> i64 {{
            let saved = state;
            let state = State {{ a: state.b, b: state.a, unused: 0 }};
            {control}
            return saved.a.x + state.b.y;
        }}"
        ));
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(&mut module.functions[0], &layouts);
        assert_eq!(module, before);
    }
}

#[test]
fn stable_versions_remain_visible_to_nested_reads_without_changing_loop_carries() {
    let mut module = module(
        "fn helper(state: State, flag: bool) -> i64 {
        let saved = state;
        let state = State { a: Pair { x: 30, y: 0 }, b: Pair { x: 0, y: 7 }, unused: 0 };
        let i = 0; let total = 0;
        while i < 2 { let total = total + state.b.y; let i = i + 1; }
        if flag { return saved.a.x + total; } else { return state.a.x; }
    } fn entry(state: State, flag: bool) -> i64 { return helper(state, flag); }",
    );
    assert!(project(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(
        helper
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["i64", "bool"]
    );
    let mut bindings = BTreeSet::new();
    branches::collect_bindings(&helper.body, &mut bindings);
    assert!(bindings.contains("i") && bindings.contains("total"));
}

#[test]
fn mixed_rebinding_preserves_scalar_kinds_and_constructor_work() {
    let mut module = crate::frontend::parse_nuis_module("mod cpu Main {
        struct State { flag: bool, tag: i32, value: i64, gain: f32, scale: f64, unused: i64 }
        fn helper(state: State, divisor: i64) -> State {
            let old = state;
            let state = State { flag: old.flag, tag: old.tag, value: old.value / divisor, gain: old.gain, scale: old.scale, unused: 3 };
            return state;
        }
        fn entry(state: State, divisor: i64) -> State { return helper(state, divisor); }
    }").unwrap();
    assert!(project(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(
        helper
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["bool", "f32", "f64", "i32", "i64", "i64"]
    );
    assert_eq!(helper.return_type.as_ref().unwrap().name, "State");
    let mut divisions = 0;
    walk::visit(&helper.body, |expr| {
        if matches!(
            expr,
            NirExpr::Binary {
                op: NirBinaryOp::Div,
                ..
            }
        ) {
            divisions += 1;
        }
        true
    });
    assert_eq!(divisions, 1);
}

#[test]
fn snapshots_reject_untyped_reference_and_mismatched_versions() {
    for case in 0..3 {
        let mut module = module(
            "fn helper(state: State) -> i64 {
            let state = State { a: state.b, b: state.a, unused: 0 }; return state.a.x;
        }",
        );
        let layouts = control_values::TypedLayouts::collect(&module);
        let helper = &mut module.functions[0];
        let NirStmt::Let { ty, .. } = &mut helper.body[0] else {
            panic!()
        };
        match case {
            0 => *ty = None,
            1 => {
                ty.as_mut().unwrap().is_ref = true;
                helper.params[0].ty.is_ref = true;
            }
            _ => ty.as_mut().unwrap().name = "Pair".into(),
        }
        let before = helper.clone();
        normalize(helper, &layouts);
        assert_eq!(*helper, before);
    }
}

#[test]
fn discarded_rebound_constructor_keeps_checked_work_in_reference_execution() {
    let (prefix, _) = include_str!("../../../tests/control_flow_syntax_native/alias_snapshots.ns")
        .split_once("    fn main()")
        .unwrap();
    for (divisor, selected) in [(0, false), (2, true), (0, true)] {
        let source = format!(
            "{prefix}
            fn main() -> i64 {{
                let state = State {{ left: 12, right: 7, unused: 0 }};
                print(checked_snapshot(state, {divisor}, {selected})); return 0;
            }}
        }}"
        );
        let yir = crate::pipeline::compile_source(&source).unwrap().yir;
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        );
        if divisor == 0 && selected {
            assert!(
                trace.is_err(),
                "unused rebinding must still evaluate its checked constructor"
            );
        } else {
            let prints = trace
                .unwrap()
                .events
                .into_iter()
                .filter(|e| e.contains("cpu.print"))
                .collect::<Vec<_>>();
            assert_eq!(prints.len(), 1, "{prints:?}");
            assert!(prints[0].ends_with(": 12"), "{prints:?}");
        }
    }
}

#[test]
fn record_rebinding_admission_preserves_type_and_scalar_boundaries() {
    let mut module = module(
        "fn helper(state: State, flag: bool) -> i64 {
        if flag {
            let saved = state;
            let state = State { a: state.b, b: state.a, unused: 0 };
            return saved.a.x + state.a.x;
        }
        return state.a.x;
    }
    fn scalar(value: i64) -> i64 { let value = value + 1; return value; }
    fn entry(state: State, flag: bool) -> i64 { return helper(state, flag); }",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let admitted = scalar_helpers::collect_typed_values(&module, &layouts, &BTreeMap::new());
    assert!(admitted.contains_key("helper") && admitted.contains_key("entry"));
    assert!(!admitted.contains_key("scalar"));
    let NirStmt::If { then_body, .. } = &mut module.functions[0].body[0] else {
        panic!()
    };
    let NirStmt::Let { ty, .. } = &mut then_body[1] else {
        panic!()
    };
    ty.as_mut().unwrap().name = "Pair".into();
    let rejected = scalar_helpers::collect_typed_values(&module, &layouts, &BTreeMap::new());
    assert!(!rejected.contains_key("helper") && !rejected.contains_key("entry"));
}
