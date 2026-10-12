use super::*;
use crate::frontend::parse_nuis_module;

pub(super) fn source(ty: &str, selected: &str) -> String {
    format!("mod cpu Main {{
        fn decide(v: bool) -> bool {{print(98);return v;}}
        fn relay(v: i64) -> i64 {{print(96);return v;}}
        fn small(v: i32) -> i32 {{print(94);return v;}}
        fn floating(v: {ty}) -> {ty} {{print(93);return v;}}
        fn observe(gate: bool, first: bool, saved: bool, seed: i64, int_seed: i32, float_seed: {ty}) -> bool {{
            print(99);if decide(gate) {{let local=seed;if first {{let a=int_seed;let b=float_seed;{selected}}}
                return (local==seed)&&decide(saved);}} else {{return saved;}}
            print(80);return false;
        }} fn main() -> i64 {{return 0;}}
    }}")
}
pub(super) fn rewrite(text: &str) -> (NirModule, BTreeSet<String>) {
    let mut module = parse_nuis_module(text).unwrap();
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let generated = outline(&mut module, &mut names);
    (module, generated)
}
pub(super) fn reject(text: &str) {
    let mut module = parse_nuis_module(text).unwrap();
    let before = module.clone();
    let mut names: BTreeSet<_> = module.functions.iter().map(|f| f.name.clone()).collect();
    let before_names = names.clone();
    assert!(outline(&mut module, &mut names).is_empty());
    assert_eq!(module, before);
    assert_eq!(names, before_names);
}
pub(super) fn scope(ty: &str) -> Scope {
    Scope::from([
        ("local".into(), scalar_type("i64")),
        ("seed".into(), scalar_type("i64")),
        ("a".into(), scalar_type("i32")),
        ("int_seed".into(), scalar_type("i32")),
        ("b".into(), scalar_type(ty)),
        ("float_seed".into(), scalar_type(ty)),
        ("first".into(), scalar_type("bool")),
        ("saved".into(), scalar_type("bool")),
    ])
}
pub(super) fn stage(scope: &Scope) -> regions::Staging {
    regions::Staging {
        local: scope.clone(),
        defined: BTreeSet::from(["a".into(), "b".into()]),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: vec![],
    }
}
fn default_budget() -> nested::Budget {
    nested::Budget {
        nodes: 64,
        expressions: 256,
        logical_edges: 32,
    }
}
pub(super) fn let_var(name: &str, value: &str) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: None,
        value: NirExpr::Var(value.into()),
    }
}

#[test]
fn return_private_fields_accepts_orders_retention_nested_sequences_and_one_outward_target() {
    for ty in ["f32", "f64"] {
        for selected in [
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);}else{let b=floating(b);let a=small(a);let local=relay(local);}",
            "if saved {let a=small(a);let local=relay(local);let b=floating(b);}else{} if a==int_seed {let b=floating(b);let a=small(a);} let local=relay(local);",
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);if b==float_seed {let b=floating(b);let a=small(a);let local=relay(local);}let local=relay(local);}else{let b=floating(b);let local=relay(local);let a=small(a);}if a==int_seed {let local=relay(local);}let ignored=relay(local);",
        ] {
            let (module,names)=rewrite(&source(ty,selected));assert!(!names.is_empty(),"{selected}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.structs.iter().any(|s|s.name.starts_with("__nuis_effect_fields_value")&&s.fields.len()==3));
            let observe=module.functions.iter().find(|f|f.name=="observe").unwrap();
            for statement in &observe.body {
                if let NirStmt::Let{value:NirExpr::Call{callee,..},..}=statement {
                    let helper=module.functions.iter().find(|f|&f.name==callee).unwrap();
                    assert_eq!(helper.return_type,Some(scalar_type("bool")));
                    assert!(!helper.params.iter().any(|p|["local","a","b","ignored"].contains(&p.name.as_str())));
                }
            }
            assert!(module.functions.iter().filter(|f|names.contains(&f.name)).all(|f|f.params.len()<=32));
        }
    }
}

