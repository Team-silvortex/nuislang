use super::super::tests::{execute, outline_test, source as simple_source};
use super::*;

#[path = "conditional_returns_continuation_native_tests.rs"]
mod native;

fn continuing(kind: &str) -> &'static str {
    match kind {
        "scalar" => "let ignored = 10 / divisor;",
        "record" => "let packet = produce(divisor); let ignored = packet.value;",
        "calls" => "let packet = produce(divisor); let ignored = helper(packet);",
        "hygiene" => "let __nuis_return_condition_0 = produce(divisor); const __nuis_return_exit_0: bool = gate;",
        "tree" => "if other { let packet = produce(divisor); let ignored = helper(packet); } else { const ignored: i64 = 10 / divisor; }",
        _ => unreachable!(),
    }
}

fn branch(shape: &str, returned: &str, continuation: &str) -> String {
    let yes = format!("if nested {{ {returned} }} else {{ {continuation} }}");
    let no = format!("if nested {{ {continuation} }} else {{ {returned} }}");
    match shape {
        "then" => format!("if outer {{ {yes} }}"),
        "else" => format!("if outer {{ }} else {{ {yes} }}"),
        "both" => format!("if outer {{ {yes} }} else {{ {no} }}"),
        "outer-then" => format!("if outer {{ {returned} }} else {{ {continuation} }}"),
        "outer-else" => format!("if outer {{ {continuation} }} else {{ {returned} }}"),
        _ => unreachable!(),
    }
}

fn paths(shape: &str, outer: bool, nested: bool) -> (bool, bool) {
    match shape {
        "then" => (outer && nested, outer && !nested),
        "else" => (!outer && nested, !outer && !nested),
        "both" => (outer == nested, outer != nested),
        "outer-then" => (outer, !outer),
        "outer-else" => (!outer, outer),
        _ => unreachable!(),
    }
}

#[allow(clippy::too_many_arguments)]
fn source(
    shape: &str,
    kind: &str,
    op: &str,
    outer: bool,
    nested: bool,
    other: bool,
    gate: bool,
    divisor: i64,
    early: bool,
) -> String {
    let old = format!("if outer {{ return gate {op} helper(produce(divisor)); }}");
    let returned = format!("return gate {op} helper(produce(divisor));");
    simple_source("then", op, outer, gate, divisor, early)
        .replace(
            "early: bool) -> bool",
            "early: bool, nested: bool, other: bool) -> bool",
        )
        .replace(
            &format!("event({outer}, {gate}, {divisor}, {early})"),
            &format!("event({outer}, {gate}, {divisor}, {early}, {nested}, {other})"),
        )
        .replace(&old, &branch(shape, &returned, continuing(kind)))
}

