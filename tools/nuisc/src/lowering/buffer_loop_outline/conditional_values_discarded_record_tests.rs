use super::*;

fn module() -> NirModule {
    let mut module = crate::frontend::parse_nuis_module(include_str!(
        "../../../tests/native_application_bridge/discarded_record_prefix.ns"
    ))
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    module
}
fn packet(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "packet")
        .unwrap()
}
fn arms(stmt: &mut NirStmt) -> (&mut NirExpr, &mut Vec<NirStmt>, &mut Vec<NirStmt>) {
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        panic!()
    };
    (condition, then_body, else_body)
}
fn attempt(module: &mut NirModule, denied: Option<&str>) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let mut catalog = scalar_helpers::collect_capture_values(module, &layouts);
    if let Some(name) = denied {
        catalog.remove(name);
    }
    let mut names = module
        .functions
        .iter()
        .map(|f| f.name.clone())
        .collect::<BTreeSet<_>>();
    let before_names = names.clone();
    let mut helpers = Vec::new();
    let checked = BTreeSet::new();
    let f = packet(module);
    let mut scope = f
        .params
        .iter()
        .map(|p| (p.name.clone(), p.ty.clone()))
        .collect::<Scope>();
    let before_scope = scope.clone();
    let mut bindings = scope.keys().cloned().collect();
    branches::collect_bindings(&f.body, &mut bindings);
    let (before, suffix) = f.body.split_at_mut(2);
    let accepted = Builder {
        catalog: &catalog,
        layouts: &layouts,
        names: &mut names,
        checked: &checked,
        helpers: &mut helpers,
        bindings,
        extract_selections: false,
        predicate_returns: false,
    }
    .discarded_record_declaration(&mut before[1], &mut scope, suffix);
    if !accepted {
        assert_eq!(names, before_names);
        assert_eq!(scope, before_scope);
        assert!(helpers.is_empty());
    }
    module.functions.extend(helpers);
    accepted
}
fn reject(module: &mut NirModule) {
    let before = module.clone();
    assert!(!attempt(module, None));
    assert_eq!(module, &before);
}

#[test]
fn discarded_record_prefix_has_independent_authority_and_no_exported_identity() {
    for source in [
        include_str!("../../../tests/native_application_bridge/record_prefix_join.ns"),
        include_str!("../../../tests/native_application_bridge/nested_prefix_join.ns"),
    ] {
        let mut other = crate::frontend::parse_nuis_module(source).unwrap();
        crate::optimize::simplify_nir_module(&mut other);
        reject(&mut other);
    }
    for reversed in [false, true] {
        let mut module = module();
        if reversed {
            module.functions.reverse();
        }
        let layouts = control_values::TypedLayouts::collect(&module);
        let catalog = scalar_helpers::collect_capture_values(&module, &layouts);
        let f = packet(&mut module);
        let scope = f
            .params
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();
        assert!(scalar_joins::nested_prefix::declaration(
            &f.body[1],
            &scope,
            &f.body[2..],
            &catalog,
            &layouts
        )
        .is_none());
        assert!(scalar_joins::nested_prefix::record_declaration(
            &f.body[1],
            &scope,
            &f.body[2..],
            &catalog,
            &layouts
        )
        .is_none());
        let plan = scalar_joins::nested_prefix::discarded_record_declaration(
            &f.body[1],
            &scope,
            &f.body[2..],
            &catalog,
            &layouts,
        )
        .unwrap();
        for arm in [&plan.yes, &plan.no] {
            let [scalar_joins::nested_prefix::Local::Join { join, export }] = arm.locals.as_slice()
            else {
                panic!()
            };
            assert!(!export);
            assert_eq!(join.yes.ty, &scalar_type("Record"));
        }
        assert!(attempt(&mut module, None));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let helpers = module
            .functions
            .iter()
            .filter(|f| f.name.starts_with("__nuis_"))
            .collect::<Vec<_>>();
        assert_eq!(helpers.len(), 3);
        assert_eq!(
            helpers
                .iter()
                .filter(|f| f.return_type.as_ref() == Some(&scalar_type("Record")))
                .count(),
            2
        );
        assert!(helpers
            .iter()
            .all(|f| f.params.iter().all(|p| p.name != "unused" && p.name != "p")));
        let before = module.clone();
        assert!(!attempt(&mut module, None));
        assert_eq!(module, before);
        let carries = control_values::layouts(&module);
        assert!(scalar_helpers::collect_iteration_calls(&module, &carries).contains_key("packet"));
    }
}

#[test]
fn discarded_record_prefix_revokes_missing_or_effectful_dependencies_atomically() {
    for name in ["discarded_decide", "discarded_first", "discarded_last"] {
        let mut module = module();
        let before = module.clone();
        assert!(!attempt(&mut module, Some(name)));
        assert_eq!(module, before);
        module
            .functions
            .iter_mut()
            .find(|f| f.name == name)
            .unwrap()
            .body
            .insert(0, NirStmt::Print(NirExpr::Int(99)));
        reject(&mut module);
    }
}

#[test]
fn discarded_record_prefix_rejects_erased_reads_effects_resources_and_bad_types() {
    for mutation in 0..7 {
        let mut module = module();
        if mutation == 5 {
            module
                .structs
                .iter_mut()
                .find(|s| s.name == "Leaf")
                .unwrap()
                .fields[0]
                .ty
                .is_ref = true;
        }
        let f = packet(&mut module);
        let (_, yes, _) = arms(&mut f.body[1]);
        if mutation == 0 {
            let NirStmt::Let { value, .. } = yes.last_mut().unwrap() else {
                panic!()
            };
            *value = NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var("unused".into())),
                field: "check".into(),
            };
        } else {
            let (condition, left, right) = arms(&mut yes[0]);
            assert!(right.is_empty());
            match mutation {
                1 => left.insert(0, NirStmt::Print(NirExpr::Int(99))),
                2 => *condition = NirExpr::Int(1),
                3 => {
                    let NirStmt::Let { ty, .. } = left.last_mut().unwrap() else {
                        panic!()
                    };
                    *ty = None;
                }
                4 => {
                    let NirStmt::Let { ty, .. } = left.last_mut().unwrap() else {
                        panic!()
                    };
                    *ty = Some(scalar_type("Leaf"));
                }
                5 => {}
                6 => {
                    let NirStmt::Const { name, .. } = left.first_mut().unwrap() else {
                        panic!()
                    };
                    *name = "picked".into();
                }
                _ => unreachable!(),
            }
        }
        reject(&mut module);
    }
}
