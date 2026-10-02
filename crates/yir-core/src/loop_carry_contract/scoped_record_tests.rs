use super::*;

#[test]
fn scoped_readonly_records_share_dependencies_without_inventing_backedges() {
    for scalar in [false, true] {
        let mut args = "begin end step lt add cpu scoped_call 0 helper"
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if scalar {
            args[6] = "scoped_call_i64_carry".into();
            args.extend(["seed".into(), "$carry".into()]);
        }
        args.push("$value_record:Input{flag:bool;tag:i32;gain:f32;value:f64;index:i64}|flag|tag|gain|value|$current".into());
        args[7] = (args.len() - 8).to_string();
        let dependencies = |args: &[String]| {
            if scalar {
                parse_scoped_i64_carry(args)?.unwrap().dependencies()
            } else {
                parse_scoped_call(args)?.unwrap().dependencies()
            }
        };
        let expected = if scalar {
            vec!["seed", "flag", "tag", "gain", "value"]
        } else {
            vec!["flag", "tag", "gain", "value"]
        };
        assert_eq!(dependencies(&args).unwrap(), expected);
        let op = crate::Operation::parse("cpu.loop_while_i64_effect", args.clone()).unwrap();
        let profile = crate::glm_profile_for_operation(&op);
        assert_eq!(
            profile
                .accesses
                .iter()
                .map(|a| a.input.as_str())
                .collect::<Vec<_>>(),
            [vec!["begin", "end", "step"], expected].concat()
        );
        assert!(profile
            .accesses
            .iter()
            .all(|a| a.class == crate::GlmValueClass::Val && a.mode == crate::GlmUseMode::Read));
        for leaf in [
            "$carry",
            "$owned_struct_carry:0:seed",
            "$unknown",
            "copy_owned:seed",
            "move_owned:seed",
        ] {
            let mut invalid = args.clone();
            *invalid.last_mut().unwrap() = format!("$value_record:Input{{index:i64}}|{leaf}");
            assert!(dependencies(&invalid).is_err(), "{invalid:?}");
        }
        for end in 7..args.len() {
            assert!(dependencies(&args[..end]).is_err());
        }
        for (index, value) in [(5, "shader"), (7, "18446744073709551615"), (8, "$current")] {
            let mut invalid = args.clone();
            invalid[index] = value.into();
            assert!(dependencies(&invalid).is_err());
        }
    }
}

#[test]
fn scoped_plain_owned_captures_keep_resource_glm_modes() {
    let args = "begin end step lt add cpu scoped_call 4 helper $current copy_owned:buffer move_owned:owner"
        .split_whitespace().map(str::to_owned).collect::<Vec<_>>();
    assert_eq!(
        parse_scoped_call(&args)
            .unwrap()
            .unwrap()
            .dependencies()
            .unwrap(),
        ["buffer", "owner"]
    );
    let op = crate::Operation::parse("cpu.loop_while_i64_effect", args).unwrap();
    let profile = crate::glm_profile_for_operation(&op);
    assert_eq!(profile.accesses[3].class, crate::GlmValueClass::Res);
    assert_eq!(profile.accesses[3].mode, crate::GlmUseMode::Read);
    assert_eq!(profile.accesses[4].class, crate::GlmValueClass::Res);
    assert_eq!(profile.accesses[4].mode, crate::GlmUseMode::Own);
    assert_eq!(profile.effect, crate::GlmEffect::DomainMove);
}

fn args(inputs: &[String], explicit: bool) -> Vec<String> {
    let mut args =
        "begin end step lt add cpu scoped_call_i64_carries 0 update State{carry0:i64;carry1:i64}"
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
    if explicit {
        args.extend(super::encode_scoped_i64_seeds(&[
            "first".into(),
            "second".into(),
        ]));
    }
    args.extend_from_slice(inputs);
    args[7] = (args.len() - 8).to_string();
    args
}

fn record() -> String {
    ScopedRecordInput::encode(
        "Pair{right:i64;left:i64}",
        &[
            "$owned_struct_carry:1:second".into(),
            "$owned_struct_carry:0:first".into(),
        ],
    )
    .unwrap()
}

#[test]
fn scoped_record_maps_preserve_slot_order_and_seed_dependencies() {
    for explicit in [false, true] {
        let args = args(&["$current".into(), record()], explicit);
        let call = super::parse_scoped_i64_carries(&args).unwrap().unwrap();
        assert_eq!(call.seeds, ["first", "second"]);
        assert_eq!(call.dependencies().unwrap(), ["second", "first"]);
        let record = ScopedRecordInput::parse(&call.operands[1])
            .unwrap()
            .unwrap();
        assert_eq!(record.layout.fields()[0].0, "right");
        assert_eq!(
            record.operands,
            [
                "$owned_struct_carry:1:second",
                "$owned_struct_carry:0:first"
            ]
        );
        let op = crate::Operation::parse("cpu.loop_while_i64_effect", args).unwrap();
        assert_eq!(
            crate::glm_profile_for_operation(&op)
                .accesses
                .iter()
                .map(|access| access.input.as_str())
                .collect::<Vec<_>>(),
            ["begin", "end", "step", "second", "first"]
        );
    }
}

