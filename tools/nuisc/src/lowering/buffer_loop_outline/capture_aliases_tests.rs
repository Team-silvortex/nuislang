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
fn immutable_alias_chains_expose_only_used_nested_fields() {
    let mut module = module(
        "fn helper(state: State) -> i64 {
            let saved = state; const second: State = saved;
            let part: Pair = second.a; let leaf = part.x;
            return leaf + saved.b.y;
        }
        fn entry(state: State) -> i64 { return helper(state); }",
    );
    assert!(project(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(helper.params.len(), 2);
    assert!(helper.params.iter().all(|p| p.ty.name == "i64"));
    assert_eq!(helper.body.len(), 2);
    assert!(matches!(&helper.body[0], NirStmt::Let { name, .. } if name == "leaf"));
    assert_eq!(function(&module, "entry").params[0].ty.name, "State");
}

#[test]
fn alias_demand_propagates_across_private_call_chains_in_both_orders() {
    for reverse in [false, true] {
        let mut module = module(
            "fn inner(state: State) -> i64 {
                let saved = state; return saved.a.x;
            }
            fn outer(state: State) -> i64 {
                let snapshot = state; return inner(snapshot) + snapshot.b.y;
            }
            fn entry(state: State) -> i64 { return outer(state); }",
        );
        if reverse {
            module.functions.reverse();
        }
        assert!(project(&mut module, &["inner", "outer"]));
        assert_eq!(function(&module, "inner").params.len(), 1);
        assert_eq!(function(&module, "outer").params.len(), 2);
        assert!(!project(&mut module, &["inner", "outer"]));
    }
}

#[test]
fn branch_local_aliases_keep_independent_lexical_origins() {
    let mut module = module(
        "fn helper(state: State, flag: bool) -> i64 {
            if flag { let left = state.a; return left.x; }
            else { let right = state.b; return right.y; }
        }
        fn entry(state: State, flag: bool) -> i64 { return helper(state, flag); }",
    );
    assert!(project(&mut module, &["helper"]));
    assert_eq!(function(&module, "helper").params.len(), 3);
    let helper = function(&module, "helper");
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &helper.body[0]
    else {
        panic!()
    };
    assert_eq!(then_body.len(), 1);
    assert_eq!(else_body.len(), 1);
    assert_ne!(then_body, else_body);
}

