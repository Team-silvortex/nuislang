use super::support::*;

fn assert_materialized(instruction: &str, literal: &str, expected: &str) {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "literal", instruction, vec![literal]);
    push_cpu_const_i64(&mut module, "result", "17");
    push_dep(&mut module, "literal", "result");
    for reversed in [false, true] {
        if reversed {
            module.nodes.reverse();
        }
        let llvm = emit_module(&module).unwrap();
        assert!(llvm.contains(expected), "{literal}: {llvm}");
        assert!(!llvm.contains("fadd float 0.0") && !llvm.contains("fadd double 0.0"));
        assert!(!llvm.contains("deferred lowering"));
    }
}

#[test]
fn float_literals_materialize_f32_bits_without_zero_addition() {
    for literal in [
        "0.0", "-0.0", "1.25", "-2.5", "0.1", "1e-45", "inf", "-inf", "NaN",
    ] {
        let bits = literal.parse::<f32>().unwrap().to_bits();
        assert_materialized(
            "cpu.const_f32",
            literal,
            &format!("bitcast i32 {bits} to float"),
        );
    }
}

#[test]
fn float_literals_materialize_f64_bits_without_zero_addition() {
    for literal in [
        "0.0", "-0.0", "1.25", "-2.5", "0.1", "5e-324", "inf", "-inf", "NaN",
    ] {
        let bits = literal.parse::<f64>().unwrap().to_bits();
        assert_materialized(
            "cpu.const_f64",
            literal,
            &format!("bitcast i64 {bits} to double"),
        );
    }
}

#[test]
fn float_literals_reject_invalid_and_wrong_arity_before_emission() {
    for instruction in ["cpu.const_f32", "cpu.const_f64"] {
        for args in [vec![], vec!["not-a-number"], vec!["0.0", "1.0"]] {
            let mut module = module_with_cpu0();
            push_cpu_node(&mut module, "literal", instruction, args);
            assert!(emit_module(&module).is_err());
        }
    }
}
