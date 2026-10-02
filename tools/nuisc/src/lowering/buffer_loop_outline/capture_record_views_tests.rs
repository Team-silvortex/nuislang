use super::*;

const COPY: &str = "State { left: Leaf { value: state.left.value,
    gain: state.left.gain, tag: state.left.tag, enabled: state.left.enabled },
    right: other.right, count: state.count }";

fn module(body: &str, callers: &str) -> NirModule {
    crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: f64, gain: f32, tag: i32, enabled: bool }}
        struct State {{ left: Leaf, right: Leaf, count: i64 }}
        fn relay(state: State) -> State {{ return state; }}
        fn consume(state: State) -> i64 {{ return state.count; }}
        fn helper(state: State, other: State) -> i64 {{ {body} }}
        {callers}
    }}"
    ))
    .unwrap()
}

fn helper(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "helper")
        .unwrap()
}

fn normalize_and_prune(module: &mut NirModule) {
    let layouts = control_values::TypedLayouts::collect(module);
    normalize(helper(module), &layouts);
    dead_records::normalize(helper(module), &layouts);
    crate::nir_verify::verify_nir_module(module).unwrap();
}

#[test]
fn record_views_project_leaf_subrecord_and_alias_chains_without_erasing_checked_work() {
    for alias in [false, true] {
        let copied = if alias {
            "const saved: State = snapshot;"
        } else {
            ""
        };
        let name = if alias { "saved" } else { "snapshot" };
        let mut module = module(
            &format!(
                "let snapshot: State = {COPY}; {copied}
            let checked = 10 / {name}.count;
            let part = {name}.left; let tag = part.tag;
            return {name}.count;"
            ),
            "",
        );
        normalize_and_prune(&mut module);
        let body = &helper(&mut module).body;
        assert_eq!(body.len(), 3);
        let NirStmt::Let {
            name,
            value: NirExpr::Binary { rhs, .. },
            ..
        } = &body[0]
        else {
            panic!()
        };
        assert_eq!(name, "checked");
        assert_eq!(access(rhs), Some(vec!["state".into(), "count".into()]));
        let NirStmt::Let { name, value, .. } = &body[1] else {
            panic!()
        };
        assert_eq!(name, "tag");
        assert_eq!(
            access(value),
            Some(vec!["state".into(), "left".into(), "tag".into()])
        );
        let before = module.clone();
        normalize_and_prune(&mut module);
        assert_eq!(module, before);
    }
}

#[test]
fn record_views_do_not_project_computed_checked_or_codec_constructors() {
    for value in [
        "relay(state)".to_owned(),
        COPY.replace("state.count", "state.count / (state.count - state.count)"),
        COPY.replace("state.count", "state.count + other.count"),
        COPY.replace("state.left.tag", "i32_from_i64(state.count)"),
    ] {
        let mut module = module(
            &format!("let snapshot = {value}; let observed = snapshot.right.tag; return 0;"),
            "",
        );
        let before = module.clone();
        normalize_and_prune(&mut module);
        assert_eq!(module, before, "{value}");
    }
}

#[test]
fn record_views_keep_whole_escapes_and_mutated_or_shadowed_sources() {
    for body in [
        format!("let snapshot = {COPY}; return consume(snapshot);"),
        format!("let snapshot = {COPY}; let snapshot = other; return snapshot.count;"),
        format!("let snapshot = {COPY}; let state = other; return snapshot.count;"),
        format!("if state.count > 0 {{ let other = state; }} let snapshot = {COPY}; return snapshot.count;"),
        format!("if state.count > 0 {{ let snapshot = {COPY}; return snapshot.count; }} else {{ let snapshot = other; return snapshot.count; }}"),
        "let local = relay(state); let snapshot = State { left: local.left, right: local.right, count: local.count }; return snapshot.count;".into(),
    ] {
        let mut module = module(&body, "");
        let before = module.clone();
        normalize_and_prune(&mut module);
        assert_eq!(module, before, "{body}");
    }
}

#[test]
fn record_views_keep_lexical_branches_and_zero_trip_snapshot_versions() {
    let mut module = module(&format!(
        "let outer = {COPY}; let i = 0;
        while i < state.count {{
            let i = i + 1;
            if i == 1 {{ let inner = State {{ left: other.left, right: state.right, count: other.count }};
                let checked = 10 / inner.count; continue; }}
            let checked_outer = 10 / outer.count; break;
        }} return outer.count;"
    ), "");
    normalize_and_prune(&mut module);
    let mut names = BTreeSet::new();
    branches::collect_bindings(&helper(&mut module).body, &mut names);
    assert!(!names.contains("outer") && !names.contains("inner"));
    assert!(names.contains("checked") && names.contains("checked_outer"));
    let mut divisors = BTreeSet::new();
    walk::visit(&helper(&mut module).body, |expr| {
        if let NirExpr::Binary { rhs, .. } = expr {
            if let Some(path) = access(rhs) {
                divisors.insert(path);
            }
        }
        true
    });
    assert!(divisors.contains(&vec!["state".into(), "count".into()]));
    assert!(divisors.contains(&vec!["other".into(), "count".into()]));
}