#[test]
fn conditional_return_continuations_keep_selected_pure_work_and_parent_suffix_order() {
    for shape in ["then", "else", "both", "outer-then", "outer-else"] {
        for kind in ["record", "calls"] {
            for outer in [false, true] {
                for nested in [false, true] {
                    for early in [false, true] {
                        for (op, gate, divisor) in [
                            ("&&", false, 0),
                            ("||", true, 0),
                            ("&&", true, 2),
                            ("||", false, -2),
                            ("&&", true, 0),
                            ("||", false, 0),
                        ] {
                            let (returned, continuation) = paths(shape, outer, nested);
                            let (returned, continuation) =
                                (!early && returned, !early && continuation);
                            let rhs = returned && if op == "&&" { gate } else { !gate };
                            let trapped = divisor == 0 && (rhs || continuation);
                            let value = returned && if rhs { divisor > 0 } else { gate };
                            let result = if value { 11 } else { 19 };
                            let mut prints = Vec::new();
                            if !early {
                                prints.push(99);
                                if !returned {
                                    prints.push(77);
                                }
                            }
                            prints.push(result);
                            execute(
                                &source(
                                    shape, kind, op, outer, nested, false, gate, divisor, early,
                                ),
                                (!trapped).then_some(result),
                                &prints,
                                usize::from(rhs) + usize::from(continuation && kind == "calls"),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn integer_source(shape: &str, outer: bool, nested: bool, divisor: i64, declared: &str) -> String {
    let binding = |name: &str| {
        if declared == "const" {
            format!("const {name}: i64")
        } else {
            format!("let {name}")
        }
    };
    let returned = format!("{} = 10 / divisor - 5; return current;", binding("current"));
    let continuation = format!("{} = 10 / divisor - 5;", binding("ignored"));
    let branch = branch(shape, &returned, &continuation);
    format!("mod cpu Main {{ @noinline fn event(outer: bool, nested: bool, divisor: i64) -> i64 {{ print(99); {branch} print(77); return 19; }} fn main() -> i64 {{ return event({outer}, {nested}, {divisor}); }} }}")
}

fn suffix_source(returned: bool, gate: bool) -> String {
    source(
        "outer-then",
        "calls",
        "&&",
        returned,
        false,
        false,
        gate,
        2,
        false,
    )
    .replace(
        "other: bool) -> bool",
        "other: bool, end_divisor: i64) -> bool",
    )
    .replace(
        &format!("event({returned}, {gate}, 2, false, false, false)"),
        &format!("event({returned}, {gate}, 2, false, false, false, 0)"),
    )
    .replace(
        "print(77); return false;",
        "print(77); return helper(produce(end_divisor));",
    )
}

#[test]
fn conditional_return_continuations_keep_zero_returns_and_do_not_speculate_the_parent_suffix() {
    for shape in ["then", "else", "both", "outer-then", "outer-else"] {
        for outer in [false, true] {
            for nested in [false, true] {
                for divisor in [0, 2, -2] {
                    for declared in ["let", "const"] {
                        let source = integer_source(shape, outer, nested, divisor, declared);
                        let (returned, continuation) = paths(shape, outer, nested);
                        let expected = if divisor == 0 && (returned || continuation) {
                            None
                        } else if returned {
                            Some(10 / divisor - 5)
                        } else {
                            Some(19)
                        };
                        execute(
                            &source,
                            expected,
                            if returned { &[99] } else { &[99, 77] },
                            0,
                        );
                    }
                }
            }
        }
    }
    for returned in [false, true] {
        for gate in [false, true] {
            let result = if gate { 11 } else { 19 };
            execute(
                &suffix_source(returned, gate),
                returned.then_some(result),
                &[99, result],
                usize::from(gate),
            );
        }
    }
}

#[test]
fn conditional_return_continuations_keep_all_fallthrough_subtrees_and_sibling_scopes() {
    for outer in [false, true] {
        for nested in [false, true] {
            for other in [false, true] {
                for gate in [false, true] {
                    for divisor in [0, 2, -2] {
                        let (returned, continuation) = paths("both", outer, nested);
                        let rhs = returned && gate;
                        let result = if rhs && divisor > 0 { 11 } else { 19 };
                        let prints = if returned {
                            vec![99, result]
                        } else {
                            vec![99, 77, result]
                        };
                        execute(
                            &source(
                                "both", "tree", "&&", outer, nested, other, gate, divisor, false,
                            ),
                            (!(divisor == 0 && (rhs || continuation))).then_some(result),
                            &prints,
                            usize::from(rhs) + usize::from(continuation && other),
                        );
                    }
                }
            }
        }
    }
}

pub(super) fn assert_seed_after_work(original: &[NirStmt], seeded: &[NirStmt]) {
    match original.last() {
        Some(NirStmt::If {
            condition,
            then_body,
            else_body,
        }) => {
            assert_eq!(original.len(), seeded.len());
            assert_eq!(&original[..original.len() - 1], &seeded[..seeded.len() - 1]);
            let Some(NirStmt::If {
                condition: gate,
                then_body: yes,
                else_body: no,
            }) = seeded.last()
            else {
                panic!()
            };
            assert_eq!(condition, gate);
            assert_seed_after_work(then_body, yes);
            assert_seed_after_work(else_body, no);
        }
        Some(NirStmt::Return(Some(_))) => assert_eq!(original, seeded),
        _ => {
            assert_eq!(seeded.len(), original.len() + 1);
            assert_eq!(&seeded[..original.len()], original);
            assert_eq!(
                seeded.last(),
                Some(&NirStmt::Return(Some(NirExpr::Bool(false))))
            );
        }
    }
}

#[test]
fn conditional_return_continuations_append_only_leaf_seeds_preserve_captures_hygiene_and_idempotence(
) {
    for shape in ["then", "else", "both", "outer-then", "outer-else"] {
        for kind in ["scalar", "record", "calls", "hygiene", "tree"] {
            let mut module = crate::frontend::parse_nuis_module(&source(
                shape, kind, "&&", true, true, true, true, 2, false,
            ))
            .unwrap();
            let before = module.clone();
            let generated = outline_test(&mut module);
            assert_eq!(generated.len(), 1, "{shape}/{kind}");
            let old = before.functions.iter().find(|f| f.name == "event").unwrap();
            let new = module.functions.iter().find(|f| f.name == "event").unwrap();
            assert_eq!(&new.body[..2], &old.body[..2]);
            assert_eq!(
                &new.body[new.body.len() - 2..],
                &old.body[old.body.len() - 2..]
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
            } = &old.body[2]
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
            for (original, seeded) in [(then_body, yes), (else_body, no)] {
                assert_seed_after_work(original, seeded);
                let mut names = BTreeSet::new();
                branches::collect_bindings(seeded, &mut names);
                assert!(!names.contains(&helper.params[0].name));
            }
            let mut captures = vec!["divisor", "gate"];
            if !shape.starts_with("outer-") {
                captures.push("nested");
            }
            if kind == "tree" {
                captures.push("other");
            }
            assert_eq!(
                helper
                    .params
                    .iter()
                    .skip(1)
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>(),
                captures
            );
            for function in &before.functions {
                if function.name != "event" {
                    assert_eq!(
                        module
                            .functions
                            .iter()
                            .find(|f| f.name == function.name)
                            .unwrap(),
                        function
                    );
                }
            }
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let once = module.clone();
            assert!(outline_test(&mut module).is_empty());
            assert_eq!(module, once);
        }
    }
}

#[test]
fn conditional_return_continuations_retain_effect_rebind_scope_kind_capture_and_no_return_vetoes() {
    let original = source(
        "outer-then",
        "record",
        "&&",
        true,
        false,
        false,
        true,
        2,
        false,
    );
    // Keep the original intermediate continuation fixture, including its trap.
    let suffix = "if gate { let ignored = 10 / divisor; } let later = divisor;";
    for outer in [false, true] {
        for gate in [false, true] {
            for divisor in [0, 2, -2] {
                let source = source(
                    "outer-then",
                    "record",
                    "&&",
                    outer,
                    false,
                    false,
                    gate,
                    divisor,
                    false,
                )
                .replace(continuing("record"), suffix);
                let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
                assert_eq!(outline_test(&mut module).len(), 1);
                let result = if outer && gate && divisor > 0 { 11 } else { 19 };
                let prints = if outer {
                    vec![99, result]
                } else {
                    vec![99, 77, result]
                };
                execute(
                    &source,
                    (!(gate && divisor == 0)).then_some(result),
                    &prints,
                    usize::from(outer && gate),
                );
            }
        }
    }
    for (gate, divisor) in [(false, 0), (true, 2), (true, 0)] {
        let admitted = source(
            "outer-then",
            "record",
            "&&",
            false,
            false,
            false,
            gate,
            divisor,
            false,
        )
        .replace(
            continuing("record"),
            "let selected = gate && helper(produce(divisor));",
        );
        execute(
            &admitted,
            (!gate || divisor != 0).then_some(19),
            &[99, 77, 19],
            usize::from(gate),
        );
    }
    for candidate in [
        "print(88);",
        "let divisor = 10 / divisor;",
        "let first = divisor; let first = divisor;",
        "while gate { let ignored = 10 / divisor; }",
        "if gate { let local = divisor; } else { let ignored = local; }",
        "let declared: bool = 10 / divisor;",
        "let ignored = divisor; return;",
    ] {
        let mut module = match crate::frontend::parse_nuis_module(
            &original.replace(continuing("record"), candidate),
        ) {
            Ok(module) => module,
            Err(error) => {
                assert!(
                    candidate.contains("= local")
                        || candidate.contains("declared: bool")
                        || candidate.ends_with("return;"),
                    "{candidate}: {error}"
                );
                continue;
            }
        };
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{candidate}");
        assert_eq!(module, before);
    }
    let mut module = crate::frontend::parse_nuis_module(&original.replace(
        "return gate && helper(produce(divisor));",
        continuing("record"),
    ))
    .unwrap();
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
    for mutation in ["effect", "borrow", "kind", "aggregate"] {
        let mut module = crate::frontend::parse_nuis_module(&original).unwrap();
        if mutation == "effect" {
            module
                .functions
                .iter_mut()
                .find(|f| f.name == "produce")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(88)));
        } else {
            let event = module
                .functions
                .iter_mut()
                .find(|f| f.name == "event")
                .unwrap();
            if mutation == "borrow" {
                event.params[2].ty.is_ref = true;
            } else {
                let NirStmt::If { else_body, .. } = &mut event.body[2] else {
                    panic!()
                };
                let NirStmt::Let { ty, value, .. } = &mut else_body[0] else {
                    panic!()
                };
                if mutation == "kind" {
                    *ty = Some(scalar_type("bool"));
                } else {
                    *value = NirExpr::Var("original_packet".into());
                    event.params.push(NirParam {
                        name: "original_packet".into(),
                        ty: scalar_type("Packet"),
                    });
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
}

#[test]
fn conditional_return_continuations_bound_all_original_leaf_work_before_synthesizing_seeds() {
    for (shape, counts) in [("outer-then", [32, 33]), ("then", [29, 30])] {
        for count in counts {
            let locals = (0..count)
                .map(|i| format!("let fresh{i} = 10 / divisor;"))
                .collect::<String>();
            let original = source(shape, "scalar", "&&", true, true, false, true, 2, false);
            let source = if shape == "outer-then" {
                original.replace(continuing("scalar"), &locals)
            } else {
                original.replace("if nested {", &format!("{locals} if nested {{"))
            };
            let mut module = crate::frontend::parse_nuis_module(&source).unwrap();
            let before = module.clone();
            let rejected = count == counts[1];
            assert_eq!(
                outline_test(&mut module).is_empty(),
                rejected,
                "{shape}/{count}"
            );
            if rejected {
                assert_eq!(module, before);
            } else {
                let compiled = crate::pipeline::compile_source(&source).unwrap();
                yir_lower_llvm::emit_module(&compiled.yir).unwrap();
            }
        }
    }
    fn tree(depth: usize) -> NirExpr {
        if depth == 0 {
            NirExpr::Int(1)
        } else {
            binary(NirBinaryOp::Add, tree(depth - 1), tree(depth - 1))
        }
    }
    let module = crate::frontend::parse_nuis_module(&source(
        "outer-then",
        "record",
        "&&",
        true,
        false,
        false,
        true,
        2,
        false,
    ))
    .unwrap();
    let layouts = control_values::TypedLayouts::collect(&module);
    let carries = control_values::layouts(&module);
    let control = scalar_helpers::collect_with_layouts(&module, &carries);
    let catalog = scalar_helpers::collect_typed_values(&module, &layouts, &control);
    let binding = |name: &str, value| NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type("i64")),
        value,
    };
    let exact = vec![
        binding("one", tree(10)),
        binding("two", tree(10)),
        binding("three", NirExpr::Int(1)),
        binding("four", NirExpr::Int(1)),
    ];
    let accepted = prepare(
        &exact,
        &scalar_type("bool"),
        &Scope::new(),
        &catalog,
        &layouts,
    )
    .unwrap();
    assert!(!accepted.has_return);
    assert_eq!(accepted.ready, NirExpr::Bool(false));
    assert_seed_after_work(&exact, &accepted.value.body);
    let mut excess = exact.clone();
    excess.push(binding("five", NirExpr::Int(1)));
    assert!(prepare(
        &excess,
        &scalar_type("bool"),
        &Scope::new(),
        &catalog,
        &layouts
    )
    .is_none());
    let mut linear = NirExpr::Int(1);
    for _ in 0..64 {
        linear = binary(NirBinaryOp::Add, NirExpr::Int(1), linear);
    }
    assert!(prepare(
        &[binding("deep", linear)],
        &scalar_type("bool"),
        &Scope::new(),
        &catalog,
        &layouts
    )
    .is_none());
}
