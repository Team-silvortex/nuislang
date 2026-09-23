use super::*;

fn child(value: i64) -> Box<NirExpr> {
    Box::new(NirExpr::Int(value))
}

fn values(expr: &NirExpr) -> Vec<i64> {
    let mut values = Vec::new();
    walk_child_exprs(expr, &mut |child| {
        let NirExpr::Int(value) = child else {
            panic!("unexpected {child:?}");
        };
        values.push(*value);
    });
    values
}

#[test]
fn expression_walk_visits_conversion_and_owned_object_operands() {
    for wrap in [
        NirExpr::CastI64ToI32,
        NirExpr::CastI32ToI64,
        NirExpr::CastI64ToBool,
        NirExpr::CastBoolToI64,
        NirExpr::CastI64ToF32,
        NirExpr::CastF32ToI64,
        NirExpr::CastI64ToF64,
        NirExpr::CastF64ToI64,
        NirExpr::OwnedObjectSize,
        NirExpr::KernelRelu,
    ] {
        assert_eq!(values(&wrap(child(7))), [7]);
    }
    assert_eq!(
        values(&NirExpr::OwnedObjectReadI64 {
            object: child(1),
            index: child(2)
        }),
        [1, 2]
    );
    assert_eq!(
        values(&NirExpr::CpuExternCallI32 {
            abi: "c".into(),
            interface: None,
            callee: "external".into(),
            args: vec![NirExpr::Int(1), NirExpr::Int(2)],
        }),
        [1, 2]
    );
}

#[test]
fn expression_walk_visits_provider_options_and_nested_backend_operands() {
    assert_eq!(
        values(&NirExpr::DataProviderRequestIngress {
            request_handle: child(1),
            descriptor_table_handle: child(2),
            descriptor_count: child(3),
            provider_key: child(4),
            capability_hash: child(5),
            capsule_token: Some(child(6)),
            input_role_count: None,
            output_role_count: Some(child(8)),
        }),
        [1, 2, 3, 4, 5, 6, 8]
    );
    assert_eq!(
        values(&NirExpr::KernelElementAt {
            input: child(1),
            row: child(2),
            col: child(3)
        }),
        [1, 2, 3]
    );
    assert_eq!(
        values(&NirExpr::KernelMatmul {
            lhs: child(1),
            rhs: child(2)
        }),
        [1, 2]
    );
    assert_eq!(
        values(&NirExpr::ShaderBindSet {
            pipeline: child(1),
            bindings: vec![NirExpr::Int(2), NirExpr::Int(3)]
        }),
        [1, 2, 3]
    );
}
