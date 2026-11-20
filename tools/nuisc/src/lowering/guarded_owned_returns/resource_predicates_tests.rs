use super::*;

fn predicate<'a>(
    value: &'a NirExpr,
    scope: &[NirParam],
    live: &BTreeSet<String>,
) -> Option<(&'a NirExpr, &'a NirExpr, bool, bool)> {
    super::predicate(value, scope, live, &PureValueCalls::default())
}

fn binary(op: NirBinaryOp, left: NirExpr, right: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(left),
        rhs: Box::new(right),
    }
}

fn read() -> NirExpr {
    NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".into())),
        index: Box::new(NirExpr::Int(0)),
    }
}

fn scope() -> Vec<NirParam> {
    [
        ("buffer", "Buffer", true),
        ("flag", "bool", false),
        ("count", "i64", false),
    ]
    .into_iter()
    .map(|(name, ty, is_ref)| NirParam {
        name: name.into(),
        ty: NirTypeRef {
            name: ty.into(),
            is_ref,
            is_optional: false,
            generic_args: vec![],
        },
    })
    .collect()
}

#[test]
fn resource_predicate_identity_handoff_never_bypasses_cleanup_or_partial_owners() {
    let live = BTreeSet::from(["a".to_owned(), "b".to_owned()]);
    for (transfers, cleanup, allowed) in [
        (vec!["a", "b"], vec![], true),
        (vec!["b", "a"], vec![], true),
        (vec!["a"], vec!["b"], false),
        (vec!["a", "b"], vec!["a"], false),
        (vec!["a"], vec![], false),
        (vec!["a", "missing"], vec![], false),
        (vec!["a", "a"], vec![], false),
    ] {
        let edges = OwnerEdges {
            transfers: transfers.into_iter().map(str::to_owned).collect(),
            cleanup: cleanup.into_iter().map(str::to_owned).collect(),
        };
        assert_eq!(identity_handoff(&edges, &live), allowed);
    }
}

#[test]
fn resource_predicate_proof_preserves_exact_types_and_live_owners() {
    let scope = scope();
    let live = BTreeSet::from(["saved".into()]);
    for value in [
        read(),
        NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
    ] {
        let comparison = binary(NirBinaryOp::Gt, value, NirExpr::Int(0));
        let mut root = binary(NirBinaryOp::Or, NirExpr::Var("flag".into()), comparison);
        for inverted in [false, true, false] {
            let (_, _, disjunction, not) = predicate(&root, &scope, &live).unwrap();
            assert!(disjunction);
            assert_eq!(not, inverted);
            root = binary(NirBinaryOp::Eq, root, NirExpr::Bool(false));
        }
    }
    let dead = binary(
        NirBinaryOp::And,
        NirExpr::Bool(false),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
            NirExpr::Int(0),
        ),
    );
    assert!(predicate(&dead, &scope, &live).is_none());
    for leaf in [
        binary(NirBinaryOp::Gt, read(), NirExpr::Bool(false)),
        binary(
            NirBinaryOp::Xor,
            NirExpr::Bool(true),
            binary(
                NirBinaryOp::And,
                NirExpr::Int(1),
                binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
            ),
        ),
        NirExpr::Call {
            callee: "unknown".into(),
            args: vec![read()],
        },
        NirExpr::BytesLen(Box::new(NirExpr::Move(Box::new(NirExpr::Var(
            "saved".into(),
        ))))),
    ] {
        assert!(predicate(
            &binary(NirBinaryOp::And, NirExpr::Bool(false), leaf),
            &scope,
            &live
        )
        .is_none());
    }
    let mut wrong = scope.clone();
    wrong[0].ty.is_ref = false;
    let root = binary(
        NirBinaryOp::And,
        NirExpr::Bool(true),
        binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
    );
    assert!(predicate(&root, &wrong, &live).is_none());
    let read_only_lhs = binary(
        NirBinaryOp::And,
        binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
        NirExpr::Var("flag".into()),
    );
    assert!(predicate(&read_only_lhs, &scope, &live).is_none());
}

