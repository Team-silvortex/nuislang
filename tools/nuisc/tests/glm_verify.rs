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
