use super::tests::{let_var, reject, rewrite, scope, source, stage};
use super::*;

#[test]
fn return_retained_private_fields_accepts_child_only_initial_arms_and_shared_owner_suffix() {
    for ty in ["f32", "f64"] {
        for selected in [
            "if saved {let a=small(a);let b=floating(b);}else{let b=floating(b);let a=small(a);}let local=relay(local);",
            "if saved {let a=small(a);}else{let b=floating(b);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);if a==int_seed {let b=floating(b);let a=small(a);}}else{}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);}else{}if a==int_seed {let b=floating(b);let local=relay(local);}let ignored=relay(local);",
        ] {
            let (module, names) = rewrite(&source(ty, selected));
            assert!(!names.is_empty(), "{selected}");
            crate::nir_verify::verify_nir_module(&module).unwrap();
            assert!(module.structs.iter().any(|s| s.name.starts_with("__nuis_effect_fields_value") && s.fields.len()==3));
            let original = module.functions.iter().find(|f| f.name=="observe").unwrap();
            assert!(original.body.iter().any(|s| matches!(s, NirStmt::Let{value:NirExpr::Call{callee,..},..} if names.contains(callee))));
        }
    }
}

#[test]
fn return_retained_private_fields_requires_existing_target_and_two_lexical_mutable_children() {
    for ty in ["f32", "f64"] {
        let s = scope(ty);
        let mut entering = s.clone();
        entering.remove("a");
        entering.remove("b");
        let owner = Target("local");
        let yes = [let_var("a", "int_seed"), let_var("b", "float_seed")];
        // A delegated target need not be freshly written inside the child stage.
        assert!(!stage(&s).defined.contains("local"));
        assert!(classify(&yes, &[], &entering, &stage(&s), &owner, 0)
            .unwrap()
            .is_some());
        for name in ["a", "b"] {
            let mut incoming = entering.clone();
            incoming.insert(name.into(), s[name].clone());
            assert!(classify(&yes, &[], &incoming, &stage(&s), &owner, 0).is_none());
            let mut undefined = stage(&s);
            undefined.defined.remove(name);
            assert!(classify(&yes, &[], &entering, &undefined, &owner, 0).is_none());
        }
        for name in ["local", "a", "b"] {
            let mut sealed = stage(&s);
            sealed.constants.insert(name.into());
            assert!(classify(&yes, &[], &entering, &sealed, &owner, 0).is_none());
            for qualifier in ["ref", "optional", "generic", "bool", "String", "u32"] {
                let mut bad = s.clone();
                let t = bad.get_mut(name).unwrap();
                match qualifier {
                    "ref" => t.is_ref = true,
                    "optional" => t.is_optional = true,
                    "generic" => t.generic_args.push(scalar_type("i32")),
                    other => *t = scalar_type(other),
                }
                if qualifier == "bool" && name != "local" {
                    assert!(classify(&yes, &[], &entering, &stage(&bad), &owner, 0)
                        .unwrap()
                        .is_some());
                    continue;
                }
                assert!(
                    classify(&yes, &[], &entering, &stage(&bad), &owner, 0).is_none(),
                    "{name}/{qualifier}"
                );
            }
        }
        let mut absent = s.clone();
        absent.remove("local");
        assert!(classify(&yes, &[], &entering, &stage(&absent), &owner, 0).is_none());
        let pair = [let_var("local", "seed"), let_var("a", "int_seed")];
        assert!(classify(&pair, &[], &entering, &stage(&s), &owner, 0)
            .unwrap()
            .is_none());
        assert!(classify(&yes[..1], &[], &entering, &stage(&s), &owner, 0)
            .unwrap()
            .is_none());
        let mut extra = stage(&s);
        extra.local.insert("extra".into(), scalar_type("i64"));
        extra.defined.insert("extra".into());
        extra.constants.insert("extra".into());
        let three = [yes[0].clone(), yes[1].clone(), let_var("extra", "seed")];
        assert!(classify(&three, &[], &entering, &extra, &owner, 0).is_none());
        let selected = "if saved {let a=small(a);let b=floating(b);}let local=relay(local);";
        let no_authority = source(ty, selected)
            .replace("let local=seed;", "")
            .replace("local", "seed");
        reject(&no_authority);
        // A root child-only body is not itself evidence of an outward owner.
        reject(&source(
            ty,
            "if saved {let a=small(a);let b=floating(b);}else{}",
        ));
    }
}

#[test]
fn return_retained_private_fields_rejects_seals_leaks_and_other_entering_writes_atomically() {
    for ty in ["f32", "f64"] {
        for selected in [
            "if saved {let a=small(a);let b=floating(b);let seed=local;}let local=relay(local);",
            "const local:i64=local;if saved {let a=small(a);let b=floating(b);}let local=relay(local);",
            "const a:i32=int_seed;if saved {let a=small(a);let b=floating(b);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);const b:FLOAT=b;}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);let secret=seed;}let local=secret;",
            "if saved {let a=small(a);let b=floating(b);print(80);}let local=relay(local);",
            "if saved {let a=small(a);let b=floating(b);}let seed=local;let local=relay(local);",
        ] {
            let text=source(ty,&selected.replace("FLOAT",ty));
            match crate::frontend::parse_nuis_module(&text) {
                Ok(_)=>reject(&text), Err(e)=>assert!(e.contains("unknown value"),"{e}"),
            }
        }
    }
}

#[test]
fn return_retained_private_fields_counts_original_work_and_captures_retention() {
    for ty in ["f32", "f64"] {
        let s = scope(ty);
        let owner = Target("local");
        let yes = [let_var("a", "int_seed"), let_var("b", "float_seed")];
        let children = classify(&yes, &[], &Scope::new(), &stage(&s), &owner, 0)
            .unwrap()
            .unwrap();
        let suffix = [let_var("local", "seed")];
        for (nodes, expressions, accepted) in [(4, 4, true), (3, 4, false), (4, 3, false)] {
            let mut budget = nested::Budget {
                nodes,
                expressions,
                logical_edges: 0,
            };
            let proof = prove(
                &NirExpr::Bool(true),
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
            assert_eq!(proof.is_some(), accepted);
            if let Some(proof) = proof {
                assert_eq!(proof.name, "local");
                assert_eq!(proof.ty, scalar_type("i64"));
                assert!(proof.inputs.contains("local"));
                assert_eq!(
                    (budget.nodes, budget.expressions, budget.logical_edges),
                    (0, 0, 0)
                );
            }
        }
        for count in [16, 17] {
            let arm = (0..count).map(|i| yes[i % 2].clone()).collect::<Vec<_>>();
            let mut budget = nested::Budget {
                nodes: 64,
                expressions: 256,
                logical_edges: 32,
            };
            assert_eq!(
                prove(
                    &NirExpr::Bool(true),
                    &arm,
                    &[],
                    &suffix,
                    &s,
                    &owner,
                    &children,
                    &BTreeMap::new(),
                    &mut budget,
                    0
                )
                .is_some(),
                count == 16
            );
        }
        for depth in [7, 8] {
            let mut budget = nested::Budget {
                nodes: 64,
                expressions: 256,
                logical_edges: 32,
            };
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
                    &mut budget,
                    depth
                )
                .is_some(),
                depth == 7
            );
        }
    }
}
