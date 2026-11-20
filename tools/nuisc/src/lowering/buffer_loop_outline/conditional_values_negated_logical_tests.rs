use super::*;

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn negate(value: NirExpr) -> NirExpr {
    binary(NirBinaryOp::Eq, value, NirExpr::Bool(false))
}

fn call(name: &str, value: NirExpr) -> NirExpr {
    NirExpr::Call {
        callee: name.into(),
        args: vec![value],
    }
}

fn logical() -> NirExpr {
    binary(
        NirBinaryOp::Or,
        NirExpr::Var("gate".into()),
        call("flag", NirExpr::Var("value".into())),
    )
}

fn probe(value: &NirExpr) -> (Option<NirExpr>, Vec<NirFunction>) {
    let module = crate::frontend::parse_nuis_module(
        "mod cpu Main {
          fn flag(value: bool) -> bool { return value; }
          fn noisy(value: bool) -> bool { print(1); return value; }
          fn main() -> i64 { return 0; }
        }",
    )
    .unwrap();
    let layouts = control_values::TypedLayouts::collect(&module);
    let carries = control_values::layouts(&module);
    let control = scalar_helpers::collect_with_layouts(&module, &carries);
    let catalog = scalar_helpers::collect_typed_values(&module, &layouts, &control);
    let scope = BTreeMap::from([
        ("gate".into(), scalar_type("bool")),
        ("value".into(), scalar_type("bool")),
    ]);
    let mut names = module
        .functions
        .iter()
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    let original_names = names.clone();
    let mut helpers = Vec::new();
    let checked = BTreeSet::new();
    let mut builder = Builder {
        catalog: &catalog,
        layouts: &layouts,
        names: &mut names,
        checked: &checked,
        helpers: &mut helpers,
        bindings: scope.keys().cloned().collect(),
        extract_selections: false,
        predicate_returns: false,
    };
    let bindings = builder.bindings.clone();
    let result = builder.gated_condition(value, &scope);
    if result.is_none() {
        assert_eq!(bindings, builder.bindings);
        assert_eq!(original_names, names);
        assert!(helpers.is_empty());
    }
    (result, helpers)
}

#[test]
fn negated_logical_roots_share_guards_without_widening_legacy_profiles() {
    for count in [1, 2, 3] {
        let value = (0..count).fold(logical(), |value, _| negate(value));
        assert!(!prefix::computed_logical_root(&value));
        assert!(prefix::computed_predicate_root(&value));
        let (result, helpers) = probe(&value);
        let mut result = result.as_ref().unwrap();
        for _ in 0..count {
            result = prefix::negated_predicate(result).unwrap();
        }
        assert!(matches!(result, NirExpr::Call { .. }));
        assert_eq!(helpers.len(), 1);
        assert_eq!(helpers[0].return_type, Some(scalar_type("bool")));
    }
    let nested = negate(binary(NirBinaryOp::And, negate(logical()), logical()));
    assert_eq!(probe(&nested).1.len(), 3);
}

#[test]
fn negated_logical_roots_count_wrapper_work_depth_and_all_lazy_edges() {
    for (depth, allowed) in [(63, true), (64, false)] {
        let value = (0..depth).fold(NirExpr::Bool(true), |value, _| negate(value));
        assert_eq!(prefix::computed_predicate_root(&value), allowed);
    }
    for (args, allowed) in [(4093, true), (4094, false)] {
        let value = negate(NirExpr::Call {
            callee: "flag".into(),
            args: vec![NirExpr::Bool(true); args],
        });
        assert_eq!(prefix::computed_predicate_root(&value), allowed);
        assert!(probe(&value).0.is_none());
    }
    for (edges, allowed) in [(32, true), (33, false)] {
        let value = negate((0..edges).fold(NirExpr::Bool(true), |left, _| {
            binary(NirBinaryOp::And, left, NirExpr::Bool(true))
        }));
        assert_eq!(prefix::computed_predicate_root(&value), allowed);
    }
}

#[test]
fn negated_logical_roots_reject_effects_resources_hidden_logic_and_bad_types_atomically() {
    for rhs in [
        call("missing", NirExpr::Bool(true)),
        call("flag", NirExpr::Int(1)),
        call("noisy", NirExpr::Bool(true)),
        NirExpr::Var("later".into()),
        NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("pixels".into())),
            index: Box::new(NirExpr::Int(99)),
        },
    ] {
        // A constant, skipping LHS never exempts the RHS from proof.
        assert!(
            probe(&negate(binary(NirBinaryOp::Or, NirExpr::Bool(true), rhs)))
                .0
                .is_none()
        );
    }
    for value in [
        negate(call("flag", logical())),
        negate(binary(
            NirBinaryOp::Eq,
            logical(),
            NirExpr::Var("gate".into()),
        )),
        negate(binary(NirBinaryOp::Xor, logical(), NirExpr::Bool(true))),
    ] {
        assert!(!prefix::computed_predicate_root(&value));
        assert!(probe(&value).0.is_none());
    }
}
