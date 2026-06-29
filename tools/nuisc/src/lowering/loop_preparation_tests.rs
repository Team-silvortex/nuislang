use super::*;
use crate::frontend::parse_nuis_module;
use crate::lowering::loop_carries::tail_recursive_prev_carry_binding;

#[test]
fn diagnose_unsupported_stmt_carry_tree_allows_previous_value_keep_branch() {
    let stmt = NirStmt::If {
        condition: NirExpr::Binary {
            op: NirBinaryOp::Gt,
            lhs: Box::new(NirExpr::Var("current".to_owned())),
            rhs: Box::new(NirExpr::Int(1)),
        },
        then_body: vec![NirStmt::Let {
            name: "acc".to_owned(),
            ty: None,
            value: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("acc".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            },
        }],
        else_body: vec![NirStmt::Let {
            name: "acc".to_owned(),
            ty: None,
            value: NirExpr::Var(tail_recursive_prev_carry_binding(0)),
        }],
    };

    let diagnostic = diagnose_unsupported_stmt_carry_tree(
        &stmt,
        "acc",
        "current",
        &[],
        &BTreeSet::new(),
        &BTreeMap::new(),
    );
    assert!(diagnostic.is_none());
}

#[test]
fn extracts_loop_carry_name_from_if_expression_with_branch_local_temp_prefix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              let branch_value: i64 = if value > 2 {
                let picked: i64 = value;
                picked
              } else {
                let picked: i64 = 0;
                picked
              };
              let acc: i64 = acc + branch_value;
              if acc > 8 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    let NirStmt::While { body, .. } = function
        .body
        .iter()
        .find(|stmt| matches!(stmt, NirStmt::While { .. }))
        .expect("expected while body")
    else {
        unreachable!();
    };

    let branch_stmt = &body[1];
    assert_eq!(
        extract_non_temp_loop_carry_name(branch_stmt, &BTreeSet::new(), &BTreeMap::new())
            .as_deref(),
        Some("branch_value"),
        "loop body shape was: {body:#?}"
    );
}

#[test]
fn prepares_async_post_flow_carry_sequence_with_shared_suffix_branch_value() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              let branch_value: i64 = if value > 2 {
                let picked: i64 = value;
                picked
              } else {
                let picked: i64 = 0;
                picked
              };
              let acc: i64 = acc + branch_value;
              if acc > 8 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    let NirStmt::While { body, .. } = function
        .body
        .iter()
        .find(|stmt| matches!(stmt, NirStmt::While { .. }))
        .expect("expected while body")
    else {
        unreachable!();
    };

    let middle = &body[1..body.len() - 1];
    let prepared = prepare_loop_carry_sequence(
        middle,
        "value",
        &BTreeSet::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .expect("expected carry sequence to prepare");
    assert_eq!(
        prepared
            .iter()
            .map(|carry| carry.binding_name.as_str())
            .collect::<Vec<_>>(),
        vec!["acc"]
    );
}

#[test]
fn prepares_async_post_flow_carry_sequence_with_derived_conditional_temp_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              let branch_value: i64 = if value > 2 {
                let picked: i64 = value;
                picked
              } else {
                let picked: i64 = 0;
                picked
              };
              let widened: i64 = branch_value + 1;
              let acc: i64 = acc + widened;
              if acc > 8 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    let NirStmt::While { body, .. } = function
        .body
        .iter()
        .find(|stmt| matches!(stmt, NirStmt::While { .. }))
        .expect("expected while body")
    else {
        unreachable!();
    };

    let middle = &body[1..body.len() - 1];
    let prepared = prepare_loop_carry_sequence(
        middle,
        "value",
        &BTreeSet::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .expect("expected derived conditional temp carry sequence to prepare");
    assert_eq!(
        prepared
            .iter()
            .map(|carry| carry.binding_name.as_str())
            .collect::<Vec<_>>(),
        vec!["acc"]
    );
}

#[test]
fn prepares_async_post_flow_carry_sequence_with_remixed_loop_state_suffix() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          async fn step(value: i64) -> i64 {
            return value + 1;
          }

          async fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 7 {
              let value: i64 = await step(value);
              let branch_value: i64 = if value > 2 {
                let picked: i64 = value;
                picked
              } else {
                let picked: i64 = 0;
                picked
              };
              let widened: i64 = branch_value + 1;
              let normalized: i64 = widened + value;
              let acc: i64 = acc + normalized;
              if acc > 8 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("expected main function");
    let NirStmt::While { body, .. } = function
        .body
        .iter()
        .find(|stmt| matches!(stmt, NirStmt::While { .. }))
        .expect("expected while body")
    else {
        unreachable!();
    };

    let middle = &body[1..body.len() - 1];
    let prepared = prepare_loop_carry_sequence(
        middle,
        "value",
        &BTreeSet::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .expect("expected remixed loop-state carry sequence to prepare");
    assert_eq!(
        prepared
            .iter()
            .map(|carry| carry.binding_name.as_str())
            .collect::<Vec<_>>(),
        vec!["acc"]
    );
}

#[test]
fn parses_derived_conditional_temp_binding_with_loop_state_remix() {
    let stmt = NirStmt::Let {
        name: "normalized".to_owned(),
        ty: None,
        value: NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("widened".to_owned())),
            rhs: Box::new(NirExpr::Var("value".to_owned())),
        },
    };
    let mut conditional_temps = BTreeMap::new();
    conditional_temps.insert(
        "widened".to_owned(),
        PreparedConditionalTempBinding {
            binding_name: "widened".to_owned(),
            condition: PreparedLoopFlowCondition::Simple(PreparedLoopCarryCondition {
                lhs: PreparedCarryCondSource::Current,
                compare: PreparedLoopCompare::Gt,
                rhs: NirExpr::Int(2),
            }),
            then_expr: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            },
            else_expr: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Int(0)),
                rhs: Box::new(NirExpr::Int(1)),
            },
        },
    );

    let prepared = parse_derived_conditional_temp_binding(
        &stmt,
        "value",
        &[],
        &conditional_temps,
        &BTreeSet::new(),
        &BTreeMap::new(),
    )
    .expect("expected remixed derived conditional temp binding");
    assert_eq!(prepared.binding_name, "normalized");
}

#[test]
fn parses_conditional_temp_driven_carry_update_with_loop_state_remix() {
    let stmt = NirStmt::Let {
        name: "acc".to_owned(),
        ty: None,
        value: NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Var("normalized".to_owned())),
        },
    };
    let mut conditional_temps = BTreeMap::new();
    conditional_temps.insert(
        "normalized".to_owned(),
        PreparedConditionalTempBinding {
            binding_name: "normalized".to_owned(),
            condition: PreparedLoopFlowCondition::Simple(PreparedLoopCarryCondition {
                lhs: PreparedCarryCondSource::Current,
                compare: PreparedLoopCompare::Gt,
                rhs: NirExpr::Int(2),
            }),
            then_expr: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                }),
                rhs: Box::new(NirExpr::Var("value".to_owned())),
            },
            else_expr: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Int(0)),
                    rhs: Box::new(NirExpr::Int(1)),
                }),
                rhs: Box::new(NirExpr::Var("value".to_owned())),
            },
        },
    );

    let prepared = parse_conditional_temp_driven_loop_carry_update(
        &stmt,
        "value",
        &[],
        &conditional_temps,
        &BTreeSet::new(),
        &BTreeMap::new(),
    )
    .expect("expected remixed conditional temp carry update");
    assert_eq!(prepared.binding_name, "acc");
}
