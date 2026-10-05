use super::*;

#[test]
fn conditional_return_staged_initializers_charge_original_and_expanded_statements() {
    for (expanded, counts) in [(false, [29, 30]), (true, [26, 27])] {
        for count in counts {
            let copies = (0..count)
                .map(|i| format!("let unused_{i} = stamp;"))
                .collect::<String>();
            let mut text = simple_source("checked", true, 2)
                .replace("let value =", &format!("{copies} let value ="));
            if expanded {
                text = text.replace(
                    "return value > 0;",
                    "if outer { let marker = stamp; } return value > 0;",
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
}

#[test]
fn conditional_return_staged_initializers_charge_complete_roots_before_typing() {
    let tail = [NirStmt::Return(Some(NirExpr::Bool(true)))];
    let atom = NirExpr::Int(1);
    for args in [4092, 4093] {
        let value = NirExpr::Call {
            callee: "observe".into(),
            args: vec![atom.clone(); args],
        };
        let body = [
            NirStmt::Let {
                name: "local".into(),
                ty: None,
                value: value.clone(),
            },
            NirStmt::Let {
                name: "copy".into(),
                ty: None,
                value: atom.clone(),
            },
            NirStmt::Print(atom.clone()),
            tail[0].clone(),
        ];
        assert_eq!(super::super::super::preflight(&body, 3), args == 4092);
        assert_eq!(
            suffix::reserve_prefix(&tail, &[&value, &atom, &atom]),
            args == 4092
        );
    }
    for depth in [63, 64] {
        let mut value = NirExpr::Int(1);
        for _ in 0..depth {
            value = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(value),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        assert_eq!(conditional_values::prefix::expression(&value), depth == 63);
        assert_eq!(suffix::reserve_prefix(&tail, &[&value]), depth == 63);
    }
    // Over-budget, ill-typed initializer roots veto before cloning or recursive inference.
    let text = staged_source("call", "return", "atom", "both", INPUTS[2], 2);
    let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
    let NirStmt::If { else_body, .. } = &mut event(&mut module).body[2] else {
        panic!()
    };
    let NirStmt::Const { value, .. } = &mut else_body[1] else {
        panic!()
    };
    *value = NirExpr::Call {
        callee: "observe".into(),
        args: vec![NirExpr::Int(1); 4096],
    };
    let before = module.clone();
    assert!(outline_test(&mut module).is_empty());
    assert_eq!(module, before);
}
