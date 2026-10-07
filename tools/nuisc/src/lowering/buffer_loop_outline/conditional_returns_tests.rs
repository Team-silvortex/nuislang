use super::*;

#[path = "conditional_returns_effect_boundary_tests.rs"]
mod effect_boundaries;
#[path = "conditional_returns_native_tests.rs"]
pub(super) mod native;
pub(super) use effect_boundaries::assert_selected_effect_not_pure;

pub(super) fn source(
    shape: &str,
    op: &str,
    outer: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let value = format!("gate {op} helper(produce(divisor))");
    let branch = match shape {
        "then" => format!("if outer {{ return {value}; }}"),
        "else" => format!("if outer {{ }} else {{ return {value}; }}"),
        "both" => {
            format!("if outer {{ return {value}; }} else {{ return helper(produce(divisor)); }}")
        }
        _ => unreachable!(),
    };
    format!(
        "mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(divisor: i64) -> Packet {{
            return Packet {{ unused: 10 / divisor, value: divisor }};
        }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, gate: bool, divisor: i64, early: bool) -> bool {{
            if early {{ return false; }}
            print(99); {branch} print(77); return false;
        }}
        fn main() -> i64 {{
            let result = event({outer}, {gate}, {divisor}, {early});
            if result {{ print(11); return 11; }}
            print(19); return 19;
        }}
    }}"
    )
}

pub(in crate::lowering::buffer_loop_outline) fn execute(
    source: &str,
    expected: Option<i64>,
    prints: &[i64],
    calls: usize,
) {
    let compiled =
        crate::pipeline::compile_source(source).unwrap_or_else(|error| panic!("{error}\n{source}"));
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
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        );
        if let Some(expected) = expected {
            let trace = trace.unwrap_or_else(|error| panic!("{error}\n{source}"));
            let main = yir
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap();
            assert_eq!(
                trace.values[&main.result.as_ref().unwrap().node],
                yir_core::Value::Int(expected)
            );
            let observed = trace
                .events
                .iter()
                .filter(|event| {
                    event.contains("cpu.print")
                        || (event.contains("cpu.guard_print ")
                            && event.contains(": if true then print "))
                        || (event.contains("cpu.guard_print_return")
                            && event.contains(": if true then print "))
                })
                .collect::<Vec<_>>();
            assert_eq!(observed.len(), prints.len(), "{observed:?}\n{source}");
            for (event, printed) in observed.iter().zip(prints) {
                assert!(
                    event.ends_with(&format!(": {printed}"))
                        || event.ends_with(&format!("then print {printed}"))
                        || event.ends_with(&format!("and return {printed}")),
                    "{event}\n{source}"
                );
            }
            let invocations = trace
                .events
                .iter()
                .filter(|event| event.contains("cpu.call_bool") && event.contains("] helper("))
                .count();
            assert_eq!(invocations, calls, "{source}\n{:?}", trace.events);
        } else {
            let error = trace.unwrap_err();
            assert!(error.contains("zero"), "{error}\n{source}");
        }
    }
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
}

#[test]
fn conditional_returns_preserve_parent_effects_two_guards_and_actual_exits() {
    for shape in ["then", "else", "both"] {
        for outer in [false, true] {
            for early in [false, true] {
                for (op, gate, divisor) in [
                    ("&&", false, 0),
                    ("||", true, 0),
                    ("&&", true, 2),
                    ("||", false, -2),
                    ("&&", true, 0),
                    ("||", false, 0),
                ] {
                    let entered = !early
                        && match shape {
                            "then" => outer,
                            "else" => !outer,
                            "both" => true,
                            _ => unreachable!(),
                        };
                    let rhs = entered
                        && if shape == "both" && !outer {
                            true
                        } else if op == "&&" {
                            gate
                        } else {
                            !gate
                        };
                    let source = source(shape, op, outer, gate, divisor, early);
                    if rhs && divisor == 0 {
                        execute(&source, None, &[], 0);
                        continue;
                    }
                    let value = entered && if rhs { divisor > 0 } else { gate };
                    let result = if value { 11 } else { 19 };
                    let mut prints = Vec::new();
                    if !early {
                        prints.push(99);
                        if !entered {
                            prints.push(77);
                        }
                    }
                    prints.push(result);
                    execute(&source, Some(result), &prints, usize::from(rhs));
                }
            }
        }
    }
}