#[test]
fn nested_resource_predicates_prove_both_subtrees_and_count_all_lazy_edges() {
    let scope = scope();
    let live = BTreeSet::new();
    let leaf = || binary(NirBinaryOp::Gt, read(), NirExpr::Int(0));
    for op in [NirBinaryOp::And, NirBinaryOp::Or] {
        let nested = binary(op, NirExpr::Var("flag".into()), leaf());
        for (left, right) in [(nested.clone(), leaf()), (leaf(), nested.clone())] {
            let root = binary(op, left, right);
            assert!(predicate(&root, &scope, &live).is_some());
            let inverted = binary(NirBinaryOp::Eq, root, NirExpr::Bool(false));
            assert!(predicate(&inverted, &scope, &live).is_some());
        }
    }
    fn tree(edges: usize) -> NirExpr {
        if edges == 0 {
            return binary(NirBinaryOp::Gt, read(), NirExpr::Int(0));
        }
        let left = (edges - 1) / 2;
        binary(NirBinaryOp::And, tree(left), tree(edges - 1 - left))
    }
    assert!(predicate(&tree(32), &scope, &live).is_some());
    assert!(predicate(&tree(33), &scope, &live).is_none());
    for wrapper in [
        binary(NirBinaryOp::Eq, tree(1), NirExpr::Bool(true)),
        binary(NirBinaryOp::Xor, tree(1), NirExpr::Bool(false)),
    ] {
        assert!(predicate(&binary(NirBinaryOp::And, tree(1), wrapper), &scope, &live).is_some());
    }
    for invalid in [
        binary(NirBinaryOp::Eq, tree(1), NirExpr::Int(1)),
        binary(NirBinaryOp::Lt, tree(1), NirExpr::Bool(false)),
        binary(NirBinaryOp::Xor, tree(1), NirExpr::Int(0)),
        NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
    ] {
        assert!(predicate(&binary(NirBinaryOp::And, tree(1), invalid), &scope, &live).is_none());
    }
}

#[test]
fn resource_predicate_proof_bounds_original_depth_and_work() {
    let scope = scope();
    let live = BTreeSet::new();
    let base = binary(
        NirBinaryOp::And,
        NirExpr::Bool(true),
        binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
    );
    let mut root = base;
    for wrappers in 0..=61 {
        assert_eq!(
            predicate(&root, &scope, &live).is_some(),
            wrappers <= 60,
            "wrappers={wrappers}"
        );
        root = binary(NirBinaryOp::Eq, root, NirExpr::Bool(false));
    }
    fn sum(leaves: usize) -> NirExpr {
        if leaves == 1 {
            return NirExpr::Int(0);
        }
        binary(NirBinaryOp::Add, sum(leaves / 2), sum(leaves - leaves / 2))
    }
    for leaves in [2044, 2045] {
        let left = binary(NirBinaryOp::Eq, sum(leaves), NirExpr::Int(0));
        let root = binary(
            NirBinaryOp::And,
            left,
            binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
        );
        assert_eq!(predicate(&root, &scope, &live).is_some(), leaves == 2044);
    }
    let exact_limit = binary(
        NirBinaryOp::And,
        binary(NirBinaryOp::Eq, sum(2045), NirExpr::Int(0)),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
            NirExpr::Int(0),
        ),
    );
    assert!(predicate(&exact_limit, &scope, &BTreeSet::from(["saved".into()])).is_some());
}

