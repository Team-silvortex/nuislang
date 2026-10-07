use super::*;

#[test]
fn conditional_return_control_regions_charge_child_statements_and_expanded_tail() {
    for (expanded, counts) in [(false, [29, 30]), (true, [26, 27])] {
        for count in counts {
            let copies = (0..count)
                .map(|i| format!("let unused_{i} = gate;"))
                .collect::<String>();
            let text = source("computed", false, "return", "then", 2, INPUTS[2]);
            let start = text.find("let ready =").unwrap();
            let end = text[start..].find("return ready;").unwrap() + start;
            let tail = if expanded {
                "if gate { let marker = gate; } return true;"
            } else {
                "return true;"
            };
            let text = format!(
                "{}if helper(produce(value)) {{ {copies} print(88); }} {tail}{}",
                &text[..start],
                &text[end + "return ready;".len()..]
            );
            let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
            let before = module.clone();
            assert_eq!(
                outline_test(&mut module).len(),
                if count == counts[0] { 2 } else { 0 },
                "expanded={expanded} count={count}"
            );
            if count == counts[1] {
                assert_eq!(module, before);
            }
        }
    }
}

#[test]
fn conditional_return_control_regions_share_complete_root_budgets_before_typing() {
    let edge = |edges| {
        (0..edges).fold(NirExpr::Bool(true), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Bool(true)),
        })
    };
    let tail = [NirStmt::Return(Some(NirExpr::Bool(true)))];
    for edges in [16, 17] {
        let prefix = [NirStmt::If {
            condition: edge(16),
            then_body: vec![
                NirStmt::Let {
                    name: "value".into(),
                    ty: None,
                    value: edge(edges),
                },
                NirStmt::Print(NirExpr::Int(1)),
            ],
            else_body: vec![],
        }];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), edges == 16);
        let mut original = prefix.to_vec();
        original.extend(tail.clone());
        assert_eq!(
            effects::regions::bounds::preflight(&original, 1),
            edges == 16
        );
    }
    for args in [4093, 4094] {
        let prefix = [NirStmt::If {
            condition: NirExpr::Call {
                callee: "helper".into(),
                args: vec![NirExpr::Int(1); args],
            },
            then_body: vec![NirStmt::Print(NirExpr::Int(1))],
            else_body: vec![],
        }];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), args == 4093);
    }
    for depth in [63, 64] {
        let value = (0..depth).fold(NirExpr::Int(1), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Int(1)),
        });
        let prefix = [NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Print(value)],
            else_body: vec![],
        }];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), depth == 63);
    }
    // Tail fanout is charged together with every child condition and binding.
    for edges in [10, 11] {
        let prefix = [NirStmt::If {
            condition: edge(edges),
            then_body: vec![NirStmt::Print(NirExpr::Int(1))],
            else_body: vec![],
        }];
        let tail = [
            NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body: vec![],
                else_body: vec![],
            },
            NirStmt::Return(Some(edge(11))),
        ];
        let mut original = prefix.to_vec();
        original.extend(tail.clone());
        assert!(effects::regions::bounds::preflight(&original, 1));
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), edges == 10);
    }
    assert!(!suffix::reserve_staged_prefix(
        &tail,
        &[NirStmt::If {
            condition: edge(1),
            then_body: vec![NirStmt::Print(edge(1))],
            else_body: vec![],
        }],
    ));
    let mut module = crate::frontend::parse_nuis_module(&source(
        "computed", false, "return", "both", 1, INPUTS[2],
    ))
    .unwrap();
    let NirStmt::If { else_body, .. } = &mut event(&mut module).body[2] else {
        panic!()
    };
    let NirStmt::If { condition, .. } = &mut else_body[2] else {
        panic!()
    };
    *condition = NirExpr::Call {
        callee: "helper".into(),
        args: vec![NirExpr::Int(1); 4096],
    };
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
