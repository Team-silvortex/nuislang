use nuis_semantics::model::{
    nir_glm_profile, NirExpr, NirFunction, NirGlmEffect, NirGlmUseMode, NirGlmValueClass,
    NirModule, NirMutexCapabilityOp, NirStmt, NirVisibility,
};
use nuisc::nir_verify::verify_nir_module;

fn capability(op: NirMutexCapabilityOp, name: &str) -> NirExpr {
    NirExpr::CpuMutexCapability {
        op,
        args: vec![NirExpr::Var(name.to_owned())],
    }
}

fn module_with_body(body: Vec<NirStmt>) -> NirModule {
    NirModule {
        annotations: vec![],
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
fn shared_mutex_glm_distinguishes_read_permit_from_owned_lease_transitions() {
    let permit = NirExpr::CpuMutexCapability {
        op: NirMutexCapabilityOp::Permit,
        args: vec![NirExpr::Var("shared".to_owned()), NirExpr::Int(0)],
    };
    let permit_profile = nir_glm_profile(&permit).expect("permit GLM profile");
    assert_eq!(permit_profile.result_class, NirGlmValueClass::Res);
    assert_eq!(permit_profile.accesses.len(), 2);
    assert_eq!(permit_profile.accesses[0].mode, NirGlmUseMode::Read);
    assert_eq!(permit_profile.effect, NirGlmEffect::None);

    for op in [
        NirMutexCapabilityOp::Share,
        NirMutexCapabilityOp::SharedClose,
        NirMutexCapabilityOp::PermitLock,
        NirMutexCapabilityOp::LeaseUnlock,
    ] {
        let profile = nir_glm_profile(&capability(op, "resource")).expect("owned transition");
        assert_eq!(profile.accesses[0].mode, NirGlmUseMode::Own);
        assert_eq!(profile.effect, NirGlmEffect::DomainMove);
    }
}

#[test]
fn glm_verifier_rejects_permit_issue_after_shared_close() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "shared".to_owned(),
            ty: None,
            value: NirExpr::Int(1),
        },
        NirStmt::Let {
            name: "revoked".to_owned(),
            ty: None,
            value: capability(NirMutexCapabilityOp::SharedClose, "shared"),
        },
        NirStmt::Expr(NirExpr::CpuMutexCapability {
            op: NirMutexCapabilityOp::Permit,
            args: vec![NirExpr::Var("shared".to_owned()), NirExpr::Int(0)],
        }),
    ]);
    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `shared`"));
}

#[test]
fn glm_verifier_rejects_second_lock_of_consumed_mutex_permit() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "permit".to_owned(),
            ty: None,
            value: NirExpr::Int(1),
        },
        NirStmt::Let {
            name: "lease".to_owned(),
            ty: None,
            value: capability(NirMutexCapabilityOp::PermitLock, "permit"),
        },
        NirStmt::Expr(capability(NirMutexCapabilityOp::PermitLock, "permit")),
    ]);
    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `permit`"));
}

#[test]
fn glm_verifier_allows_lease_read_then_rejects_read_after_unlock() {
    let module = module_with_body(vec![
        NirStmt::Let {
            name: "lease".to_owned(),
            ty: None,
            value: NirExpr::Int(1),
        },
        NirStmt::Let {
            name: "value".to_owned(),
            ty: None,
            value: capability(NirMutexCapabilityOp::LeaseValue, "lease"),
        },
        NirStmt::Expr(capability(NirMutexCapabilityOp::LeaseUnlock, "lease")),
        NirStmt::Expr(capability(NirMutexCapabilityOp::LeaseValue, "lease")),
    ]);
    let error = verify_nir_module(&module).unwrap_err();
    assert!(error.contains("use of moved value `lease`"));
}
