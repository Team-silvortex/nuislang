use super::*;

#[path = "capture_record_loop_demand_tests.rs"]
mod loop_demand;
#[path = "capture_record_joins_tests.rs"]
mod materialized;
#[path = "capture_record_scopes_tests.rs"]
mod scopes;

const SOURCE: &str =
    include_str!("../../../tests/control_flow_syntax_native/scoped_sparse_typed_record_carries.ns");

#[test]
fn sparse_typed_record_inputs_keep_full_seeds_and_exact_leaf_backedges() {
    let source = crate::frontend::parse_nuis_module(SOURCE).unwrap();
    let catalog = scalar_helpers::collect_with_layouts(&source, &control_values::layouts(&source));
    assert!(catalog.contains_key("step"));
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
    assert_eq!(call.seeds.len(), 9);
    let mapped = call
        .operands
        .iter()
        .filter_map(|arg| {
            yir_core::parse_loop_owned_struct_carry(arg)
                .unwrap()
                .map(|(slot, _)| slot)
        })
        .collect::<Vec<_>>();
    assert_eq!(mapped, [0, 1, 2, 3]);
    assert_eq!(call.operands.len(), 6);
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
}

#[test]
fn sparse_typed_record_inputs_support_flat_shapes_and_unread_complete_states() {
    let flat = "mod cpu Main {
        struct State { value: f64, flag: bool, unused: i64 }
        fn walk(limit: i64) -> State {
            let carry = State { value: 1.5, flag: false, unused: 27 }; let i = 0;
            while i < limit { let i = i + 1; let carry = carry;
                let flag = carry.flag;
                let carry = State { value: carry.value, flag: !flag, unused: 99 }; }
            return carry;
        }
        fn main() -> i64 { let zero = walk(0); let done = walk(3);
            if done.flag { return zero.unused + done.unused; } return 200; }
    }";
    let mut yir = crate::pipeline::compile_source(flat).unwrap().yir;
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
    assert_eq!(
        call.operands
            .iter()
            .filter(|arg| arg.starts_with("$owned_struct_carry:"))
            .count(),
        2
    );
    assert_eq!(
        scoped_inputs::carry_tests::reference(&mut yir).unwrap(),
        126
    );

    let source = SOURCE
        .replace("saved.left.value", "1.5")
        .replace("saved.left.gain", "2.5")
        .replace("saved.left.tag", "i32_from_i64(7)")
        .replace("saved.left.enabled", "false");
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
    assert_eq!(call.seeds.len(), 9);
    assert!(!call
        .operands
        .iter()
        .any(|arg| arg.starts_with("$owned_struct_carry:")));
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
}

#[test]
fn sparse_typed_record_projection_preserves_subrecord_aliases_and_initializer_traps() {
    let source = SOURCE
        .replace(
            "let enabled = saved.left.enabled;",
            "let leaf = saved.left; let enabled = leaf.enabled;",
        )
        .replace("saved.left.", "leaf.");
    let mut yir = crate::pipeline::compile_source(&source).unwrap().yir;
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
    for source in [
        source
            .replace("i32_from_i64(7), 3", "i32_from_i64(7), 0")
            .replace(
                "i32_from_i64(7), enabled: true",
                "i32_from_i64(7 / 0), enabled: true",
            ),
        source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 / (limit - i))"),
    ] {
        let mut yir = crate::pipeline::compile_source(&source).unwrap().yir;
        assert!(scoped_inputs::carry_tests::reference(&mut yir).is_err());
    }
}

