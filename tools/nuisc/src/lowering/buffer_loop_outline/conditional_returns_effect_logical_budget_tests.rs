use super::*;

fn logical(edges: usize) -> NirExpr {
    (0..edges).fold(NirExpr::Bool(true), |lhs, _| NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(lhs),
        rhs: Box::new(NirExpr::Bool(true)),
    })
}

fn bind(name: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: None,
        value,
    }
}

#[test]
fn conditional_return_logical_initializers_share_original_and_expanded_root_budgets() {
    for edges in [16, 17] {
        let prefix = [
            bind("first", logical(16)),
            bind("second", logical(edges)),
            NirStmt::Print(NirExpr::Int(1)),
        ];
        let tail = [NirStmt::Return(Some(NirExpr::Bool(true)))];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), edges == 16);
        let mut body = prefix.to_vec();
        body.extend(tail);
        assert_eq!(super::super::super::super::preflight(&body, 3), edges == 16);
    }
    for edges in [10, 11] {
        let prefix = [
            bind("first", logical(edges)),
            NirStmt::Print(NirExpr::Int(1)),
        ];
        let tail = [
            NirStmt::If {
                condition: NirExpr::Bool(true),
                then_body: vec![],
                else_body: vec![],
            },
            NirStmt::Return(Some(logical(11))),
        ];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), edges == 10);
    }
    // Print roots do not inherit initializer authority, even in the same plan.
    let tail = [NirStmt::Return(Some(NirExpr::Bool(true)))];
    assert!(!suffix::reserve_staged_prefix(
        &tail,
        &[bind("first", logical(1)), NirStmt::Print(logical(1))]
    ));
    assert!(!suffix::reserve_prefix(&tail, &[&logical(1)]));
    for args in [4092, 4093] {
        let value = NirExpr::Call {
            callee: "observe".into(),
            args: vec![NirExpr::Int(1); args],
        };
        let prefix = [
            bind("first", value),
            bind("copy", NirExpr::Int(1)),
            NirStmt::Print(NirExpr::Int(1)),
        ];
        assert_eq!(suffix::reserve_staged_prefix(&tail, &prefix), args == 4092);
    }
    for depth in [63, 64] {
        let value = (0..depth).fold(NirExpr::Int(1), |lhs, _| NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(NirExpr::Int(1)),
        });
        assert_eq!(
            suffix::reserve_staged_prefix(&tail, &[bind("first", value)]),
            depth == 63
        );
    }
}

#[test]
fn conditional_return_logical_initializers_charge_removed_aliases_and_veto_before_typing() {
    for (expanded, counts) in [(false, [27, 28]), (true, [24, 25])] {
        for count in counts {
            let copies = (0..count)
                .map(|i| format!("let unused_{i} = gate;"))
                .collect::<String>();
            let mut text = source(0, "call", "return", "then", 0, INPUTS[2])
                .replace("let ready =", &format!("{copies} let ready ="));
            if expanded {
                text = text.replace(
                    "return copy;",
                    "if gate { let marker = gate; } return copy;",
                );
            }
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
    let mut module =
        crate::frontend::parse_nuis_module(&source(0, "call", "return", "both", 2, INPUTS[2]))
            .unwrap();
    let NirStmt::If { else_body, .. } = &mut event(&mut module).body[2] else {
        panic!()
    };
    let NirStmt::Const { value, .. } = &mut else_body[1] else {
        panic!()
    };
    *value = NirExpr::Binary {
        op: NirBinaryOp::And,
        lhs: Box::new(NirExpr::Bool(true)),
        rhs: Box::new(NirExpr::Call {
            callee: "observe".into(),
            args: vec![NirExpr::Int(1); 4096],
        }),
    };
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
