use super::*;

#[test]
fn conditional_return_print_aliases_charge_original_and_expanded_statements_before_elision() {
    let base = source("return", "atom", "literal", "then", INPUTS[2]);
    for count in [30, 31] {
        let copies = (0..count)
            .map(|i| format!("let unused_{i} = left;"))
            .collect::<String>();
        let text = base.replace("print(88);", &format!("{copies} print(88);"));
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), usize::from(count == 30));
        if count == 31 {
            assert_eq!(module, before);
        }
    }
    for count in [27, 28] {
        let copies = (0..count)
            .map(|i| format!("let unused_{i} = left;"))
            .collect::<String>();
        let text = base
            .replace("print(88);", &format!("{copies} print(88);"))
            .replace(
                "return helper(produce(left)) && (gate || helper(produce(right)));",
                "if gate { let marker = left; } return helper(produce(right));",
            );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let before = module.clone();
        let generated = outline_test(&mut module);
        assert_eq!(generated.len(), usize::from(count == 27));
        if count == 28 {
            assert_eq!(module, before);
        }
    }
}

#[test]
fn conditional_return_print_aliases_charge_initializers_and_complete_argument_nodes() {
    let tail = [NirStmt::Return(Some(NirExpr::Bool(true)))];
    let atom = NirExpr::Int(1);
    for args in [4092, 4093] {
        let value = NirExpr::Call {
            callee: "observe".into(),
            args: vec![NirExpr::Int(1); args],
        };
        let body = [
            NirStmt::Let {
                name: "alias".into(),
                ty: None,
                value: atom.clone(),
            },
            NirStmt::Print(value.clone()),
            NirStmt::Print(atom.clone()),
            tail[0].clone(),
        ];
        assert_eq!(super::super::super::preflight(&body, 3), args == 4092);
        assert_eq!(
            suffix::reserve_prefix(&tail, &[&atom, &value, &atom]),
            args == 4092
        );
    }
}
