use super::*;

fn layouts() -> control_values::CarryLayouts {
    control_values::CarryLayouts::from([
        (
            "Pair".into(),
            vec![
                ("total".into(), scalar_type("i64")),
                ("flag".into(), scalar_type("bool")),
            ],
        ),
        ("Outer".into(), vec![("inner".into(), scalar_type("Pair"))]),
    ])
}

fn projection(name: &str, path: &[&str]) -> NirExpr {
    path.iter().fold(NirExpr::Var(name.into()), |base, field| {
        NirExpr::FieldAccess {
            base: Box::new(base),
            field: (*field).into(),
        }
    })
}

#[test]
fn field_print_epoch_checks_exact_entry_shape_and_original_i64_path() {
    let layouts = layouts();
    let scope = Scope::from([("live".into(), scalar_type("Outer"))]);
    let changed = BTreeSet::from(["live".into()]);
    let epoch = Epoch::enter(&scope, &changed, &changed, &layouts);
    let value = projection("live", &["inner", "total"]);
    assert_eq!(
        epoch.field(&value, &scope, &layouts),
        Some(scalar_type("i64"))
    );
    assert!(epoch
        .reads(&value, &scope, &BTreeSet::new(), &changed)
        .unwrap()
        .is_empty());
    for path in [
        vec![],
        vec!["inner"],
        vec!["inner", "flag"],
        vec!["total"],
        vec!["inner", "missing"],
    ] {
        assert!(epoch
            .field(&projection("live", &path), &scope, &layouts)
            .is_none());
    }
    assert!(epoch
        .field(&projection("future", &["inner", "total"]), &scope, &layouts)
        .is_none());
    for mutate in 0..4 {
        let mut stale = layouts.clone();
        match mutate {
            0 => stale.get_mut("Pair").unwrap()[0].1 = scalar_type("i32"),
            1 => stale.get_mut("Pair").unwrap()[1].0 = "renamed".into(),
            2 => {
                stale.insert("Other".into(), stale["Pair"].clone());
                stale.get_mut("Outer").unwrap()[0].1 = scalar_type("Other");
            }
            _ => stale.get_mut("Pair").unwrap().reverse(),
        }
        assert!(epoch.field(&value, &scope, &stale).is_none());
    }
    let mut stale_scope = scope.clone();
    stale_scope.insert("live".into(), scalar_type("Pair"));
    assert!(epoch.field(&value, &stale_scope, &layouts).is_none());
}

#[test]
fn field_print_epoch_rejects_nonowned_malformed_and_unready_child_roots() {
    let layouts = layouts();
    let changed = BTreeSet::from(["live".into()]);
    for variant in 0..5 {
        let mut ty = scalar_type("Outer");
        match variant {
            0 => ty.is_ref = true,
            1 => ty.is_optional = true,
            2 => ty.generic_args.push(scalar_type("i64")),
            3 => ty.name = "Unknown".into(),
            _ => ty.name = "i64".into(),
        }
        let scope = Scope::from([("live".into(), ty)]);
        assert!(!Epoch::enter(&scope, &changed, &changed, &layouts)
            .fields
            .contains("live", &scope));
    }
    let scope = Scope::from([("live".into(), scalar_type("Outer"))]);
    for variant in 0..4 {
        let mut invalid = layouts.clone();
        match variant {
            0 => invalid.get_mut("Pair").unwrap()[0].1.is_ref = true,
            1 => {
                let field = invalid["Pair"][0].clone();
                invalid.get_mut("Pair").unwrap().push(field);
            }
            2 => invalid.get_mut("Pair").unwrap()[0].1 = scalar_type("Outer"),
            _ => invalid.get_mut("Pair").unwrap()[0].1 = scalar_type("Resource"),
        }
        assert!(!Epoch::enter(&scope, &changed, &changed, &invalid)
            .fields
            .contains("live", &scope));
    }
    assert!(!Epoch::enter(&scope, &BTreeSet::new(), &changed, &layouts)
        .fields
        .contains("live", &scope));
    assert!(!Epoch::enter(&Scope::new(), &changed, &changed, &layouts)
        .fields
        .contains("live", &scope));
    assert!(
        !Epoch::enter_child(&scope, &changed, &changed, &BTreeSet::new(), &layouts)
            .fields
            .contains("live", &scope)
    );
    assert!(
        Epoch::enter_child(&scope, &changed, &changed, &changed, &layouts)
            .fields
            .contains("live", &scope)
    );
}

