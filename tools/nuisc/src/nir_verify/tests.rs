use super::{expr_is_fixed_readable_carry_source, verify_nir_module};
use crate::frontend::parse_nuis_module;
use nuis_semantics::model::{
    NirBinaryOp, NirDataFlowState, NirExpr, NirFunction, NirModule, NirStmt, NirVisibility,
};

fn module_with_body(body: Vec<NirStmt>) -> NirModule {
    NirModule {
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![],
        traits: vec![],
        impls: vec![],
        functions: vec![NirFunction {
            name: "main".to_owned(),
            annotations: vec![],
            visibility: NirVisibility::Private,
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: vec![],
            where_bounds: vec![],
            params: vec![],
            return_type: None,
            body,
        }],
    }
}

#[test]
fn explicit_borrow_end_allows_owner_write() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn lowered_match_with_balanced_borrows_allows_owner_write_after_match() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let head: ref Node = move(alloc_node(10, null()));
            match 1 {
              1 => {
                let head_ref: ref Node = borrow(head);
                borrow_end(head_ref);
              }
              _ => {
                let head_ref: ref Node = borrow(head);
                borrow_end(head_ref);
              }
            }
            head.value = 77;
          }
        }
        "#,
    )
    .unwrap();

    verify_nir_module(&module).unwrap();
}

#[test]
fn lowered_match_with_unbalanced_borrows_rejects_owner_write_after_match() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() {
            let head: ref Node = move(alloc_node(10, null()));
            match 1 {
              1 => {
                let head_ref: ref Node = borrow(head);
              }
              _ => {
                let head_ref: ref Node = borrow(head);
              }
            }
            head.value = 77;
          }
        }
        "#,
    )
    .unwrap();

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn fixed_readable_carry_source_helper_accepts_load_value_and_load_at() {
    assert!(expr_is_fixed_readable_carry_source(&NirExpr::LoadValue(
        Box::new(NirExpr::Var("head".to_owned()),)
    )));
    assert!(expr_is_fixed_readable_carry_source(&NirExpr::LoadAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Int(0)),
    }));
}

#[test]
fn fixed_readable_carry_source_helper_rejects_non_read_memory_shapes() {
    assert!(!expr_is_fixed_readable_carry_source(&NirExpr::LoadNext(
        Box::new(NirExpr::Var("head".to_owned()),)
    )));
    assert!(!expr_is_fixed_readable_carry_source(&NirExpr::StoreAt {
        buffer: Box::new(NirExpr::Var("buffer".to_owned())),
        index: Box::new(NirExpr::Int(0)),
        value: Box::new(NirExpr::Int(9)),
    }));
}

#[test]
fn borrowed_load_value_and_load_at_remain_verifier_valid() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::LoadValue(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Let {
            name: "buffer".to_owned(),
            ty: None,
            value: NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(4)),
                fill: Box::new(NirExpr::Int(0)),
            },
        },
        NirStmt::Let {
            name: "buffer_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("buffer".to_owned()))),
        },
        NirStmt::Expr(NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer_ref".to_owned())),
            index: Box::new(NirExpr::Int(1)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn owner_write_while_borrowed_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn store_value_with_borrowed_target_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head_ref".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("store_value(..., target) expects owned address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn owner_write_after_conditional_borrow_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            }],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn owner_use_after_conditional_move_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "taken".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::Var("head".to_owned()))),
            }],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::LoadValue(Box::new(NirExpr::Var(
            "head".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn owner_write_after_branch_ended_borrow_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                "head_ref".to_owned(),
            ))))],
            else_body: vec![NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                "head_ref".to_owned(),
            ))))],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn scalar_binding_created_in_both_branches_is_available_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "x".to_owned(),
                ty: None,
                value: NirExpr::Int(1),
            }],
            else_body: vec![NirStmt::Let {
                name: "x".to_owned(),
                ty: None,
                value: NirExpr::Int(2),
            }],
        },
        NirStmt::Expr(NirExpr::Var("x".to_owned())),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn scalar_binding_created_in_only_one_branch_is_unbound_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "x".to_owned(),
                ty: None,
                value: NirExpr::Int(1),
            }],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::Var("x".to_owned())),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of unbound value `x`"));
}

