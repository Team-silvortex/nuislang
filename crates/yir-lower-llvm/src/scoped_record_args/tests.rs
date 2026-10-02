use super::*;
use crate::native_session::value_transport::NativeValueLayout;

#[test]
fn scoped_record_mixed_captures_require_exact_kinds_before_packing() {
    let layout = "S{b:bool;n:i32;f:f32;d:f64;i:i64}";
    let kind = CpuCallParameterKind::Record(NativeValueLayout::parse(layout).unwrap());
    let operand = format!("$value_record:{layout}|flag|tag|gain|value|$current");
    let values = [
        LlvmValueRef::Bool {
            i1: "%flag".into(),
            i64: "%flag_word".into(),
        },
        LlvmValueRef::I32("%tag".into()),
        LlvmValueRef::F32("%gain".into()),
        LlvmValueRef::F64("%value".into()),
    ];
    let registers = ["flag", "tag", "gain", "value"]
        .into_iter()
        .zip(values.iter().cloned())
        .map(|(name, value)| (name.into(), value))
        .collect::<BTreeMap<_, _>>();
    let prepared = prepare(&operand, &kind, "%index", &registers, &BTreeMap::new())
        .unwrap()
        .unwrap();
    let mut body = Vec::new();
    prepared.emit(&mut body, &mut 0);
    assert!(body.iter().any(|line| line.contains("[5 x i64]")));
    assert!(body
        .iter()
        .any(|line| line.contains("bitcast float %gain to i32")));
    assert!(body
        .iter()
        .any(|line| line.contains("bitcast double %value to i64")));
    for name in registers.keys() {
        for value in &values {
            let mut wrong = registers.clone();
            wrong.insert(name.clone(), value.clone());
            assert_eq!(
                prepare(&operand, &kind, "%index", &wrong, &BTreeMap::new()).is_ok(),
                std::mem::discriminant(&registers[name]) == std::mem::discriminant(value)
            );
        }
        let mut missing = registers.clone();
        missing.remove(name);
        assert!(prepare(&operand, &kind, "%index", &missing, &BTreeMap::new()).is_err());
    }
}

#[test]
fn scoped_readonly_record_actions_validate_before_native_emission() {
    let kind = CpuCallParameterKind::Record(NativeValueLayout::parse("S{x:f64}").unwrap());
    for scalar in [false, true] {
        let mut params = vec![kind.clone()];
        let mut args = "initial limit step lt add cpu scoped_call 0 helper"
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if scalar {
            params.insert(0, CpuCallParameterKind::Scalar(CpuCallScalarKind::I64));
            args[6] = "scoped_call_i64_carry".into();
            args.extend(["seed".into(), "$carry".into()]);
        }
        args.push("$value_record:S{x:f64}|value".into());
        args[7] = (args.len() - 8).to_string();
        let signature = crate::CpuHelperSignature {
            params,
            implicit_parameters: vec![],
            mutex_permit_params: vec![None; if scalar { 2 } else { 1 }],
            ret: CpuCallScalarKind::I64,
            owned_struct_return: false,
            owned_struct_layout: None,
            native_value_return: None,
            owned_external_buffer_return: None,
        };
        let signatures = BTreeMap::from([("helper".into(), signature)]);
        let registers = BTreeMap::from([("value".into(), LlvmValueRef::F64("%value".into()))]);
        let overrides = BTreeMap::from([("$carry".into(), LlvmValueRef::I64("%carry".into()))]);
        for input in [
            "$value_record:S{x:f64}|value",
            "$value_record:S{x:i64}|$owned_struct_carry:0:seed",
            "$value_record:S{x:f32}|value",
            "$value_record:S{x:f64}|missing",
        ] {
            *args.last_mut().unwrap() = input.into();
            let node = yir_core::Node {
                name: "loop".into(),
                resource: "cpu0".into(),
                op: yir_core::Operation::parse("cpu.loop_while_i64_effect", args.clone()).unwrap(),
            };
            let mut body = vec!["entry:".into()];
            let mut reg = 73;
            let result = crate::loop_effect_action::begin_loop_effect_action(
                &node,
                5,
                &mut body,
                &registers,
                &BTreeMap::new(),
                &signatures,
                &BTreeMap::new(),
                &overrides,
                "%index",
                &mut reg,
            );
            if input == "$value_record:S{x:f64}|value" {
                result.unwrap();
                assert!(body
                    .iter()
                    .any(|line| line.contains("call i64 @nuis_fn_helper(")));
                assert!(reg > 73);
            } else {
                assert!(result.is_err());
                assert_eq!(body, ["entry:"]);
                assert_eq!(reg, 73);
            }
        }
    }
}

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
