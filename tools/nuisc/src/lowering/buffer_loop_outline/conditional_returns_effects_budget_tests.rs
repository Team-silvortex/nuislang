use super::*;

#[test]
fn conditional_return_effects_reserve_original_and_expanded_statement_budget_atomically() {
    let base = source("return", "atom", "literal", "then", INPUTS[2]);
    let root = "helper(produce(left)) && (gate || helper(produce(right)))";
    for count in [30, 31] {
        let locals = (0..count)
            .map(|n| format!("let item{n} = gate;"))
            .collect::<String>();
        let text = base.replace(
            &format!("return {root};"),
            &format!("{locals} return {root};"),
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        assert_eq!(outline_test(&mut module).is_empty(), count == 31);
        if count == 30 {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert_eq!(module, before);
        }
    }
    let locals = (0..13)
        .map(|n| format!("let item{n} = gate;"))
        .collect::<String>();
    for prints in [1, 2] {
        let prefix = "print(88);".repeat(prints);
        let text = base.replace(&format!("print(88); return {root};"), &format!("{prefix} if gate {{ let yes = gate; }} else {{ let no = gate; }} {locals} return {root};"));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        // Expanded pure plan: If + 2 branch locals + 2*(13 locals + Return) = 31.
        assert_eq!(outline_test(&mut module).is_empty(), prints == 2);
        if prints == 1 {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_return_effects_share_node_edge_and_depth_budgets_with_print_atoms() {
    for args in [4092, 4093] {
        let body = vec![
            NirStmt::Print(NirExpr::Int(1)),
            NirStmt::Return(Some(NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(NirExpr::Call {
                    callee: "decide".into(),
                    args: vec![NirExpr::Bool(true); args],
                }),
                rhs: Box::new(NirExpr::Bool(true)),
            })),
        ];
        // Print + binary + call + RHS + arguments share 4096/4097 nodes.
        assert_eq!(super::super::preflight(&body, 1), args == 4092);
        assert_eq!(
            suffix::reserve_prefix(&body[1..], &[&NirExpr::Int(1)]),
            args == 4092
        );
    }
    for edges in [32, 33] {
        let mut value = NirExpr::Call {
            callee: "helper".into(),
            args: vec![NirExpr::Var("packet".into())],
        };
        for _ in 0..edges {
            value = NirExpr::Binary {
                op: NirBinaryOp::And,
                lhs: Box::new(NirExpr::Var("gate".into())),
                rhs: Box::new(value),
            };
        }
        let body = vec![
            NirStmt::Print(NirExpr::Int(1)),
            NirStmt::Return(Some(value)),
        ];
        assert_eq!(super::super::preflight(&body, 1), edges == 32);
        assert_eq!(
            suffix::reserve_prefix(&body[1..], &[&NirExpr::Int(1)]),
            edges == 32
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
        let body = vec![
            NirStmt::Print(NirExpr::Int(1)),
            NirStmt::Return(Some(value)),
        ];
        assert_eq!(super::super::preflight(&body, 1), depth == 63);
        assert_eq!(
            suffix::reserve_prefix(&body[1..], &[&NirExpr::Int(1)]),
            depth == 63
        );
    }
}
