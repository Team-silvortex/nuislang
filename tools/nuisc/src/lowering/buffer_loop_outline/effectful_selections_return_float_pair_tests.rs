use super::*;

macro_rules! float_cases {
    ($module:ident, $kind:literal) => {
        mod $module {
            use super::*;
            const KIND: &str = $kind;
            use super::super::super::tests::{reject_module, source};
            use crate::frontend::parse_nuis_module;

            fn parsed(body: &str) -> NirModule {
                parse_nuis_module(&source(body).replace("v: i64) -> i64", &format!("v: {KIND}) -> {KIND}")).replace(
                    "fn observe(",
                    &format!("fn relay(value: {KIND}) -> {KIND} {{ print(96); return value; }} fn observe("),
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
            fn return_float_pair_mixed_logical_effectful_scalar_selections_keep_both_orders_nested_sequences_retention_and_one_bool_export(
            ) {
                for selection in [
                    "if saved {let local = child == 7.0;let child = relay(child);} else {let child = relay(child);let local = child == 7.0;}",
                    "if saved {if first {let child = relay(child);let local = child == 7.0;} let child = relay(child);if local {let child = relay(child);let local = first;} let local = child == 7.0;} else {let local = first;let child = relay(child);}",
                    "if saved {let local = first;let child = relay(child);} else {let local = first;} if local {let child = relay(child);} else {let local = child == 7.0;} let local = local && decide(child == 7.0);",
                ] {
                    for inverted in [false, true] {
                        let selected = format!("let local = saved;if first {{let child = token(7.0);{selection}}} return local && decide(first);");
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
                        assert!(module.structs.iter().any(|d|d.fields.iter().map(|f|f.ty.name.as_str()).collect::<Vec<_>>()==["bool",KIND]));
                    }
                }
            }

            fn carrier(
                scope: &Scope,
                names: &mut BTreeSet<String>,
                bindings: &mut BTreeSet<String>,
            ) -> Option<(Vec<NirStmt>, Vec<NirFunction>, Vec<NirStructDef>)> {
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
                let mut structs = Vec::new();
                let output = transport::install(p, names, bindings, &mut helpers, &mut structs);
                Some((output, helpers, structs))
            }

            #[test]
            fn return_float_pair_mixed_logical_effectful_scalar_selections_keep_full_width_fields_literal_zero_guards_and_collision_free_installation(
            ) {
                let scope = Scope::from([
                    ("local".into(), scalar_type("bool")),
                    ("child".into(), scalar_type(KIND)),
                ]);
                let mut names = BTreeSet::from([
                    "__nuis_effect_pair_value_0".into(),
                    "__nuis_effect_pair_value_1".into(),
                ]);
                let mut bindings = BTreeSet::from(["__nuis_effect_call_pair_0".into()]);
                let (output, helpers, structs) = carrier(&scope, &mut names, &mut bindings).unwrap();
                assert_eq!((helpers.len(), structs.len()), (2, 1));
                let definition = &structs[0];
                assert_eq!(definition.name, "__nuis_effect_pair_value_2");
                assert_eq!(definition.visibility, NirVisibility::Private);
                assert_eq!(
                    definition
                        .fields
                        .iter()
                        .map(|f| (f.name.as_str(), f.ty.name.as_str()))
                        .collect::<Vec<_>>(),
                    [("owner", "bool"), ("child", KIND)]
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
                            ("owner".into(), NirExpr::Bool(false)),
                            ("child".into(), if KIND == "f32" { NirExpr::F32("0.0".into()) } else { NirExpr::F64("0.0".into()) })
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
                // Both projections read one selected result. There is no arithmetic codec,
                // narrowing conversion, duplicated call or second outward result.
                for (stmt, (name, field, ty)) in output[output.len() - 2..]
                    .iter()
                    .zip([("local", "owner", "bool"), ("child", "child", KIND)])
                {
                    assert!(
                        matches!(stmt,NirStmt::Let {name:n,ty:Some(t),value:NirExpr::FieldAccess {base,field:f}} if n==name && t==&scalar_type(ty) && f==field && matches!(base.as_ref(),NirExpr::Var(n) if n=="__nuis_effect_call_pair_1"))
                    );
                }
                assert_eq!(
                    output
                        .iter()
                        .filter(|s| matches!(
                            s,
                            NirStmt::Let {
                                value: NirExpr::Call { .. },
                                ..
                            }
                        ))
                        .count(),
                    2
                );
                let mut module=parsed("if decide(gate) {let local = saved;if first {let child = token(7.0);if saved {let local = first;let child = relay(child);}} return local && first;} else {return saved;}");
                let mut existing = definition.clone();
                existing.name = "__nuis_effect_pair_value_0".into();
                module.structs.push(existing.clone());
                // The public outliner entry must reserve source types even if its caller
                // supplied only function names, as the direct NIR harness does.
                let mut function_names = module.functions.iter().map(|f| f.name.clone()).collect();
                let generated = outline(&mut module, &mut function_names);
                assert!(!generated.is_empty());
                assert_eq!(module.structs[0], existing);
                assert!(module
                    .structs
                    .iter()
                    .skip(1)
                    .all(|d| d.name != "__nuis_effect_pair_value_0"));
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let mut rejected = parsed("if decide(gate) {let local = saved;if first {let child = token(7.0);let sibling = token(8.0);if saved {let local = first;let child = relay(child);let sibling = relay(sibling);}} return local && first;} else {return saved;}");
                rejected.structs.push(existing);
                reject_module(rejected);
            }

            #[test]
            fn return_float_pair_mixed_logical_effectful_scalar_selections_reject_incoming_third_writes_seals_hidden_calls_and_exits_atomically(
            ) {
                for body in [
                    "let child = token(7.0);if saved {let child = relay(child);}",
                    "let child = token(7.0);if saved {let local = first;let child = relay(child);let saved = first;}",
                    "let child = token(7.0);let sibling = token(8.0);if saved {let local = first;let child = relay(child);let sibling = relay(sibling);}",
                    "let child = token(7.0);if saved {let local = first;const child: bool = first || false;}",
                    "let child = token(7.0);if saved {let local = first;let child = relay(child);return local;}",
                    "let child = token(7.0);if saved {let local = first;let child = relay(child);print(96);}",
                    "let child = token(7.0);if saved {let local = boolean(first || saved);let child = relay(child);}",
                ] {
                    reject_module(parsed(&format!("if decide(gate) {{let local = saved;if first {{{body}}} return local && first;}} else {{return saved;}}")));
                }
                for kind in ["buffer", "void", "ref", "optional", "generic"] {
                    let mut ty = scalar_type(KIND);
                    match kind {
                        "ref" => ty.is_ref = true,
                        "optional" => ty.is_optional = true,
                        "generic" => ty.generic_args.push(scalar_type(KIND)),
                        _ => ty = scalar_type(kind),
                    };
                    assert!(
                        carrier(
                            &Scope::from([("local".into(), scalar_type("bool")), ("child".into(), ty)]),
                            &mut BTreeSet::new(),
                            &mut BTreeSet::new()
                        )
                        .is_none(),
                        "{kind}"
                    );
                }
                let scope = Scope::from([
                    ("local".into(), scalar_type("bool")),
                    ("child".into(), scalar_type(KIND)),
                ]);
                let staged = regions::Staging {
                    local: scope.clone(),
                    defined: BTreeSet::from(["local".into(), "child".into()]),
                    inputs: BTreeSet::new(),
                    calls: BTreeSet::new(),
                    constants: BTreeSet::new(),
                    statements: vec![],
                };
                assert!(classify(
                    &statements("let local = child == 7.0;let child = relay(child);"),
                    &[],
                    &scope,
                    &staged,
                    &Target("local"),
                    0
                )
                .is_none());
            }

            fn statements(text: &str) -> Vec<NirStmt> {
                let module = parsed(&format!("let local = false;let child = token(7.0);{text}"));
                let observe = module
                    .functions
                    .iter()
                    .find(|f| f.name == "observe")
                    .unwrap();
                observe.body[3..observe.body.len() - 2].to_vec()
            }

            fn budget() -> nested::Budget {
                nested::Budget {
                    nodes: 64,
                    expressions: 256,
                    logical_edges: 32,
                }
            }

            fn proof(
                yes: &[NirStmt],
                no: &[NirStmt],
                tail: &[NirStmt],
                budget: &mut nested::Budget,
            ) -> Option<nested::Proof> {
                continue_flat(
                    &NirExpr::Bool(true),
                    yes,
                    no,
                    tail,
                    &Scope::from([
                        ("local".into(), scalar_type("bool")),
                        ("child".into(), scalar_type(KIND)),
                    ]),
                    &Target("local"),
                    &Child("child".into()),
                    &Signatures::from([(
                        "relay".into(),
                        (vec![scalar_type(KIND)], scalar_type(KIND)),
                    )]),
                    budget,
                    0,
                )
            }

            #[test]
            fn return_float_pair_mixed_logical_effectful_scalar_selections_charge_original_nodes_expressions_and_edges_once(
            ) {
                for (nodes, expressions, accepted) in [(6, 12, true), (5, 12, false), (6, 11, false)] {
                    let mut b = nested::Budget {
                        nodes,
                        expressions,
                        logical_edges: 0,
                    };
                    assert_eq!(
                        proof(
                            &statements("let local = child == 7.0;let child = relay(child);"),
                            &[],
                            &statements("let local = child == 7.0;let child = relay(child);let local = local;"),
                            &mut b
                        )
                        .is_some(),
                        accepted
                    );
                    if accepted {
                        assert_eq!((b.nodes, b.expressions, b.logical_edges), (0, 0, 0));
                    }
                }
            }

            #[test]
            fn return_float_pair_mixed_logical_effectful_scalar_selections_keep_lexical_and_capture_limits_with_retained_versions(
            ) {
                let mut yes=statements("if local {let secret = child;let local = false;let child = relay(child);} let local = child == 7.0;");
                let NirStmt::Let {
                    value: NirExpr::Binary { lhs, .. },
                    ..
                } = yes.last_mut().unwrap()
                else {
                    unreachable!()
                };
                *lhs = Box::new(NirExpr::Var("secret".into()));
                assert!(proof(
                    &yes,
                    &[],
                    &statements("let local = child == 7.0;"),
                    &mut budget()
                )
                .is_none());
                for count in [31, 32] {
                    let mut scope = Scope::from([
                        ("local".into(), scalar_type("bool")),
                        ("child".into(), scalar_type(KIND)),
                    ]);
                    let inputs = (0..count - 2)
                        .map(|i| format!("p{i}"))
                        .collect::<BTreeSet<_>>();
                    scope.extend(inputs.iter().map(|n| (n.clone(), scalar_type(KIND))));
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
            fn return_float_pair_mixed_logical_effectful_scalar_selections_install_linear_helpers_and_private_layouts_without_tail_copy(
            ) {
                for count in [1, 4, 8] {
                    let tail = "if local {let local = false;let child = relay(child);} ".repeat(count)
                        + "let local = child == 7.0;";
                    let p = proof(
                        &statements("let local = child == 7.0;let child = relay(child);"),
                        &[],
                        &statements(&tail),
                        &mut budget(),
                    )
                    .unwrap();
                    let SelectedValue::Region(region, _) = p.value else {
                        unreachable!()
                    };
                    let mut helpers = Vec::new();
                    let mut structs = Vec::new();
                    regions::install(
                        region,
                        &mut BTreeSet::new(),
                        &mut BTreeSet::new(),
                        &mut helpers,
                        &mut structs,
                    );
                    assert_eq!((helpers.len(), structs.len()), (2 * (count + 1), count + 1));
                    assert!(helpers.iter().all(|f| structs
                        .iter()
                        .any(|d| f.return_type == Some(scalar_type(&d.name)))
                        && matches!(f.body.first(), Some(NirStmt::If { .. }))));
                }
            }

        }
    };
}

float_cases!(f32_cases, "f32");
float_cases!(f64_cases, "f64");