fn fixture() -> (NirModule, WordInput) {
    let words = (0..10)
        .map(|i| format!("carry{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let zeros = (0..10)
        .map(|i| format!("carry{i}: 0"))
        .collect::<Vec<_>>()
        .join(", ");
    let leaf = "Leaf { value: 1, flag: false, tag: i32_from_i64(7), gain: 2.5, scale: 1.5 }";
    let packet = format!("Packet {{ left: {leaf}, right: {leaf} }}");
    let mut module = crate::frontend::parse_nuis_module(&format!(
        "mod cpu Main {{
        struct Leaf {{ value: i64, flag: bool, tag: i32, gain: f32, scale: f64 }}
        struct Packet {{ left: Leaf, right: Leaf }}
        struct Words {{ {words} }}
        fn helper(words: Words, index: i64) -> Words {{
            let old: Packet = {packet}; let saved: Packet = old;
            let old: Packet = Packet {{ left: Leaf {{ value: saved.left.value + index,
                flag: !saved.left.flag, tag: saved.left.tag, gain: saved.left.gain,
                scale: saved.left.scale }}, right: {leaf} }};
            return Words {{ {zeros} }};
        }}
        fn entry(state: Packet) -> Packet {{ let carry = state; let i = 0;
            while i < 3 {{ let result: Words = helper(Words {{ {zeros} }}, i);
                let carry: Packet = {packet}; let i = i + 1; }} return carry;
        }} }}"
    ))
    .unwrap();
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    let shape = Shape::from_definitions(&scalar_type("Packet"), &definitions).unwrap();
    let input = WordInput {
        param: module.functions[0].params[0].clone(),
        shape,
    };
    if let NirStmt::Let { value, .. } = &mut module.functions[0].body[0] {
        *value = input.reconstruction();
    } else {
        panic!();
    }
    module.functions[0].body[3] = NirStmt::Return(Some(argument(&input, "old")));
    let NirStmt::While { body, .. } = &mut module.functions[1].body[2] else {
        panic!();
    };
    let NirStmt::Let {
        value: NirExpr::Call { args, .. },
        ..
    } = &mut body[0]
    else {
        panic!();
    };
    args[0] = argument(&input, "carry");
    let output = WordInput {
        param: NirParam {
            name: "result".into(),
            ty: input.param.ty.clone(),
        },
        shape: input.shape.clone(),
    };
    let NirStmt::Let { value, .. } = &mut body[1] else {
        panic!();
    };
    *value = output.reconstruction();
    (module, input)
}

fn argument(input: &WordInput, binding: &str) -> NirExpr {
    NirExpr::StructLiteral {
        type_name: input.param.ty.name.clone(),
        type_args: vec![],
        fields: input
            .shape
            .leaves()
            .iter()
            .enumerate()
            .map(|(index, (path, ty))| {
                (
                    format!("carry{index}"),
                    scalar_carries::encode(ty, source_value(binding, path)),
                )
            })
            .collect(),
    }
}

fn project_fixture(module: &mut NirModule) -> bool {
    let layouts = control_values::TypedLayouts::collect(module);
    let names = BTreeSet::from(["helper".into()]);
    super::super::project(module, &names, &names, &layouts, &BTreeMap::new()).changed
}

#[test]
fn caller_record_constructors_cannot_bypass_registered_word_transport_proofs() {
    let (module, input) = fixture();
    let layouts = control_values::TypedLayouts::collect(&module);
    let mut caller = module.functions[1].clone();
    let original = argument(&input, "state");
    caller.body = vec![NirStmt::Return(Some(NirExpr::Call {
        callee: "helper".into(),
        args: vec![original, NirExpr::Int(0)],
    }))];
    let mut plan = Plan {
        predicate_result: false,
        inputs: vec![
            Input::Fields(vec![Projection {
                path: vec!["carry0".into()],
                param: NirParam {
                    name: "word".into(),
                    ty: scalar_type("i64"),
                },
            }]),
            Input::Keep(module.functions[0].params[1].clone()),
        ],
        original_types: module.functions[0]
            .params
            .iter()
            .map(|param| param.ty.clone())
            .collect(),
        replacements: BTreeMap::new(),
        word_inputs: BTreeMap::new(),
    };
    let words = input.param.clone();
    plan.word_inputs.insert(0, input);
    assert!(caller_records::valid(&caller, "helper", &plan, &layouts));
    assert!(caller_spills::prepare(
        &caller,
        "helper",
        &plan,
        &layouts,
        &ScalarHelpers::new(),
        &BTreeSet::new(),
    )
    .is_none());

    caller.params = vec![words.clone()];
    let NirStmt::Return(Some(NirExpr::Call { args, .. })) = &mut caller.body[0] else {
        panic!()
    };
    let fields = (0..10)
        .map(|i| {
            let field = format!("carry{i}");
            (field.clone(), source_value(&words.name, &[field]))
        })
        .collect();
    args[0] = NirExpr::StructLiteral {
        type_name: words.ty.name.clone(),
        type_args: vec![],
        fields,
    };
    // A total, exact constructor is still not the registered codec of Packet.
    assert!(!caller_records::valid(&caller, "helper", &plan, &layouts));
    plan.word_inputs.clear();
    assert!(caller_records::valid(&caller, "helper", &plan, &layouts));
}

