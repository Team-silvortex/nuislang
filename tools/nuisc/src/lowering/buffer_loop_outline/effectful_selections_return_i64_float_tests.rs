use super::*;

macro_rules! float_child_cases {
    ($module:ident, $ty:literal, $zero:expr) => {
        mod $module {
            use super::*;
            const TY: &str = $ty;
            fn neutral() -> NirExpr { $zero }
            use crate::frontend::parse_nuis_module;

            fn source(body: &str) -> String {
                format!(
                    "mod cpu Main {{
                    fn decide(value: bool) -> bool {{ print(98); return value; }}
                    fn boolean(value: bool) -> bool {{ return value; }}
                    fn calculate(value: bool) -> FLOAT {{ return 0.0; }}
                    fn floating(value: FLOAT) -> FLOAT {{ print(95); return value; }}
                    fn relay(value: i64) -> i64 {{ print(96); return value; }}
                    fn observe(gate: bool, first: bool, saved: bool, seed: i64, float_seed: FLOAT, small_seed: i32) -> bool {{
                        print(99); {body} print(80); return false;
                    }} fn main() -> i64 {{ return 0; }}
                }}"
                ).replace("FLOAT", TY)
            }
            fn parsed(body: &str) -> NirModule {
                parse_nuis_module(&source(body)).unwrap()
            }
            fn rewrite(body: &str) -> (NirModule, BTreeSet<String>) {
                let mut module = parsed(body);
                let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
                let generated = outline(&mut module, &mut names);
                (module, generated)
            }
            fn reject(mut module: NirModule) {
                let before = module.clone();
                let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
                let before_names = names.clone();
                assert!(outline(&mut module, &mut names).is_empty());
                assert_eq!(module, before);
                assert_eq!(names, before_names);
            }
            fn reject_source(body: &str) {
                match parse_nuis_module(&source(body)) {
                    Ok(module) => reject(module),
                    Err(error) => assert!(error.contains("unknown value"), "{error}"),
                }
            }
            fn body(selected: &str) -> String {
                format!(
                    "if decide(gate) {{
                    let local = seed;
                    if first {{ let child = float_seed; {selected} }}
                    else {{ let local = relay(local); }}
                    return (local == seed) && decide(first);
                }} else {{ return saved; }}"
                )
            }

            #[test]
            fn return_i64_float_private_pair_accepts_orders_retention_nested_sequences_and_one_typed_target() {
                for selected in [
                    "if saved {let local = relay(local);let child = floating(child);} else {let child = floating(child);let local = relay(local);}",
                    "if saved {let child = floating(child);let local = relay(local);} else {let local = relay(local);}",
                    "if saved {let local = relay(local);let child = floating(child);if child == float_seed {let child = floating(child);let local = relay(local);} let local = relay(local);} else {let child = floating(child);let local = relay(local);} if child == float_seed {let local = relay(local);} let ignored = relay(local);",
                ] {
                    for inverted in [false, true] {
                        let text = body(selected);
                        let text = if inverted { text.replace("if decide(gate)", "if decide(gate == false)") } else { text };
                        let (module, generated) = rewrite(&text);
                        assert!(!generated.is_empty(), "{selected}");
                        crate::nir_verify::verify_nir_module(&module).unwrap();
                        assert!(module.functions.iter().filter(|f| generated.contains(&f.name))
                            .all(|f| f.params.len() <= 32 && matches!(f.body.first(), Some(NirStmt::If {..}))));
                        assert!(module.functions.iter().any(|f| generated.contains(&f.name)
                            && f.return_type == Some(scalar_type("i64"))));
                        let observe = module.functions.iter().find(|f| f.name == "observe").unwrap();
                        for statement in &observe.body {
                            if let NirStmt::Let {value: NirExpr::Call {callee,..},..} = statement {
                                let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
                                assert_eq!(helper.return_type, Some(scalar_type("bool")));
                                assert!(!helper.params.iter().any(|p| ["local","child","ignored"].contains(&p.name.as_str())));
                            }
                        }
                    }
                }
            }

            #[test]
            fn return_i64_float_private_pair_rejects_third_writes_seals_wide_owners_hidden_leaves_and_exits_atomically(
            ) {
                for selected in [
                    "if saved {let local = relay(local);let child = floating(child);let saved = first;}",
                    "let sibling = float_seed;if saved {let local = relay(local);let child = floating(child);let sibling = float_seed;}",
                    "const child: FLOAT = float_seed;if saved {let local = relay(local);let child = float_seed;}",
                    "if saved {let local = relay(local);const child: FLOAT = child;}",
                    "if saved {let local: bool = first;let child = floating(child);}",
                    "if saved {let local = relay(local);let child = calculate((child == float_seed) || first);}",
                    "if saved {let local = relay(local);let child = float_seed;print(94);}",
                    "if saved {let local = relay(local);return child == float_seed;} else {let child = float_seed;}",
                    "if saved {let local = relay(local);let child = float_seed;} let late = missing;",
                ] { reject_source(&body(selected)); }
                for text in [
                    "if gate {let local = seed;if first {let child: i32 = small_seed;if saved {let local = relay(local);let child: i32 = small_seed;}} return (local == seed) && decide(first);} else {return saved;}",
                    "if gate {let local = seed;if first {let child = float_seed;if saved {let local = relay(local);let child = float_seed;}} else {let local = child;} return (local == seed) && decide(first);} else {return saved;}",
                    "if gate {let local = seed;if first {let child = float_seed;if saved {let local = relay(local);let child = float_seed;} let seed = 0;} return (local == seed) && decide(first);} else {return saved;}",
                ] { reject_source(text); }
                for kind in ["ref", "optional", "generic", "u32", "String"] {
                    let mut ty = scalar_type("i64");
                    match kind {
                        "ref" => ty.is_ref = true,
                        "optional" => ty.is_optional = true,
                        "generic" => ty.generic_args.push(scalar_type("i64")),
                        other => ty = scalar_type(other),
                    }
                    assert!(!owner_type(&ty), "{kind}");
                }
            }

            #[test]
            fn return_i64_float_private_pair_owned_fields_neutral_guards_and_collision_free_projections() {
                let scope = Scope::from([
                    ("local".into(), scalar_type("i64")),
                    ("child".into(), scalar_type(TY)),
                ]);
                let stage = || regions::Staging {
                    local: scope.clone(),
                    defined: BTreeSet::new(),
                    inputs: BTreeSet::new(),
                    calls: BTreeSet::new(),
                    constants: BTreeSet::new(),
                    statements: vec![],
                };
                let (selection, reads, _) = transport::prepare(
                    predicates::Predicate::Scalar(NirExpr::Bool(true)),
                    stage(),
                    stage(),
                    &scope,
                    "local",
                    "child",
                )
                .unwrap();
                assert_eq!(reads, BTreeSet::from(["local".into(), "child".into()]));
                let mut names = BTreeSet::from(["__nuis_effect_pair_value_0".into()]);
                let mut bindings = BTreeSet::from(["__nuis_effect_call_pair_0".into()]);
                let mut helpers = vec![];
                let mut structs = vec![];
                let output = transport::install(
                    selection,
                    &mut names,
                    &mut bindings,
                    &mut helpers,
                    &mut structs,
                );
                assert_eq!((helpers.len(), structs.len()), (2, 1));
                let definition = &structs[0];
                assert_eq!(definition.name, "__nuis_effect_pair_value_1");
                assert_eq!(definition.visibility, NirVisibility::Private);
                assert_eq!(
                    definition
                        .fields
                        .iter()
                        .map(|f| (f.name.as_str(), f.ty.clone()))
                        .collect::<Vec<_>>(),
                    [("owner", scalar_type("i64")), ("child", scalar_type(TY))]
                );
                for helper in &helpers {
                    assert_eq!(helper.return_type, Some(scalar_type(&definition.name)));
                    let NirStmt::If { then_body, .. } = &helper.body[0] else {
                        unreachable!()
                    };
                    let [NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))] = then_body.as_slice()
                    else {
                        unreachable!()
                    };
                    assert_eq!(
                        fields,
                        &vec![
                            ("owner".into(), NirExpr::Int(0)),
                            ("child".into(), neutral())
                        ]
                    );
                    let NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. })) =
                        helper.body.last().unwrap()
                    else {
                        unreachable!()
                    };
                    assert_eq!(
                        fields,
                        &vec![
                            ("owner".into(), NirExpr::Var("local".into())),
                            ("child".into(), NirExpr::Var("child".into()))
                        ]
                    );
                }
                // Both actual typed projections read the same owned result, with no codec.
                for (statement, (name, field, ty)) in output[output.len() - 2..]
                    .iter()
                    .zip([("local", "owner", "i64"), ("child", "child", TY)])
                {
                    assert!(
                        matches!(statement, NirStmt::Let {name:n, ty:Some(t), value:NirExpr::FieldAccess{base, field:f}}
                        if n==name && t==&scalar_type(ty) && f==field &&
                        matches!(base.as_ref(),NirExpr::Var(n) if n=="__nuis_effect_call_pair_1"))
                    );
                }
                for child_ty in ["i32", "String"] {
                    let mut bad = scope.clone();
                    bad.insert("child".into(), scalar_type(child_ty));
                    assert!(transport::prepare(
                        predicates::Predicate::Scalar(NirExpr::Bool(true)),
                        stage(),
                        stage(),
                        &bad,
                        "local",
                        "child"
                    )
                    .is_none());
                }
                for field in ["local", "child"] {
                    for qualifier in ["ref", "optional", "generic"] {
                        let mut bad = scope.clone();
                        let ty = bad.get_mut(field).unwrap();
                        match qualifier {
                            "ref" => ty.is_ref = true,
                            "optional" => ty.is_optional = true,
                            "generic" => ty.generic_args.push(scalar_type("i64")),
                            _ => unreachable!(),
                        }
                        assert!(
                            transport::prepare(
                                predicates::Predicate::Scalar(NirExpr::Bool(true)),
                                stage(),
                                stage(),
                                &bad,
                                "local",
                                "child",
                            )
                            .is_none(),
                            "{field}/{qualifier}"
                        );
                    }
                }
            }

            #[test]
            fn return_i64_float_private_pair_shared_original_budget_and_capture_boundary() {
                let scope = Scope::from([
                    ("local".into(), scalar_type("i64")),
                    ("seed".into(), scalar_type("i64")),
                    ("float_seed".into(), scalar_type(TY)),
                    ("child".into(), scalar_type(TY)),
                    ("first".into(), scalar_type("bool")),
                    ("saved".into(), scalar_type("bool")),
                ]);
                let let_var = |name: &str, value: &str| NirStmt::Let {
                    name: name.into(),
                    ty: None,
                    value: NirExpr::Var(value.into()),
                };
                let yes = vec![let_var("local", "seed"), let_var("child", "float_seed")];
                let suffix = vec![
                    NirStmt::If {
                        condition: NirExpr::Var("saved".into()),
                        then_body: vec![let_var("child", "float_seed"), let_var("local", "seed")],
                        else_body: vec![],
                    },
                    let_var("local", "seed"),
                ];
                let stage = || regions::Staging {
                    local: scope.clone(),
                    defined: BTreeSet::from(["child".into()]),
                    inputs: BTreeSet::new(),
                    calls: BTreeSet::new(),
                    constants: BTreeSet::new(),
                    statements: vec![],
                };
                let mut entering = scope.clone();
                entering.remove("child");
                let owner = Target("local");
                let child = mixed::classify(&yes, &[], &entering, &stage(), &owner, 0)
                    .unwrap()
                    .unwrap();
                let default = || nested::Budget {
                    nodes: 64,
                    expressions: 256,
                    logical_edges: 32,
                };
                let proof = |yes: &[NirStmt], tail: &[NirStmt], budget: &mut nested::Budget| {
                    mixed::continue_flat(
                        &NirExpr::Var("first".into()),
                        yes,
                        &[],
                        tail,
                        &scope,
                        &owner,
                        &child,
                        &BTreeMap::new(),
                        budget,
                        0,
                    )
                };
                for (nodes, expressions, accepted) in [(7, 7, true), (6, 7, false), (7, 6, false)] {
                    let mut budget = nested::Budget {
                        nodes,
                        expressions,
                        logical_edges: 0,
                    };
                    let outcome = proof(&yes, &suffix, &mut budget);
                    assert_eq!(outcome.is_some(), accepted);
                    if accepted {
                        assert_eq!(
                            (budget.nodes, budget.expressions, budget.logical_edges),
                            (0, 0, 0)
                        );
                        assert_eq!(outcome.unwrap().ty, scalar_type("i64"));
                    }
                }
                for count in [16, 17] {
                    let mut arm = yes.clone();
                    arm.extend((2..count).map(|_| let_var("local", "seed")));
                    assert_eq!(proof(&arm, &[], &mut default()).is_some(), count == 16);
                }
                for count in [31, 32] {
                    let mut staged = stage();
                    staged.defined.clear();
                    for index in 0..count - 2 {
                        let name = format!("capture_{index}");
                        staged.local.insert(name.clone(), scalar_type("i64"));
                        staged.inputs.insert(name);
                    }
                    let result = transport::prepare(
                        predicates::Predicate::Scalar(NirExpr::Bool(true)),
                        staged,
                        stage(),
                        &scope_with_captures(&scope, count - 2),
                        "local",
                        "child",
                    );
                    assert_eq!(result.is_some(), count == 31);
                }
                for depth in [7, 8] {
                    assert_eq!(
                        mixed::continue_flat(
                            &NirExpr::Bool(true),
                            &yes,
                            &[],
                            &[],
                            &scope,
                            &owner,
                            &child,
                            &BTreeMap::new(),
                            &mut default(),
                            depth
                        )
                        .is_some(),
                        depth == 7
                    );
                }
            }

            fn scope_with_captures(scope: &Scope, count: usize) -> Scope {
                let mut scope = scope.clone();
                for index in 0..count {
                    scope.insert(format!("capture_{index}"), scalar_type("i64"));
                }
                scope
            }

        }
    };
}

float_child_cases!(f32_child, "f32", NirExpr::F32("0.0".into()));
float_child_cases!(f64_child, "f64", NirExpr::F64("0.0".into()));
