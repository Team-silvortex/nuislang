use super::*;

#[test]
fn materialized_word_joins_compile_source_with_complete_current_state() {
    let source = include_str!(
        "../../../tests/control_flow_syntax_native/scoped_join_typed_record_carries.ns"
    );
    let mut yir = crate::pipeline::compile_source(source).unwrap().yir;
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
    assert_eq!(call.operands.len(), 6);
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
}

fn fixture_with_join(two_arms: bool, constant_value: bool) -> NirModule {
    let (mut module, _) = fixture();
    let mut updated = module.functions[0].body[2].clone();
    let NirStmt::Let { name, value, .. } = &mut updated else {
        panic!()
    };
    *name = "saved".into();
    if constant_value {
        let NirExpr::StructLiteral { fields, .. } = value else {
            panic!()
        };
        let NirExpr::StructLiteral { fields, .. } = &mut fields[0].1 else {
            panic!()
        };
        fields[0].1 = NirExpr::Int(41);
    }
    module.functions[0].body.insert(
        2,
        NirStmt::If {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Eq,
                lhs: Box::new(NirExpr::Var("index".into())),
                rhs: Box::new(NirExpr::Int(0)),
            },
            then_body: vec![updated.clone()],
            else_body: if two_arms { vec![updated] } else { vec![] },
        },
    );
    module
}

#[test]
fn materialized_word_joins_project_input_without_erasing_outer_assignments() {
    for reverse in [false, true] {
        let mut module = fixture_with_join(false, false);
        let original = module.functions[0].body.clone();
        if reverse {
            module.functions.reverse();
        }
        assert!(project_fixture(&mut module));
        let helper = module
            .functions
            .iter()
            .find(|f| f.name == "helper")
            .unwrap();
        assert_eq!(helper.params.len(), 6);
        assert_eq!(helper.body[1..], original[1..]);
        assert_ne!(helper.body[0], original[0]);
        crate::nir_verify::verify_nir_module(&module).unwrap();
        assert!(!project_fixture(&mut module));
    }
}

#[test]
fn materialized_word_joins_union_fallthrough_reads_and_kill_overwritten_versions() {
    for both in [false, true] {
        let mut module = fixture_with_join(both, true);
        assert!(project_fixture(&mut module));
        assert_eq!(module.functions[0].params.len(), if both { 5 } else { 6 });
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn materialized_word_joins_keep_all_caller_codec_proof_transactional() {
    for change in ["raw", "computed", "path"] {
        let mut module = fixture_with_join(false, false);
        let mut second = module.functions[1].clone();
        second.name = "another_entry".into();
        walk::rewrite(&mut second.body, |expr| {
            let NirExpr::Call { callee, args } = expr else {
                return;
            };
            if callee != "helper" {
                return;
            }
            if change == "raw" {
                args[0] = NirExpr::Var("carry".into());
                return;
            }
            let NirExpr::StructLiteral { fields, .. } = &mut args[0] else {
                panic!()
            };
            fields[5].1 = if change == "path" {
                fields[0].1.clone()
            } else {
                NirExpr::Binary {
                    op: NirBinaryOp::Div,
                    lhs: Box::new(NirExpr::Int(1)),
                    rhs: Box::new(NirExpr::Int(0)),
                }
            };
        });
        module.functions.push(second);
        let before = module.clone();
        assert!(!project_fixture(&mut module), "{change}");
        assert_eq!(module, before);
    }
}

#[test]
fn materialized_word_joins_keep_unused_checked_field_dependencies() {
    let mut module = fixture_with_join(false, false);
    let NirStmt::If { then_body, .. } = &mut module.functions[0].body[2] else {
        panic!()
    };
    let NirStmt::Let {
        value: NirExpr::StructLiteral { fields, .. },
        ..
    } = &mut then_body[0]
    else {
        panic!()
    };
    let NirExpr::StructLiteral { fields, .. } = &mut fields[1].1 else {
        panic!()
    };
    fields[0].1 = NirExpr::Binary {
        op: NirBinaryOp::Div,
        lhs: Box::new(NirExpr::Int(1)),
        rhs: Box::new(source_value("old", &["right".into(), "value".into()])),
    };
    let before = module.functions[0].body.clone();
    assert!(project_fixture(&mut module));
    assert_eq!(module.functions[0].params.len(), 7);
    assert_eq!(module.functions[0].body[1..], before[1..]);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn materialized_word_joins_analyze_hygienic_copies_without_renaming_control() {
    let mut module = fixture_with_join(false, false);
    module.functions[0].body.insert(
        2,
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![
                NirStmt::Let {
                    name: "child".into(),
                    ty: Some(scalar_type("Leaf")),
                    value: source_value("saved", &["left".into()]),
                },
                NirStmt::Expr(source_value("child", &["value".into()])),
            ],
            else_body: vec![
                NirStmt::Let {
                    name: "child".into(),
                    ty: Some(scalar_type("Packet")),
                    value: NirExpr::Var("saved".into()),
                },
                NirStmt::Expr(source_value("child", &["left".into(), "tag".into()])),
            ],
        },
    );
    let before = module.functions[0].body.clone();
    assert!(project_fixture(&mut module));
    assert_eq!(module.functions[0].params.len(), 6);
    assert_eq!(module.functions[0].body[1..], before[1..]);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}

#[test]
fn materialized_word_joins_keep_early_return_and_opaque_full_input_demand() {
    for opaque in [false, true] {
        let mut module = fixture_with_join(false, false);
        let (_, input) = fixture();
        let NirStmt::If { then_body, .. } = &mut module.functions[0].body[2] else {
            panic!()
        };
        then_body.insert(
            0,
            if opaque {
                NirStmt::Expr(NirExpr::Call {
                    callee: "consume".into(),
                    args: vec![NirExpr::Var("saved".into())],
                })
            } else {
                NirStmt::Return(Some(argument(&input, "saved")))
            },
        );
        let before = module.clone();
        assert!(!project_fixture(&mut module));
        assert_eq!(module, before);
    }
}
