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
fn glm_profile_classifies_thread_join_result_as_res_own() {
    let profile = nir_glm_profile(&NirExpr::CpuThreadJoinResult(Box::new(NirExpr::Var(
        "thread".to_owned(),
    ))))
    .expect("thread_join_result should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Val);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::None);
}

#[test]
fn glm_profile_classifies_mutex_lock_as_res_own_with_domain_move() {
    let profile = nir_glm_profile(&NirExpr::CpuMutexLock(Box::new(NirExpr::Var(
        "lock".to_owned(),
    ))))
    .expect("mutex_lock should have a GLM profile");
    assert_eq!(profile.result_class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses.len(), 1);
    assert_eq!(profile.accesses[0].class, NirGlmValueClass::Res);
    assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
    assert_eq!(profile.effect, NirGlmEffect::DomainMove);
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
fn glm_verifier_accepts_task_result_fanin_staged_through_buffer() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(7)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "seed".to_owned(),
            ty: None,
            value: NirExpr::LoadValue(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("head".to_owned())))),
        NirStmt::Let {
            name: "alpha_result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::CpuSpawn {
                callee: "alpha".to_owned(),
                args: vec![NirExpr::Var("seed".to_owned())],
            })),
        },
        NirStmt::Let {
            name: "beta_result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::CpuSpawn {
                callee: "beta".to_owned(),
                args: vec![NirExpr::Var("seed".to_owned())],
            })),
        },
        NirStmt::Let {
            name: "scratch".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(2)),
                fill: Box::new(NirExpr::Int(0)),
            })),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("alpha_result".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::StoreAt {
                buffer: Box::new(NirExpr::Var("scratch".to_owned())),
                index: Box::new(NirExpr::Int(0)),
                value: Box::new(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
                    "alpha_result".to_owned(),
                )))),
            })],
            else_body: vec![],
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("beta_result".to_owned()))),
            then_body: vec![NirStmt::Expr(NirExpr::StoreAt {
                buffer: Box::new(NirExpr::Var("scratch".to_owned())),
                index: Box::new(NirExpr::Int(1)),
                value: Box::new(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
                    "beta_result".to_owned(),
                )))),
            })],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("scratch".to_owned())),
            index: Box::new(NirExpr::Int(0)),
        }),
        NirStmt::Expr(NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("scratch".to_owned())),
            index: Box::new(NirExpr::Int(1)),
        }),
        NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("scratch".to_owned())))),
    ]);

    verify_nir_module(&module).unwrap();
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
fn glm_verifier_accepts_thread_task_value_inside_completed_branch() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "thread".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuThreadJoinResult(Box::new(NirExpr::Var(
                "thread".to_owned(),
            ))),
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
fn glm_verifier_rejects_thread_join_after_join_result_of_same_thread() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "thread".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::CpuThreadJoinResult(Box::new(NirExpr::Var(
            "thread".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::CpuThreadJoin(Box::new(NirExpr::Var(
            "thread".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `thread`"));
}

#[test]
fn glm_verifier_rejects_second_mutex_lock_of_same_mutex() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "lock".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::CpuMutexLock(Box::new(NirExpr::Var(
            "lock".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::CpuMutexLock(Box::new(NirExpr::Var(
            "lock".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `lock`"));
}

#[test]
fn glm_verifier_rejects_thread_task_value_inside_else_of_completed_branch() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "thread".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuThreadJoinResult(Box::new(NirExpr::Var(
                "thread".to_owned(),
            ))),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("result".to_owned()))),
            then_body: vec![],
            else_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("result".to_owned()),
            )))],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("cannot extract task_value from `result` on a non-completed lifecycle path")
    );
}

#[test]
fn glm_verifier_rejects_mutex_value_after_unlock_of_same_guard() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "guard".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::CpuMutexUnlock(Box::new(NirExpr::Var(
            "guard".to_owned(),
        )))),
        NirStmt::Expr(NirExpr::CpuMutexValue(Box::new(NirExpr::Var(
            "guard".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `guard`"));
}

#[test]
fn glm_verifier_rejects_thread_use_after_if_branch_consumes_it() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "thread".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Expr(NirExpr::CpuThreadJoin(Box::new(
                NirExpr::Var("thread".to_owned()),
            )))],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::CpuThreadJoin(Box::new(NirExpr::Var(
            "thread".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `thread`"));
}

#[test]
fn glm_verifier_rejects_mutex_use_after_if_branch_locks_it() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "lock".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Expr(NirExpr::CpuMutexLock(Box::new(
                NirExpr::Var("lock".to_owned()),
            )))],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::CpuMutexLock(Box::new(NirExpr::Var(
            "lock".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `lock`"));
}

#[test]
fn glm_verifier_accepts_branch_local_thread_consumption_without_post_merge_reuse() {
    let module = module_with_body(vec![NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![
            NirStmt::Let {
                name: "thread".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Expr(NirExpr::CpuThreadJoin(Box::new(NirExpr::Var(
                "thread".to_owned(),
            )))),
        ],
        else_body: vec![
            NirStmt::Let {
                name: "thread".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(11)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Expr(NirExpr::CpuThreadJoinResult(Box::new(NirExpr::Var(
                "thread".to_owned(),
            )))),
        ],
    }]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_accepts_branch_local_mutex_consumption_without_post_merge_reuse() {
    let module = module_with_body(vec![NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body: vec![
            NirStmt::Let {
                name: "lock".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(10)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Expr(NirExpr::CpuMutexLock(Box::new(NirExpr::Var(
                "lock".to_owned(),
            )))),
        ],
        else_body: vec![
            NirStmt::Let {
                name: "guard".to_owned(),
                ty: None,
                value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                    value: Box::new(NirExpr::Int(11)),
                    next: Box::new(NirExpr::Null),
                })),
            },
            NirStmt::Expr(NirExpr::CpuMutexUnlock(Box::new(NirExpr::Var(
                "guard".to_owned(),
            )))),
        ],
    }]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_rejects_guard_use_after_if_branch_unlocks_it() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "guard".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Expr(NirExpr::CpuMutexUnlock(Box::new(
                NirExpr::Var("guard".to_owned()),
            )))],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::CpuMutexValue(Box::new(NirExpr::Var(
            "guard".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `guard`"));
}

#[test]
fn glm_verifier_accepts_guard_read_inside_branch_and_after_merge() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "guard".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Bool(true),
            then_body: vec![NirStmt::Expr(NirExpr::CpuMutexValue(Box::new(
                NirExpr::Var("guard".to_owned()),
            )))],
            else_body: vec![],
        },
        NirStmt::Expr(NirExpr::CpuMutexValue(Box::new(NirExpr::Var(
            "guard".to_owned(),
        )))),
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
fn glm_verifier_accepts_task_value_on_rhs_of_completed_and_condition() {
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
            condition: NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::And,
                lhs: Box::new(NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var(
                    "result".to_owned(),
                )))),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Gt,
                    lhs: Box::new(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
                        "result".to_owned(),
                    )))),
                    rhs: Box::new(NirExpr::Int(0)),
                }),
            },
            then_body: vec![],
            else_body: vec![],
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_rejects_task_value_inside_else_of_completed_branch() {
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
            then_body: vec![],
            else_body: vec![NirStmt::Expr(NirExpr::CpuTaskValue(Box::new(
                NirExpr::Var("result".to_owned()),
            )))],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("cannot extract task_value from `result` on a non-completed lifecycle path")
    );
}

