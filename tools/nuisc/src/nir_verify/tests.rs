use super::verify_nir_module;
use nuis_semantics::model::{
    NirDataFlowState, NirExpr, NirFunction, NirModule, NirStmt, NirVisibility,
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
            is_async: false,
            generic_params: vec![],
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
    assert!(error.contains("cannot move borrowed pointer"));
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
    assert!(error.contains("alloc_node cannot capture borrowed pointer"));
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
    assert!(error.contains("store_next cannot write borrowed pointer"));
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