#[test]
fn return_private_fields_rejects_entering_sealed_qualified_extra_writes_hidden_work_and_leaks_atomically(
) {
    for ty in ["f32", "f64"] {
        for selected in [
            "if saved {let local=relay(local);let a=small(a);let seed=local;}",
            "const extra:i64=seed;if saved {let local=relay(local);let a=small(a);let b=floating(b);let extra=seed;}",
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);let saved=first;}",
            "const a:i32=int_seed;if saved {let local=relay(local);let a=small(a);let b=floating(b);}",
            "if saved {let local=relay(local);let a=small(a);const b:FLOAT=b;}",
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);print(94);}",
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);return saved;}",
            "if saved {let local=relay(local);let a=small(a);let b=floating(b);let secret=seed;}let local=secret;",
        ] {
            let text=source(ty,&selected.replace("FLOAT",ty));
            match parse_nuis_module(&text) {
                Ok(_)=>reject(&text),Err(error)=>assert!(error.contains("unknown value"),"{error}"),
            }
        }
        let s = scope(ty);
        let mut entering = s.clone();
        entering.remove("a");
        entering.remove("b");
        let yes = vec![
            let_var("local", "seed"),
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
        ];
        let owner = Target("local");
        assert!(classify(&yes, &[], &entering, &stage(&s), &owner, 0)
            .unwrap()
            .is_some());
        for name in ["a", "b"] {
            let mut entering = entering.clone();
            entering.insert(name.into(), s[name].clone());
            assert!(classify(&yes, &[], &entering, &stage(&s), &owner, 0).is_none());
            let mut sealed = stage(&s);
            sealed.constants.insert(name.into());
            assert!(classify(&yes, &[], &Scope::new(), &sealed, &owner, 0).is_none());
            let mut undefined = stage(&s);
            undefined.defined.remove(name);
            assert!(classify(&yes, &[], &Scope::new(), &undefined, &owner, 0).is_none());
        }
        for field in ["local", "a", "b"] {
            for kind in ["ref", "optional", "generic", "u32", "String", "bool"] {
                let mut bad = s.clone();
                let t = bad.get_mut(field).unwrap();
                match kind {
                    "ref" => t.is_ref = true,
                    "optional" => t.is_optional = true,
                    "generic" => t.generic_args.push(scalar_type("i32")),
                    other => *t = scalar_type(other),
                }
                if kind == "bool" && field != "local" {
                    assert!(classify(&yes, &[], &Scope::new(), &stage(&bad), &owner, 0)
                        .unwrap()
                        .is_some());
                    continue;
                }
                assert!(
                    classify(&yes, &[], &Scope::new(), &stage(&bad), &owner, 0).is_none(),
                    "{field}/{kind}"
                );
            }
        }
    }
}

#[test]
fn return_private_fields_generic_record_has_independent_neutrals_projections_and_no_pair_codec() {
    for ty in ["f32", "f64"] {
        let s = scope(ty);
        let children = vec!["a".into(), "b".into()];
        let mut a = stage(&s);
        a.defined.clear();
        let (selection, reads, _) = transport::prepare_fields(
            predicates::Predicate::Scalar(NirExpr::Bool(true)),
            a,
            stage(&s),
            &s,
            "local",
            &children,
        )
        .unwrap();
        assert_eq!(
            reads,
            BTreeSet::from(["local".into(), "a".into(), "b".into()])
        );
        let mut names = BTreeSet::from(["__nuis_effect_fields_value_0".into()]);
        let mut bindings = BTreeSet::from(["__nuis_effect_call_fields_0".into()]);
        let mut helpers = vec![];
        let mut structs = vec![];
        let output = transport::install_fields(
            selection,
            &mut names,
            &mut bindings,
            &mut helpers,
            &mut structs,
        );
        assert_eq!((helpers.len(), structs.len()), (2, 1));
        let def = &structs[0];
        assert_eq!(def.name, "__nuis_effect_fields_value_1");
        assert_eq!(def.visibility, NirVisibility::Private);
        assert_eq!(
            def.fields
                .iter()
                .map(|f| (f.name.as_str(), f.ty.clone()))
                .collect::<Vec<_>>(),
            [
                ("owner", scalar_type("i64")),
                ("child_0", scalar_type("i32")),
                ("child_1", scalar_type(ty))
            ]
        );
        for helper in &helpers {
            assert_eq!(helper.return_type, Some(scalar_type(&def.name)));
            let NirStmt::If { then_body, .. } = &helper.body[0] else {
                unreachable!()
            };
            let [NirStmt::Return(Some(NirExpr::StructLiteral { fields, .. }))] =
                then_body.as_slice()
            else {
                unreachable!()
            };
            assert_eq!(
                fields,
                &vec![
                    ("owner".into(), NirExpr::Int(0)),
                    (
                        "child_0".into(),
                        NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0)))
                    ),
                    (
                        "child_1".into(),
                        if ty == "f32" {
                            NirExpr::F32("0.0".into())
                        } else {
                            NirExpr::F64("0.0".into())
                        }
                    )
                ]
            );
        }
        for (stmt, (name, label)) in output[output.len() - 3..].iter().zip([
            ("local", "owner"),
            ("a", "child_0"),
            ("b", "child_1"),
        ]) {
            assert!(
                matches!(stmt,NirStmt::Let{name:n,ty:Some(t),value:NirExpr::FieldAccess{base,field}}
                if n==name&&t==&s[name]&&field==label&&matches!(base.as_ref(),NirExpr::Var(v)if v=="__nuis_effect_call_fields_1"))
            );
        }
        for children in [
            vec![],
            vec!["a".into()],
            vec!["a".into(), "a".into()],
            vec!["local".into(), "b".into()],
            vec!["a".into(), "b".into(), "local".into()],
        ] {
            assert!(transport::prepare_fields(
                predicates::Predicate::Scalar(NirExpr::Bool(true)),
                stage(&s),
                stage(&s),
                &s,
                "local",
                &children
            )
            .is_none());
        }
    }
    // Representation is type-driven, not a whitelist of tested backend/type triples.
    for owner in ["i32", "i64", "f32", "f64"] {
        for a in ["i32", "i64", "f32", "f64"] {
            for b in ["i32", "i64", "f32", "f64"] {
                let s = Scope::from([
                    ("local".into(), scalar_type(owner)),
                    ("a".into(), scalar_type(a)),
                    ("b".into(), scalar_type(b)),
                ]);
                assert!(transport::prepare_fields(
                    predicates::Predicate::Scalar(NirExpr::Bool(true)),
                    stage(&s),
                    stage(&s),
                    &s,
                    "local",
                    &["a".into(), "b".into()]
                )
                .is_some());
            }
        }
    }
}