#[test]
fn glm_verifier_rejects_task_value_on_rhs_of_completed_or_condition() {
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
            condition: NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Or,
                lhs: Box::new(NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var(
                    "result".to_owned(),
                )))),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Gt,
                    lhs: Box::new(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
                        "result".to_owned(),
                    )))),
                    rhs: Box::new(NirExpr::Int(0)),
                }),
            },
            then_body: vec![],
            else_body: vec![],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("cannot extract task_value from `result` on a non-completed lifecycle path")
    );
}

#[test]
fn glm_verifier_accepts_task_value_on_rhs_of_completed_and_while_condition() {
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
        NirStmt::While {
            condition: NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::And,
                lhs: Box::new(NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var(
                    "result".to_owned(),
                )))),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Gt,
                    lhs: Box::new(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
                        "result".to_owned(),
                    )))),
                    rhs: Box::new(NirExpr::Int(0)),
                }),
            },
            body: vec![NirStmt::Break],
        },
    ]);

    verify_nir_module(&module).unwrap();
}

#[test]
fn glm_verifier_rejects_task_value_on_rhs_of_completed_or_while_condition() {
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
        NirStmt::While {
            condition: NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::Or,
                lhs: Box::new(NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var(
                    "result".to_owned(),
                )))),
                rhs: Box::new(NirExpr::Binary {
                    op: nuis_semantics::model::NirBinaryOp::Gt,
                    lhs: Box::new(NirExpr::CpuTaskValue(Box::new(NirExpr::Var(
                        "result".to_owned(),
                    )))),
                    rhs: Box::new(NirExpr::Int(0)),
                }),
            },
            body: vec![NirStmt::Break],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(
        error.contains("cannot extract task_value from `result` on a non-completed lifecycle path")
    );
}