#[test]
fn branch_local_borrow_alias_created_in_both_branches_can_be_ended_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            }],
            else_body: vec![NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            }],
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn branch_local_borrow_alias_created_in_both_branches_keeps_owner_borrow_active_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            }],
            else_body: vec![NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            }],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn owner_write_after_loop_local_borrow_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![NirStmt::Let {
                name: "head_ref".to_owned(),
                ty: None,
                value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
            }],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn owner_write_after_preloop_borrow_and_loop_borrow_end_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                "head_ref".to_owned(),
            ))))],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn owner_write_after_balanced_loop_local_borrow_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![
                NirStmt::Let {
                    name: "head_ref".to_owned(),
                    ty: None,
                    value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
                },
                NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                    "head_ref".to_owned(),
                )))),
            ],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn loop_local_binding_is_unbound_after_while() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![NirStmt::Let {
                name: "x".to_owned(),
                ty: None,
                value: NirExpr::Int(1),
            }],
        },
        NirStmt::Expr(NirExpr::Var("x".to_owned())),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of unbound value `x`"));
}

#[test]
fn rebinding_can_reference_previous_binding_value() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Int(1),
        },
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            },
        },
        NirStmt::Expr(NirExpr::Var("value".to_owned())),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn loop_carry_rebinding_can_reference_previous_binding_value() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::While {
            condition: NirExpr::Binary {
                op: NirBinaryOp::Lt,
                lhs: Box::new(NirExpr::Var("value".to_owned())),
                rhs: Box::new(NirExpr::Int(3)),
            },
            body: vec![NirStmt::Let {
                name: "value".to_owned(),
                ty: None,
                value: NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("value".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                },
            }],
        },
        NirStmt::Expr(NirExpr::Var("value".to_owned())),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn output_pipe_binding_created_in_both_branches_keeps_pipe_kind_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "pipe".to_owned(),
                ty: None,
                value: NirExpr::DataOutputPipe(Box::new(NirExpr::Int(1))),
            }],
            else_body: vec![NirStmt::Let {
                name: "pipe".to_owned(),
                ty: None,
                value: NirExpr::DataOutputPipe(Box::new(NirExpr::Int(2))),
            }],
        },
        NirStmt::Expr(NirExpr::DataInputPipe(Box::new(NirExpr::Var(
            "pipe".to_owned(),
        )))),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn divergent_data_kinds_across_branches_degrade_pipe_capability_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "pipe".to_owned(),
                ty: None,
                value: NirExpr::DataOutputPipe(Box::new(NirExpr::Int(1))),
            }],
            else_body: vec![NirStmt::Let {
                name: "pipe".to_owned(),
                ty: None,
                value: NirExpr::Int(2),
            }],
        },
        NirStmt::Expr(NirExpr::DataInputPipe(Box::new(NirExpr::Var(
            "pipe".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("data_input_pipe expects output pipe input"));
}

#[test]
fn immutable_window_binding_created_in_both_branches_keeps_window_kind_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "window".to_owned(),
                ty: None,
                value: NirExpr::DataImmutableWindow {
                    input: Box::new(NirExpr::Int(1)),
                    offset: Box::new(NirExpr::Int(0)),
                    len: Box::new(NirExpr::Int(4)),
                },
            }],
            else_body: vec![NirStmt::Let {
                name: "window".to_owned(),
                ty: None,
                value: NirExpr::DataImmutableWindow {
                    input: Box::new(NirExpr::Int(2)),
                    offset: Box::new(NirExpr::Int(0)),
                    len: Box::new(NirExpr::Int(4)),
                },
            }],
        },
        NirStmt::Expr(NirExpr::DataReadWindow {
            window: Box::new(NirExpr::Var("window".to_owned())),
            index: Box::new(NirExpr::Int(0)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn divergent_window_mutability_across_branches_degrades_write_capability_after_if() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Let {
                name: "window".to_owned(),
                ty: None,
                value: NirExpr::DataCopyWindow {
                    input: Box::new(NirExpr::Int(1)),
                    offset: Box::new(NirExpr::Int(0)),
                    len: Box::new(NirExpr::Int(4)),
                },
            }],
            else_body: vec![NirStmt::Let {
                name: "window".to_owned(),
                ty: None,
                value: NirExpr::DataImmutableWindow {
                    input: Box::new(NirExpr::Int(2)),
                    offset: Box::new(NirExpr::Int(0)),
                    len: Box::new(NirExpr::Int(4)),
                },
            }],
        },
        NirStmt::Expr(NirExpr::DataWriteWindow {
            window: Box::new(NirExpr::Var("window".to_owned())),
            index: Box::new(NirExpr::Int(0)),
            value: Box::new(NirExpr::Int(9)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("data_write_window expects mutable window input"));
}

#[test]
fn task_value_after_completed_fact_in_both_branches_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("task".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("task".to_owned()),
            )))],
            else_body: vec![],
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn task_value_after_completed_fact_in_only_one_branch_is_treated_as_unknown() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::If {
            condition: NirExpr::Var("cond".to_owned()),
            then_body: vec![NirStmt::Expr(NirExpr::CpuTaskCompleted(Box::new(
                NirExpr::Var("task".to_owned()),
            )))],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
            "task".to_owned(),
        )))),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn task_value_inside_timed_out_condition_branch_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskTimedOut(Box::new(NirExpr::Var("task".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("task".to_owned()),
            )))],
            else_body: vec![],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot extract task_value from `task`"));
}

