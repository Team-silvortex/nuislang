use super::*;

#[test]
fn child_print_epoch_requires_ready_exact_mutable_changed_entry_names() {
    let scope = tests::scope();
    let changed = scope.keys().cloned().chain(["future".into()]).collect();
    let writable = BTreeSet::from([
        "current".into(),
        "sibling".into(),
        "flag".into(),
        "reference".into(),
        "narrow".into(),
        "float".into(),
        "future".into(),
    ]);
    let available = BTreeSet::from([
        "current".into(),
        "constant".into(),
        "flag".into(),
        "reference".into(),
        "narrow".into(),
        "float".into(),
        "future".into(),
    ]);
    let epoch = Epoch::enter_child(
        &scope,
        &writable,
        &changed,
        &available,
        &control_values::CarryLayouts::new(),
    );
    assert_eq!(epoch.seeds, BTreeSet::from(["current".into()]));
    assert!(Epoch::enter(
        &scope,
        &writable,
        &changed,
        &control_values::CarryLayouts::new()
    )
    .seeds
    .contains("sibling"));
    assert!(
        !epoch.seeds.contains("sibling"),
        "parent grants are not inherited"
    );
    assert!(Epoch::enter_child(
        &scope,
        &writable,
        &BTreeSet::new(),
        &available,
        &control_values::CarryLayouts::new()
    )
    .seeds
    .is_empty());
}

#[test]
fn child_print_epoch_lends_no_readiness_to_assignments_or_grandchildren() {
    let scope = tests::scope();
    let changed = BTreeSet::from(["current".into(), "sibling".into()]);
    let mut locals = Locals {
        scope,
        writable: changed.clone(),
        available: BTreeSet::from(["current".into()]),
    };
    let epoch = Epoch::enter_child(
        &locals.scope,
        &locals.writable,
        &changed,
        &locals.available,
        &control_values::CarryLayouts::new(),
    );
    locals.available.clear();
    let catalog = ScalarHelpers::new();
    let layouts = control_values::CarryLayouts::new();
    assert!(block(
        &[NirStmt::Print(NirExpr::Var("current".into()))],
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
    assert!(Epoch::enter_child(
        &locals.scope,
        &locals.writable,
        &changed,
        &locals.available,
        &control_values::CarryLayouts::new()
    )
    .seeds
    .is_empty());
    assert!(expression(
        &NirExpr::Var("current".into()),
        &locals,
        &changed,
        None,
        &catalog,
        &layouts,
        0
    )
    .is_none());
    let mut stale = locals.scope.clone();
    stale.insert("current".into(), scalar_type("i32"));
    assert!(epoch
        .reads(
            &NirExpr::Var("current".into()),
            &stale,
            &locals.available,
            &changed
        )
        .is_none());
}

fn source(body: &str, selected: bool, divisor: i64) -> String {
    format!("mod cpu Main{{@noinline fn observe(value:i64,divisor:i64)->i64{{return value+0/divisor;}} @noinline fn walk(selected:bool,divisor:i64)->i64{{let mut outer=0;let mut ancestor=7;while outer<2{{{body}}}return outer;}}fn main()->i64{{return walk({selected},{divisor});}}}}")
}

#[test]
fn child_print_epoch_keeps_unavailable_ancestors_and_ordinary_reads_closed() {
    for body in [
        "let mut child=0;while child<1{print(ancestor);ancestor=ancestor+1;child=child+1;}outer=outer+1;",
        "print(ancestor);let mut child=0;while child<1{print(ancestor);ancestor=ancestor+1;child=child+1;}outer=outer+1;",
        "let mut local=7;let mut child=0;while child<1{print(local);let saved=local;local=local+1;child=child+1;}outer=outer+1;",
        "let mut local=7;let mut child=0;while child<1{print(local);if local>0{print(1);}local=local+1;child=child+1;}outer=outer+1;",
        "let mut local=7;let mut child=0;while child<1{print(local);let mut nested=0;while nested<1{print(local);local=local+1;nested=nested+1;}child=child+1;}outer=outer+1;",
    ] {
        let module=crate::frontend::parse_nuis_module(&source(body,false,0)).unwrap();
        let layouts=control_values::layouts(&module);
        assert!(!scalar_helpers::collect_iteration_calls(&module,&layouts).contains_key("walk"),"{body}");
    }
}

#[test]
fn child_print_epoch_executes_fresh_entries_for_leading_and_trailing_loops() {
    for parent_leading in [false, true] {
        for child_leading in [false, true] {
            for selected in [false, true] {
                let effect="if selected{print(observe(local,divisor));}else{print(local+1);}print(local);print(child);local=local+2;print(local);";
                let child = if child_leading {
                    format!("child=child+1;{effect}")
                } else {
                    format!("{effect}child=child+1;")
                };
                let middle =
                    format!("let mut local=7+outer;let mut child=0;while child<2{{{child}}}");
                let parent = if parent_leading {
                    format!("outer=outer+1;{middle}")
                } else {
                    format!("{middle}outer=outer+1;")
                };
                let text = source(&parent, selected, if selected { 1 } else { 0 });
                let compiled = crate::pipeline::compile_source(&text).unwrap();
                let trace = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&compiled.yir),
                    &yir_verify::default_registry(),
                )
                .unwrap();
                let printed = prints(&trace.events);
                let mut expected = Vec::new();
                for parent in 0..2 {
                    let mut local = 7 + parent + i64::from(parent_leading);
                    for child in 0..2 {
                        expected.extend([
                            local + i64::from(!selected),
                            local,
                            child + i64::from(child_leading),
                            local + 2,
                        ]);
                        local += 2;
                    }
                }
                assert_eq!(printed, expected);
                let calls = trace
                    .events
                    .iter()
                    .filter(|e| e.contains("cpu.call") && e.contains("] observe("))
                    .count();
                assert_eq!(calls, if selected { 4 } else { 0 });
                yir_lower_llvm::emit_module(&compiled.yir).unwrap();
            }
        }
    }
}

fn prints(events: &[String]) -> Vec<i64> {
    events
        .iter()
        .filter(|e| {
            e.contains("cpu.print")
                || (e.contains("cpu.guard_print") && e.contains("if true then print"))
        })
        .map(|e| e.rsplit_once(' ').unwrap().1.parse().unwrap())
        .collect()
}

#[test]
fn child_print_epoch_executes_unsimplified_nir_without_hoisting_poisoned_children() {
    for (trips, expected) in [(0, Some(vec![])), (1, None)] {
        let body=format!("let mut local=7;let mut child=0;while child<{trips}{{print(observe(local,divisor));local=local+1;child=child+1;}}outer=outer+1;");
        let mut module = crate::frontend::parse_nuis_module(&source(&body, true, 0)).unwrap();
        outline_buffer_loops(&mut module).unwrap();
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let yir = crate::lowering::bootstrap::lower_nir_to_yir_builtin_cpu(&module).unwrap();
        let result = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        );
        match expected {
            Some(expected) => assert_eq!(prints(&result.unwrap().events), expected),
            None => assert!(result.unwrap_err().contains("zero")),
        }
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}
