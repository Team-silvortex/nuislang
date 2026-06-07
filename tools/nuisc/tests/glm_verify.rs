use nuis_semantics::model::{
    nir_glm_profile, NirExpr, NirFunction, NirGlmEffect, NirGlmUseMode, NirGlmValueClass,
    NirModule, NirStmt, NirVisibility,
};
use nuisc::nir_verify::verify_nir_module;

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
fn glm_profile_classifies_borrow_as_res_read() {
    let profile = nir_glm_profile(&NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))))
        .expect("borrow should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Read);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_profile_classifies_move_as_res_own_with_domain_move() {
    let profile = nir_glm_profile(&NirExpr::Move(Box::new(NirExpr::Var("head".to_owned()))))
        .expect("move should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::DomainMove);
}

#[test]
fn glm_profile_classifies_store_value_as_res_write() {
    let profile = nir_glm_profile(&NirExpr::StoreValue {
        target: Box::new(NirExpr::Var("head".to_owned())),
        value: Box::new(NirExpr::Int(77)),
    })
    .expect("store_value should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Write);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_profile_classifies_free_as_lifetime_end() {
    let profile = nir_glm_profile(&NirExpr::Free(Box::new(NirExpr::Var("head".to_owned()))))
        .expect("free should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::LifetimeEnd);
}

#[test]
fn glm_profile_classifies_join_as_res_own() {
    let profile = nir_glm_profile(&NirExpr::CpuJoin(Box::new(NirExpr::Var("task".to_owned()))))
        .expect("join should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_profile_classifies_join_result_as_res_own() {
    let profile = nir_glm_profile(&NirExpr::CpuJoinResult(Box::new(NirExpr::Var(
        "task".to_owned(),
    ))))
    .expect("join_result should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_profile_classifies_cancel_as_res_own_with_domain_move() {
    let profile = nir_glm_profile(&NirExpr::CpuCancel(Box::new(NirExpr::Var(
        "task".to_owned(),
    ))))
    .expect("cancel should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::DomainMove);
}

#[test]
fn glm_profile_classifies_timeout_as_res_own_with_domain_move() {
    let profile = nir_glm_profile(&NirExpr::CpuTimeout {
        task: Box::new(NirExpr::Var("task".to_owned())),
        limit: Box::new(NirExpr::Int(16)),
    })
    .expect("timeout should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::DomainMove);
}

#[test]
fn glm_profile_classifies_task_completed_as_res_read() {
    let profile = nir_glm_profile(&NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var(
        "result".to_owned(),
    ))))
    .expect("task_completed should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Read);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_profile_classifies_task_value_as_res_read() {
    let profile = nir_glm_profile(&NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
        "result".to_owned(),
    ))))
    .expect("task_value should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Read);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_verifier_accepts_borrow_end_then_write_then_free_sequence() {
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
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::StoreValue {
            target: Box::new(NirExpr::Var("head".to_owned())),
            value: Box::new(NirExpr::Int(77)),
        }),
        NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("head".to_owned())))),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_rejects_write_during_active_borrow() {
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
fn glm_verifier_rejects_join_result_after_join_of_same_task() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::CpuJoin(Box::new(NirExpr::Var("task".to_owned())))),
        NirStmt::Expr(NirExpr::CpuJoinResult(Box::new(NirExpr::Var(
            "task".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `task`"));
}

#[test]
fn glm_verifier_rejects_join_result_after_cancel_of_same_task() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::CpuCancel(Box::new(NirExpr::Var(
            "task".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::CpuJoinResult(Box::new(NirExpr::Var(
            "task".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `task`"));
}

#[test]
fn glm_verifier_rejects_join_result_after_timeout_of_same_task() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::CpuTimeout {
            task: Box::new(NirExpr::Var("task".to_owned())),
            limit: Box::new(NirExpr::Int(16)),
        }),
        NirStmt::Expr(NirExpr::CpuJoinResult(Box::new(NirExpr::Var(
            "task".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `task`"));
}

#[test]
fn glm_verifier_accepts_reused_task_result_observation_sequence() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::Var("task".to_owned()))),
        },
        NirStmt::Expr(NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var(
            "result".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
            "result".to_owned(),
        )))),
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_accepts_task_value_inside_completed_branch() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::Var("task".to_owned()))),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("result".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("result".to_owned()),
            )))],
            else_body: vec![],
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_rejects_task_value_inside_timed_out_branch() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::Var("task".to_owned()))),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskTimedOut(Box::new(NirExpr::Var("result".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("result".to_owned()),
            )))],
            else_body: vec![],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("cannot extract task_value from `result` on a non-completed lifecycle path")
    );
}

#[test]
fn glm_verifier_rejects_task_value_inside_cancelled_branch() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::Var("task".to_owned()))),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCancelled(Box::new(NirExpr::Var("result".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("result".to_owned()),
            )))],
            else_body: vec![],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("cannot extract task_value from `result` on a non-completed lifecycle path")
    );
}

#[test]
fn glm_verifier_rejects_use_after_free_in_expr_statements() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("head".to_owned())))),
        NirStmt::Expr(NirExpr::LoadValue(Box::new(NirExpr::Var(
            "head".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}
