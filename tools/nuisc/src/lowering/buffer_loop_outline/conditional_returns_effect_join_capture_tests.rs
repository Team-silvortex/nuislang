use super::*;

struct SourceArm {
    body: Vec<NirStmt>,
    scope: Scope,
    values: BTreeMap<String, NirExpr>,
    continues: bool,
}

impl SourceArm {
    fn new(value: Option<NirExpr>, ty: &NirTypeRef, constant: bool) -> Self {
        let Some(value) = value else {
            return Self {
                body: vec![NirStmt::Return(Some(NirExpr::Int(0)))],
                scope: Scope::new(),
                values: BTreeMap::new(),
                continues: false,
            };
        };
        let stmt = if constant {
            NirStmt::Const {
                name: "result".into(),
                ty: ty.clone(),
                value: value.clone(),
            }
        } else {
            NirStmt::Let {
                name: "result".into(),
                ty: Some(ty.clone()),
                value: value.clone(),
            }
        };
        Self {
            body: vec![stmt],
            scope: Scope::from([("result".into(), ty.clone())]),
            values: BTreeMap::from([("result".into(), value)]),
            continues: true,
        }
    }

    fn view(&self) -> Arm<'_> {
        Arm {
            body: &self.body,
            scope: &self.scope,
            values: &self.values,
            continues: self.continues,
        }
    }
}

fn installed(
    yes: Option<NirExpr>,
    no: Option<NirExpr>,
    ty: &NirTypeRef,
    constant: bool,
    inputs: Scope,
) -> (NirFunction, NirStmt) {
    let yes = SourceArm::new(yes, ty, constant);
    let no = SourceArm::new(no, ty, constant);
    let mut ready = inputs;
    ready.insert("selection".into(), scalar_type("bool"));
    let mut bindings = BTreeSet::from(["__nuis_effect_join_0".into()]);
    let (target, joined) = prepare(
        yes.view(),
        no.view(),
        &Scope::new(),
        &mut ready,
        "selection",
        &mut bindings,
    )
    .unwrap();
    assert_eq!(target, "result");
    assert_ne!(joined.name, "__nuis_effect_join_0");
    assert_eq!(ready.get(&joined.name), Some(ty));
    let mut names = BTreeSet::from(["__nuis_effect_join_value_0".into()]);
    let mut helpers = Vec::new();
    let mut output = Vec::new();
    install(
        joined,
        &NirExpr::Var("source_live".into()),
        &mut names,
        &mut helpers,
        &mut output,
    );
    assert_eq!(helpers.len(), 1);
    assert_eq!(output.len(), 1);
    (helpers.remove(0), output.remove(0))
}

fn reads(body: &[NirStmt], inputs: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            NirStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                control_values::collect_inputs(condition, inputs);
                reads(then_body, inputs);
                reads(else_body, inputs);
            }
            NirStmt::Return(Some(value)) => control_values::collect_inputs(value, inputs),
            _ => panic!("join helpers must remain pure selections or atom returns"),
        }
    }
}

fn check(function: &NirFunction, call: &NirStmt, captures: &[&str], ty: &NirTypeRef) {
    assert_ne!(function.name, "__nuis_effect_join_value_0");
    assert_eq!(function.return_type.as_ref(), Some(ty));
    assert_eq!(function.params[0].ty, scalar_type("bool"));
    assert_eq!(
        function.params[1..]
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        captures
    );
    let mut used = BTreeSet::new();
    reads(&function.body, &mut used);
    assert_eq!(
        used,
        function.params.iter().map(|p| p.name.clone()).collect()
    );
    let NirStmt::Let {
        value: NirExpr::Call { callee, args },
        ty: result_type,
        ..
    } = call
    else {
        panic!()
    };
    assert_eq!(callee, &function.name);
    assert_eq!(result_type.as_ref(), Some(ty));
    let mut expected = vec![NirExpr::Var("source_live".into())];
    expected.extend(captures.iter().map(|name| NirExpr::Var((*name).into())));
    assert_eq!(args, &expected);
    let NirStmt::If {
        condition,
        else_body,
        ..
    } = &function.body[0]
    else {
        panic!()
    };
    assert_eq!(condition, &NirExpr::Var(function.params[0].name.clone()));
    let seed = if ty == &scalar_type("bool") {
        NirExpr::Bool(false)
    } else {
        NirExpr::Int(0)
    };
    assert_eq!(else_body, &[NirStmt::Return(Some(seed))]);
}

