use super::carry_tests::reference;
use super::tests::{function, module, project_scoped};
use super::*;

const SOURCE: &str =
    include_str!("../../../tests/control_flow_syntax_native/scoped_unread_record_carries.ns");

#[test]
fn generated_unread_records_keep_seeds_without_iteration_operands() {
    let mut yir = crate::pipeline::compile_source(SOURCE).unwrap().yir;
    let call = yir
        .nodes
        .iter()
        .find_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
        })
        .unwrap();
    assert_eq!(call.seeds.len(), 3);
    assert!(call.operands.iter().all(|arg| {
        yir_core::parse_loop_owned_struct_carry(arg)
            .unwrap()
            .is_none()
    }));
    let iteration = yir
        .functions
        .iter()
        .find(|f| f.name == call.callee)
        .unwrap();
    assert_eq!(iteration.parameters.len(), 2);
    assert_eq!(reference(&mut yir).unwrap(), 132);
}

#[test]
fn generated_unread_record_projection_preserves_caller_reconstruction() {
    let source = "
        struct Words { carry0: i64, carry1: i64 }
        fn helper(old: Pair, index: i64) -> Words {
            let old: Pair = Pair { x: index, y: 9 };
            return Words { carry0: old.x, carry1: old.y };
        }
        fn entry(state: Pair) -> Pair {
            let i = 1; let carry = state;
            while i < 3 {
                let words: Words = helper(carry, i);
                let carry: Pair = Pair { x: words.carry0, y: words.carry1 };
                let i = i + 1;
            }
            return carry;
        }";
    for reverse in [false, true] {
        let mut module = module(source);
        if reverse {
            module.functions.reverse();
        }
        let entry = function(&module, "entry").clone();
        assert!(project_scoped(&mut module, &["helper"]));
        assert_eq!(function(&module, "helper").params.len(), 1);
        let NirStmt::While { body, .. } = &function(&module, "entry").body[2] else {
            panic!()
        };
        let NirStmt::Let {
            value: NirExpr::Call { args, .. },
            ..
        } = &body[0]
        else {
            panic!()
        };
        assert_eq!(args, &[NirExpr::Var("i".into())]);
        let NirStmt::While { body: original, .. } = &entry.body[2] else {
            panic!()
        };
        assert_eq!(body[1..], original[1..]);
        assert!(!project_scoped(&mut module, &["helper"]));
    }
}

#[test]
fn generated_unread_records_preserve_initial_and_per_trip_failures() {
    for source in [
        SOURCE.replace("walk(2, 1)", "walk(2, 0)"),
        SOURCE
            .replace("unused: 7", "unused: 7 / divisor")
            .replace("walk(2, 1) + walk(0, 0)", "walk(0, 0)"),
        SOURCE.replace("unused: 9", "unused: 9 / (divisor - i + 1)"),
    ] {
        let mut yir = crate::pipeline::compile_source(&source).unwrap().yir;
        assert!(reference(&mut yir).is_err(), "{source}");
    }
}

