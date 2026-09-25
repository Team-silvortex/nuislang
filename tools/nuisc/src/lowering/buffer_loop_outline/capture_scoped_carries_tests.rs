use super::tests::{function, module, project_scoped};
use super::*;

#[test]
fn generated_scoped_record_inputs_project_only_demanded_initial_fields() {
    let source = "
        struct Words { carry0: i64, carry1: i64 }
        fn helper(old: Pair, index: i64) -> Words {
            let saved: Pair = old;
            let old: Pair = Pair { x: saved.x + 1, y: saved.x / index };
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
        let entry = function(&module, "entry").clone();
        if reverse {
            module.functions.reverse();
        }
        assert!(project_scoped(&mut module, &["helper"]));
        let helper = function(&module, "helper");
        assert_eq!(helper.params.len(), 2);
        assert!(helper.params.iter().all(|p| p.ty.name == "i64"));
        assert_eq!(helper.params[1].name, "index");
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
        assert_eq!(access(&args[0]), Some(vec!["carry".into(), "x".into()]));
        assert_eq!(args[1], NirExpr::Var("i".into()));
        let NirStmt::While { body: original, .. } = &entry.body[2] else {
            panic!()
        };
        assert_eq!(body[1..], original[1..]);
        assert!(!project_scoped(&mut module, &["helper"]));
    }
}

const SOURCE: &str =
    include_str!("../../../tests/control_flow_syntax_native/scoped_projected_record_carries.ns");

#[test]
fn generated_scoped_projection_rejects_unproven_or_empty_carry_maps_transactionally() {
    let source = "
        struct Words { carry0: i64, carry1: i64 }
        fn helper(old: Pair, index: i64) -> Words {
            let old: Pair = Pair { x: old.x + index, y: 9 };
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
    for source in [
        source.replace("x: old.x + index", "x: index"),
        source.replace("y: 9", "y: old.y"),
        source.replace("x: words.carry0, y: words.carry1", "x: words.carry1, y: words.carry0"),
        source.replace("helper(carry, i)", "helper(Pair { x: carry.x, y: carry.y }, i)"),
        source.replace("old: Pair, index: i64", "old: Pair, duplicate: Pair, index: i64")
            .replace("helper(carry, i)", "helper(carry, carry, i)"),
        format!("{source} fn another(state: Pair) -> Words {{ return helper(Pair {{ x: state.x, y: state.y }}, 1); }}"),
    ] {
        let mut module = module(&source);
        let before = module.clone();
        assert!(!project_scoped(&mut module, &["helper"]), "{source}");
        assert_eq!(module, before);
    }
}

#[test]
fn generated_scoped_projection_requires_all_callers_to_prove_the_backedge() {
    let mut module = module(
        "struct Words { carry0: i64, carry1: i64 }
        fn helper(old: Pair, index: i64) -> Words {
            return Words { carry0: old.x + index, carry1: 9 };
        }
        fn first(state: Pair) -> Pair {
            let i = 0; let carry = state;
            while i < 2 {
                let words: Words = helper(carry, i);
                let carry: Pair = Pair { x: words.carry0, y: words.carry1 };
                let i = i + 1;
            } return carry;
        }
        fn second(state: Pair) -> Pair {
            let i = 0; let carry = state;
            while i < 2 {
                let words: Words = helper(carry, i);
                if i == 1 { let carry = state; }
                let i = i + 1;
            } return carry;
        }",
    );
    for reverse in [false, true] {
        if reverse {
            module.functions.reverse();
        }
        let before = module.clone();
        assert!(!project_scoped(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn generated_scoped_snapshots_leave_fallthrough_and_nested_loop_writes_whole() {
    for update in [
        "if index > 0 { let old: Pair = Pair { x: old.x + 1, y: 9 }; }",
        "let j = 0; while j < index { let old: Pair = Pair { x: old.x + 1, y: 9 }; let j = j + 1; }",
    ] {
        let mut module = module(&format!(
            "struct Words {{ carry0: i64, carry1: i64 }}
            fn helper(old: Pair, index: i64) -> Words {{
                {update} return Words {{ carry0: old.x, carry1: old.y }};
            }}
            fn entry(state: Pair) -> Pair {{
                let i = 0; let carry = state;
                while i < 2 {{
                    let words: Words = helper(carry, i);
                    let carry: Pair = Pair {{ x: words.carry0, y: words.carry1 }};
                    let i = i + 1;
                }} return carry;
            }}"
        ));
        let before = module.clone();
        assert!(!project_scoped(&mut module, &["helper"]));
        assert_eq!(module, before);
    }
}

#[test]
fn generated_partial_carries_execute_with_complete_seed_storage() {
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
    let mapped = call
        .operands
        .iter()
        .filter_map(|arg| {
            yir_core::parse_loop_owned_struct_carry(arg)
                .unwrap()
                .map(|(index, _)| index)
        })
        .collect::<Vec<_>>();
    assert_eq!(mapped, [0]);
    let iteration = yir
        .functions
        .iter()
        .find(|f| f.name == call.callee)
        .unwrap();
    assert_eq!(iteration.parameters.len(), 3);
    assert_eq!(reference(&mut yir).unwrap(), 154);
}

fn reference(yir: &mut yir_core::YirModule) -> Result<i64, String> {
    yir.nodes.reverse();
    yir.functions.reverse();
    for function in &mut yir.functions {
        function.body_nodes.reverse();
    }
    yir_verify::verify_module(yir).unwrap();
    yir_lower_llvm::emit_module(yir).unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &crate::render::render_yir(yir),
        &yir_verify::default_registry(),
    )
    .map_err(|error| error.to_string())?;
    let entry = yir
        .functions
        .iter()
        .find(|f| f.role == yir_core::YirFunctionRole::Entry)
        .unwrap();
    let yir_core::Value::Int(value) = trace.values[&entry.result.as_ref().unwrap().node] else {
        panic!()
    };
    Ok(value)
}

#[test]
fn generated_projection_retains_all_initial_and_per_trip_checked_work() {
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
fn generated_projection_scales_state_width_independently_of_helper_arity() {
    for width in [7, 64] {
        let extra = |value: &str| {
            (3..width)
                .map(|index| format!("extra{index}: {value}"))
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
        assert_eq!(call.operands.len(), 3);
        assert_eq!(
            call.operands
                .iter()
                .filter(|arg| arg.starts_with("$owned_struct_carry:"))
                .count(),
            1
        );
        assert_eq!(reference(&mut yir).unwrap(), 154);
    }
}

#[test]
fn generated_projection_preserves_break_continue_and_bool_word_identities() {
    let result = "packet.left + packet.right + packet.unused + saved.right";
    for (source, expected, omitted) in [
        (
            SOURCE.replace(
                "unused: 9\n            };",
                "unused: 9\n            }; if i == 1 { break; }",
            ),
            152,
            2,
        ),
        (
            SOURCE.replace(
                "let packet = packet;",
                "if i == 1 { continue; } let packet = packet;",
            ),
            152,
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
            153,
            2,
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
