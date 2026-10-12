use super::super::tests::{reject_module, source};
use super::*;
use crate::frontend::parse_nuis_module;

fn parsed(body: &str) -> NirModule {
    parse_nuis_module(&source(body).replace(
        "fn observe(",
        "fn relay(value: i32) -> i32 { print(96); return value; } fn observe(",
    ))
    .unwrap()
}

fn rewrite(body: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parsed(body);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}

#[test]
fn return_i32_pair_mixed_logical_effectful_scalar_selections_keep_both_orders_nested_sequences_retention_and_one_bool_export(
) {
    for selection in [
        "if saved {let local = child == i32_from_i64(7);let child = relay(child);} else {let child = relay(child);let local = child == i32_from_i64(7);}",
        "if saved {if first {let child = relay(child);let local = child == i32_from_i64(7);} let child = relay(child);if local {let child = relay(child);let local = first;} let local = child == i32_from_i64(7);} else {let local = first;let child = relay(child);}",
        "if saved {let local = first;let child = relay(child);} else {let local = first;} if local {let child = relay(child);} else {let local = child == i32_from_i64(7);} let local = local && decide(child == i32_from_i64(7));",
    ] {
        for inverted in [false, true] {
            let selected = format!("let local = saved;if first {{let child = i32_from_i64(token(7));{selection}}} return local && decide(first);");
            let (yes,no) = if inverted {("return saved;",selected.as_str())} else {(selected.as_str(),"return saved;")};
            let (module,generated)=rewrite(&format!("if decide(gate) {{{yes}}} else {{{no}}}"));
            assert!(!generated.is_empty(),"{selection}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let observe=module.functions.iter().find(|f|f.name=="observe").unwrap();
            for statement in &observe.body {
                if let NirStmt::Let {value:NirExpr::Call{callee,..},..}=statement {
                    let f=module.functions.iter().find(|f|&f.name==callee).unwrap();
                    assert_eq!(f.return_type,Some(scalar_type("bool")));
                    assert!(!f.params.iter().any(|p|["local","child"].contains(&p.name.as_str())));
                }
            }
            assert!(module.functions.iter().filter(|f|generated.contains(&f.name))
                .all(|f| f.params.len()<=32 && matches!(f.body.first(),Some(NirStmt::If {..}))));
        }
    }
}

fn eval(expr: &NirExpr, values: &BTreeMap<String, i64>) -> i64 {
    let recurse = |v: &NirExpr| eval(v, values);
    match expr {
        NirExpr::Int(v) => *v,
        NirExpr::Bool(v) => i64::from(*v),
        NirExpr::Var(n) => values[n],
        NirExpr::CastBoolToI64(v) => {
            let v = recurse(v);
            assert!((0..=1).contains(&v));
            v
        }
        NirExpr::CastI32ToI64(v) | NirExpr::CastI64ToI32(v) => {
            let v = recurse(v);
            i64::from(i32::try_from(v).unwrap())
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let (a, b) = (i128::from(recurse(lhs)), i128::from(recurse(rhs)));
            i64::try_from(match op {
                NirBinaryOp::Add => a + b,
                NirBinaryOp::Sub => a - b,
                NirBinaryOp::Mul => a * b,
                NirBinaryOp::Ge => i128::from(a >= b),
                NirBinaryOp::Eq => i128::from(a == b),
                _ => panic!("carrier contains trapping or unexpected work: {op:?}"),
            })
            .unwrap()
        }
        _ => panic!("unexpected carrier expression: {expr:?}"),
    }
}

fn carrier(
    scope: &Scope,
    bindings: &mut BTreeSet<String>,
) -> Option<(Vec<NirStmt>, Vec<NirFunction>)> {
    let stage = || regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::new(),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: vec![],
    };
    let (p, inputs, _) = transport::prepare(
        predicates::Predicate::Scalar(NirExpr::Bool(true)),
        stage(),
        stage(),
        scope,
        "local",
        "child",
    )?;
    assert!(inputs.contains("local") && inputs.contains("child"));
    let mut helpers = Vec::new();
    let output = transport::install(
        p,
        &mut BTreeSet::new(),
        bindings,
        &mut helpers,
        &mut Vec::new(),
    );
    Some((output, helpers))
}

