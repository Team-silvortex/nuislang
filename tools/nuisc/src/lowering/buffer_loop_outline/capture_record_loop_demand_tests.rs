use super::*;

#[test]
fn materialized_word_loop_returns_keep_pending_values_and_control_seeds() {
    let source = include_str!(
        "../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    );
    let mut yir = crate::pipeline::compile_source(source).unwrap().yir;
    let calls = yir
        .nodes
        .iter()
        .filter_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(calls.iter().map(|call| call.seeds.len()).max(), Some(9));
    let mut widths = calls
        .iter()
        .map(|call| call.seeds.len())
        .collect::<Vec<_>>();
    widths.sort_unstable();
    assert_eq!(widths, [8, 9]);
    assert_eq!(calls.iter().filter(|call| call.break_on_return).count(), 2);
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
}

#[test]
fn materialized_word_loops_compile_nested_source_with_complete_seeds() {
    let source = include_str!(
        "../../../tests/control_flow_syntax_native/scoped_loop_typed_record_carries.ns"
    );
    let mut yir = crate::pipeline::compile_source(source).unwrap().yir;
    let call = yir
        .nodes
        .iter()
        .filter_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
        })
        .find(|call| call.seeds.len() == 9)
        .unwrap();
    assert_eq!(call.operands.len(), 6);
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
}

fn binding(name: &str, ty: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type(ty)),
        value,
    }
}

fn path(root: &str, fields: &[&str]) -> NirExpr {
    source_value(
        root,
        &fields.iter().map(|s| (*s).into()).collect::<Vec<_>>(),
    )
}

fn constant_leaf() -> NirExpr {
    let (module, _) = fixture();
    control_values::zero_value(
        &scalar_type("Leaf"),
        &control_values::TypedLayouts::collect(&module),
    )
}

fn result(value: NirExpr) -> NirStmt {
    NirStmt::Return(Some(NirExpr::StructLiteral {
        type_name: "Words".into(),
        type_args: vec![],
        fields: (0..10)
            .map(|i| {
                (
                    format!("carry{i}"),
                    if i == 0 {
                        value.clone()
                    } else {
                        NirExpr::Int(0)
                    },
                )
            })
            .collect(),
    }))
}

fn loop_fixture(body: Vec<NirStmt>) -> NirModule {
    let (mut module, _) = fixture();
    let helper = &mut module.functions[0];
    helper.body.truncate(1);
    helper.body.extend([
        binding("left", "Leaf", path("old", &["left"])),
        binding("right", "Leaf", path("old", &["right"])),
        NirStmt::While {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Lt,
                lhs: Box::new(NirExpr::Int(0)),
                rhs: Box::new(path("index", &[])),
            },
            body,
        },
        result(path("left", &["value"])),
    ]);
    module
}

fn check(mut module: NirModule, params: usize) {
    let tail = module.functions[0].body[1..].to_vec();
    assert!(project_fixture(&mut module));
    assert_eq!(module.functions[0].params.len(), params);
    assert_eq!(module.functions[0].body[1..], tail);
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert!(!project_fixture(&mut module));
}

#[test]
fn materialized_word_loops_reach_a_fixed_point_for_cyclic_snapshots() {
    check(
        loop_fixture(vec![
            binding("before", "Leaf", path("left", &[])),
            binding("left", "Leaf", path("right", &[])),
            binding("right", "Leaf", path("before", &[])),
        ]),
        3,
    );
}

#[test]
fn materialized_word_loops_execute_cyclic_snapshots_before_and_after_projection() {
    let (mut module, mut input) = fixture();
    // Re-enter source-level lowering with a word marker: private bool casts
    // intentionally do not grant source admission. Native fixtures cover bool.
    module
        .structs
        .iter_mut()
        .find(|d| d.name == "Leaf")
        .unwrap()
        .fields[1]
        .ty = scalar_type("i64");
    for function in &mut module.functions {
        walk::rewrite(&mut function.body, |expr| {
            if let NirExpr::CastI64ToBool(word) | NirExpr::CastBoolToI64(word) = expr {
                *expr = *word.clone();
            }
        });
    }
    let definitions = module
        .structs
        .iter()
        .map(|d| (d.name.as_str(), d))
        .collect();
    input.shape = Shape::from_definitions(&scalar_type("Packet"), &definitions).unwrap();
    let mut seed = control_values::zero_value(
        &scalar_type("Packet"),
        &control_values::TypedLayouts::collect(&module),
    );
    let mut swapped = seed.clone();
    let NirExpr::StructLiteral { fields, .. } = &mut swapped else {
        panic!()
    };
    for ((_, leaf), source) in fields.iter_mut().zip(["right", "left"]) {
        let NirExpr::StructLiteral { fields, .. } = leaf else {
            panic!()
        };
        fields[0].1 = path("before", &[source, "value"]);
    }
    module.functions[0].body.truncate(1);
    module.functions[0].body.extend([
        binding("current", "Packet", path("old", &[])),
        binding("i", "i64", NirExpr::Int(0)),
        NirStmt::While {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Lt,
                lhs: Box::new(path("i", &[])),
                rhs: Box::new(path("index", &[])),
            },
            body: vec![
                binding(
                    "i",
                    "i64",
                    NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(path("i", &[])),
                        rhs: Box::new(NirExpr::Int(1)),
                    },
                ),
                binding("current", "Packet", path("current", &[])),
                binding("before", "Packet", path("current", &[])),
                binding("current", "Packet", swapped),
            ],
        },
        result(path("current", &["left", "value"])),
    ]);
    let NirExpr::StructLiteral { fields, .. } = &mut seed else {
        panic!()
    };
    for ((_, leaf), value) in fields.iter_mut().zip([12, 33]) {
        let NirExpr::StructLiteral { fields, .. } = leaf else {
            panic!()
        };
        fields[0].1 = NirExpr::Int(value);
    }
    let mut main =
        crate::frontend::parse_nuis_module("mod cpu Main { fn main() -> i64 { return 0; } }")
            .unwrap()
            .functions
            .remove(0);
    main.body = vec![binding("seed", "Packet", seed)];
    for limit in 0..=4 {
        let name = format!("result{limit}");
        main.body.push(binding(
            &name,
            "Words",
            NirExpr::Call {
                callee: "helper".into(),
                args: vec![argument(&input, "seed"), NirExpr::Int(limit)],
            },
        ));
        main.body.push(NirStmt::Print(path(&name, &["carry0"])));
    }
    main.body.push(NirStmt::Return(Some(NirExpr::Int(0))));
    module.functions.push(main);
    for projected in [false, true] {
        if projected {
            assert!(project_fixture(&mut module));
            assert_eq!(module.functions[0].params.len(), 3);
        }
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&module).unwrap();
        yir_lower_llvm::emit_module(&yir).unwrap();
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .unwrap();
        let printed = trace
            .events
            .iter()
            .filter(|e| e.contains("cpu.print"))
            .map(|e| e.rsplit_once(": ").unwrap().1.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(printed, [12, 33, 12, 33, 12]);
    }
}

