use super::*;

const SOURCE: &str = "mod cpu Main {
 struct R {value:i64,tag:bool}
 @noinline fn checked(value:i64,div:i64)->i64{return value+1/div;}
 @noinline fn printed(value:i64,div:i64)->i64{print(checked(value,div));return value;}
 @noinline fn wrapper(value:i64,div:i64)->i64{return printed(value,div);}
 @noinline fn bad(value:i64,div:i64)->i64{print(wrapper(value,div));return value;}
 @noinline fn cycle(value:i64)->i64{print(cycle(value));return value;}
 fn main()->i64{return 0;}
}";

fn module() -> NirModule {
    crate::frontend::parse_nuis_module(SOURCE).unwrap()
}

#[test]
fn computed_iteration_prints_require_independently_completed_pure_call_dependencies() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let pure = scalar_helpers::collect_with_layouts(&module, &layouts);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(pure.contains_key("checked"));
    for name in ["printed", "wrapper", "bad", "cycle"] {
        assert!(!pure.contains_key(name));
    }
    for name in ["printed", "wrapper"] {
        assert!(catalog.contains_key(name));
    }
    for name in ["bad", "cycle"] {
        assert!(!catalog.contains_key(name));
    }
    let call = |name: &str| NirExpr::Call {
        callee: name.into(),
        args: vec![NirExpr::Int(7), NirExpr::Int(1)],
    };
    assert!(scalar_helpers::completed_pure_dependencies(
        &call("checked"),
        &catalog
    ));
    for name in ["printed", "wrapper", "bad", "missing"] {
        assert!(!scalar_helpers::completed_pure_dependencies(
            &call(name),
            &catalog
        ));
    }
    let mut reachable = BTreeSet::new();
    scalar_helpers::retain_reachable(
        &module
            .functions
            .iter()
            .find(|f| f.name == "printed")
            .unwrap()
            .body,
        &catalog,
        &mut reachable,
    );
    assert!(reachable.contains("checked"));
}

#[test]
fn computed_iteration_prints_admit_exact_i64_values_without_hidden_logical_or_reference_roots() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut reference = scalar_type("i64");
    reference.is_ref = true;
    let scope = Scope::from([
        ("value".into(), scalar_type("i64")),
        ("flag".into(), scalar_type("bool")),
        ("record".into(), scalar_type("R")),
        ("reference".into(), reference),
        ("narrow".into(), scalar_type("i32")),
        ("float".into(), scalar_type("f32")),
        ("wide".into(), scalar_type("f64")),
    ]);
    for value in [
        NirExpr::Int(-1),
        NirExpr::Var("value".into()),
        NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var("record".into())),
            field: "value".into(),
        },
        NirExpr::Call {
            callee: "checked".into(),
            args: vec![NirExpr::Var("value".into()), NirExpr::Int(1)],
        },
    ] {
        assert!(admit(&value, &scope, &catalog, &layouts).is_some());
    }
    for value in [
        NirExpr::Bool(true),
        NirExpr::Var("flag".into()),
        NirExpr::Var("record".into()),
        NirExpr::Var("reference".into()),
        NirExpr::Var("narrow".into()),
        NirExpr::Var("float".into()),
        NirExpr::Var("wide".into()),
        NirExpr::Var("missing".into()),
        NirExpr::Call {
            callee: "printed".into(),
            args: vec![NirExpr::Int(1), NirExpr::Int(1)],
        },
        NirExpr::CastBoolToI64(Box::new(NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Bool(true)),
            rhs: Box::new(NirExpr::Bool(false)),
        })),
    ] {
        assert!(
            admit(&value, &scope, &catalog, &layouts).is_none(),
            "{value:?}"
        );
    }
}