#[test]
fn return_i32_pair_mixed_logical_effectful_scalar_selections_roundtrip_full_signed_extremes_both_flags_without_traps_or_name_collision(
) {
    let scope = Scope::from([
        ("local".into(), scalar_type("bool")),
        ("child".into(), scalar_type("i32")),
    ]);
    let mut bindings = BTreeSet::from([
        "local".into(),
        "child".into(),
        "__nuis_effect_call_pair_0".into(),
    ]);
    let (output, helpers) = carrier(&scope, &mut bindings).unwrap();
    assert_eq!(helpers.len(), 2);
    let NirStmt::Return(Some(pack)) = helpers[0].body.last().unwrap() else {
        unreachable!()
    };
    let decode = &output[output.len() - 2..];
    for flag in [false, true] {
        for value in [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX] {
            let mut values = BTreeMap::from([
                ("local".into(), i64::from(flag)),
                ("child".into(), i64::from(value)),
            ]);
            let word = eval(pack, &values);
            assert_eq!(
                word,
                i64::from(value) + 2_147_483_648 + i64::from(flag) * 4_294_967_296
            );
            assert!((0..=8_589_934_591).contains(&word));
            let name = bindings
                .iter()
                .find(|n| {
                    n.starts_with("__nuis_effect_call_pair")
                        && n.as_str() != "__nuis_effect_call_pair_0"
                })
                .unwrap();
            values.insert(name.clone(), word);
            for stmt in decode {
                let NirStmt::Let { name, value, .. } = stmt else {
                    unreachable!()
                };
                values.insert(name.clone(), eval(value, &values));
            }
            assert_eq!(
                (values["local"], values["child"]),
                (i64::from(flag), i64::from(value))
            );
        }
    }
    assert!(
        matches!(&decode[1],NirStmt::Let {ty:Some(ty),value:NirExpr::CastI64ToI32(_),..} if ty==&scalar_type("i32"))
    );
}

#[test]
fn return_i32_pair_mixed_logical_effectful_scalar_selections_reject_wide_children_third_writes_seals_hidden_calls_and_exits_atomically(
) {
    for body in [
        "let child = i32_from_i64(7);if saved {let child = relay(child);}",
        "let child = token(7);let sibling = token(8);if saved {let local = first;let child = token(8);let sibling = token(9);}",
        "let child = i32_from_i64(7);if saved {let local = first;let child = relay(child);let saved = first;}",
        "let child = i32_from_i64(7);let sibling = i32_from_i64(8);if saved {let local = first;let child = relay(child);let sibling = relay(sibling);}",
        "let child = i32_from_i64(7);if saved {let local = first;const child: bool = first || false;}",
        "let child = i32_from_i64(7);if saved {let local = first;let child = relay(child);return local;}",
        "let child = i32_from_i64(7);if saved {let local = first;let child = relay(child);print(96);}",
        "let child = i32_from_i64(7);if saved {let local = boolean(first || saved);let child = relay(child);}",
    ] {
        let text=format!("if decide(gate) {{let local = saved;if first {{{body}}} return local && first;}} else {{return saved;}}");
        reject_module(parsed(&text));
    }
    for kind in ["buffer", "void", "ref", "optional", "generic"] {
        let mut ty = scalar_type("i32");
        match kind {
            "ref" => ty.is_ref = true,
            "optional" => ty.is_optional = true,
            "generic" => ty.generic_args.push(scalar_type("i32")),
            _ => ty = scalar_type(kind),
        }
        let scope = Scope::from([("local".into(), scalar_type("bool")), ("child".into(), ty)]);
        assert!(carrier(&scope, &mut BTreeSet::new()).is_none(), "{kind}");
    }
}

fn statements(text: &str) -> Vec<NirStmt> {
    let module = parsed(&format!(
        "let local = false;let child = i32_from_i64(7);{text}"
    ));
    let observe = module
        .functions
        .iter()
        .find(|f| f.name == "observe")
        .unwrap();
    observe.body[3..observe.body.len() - 2].to_vec()
}

fn proof(yes: &str, no: &str, tail: &str, budget: &mut nested::Budget) -> Option<nested::Proof> {
    super::continue_flat(
        &NirExpr::Bool(true),
        &statements(yes),
        &statements(no),
        &statements(tail),
        &Scope::from([
            ("local".into(), scalar_type("bool")),
            ("child".into(), scalar_type("i32")),
        ]),
        &Target("local"),
        &Child("child".into()),
        &Signatures::from([(
            "relay".into(),
            (vec![scalar_type("i32")], scalar_type("i32")),
        )]),
        budget,
        0,
    )
}

