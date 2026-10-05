use super::super::tests::{execute, outline_test};
use super::*;

#[path = "conditional_computed_gates_native_tests.rs"]
mod native;

type Input = (i64, i64, i64, bool, bool, i64);
const INPUTS: [Input; 8] = [
    (-2, 0, 0, false, false, 2),
    (2, 0, 0, false, false, 0),
    (2, 2, 2, true, false, 0),
    (-2, -2, -2, true, false, 2),
    (0, 2, 2, true, false, 2),
    (2, -2, 0, false, false, 2),
    (0, 0, 0, true, true, 0),
    (-2, 2, 0, false, true, 0),
];

fn left(kind: &str) -> &'static str {
    match kind {
        "call" => "helper(produce(left))",
        "field" => "produce(left).value > 0",
        "division" => "10 / left > 0",
        "comparison" => "(left > 0) == true",
        _ => unreachable!(),
    }
}

fn right(kind: &str) -> &'static str {
    match kind {
        "call" => "helper(produce(right))",
        "field" => "produce(right).value > 0",
        "division" => "10 / right > 0",
        "atom" => "gate",
        _ => unreachable!(),
    }
}

fn selected(op: &str, gate: bool) -> bool {
    if op == "&&" {
        gate
    } else {
        !gate
    }
}

fn source(place: &str, kind: &str, rhs: &str, op: &str, input: Input) -> String {
    let (l, r, w, gate, early, tail) = input;
    let logical = format!("({}) {op} ({})", left(kind), right(rhs));
    let returned = "return gate && helper(produce(work));";
    let body = match place {
        "let" => format!("let result: bool = {logical}; print(77); return result;"),
        "inferred" => format!("let result = {logical}; print(77); return result;"),
        "const" => format!("const result: bool = {logical}; print(77); return result;"),
        "return" => format!("return {logical};"),
        "then" => format!("if {logical} {{ {returned} }}"),
        "else" => format!("if {logical} {{ }} else {{ {returned} }}"),
        "both" => format!("if {logical} {{ {returned} }} else {{ return helper(produce(work)); }}"),
        "partial" => format!("if {logical} {{ if gate {{ return false; }} else {{ let ignored = helper(produce(work)); }} }}"),
        "suffix" => format!("if {logical} {{ if gate {{ return false; }} let ignored = helper(produce(work)); }}"),
        _ => unreachable!(),
    };
    let parent_tail = if matches!(place, "let" | "inferred" | "const" | "return") {
        ""
    } else {
        "let parent_checked = 20 / tail; print(77); return false;"
    };
    format!("mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(left: i64, right: i64, work: i64, gate: bool, early: bool, tail: i64) -> bool {{
            if early {{ return false; }} print(99); {body} {parent_tail}
        }}
        fn main() -> i64 {{ let result = event({l}, {r}, {w}, {gate}, {early}, {tail});
            if result {{ print(11); return 11; }} print(19); return 19;
        }}
    }}")
}

