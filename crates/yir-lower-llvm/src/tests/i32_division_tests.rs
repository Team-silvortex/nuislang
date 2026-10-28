use super::support::*;

#[test]
fn checked_i32_division_and_remainder_keep_i32_and_guard_before_operation() {
    for (instruction, opcode) in [
        ("cpu.div", "sdiv"),
        ("cpu.div_i32", "sdiv"),
        ("cpu.rem", "srem"),
    ] {
        for divisor in ["0", "-1", "3"] {
            let mut module = module_with_cpu0();
            push_cpu_node(&mut module, "left", "cpu.const_i32", vec!["-2147483648"]);
            push_cpu_node(&mut module, "right", "cpu.const_i32", vec![divisor]);
            push_cpu_node(&mut module, "result", instruction, vec!["left", "right"]);
            push_dep(&mut module, "left", "result");
            push_dep(&mut module, "right", "result");
            let llvm = emit_module(&module).unwrap();
            let operation = llvm.find(&format!("{opcode} i32")).unwrap();
            assert!(!llvm.contains(&format!("{opcode} i64")));
            if divisor != "3" {
                let prefix = &llvm[..operation];
                assert!(prefix.contains("icmp eq i32"));
                assert!(prefix.contains(", -2147483648"));
                assert!(prefix.contains(", -1"));
                assert!(prefix.contains(", 0"));
                assert!(prefix.contains("integer_divisor_invalid"));
                assert!(prefix.contains("call void @llvm.trap()"));
                assert!(prefix.contains("unreachable"));
            }
        }
    }
}

#[test]
fn checked_i32_division_and_remainder_use_distinct_guard_labels() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "left", "cpu.const_i32", vec!["7"]);
    push_cpu_node(&mut module, "right", "cpu.const_i32", vec!["-1"]);
    for (name, instruction) in [("quotient", "cpu.div"), ("remainder", "cpu.rem")] {
        push_cpu_node(&mut module, name, instruction, vec!["left", "right"]);
        push_dep(&mut module, "left", name);
        push_dep(&mut module, "right", name);
    }
    let llvm = emit_module(&module).unwrap();
    let labels = llvm
        .lines()
        .filter(|line| line.ends_with(':') && line.contains("integer_divisor_"))
        .collect::<Vec<_>>();
    assert_eq!(labels.len(), 4);
    assert_eq!(
        labels
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert_eq!(llvm.matches("call void @llvm.trap()").count(), 2);
    assert_eq!(llvm.matches("sdiv i32").count(), 1);
    assert_eq!(llvm.matches("srem i32").count(), 1);
}