#[test]
fn conditional_returns_keep_i64_checked_work_and_current_binding_versions() {
    for shape in ["then", "else", "both"] {
        for outer in [false, true] {
            for divisor in [0, 2, -2] {
                let branch = match shape {
                    "then" => "if outer { return 10 / divisor; }",
                    "else" => "if outer { } else { return 10 / divisor; }",
                    "both" => "if outer { return 10 / divisor; } else { return 20 / divisor; }",
                    _ => unreachable!(),
                };
                let source = format!(
                    "mod cpu Main {{
                    @noinline fn event(outer: bool, divisor: i64) -> i64 {{
                        print(99); let divisor: i64 = divisor + 2;
                        {branch} print(77); return 19;
                    }}
                    fn main() -> i64 {{ return event({outer}, {divisor}); }}
                }}"
                );
                let divisor = divisor + 2;
                let selected = shape == "both" || if shape == "then" { outer } else { !outer };
                let expected = if selected && divisor == 0 {
                    None
                } else if !selected {
                    Some(19)
                } else {
                    Some(if shape == "both" && !outer { 20 } else { 10 } / divisor)
                };
                let prints = if selected { vec![99] } else { vec![99, 77] };
                execute(&source, expected, &prints, 0);
            }
        }
    }
}

#[test]
fn conditional_returns_keep_current_inferred_bool_versions_after_rebinding() {
    for declaration in ["let gate: bool", "let gate"] {
        for op in ["&&", "||"] {
            for gate in [false, true] {
                for outer in [false, true] {
                    for divisor in [0, 2, -2] {
                        let source = source("then", op, outer, gate, divisor, false).replace(
                            "print(99);",
                            &format!("print(99); {declaration} = gate == false;"),
                        );
                        let rhs = outer && if op == "&&" { !gate } else { gate };
                        let value = outer && if rhs { divisor > 0 } else { !gate };
                        let result = if value { 11 } else { 19 };
                        let prints = if outer {
                            vec![99, result]
                        } else {
                            vec![99, 77, result]
                        };
                        execute(
                            &source,
                            (!(rhs && divisor == 0)).then_some(result),
                            &prints,
                            usize::from(rhs),
                        );
                    }
                }
            }
        }
    }
}

pub(super) fn outline_test(module: &mut NirModule) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let carries = control_values::layouts(module);
    let control = scalar_helpers::collect_with_layouts(module, &carries);
    let catalog = scalar_helpers::collect_typed_values(module, &layouts, &control);
    let roots = scalar_helpers::control_roots(module, &carries, &control);
    let mut names = module
        .functions
        .iter()
        .map(|f| f.name.clone())
        .chain(module.structs.iter().map(|s| s.name.clone()))
        .chain(module.enums.iter().map(|e| e.name.clone()))
        .chain(module.externs.iter().map(|f| f.name.clone()))
        .collect();
    outline(module, &catalog, &roots, &layouts, &mut names)
}

#[test]
fn conditional_returns_preserve_complete_expressions_parent_tail_and_idempotence() {
    for shape in ["then", "else", "both"] {
        let mut module =
            crate::frontend::parse_nuis_module(&source(shape, "&&", true, true, 2, false)).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), 1);
        let original = before.functions.iter().find(|f| f.name == "event").unwrap();
        let after = module.functions.iter().find(|f| f.name == "event").unwrap();
        assert_eq!(&after.body[..2], &original.body[..2]);
        assert_eq!(
            &after.body[after.body.len() - 2..],
            &original.body[original.body.len() - 2..]
        );
        let helper = module
            .functions
            .iter()
            .find(|f| generated.contains(&f.name))
            .unwrap();
        let NirStmt::If {
            then_body,
            else_body,
            ..
        } = &original.body[2]
        else {
            panic!()
        };
        let NirStmt::If {
            then_body: yes,
            else_body: no,
            ..
        } = &helper.body[0]
        else {
            panic!()
        };
        for (old, new) in [(then_body, yes), (else_body, no)] {
            if old.is_empty() {
                assert_eq!(new, &vec![NirStmt::Return(Some(NirExpr::Bool(false)))]);
            } else {
                assert_eq!(old, new);
            }
        }
        assert_eq!(
            helper
                .params
                .iter()
                .skip(1)
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["divisor", "gate"]
        );
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let once = module.clone();
        assert!(outline_test(&mut module).is_empty());
        assert_eq!(module, once);
    }
}

