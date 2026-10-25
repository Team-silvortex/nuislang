use super::*;

pub(super) fn scope() -> Scope {
    let mut reference = scalar_type("i64");
    reference.is_ref = true;
    Scope::from([
        ("current".into(), scalar_type("i64")),
        ("sibling".into(), scalar_type("i64")),
        ("constant".into(), scalar_type("i64")),
        ("narrow".into(), scalar_type("i32")),
        ("float".into(), scalar_type("f32")),
        ("flag".into(), scalar_type("bool")),
        ("reference".into(), reference),
    ])
}

#[test]
fn print_epoch_certifies_only_initialized_exact_i64_mutable_entry_carries() {
    let scope = scope();
    let updates = scope.keys().cloned().chain(["future".into()]).collect();
    let writable = BTreeSet::from([
        "current".into(),
        "sibling".into(),
        "narrow".into(),
        "float".into(),
        "flag".into(),
        "reference".into(),
        "future".into(),
    ]);
    let epoch = Epoch::enter(
        &scope,
        &writable,
        &updates,
        &control_values::CarryLayouts::new(),
    );
    assert_eq!(
        epoch.seeds,
        BTreeSet::from(["current".into(), "sibling".into()])
    );
    for name in ["constant", "future", "narrow", "float", "flag", "reference"] {
        assert!(epoch
            .reads(
                &NirExpr::Var(name.into()),
                &scope,
                &BTreeSet::new(),
                &updates
            )
            .is_none());
    }
    let mut stale = scope.clone();
    stale.insert("current".into(), scalar_type("i32"));
    assert!(epoch
        .reads(
            &NirExpr::Var("current".into()),
            &stale,
            &BTreeSet::new(),
            &updates
        )
        .is_none());
}

#[test]
fn print_epoch_lends_reads_only_to_one_print_without_mutating_availability() {
    let scope = scope();
    let updates = BTreeSet::from(["current".into(), "sibling".into()]);
    let epoch = Epoch::enter(
        &scope,
        &updates,
        &updates,
        &control_values::CarryLayouts::new(),
    );
    let mut locals = Locals {
        scope,
        writable: updates.clone(),
        available: BTreeSet::new(),
    };
    let catalog = ScalarHelpers::new();
    let layouts = control_values::CarryLayouts::new();
    let printed = NirStmt::Print(NirExpr::Var("current".into()));
    assert!(block(
        &[printed.clone()],
        &mut locals,
        &updates,
        &catalog,
        &layouts,
        validation::LiteralPrints::Scalars,
        &epoch,
        0
    )
    .is_some());
    assert!(locals.available.is_empty());
    assert!(expression(
        &NirExpr::Var("current".into()),
        &locals,
        &updates,
        None,
        &catalog,
        &layouts,
        0
    )
    .is_none());
    for mode in [
        validation::LiteralPrints::Allow,
        validation::LiteralPrints::Reject,
    ] {
        assert!(block(
            &[printed.clone()],
            &mut locals.clone(),
            &updates,
            &catalog,
            &layouts,
            mode,
            &epoch,
            0
        )
        .is_none());
    }
    let copy = NirStmt::Let {
        name: "temporary".into(),
        ty: None,
        value: NirExpr::Var("current".into()),
    };
    assert!(block(
        &[printed, copy],
        &mut locals,
        &updates,
        &catalog,
        &layouts,
        validation::LiteralPrints::Scalars,
        &epoch,
        0
    )
    .is_none());
    assert!(Epoch::default()
        .reads(
            &NirExpr::Var("current".into()),
            &locals.scope,
            &locals.available,
            &updates
        )
        .is_none());
}

fn source(body: &str) -> String {
    format!("mod cpu Main{{@noinline fn observe(value:i64,divisor:i64)->i64{{return value+0/divisor;}} @noinline fn walk(selected:bool,divisor:i64)->i64{{let mut index=0;let mut current=7;let mut sibling=9;while index<2{{{body}}}return current;}}fn main()->i64{{return 0;}}}}")
}

#[test]
fn print_epoch_keeps_assignment_condition_forward_local_and_child_permissions_separate() {
    for body in [
        "print(current);let saved=current;current=current+1;index=index+1;",
        "print(current);if current>0{print(1);}current=current+1;index=index+1;",
        "print(current);sibling=current;current=current+1;index=index+1;",
        "print(future);let future=1;current=current+1;index=index+1;",
        "if selected{let local=1;}print(local);current=current+1;index=index+1;",
        "let mut child=0;while child<1{print(current);current=current+1;child=child+1;}index=index+1;",
    ] {
        let Ok(module) = crate::frontend::parse_nuis_module(&source(body)) else { continue; };
        let layouts = control_values::layouts(&module);
        assert!(!scalar_helpers::collect_iteration_calls(&module, &layouts).contains_key("walk"), "{body}");
    }
}

#[test]
fn print_epoch_executes_leading_trailing_and_branch_versions_in_source_order() {
    for leading in [false, true] {
        for selected in [false, true] {
            let work="if selected{print(observe(current,divisor));}else{print(current+1);}print(current);current=current+2;print(current);";
            let body = if leading {
                format!("index=index+1;{work}")
            } else {
                format!("{work}index=index+1;")
            };
            let source = source(&body).replace(
                "return 0;",
                &format!("return walk({selected},{});", if selected { 1 } else { 0 }),
            );
            let compiled = crate::pipeline::compile_source(&source).unwrap();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&compiled.yir),
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
                    vec![8, 7, 9, 10, 9, 11]
                }
            );
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        }
    }
}