#[test]
fn sparse_word_projection_is_typed_transactional_and_storage_order_independent() {
    for reverse in [false, true] {
        let (mut module, _) = fixture();
        if reverse {
            module.functions.reverse();
        }
        assert!(project_fixture(&mut module));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let helper = module
            .functions
            .iter()
            .find(|f| f.name == "helper")
            .unwrap();
        assert_eq!(helper.params.len(), 6);
        assert!(helper.params.iter().all(|p| p.ty == scalar_type("i64")));
        assert!(!project_fixture(&mut module));
    }
}

#[test]
fn sparse_word_projection_rejects_unproven_codecs_paths_and_computation() {
    for change in [
        "order", "codec", "parent", "binding", "nominal", "missing", "computed",
    ] {
        let (mut module, input) = fixture();
        let NirStmt::While { body, .. } = &mut module.functions[1].body[2] else {
            panic!();
        };
        let NirStmt::Let {
            value: NirExpr::Call { args, .. },
            ..
        } = &mut body[0]
        else {
            panic!();
        };
        let NirExpr::StructLiteral {
            type_name, fields, ..
        } = &mut args[0]
        else {
            panic!();
        };
        match change {
            "order" => fields.swap(0, 5),
            "codec" => {
                fields[1].1 = NirExpr::CastI32ToI64(Box::new(source_value(
                    "carry",
                    &["left".into(), "flag".into()],
                )))
            }
            "parent" => fields[5].1 = fields[0].1.clone(),
            "binding" => fields[0].1 = source_value("saved", &["left".into(), "value".into()]),
            "nominal" => *type_name = "Packet".into(),
            "missing" => {
                fields.pop();
            }
            "computed" => {
                fields[5].1 = NirExpr::Binary {
                    op: NirBinaryOp::Div,
                    lhs: Box::new(NirExpr::Int(1)),
                    rhs: Box::new(NirExpr::Int(0)),
                }
            }
            _ => unreachable!(),
        }
        assert!(!input.valid_argument(&args[0]), "{change}");
        let before = module.clone();
        assert!(!project_fixture(&mut module), "{change}");
        assert_eq!(module, before, "{change}");
    }
}

#[test]
fn sparse_word_projection_requires_canonical_seeds_at_every_caller() {
    for computed in [false, true] {
        let (mut module, input) = fixture();
        let mut caller = module.functions[0].clone();
        caller.name = "another".into();
        let mut arg = NirExpr::Var("words".into());
        if computed {
            arg = argument(&input, "old");
            let NirExpr::StructLiteral { fields, .. } = &mut arg else {
                panic!();
            };
            fields[9].1 = NirExpr::PackF64Word(Box::new(NirExpr::Call {
                callee: "compute".into(),
                args: vec![],
            }));
        }
        caller.body = vec![NirStmt::Return(Some(NirExpr::Call {
            callee: "helper".into(),
            args: vec![arg, NirExpr::Int(0)],
        }))];
        module.functions.push(caller);
        let before = module.clone();
        assert!(!project_fixture(&mut module));
        assert_eq!(module, before);
    }
}

#[test]
fn sparse_word_projection_preserves_loop_writes_and_rejects_unproven_inputs() {
    for change in ["whole", "loop", "decode"] {
        let (mut module, _) = fixture();
        let body = &mut module.functions[0].body;
        let NirStmt::Let {
            value: computed, ..
        } = body[2].clone()
        else {
            panic!();
        };
        match change {
            "whole" => body.insert(
                1,
                NirStmt::Expr(NirExpr::Call {
                    callee: "consume".into(),
                    args: vec![NirExpr::Var("old".into())],
                }),
            ),
            "loop" => body.insert(
                2,
                NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body: vec![NirStmt::Let {
                        name: "old".into(),
                        ty: Some(scalar_type("Packet")),
                        value: computed,
                    }],
                },
            ),
            "decode" => walk::rewrite(&mut body[..1], |expr| {
                if let NirExpr::UnpackF64Word(word) = expr {
                    *expr = NirExpr::UnpackF32Word(word.clone());
                }
            }),
            _ => unreachable!(),
        }
        let before = module.clone();
        if change == "loop" {
            assert!(project_fixture(&mut module));
            assert_eq!(module.functions[0].params.len(), 6);
            assert_eq!(module.functions[0].body[1..], before.functions[0].body[1..]);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert!(!project_fixture(&mut module), "{change}");
            assert_eq!(module, before);
        }
    }
}
