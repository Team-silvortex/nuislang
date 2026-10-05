use super::*;

#[test]
fn conditional_return_computed_prints_share_complete_argument_and_tail_budgets() {
    for args in [4092, 4093] {
        let value = NirExpr::Call {
            callee: "observe".into(),
            args: vec![NirExpr::Int(1); args],
        };
        let tail = vec![NirStmt::Return(Some(NirExpr::Bool(true)))];
        let mut body = vec![NirStmt::Print(value.clone())];
        body.extend(tail.clone());
        // Call + all original args + tail + second print: 4095/4096.
        // Adding a third print consumes the final node or exceeds the budget.
        body.insert(1, NirStmt::Print(NirExpr::Int(9)));
        assert!(super::super::super::preflight(&body, 2));
        assert!(suffix::reserve_prefix(&tail, &[&value, &NirExpr::Int(9)]));
        let extra = NirExpr::Int(8);
        body.insert(2, NirStmt::Print(extra.clone()));
        assert_eq!(super::super::super::preflight(&body, 3), args == 4092);
        assert_eq!(
            suffix::reserve_prefix(&tail, &[&value, &NirExpr::Int(9), &extra]),
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
        assert_eq!(
            suffix::reserve_prefix(&[NirStmt::Return(Some(NirExpr::Bool(true)))], &[&value]),
            depth == 63
        );
    }
}
