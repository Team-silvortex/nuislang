use super::*;
use yir_core::{Operation, ResourceKind};

fn resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn node(name: &str, instruction: &str, args: &[&str]) -> Node {
    Node {
        name: name.to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            instruction,
            args.iter().map(|arg| (*arg).to_owned()).collect(),
        )
        .unwrap(),
    }
}

#[test]
fn f32_word_codec_preserves_bits_without_numeric_conversion() {
    let pack = node("packed", "cpu.pack_f32_word", &["input"]);
    let unpack = node("unpacked", "cpu.unpack_f32_word", &["word"]);
    for bits in [
        0_u32,
        0x8000_0000,
        1,
        0x807f_ffff,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_1234,
        0xffc0_5678,
        0x7f80_0001,
        0xffff_ffff,
    ] {
        let mut state = ExecutionState::default();
        state.bind_value("input", Value::F32(f32::from_bits(bits)));
        let word = CpuMod.execute(&pack, &resource(), &mut state).unwrap();
        assert_eq!(word, Value::Int(i64::from(bits)));
        for high in [0, 0xabcd_1234_0000_0000_u64] {
            state.bind_value("word", Value::Int((high | u64::from(bits)) as i64));
            let Value::F32(value) = CpuMod.execute(&unpack, &resource(), &mut state).unwrap()
            else {
                panic!("expected f32")
            };
            assert_eq!(value.to_bits(), bits);
        }
    }
}

#[test]
fn f32_word_codec_requires_exact_kinds_and_unary_arity() {
    for instruction in ["cpu.pack_f32_word", "cpu.unpack_f32_word"] {
        for args in [vec![], vec!["input", "extra"]] {
            assert!(CpuMod
                .describe(&node("bad", instruction, &args), &resource())
                .is_err());
        }
        let operation = node("value", instruction, &["input"]);
        assert!(CpuMod.describe(&operation, &resource()).is_ok());
        for value in [
            Value::Bool(false),
            Value::I32(0),
            Value::F64(0.0),
            Value::Int(0),
            Value::F32(0.0),
        ] {
            let valid = matches!(
                (&value, instruction),
                (Value::F32(_), "cpu.pack_f32_word") | (Value::Int(_), "cpu.unpack_f32_word")
            );
            let mut state = ExecutionState::default();
            state.bind_value("input", value);
            assert_eq!(
                CpuMod.execute(&operation, &resource(), &mut state).is_ok(),
                valid
            );
        }
    }
}

#[test]
fn f64_word_codec_preserves_all_bits_without_numeric_conversion() {
    let pack = node("packed", "cpu.pack_f64_word", &["input"]);
    let unpack = node("unpacked", "cpu.unpack_f64_word", &["word"]);
    for bits in [
        0_u64,
        0x8000_0000_0000_0000,
        1,
        0x800f_ffff_ffff_ffff,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_1234,
        0xfff8_0000_0000_5678,
        0x7ff0_0000_0000_0001,
        0xffff_ffff_ffff_ffff,
    ] {
        let mut state = ExecutionState::default();
        state.bind_value("input", Value::F64(f64::from_bits(bits)));
        let word = CpuMod.execute(&pack, &resource(), &mut state).unwrap();
        assert_eq!(word, Value::Int(bits as i64));
        state.bind_value("word", word);
        let Value::F64(value) = CpuMod.execute(&unpack, &resource(), &mut state).unwrap() else {
            panic!("expected f64")
        };
        assert_eq!(value.to_bits(), bits);
    }
}

#[test]
fn f64_word_codec_requires_exact_kinds_and_unary_arity() {
    for instruction in ["cpu.pack_f64_word", "cpu.unpack_f64_word"] {
        for args in [vec![], vec!["input", "extra"]] {
            assert!(CpuMod
                .describe(&node("bad", instruction, &args), &resource())
                .is_err());
        }
        let operation = node("value", instruction, &["input"]);
        assert!(CpuMod.describe(&operation, &resource()).is_ok());
        for value in [
            Value::Bool(false),
            Value::I32(0),
            Value::F32(0.0),
            Value::Int(0),
            Value::F64(0.0),
        ] {
            let valid = matches!(
                (&value, instruction),
                (Value::F64(_), "cpu.pack_f64_word") | (Value::Int(_), "cpu.unpack_f64_word")
            );
            let mut state = ExecutionState::default();
            state.bind_value("input", value);
            assert_eq!(
                CpuMod.execute(&operation, &resource(), &mut state).is_ok(),
                valid
            );
        }
    }
}

