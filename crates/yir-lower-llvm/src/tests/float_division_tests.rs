use super::support::*;
use crate::{emit_loop_numeric_op, CpuLoopScalarKind};

#[test]
fn scalar_float_division_checks_both_zero_signs_before_generic_and_typed_divide() {
    for (constant, ty, typed) in [
        ("cpu.const_f32", "float", "cpu.div_f32"),
        ("cpu.const_f64", "double", "cpu.div_f64"),
    ] {
        for instruction in ["cpu.div", typed] {
            for divisor in ["0.0", "-0.0", "2.0", "NaN"] {
                let mut module = module_with_cpu0();
                push_cpu_node(&mut module, "left", constant, vec!["1.0"]);
                push_cpu_node(&mut module, "right", constant, vec![divisor]);
                push_cpu_node(&mut module, "divided", instruction, vec!["left", "right"]);
                push_dep(&mut module, "left", "divided");
                push_dep(&mut module, "right", "divided");
                let llvm = emit_module(&module).unwrap();
                let check = llvm.find(&format!("fcmp oeq {ty}")).unwrap();
                let divide = llvm.find(&format!("fdiv {ty}")).unwrap();
                assert!(check < divide);
                assert!(llvm[check..divide].contains("float_divisor_invalid_"));
                assert!(llvm[check..divide].contains("call void @llvm.trap()"));
                assert!(llvm[check..divide].contains("unreachable"));
            }
        }
    }
}

#[test]
fn loop_float_division_reuses_checked_zero_guard_with_unique_labels() {
    for kind in [CpuLoopScalarKind::F32, CpuLoopScalarKind::F64] {
        let mut body = Vec::new();
        let mut next = 0;
        for _ in 0..2 {
            emit_loop_numeric_op(&mut body, &mut next, kind, "div", "%left", "%right").unwrap();
        }
        let labels = body
            .iter()
            .filter(|line| line.ends_with(':'))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(labels.len(), 4);
        assert_eq!(
            body.iter().filter(|line| line.contains("fcmp oeq")).count(),
            2
        );
        assert_eq!(body.iter().filter(|line| line.contains("fdiv ")).count(), 2);
        assert_eq!(
            body.iter()
                .filter(|line| line.contains("llvm.trap"))
                .count(),
            2
        );
    }
}