#[test]
fn resource_predicate_calls_require_completed_pure_exact_signatures() {
    let module = crate::pipeline::compile_source(
        r#"mod cpu Main {
      struct Choice { value: bool }
      fn choose(flag: bool) -> Choice { return Choice { value: flag }; }
      fn positive(value: i64) -> bool { return choose(value > 0).value; }
      fn main() -> i64 { return 0; }
    }"#,
    )
    .unwrap()
    .nir;
    let calls = PureValueCalls::collect(&module);
    let call = |callee: &str, arg| NirExpr::Call {
        callee: callee.into(),
        args: vec![arg],
    };
    let root = |leaf| binary(NirBinaryOp::And, NirExpr::Bool(false), leaf);
    let scope = scope();
    let live = BTreeSet::from(["saved".into()]);
    let valid = root(call("positive", read()));
    assert!(super::predicate(&valid, &scope, &live, &calls).is_some());
    let pure = root(call("positive", NirExpr::Var("count".into())));
    assert!(arguments::contains_logic(&pure));
    assert!(!arguments::contains_argument_logic(&pure));
    assert!(
        !admitted(&pure, &scope, &live, &calls, Kind::Bool),
        "pure predicates retain scalar-helper routing"
    );
    let field = |field: &str| NirExpr::FieldAccess {
        base: Box::new(call(
            "choose",
            binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
        )),
        field: field.into(),
    };
    assert!(super::predicate(&root(field("value")), &scope, &live, &calls).is_some());
    for invalid in [
        call("positive", binary(NirBinaryOp::Gt, read(), NirExpr::Int(0))),
        call("choose", read()),
        call("positive", NirExpr::Var("buffer".into())),
        call(
            "positive",
            NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
        ),
        call(
            "choose",
            binary(
                NirBinaryOp::Xor,
                root(binary(NirBinaryOp::Gt, read(), NirExpr::Int(0))),
                NirExpr::Int(0),
            ),
        ),
        field("unknown"),
    ] {
        assert!(super::predicate(&root(invalid), &scope, &live, &calls).is_none());
    }
    for mutation in 0..3 {
        let mut changed = module.clone();
        let helper = changed
            .functions
            .iter_mut()
            .find(|f| f.name == "positive")
            .unwrap();
        match mutation {
            0 => helper.body.insert(0, NirStmt::Print(NirExpr::Int(0))),
            1 => helper.params[0].ty.is_ref = true,
            _ => {
                helper.body = vec![NirStmt::Return(Some(call(
                    "positive",
                    NirExpr::Var("value".into()),
                )))]
            }
        }
        assert!(
            super::predicate(&valid, &scope, &live, &PureValueCalls::collect(&changed)).is_none()
        );
    }
}

