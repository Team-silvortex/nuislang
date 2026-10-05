use super::*;

fn chain(edges: usize) -> NirExpr {
    let mut value = NirExpr::Call {
        callee: "helper".into(),
        args: vec![NirExpr::Call {
            callee: "produce".into(),
            args: vec![NirExpr::Var("left".into())],
        }],
    };
    for _ in 0..edges {
        value = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Var("gate".into())),
            rhs: Box::new(value),
        };
    }
    value
}

#[test]
fn conditional_logical_trees_share_edge_node_and_depth_budgets_before_recursive_work() {
    for edges in [32, 33] {
        let value = chain(edges);
        assert_eq!(
            conditional_values::prefix::computed_logical_root(&value),
            edges == 32
        );
        assert!(!conditional_values::prefix::expression_roots(vec![(
            &value, 0, true
        )]));
        let mut module =
            crate::frontend::parse_nuis_module(&source("return", &trees()[0], INPUTS[2])).unwrap();
        let NirStmt::If { then_body, .. } = &mut event(&mut module).body[2] else {
            panic!()
        };
        then_body[0] = NirStmt::Return(Some(value));
        let before = module.clone();
        let generated = outline_values(&mut module);
        assert_eq!(generated.len(), if edges == 32 { 32 } else { 0 });
        if edges == 32 {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert_eq!(module, before);
        }
    }
    let edge = chain(2);
    assert!(conditional_values::prefix::computed_expression_roots(
        vec![(&edge, 0, true); 16]
    ));
    assert!(!conditional_values::prefix::computed_expression_roots(
        vec![(&edge, 0, true); 17]
    ));
    for args in [4091, 4092] {
        let value = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Call {
                callee: "decide".into(),
                args: vec![NirExpr::Bool(true); args],
            }),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Or,
                lhs: Box::new(NirExpr::Bool(false)),
                rhs: Box::new(NirExpr::Bool(true)),
            }),
        };
        // Arguments + call + two logical edges + two RHS atoms = 4096/4097.
        assert_eq!(
            conditional_values::prefix::computed_logical_root(&value),
            args == 4091
        );
        assert!(!conditional_values::prefix::computed_expression_roots(
            vec![(&value, 0, true), (&NirExpr::Bool(true), 0, false)]
        ));
    }
    for depth in [61, 62] {
        let mut argument = NirExpr::Int(1);
        for _ in 0..depth {
            argument = NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(argument),
                rhs: Box::new(NirExpr::Int(1)),
            };
        }
        let value = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Call {
                callee: "decide".into(),
                args: vec![argument],
            }),
            rhs: Box::new(NirExpr::Bool(true)),
        };
        assert_eq!(
            conditional_values::prefix::computed_logical_root(&value),
            depth == 61
        );
    }
}

#[test]
fn conditional_logical_trees_bound_original_and_expanded_return_suffix_fanout() {
    let tree = Tree::edge(
        false,
        Tree::Gate,
        Tree::edge(true, Tree::Gate, Tree::Leaf(0)),
    );
    let base = source("return", &tree, INPUTS[2]);
    for count in [8, 9] {
        let suffix = (0..count)
            .map(|n| format!("let item{n} = {};", tree.source()))
            .collect::<String>();
        let text = base.replace(&format!("return {};", tree.source()), &format!("if gate {{ let branch_left = gate; }} else {{ let branch_right = gate; }} {suffix} return false;"));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        // The original has 16/18 edges; expansion shares 32/36, not a fresh
        // per-root budget for each copy of the common suffix.
        assert_eq!(outline_test(&mut module).is_empty(), count == 9);
        if count == 8 {
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_logical_trees_do_not_authorize_logical_nodes_inside_ordinary_leaves() {
    let edge = chain(2);
    let leaves = [
        NirExpr::Call {
            callee: "decide".into(),
            args: vec![edge.clone()],
        },
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(edge.clone()),
            rhs: Box::new(NirExpr::Bool(true)),
        },
        NirExpr::StructLiteral {
            type_name: "Packet".into(),
            type_args: vec![],
            fields: vec![("value".into(), edge.clone())],
        },
    ];
    for leaf in leaves {
        let value = NirExpr::Binary {
            op: NirBinaryOp::And,
            lhs: Box::new(NirExpr::Bool(true)),
            rhs: Box::new(leaf),
        };
        assert!(!conditional_values::prefix::computed_logical_root(&value));
    }
    assert!(!conditional_values::prefix::computed_expression_roots(
        vec![(&edge, 0, false)]
    ));
}