#[test]
fn task_value_after_completed_condition_loop_is_treated_as_unknown() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Int(0),
        },
        NirStmt::While {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("task".to_owned()))),
            body: vec![],
        },
        NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
            "task".to_owned(),
        )))),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn owner_write_after_loop_balanced_traversal_borrows_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![
                NirStmt::Let {
                    name: "head_ref".to_owned(),
                    ty: None,
                    value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
                },
                NirStmt::Let {
                    name: "next_ptr".to_owned(),
                    ty: None,
                    value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
                },
                NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                    "next_ptr".to_owned(),
                )))),
                NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                    "head_ref".to_owned(),
                )))),
            ],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn owner_write_after_loop_with_unbalanced_traversal_borrow_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![
                NirStmt::Let {
                    name: "head_ref".to_owned(),
                    ty: None,
                    value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
                },
                NirStmt::Let {
                    name: "next_ptr".to_owned(),
                    ty: None,
                    value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
                },
                NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
                    "next_ptr".to_owned(),
                )))),
            ],
        },
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn free_of_traversal_alias_inside_loop_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![
                NirStmt::Let {
                    name: "head_ref".to_owned(),
                    ty: None,
                    value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
                },
                NirStmt::Let {
                    name: "next_ptr".to_owned(),
                    ty: None,
                    value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
                },
                NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("next_ptr".to_owned())))),
            ],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("free(...) expects owned address"));
    assert!(error.contains("borrowed traversal alias"));
}

