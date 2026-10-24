use super::*;

fn field(base: NirExpr, name: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: name.into(),
    }
}

fn comparison(value: NirExpr, op: NirBinaryOp, reverse: bool) -> NirExpr {
    let (lhs, rhs) = if reverse {
        (NirExpr::Bool(true), value)
    } else {
        (value, NirExpr::Bool(true))
    };
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn layouts() -> control_values::CarryLayouts {
    control_values::CarryLayouts::from([
        (
            "State".into(),
            vec![
                ("flag".into(), scalar_type("bool")),
                ("total".into(), scalar_type("i64")),
            ],
        ),
        ("Outer".into(), vec![("inner".into(), scalar_type("State"))]),
    ])
}

#[test]
fn loop_bool_projections_preserve_ordered_self_and_sibling_read_authority() {
    let layouts = layouts();
    let catalog = ScalarHelpers::new();
    for nested in [false, true] {
        let ty = scalar_type(if nested { "Outer" } else { "State" });
        let mut locals = Locals {
            scope: Scope::from([("live".into(), ty.clone()), ("other".into(), ty)]),
            writable: BTreeSet::from(["live".into(), "other".into()]),
            available: BTreeSet::new(),
        };
        let updates = locals.writable.clone();
        for op in [NirBinaryOp::Eq, NirBinaryOp::Ne] {
            for reverse in [false, true] {
                let value = |name: &str| {
                    let base = NirExpr::Var(name.into());
                    let base = if nested { field(base, "inner") } else { base };
                    comparison(field(base, "flag"), op, reverse)
                };
                let check = |name: &str, locals: &Locals, own| {
                    expression(&value(name), locals, &updates, own, &catalog, &layouts, 0)
                };
                locals.available.clear();
                assert_eq!(
                    check("live", &locals, Some("live")),
                    Some(scalar_type("bool"))
                );
                assert!(check("live", &locals, None).is_none());
                assert!(check("other", &locals, Some("live")).is_none());
                locals.available.insert("other".into());
                assert_eq!(
                    check("other", &locals, Some("live")),
                    Some(scalar_type("bool"))
                );
            }
        }
    }
}

#[test]
fn loop_bool_projections_keep_type_call_and_compound_expression_boundaries() {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
            struct State { flag: bool, total: i64 }
            fn make(flag: bool) -> State { return State { flag: flag, total: 1 }; }
            fn main() -> i64 { return 0; }
        }",
    )
    .unwrap();
    let layouts = layouts();
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    assert!(catalog.contains_key("make"));
    let locals = Locals {
        scope: Scope::from([("live".into(), scalar_type("State"))]),
        writable: BTreeSet::new(),
        available: BTreeSet::new(),
    };
    let check = |value: &NirExpr, catalog: &ScalarHelpers| {
        expression(value, &locals, &BTreeSet::new(), None, catalog, &layouts, 0)
    };
    let call = |args| NirExpr::Call {
        callee: "make".into(),
        args,
    };
    let projected = |base, name| comparison(field(base, name), NirBinaryOp::Ne, false);
    let valid = projected(call(vec![NirExpr::Bool(false)]), "flag");
    assert_eq!(check(&valid, &catalog), Some(scalar_type("bool")));
    assert!(check(&valid, &ScalarHelpers::new()).is_none());
    for value in [
        projected(call(vec![NirExpr::Int(0)]), "flag"),
        projected(NirExpr::Var("missing".into()), "flag"),
        projected(NirExpr::Var("live".into()), "missing"),
        projected(NirExpr::Var("live".into()), "total"),
        comparison(
            NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(field(NirExpr::Var("live".into()), "flag")),
                rhs: Box::new(NirExpr::Bool(true)),
            },
            NirBinaryOp::Ne,
            false,
        ),
    ] {
        assert!(check(&value, &catalog).is_none(), "{value:?}");
    }
}
