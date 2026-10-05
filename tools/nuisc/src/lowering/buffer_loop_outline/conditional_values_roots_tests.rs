use super::*;

#[path = "conditional_values_roots_native_tests.rs"]
mod native;

fn outlined(module: &mut NirModule, full_control: bool) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let carries = control_values::layouts(module);
    let control = scalar_helpers::collect_with_layouts(module, &carries);
    let catalog = scalar_helpers::collect_typed_values(module, &layouts, &control);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    let roots = if full_control {
        BTreeSet::from(["event".into()])
    } else {
        BTreeSet::new()
    };
    outline(module, &catalog, &roots, &layouts, &mut names)
}

fn event(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "event")
        .unwrap()
}

fn source(root: &str, op: &str, gate: bool, divisor: i64, effectful: bool) -> String {
    let selected = format!("gate {op} helper(produce(divisor))");
    let body = match root {
        "let" => format!("let result: bool = {selected}; return result;"),
        "inferred" => format!("let result = {selected}; return result;"),
        "const" => format!("const result: bool = {selected}; return result;"),
        "return" => format!("return {selected};"),
        _ => unreachable!(),
    };
    let effect = if effectful { "print(99);" } else { "" };
    format!(
        "mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        fn produce(divisor: i64) -> Packet {{
            return Packet {{ unused: 10 / divisor, value: divisor }};
        }}
        fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(gate: bool, divisor: i64) -> bool {{ {effect} {body} }}
        fn main() -> i64 {{
            let result = event({gate}, {divisor});
            if result {{ print(11); return 11; }}
            print(19); return 19;
        }}
    }}"
    )
}