#[test]
fn owner_use_after_while_body_move_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "cond".to_owned(),
            ty: None,
            value: NirExpr::Bool(true),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::While {
            condition: NirExpr::Var("cond".to_owned()),
            body: vec![NirStmt::Let {
                name: "taken".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::Var("head".to_owned()))),
            }],
        },
        NirStmt::Expr(NirExpr::LoadValue(Box::new(NirExpr::Var(
            "head".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn load_value_from_borrowed_target_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::LoadValue(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn load_at_from_borrowed_buffer_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "scratch".to_owned(),
            ty: None,
            value: NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(4)),
                fill: Box::new(NirExpr::Int(0)),
            },
        },
        NirStmt::Let {
            name: "scratch_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("scratch".to_owned()))),
        },
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: NirExpr::LoadAt {
                buffer: Box::new(NirExpr::Var("scratch_ref".to_owned())),
                index: Box::new(NirExpr::Int(0)),
            },
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn buffer_len_from_borrowed_buffer_is_allowed() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "scratch".to_owned(),
            ty: None,
            value: NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(4)),
                fill: Box::new(NirExpr::Int(0)),
            },
        },
        NirStmt::Let {
            name: "scratch_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("scratch".to_owned()))),
        },
        NirStmt::Let {
            name: "len".to_owned(),
            ty: None,
            value: NirExpr::BufferLen(Box::new(NirExpr::Var("scratch_ref".to_owned()))),
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn move_of_borrowed_pointer_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "taken".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("move(...) expects owned address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn move_of_loaded_next_from_borrowed_source_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "next_ptr".to_owned(),
            ty: None,
            value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
        NirStmt::Let {
            name: "taken".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::Var("next_ptr".to_owned()))),
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("move(...) expects owned address"));
    assert!(error.contains("borrowed traversal alias"));
}

#[test]
fn free_of_borrowed_pointer_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("head_ref".to_owned())))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("free(...) expects owned address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn alloc_node_with_borrowed_next_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "tail_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("tail".to_owned()))),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail_ref".to_owned())),
            },
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("alloc_node(..., next) requires owned structural address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn store_next_with_borrowed_pointer_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "tail_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("tail".to_owned()))),
        },
        NirStmt::Expr(NirExpr::StoreNext {
            target: Box::new(NirExpr::Var("head".to_owned())),
            next: Box::new(NirExpr::Var("tail_ref".to_owned())),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("store_next(..., next) requires owned structural address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn store_next_with_loaded_next_from_borrowed_source_mentions_traversal_alias() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::Let {
            name: "other".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(99)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "next_ptr".to_owned(),
            ty: None,
            value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
        NirStmt::Expr(NirExpr::StoreNext {
            target: Box::new(NirExpr::Var("other".to_owned())),
            next: Box::new(NirExpr::Var("next_ptr".to_owned())),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("store_next(..., next) requires owned structural address"));
    assert!(error.contains("borrowed traversal alias"));
}

#[test]
fn store_next_with_borrowed_target_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::StoreNext {
            target: Box::new(NirExpr::Var("head_ref".to_owned())),
            next: Box::new(NirExpr::Var("tail".to_owned())),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("store_next(..., target) expects owned address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn store_at_with_borrowed_buffer_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "scratch".to_owned(),
            ty: None,
            value: NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(4)),
                fill: Box::new(NirExpr::Int(0)),
            },
        },
        NirStmt::Let {
            name: "scratch_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("scratch".to_owned()))),
        },
        NirStmt::Expr(NirExpr::StoreAt {
            buffer: Box::new(NirExpr::Var("scratch_ref".to_owned())),
            index: Box::new(NirExpr::Int(0)),
            value: Box::new(NirExpr::Int(7)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("store_at(..., buffer) expects owned address"));
    assert!(error.contains("borrowed address alias"));
}

#[test]
fn borrow_end_without_active_borrow_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot end borrow"));
}

#[test]
fn ending_traversal_alias_does_not_release_direct_owner_borrow() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "next_ptr".to_owned(),
            ty: None,
            value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "next_ptr".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn borrowing_traversal_alias_keeps_owner_borrow_active() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "tail".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(30)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Var("tail".to_owned())),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "next_ptr".to_owned(),
            ty: None,
            value: NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
        NirStmt::Let {
            name: "tail_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("next_ptr".to_owned()))),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot write `head` while borrow(s) are active"));
}

#[test]
fn rebind_of_owner_while_borrowed_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(11)),
                next: Box::new(NirExpr::Null),
            })),
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot rebind `head` while borrow(s) are active"));
}

#[test]
fn rebind_of_borrow_alias_while_active_is_rejected() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Null,
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot rebind borrow alias `head_ref`"));
}