#[test]
fn scoped_record_maps_reject_duplicate_missing_and_mismatched_seeds() {
    for explicit in [false, true] {
        for inputs in [
            vec![record(), "$owned_struct_carry:1:second".into()],
            vec![record(), record()],
            vec![record().replace("carry:1:second", "carry:2:second")],
            vec![record().replace("carry:1:second", "carry:0:first")],
        ] {
            assert!(super::parse_scoped_i64_carries(&args(&inputs, explicit)).is_err());
        }
    }
    assert!(super::parse_scoped_i64_carries(&args(
        &[record().replace("carry:1:second", "carry:1:wrong")],
        true
    ))
    .is_err());
    assert!(super::parse_scoped_i64_carries(&args(
        &["$value_record:S{x:i64}|$current".into()],
        false
    ))
    .is_err());
    assert!(super::parse_scoped_i64_carries(&args(
        &["$value_record:S{x:i64}|$current".into()],
        true
    ))
    .is_ok());
}

#[test]
fn scoped_record_descriptor_is_bounded_and_resource_free() {
    for count in [1, 7, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}:i64"))
            .collect::<Vec<_>>()
            .join(";");
        let inputs = vec!["capture".into(); count];
        assert_eq!(
            ScopedRecordInput::encode(&format!("State{{{fields}}}"), &inputs).is_ok(),
            count <= 64
        );
    }
    for invalid in [
        "$value_record:",
        "$value_record:S{}|input",
        "$value_record:S{x:i64}|",
        "$value_record:S{x:i64}|one|two",
        "$value_record:S{x:i64;y:i64}|one",
        "$value_record:S{x:i64;x:i64}|one|two",
        "$value_record:S{x:Bytes}|one",
        "$value_record:S{x:bool}|$current",
        "$value_record:S{x:f32}|$owned_struct_carry:0:one",
        "$value_record:S{x:i64}|$carry",
        "$value_record:S{x:i64}|copy_owned:one",
        "$value_record:S{x:i64}|move_owned:one",
        "$value_record:S{x:i64}|$value_record:T{x:i64}|one",
        "$value_record:S{x:i64}|a b",
        "$value_record:S{x:i64}|$owned_struct_carry:0:$current",
    ] {
        assert!(ScopedRecordInput::parse(invalid).is_err(), "{invalid}");
    }
    assert!(ScopedRecordInput::encode("S{x:i64}", &["a|b".into()]).is_err());
    assert!(ScopedRecordInput::parse("ordinary_node").unwrap().is_none());
    assert!(
        ScopedRecordInput::parse("$value_record:S{nested:N{x:i64};i:i64}|captured|$current")
            .unwrap()
            .is_some()
    );
}

#[test]
fn scoped_record_mixed_captures_keep_kinds_and_only_i64_state_mappings() {
    let encoded = "$value_record:Mixed{b:bool;n:i32;f:f32;d:f64;i:i64;c:i64}|flag|tag|gain|value|$current|$owned_struct_carry:0:first";
    let record = ScopedRecordInput::parse(encoded).unwrap().unwrap();
    assert_eq!(
        record
            .layout
            .fields()
            .iter()
            .map(|(_, kind)| *kind)
            .collect::<Vec<_>>(),
        [
            crate::native_scalar_session::ScalarKind::Bool,
            crate::native_scalar_session::ScalarKind::I32,
            crate::native_scalar_session::ScalarKind::F32,
            crate::native_scalar_session::ScalarKind::F64,
            crate::native_scalar_session::ScalarKind::I64,
            crate::native_scalar_session::ScalarKind::I64,
        ]
    );
    let call = args(&[encoded.into()], true);
    assert_eq!(
        super::parse_scoped_i64_carries(&call)
            .unwrap()
            .unwrap()
            .dependencies()
            .unwrap(),
        ["flag", "tag", "gain", "value", "first", "second"]
    );
    for kind in ["bool", "i32", "f32", "f64"] {
        for operand in ["$current", "$owned_struct_carry:0:first"] {
            assert!(
                ScopedRecordInput::encode(&format!("S{{x:{kind}}}"), &[operand.into()]).is_err()
            );
        }
    }
}
