use super::*;

fn signed_word(name: &str, narrow: bool) -> NirExpr {
    let inner = Box::new(NirExpr::Var(name.into()));
    let packed = if narrow {
        NirExpr::PackF32Word(inner)
    } else {
        NirExpr::PackF64Word(inner)
    };
    let bits = Box::new(NirExpr::Binary {
        op: NirBinaryOp::Xor,
        lhs: Box::new(packed),
        rhs: Box::new(NirExpr::Int(if narrow { 1 << 31 } else { i64::MIN })),
    });
    if narrow {
        NirExpr::UnpackF32Word(bits)
    } else {
        NirExpr::UnpackF64Word(bits)
    }
}

#[test]
fn loop_word_temporaries_preserve_exact_width_and_ordered_self_or_sibling_reads() {
    for narrow in [false, true] {
        let ty = scalar_type(if narrow { "f32" } else { "f64" });
        let mut locals = Locals {
            scope: Scope::from([
                ("current".into(), ty.clone()),
                ("sibling".into(), ty.clone()),
            ]),
            writable: BTreeSet::from(["current".into(), "sibling".into()]),
            available: BTreeSet::new(),
        };
        let updates = locals.writable.clone();
        let catalog = ScalarHelpers::new();
        let layouts = control_values::CarryLayouts::new();
        let check = |value: &NirExpr, locals: &Locals, own| {
            expression(value, locals, &updates, own, &catalog, &layouts, 0)
        };
        assert_eq!(
            check(&signed_word("current", narrow), &locals, Some("current")),
            Some(ty.clone())
        );
        assert!(check(&signed_word("current", narrow), &locals, None).is_none());
        assert!(check(&signed_word("sibling", narrow), &locals, Some("current")).is_none());
        locals.available.insert("sibling".into());
        assert_eq!(
            check(&signed_word("sibling", narrow), &locals, Some("current")),
            Some(ty)
        );
    }
}

#[test]
fn loop_word_temporaries_reject_wrong_kinds_unknown_names_and_excess_depth() {
    let locals = Locals {
        scope: Scope::from([
            ("float".into(), scalar_type("f32")),
            ("word".into(), scalar_type("i64")),
        ]),
        writable: BTreeSet::new(),
        available: BTreeSet::new(),
    };
    let catalog = ScalarHelpers::new();
    let layouts = control_values::CarryLayouts::new();
    let check = |value: &NirExpr| {
        expression(
            value,
            &locals,
            &BTreeSet::new(),
            None,
            &catalog,
            &layouts,
            0,
        )
    };
    for value in [
        NirExpr::PackF64Word(Box::new(NirExpr::Var("float".into()))),
        NirExpr::UnpackF32Word(Box::new(NirExpr::Var("float".into()))),
        NirExpr::PackF32Word(Box::new(NirExpr::Var("word".into()))),
        signed_word("missing", true),
        NirExpr::Binary {
            op: NirBinaryOp::Xor,
            lhs: Box::new(NirExpr::Bool(true)),
            rhs: Box::new(NirExpr::Bool(false)),
        },
    ] {
        assert!(check(&value).is_none());
    }
    let mut value = NirExpr::Var("word".into());
    for _ in 0..33 {
        value = NirExpr::PackF32Word(Box::new(NirExpr::UnpackF32Word(Box::new(value))));
    }
    assert!(check(&value).is_none());
}