#[test]
fn conditional_return_join_captures_drop_only_unused_single_value_selectors() {
    for word in [false, true] {
        let ty = scalar_type(if word { "i64" } else { "bool" });
        for constant in [false, true] {
            for swapped in [false, true] {
                for staged in [false, true] {
                    let atom = match (staged, word) {
                        (true, _) => NirExpr::Var("snapshot".into()),
                        (false, true) => NirExpr::Int(0),
                        (false, false) => NirExpr::Bool(false),
                    };
                    let (yes, no) = if swapped {
                        (None, Some(atom))
                    } else {
                        (Some(atom), None)
                    };
                    let (function, call) = installed(
                        yes,
                        no,
                        &ty,
                        constant,
                        Scope::from([("snapshot".into(), ty.clone())]),
                    );
                    let captures: &[&str] = if staged { &["snapshot"] } else { &[] };
                    check(&function, &call, captures, &ty);
                    assert_eq!(function.params.len(), if staged { 2 } else { 1 });
                }
            }
        }
    }
}

#[test]
fn conditional_return_join_captures_retain_condition_as_data_and_hygienic_live_gate() {
    for name in ["selection", "__nuis_join_live_0"] {
        for swapped in [false, true] {
            let value = Some(NirExpr::Var(name.into()));
            let (yes, no) = if swapped {
                (None, value)
            } else {
                (value, None)
            };
            let ty = scalar_type("bool");
            let (function, call) = installed(
                yes,
                no,
                &ty,
                false,
                Scope::from([(name.into(), ty.clone())]),
            );
            check(&function, &call, &[name], &ty);
            assert_ne!(function.params[0].name, name);
            assert_eq!(function.params[1].ty, ty);
        }
    }
}

#[test]
fn conditional_return_join_captures_preserve_paired_selection_and_deduplicate_atoms() {
    for word in [false, true] {
        let ty = scalar_type(if word { "i64" } else { "bool" });
        for constant in [false, true] {
            for shape in ["literal", "same", "distinct"] {
                let atom = if word {
                    NirExpr::Int(0)
                } else {
                    NirExpr::Bool(false)
                };
                let (yes, no, captures) = match shape {
                    "literal" => (atom.clone(), atom, vec![]),
                    "same" => (
                        NirExpr::Var("left".into()),
                        NirExpr::Var("left".into()),
                        vec!["left"],
                    ),
                    "distinct" => (
                        NirExpr::Var("left".into()),
                        NirExpr::Var("right".into()),
                        vec!["left", "right", "selection"],
                    ),
                    _ => unreachable!(),
                };
                let (function, call) = installed(
                    Some(yes),
                    Some(no),
                    &ty,
                    constant,
                    Scope::from([("left".into(), ty.clone()), ("right".into(), ty.clone())]),
                );
                check(&function, &call, &captures, &ty);
                let NirStmt::If { then_body, .. } = &function.body[0] else {
                    panic!()
                };
                if shape == "distinct" {
                    let NirStmt::If { condition, .. } = &then_body[0] else {
                        panic!()
                    };
                    assert_eq!(condition, &NirExpr::Var("selection".into()));
                } else {
                    assert!(matches!(&then_body[0], NirStmt::Return(Some(_))));
                }
            }
        }
    }
}