#[test]
fn glm_verifier_rejects_use_of_moved_value_on_rhs_of_and_condition() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::And,
                lhs: Box::new(NirExpr::IsNull(Box::new(NirExpr::Move(Box::new(
                    NirExpr::Var("head".to_owned()),
                ))))),
                rhs: Box::new(NirExpr::IsNull(Box::new(NirExpr::Var("head".to_owned())))),
            },
            then_body: vec![],
            else_body: vec![],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn glm_verifier_rejects_use_after_move_in_if_condition() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::IsNull(Box::new(NirExpr::Move(Box::new(NirExpr::Var(
                "head".to_owned(),
            ))))),
            then_body: vec![],
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
fn glm_verifier_rejects_consume_on_rhs_when_lhs_creates_temporary_borrow() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::If {
            condition: NirExpr::Binary {
                op: nuis_semantics::model::NirBinaryOp::And,
                lhs: Box::new(NirExpr::IsNull(Box::new(NirExpr::Borrow(Box::new(
                    NirExpr::Var("head".to_owned()),
                ))))),
                rhs: Box::new(NirExpr::IsNull(Box::new(NirExpr::Move(Box::new(
                    NirExpr::Var("head".to_owned()),
                ))))),
            },
            then_body: vec![],
            else_body: vec![],
        },
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot consume `head` while borrow(s) are active"));
}

#[test]
fn glm_verifier_rejects_use_of_moved_value_on_rhs_of_plain_binary_expr() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::Binary {
            op: nuis_semantics::model::NirBinaryOp::Add,
            lhs: Box::new(NirExpr::LoadValue(Box::new(NirExpr::Move(Box::new(
                NirExpr::Var("head".to_owned()),
            ))))),
            rhs: Box::new(NirExpr::LoadValue(Box::new(NirExpr::Var(
                "head".to_owned(),
            )))),
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn glm_verifier_rejects_use_after_nested_move_expression_statement() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::IsNull(Box::new(NirExpr::Move(Box::new(
            NirExpr::Var("head".to_owned()),
        ))))),
        NirStmt::Expr(NirExpr::LoadValue(Box::new(NirExpr::Var(
            "head".to_owned(),
        )))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn glm_verifier_rejects_use_of_moved_value_in_later_call_argument() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::Call {
            callee: "sum".to_owned(),
            args: vec![
                NirExpr::LoadValue(Box::new(NirExpr::Move(Box::new(NirExpr::Var(
                    "head".to_owned(),
                ))))),
                NirExpr::LoadValue(Box::new(NirExpr::Var("head".to_owned()))),
            ],
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn glm_verifier_rejects_use_of_moved_value_in_later_struct_field() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::StructLiteral {
            type_name: "Pair".to_owned(),
            type_args: vec![],
            fields: vec![
                (
                    "lhs".to_owned(),
                    NirExpr::LoadValue(Box::new(NirExpr::Move(Box::new(NirExpr::Var(
                        "head".to_owned(),
                    ))))),
                ),
                (
                    "rhs".to_owned(),
                    NirExpr::LoadValue(Box::new(NirExpr::Var("head".to_owned()))),
                ),
            ],
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `head`"));
}

#[test]
fn glm_verifier_rejects_consume_in_later_call_argument_after_temporary_borrow() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::Call {
            callee: "sum".to_owned(),
            args: vec![
                NirExpr::IsNull(Box::new(NirExpr::Borrow(Box::new(NirExpr::Var(
                    "head".to_owned(),
                ))))),
                NirExpr::IsNull(Box::new(NirExpr::Move(Box::new(NirExpr::Var(
                    "head".to_owned(),
                ))))),
            ],
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot consume `head` while borrow(s) are active"));
}

#[test]
fn glm_verifier_rejects_consume_in_later_struct_field_after_temporary_borrow() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(10)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Expr(NirExpr::StructLiteral {
            type_name: "Pair".to_owned(),
            type_args: vec![],
            fields: vec![
                (
                    "lhs".to_owned(),
                    NirExpr::IsNull(Box::new(NirExpr::Borrow(Box::new(NirExpr::Var(
                        "head".to_owned(),
                    ))))),
                ),
                (
                    "rhs".to_owned(),
                    NirExpr::IsNull(Box::new(NirExpr::Move(Box::new(NirExpr::Var(
                        "head".to_owned(),
                    ))))),
                ),
            ],
        }),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot consume `head` while borrow(s) are active"));
}

#[test]
fn glm_verifier_rejects_store_at_value_consume_after_buffer_temporary_borrow() {
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
            name: "scratch".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(2)),
                fill: Box::new(NirExpr::Int(0)),
            })),
        },
        NirStmt::Expr(NirExpr::StoreAt {
            buffer: Box::new(NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned())))),
            index: Box::new(NirExpr::Int(0)),
            value: Box::new(NirExpr::Move(Box::new(NirExpr::Var("head".to_owned())))),
        }),
        NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("scratch".to_owned())))),
    ]);

    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("cannot consume `head` while borrow(s) are active"));
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