#[test]
fn alias_subrecord_returns_preserve_nominal_type_and_full_prefix_demand() {
    let mut module = module(
        "fn helper(state: State) -> Pair {
            let saved = state; let part = saved.a; return part;
        }
        fn entry(state: State) -> Pair { return helper(state); }",
    );
    assert!(project(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(helper.params.len(), 1);
    assert_eq!(helper.params[0].ty.name, "Pair");
    assert_eq!(helper.return_type.as_ref().unwrap().name, "Pair");
    assert_eq!(helper.body.len(), 1);
}

#[test]
fn whole_alias_uses_and_caller_veto_leave_the_original_body_unchanged() {
    for (body, call) in [
        ("let saved = state; return consume(saved);", "helper(state)"),
        (
            "let saved = state; return saved.a.x;",
            "helper(relay(state))",
        ),
    ] {
        let mut module = module(&format!(
            "fn consume(state: State) -> i64 {{ return state.a.x; }}
            fn relay(state: State) -> State {{ return state; }}
            fn helper(state: State) -> i64 {{ {body} }}
            fn entry(state: State) -> i64 {{ return {call}; }}"
        ));
        let before = function(&module, "helper").body.clone();
        assert!(!project(&mut module, &["helper"]));
        assert_eq!(function(&module, "helper").body, before);
    }
}

#[test]
fn control_flow_rebindings_keep_snapshot_captures_whole() {
    for body in [
        "let saved = state; if flag { let state = State { a: state.b, b: state.a, unused: 0 }; } return saved.a.x;",
        "let saved = state; while flag { let state = State { a: state.b, b: state.a, unused: 0 }; break; } return saved.a.x;",
    ] {
        let mut module = module(&format!("fn helper(state: State, flag: bool) -> i64 {{ {body} }}
            fn entry(state: State, flag: bool) -> i64 {{ return helper(state, flag); }}"));
        let before = function(&module, "helper").body.clone();
        assert!(!project(&mut module, &["helper"]));
        assert_eq!(function(&module, "helper").body, before);
    }
}

#[test]
fn repeated_branch_alias_names_keep_exact_subrecords() {
    let mut module = module(
        "fn helper(state: State, flag: bool) -> i64 {
            if flag { let saved = state.a; return saved.x; }
            else { let saved = state.b; return saved.y; }
        }
        fn entry(state: State, flag: bool) -> i64 { return helper(state, flag); }",
    );
    assert!(project(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(
        helper
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["Pair", "Pair", "bool"]
    );
    let NirStmt::If {
        then_body,
        else_body,
        ..
    } = &helper.body[0]
    else {
        panic!()
    };
    for body in [then_body, else_body] {
        assert_eq!(body.len(), 2);
        assert!(matches!(&body[0], NirStmt::Let { name, .. } if name == "saved"));
    }
}

#[test]
fn iteration_local_aliases_stay_whole_but_invariant_outer_alias_reads_project() {
    for inside in [false, true] {
        let alias = "let saved = state;";
        let mut module = module(&format!(
            "fn helper(state: State) -> i64 {{
                {} let i = 0; let total = 0;
                while i < 2 {{ {} let total = total + saved.a.x; let i = i + 1; }}
                return total;
            }}
            fn entry(state: State) -> i64 {{ return helper(state); }}",
            if inside { "" } else { alias },
            if inside { alias } else { "" }
        ));
        assert_eq!(project(&mut module, &["helper"]), !inside);
        assert_eq!(
            function(&module, "helper").params[0].ty.name,
            if inside { "State" } else { "i64" }
        );
    }
}

#[test]
fn alias_normalization_does_not_hide_reference_or_annotation_mismatches() {
    for reference in [false, true] {
        let mut module = module(
            "fn helper(state: State) -> i64 { let saved: State = state; return saved.a.x; }",
        );
        let layouts = control_values::TypedLayouts::collect(&module);
        let helper = &mut module.functions[0];
        if reference {
            helper.params[0].ty.is_ref = true;
        } else {
            let NirStmt::Let { ty, .. } = &mut helper.body[0] else {
                panic!()
            };
            ty.as_mut().unwrap().name = "Pair".into();
        }
        let before = helper.body.clone();
        normalize(helper, &layouts);
        assert_eq!(helper.body, before);
    }
}

#[test]
fn mixed_alias_leaves_preserve_exact_scalar_kinds() {
    let mut module = crate::frontend::parse_nuis_module("mod cpu Main {
        struct Payload { flag: bool, tag: i32, value: i64, gain: f32, scale: f64 }
        struct Wide { payload: Payload, unused: i64 }
        fn helper(wide: Wide) -> Payload {
            let saved = wide; let payload = saved.payload;
            return Payload { flag: payload.flag, tag: payload.tag, value: payload.value, gain: payload.gain, scale: payload.scale };
        }
        fn entry(wide: Wide) -> Payload { return helper(wide); }
    }").unwrap();
    assert!(project(&mut module, &["helper"]));
    let helper = function(&module, "helper");
    assert_eq!(helper.body.len(), 1);
    assert_eq!(helper.return_type.as_ref().unwrap().name, "Payload");
    assert_eq!(
        helper
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["bool", "f32", "f64", "i32", "i64"]
    );
    assert_eq!(function(&module, "entry").params[0].ty.name, "Wide");
}

#[test]
fn rebound_origin_still_executes_the_old_alias_snapshot() {
    let source = include_str!("../../../tests/control_flow_syntax_native/alias_snapshots.ns");
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
    assert_eq!(prints.len(), 8, "{prints:?}");
    for (line, value) in prints.iter().zip([42, 12, 13, 12, 42, 12, 42, 12]) {
        assert!(line.ends_with(&format!(": {value}")), "{prints:?}");
    }
    yir_lower_llvm::emit_module(&yir).unwrap();
}
