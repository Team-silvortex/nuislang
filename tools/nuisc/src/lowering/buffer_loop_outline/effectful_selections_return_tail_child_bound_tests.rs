use super::*;

#[test]
fn return_quint_tail_in_fields_bounds_six_authorized_children_across_the_whole_tail() {
    let owner = Target("local");
    let mut scope = Scope::from([
        ("local".into(), scalar_type("i64")),
        ("seed".into(), scalar_type("i64")),
    ]);
    let names = (0..6).map(|i| format!("field_{i}")).collect::<Vec<_>>();
    for name in &names {
        scope.insert(name.clone(), scalar_type("i64"));
    }
    let staged = regions::Staging {
        local: scope.clone(),
        defined: names.iter().cloned().collect(),
        inputs: BTreeSet::new(),
        calls: BTreeSet::new(),
        constants: BTreeSet::new(),
        statements: vec![],
    };
    let binding = |name: &str| NirStmt::Let {
        name: name.into(),
        ty: None,
        value: NirExpr::Var("seed".into()),
    };
    let original = names.iter().map(|name| binding(name)).collect::<Vec<_>>();
    let entering = Scope::from([("seed".into(), scalar_type("i64"))]);
    let children = classify(&original, &[], &entering, &staged, &owner, 0)
        .unwrap()
        .unwrap();
    let five = children
        .tail_children(&original[..5], &[], &scope, &owner, 0)
        .unwrap();
    assert_eq!(five, names[..5].iter().cloned().collect());
    assert!(children
        .tail_children(&original, &[], &scope, &owner, 0)
        .is_none());
    let separate = NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![binding(&names[5])],
        else_body: vec![],
    };
    let across = original[..5]
        .iter()
        .cloned()
        .chain([separate])
        .collect::<Vec<_>>();
    assert!(children
        .tail_children(&across, &[], &scope, &owner, 0)
        .is_none());
    let including_owner = original[..5]
        .iter()
        .cloned()
        .chain([binding("local")])
        .collect::<Vec<_>>();
    assert_eq!(
        children
            .tail_children(&including_owner, &[], &scope, &owner, 0)
            .unwrap(),
        five
    );
    let root = [binding("seed")];
    assert!(children
        .tail_children(&root, &[], &scope, &owner, 0)
        .is_none());
}