#[test]
fn return_i32_pair_mixed_logical_effectful_scalar_selections_charge_original_casts_nodes_and_edges_once(
) {
    for (nodes, expressions, accepted) in [(6, 14, true), (5, 14, false), (6, 13, false)] {
        let mut budget = nested::Budget {
            nodes,
            expressions,
            logical_edges: 0,
        };
        let p = proof(
            "let local = child == i32_from_i64(7);let child = relay(child);",
            "",
            "let local = child == i32_from_i64(7);let child = relay(child);let local = local;",
            &mut budget,
        );
        assert_eq!(p.is_some(), accepted);
        if accepted {
            assert_eq!(
                (budget.nodes, budget.expressions, budget.logical_edges),
                (0, 0, 0)
            );
        }
    }
}

#[test]
fn return_i32_pair_mixed_logical_effectful_scalar_selections_keep_lexical_and_capture_bounds_with_retained_versions(
) {
    let budget = || nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    };
    assert!(proof("if local {let local = false;let child = relay(child);} let local = child == i32_from_i64(7);","",
        "if local {let child = relay(child);} let local = child == i32_from_i64(7);",&mut budget()).is_some());
    let mut yes=statements("if local {let secret = child;let local = false;let child = relay(child);} let local = child == i32_from_i64(7);");
    let NirStmt::Let {
        value: NirExpr::Binary { lhs, .. },
        ..
    } = yes.last_mut().unwrap()
    else {
        unreachable!()
    };
    *lhs = Box::new(NirExpr::Var("secret".into()));
    assert!(super::continue_flat(
        &NirExpr::Bool(true),
        &yes,
        &[],
        &statements("let local = child == i32_from_i64(7);"),
        &Scope::from([
            ("local".into(), scalar_type("bool")),
            ("child".into(), scalar_type("i32"))
        ]),
        &Target("local"),
        &Child("child".into()),
        &Signatures::from([(
            "relay".into(),
            (vec![scalar_type("i32")], scalar_type("i32"))
        )]),
        &mut budget(),
        0
    )
    .is_none());
    for count in [31, 32] {
        let mut scope = Scope::from([
            ("local".into(), scalar_type("bool")),
            ("child".into(), scalar_type("i32")),
        ]);
        let inputs = (0..count - 2)
            .map(|i| format!("p{i}"))
            .collect::<BTreeSet<_>>();
        scope.extend(inputs.iter().map(|n| (n.clone(), scalar_type("i64"))));
        let stage = || regions::Staging {
            local: scope.clone(),
            defined: BTreeSet::new(),
            inputs: inputs.clone(),
            calls: BTreeSet::new(),
            constants: BTreeSet::new(),
            statements: vec![],
        };
        assert_eq!(
            transport::prepare(
                predicates::Predicate::Scalar(NirExpr::Bool(true)),
                stage(),
                stage(),
                &scope,
                "local",
                "child"
            )
            .is_some(),
            count == 31
        );
    }
}

#[test]
fn return_i32_pair_mixed_logical_effectful_scalar_selections_install_two_guarded_helpers_per_original_selection_without_tail_copy(
) {
    for count in [1, 4, 8] {
        let yes = "let local = child == i32_from_i64(7);let child = relay(child);";
        let tail = "if local {let local = false;let child = relay(child);} ".repeat(count)
            + "let local = child == i32_from_i64(7);";
        let p = proof(
            yes,
            "",
            &tail,
            &mut nested::Budget {
                nodes: 64,
                expressions: 256,
                logical_edges: 32,
            },
        )
        .unwrap();
        let SelectedValue::Region(region, _) = p.value else {
            unreachable!()
        };
        let mut helpers = Vec::new();
        regions::install(
            region,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut helpers,
            &mut Vec::new(),
        );
        assert_eq!(helpers.len(), 2 * (count + 1));
        assert!(helpers
            .iter()
            .all(|f| f.return_type == Some(scalar_type("i64"))
                && matches!(f.body.first(), Some(NirStmt::If { .. }))));
    }
}