#[test]
fn conditional_root_values_preserve_lazy_bindings_constants_and_returns() {
    for root in ["let", "inferred", "const", "return"] {
        for effectful in [false, true] {
            for (op, gate, divisor) in [
                ("&&", false, 0),
                ("||", true, 0),
                ("&&", true, 2),
                ("||", false, 2),
                ("&&", true, -2),
                ("||", false, -2),
                ("&&", true, 0),
                ("||", false, 0),
            ] {
                let source = source(root, op, gate, divisor, effectful);
                let compiled = crate::pipeline::compile_source(&source).unwrap();
                let selected = if op == "&&" { gate } else { !gate };
                let expected = if selected && divisor == 0 {
                    None
                } else {
                    Some(if if selected { divisor > 0 } else { gate } {
                        11
                    } else {
                        19
                    })
                };
                for reversed in [false, true] {
                    let mut yir = compiled.yir.clone();
                    if reversed {
                        yir.nodes.reverse();
                        yir.functions.reverse();
                        for function in &mut yir.functions {
                            function.body_nodes.reverse();
                        }
                    }
                    let result = yir_runtime_host::execute_module_source_with_registry(
                        &crate::render::render_yir(&yir),
                        &yir_verify::default_registry(),
                    );
                    if let Some(expected) = expected {
                        let trace = result.unwrap_or_else(|error| {
                            panic!("{root}/{effectful}/{op}/{gate}/{divisor}: {error}")
                        });
                        let main = yir
                            .functions
                            .iter()
                            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                            .unwrap();
                        assert_eq!(
                            trace.values[&main.result.as_ref().unwrap().node],
                            yir_core::Value::Int(expected),
                            "{root}/{effectful}/{op}/{gate}/{divisor}/reversed={reversed}"
                        );
                        let prints = trace
                            .events
                            .iter()
                            .filter(|event| {
                                event.contains("cpu.print")
                                    || (event.contains("cpu.guard_print_return")
                                        && event.contains(": if true then print "))
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(
                            prints.len(),
                            if effectful { 2 } else { 1 },
                            "{root}/{effectful}/{op}/{gate}/{divisor}: {prints:?}"
                        );
                        if effectful {
                            assert!(prints[0].ends_with(": 99"), "{prints:?}");
                        }
                        let last = prints.last().unwrap();
                        assert!(
                            last.ends_with(&format!(": {expected}"))
                                || last.ends_with(&format!("and return {expected}")),
                            "{prints:?}"
                        );
                    } else {
                        let error = result.unwrap_err();
                        assert!(error.contains("zero"), "{error}");
                    }
                }
                yir_lower_llvm::emit_module(&compiled.yir).unwrap();
            }
        }
    }
}

#[test]
fn conditional_root_values_preserve_selected_versions_nested_bindings_and_early_exits() {
    for shape in ["rebind", "changed", "branch", "return"] {
        for early in [false, true] {
            for (op, gate, divisor) in [
                ("&&", false, 0),
                ("||", true, 0),
                ("&&", true, 2),
                ("||", false, -2),
            ] {
                let selected = format!("gate {op} helper(produce(divisor))");
                let body = match shape {
                    "rebind" => format!("let gate: bool = gate; let gate: bool = {selected}; return gate;"),
                    "changed" => format!("let gate: bool = {}; let gate: bool = {selected}; return gate;", op == "||"),
                    "branch" => format!("let result: bool = true; if outer {{ let result: bool = {selected}; }} else {{ let result: bool = {selected}; }} return result;"),
                    "return" => format!("if outer {{ return {selected}; }} return {selected};"),
                    _ => unreachable!(),
                };
                let base = source("return", op, gate, divisor, true);
                let base = base
                    .replace(
                        &format!("print(99); return {selected};"),
                        &format!("if early {{ return false; }} print(99); {body}"),
                    )
                    .replace(
                        "divisor: i64) -> bool",
                        "divisor: i64, outer: bool, early: bool) -> bool",
                    );
                // Nested fallible return arms still require the pure control
                // route. Effectful parents keep their existing rejection.
                let base = if shape == "return" {
                    base.replace("print(99);", "")
                } else {
                    base
                };
                for outer in [false, true] {
                    let source = base.replace(
                        &format!("event({gate}, {divisor})"),
                        &format!("event({gate}, {divisor}, {outer}, {early})"),
                    );
                    let compiled =
                        crate::pipeline::compile_source(&source).unwrap_or_else(|error| {
                            panic!("{shape}/{early}/{op}/{gate}/{divisor}/{outer}: {error}")
                        });
                    let gate = if shape == "changed" { op == "||" } else { gate };
                    let selected = if op == "&&" { gate } else { !gate };
                    let expected = if !early && if selected { divisor > 0 } else { gate } {
                        11
                    } else {
                        19
                    };
                    for reversed in [false, true] {
                        let mut yir = compiled.yir.clone();
                        if reversed {
                            yir.nodes.reverse();
                            yir.functions.reverse();
                            for function in &mut yir.functions {
                                function.body_nodes.reverse();
                            }
                        }
                        let trace = yir_runtime_host::execute_module_source_with_registry(
                            &crate::render::render_yir(&yir),
                            &yir_verify::default_registry(),
                        )
                        .unwrap_or_else(|error| {
                            panic!("{shape}/{early}/{op}/{gate}/{divisor}/{outer}: {error}")
                        });
                        let main = yir
                            .functions
                            .iter()
                            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                            .unwrap();
                        assert_eq!(
                            trace.values[&main.result.as_ref().unwrap().node],
                            yir_core::Value::Int(expected),
                            "{shape}/{early}/{op}/{gate}/{divisor}/{outer}"
                        );
                        let prints = trace
                            .events
                            .iter()
                            .filter(|event| {
                                event.contains("cpu.print")
                                    || (event.contains("cpu.guard_print_return")
                                        && event.contains(": if true then print "))
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(prints.len(), if early || shape == "return" { 1 } else { 2 });
                        if !early && shape != "return" {
                            assert!(prints[0].ends_with(": 99"), "{prints:?}");
                        }
                    }
                    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
                }
            }
        }
    }
}

#[test]
fn conditional_root_values_retain_effectful_nested_return_speculation_rejection() {
    for op in ["&&", "||"] {
        let original = source("return", op, false, 0, true);
        let selected = format!("print(99); return gate {op} helper(produce(divisor));");
        let admitted = original.replace(
            &selected,
            &format!(
                "print(99); if gate {{ if gate {{ return gate {op} helper(produce(divisor)); }} }} return false;"
            ),
        );
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        let admitted = admitted.replace(
            "if gate { if gate",
            "if gate { let selected: bool = gate; if selected",
        );
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        let admitted = admitted.replace("= gate;", "= gate == true;");
        let compiled = crate::pipeline::compile_source(&admitted).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        // Stored signals admit the pure local predicate, not branch effects.
        for (gate, divisor) in [(false, 0), (true, 2), (true, -2), (true, 0)] {
            let admitted = source("return", op, gate, divisor, true)
                .replace(&selected, &format!("print(99); if gate {{ let selected: bool = helper(produce(divisor)); if selected {{ return gate {op} helper(produce(divisor)); }} }} return false;"))
                .replace("fn produce(", "@noinline fn produce(")
                .replace("fn helper(", "@noinline fn helper(");
            let compiled = crate::pipeline::compile_source(&admitted).unwrap();
            for reversed in [false, true] {
                let mut yir = compiled.yir.clone();
                if reversed {
                    yir.nodes.reverse();
                    yir.edges.reverse();
                    yir.functions.reverse();
                    for function in &mut yir.functions {
                        function.body_nodes.reverse();
                    }
                }
                let execution = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                );
                if gate && divisor == 0 {
                    assert!(execution.unwrap_err().contains("zero"));
                    continue;
                }
                let trace = execution.unwrap();
                let main = yir
                    .functions
                    .iter()
                    .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                    .unwrap();
                assert_eq!(
                    trace.values[&main.result.as_ref().unwrap().node],
                    yir_core::Value::Int(if gate && divisor > 0 { 11 } else { 19 })
                );
                let calls = trace
                    .events
                    .iter()
                    .filter(|e| e.contains("cpu.call_bool") && e.contains("] helper("))
                    .count();
                assert_eq!(
                    calls,
                    usize::from(gate) + usize::from(gate && divisor > 0 && op == "&&")
                );
            }
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        }
        // Keep the formerly rejected outer prefix as positive evidence, while
        // the same effect inside the pure nested tail stays independently vetoed.
        for (gate, divisor) in [(false, 0), (true, 2), (true, -2), (true, 0)] {
            let effectful = source("return", op, gate, divisor, true)
                .replace(&selected, &format!("print(99); if gate {{ print(88); if gate {{ return gate {op} helper(produce(divisor)); }} }} return false;"))
                .replace("fn produce(", "@noinline fn produce(")
                .replace("fn helper(", "@noinline fn helper(");
            let rhs = gate && op == "&&";
            let result = if gate && (op == "||" || divisor > 0) {
                11
            } else {
                19
            };
            let prints = if gate {
                vec![99, 88, result]
            } else {
                vec![99, result]
            };
            super::super::conditional_returns::tests::execute(
                &effectful,
                (!(rhs && divisor == 0)).then_some(result),
                &prints,
                usize::from(rhs),
            );
        }
        let effectful = original.replace(&selected, &format!("print(99); if gate {{ if gate {{ print(88); return gate {op} helper(produce(divisor)); }} }} return false;"));
        let error = crate::pipeline::compile_source(&effectful)
            .err()
            .expect("unproven branch-effect return must stay rejected");
        assert!(
            error.contains("conditional fallible return requires guarded helper lowering"),
            "{error}"
        );
    }
}

#[test]
fn conditional_root_values_keep_inferred_local_kinds_for_following_logical_edges() {
    for effectful in [false, true] {
        for (op, gate, divisor) in [
            ("&&", false, 0),
            ("||", true, 0),
            ("&&", true, 2),
            ("||", false, 2),
            ("&&", true, -2),
            ("||", false, -2),
            ("&&", true, 0),
            ("||", false, 0),
        ] {
            let single = format!("let result = gate {op} helper(produce(divisor)); return result;");
            let chain = format!("let first = gate {op} helper(produce(divisor)); let result = first {op} helper(produce(divisor)); return result;");
            let source = source("inferred", op, gate, divisor, effectful)
                .replace(&single, &chain)
                .replace("fn helper(", "@noinline fn helper(")
                .replace("fn produce(", "@noinline fn produce(");
            let compiled = crate::pipeline::compile_source(&source).unwrap();
            let selected = if op == "&&" { gate } else { !gate };
            for reversed in [false, true] {
                let mut yir = compiled.yir.clone();
                if reversed {
                    yir.nodes.reverse();
                    yir.functions.reverse();
                    for function in &mut yir.functions {
                        function.body_nodes.reverse();
                    }
                }
                let result = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&yir),
                    &yir_verify::default_registry(),
                );
                if selected && divisor == 0 {
                    assert!(result.unwrap_err().contains("zero"));
                    continue;
                }
                let trace = result
                    .unwrap_or_else(|error| panic!("{effectful}/{op}/{gate}/{divisor}: {error}"));
                let first = if selected { divisor > 0 } else { gate };
                let expected = if first { 11 } else { 19 };
                let main = yir
                    .functions
                    .iter()
                    .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                    .unwrap();
                assert_eq!(
                    trace.values[&main.result.as_ref().unwrap().node],
                    yir_core::Value::Int(expected)
                );
                let second_selected = if op == "&&" { first } else { !first };
                let calls = trace
                    .events
                    .iter()
                    .filter(|event| event.contains("cpu.call_bool") && event.contains("] helper("))
                    .count();
                assert_eq!(calls, usize::from(selected) + usize::from(second_selected));
            }
            yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        }
    }
}

#[test]
fn conditional_root_values_keep_kind_loop_nested_effect_and_work_vetoes_unchanged() {
    for root in ["let", "const", "return"] {
        for mutation in 0..12 {
            let mut module =
                crate::frontend::parse_nuis_module(&source(root, "&&", false, 0, false)).unwrap();
            match mutation {
                0..=3 => {
                    let ty = if root == "return" {
                        event(&mut module).return_type.as_mut().unwrap()
                    } else {
                        match &mut event(&mut module).body[0] {
                            NirStmt::Let { ty, .. } => ty.as_mut().unwrap(),
                            NirStmt::Const { ty, .. } => ty,
                            _ => panic!(),
                        }
                    };
                    match mutation {
                        0 => ty.name = "i64".into(),
                        1 => ty.is_ref = true,
                        2 => ty.is_optional = true,
                        3 => ty.generic_args.push(scalar_type("bool")),
                        _ => unreachable!(),
                    }
                }
                4 => {
                    event(&mut module).params[0].ty.is_ref = true;
                }
                5 => module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "helper")
                    .unwrap()
                    .body
                    .insert(0, NirStmt::Print(NirExpr::Int(1))),
                6 => {
                    let body = std::mem::take(&mut event(&mut module).body);
                    event(&mut module).body = vec![NirStmt::While {
                        condition: NirExpr::Bool(false),
                        body,
                    }];
                }
                7..=11 => {
                    let value = match &mut event(&mut module).body[0] {
                        NirStmt::Let { value, .. }
                        | NirStmt::Const { value, .. }
                        | NirStmt::Return(Some(value)) => value,
                        _ => panic!(),
                    };
                    let NirExpr::Binary { lhs, rhs, .. } = value else {
                        panic!()
                    };
                    match mutation {
                        7 => {
                            *lhs = Box::new(NirExpr::Binary {
                                op: NirBinaryOp::Eq,
                                lhs: lhs.clone(),
                                rhs: Box::new(NirExpr::Bool(true)),
                            })
                        }
                        8 => {
                            *rhs = Box::new(NirExpr::Binary {
                                op: NirBinaryOp::And,
                                lhs: rhs.clone(),
                                rhs: Box::new(NirExpr::Bool(true)),
                            })
                        }
                        9 => *rhs = Box::new(NirExpr::Var("missing".into())),
                        10 => {
                            for _ in 0..70 {
                                *rhs = Box::new(NirExpr::Binary {
                                    op: NirBinaryOp::Eq,
                                    lhs: rhs.clone(),
                                    rhs: Box::new(NirExpr::Bool(true)),
                                });
                            }
                        }
                        11 => {
                            for _ in 0..13 {
                                *rhs = Box::new(NirExpr::Binary {
                                    op: NirBinaryOp::Eq,
                                    lhs: rhs.clone(),
                                    rhs: rhs.clone(),
                                });
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
            let before = module.clone();
            if matches!(mutation, 7 | 8) {
                assert_eq!(outlined(&mut module, true).len(), 1, "{root}");
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let candidate = if mutation == 7 {
                    source(root, "&&", false, 0, false).replace("gate &&", "(gate == true) &&")
                } else {
                    source(root, "&&", false, 0, false).replace(
                        "gate && helper(produce(divisor))",
                        "gate && (helper(produce(divisor)) && true)",
                    )
                };
                let compiled = crate::pipeline::compile_source(&candidate).unwrap();
                let trace = yir_runtime_host::execute_module_source_with_registry(
                    &crate::render::render_yir(&compiled.yir),
                    &yir_verify::default_registry(),
                )
                .unwrap();
                let main = compiled
                    .yir
                    .functions
                    .iter()
                    .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                    .unwrap();
                assert_eq!(
                    trace.values[&main.result.as_ref().unwrap().node],
                    yir_core::Value::Int(19)
                );
                yir_lower_llvm::emit_module(&compiled.yir).unwrap();
                continue;
            }
            assert!(outlined(&mut module, true).is_empty(), "{root}/{mutation}");
            assert_eq!(module, before, "{root}/{mutation}");
        }
    }
}

#[test]
fn conditional_root_values_keep_complete_rhs_binding_identity_and_idempotence() {
    for root in ["let", "inferred", "const", "return"] {
        for full_control in [false, true] {
            for op in ["&&", "||"] {
                let mut module =
                    crate::frontend::parse_nuis_module(&source(root, op, false, 0, false)).unwrap();
                let original = module.clone();
                let generated = outlined(&mut module, full_control);
                assert_eq!(generated.len(), 1);
                crate::nir_verify::verify_nir_module(&module).unwrap();
                let helper = module
                    .functions
                    .iter()
                    .find(|f| generated.contains(&f.name))
                    .unwrap();
                assert_eq!(helper.return_type, Some(scalar_type("bool")));
                assert_eq!(helper.params.len(), 2);
                assert_eq!(helper.params[1].name, "divisor");
                let old = &original
                    .functions
                    .iter()
                    .find(|f| f.name == "event")
                    .unwrap()
                    .body[0];
                let value = match old {
                    NirStmt::Let { value, .. }
                    | NirStmt::Const { value, .. }
                    | NirStmt::Return(Some(value)) => value,
                    _ => panic!(),
                };
                let NirExpr::Binary { rhs, .. } = value else {
                    panic!()
                };
                let NirStmt::If {
                    then_body,
                    else_body,
                    ..
                } = &helper.body[0]
                else {
                    panic!()
                };
                let (selected, skipped) = if op == "&&" {
                    (then_body, else_body)
                } else {
                    (else_body, then_body)
                };
                assert_eq!(selected, &vec![NirStmt::Return(Some((**rhs).clone()))]);
                assert_eq!(
                    skipped,
                    &vec![NirStmt::Return(Some(NirExpr::Bool(op == "||")))]
                );
                let new = &event(&mut module).body[0];
                match (old, new) {
                    (
                        NirStmt::Let { name, ty, .. },
                        NirStmt::Let {
                            name: new_name,
                            ty: new_ty,
                            value,
                        },
                    ) => {
                        assert_eq!((name, ty), (new_name, new_ty));
                        assert!(
                            matches!(value, NirExpr::Call { callee, args } if generated.contains(callee) && args[0] == NirExpr::Var("gate".into()))
                        );
                    }
                    (
                        NirStmt::Const { name, ty, .. },
                        NirStmt::Const {
                            name: new_name,
                            ty: new_ty,
                            value,
                        },
                    ) => {
                        assert_eq!((name, ty), (new_name, new_ty));
                        assert!(
                            matches!(value, NirExpr::Call { callee, .. } if generated.contains(callee))
                        );
                    }
                    (NirStmt::Return(_), NirStmt::Return(Some(NirExpr::Call { callee, .. }))) => {
                        assert!(generated.contains(callee));
                    }
                    _ => panic!("changed statement kind"),
                }
                for function in &original.functions {
                    if function.name != "event" {
                        assert_eq!(
                            module.functions.iter().find(|f| f.name == function.name),
                            Some(function)
                        );
                    }
                }
                let before = module.clone();
                assert!(outlined(&mut module, full_control).is_empty());
                assert_eq!(module, before);
            }
        }
    }
}