#[test]
fn resource_argument_snapshots_preserve_order_and_prove_before_allocating_names() {
    let module = crate::pipeline::compile_source(
        r#"mod cpu Main {
      fn combine(left: i64, flag: bool, right: i64) -> bool { return flag && left > right; }
      fn main() -> i64 { return 0; }
    }"#,
    )
    .unwrap()
    .nir;
    let mut template = module
        .functions
        .iter()
        .find(|f| f.name == "main")
        .unwrap()
        .clone();
    template.body.insert(
        0,
        NirStmt::Let {
            name: "__nuis_resource_predicate_left".into(),
            ty: None,
            value: NirExpr::Int(0),
        },
    );
    let scope = scope();
    let live = BTreeSet::from(["saved".into()]);
    let mut context = OutlineContext {
        reserved: BTreeSet::new(),
        pending: VecDeque::new(),
        roots: 1,
        outlines: Outlines::default(),
        calls: PureValueCalls::collect(&module),
        return_layouts: BTreeMap::new(),
    };
    let logical = binary(
        NirBinaryOp::And,
        NirExpr::Var("flag".into()),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
            NirExpr::Int(0),
        ),
    );
    let value = NirExpr::Call {
        callee: "combine".into(),
        args: vec![
            read(),
            logical.clone(),
            binary(
                NirBinaryOp::Div,
                NirExpr::Int(1),
                NirExpr::Var("count".into()),
            ),
        ],
    };
    assert!(admitted(&value, &scope, &live, &context.calls, Kind::Bool));
    let mut bad = value.clone();
    let NirExpr::Call { args, .. } = &mut bad else {
        unreachable!()
    };
    args[2] = NirExpr::Bool(false);
    let declaration = NirStmt::Let {
        name: "flag".into(),
        ty: Some(scope[1].ty.clone()),
        value: bad,
    };
    let mut suffix = VecDeque::from(template.body.clone());
    let original_suffix = suffix.clone();
    assert!(split(
        &declaration,
        &template,
        &mut suffix,
        &scope,
        &live,
        1,
        &mut context
    )
    .unwrap()
    .is_none());
    assert_eq!(suffix, original_suffix);
    assert!(
        context.reserved.is_empty()
            && context.pending.is_empty()
            && context.outlines.owners.is_empty()
    );
    let (prefix, rewritten) = arguments::extract(&value, &template, &scope, &live, &mut context);
    assert_eq!(prefix.len(), 3);
    let NirExpr::Call { args, .. } = rewritten else {
        unreachable!()
    };
    let mut names = BTreeSet::new();
    for (index, statement) in prefix.iter().enumerate() {
        let NirStmt::Let {
            name,
            ty: Some(ty),
            value,
        } = statement
        else {
            unreachable!()
        };
        assert_ne!(name, "__nuis_resource_predicate_left");
        assert!(names.insert(name));
        assert_eq!(args[index], NirExpr::Var(name.clone()));
        assert_eq!(ty.name, if index == 1 { "bool" } else { "i64" });
        if index == 1 {
            assert_eq!(value, &logical);
        }
    }
    assert!(matches!(
        &prefix[0],
        NirStmt::Let {
            value: NirExpr::LoadAt { .. },
            ..
        }
    ));
    assert!(matches!(
        &prefix[2],
        NirStmt::Let {
            value: NirExpr::Binary {
                op: NirBinaryOp::Div,
                ..
            },
            ..
        }
    ));
}

#[test]
fn resource_argument_budgets_count_hidden_logic_and_pending_arguments() {
    fn tree(edges: usize) -> NirExpr {
        if edges == 0 {
            return binary(NirBinaryOp::Gt, read(), NirExpr::Int(0));
        }
        let left = (edges - 1) / 2;
        binary(NirBinaryOp::And, tree(left), tree(edges - left - 1))
    }
    for count in [32, 33] {
        for op in [NirBinaryOp::Eq, NirBinaryOp::Ne, NirBinaryOp::Xor] {
            let value = binary(op, tree(count), NirExpr::Bool(true));
            assert_eq!(bounded(&value).is_some(), count == 32);
        }
        let value = NirExpr::Call {
            callee: "choose".into(),
            args: vec![tree(count)],
        };
        assert_eq!(bounded(&value).is_some(), count == 32);
    }
    for count in [4095, 4096] {
        let value = NirExpr::Call {
            callee: "unknown".into(),
            args: vec![NirExpr::Int(0); count],
        };
        assert_eq!(bounded(&value).is_some(), count == 4095);
    }
}

