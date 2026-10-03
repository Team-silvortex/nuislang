use super::*;
use crate::frontend::parse_nuis_module;

fn masks(prefix: &str, effects: &str) -> BTreeMap<String, Vec<bool>> {
    let module = parse_nuis_module(&format!(
        "mod cpu Main {{
            struct State {{ value: i64, narrow: i32, flag: bool, small: f32, wide: f64 }}
            @noinline fn relay(value: i64) -> i64 {{ return value; }}
            fn work(seed: State, branch: bool) -> State {{
                {prefix} while branch {{ {effects} }} return seed;
            }}
        }}"
    ))
    .unwrap();
    let work = module.functions.iter().find(|f| f.name == "work").unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let scope = work
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect();
    let mut budget = Budget(MAX_WORK);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut budget).unwrap();
    let mut clock = 0;
    let prefix = &work.body[..work.body.len() - 2];
    bind_prefix(
        prefix,
        &mut snapshots,
        &layouts,
        &catalog,
        &mut budget,
        &mut clock,
    );
    let NirStmt::While { body, .. } = &work.body[work.body.len() - 2] else {
        panic!("expected fixture loop");
    };
    let scope = snapshots
        .env
        .iter()
        .map(|(name, v)| (name.clone(), v.ty.clone()))
        .collect();
    analyze_at(
        body,
        &scope,
        &layouts,
        &catalog,
        &mut budget,
        Some(&snapshots),
    )
    .unwrap()
    .into_iter()
    .map(|(name, (_, mask))| (name, mask))
    .collect()
}

fn bind_prefix(
    body: &[NirStmt],
    snapshots: &mut Snapshots,
    layouts: &control_values::CarryLayouts,
    catalog: &ScalarHelpers,
    budget: &mut Budget,
    clock: &mut usize,
) {
    for stmt in body {
        if let NirStmt::If {
            then_body,
            else_body,
            ..
        } = stmt
        {
            let mut left = snapshots.clone();
            let mut right = snapshots.clone();
            bind_prefix(then_body, &mut left, layouts, catalog, budget, clock);
            bind_prefix(else_body, &mut right, layouts, catalog, budget, clock);
            snapshots.join(&left, &right, budget, clock).unwrap();
        } else {
            snapshots
                .bind(stmt, layouts, catalog, budget, clock, 0)
                .unwrap();
        }
    }
}

const ENTRY: &str = "let carry = State {
    value: 7, narrow: i32_from_i64(4294967303), flag: true, small: 1.5, wide: 2.5
};";
const REPEAT: &str = "let carry = State {
    value: 7, narrow: i32_from_i64(7), flag: true, small: 1.5, wide: 2.5
};";

#[test]
fn return_invariant_literals_keep_exact_typed_values_across_nested_writes_and_joins() {
    for prefix in [
        ENTRY.to_owned(),
        format!("{ENTRY} if branch {{ {REPEAT} }} else {{ {REPEAT} }}"),
    ] {
        for effects in [
            REPEAT.to_owned(),
            format!("if branch {{ {REPEAT} }} else {{ {REPEAT} }}"),
            format!("while branch {{ {REPEAT} break; }} {REPEAT}"),
        ] {
            assert_eq!(
                masks(&prefix, &effects)["carry"],
                [true; 5],
                "{prefix} {effects}"
            );
        }
    }
}

#[test]
fn return_invariant_literals_check_intermediate_changed_values_and_opaque_work() {
    for changed in [
        REPEAT.replace("value: 7", "value: 8"),
        REPEAT.replace("value: 7", "value: 7 + 0"),
        REPEAT.replace("value: 7", "value: relay(7)"),
    ] {
        for effects in [
            changed.clone(),
            format!("{changed} {REPEAT}"),
            format!("if branch {{ {changed} }} else {{ {REPEAT} }}"),
            format!("while branch {{ {changed} break; }} {REPEAT}"),
        ] {
            assert_eq!(
                masks(ENTRY, &effects)["carry"],
                [false, true, true, true, true],
                "{effects}"
            );
        }
    }
    for (old, new, index) in [
        ("i32_from_i64(7)", "i32_from_i64(8)", 1),
        ("flag: true", "flag: false", 2),
        ("small: 1.5", "small: 1.25", 3),
        ("wide: 2.5", "wide: 2.25", 4),
    ] {
        let mut expected = [true; 5];
        expected[index] = false;
        assert_eq!(masks(ENTRY, &REPEAT.replace(old, new))["carry"], expected);
    }
}

fn value(expr: &NirExpr, budget: &mut Budget, depth: usize) -> Option<Rc<Value>> {
    let (module, _, _) = tests::fixture("");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    Proof::new(&layouts, &catalog, budget).expression(expr, &Env::new(), depth)
}

fn known(expr: NirExpr) -> Rc<Value> {
    let value = value(&expr, &mut Budget(MAX_WORK), 0).unwrap();
    assert!(value.words[0].is_some());
    value
}

