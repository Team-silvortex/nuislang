use super::*;

fn binding(name: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.to_owned(),
        ty: None,
        value,
    }
}

#[test]
fn dynamic_branch_invalidates_rebound_literals_without_pruning_live_arm_state() {
    for other in [vec![], vec![binding("value", NirExpr::Int(5))]] {
        let mut module = sample_module(vec![
            binding("value", NirExpr::Int(3)),
            NirStmt::If {
                condition: NirExpr::Var("flag".to_owned()),
                then_body: vec![binding("value", NirExpr::Int(7))],
                else_body: other,
            },
            NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
        ]);
        simplify_nir_module(&mut module);
        let body = &module.functions[0].body;
        assert_eq!(
            body.last(),
            Some(&NirStmt::Return(Some(NirExpr::Var("value".to_owned()))))
        );
        let branch = body
            .iter()
            .find_map(|stmt| match stmt {
                NirStmt::If { then_body, .. } => Some(then_body),
                _ => None,
            })
            .unwrap();
        assert_eq!(branch, &vec![binding("value", NirExpr::Int(7))]);
    }
}

#[test]
fn selected_constant_branch_uses_its_outgoing_environment() {
    for selected in [false, true] {
        let mut module = sample_module(vec![
            binding("value", NirExpr::Int(3)),
            NirStmt::If {
                condition: NirExpr::Bool(selected),
                then_body: vec![binding("value", NirExpr::Int(7))],
                else_body: vec![binding("value", NirExpr::Int(5))],
            },
            NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
        ]);
        simplify_nir_module(&mut module);
        assert_eq!(
            module.functions[0].body,
            vec![NirStmt::Return(Some(NirExpr::Int(if selected {
                7
            } else {
                5
            })))]
        );
    }
}

#[test]
fn nested_loop_mutation_invalidates_the_enclosing_branch_literal() {
    let mut module = sample_module(vec![
        binding("value", NirExpr::Int(3)),
        NirStmt::If {
            condition: NirExpr::Var("flag".to_owned()),
            then_body: vec![NirStmt::While {
                condition: NirExpr::Binary {
                    op: NirBinaryOp::Lt,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(7)),
                },
                body: vec![binding(
                    "value",
                    NirExpr::Binary {
                        op: NirBinaryOp::Add,
                        lhs: Box::new(NirExpr::Var("value".to_owned())),
                        rhs: Box::new(NirExpr::Int(1)),
                    },
                )],
            }],
            else_body: vec![],
        },
        NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
    ]);
    simplify_nir_module(&mut module);
    assert_eq!(
        module.functions[0].body.last(),
        Some(&NirStmt::Return(Some(NirExpr::Var("value".to_owned()))))
    );
}

#[test]
fn dynamic_branch_keeps_unchanged_literals_without_leaking_branch_locals() {
    let mut module = sample_module(vec![
        binding("value", NirExpr::Int(3)),
        NirStmt::If {
            condition: NirExpr::Var("flag".to_owned()),
            then_body: vec![
                binding("local", NirExpr::Int(7)),
                NirStmt::Print(NirExpr::Var("local".to_owned())),
            ],
            else_body: vec![],
        },
        NirStmt::Print(NirExpr::Var("local".to_owned())),
        NirStmt::Return(Some(NirExpr::Var("value".to_owned()))),
    ]);
    simplify_nir_module(&mut module);
    let body = &module.functions[0].body;
    assert_eq!(body.last(), Some(&NirStmt::Return(Some(NirExpr::Int(3)))));
    assert_eq!(
        body[body.len() - 2],
        NirStmt::Print(NirExpr::Var("local".to_owned()))
    );
}
