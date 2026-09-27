use super::*;
use crate::native_session::value_transport::NativeValueLayout;

#[test]
fn scoped_record_layout_and_exact_leaf_kinds_are_checked_before_emission() {
    let kind = CpuCallParameterKind::Record(NativeValueLayout::parse("S{x:i64;y:i64}").unwrap());
    let operand = "$value_record:S{x:i64;y:i64}|$current|$owned_struct_carry:0:seed";
    let overrides = BTreeMap::from([(
        "$owned_struct_carry:0:seed".into(),
        LlvmValueRef::I64("%carry".into()),
    )]);
    let prepared = prepare(operand, &kind, "%iteration", &BTreeMap::new(), &overrides)
        .unwrap()
        .unwrap();
    let mut body = Vec::new();
    prepared.emit(&mut body, &mut 0);
    assert!(body.iter().any(|line| line.contains("i64 %iteration, 0")));
    assert!(body.iter().any(|line| line.contains("i64 %carry, 1")));
    for invalid in [
        operand.replace("S{", "Other{"),
        operand.replace("x:i64;y:i64", "y:i64;x:i64"),
        operand.replace("x:i64", "x:bool"),
        operand.replace("$current", "missing"),
        "record_node".into(),
    ] {
        assert!(prepare(&invalid, &kind, "%iteration", &BTreeMap::new(), &overrides).is_err());
    }
    assert!(prepare(
        operand,
        &kind,
        "%iteration",
        &BTreeMap::new(),
        &BTreeMap::new()
    )
    .is_err());
    let wrong = BTreeMap::from([(
        "$owned_struct_carry:0:seed".into(),
        LlvmValueRef::I32("%carry".into()),
    )]);
    assert!(prepare(operand, &kind, "%iteration", &BTreeMap::new(), &wrong).is_err());
    assert!(leaves(
        operand,
        &CpuCallParameterKind::Scalar(CpuCallScalarKind::I64)
    )
    .is_err());
}

#[test]
fn scoped_call_validates_all_record_arguments_before_packing() {
    let kind = CpuCallParameterKind::Record(NativeValueLayout::parse("S{x:i64}").unwrap());
    let signatures = BTreeMap::from([(
        "helper".into(),
        crate::CpuHelperSignature {
            params: vec![kind.clone(), kind],
            implicit_parameters: vec![],
            mutex_permit_params: vec![None, None],
            ret: CpuCallScalarKind::I64,
            owned_struct_return: true,
            owned_struct_layout: Some(
                yir_core::parse_owned_struct_layout("State{carry0:i64}").unwrap(),
            ),
            native_value_return: None,
            owned_external_buffer_return: None,
        },
    )]);
    let node = yir_core::Node {
        name: "loop".into(),
        resource: "cpu0".into(),
        op: yir_core::Operation::parse(
            "cpu.loop_while_i64_effect",
            [
                "initial",
                "limit",
                "step",
                "lt",
                "add",
                "cpu",
                "scoped_call_i64_carries",
                "4",
                "helper",
                "State{carry0:i64}",
                "$value_record:S{x:i64}|$owned_struct_carry:0:seed",
                "$value_record:S{x:i64}|wrong",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        )
        .unwrap(),
    };
    let registers = BTreeMap::from([("wrong".into(), LlvmValueRef::I32("1".into()))]);
    let overrides = BTreeMap::from([(
        "$owned_struct_carry:0:seed".into(),
        LlvmValueRef::I64("%carry".into()),
    )]);
    let mut body = vec!["entry:".into()];
    let mut next_reg = 73;
    let result = crate::loop_effect_action::begin_loop_effect_action(
        &node,
        5,
        &mut body,
        &registers,
        &BTreeMap::new(),
        &signatures,
        &BTreeMap::new(),
        &overrides,
        "%current",
        &mut next_reg,
    );
    assert!(result.err().unwrap().contains("requires exact i64"));
    assert_eq!(body, ["entry:"]);
    assert_eq!(next_reg, 73);
}