#[test]
fn return_invariant_literal_storage_identity_keeps_types_rounding_and_signed_zero() {
    for expr in [
        NirExpr::Int(1),
        NirExpr::Bool(true),
        NirExpr::F32("1.0".into()),
        NirExpr::F64("1.0".into()),
    ] {
        let left = known(expr.clone());
        let right = known(expr);
        assert!(left == right);
    }
    assert!(known(NirExpr::Int(1)).words != known(NirExpr::Bool(true)).words);
    assert!(known(NirExpr::F32("0.0".into())).words != known(NirExpr::F64("0.0".into())).words);
    assert!(known(NirExpr::F32("1.5".into())) == known(NirExpr::F32("1.50000001".into())));
    assert!(known(NirExpr::F64("1.5".into())) == known(NirExpr::F64("1.5000".into())));
    for pair in [
        (NirExpr::F32("0.0".into()), NirExpr::F32("-0.0".into())),
        (NirExpr::F64("0.0".into()), NirExpr::F64("-0.0".into())),
        (
            NirExpr::F64("1.5".into()),
            NirExpr::F64("1.50000001".into()),
        ),
    ] {
        assert!(known(pair.0) != known(pair.1));
    }
}

#[test]
fn return_invariant_literal_casts_wrap_and_sign_extend_without_codec_authority() {
    let narrow = |v| NirExpr::CastI64ToI32(Box::new(NirExpr::Int(v)));
    assert!(known(narrow(7)) == known(narrow(4294967303)));
    let extended = NirExpr::CastI32ToI64(Box::new(narrow(4294967295)));
    assert!(known(extended) == known(NirExpr::Int(-1)));
    assert!(known(narrow(7)).words != known(NirExpr::Int(7)).words);
    for expr in [
        NirExpr::CastI64ToI32(Box::new(NirExpr::Bool(true))),
        NirExpr::CastI32ToI64(Box::new(NirExpr::Int(7))),
    ] {
        assert!(value(&expr, &mut Budget(MAX_WORK), 0).is_none());
    }
    for expr in [
        NirExpr::PackF64Word(Box::new(NirExpr::F64("7.0".into()))),
        NirExpr::UnpackF32Word(Box::new(NirExpr::Int(0))),
    ] {
        assert_eq!(
            value(&expr, &mut Budget(MAX_WORK), 0).unwrap().words.len(),
            1
        );
        assert!(value(&expr, &mut Budget(MAX_WORK), 0).unwrap().words[0].is_none());
    }
}

#[test]
fn return_invariant_literal_parsing_is_bounded_and_nonfinite_text_stays_unknown() {
    for text in ["NaN", "inf", "-inf", "1e9999", "not-a-float"] {
        for expr in [NirExpr::F32(text.into()), NirExpr::F64(text.into())] {
            assert!(value(&expr, &mut Budget(MAX_WORK), 0).unwrap().words[0].is_none());
        }
    }
    let expr = NirExpr::F64("0".repeat(MAX_WORK));
    assert!(value(&expr, &mut Budget(MAX_WORK), 0).is_none());
    assert!(value(&NirExpr::Int(7), &mut Budget(0), 0).is_none());
    assert!(value(&NirExpr::Int(7), &mut Budget(MAX_WORK), MAX_DEPTH).is_none());
}

#[test]
fn return_invariant_literals_do_not_recover_delayed_or_mismatched_snapshot_values() {
    let prefix = format!("{ENTRY} let delayed = carry;");
    let effects = format!(
        "let carry = delayed; {}",
        REPEAT
            .replace("let carry", "let delayed")
            .replace("value: 7", "value: 8")
    );
    assert_eq!(
        masks(&prefix, &effects)["carry"],
        [false, true, true, true, true]
    );
    let prefix = format!("{ENTRY} let tag = 8;");
    let effects = REPEAT.replace("value: 7", "value: tag");
    assert_eq!(
        masks(&prefix, &effects)["carry"],
        [false, true, true, true, true]
    );
}

#[test]
fn return_invariant_literal_exhaustion_does_not_publish_snapshot_or_advance_clock() {
    let (module, scope, _) = tests::fixture("");
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    let mut snapshots = Snapshots::new(&scope, &layouts, &catalog, &mut Budget(MAX_WORK)).unwrap();
    let before = snapshots.env.clone();
    let mut clock = 17;
    let stmt = NirStmt::Let {
        name: "fresh".into(),
        ty: Some(scalar_type("f64")),
        value: NirExpr::F64("0".repeat(MAX_WORK)),
    };
    assert!(snapshots
        .bind(
            &stmt,
            &layouts,
            &catalog,
            &mut Budget(MAX_WORK),
            &mut clock,
            0
        )
        .is_none());
    assert!(snapshots.env == before);
    assert_eq!(clock, 17);
}