#[test]
fn borrow_alias_can_be_rebound_after_borrow_end() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Null,
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn data_input_pipe_requires_output_pipe_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataInputPipe(Box::new(
        NirExpr::Int(7),
    )))]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("data_input_pipe expects output pipe input"));
}

#[test]
fn data_output_pipe_rejects_nested_pipe() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataOutputPipe(Box::new(
        NirExpr::DataOutputPipe(Box::new(NirExpr::Int(7))),
    )))]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("data_output_pipe cannot wrap nested pipe"));
}

#[test]
fn data_window_rejects_marker_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataCopyWindow {
        input: Box::new(NirExpr::DataMarker("ready".to_owned())),
        offset: Box::new(NirExpr::Int(0)),
        len: Box::new(NirExpr::Int(1)),
    })]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot create nested data window"));
}

#[test]
fn data_window_rejects_nested_window_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataCopyWindow {
        input: Box::new(NirExpr::DataImmutableWindow {
            input: Box::new(NirExpr::Int(7)),
            offset: Box::new(NirExpr::Int(0)),
            len: Box::new(NirExpr::Int(1)),
        }),
        offset: Box::new(NirExpr::Int(0)),
        len: Box::new(NirExpr::Int(1)),
    })]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot create nested data window"));
}

#[test]
fn data_profile_send_rejects_handle_table_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataProfileSendUplink {
        unit: "FabricPlane".to_owned(),
        input: Box::new(NirExpr::DataHandleTable(vec![(
            "host".to_owned(),
            "cpu0".to_owned(),
        )])),
    })]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("data_profile_send cannot wrap illegal window payload"));
}

#[test]
fn data_profile_send_rejects_mutable_window_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataProfileSendUplink {
        unit: "FabricPlane".to_owned(),
        input: Box::new(NirExpr::DataCopyWindow {
            input: Box::new(NirExpr::Int(7)),
            offset: Box::new(NirExpr::Int(0)),
            len: Box::new(NirExpr::Int(1)),
        }),
    })]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("requires immutable window payload"));
}

#[test]
fn data_profile_send_accepts_frozen_window_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataProfileSendUplink {
        unit: "FabricPlane".to_owned(),
        input: Box::new(NirExpr::DataFreezeWindow(Box::new(
            NirExpr::DataCopyWindow {
                input: Box::new(NirExpr::Int(7)),
                offset: Box::new(NirExpr::Int(0)),
                len: Box::new(NirExpr::Int(1)),
            },
        ))),
    })]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn data_write_window_requires_mutable_window_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataWriteWindow {
        window: Box::new(NirExpr::DataImmutableWindow {
            input: Box::new(NirExpr::Int(7)),
            offset: Box::new(NirExpr::Int(0)),
            len: Box::new(NirExpr::Int(1)),
        }),
        index: Box::new(NirExpr::Int(0)),
        value: Box::new(NirExpr::Int(9)),
    })]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("expects mutable window input"));
}

#[test]
fn data_read_window_accepts_immutable_window_source() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataReadWindow {
        window: Box::new(NirExpr::DataImmutableWindow {
            input: Box::new(NirExpr::Int(7)),
            offset: Box::new(NirExpr::Int(0)),
            len: Box::new(NirExpr::Int(1)),
        }),
        index: Box::new(NirExpr::Int(0)),
    })]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn data_read_window_accepts_data_value_of_window_result() {
    let module = module_with_body(vec![NirStmt::Expr(NirExpr::DataReadWindow {
        window: Box::new(NirExpr::DataValue(Box::new(NirExpr::DataResult {
            value: Box::new(NirExpr::DataImmutableWindow {
                input: Box::new(NirExpr::Int(7)),
                offset: Box::new(NirExpr::Int(0)),
                len: Box::new(NirExpr::Int(1)),
            }),
            state: NirDataFlowState::Windowed,
        }))),
        index: Box::new(NirExpr::Int(0)),
    })]);

    verify_nir_module(&module).unwrap();
}