#[test]
fn conditional_computed_gates_preserve_value_roots_left_checks_and_lazy_rhs_once() {
    let mut cases = 0;
    for place in ["let", "inferred", "const", "return"] {
        for kind in ["call", "field", "division", "comparison"] {
            for op in ["&&", "||"] {
                for input in INPUTS {
                    let (l, r, _, _, early, _) = input;
                    let rhs = !early && selected(op, l > 0);
                    let value = !early && if rhs { r > 0 } else { l > 0 };
                    let failed = !early && l == 0 && kind != "comparison" || rhs && r == 0;
                    let result = if value { 11 } else { 19 };
                    let prints = if early {
                        vec![19]
                    } else if place == "return" {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    };
                    execute(
                        &source(place, kind, "call", op, input),
                        (!failed).then_some(result),
                        &prints,
                        usize::from(!early && kind == "call") + usize::from(rhs),
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 256);
}

#[test]
fn conditional_computed_gates_compose_saved_entries_complete_partial_and_suffix_returns() {
    let mut cases = 0;
    for place in ["then", "else", "both", "partial", "suffix"] {
        for kind in ["call", "field", "division"] {
            for right_kind in ["call", "field", "division"] {
                for op in ["&&", "||"] {
                    for input in INPUTS {
                        let (l, r, work, gate, early, tail) = input;
                        let rhs = !early && selected(op, l > 0);
                        let condition = if rhs { r > 0 } else { l > 0 };
                        let entered = !early
                            && (place == "both"
                                || if place == "else" {
                                    !condition
                                } else {
                                    condition
                                });
                        let partial = matches!(place, "partial" | "suffix");
                        let returned = entered && (!partial || gate);
                        let work_selected = entered
                            && if partial {
                                !gate
                            } else {
                                place == "both" && !condition || gate
                            };
                        let failed = !early && l == 0
                            || rhs && r == 0
                            || work_selected && work == 0
                            || !early && !returned && tail == 0;
                        let result = if !partial && returned && work_selected && work > 0 {
                            11
                        } else {
                            19
                        };
                        let prints = if early {
                            vec![19]
                        } else if returned {
                            vec![99, result]
                        } else {
                            vec![99, 77, result]
                        };
                        execute(
                            &source(place, kind, right_kind, op, input),
                            (!failed).then_some(result),
                            &prints,
                            usize::from(!early && kind == "call")
                                + usize::from(rhs && right_kind == "call")
                                + usize::from(work_selected),
                        );
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 720);
}

fn composed(mode: &str, op: &str, input: Input) -> String {
    let (l, r, work, gate, early, tail) = input;
    match mode {
        "record" => source("then", "call", "call", op, input)
            .replace("print(99);", "print(99); let packet = produce(left);")
            .replace("helper(produce(left))", "helper(packet)")
            .replace("return gate && helper(produce(work));", "if helper(produce(work)) { return false; }"),
        "rebind" => source("let", "comparison", "call", op, input)
            .replace("let result: bool =", "let gate: bool =")
            .replace("(left > 0) == true", "gate == true")
            .replace("print(77); return result;", "let result = gate && helper(produce(work)); print(77); return result;"),
        "zero" => format!("mod cpu Main {{
            struct Packet {{ unused: i64, value: i64 }}
            @noinline fn produce(value: i64) -> Packet {{ return Packet {{ unused: 10 / value, value: value }}; }}
            @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
            @noinline fn event(left: i64, right: i64, tail: i64, early: bool) -> i64 {{
                if early {{ return 0; }} print(99);
                if helper(produce(left)) {op} helper(produce(right)) {{ return 0; }}
                print(77); return 20 / tail;
            }}
            fn main() -> i64 {{ let result = event({l}, {r}, {tail}, {early}); print(result); return result; }}
        }}"),
        "atom" => source("return", "call", "atom", op, (l, r, work, gate, early, tail)),
        _ => unreachable!(),
    }
}

#[test]
fn conditional_computed_gates_keep_parent_records_current_rebinding_total_rhs_and_real_zero_exits()
{
    let mut cases = 0;
    for mode in ["record", "rebind", "zero", "atom"] {
        for op in ["&&", "||"] {
            for input in INPUTS {
                let (l, r, work, gate, early, tail) = input;
                let left_value = if mode == "rebind" { gate } else { l > 0 };
                let rhs = !early && selected(op, left_value);
                let condition = if rhs {
                    if mode == "atom" {
                        gate
                    } else {
                        r > 0
                    }
                } else {
                    left_value
                };
                let work_selected = !early && condition && matches!(mode, "record" | "rebind");
                let returned = mode != "record" || work_selected && work > 0;
                let failed = !early && mode != "rebind" && l == 0
                    || rhs && mode != "atom" && r == 0
                    || work_selected && work == 0
                    || !early
                        && (mode == "zero" && !condition || mode == "record" && !returned)
                        && tail == 0;
                let result = if mode == "zero" {
                    if early || condition {
                        0
                    } else if tail != 0 {
                        20 / tail
                    } else {
                        0
                    }
                } else if mode == "record" || early {
                    19
                } else if mode == "rebind" {
                    if condition && work > 0 {
                        11
                    } else {
                        19
                    }
                } else if condition {
                    11
                } else {
                    19
                };
                let prints = if early {
                    vec![result]
                } else if mode == "zero" {
                    if condition {
                        vec![99, result]
                    } else {
                        vec![99, 77, result]
                    }
                } else if mode == "record" && returned || mode == "atom" {
                    vec![99, result]
                } else {
                    vec![99, 77, result]
                };
                let calls = usize::from(!early && mode != "rebind")
                    + usize::from(rhs && mode != "atom")
                    + usize::from(work_selected);
                execute(
                    &composed(mode, op, input),
                    (!failed).then_some(result),
                    &prints,
                    calls,
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 64);
}

fn outline_values(module: &mut NirModule, full_control: bool) -> BTreeSet<String> {
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
    conditional_values::outline(module, &catalog, &roots, &layouts, &mut names)
}

#[test]
fn conditional_computed_gates_keep_original_gate_first_selected_rhs_and_parent_only_record_transport(
) {
    for place in ["let", "inferred", "const", "return", "then"] {
        for full_control in [false, true] {
            let mut module = crate::frontend::parse_nuis_module(&source(
                place,
                "call",
                "call",
                "&&",
                (2, 2, 2, true, false, 2),
            ))
            .unwrap();
            let before = module.clone();
            let generated = outline_values(&mut module, full_control);
            assert!(!generated.is_empty());
            let original = before.functions.iter().find(|f| f.name == "event").unwrap();
            let original_root = match &original.body[2] {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Return(Some(value)) => value,
                NirStmt::If { condition, .. } => condition,
                _ => panic!(),
            };
            let NirExpr::Binary { lhs, rhs, .. } = original_root else {
                panic!()
            };
            let new = module.functions.iter().find(|f| f.name == "event").unwrap();
            let value = match &new.body[2] {
                NirStmt::Let { value, .. }
                | NirStmt::Const { value, .. }
                | NirStmt::Return(Some(value)) => value,
                NirStmt::If { condition, .. } => condition,
                _ => panic!(),
            };
            let NirExpr::Call { callee, args } = value else {
                panic!()
            };
            assert_eq!(&args[0], lhs.as_ref());
            assert_eq!(args.len(), 2);
            let helper = module.functions.iter().find(|f| &f.name == callee).unwrap();
            let NirStmt::If { then_body, .. } = &helper.body[0] else {
                panic!()
            };
            assert_eq!(then_body, &vec![NirStmt::Return(Some((**rhs).clone()))]);
            assert!(helper.params.iter().all(|p| scalar(&p.ty)));
            crate::nir_verify::verify_nir_module(&module).unwrap();
            let once = module.clone();
            assert!(outline_values(&mut module, full_control).is_empty());
            assert_eq!(module, once);
        }
    }
    let text = composed("record", "||", (2, 0, 2, false, false, 0));
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let generated = outline_test(&mut module);
    assert_eq!(generated.len(), 1);
    let helper = module
        .functions
        .iter()
        .find(|f| generated.contains(&f.name))
        .unwrap();
    assert!(helper.params.iter().all(|p| scalar(&p.ty)));
    let event = module.functions.iter().find(|f| f.name == "event").unwrap();
    let NirStmt::Let { ty, value, .. } = &event.body[3] else {
        panic!()
    };
    assert_eq!(ty.as_ref(), Some(&scalar_type("bool")));
    let NirExpr::Binary { lhs, .. } = value else {
        panic!()
    };
    assert_eq!(
        lhs.as_ref(),
        &NirExpr::Call {
            callee: "helper".into(),
            args: vec![NirExpr::Var("packet".into())]
        }
    );
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

fn tree(depth: usize) -> NirExpr {
    if depth == 0 {
        return NirExpr::Int(1);
    }
    NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(tree(depth - 1)),
        rhs: Box::new(tree(depth - 1)),
    }
}

fn trim_leaf(expr: &mut NirExpr) {
    let NirExpr::Binary { lhs, .. } = expr else {
        panic!()
    };
    if matches!(lhs.as_ref(), NirExpr::Int(_)) {
        *expr = NirExpr::Int(1);
    } else {
        trim_leaf(lhs);
    }
}

#[test]
fn conditional_computed_gates_share_whole_root_work_depth_before_inference_and_keep_arm_modes_separate(
) {
    let base = "mod cpu Main {
        fn decide(input: i64) -> bool { return input > 0; }
        fn narrow(input: i32) -> bool { return false; }
        fn event(input: i64, gate: bool) -> bool { print(99); if decide(input) && gate { return false; } return false; }
    }";
    for oversized in [false, true] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let mut argument = tree(11);
        trim_leaf(&mut argument);
        // 4093 argument nodes + call + logical root + RHS atom = 4096.
        let gate = if oversized {
            NirExpr::Call {
                callee: "narrow".into(),
                args: vec![NirExpr::CastI64ToI32(Box::new(argument))],
            }
        } else {
            NirExpr::Call {
                callee: "decide".into(),
                args: vec![argument],
            }
        };
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { condition, .. } = &mut event.body[1] else {
            panic!()
        };
        *condition = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(gate),
            rhs: Box::new(NirExpr::Var("gate".into())),
        };
        assert_eq!(
            conditional_values::prefix::computed_logical_root(condition),
            !oversized
        );
        assert!(!conditional_values::prefix::expression_roots(vec![(
            condition, 0, true
        )]));
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), oversized);
        if oversized {
            assert_eq!(module, before);
        }
    }
    for count in [61, 62] {
        let mut module = crate::frontend::parse_nuis_module(base).unwrap();
        let mut arg = NirExpr::Var("input".into());
        for _ in 0..count {
            arg = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(arg),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let event = module
            .functions
            .iter_mut()
            .find(|f| f.name == "event")
            .unwrap();
        let NirStmt::If { condition, .. } = &mut event.body[1] else {
            panic!()
        };
        let NirExpr::Binary { lhs, .. } = condition else {
            panic!()
        };
        *lhs = Box::new(NirExpr::Call {
            callee: "decide".into(),
            args: vec![arg],
        });
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 62);
        if count == 62 {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_computed_gates_retain_nested_effect_type_loop_capture_vetoes_and_internal_arm_admission(
) {
    let base = source("then", "call", "call", "&&", (2, 2, 2, true, false, 2));
    for root in [
        "(helper(produce(left)) && gate) && helper(produce(right))",
        "helper(produce(left)) && (gate || helper(produce(right)))",
    ] {
        let candidate = base.replace("(helper(produce(left))) && (helper(produce(right)))", root);
        let mut module = crate::frontend::parse_nuis_module(&candidate).unwrap();
        assert_eq!(outline_test(&mut module).len(), 1);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let compiled = crate::pipeline::compile_source(&candidate).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        let candidate = source("let", "call", "call", "&&", (2, 2, 2, true, false, 2))
            .replace("(helper(produce(left))) && (helper(produce(right)))", root);
        let mut module = crate::frontend::parse_nuis_module(&candidate).unwrap();
        assert!(!outline_values(&mut module, true).is_empty());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let compiled = crate::pipeline::compile_source(&candidate).unwrap();
        yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    }
    for mutation in [
        "borrow", "optional", "generic", "effect", "unknown", "branch", "capture",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
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
            match mutation {
                "borrow" => event.params[0].ty.is_ref = true,
                "optional" => event.params[0].ty.is_optional = true,
                "generic" => event.params[0].ty.generic_args.push(scalar_type("i64")),
                "capture" => event.params[3].ty.is_ref = true,
                _ => {
                    let NirStmt::If {
                        condition,
                        then_body,
                        ..
                    } = &mut event.body[2]
                    else {
                        panic!()
                    };
                    if mutation == "branch" {
                        then_body.insert(0, NirStmt::Print(NirExpr::Int(88)));
                    } else {
                        let NirExpr::Binary { lhs, .. } = condition else {
                            panic!()
                        };
                        *lhs = Box::new(NirExpr::Call {
                            callee: "unknown".into(),
                            args: vec![],
                        });
                    }
                }
            }
        }
        let before = module.clone();
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before);
    }
    let inner = base
        .replace(
            "if (helper(produce(left))) && (helper(produce(right)))",
            "if gate",
        )
        .replace(
            "return gate && helper(produce(work));",
            "if helper(produce(left)) && helper(produce(right)) { return false; }",
        );
    let mut module = crate::frontend::parse_nuis_module(&inner).unwrap();
    assert_eq!(outline_test(&mut module).len(), 1);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    execute(&inner, Some(19), &[99, 19], 2);
    for place in ["let", "then"] {
        let text = source(place, "call", "call", "&&", (2, 2, 2, true, false, 2));
        for atom_gate in [false, true] {
            let text = if atom_gate {
                text.replace("(helper(produce(left)))", "gate")
            } else {
                text.clone()
            };
            let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
            let event = module
                .functions
                .iter_mut()
                .find(|f| f.name == "event")
                .unwrap();
            let body = std::mem::take(&mut event.body);
            event.body = vec![
                NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body,
                },
                NirStmt::Return(Some(NirExpr::Bool(false))),
            ];
            let before = module.clone();
            let generated = outline_values(&mut module, true);
            if place == "then" && atom_gate {
                assert_eq!(generated.len(), 1);
                crate::nir_verify::verify_nir_module(&module).unwrap();
            } else {
                assert!(generated.is_empty());
                assert_eq!(module, before);
            }
        }
    }
}