#[test]
fn generated_unread_record_width_is_independent_of_iteration_arity() {
    for width in [7, 64] {
        let extra = |value: &str| {
            (3..width)
                .map(|i| format!("extra{i}: {value}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let source = SOURCE
            .replace("unused: i64", &format!("unused: i64, {}", extra("i64")))
            .replace("unused: 7", &format!("unused: 7, {}", extra("27")))
            .replace("unused: 9", &format!("unused: 9, {}", extra("99")));
        let mut yir = crate::pipeline::compile_source(&source).unwrap().yir;
        let call = yir
            .nodes
            .iter()
            .find_map(|node| {
                yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                    .ok()
                    .flatten()
            })
            .unwrap();
        assert_eq!(call.seeds.len(), width);
        assert_eq!(call.operands.len(), 2);
        assert_eq!(reference(&mut yir).unwrap(), 132);
    }
}

#[test]
fn generated_unread_record_elision_preserves_control_seed_identities() {
    let result = "packet.left + packet.right + packet.unused + saved.right";
    for (source, expected, omitted) in [
        (
            SOURCE.replace(
                "unused: 9\n            };",
                "unused: 9\n            }; if i == 1 { break; }",
            ),
            130,
            3,
        ),
        (
            SOURCE.replace(
                "let packet = packet;",
                "if i == 1 { continue; } let packet = packet;",
            ),
            132,
            0,
        ),
        (
            SOURCE
                .replace("let i = 0;", "let i = 0; let flag = false;")
                .replace(
                    "unused: 9\n            };",
                    "unused: 9\n            }; let flag = !flag; if i == 1 { break; }",
                )
                .replace(
                    &format!("return {result};"),
                    &format!("if flag {{ return {result} + 1; }} return {result};"),
                ),
            131,
            3,
        ),
    ] {
        let mut yir = crate::pipeline::compile_source(&source).unwrap().yir;
        let call = yir
            .nodes
            .iter()
            .find_map(|node| {
                yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                    .ok()
                    .flatten()
            })
            .unwrap();
        let mapped = call
            .operands
            .iter()
            .filter(|arg| arg.starts_with("$owned_struct_carry:"))
            .count();
        assert_eq!(call.seeds.len() - mapped, omitted, "{source}");
        assert_eq!(reference(&mut yir).unwrap(), expected, "{source}");
    }
}

#[test]
fn generated_unread_record_proofs_require_all_callers_to_agree_on_slots() {
    let source = "
        struct Words { carry0: i64, carry1: i64, carry2: i64, carry3: i64 }
        fn helper(old: Pair, kept: Pair) -> Words {
            return Words { carry0: 1, carry1: 2, carry2: kept.x, carry3: kept.y };
        }
        fn entry(state: Pair) -> Pair {
            let i = 0; let carry = state; let other = state;
            while i < 2 {
                let words: Words = helper(carry, other);
                let carry: Pair = Pair { x: words.carry0, y: words.carry1 };
                let other: Pair = Pair { x: words.carry2, y: words.carry3 };
                let i = i + 1;
            } return carry;
        }";
    for swapped in [false, true] {
        let second = source.split_once("fn entry").unwrap().1;
        let second = if swapped {
            second.replace("helper(carry, other)", "helper(other, carry)")
        } else {
            second.to_owned()
        };
        let combined = format!("{source} fn second{second}");
        for reverse in [false, true] {
            let mut module = module(&combined);
            if reverse {
                module.functions.reverse();
            }
            let before = module.clone();
            let names = BTreeSet::from(["helper".into()]);
            let layouts = control_values::TypedLayouts::collect(&module);
            let projected = project(&mut module, &names, &names, &layouts);
            assert_eq!(projected.changed, !swapped);
            assert_eq!(projected.elided_records.contains_key("helper"), !swapped);
            if swapped {
                assert_eq!(module, before);
            }
            crate::nir_verify::verify_nir_module(&module).unwrap();
        }
    }
}

#[test]
fn generated_name_does_not_authorize_source_record_seed_elision() {
    let source = "
        struct Words { carry0: i64, carry1: i64 }
        fn helper(old: Pair) -> Words { return Words { carry0: 1, carry1: 2 }; }
        fn entry(state: Pair) -> Pair {
            let i = 0; let carry = state;
            while i < 2 {
                let words: Words = helper(carry);
                let carry: Pair = Pair { x: words.carry0, y: words.carry1 };
                let i = i + 1;
            } return carry;
        }";
    for name in ["helper", "__nuis_scalar_iteration_forged"] {
        let mut module = module(&source.replace("helper", name));
        let before = module.clone();
        let layouts = control_values::TypedLayouts::collect(&module);
        let projected = project(
            &mut module,
            &BTreeSet::from([name.into()]),
            &BTreeSet::new(),
            &layouts,
        );
        assert!(!projected.changed);
        assert!(projected.elided_records.is_empty());
        assert_eq!(module, before);
    }
}

#[test]
fn generated_unread_records_keep_distinct_complete_slot_ranges() {
    let source = SOURCE
        .replace("let i = 0;", "let other = Packet { left: 5, right: 6, unused: 7 }; let i = 0;")
        .replace("let i = i + 1;", "let i = i + 1; let other = other; let other = Packet { left: i * 10, right: 0, unused: 0 };")
        .replace("+ saved.right;", "+ saved.right + other.left;");
    let mut yir = crate::pipeline::compile_source(&source).unwrap().yir;
    let call = yir
        .nodes
        .iter()
        .find_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
        })
        .unwrap();
    assert_eq!(call.seeds.len(), 6);
    assert_eq!(call.operands.len(), 2);
    assert_eq!(reference(&mut yir).unwrap(), 157);
}

#[test]
fn generated_unread_record_elision_does_not_publish_a_vetoed_proof() {
    let mut module = module(
        "
        struct Words { carry0: i64, carry1: i64 }
        fn helper(old: Pair) -> Words { return Words { carry0: 1, carry1: 2 }; }
        fn entry(state: Pair) -> Pair {
            let i = 0; let carry = state;
            while i < 2 {
                let words: Words = helper(carry);
                let carry: Pair = Pair { x: words.carry0, y: words.carry1 };
                let i = i + 1;
            } return carry;
        }
        fn second() -> Words { return helper(Pair { x: 1, y: 2 }); }",
    );
    let before = module.clone();
    let names = BTreeSet::from(["helper".into()]);
    let layouts = control_values::TypedLayouts::collect(&module);
    let projected = project(&mut module, &names, &names, &layouts);
    assert!(!projected.changed);
    assert!(projected.elided_records.is_empty());
    assert_eq!(module, before);
}