#[test]
fn record_views_require_exact_nominal_fields_binding_types_and_value_inputs() {
    for mutation in 0..10 {
        let mut module = module(
            &format!("let snapshot: State = {COPY}; return snapshot.count;"),
            "",
        );
        let NirStmt::Let {
            ty,
            value:
                NirExpr::StructLiteral {
                    type_name,
                    type_args,
                    fields,
                },
            ..
        } = &mut helper(&mut module).body[0]
        else {
            panic!()
        };
        match mutation {
            0 => *type_name = "Leaf".into(),
            1 => fields[2].1 = fields[1].1.clone(),
            2 => fields[2].0 = "left".into(),
            3 => fields[2].0 = "missing".into(),
            4 => *ty = Some(scalar_type("Leaf")),
            5 => type_args.push(scalar_type("i64")),
            6 => helper(&mut module).params[0].ty.is_ref = true,
            7 => helper(&mut module).params[0].ty.is_optional = true,
            8 => helper(&mut module).params[0].ty.name = "Buffer".into(),
            9 => {
                fields.pop();
            }
            _ => unreachable!(),
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        normalize(helper(&mut module), &layouts);
        assert_eq!(module, before, "{mutation}");
    }
}

#[test]
fn record_views_keep_all_callers_transactional() {
    for computed in [false, true] {
        let call = if computed { "relay(state)" } else { "state" };
        let mut module = module(
            &format!(
                "let snapshot = {COPY}; let checked = 10 / snapshot.count; return snapshot.count;"
            ),
            &format!(
                "fn first(state: State) -> i64 {{ return helper(state, state); }}
             fn second(state: State) -> i64 {{ return helper({call}, state); }}"
            ),
        );
        let layouts = control_values::TypedLayouts::collect(&module);
        let before = module.clone();
        let result = project(
            &mut module,
            &BTreeSet::from(["helper".into()]),
            &BTreeSet::new(),
            &layouts,
            &BTreeMap::new(),
        );
        assert_eq!(result.changed, !computed);
        if computed {
            assert_eq!(module, before);
        } else {
            assert_eq!(helper(&mut module).params.len(), 1);
            assert_eq!(helper(&mut module).params[0].ty.name, "i64");
            assert_eq!(helper(&mut module).body.len(), 2);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn record_views_project_mixed_leaf_types_instead_of_word_codecs() {
    let mut module = module(
        &format!(
            "let snapshot = {COPY};
         let flag = snapshot.left.enabled; let gain = snapshot.left.gain;
         let tag = snapshot.left.tag; let value = snapshot.left.value;
         return snapshot.count;"
        ),
        "fn entry(state: State) -> i64 { return helper(state, state); }",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    assert!(
        project(
            &mut module,
            &BTreeSet::from(["helper".into()]),
            &BTreeSet::new(),
            &layouts,
            &BTreeMap::new()
        )
        .changed
    );
    assert_eq!(
        helper(&mut module)
            .params
            .iter()
            .map(|p| p.ty.name.as_str())
            .collect::<Vec<_>>(),
        ["i64", "bool", "f32", "i32", "f64"]
    );
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn record_views_preserve_reference_results_and_selected_failures() {
    for count in [0, 2] {
        for divisor in [0, 2] {
            let mut module = crate::frontend::parse_nuis_module(&format!(
                "mod cpu Main {{
                struct State {{ count: i64, divisor: i64 }}
                @noinline fn helper(state: State, other: State) -> i64 {{
                    let snapshot = State {{ count: state.count, divisor: other.divisor }};
                    let saved = snapshot;
                    if state.count > 0 {{
                        let checked = 10 / saved.divisor;
                        return saved.count;
                    }} return snapshot.count;
                }}
                fn main() -> i64 {{
                    return helper(State {{ count: {count}, divisor: 7 }},
                        State {{ count: 99, divisor: {divisor} }});
                }} }}"
            ))
            .unwrap();
            let execute = |module: &NirModule| {
                let mut yir = crate::lowering::lower_nir_to_yir_builtin_cpu(module).unwrap();
                yir.nodes.reverse();
                yir.functions.reverse();
                for function in &mut yir.functions {
                    function.body_nodes.reverse();
                }
                let trace = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                )
                .map_err(|_| ())?;
                let main = yir
                    .functions
                    .iter()
                    .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                    .unwrap();
                Ok(trace.values[&main.result.as_ref().unwrap().node].clone())
            };
            let before = execute(&module);
            normalize_and_prune(&mut module);
            assert_eq!(execute(&module), before);
            assert_eq!(
                before,
                if count > 0 && divisor == 0 {
                    Err(())
                } else {
                    Ok(yir_core::Value::Int(count))
                }
            );
        }
    }
}

#[test]
fn record_views_bound_work_and_depth_without_partial_rewrites() {
    let mut module = module(
        &format!("let snapshot = {COPY}; return snapshot.count;"),
        "",
    );
    let layouts = control_values::TypedLayouts::collect(&module);
    let expected = normalized(helper(&mut module), &layouts, 65_536).unwrap();
    assert_ne!(expected, helper(&mut module).body);
    for budget in 0..512 {
        if let Some(actual) = normalized(helper(&mut module), &layouts, budget) {
            assert_eq!(actual, expected);
        }
    }
    let NirStmt::Return(Some(value)) = helper(&mut module).body.last_mut().unwrap() else {
        panic!()
    };
    for _ in 0..64 {
        *value = NirExpr::FieldAccess {
            base: Box::new(value.clone()),
            field: "left".into(),
        };
    }
    let before = module.clone();
    normalize(helper(&mut module), &layouts);
    assert_eq!(module, before);
    for _ in 0..64 {
        let body = std::mem::take(&mut helper(&mut module).body);
        helper(&mut module).body = vec![NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: body,
            else_body: Vec::new(),
        }];
    }
    let before = module.clone();
    normalize(helper(&mut module), &layouts);
    assert_eq!(module, before);
}