#[test]
fn field_print_epoch_never_launders_whole_records_or_ordinary_readiness() {
    let layouts = layouts();
    let mut locals = Locals {
        scope: Scope::from([("live".into(), scalar_type("Outer"))]),
        writable: BTreeSet::from(["live".into()]),
        available: BTreeSet::new(),
    };
    let changed = locals.writable.clone();
    let epoch = Epoch::enter(&locals.scope, &changed, &changed, &layouts);
    let module = crate::frontend::parse_nuis_module("mod cpu Main { struct Pair{total:i64,flag:bool} struct Outer{inner:Pair} fn whole(value:Outer)->i64{return value.inner.total;} fn main()->i64{return 0;} }").unwrap();
    let catalog = scalar_helpers::collect_with_layouts(&module, &layouts);
    assert!(catalog.contains_key("whole"));
    let value = projection("live", &["inner", "total"]);
    assert!(block(
        &[NirStmt::Print(value.clone())],
        &mut locals,
        &changed,
        &catalog,
        &layouts,
        validation::LiteralPrints::Scalars,
        &epoch,
        0
    )
    .is_some());
    assert!(locals.available.is_empty());
    assert!(expression(&value, &locals, &changed, None, &catalog, &layouts, 0).is_none());
    for value in [
        NirExpr::Call {
            callee: "whole".into(),
            args: vec![NirExpr::Var("live".into())],
        },
        NirExpr::Call {
            callee: "whole".into(),
            args: vec![NirExpr::StructLiteral {
                type_name: "Outer".into(),
                type_args: vec![],
                fields: vec![("inner".into(), projection("live", &["inner"]))],
            }],
        },
    ] {
        assert!(block(
            &[NirStmt::Print(value)],
            &mut locals,
            &changed,
            &catalog,
            &layouts,
            validation::LiteralPrints::Scalars,
            &epoch,
            0
        )
        .is_none());
    }
    assert!(locals.available.is_empty());
    assert!(!Epoch::enter_child(
        &locals.scope,
        &changed,
        &changed,
        &locals.available,
        &layouts
    )
    .fields
    .contains("live", &locals.scope));
}

fn source(body: &str, selected: bool, divisor: i64) -> String {
    format!("mod cpu Main{{struct Pair{{total:i64,flag:bool}} struct Outer{{inner:Pair}} @noinline fn observe(v:i64,d:i64)->i64{{return v+0/d;}} @noinline fn whole(v:Outer)->i64{{return v.inner.total;}} @noinline fn walk(selected:bool,divisor:i64)->i64{{let mut index=0;let mut live=Outer{{inner:Pair{{total:7,flag:true}}}};while index<2{{{body}}}return live.inner.total;}}fn main()->i64{{return walk({selected},{divisor});}}}}")
}

#[test]
fn field_print_epoch_source_keeps_assignment_condition_and_ancestor_boundaries() {
    for body in [
        "print(live.inner.total);let saved=live.inner.total;live=live;index=index+1;",
        "print(live.inner.total);if live.inner.total>0{print(1);}live=live;index=index+1;",
        "print(live.inner.total);print(whole(live));live=live;index=index+1;",
        "let mut child=0;while child<1{print(live.inner.total);live=live;child=child+1;}index=index+1;",
        "print(live.inner.total);let mut child=0;while child<1{print(live.inner.total);live=live;child=child+1;}index=index+1;",
    ] {
        let module = crate::frontend::parse_nuis_module(&source(body, false, 0)).unwrap();
        let layouts = control_values::layouts(&module);
        assert!(!scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("walk"), "{body}");
    }
}

#[test]
fn field_print_epoch_raw_nir_keeps_old_new_fields_and_skips_poisoned_guards() {
    for leading in [false, true] {
        for selected in [false, true] {
            let effect="if selected{print(observe(live.inner.total,divisor));}print(live.inner.total);live=Outer{inner:Pair{total:live.inner.total+2,flag:true}};print(live.inner.total);";
            let body = if leading {
                format!("index=index+1;{effect}")
            } else {
                format!("{effect}index=index+1;")
            };
            let module = crate::frontend::parse_nuis_module(&source(
                &body,
                selected,
                if selected { 1 } else { 0 },
            ))
            .unwrap();
            let layouts = control_values::layouts(&module);
            assert!(scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("walk"));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            // This entry owns outlining and its record transport metadata.
            let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            )
            .unwrap();
            let printed = trace
                .events
                .iter()
                .filter(|e| {
                    e.contains("cpu.print")
                        || (e.contains("cpu.guard_print") && e.contains("if true then print"))
                })
                .map(|e| e.rsplit_once(' ').unwrap().1.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                printed,
                if selected {
                    vec![7, 7, 9, 9, 9, 11]
                } else {
                    vec![7, 9, 9, 11]
                }
            );
            yir_lower_llvm::emit_module(&yir).unwrap();
        }
    }
}