#[test]
fn conditional_return_equal_effect_joins_retain_equal_condition_data_and_nonseed_atoms() {
    for constant in [false, true] {
        for (ty, value, captures) in [
            (
                scalar_type("bool"),
                NirExpr::Var("selection".into()),
                vec!["selection"],
            ),
            (
                scalar_type("bool"),
                NirExpr::Var("__nuis_join_live_0".into()),
                vec!["__nuis_join_live_0"],
            ),
            (scalar_type("bool"), NirExpr::Bool(true), vec![]),
            (scalar_type("i64"), NirExpr::Int(7), vec![]),
        ] {
            let (function, call) = installed(
                Some(value.clone()),
                Some(value.clone()),
                &ty,
                constant,
                Scope::from([("__nuis_join_live_0".into(), scalar_type("bool"))]),
            );
            check(&function, &call, &captures, &ty);
            let NirStmt::If { then_body, .. } = &function.body[0] else {
                panic!()
            };
            assert_eq!(then_body, &[NirStmt::Return(Some(value))]);
            assert!(captures.iter().all(|name| *name != function.params[0].name));
        }
    }
}

#[test]
fn conditional_return_equal_effect_joins_do_not_skip_either_arm_validation() {
    for mutation in [
        "missing",
        "wrong-type",
        "borrow",
        "wrong-atom",
        "kind",
        "name",
        "parent",
        "unready",
    ] {
        let ty = scalar_type("i64");
        let value = NirExpr::Var("snapshot".into());
        let yes = SourceArm::new(Some(value.clone()), &ty, false);
        let mut no = SourceArm::new(Some(value), &ty, false);
        let mut scope = Scope::new();
        let mut ready = Scope::from([
            ("selection".into(), scalar_type("bool")),
            ("snapshot".into(), ty.clone()),
        ]);
        match mutation {
            "missing" => {
                no.values.clear();
            }
            "wrong-type" => {
                no.scope.insert("result".into(), scalar_type("bool"));
            }
            "borrow" => {
                no.scope.get_mut("result").unwrap().is_ref = true;
            }
            "wrong-atom" => {
                no.values.insert("result".into(), NirExpr::Bool(false));
            }
            "kind" => {
                no.body[0] = NirStmt::Const {
                    name: "result".into(),
                    ty: ty.clone(),
                    value: NirExpr::Var("snapshot".into()),
                };
            }
            "name" => {
                let NirStmt::Let { name, .. } = &mut no.body[0] else {
                    panic!()
                };
                *name = "different".into();
            }
            "parent" => {
                scope.insert("result".into(), ty.clone());
            }
            "unready" => {
                ready.remove("snapshot");
            }
            _ => unreachable!(),
        }
        let mut bindings = BTreeSet::from(["reserved".into()]);
        let before = (ready.clone(), bindings.clone());
        assert!(
            prepare(
                yes.view(),
                no.view(),
                &scope,
                &mut ready,
                "selection",
                &mut bindings
            )
            .is_none(),
            "{mutation}"
        );
        assert_eq!((ready, bindings), before, "{mutation}");
    }
}

#[test]
fn conditional_return_join_captures_validate_unused_condition_before_atomic_publication() {
    for paired in [false, true] {
        for condition in [
            None,
            Some(scalar_type("i64")),
            Some(NirTypeRef {
                is_ref: true,
                ..scalar_type("bool")
            }),
        ] {
            let ty = scalar_type("i64");
            let yes = SourceArm::new(Some(NirExpr::Int(0)), &ty, false);
            let no = SourceArm::new(paired.then_some(NirExpr::Int(0)), &ty, false);
            let mut ready = Scope::new();
            if let Some(ty) = condition {
                ready.insert("selection".into(), ty);
            }
            let mut bindings = BTreeSet::from(["reserved".into()]);
            let before = (ready.clone(), bindings.clone());
            assert!(prepare(
                yes.view(),
                no.view(),
                &Scope::new(),
                &mut ready,
                "selection",
                &mut bindings
            )
            .is_none());
            assert_eq!((ready, bindings), before);
        }
    }
}