#[test]
fn materialized_word_loops_keep_zero_trip_break_continue_and_return_demands() {
    for exit in [
        NirStmt::Break,
        NirStmt::Continue,
        result(path("left", &["value"])),
    ] {
        check(
            loop_fixture(vec![
                binding("left", "Leaf", path("right", &[])),
                exit,
                binding("left", "Leaf", constant_leaf()),
                NirStmt::Expr(path("old", &[])),
            ]),
            3,
        );
    }
}

#[test]
fn materialized_word_loops_bind_breaks_to_the_nearest_loop() {
    check(
        loop_fixture(vec![
            NirStmt::While {
                condition: NirExpr::Bool(true),
                body: vec![binding("left", "Leaf", path("right", &[])), NirStmt::Break],
            },
            binding("left", "Leaf", constant_leaf()),
        ]),
        2,
    );
}

#[test]
fn materialized_word_loops_bind_continues_to_the_nearest_header() {
    check(
        loop_fixture(vec![
            NirStmt::While {
                condition: path("right", &["flag"]),
                body: vec![
                    binding("left", "Leaf", path("right", &[])),
                    NirStmt::Continue,
                ],
            },
            binding("left", "Leaf", constant_leaf()),
        ]),
        3,
    );
}

#[test]
fn materialized_word_loops_reject_exits_without_a_loop_target() {
    for exit in [NirStmt::Break, NirStmt::Continue] {
        let mut module = loop_fixture(vec![binding("left", "Leaf", path("right", &[]))]);
        module.functions[0].body.insert(4, exit);
        let before = module.clone();
        assert!(!project_fixture(&mut module));
        assert_eq!(module, before);
    }
}

#[test]
fn materialized_word_loops_observe_headers_and_unused_checked_fields() {
    let mut module = loop_fixture(vec![binding("left", "Leaf", path("right", &[]))]);
    let NirStmt::While { condition, body } = &mut module.functions[0].body[3] else {
        panic!()
    };
    *condition = path("left", &["flag"]);
    let mut checked = constant_leaf();
    let NirExpr::StructLiteral { fields, .. } = &mut checked else {
        panic!()
    };
    fields[0].1 = NirExpr::Binary {
        op: NirBinaryOp::Div,
        lhs: Box::new(NirExpr::Int(1)),
        rhs: Box::new(NirExpr::CastI32ToI64(Box::new(path(
            "old",
            &["right", "tag"],
        )))),
    };
    body.push(binding("checked", "Leaf", checked));
    // Both value and flag are needed for each original record; the unused
    // constructor additionally observes the original right tag.
    check(module, 6);
}

#[test]
fn materialized_word_loops_keep_whole_input_and_all_caller_proofs() {
    for whole in [false, true] {
        let mut module = loop_fixture(vec![binding("left", "Leaf", path("right", &[]))]);
        if whole {
            let NirStmt::While { body, .. } = &mut module.functions[0].body[3] else {
                panic!()
            };
            body.push(NirStmt::Expr(NirExpr::Call {
                callee: "consume".into(),
                args: vec![path("old", &[])],
            }));
        } else {
            let mut caller = module.functions[1].clone();
            caller.name = "second_entry".into();
            walk::rewrite(&mut caller.body, |expr| {
                if let NirExpr::Call { callee, args } = expr {
                    if callee == "helper" {
                        args[0] = path("carry", &[]);
                    }
                }
            });
            module.functions.push(caller);
        }
        let before = module.clone();
        assert!(!project_fixture(&mut module));
        assert_eq!(module, before);
    }
}