#[test]
fn resource_wrapper_snapshots_are_ordered_and_leave_lazy_children_intact() {
    let module = crate::pipeline::compile_source("mod cpu Main { fn main() -> i64 { return 0; } }")
        .unwrap()
        .nir;
    let template = &module.functions[0];
    let scope = scope();
    let live = BTreeSet::from(["saved".into()]);
    let first = binary(NirBinaryOp::Gt, read(), NirExpr::Int(0));
    let lazy = binary(
        NirBinaryOp::Or,
        NirExpr::Var("flag".into()),
        binary(
            NirBinaryOp::Gt,
            NirExpr::BytesLen(Box::new(NirExpr::Var("saved".into()))),
            NirExpr::Int(0),
        ),
    );
    for op in [NirBinaryOp::Eq, NirBinaryOp::Ne, NirBinaryOp::Xor] {
        for (left, right) in [
            (first.clone(), lazy.clone()),
            (lazy.clone(), first.clone()),
            (lazy.clone(), lazy.clone()),
        ] {
            let value = binary(op, left.clone(), right.clone());
            let mut context = OutlineContext {
                reserved: BTreeSet::new(),
                pending: VecDeque::new(),
                roots: 1,
                outlines: Outlines::default(),
                calls: PureValueCalls::default(),
                return_layouts: BTreeMap::new(),
            };
            assert!(admitted(&value, &scope, &live, &context.calls, Kind::Bool));
            let (prefix, rewritten) =
                arguments::extract(&value, template, &scope, &live, &mut context);
            let mut cursor = 0;
            let mut names = vec![];
            for original in [left, right] {
                let expected = if original == first {
                    let NirStmt::Let {
                        name,
                        ty: Some(ty),
                        value,
                    } = &prefix[cursor]
                    else {
                        panic!("ready read snapshot required");
                    };
                    assert_eq!(ty.name, "i64");
                    assert_eq!(value, &read());
                    cursor += 1;
                    binary(NirBinaryOp::Gt, NirExpr::Var(name.clone()), NirExpr::Int(0))
                } else {
                    original
                };
                let statement = &prefix[cursor];
                cursor += 1;
                let NirStmt::Let {
                    name,
                    ty: Some(ty),
                    value,
                } = statement
                else {
                    panic!("typed snapshot required");
                };
                assert_eq!(ty.name, "bool");
                assert_eq!(value, &expected, "neither lazy child may be hoisted");
                names.push(name.clone());
            }
            assert_eq!(cursor, prefix.len());
            assert_ne!(names[0], names[1]);
            assert_eq!(
                rewritten,
                binary(
                    op,
                    NirExpr::Var(names[0].clone()),
                    NirExpr::Var(names[1].clone())
                )
            );
        }
    }
}

#[test]
fn resource_wrapper_rejection_is_atomic_for_bad_types_owners_and_budget() {
    let module = crate::pipeline::compile_source("mod cpu Main { fn main() -> i64 { return 0; } }")
        .unwrap()
        .nir;
    let template = &module.functions[0];
    let scope = scope();
    let live = BTreeSet::from(["saved".into()]);
    let lazy = binary(
        NirBinaryOp::And,
        NirExpr::Var("flag".into()),
        binary(NirBinaryOp::Gt, read(), NirExpr::Int(0)),
    );
    let mut too_deep = lazy.clone();
    for _ in 0..64 {
        too_deep = binary(NirBinaryOp::Xor, too_deep, NirExpr::Bool(false));
    }
    for value in [
        binary(NirBinaryOp::Eq, lazy.clone(), NirExpr::Int(0)),
        binary(NirBinaryOp::Xor, lazy.clone(), NirExpr::Int(0)),
        binary(NirBinaryOp::Lt, lazy.clone(), NirExpr::Bool(false)),
        binary(
            NirBinaryOp::Eq,
            lazy.clone(),
            binary(
                NirBinaryOp::Gt,
                NirExpr::BytesLen(Box::new(NirExpr::Var("dead".into()))),
                NirExpr::Int(0),
            ),
        ),
        binary(
            NirBinaryOp::Xor,
            lazy,
            NirExpr::Call {
                callee: "unknown".into(),
                args: vec![read()],
            },
        ),
        too_deep,
    ] {
        let mut context = OutlineContext {
            reserved: BTreeSet::new(),
            pending: VecDeque::new(),
            roots: 1,
            outlines: Outlines::default(),
            calls: PureValueCalls::default(),
            return_layouts: BTreeMap::new(),
        };
        let mut suffix = VecDeque::from(template.body.clone());
        let original = suffix.clone();
        let statement = NirStmt::Let {
            name: "flag".into(),
            ty: Some(scope[1].ty.clone()),
            value,
        };
        assert!(split(
            &statement,
            template,
            &mut suffix,
            &scope,
            &live,
            1,
            &mut context
        )
        .unwrap()
        .is_none());
        assert_eq!(suffix, original);
        assert!(
            context.reserved.is_empty()
                && context.pending.is_empty()
                && context.outlines.owners.is_empty()
        );
    }
}