#[test]
fn return_private_fields_original_forest_capture_statement_and_depth_bounds() {
    for ty in ["f32", "f64"] {
        let s = scope(ty);
        let mut entering = s.clone();
        entering.remove("a");
        entering.remove("b");
        let owner = Target("local");
        let yes = vec![
            let_var("local", "seed"),
            let_var("a", "int_seed"),
            let_var("b", "float_seed"),
        ];
        let children = classify(&yes, &[], &entering, &stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        let suffix = vec![
            NirStmt::If {
                condition: NirExpr::Var("saved".into()),
                then_body: vec![let_var("a", "int_seed"), let_var("b", "float_seed")],
                else_body: vec![],
            },
            let_var("local", "seed"),
        ];
        for (nodes, expressions, accepted) in [(8, 8, true), (7, 8, false), (8, 7, false)] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: 0,
            };
            let outcome = prove(
                &NirExpr::Var("first".into()),
                &yes,
                &[],
                &suffix,
                &s,
                &owner,
                &children,
                &BTreeMap::new(),
                &mut budget,
                0,
            );
            assert_eq!(outcome.is_some(), accepted);
            if accepted {
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
        for count in [64, 65] {
            let mut arm = yes.clone();
            arm.extend((3..16).map(|_| let_var("local", "seed")));
            let forest = vec![
                NirStmt::If {
                    condition: NirExpr::Var("saved".into()),
                    then_body: arm.clone(),
                    else_body: arm,
                },
                let_var("local", "seed"),
            ];
            let no = (0..16)
                .map(|_| let_var("local", "seed"))
                .collect::<Vec<_>>();
            let tail = vec![
                NirStmt::If {
                    condition: NirExpr::Var("saved".into()),
                    then_body: (0..count - 53).map(|_| let_var("a", "int_seed")).collect(),
                    else_body: vec![],
                },
                let_var("local", "seed"),
            ];
            assert_eq!(
                prove(
                    &NirExpr::Var("first".into()),
                    &forest,
                    &no,
                    &tail,
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut default_budget(),
                    0
                )
                .is_some(),
                count == 64
            );
        }
        for count in [16, 17] {
            let mut arm = yes.clone();
            arm.extend((3..count).map(|_| let_var("local", "seed")));
            assert_eq!(
                prove(
                    &NirExpr::Bool(true),
                    &arm,
                    &[],
                    &[],
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut default_budget(),
                    0
                )
                .is_some(),
                count == 16
            );
        }
        for count in [31, 32] {
            let mut s = s.clone();
            let mut a = stage(&s);
            a.defined.clear();
            for i in 0..count - 3 {
                let n = format!("capture_{i}");
                s.insert(n.clone(), scalar_type("i64"));
                a.inputs.insert(n);
            }
            assert_eq!(
                transport::prepare_fields(
                    predicates::Predicate::Scalar(NirExpr::Bool(true)),
                    a,
                    stage(&s),
                    &s,
                    "local",
                    &["a".into(), "b".into()]
                )
                .is_some(),
                count == 31
            );
        }
        for depth in [7, 8] {
            assert_eq!(
                prove(
                    &NirExpr::Bool(true),
                    &yes,
                    &[],
                    &[],
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut default_budget(),
                    depth
                )
                .is_some(),
                depth == 7
            );
        }
    }
}