#[test]
fn conditional_returns_keep_loop_nested_effect_resource_kind_and_work_vetoes() {
    let original = source("then", "&&", true, true, 2, false);
    let selected = "if outer { return gate && helper(produce(divisor)); }";
    let admitted = original.replace(selected, "if outer { let selected: bool = gate; if selected { return gate && helper(produce(divisor)); } }");
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let admitted = original.replace(selected, "if outer { let selected: bool = helper(produce(divisor)); if selected { return gate && helper(produce(divisor)); } }");
    let compiled = crate::pipeline::compile_source(&admitted).unwrap();
    yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let admitted = original.replace(
        selected,
        "if outer == true { return helper(produce(divisor)); }",
    );
    execute(&admitted, Some(11), &[99, 11], 1);
    let admitted = original.replace(
        selected,
        "if outer { return gate && (gate || helper(produce(divisor))); }",
    );
    execute(&admitted, Some(11), &[99, 11], 0);
    // Former leading-print vetoes are now independently admitted parent effects.
    let admitted = original.replace(
        selected,
        "if outer { print(88); return helper(produce(divisor)); }",
    );
    execute(&admitted, Some(11), &[99, 88, 11], 1);
    let admitted = original.replace(
        selected,
        "if outer { return helper(produce(divisor)); } else { print(88); }",
    );
    execute(&admitted, Some(11), &[99, 11], 1);
    for replacement in [
        "while outer { return gate && helper(produce(divisor)); }",
        "if outer { let gate: bool = helper(produce(divisor)); return gate; }",
    ] {
        let mut module =
            crate::frontend::parse_nuis_module(&original.replace(selected, replacement)).unwrap();
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{replacement}");
        assert_eq!(module, before);
    }
    for mutation in [
        "borrow", "optional", "generic", "effect", "result", "async", "capture", "depth", "work",
        "pure",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&original).unwrap();
        match mutation {
            "effect" => module
                .functions
                .iter_mut()
                .find(|f| f.name == "helper")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(88))),
            "pure" => {
                let event = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "event")
                    .unwrap();
                event.body.retain(|stmt| !matches!(stmt, NirStmt::Print(_)));
            }
            _ => {
                let event = module
                    .functions
                    .iter_mut()
                    .find(|f| f.name == "event")
                    .unwrap();
                match mutation {
                    "borrow" => event.params[0].ty.is_ref = true,
                    "optional" => event.params[0].ty.is_optional = true,
                    "generic" => event.params[0].ty.generic_args.push(scalar_type("i64")),
                    "result" => event.return_type = Some(scalar_type("i64")),
                    "async" => event.is_async = true,
                    "capture" => event.params[1].ty.is_ref = true,
                    "depth" | "work" => {
                        let mut value = NirExpr::Int(1);
                        for _ in 0..if mutation == "depth" { 70 } else { 13 } {
                            let lhs = if mutation == "depth" {
                                NirExpr::Int(1)
                            } else {
                                value.clone()
                            };
                            value = NirExpr::Binary {
                                op: NirBinaryOp::Add,
                                lhs: Box::new(lhs),
                                rhs: Box::new(value),
                            };
                        }
                        let NirStmt::If { then_body, .. } = &mut event.body[2] else {
                            panic!()
                        };
                        *then_body = vec![NirStmt::Return(Some(NirExpr::Binary {
                            op: NirBinaryOp::Gt,
                            lhs: Box::new(value),
                            rhs: Box::new(NirExpr::Int(0)),
                        }))];
                    }
                    _ => unreachable!(),
                }
            }
        };
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
}