#[test]
fn logical_operations_preserve_bool_values() {
    let mut state = ExecutionState::default();
    state.bind_value("truthy", Value::Bool(true));
    state.bind_value("falsy", Value::Bool(false));

    assert_eq!(
        CpuMod
            .execute(
                &node("both", "cpu.and", &["truthy", "falsy"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        CpuMod
            .execute(
                &node("either", "cpu.or", &["truthy", "falsy"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        CpuMod
            .execute(
                &node("inverse", "cpu.not", &["falsy"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn bitwise_operations_keep_their_i64_contract() {
    let mut state = ExecutionState::default();
    state.bind_value("lhs", Value::Int(6));
    state.bind_value("rhs", Value::Int(3));

    assert_eq!(
        CpuMod
            .execute(
                &node("masked", "cpu.and", &["lhs", "rhs"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Int(2)
    );
}

#[test]
fn invalid_integer_divisors_report_errors_without_panicking() {
    for instruction in ["cpu.div", "cpu.rem", "cpu.div_i32"] {
        for (lhs, rhs, diagnostic) in [(7, 0, "zero"), (i64::MIN, -1, "overflow")] {
            let mut state = ExecutionState::default();
            if instruction == "cpu.div_i32" {
                state.bind_value("lhs", Value::I32(if rhs == 0 { 7 } else { i32::MIN }));
                state.bind_value("rhs", Value::I32(rhs as i32));
            } else {
                state.bind_value("lhs", Value::Int(lhs));
                state.bind_value("rhs", Value::Int(rhs));
            }
            let error = CpuMod
                .execute(
                    &node("invalid", instruction, &["lhs", "rhs"]),
                    &resource(),
                    &mut state,
                )
                .unwrap_err();
            assert!(error.contains(diagnostic), "{error}");
        }
    }
}

#[test]
fn scalar_integer_arithmetic_wraps_independently_of_host_build_profile() {
    for (instruction, lhs, rhs, expected) in [
        ("cpu.add", i64::MAX, 1, i64::MIN),
        ("cpu.add", i64::MIN, -1, i64::MAX),
        ("cpu.sub", i64::MIN, 1, i64::MAX),
        ("cpu.sub", i64::MAX, -1, i64::MIN),
        ("cpu.mul", i64::MIN, -1, i64::MIN),
        ("cpu.mul", i64::MAX, 2, -2),
        ("cpu.mul", i64::MIN, 2, 0),
    ] {
        let mut state = ExecutionState::default();
        state.bind_value("lhs", Value::Int(lhs));
        state.bind_value("rhs", Value::Int(rhs));
        assert_eq!(
            CpuMod
                .execute(
                    &node("wrap", instruction, &["lhs", "rhs"]),
                    &resource(),
                    &mut state
                )
                .unwrap(),
            Value::Int(expected)
        );
    }
    for (instruction, lhs, rhs, expected) in [
        ("cpu.add_i32", i32::MAX, 1, i32::MIN),
        ("cpu.add_i32", i32::MIN, -1, i32::MAX),
        ("cpu.sub_i32", i32::MIN, 1, i32::MAX),
        ("cpu.sub_i32", i32::MAX, -1, i32::MIN),
        ("cpu.mul_i32", i32::MIN, -1, i32::MIN),
        ("cpu.mul_i32", i32::MAX, 2, -2),
        ("cpu.mul_i32", i32::MIN, 2, 0),
    ] {
        let mut state = ExecutionState::default();
        state.bind_value("lhs", Value::I32(lhs));
        state.bind_value("rhs", Value::I32(rhs));
        assert_eq!(
            CpuMod
                .execute(
                    &node("wrap", instruction, &["lhs", "rhs"]),
                    &resource(),
                    &mut state
                )
                .unwrap(),
            Value::I32(expected)
        );
    }
    let mut state = ExecutionState::default();
    state.bind_value("minimum", Value::Int(i64::MIN));
    assert_eq!(
        CpuMod
            .execute(
                &node("negated", "cpu.neg", &["minimum"]),
                &resource(),
                &mut state
            )
            .unwrap(),
        Value::Int(i64::MIN)
    );
}

#[test]
fn generic_comparisons_feed_logical_operations_as_bool_values() {
    let mut state = ExecutionState::default();
    state.bind_value("low", Value::Int(2));
    state.bind_value("high", Value::Int(7));

    let below = CpuMod
        .execute(
            &node("below", "cpu.lt", &["low", "high"]),
            &resource(),
            &mut state,
        )
        .unwrap();
    state.bind_value("below", below);
    let distinct = CpuMod
        .execute(
            &node("distinct", "cpu.ne", &["low", "high"]),
            &resource(),
            &mut state,
        )
        .unwrap();
    state.bind_value("distinct", distinct);

    assert_eq!(
        CpuMod
            .execute(
                &node("valid", "cpu.and", &["below", "distinct"]),
                &resource(),
                &mut state,
            )
            .unwrap(),
        Value::Bool(true)
    );
}