#[test]
fn computed_iteration_prints_share_one_root_budget_across_branches_and_loops() {
    let mut tree = NirExpr::Int(1);
    for _ in 0..10 {
        tree = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(tree.clone()),
            rhs: Box::new(tree),
        };
    }
    let mut body = vec![
        NirStmt::Print(tree.clone()),
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Print(tree)],
            else_body: vec![],
        },
        NirStmt::While {
            condition: NirExpr::Bool(false),
            body: vec![
                NirStmt::Print(NirExpr::Int(1)),
                NirStmt::Print(NirExpr::Int(1)),
            ],
        },
    ];
    assert!(bounded(&body, &ScalarHelpers::new()));
    body.push(NirStmt::Print(NirExpr::Int(1)));
    assert!(!bounded(&body, &ScalarHelpers::new()));
    for (depth, expected) in [(63, true), (64, false)] {
        let value = (0..depth).fold(NirExpr::Int(1), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Int(1)),
        });
        assert_eq!(
            bounded(&[NirStmt::Print(value)], &ScalarHelpers::new()),
            expected
        );
    }
}

#[test]
fn computed_iteration_prints_preserve_legacy_literal_buffer_and_pure_boundaries() {
    let module = module();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let mut scope = Scope::from([("value".into(), scalar_type("i64"))]);
    let mutations = MutationScope {
        writable: BTreeSet::new(),
        protected: BTreeSet::new(),
    };
    for (types, expected) in [
        (validation::EffectTypes::Buffer(&catalog), false),
        (
            validation::EffectTypes::Values(&catalog, &layouts, validation::LiteralPrints::Reject),
            false,
        ),
        (
            validation::EffectTypes::Values(&catalog, &layouts, validation::LiteralPrints::Allow),
            false,
        ),
        (
            validation::EffectTypes::Values(&catalog, &layouts, validation::LiteralPrints::Scalars),
            true,
        ),
    ] {
        assert_eq!(
            validation::validate_effects(
                &[NirStmt::Print(NirExpr::Var("value".into()))],
                &mut scope,
                &mut BTreeSet::new(),
                types,
                &mutations,
                &mut vec![]
            )
            .is_some(),
            expected
        );
    }
}

#[test]
fn computed_iteration_prints_execute_only_selected_values_with_current_iteration_inputs() {
    for (gate, div, expected) in [(false, 0, vec![80, 8, 9]), (true, 1, vec![80, 9, 8, 10, 9])] {
        let source=format!("mod cpu Main{{@noinline fn checked(value:i64,div:i64)->i64{{return value+1/div;}} @noinline fn walk(value:i64,gate:bool,div:i64)->i64{{print(80);let mut index=0;while index<2{{index=index+1;if gate{{print(checked(value+index,div));}}print(index+value);}}return value;}}fn main()->i64{{return walk(7,{gate},{div});}}}}");
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
        assert_eq!(printed, expected);
        assert_eq!(
            trace
                .events
                .iter()
                .filter(|e| e.contains("cpu.call") && e.contains("] checked("))
                .count(),
            if gate { 2 } else { 0 }
        );
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}

#[test]
fn computed_iteration_prints_keep_scalar_branch_capture_hygiene_and_late_failure_order() {
    for (outer, div) in [(false, 0), (true, 1), (true, 0)] {
        let source=format!("mod cpu Main{{@noinline fn checked(value:i64,div:i64)->i64{{return value+1/div;}} @noinline fn selected(outer:bool,div:i64)->i64{{if outer{{const local:i64=7;print(checked(local,div));}}else{{const local:i64=9;print(local);}}print(83);return 0;}} @noinline fn walk(outer:bool,div:i64)->i64{{let mut index=0;let mut current=0;while index<1{{current=selected(outer,div);index=index+1;}}return current;}}fn main()->i64{{return walk({outer},{div});}}}}");
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&compiled.yir),
            &yir_verify::default_registry(),
        );
        if outer && div == 0 {
            assert!(trace.unwrap_err().contains("zero"));
        } else {
            let trace = trace.unwrap();
            let printed = trace
                .events
                .iter()
                .filter(|e| {
                    e.contains("cpu.print")
                        || (e.contains("cpu.guard_print") && e.contains("if true then print"))
                })
                .map(|e| e.rsplit_once(' ').unwrap().1.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(printed, [if outer { 8 } else { 9 }, 83]);
            assert_eq!(
                trace
                    .events
                    .iter()
                    .filter(|e| e.contains("cpu.call") && e.contains("] checked("))
                    .count(),
                usize::from(outer)
            );
        }
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
}