#[test]
fn glm_verifier_accepts_memory_task_roundtrip_after_borrow_end() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "head".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocNode {
                value: Box::new(NirExpr::Int(5)),
                next: Box::new(NirExpr::Null),
            })),
        },
        NirStmt::Let {
            name: "head_ref".to_owned(),
            ty: None,
            value: NirExpr::Borrow(Box::new(NirExpr::Var("head".to_owned()))),
        },
        NirStmt::Let {
            name: "seed".to_owned(),
            ty: None,
            value: NirExpr::LoadValue(Box::new(NirExpr::Var("head_ref".to_owned()))),
        },
        NirStmt::Expr(NirExpr::BorrowEnd(Box::new(NirExpr::Var(
            "head_ref".to_owned(),
        )))),
        NirStmt::Let {
            name: "scratch".to_owned(),
            ty: None,
            value: NirExpr::Move(Box::new(NirExpr::AllocBuffer {
                len: Box::new(NirExpr::Int(2)),
                fill: Box::new(NirExpr::Int(0)),
            })),
        },
        NirStmt::Expr(NirExpr::StoreAt {
            buffer: Box::new(NirExpr::Var("scratch".to_owned())),
            index: Box::new(NirExpr::Int(0)),
            value: Box::new(NirExpr::Var("seed".to_owned())),
        }),
        NirStmt::Let {
            name: "task".to_owned(),
            ty: None,
            value: NirExpr::CpuSpawn {
                callee: "ping".to_owned(),
                args: vec![NirExpr::Var("seed".to_owned())],
            },
        },
        NirStmt::Let {
            name: "result".to_owned(),
            ty: None,
            value: NirExpr::CpuJoinResult(Box::new(NirExpr::Var("task".to_owned()))),
        },
        NirStmt::If {
            condition: NirExpr::CpuTaskCompleted(Box::new(NirExpr::Var("result".to_owned()))),
            then_body: vec![
                NirStmt::Let {
                    name: "observed".to_owned(),
                    ty: None,
                    value: NirExpr::CpuTaskValue(Box::new(NirExpr::Var("result".to_owned()))),
                },
                NirStmt::Expr(NirExpr::StoreAt {
                    buffer: Box::new(NirExpr::Var("scratch".to_owned())),
                    index: Box::new(NirExpr::Int(1)),
                    value: Box::new(NirExpr::Var("observed".to_owned())),
                }),
                NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("scratch".to_owned())))),
                NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("head".to_owned())))),
            ],
            else_body: vec![
                NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("scratch".to_owned())))),
                NirStmt::Expr(NirExpr::Free(Box::new(NirExpr::Var("head".to_owned())))),
            ],
        },
    ]);

    verify_nir_module(&module).unwrap();
}
