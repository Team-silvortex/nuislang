use super::support::*;

#[test]
fn emits_guarded_continue_as_a_real_self_backedge() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "condition", "cpu.const_bool", vec!["false"]);
    push_cpu_node(
        &mut module,
        "repeat",
        "cpu.guard_loop_continue",
        vec!["condition"],
    );
    push_cpu_const_i64(&mut module, "later", "7");
    push_dep(&mut module, "condition", "repeat");

    let llvm_ir = emit_module(&module).expect("guarded continue should lower");
    assert!(llvm_ir.contains("guard_loop_continue_repeat."));
    assert!(llvm_ir.contains("guard_loop_continue_cont."));
    let repeat_label = llvm_ir
        .lines()
        .find(|line| line.starts_with("guard_loop_continue_repeat."))
        .expect("repeat label")
        .trim_end_matches(':');
    assert!(llvm_ir.contains(&format!("br label %{repeat_label}")));
    assert!(
        llvm_ir.find("guard_loop_continue_cont.").unwrap()
            < llvm_ir.find("= add i64 0, 7").unwrap()
    );
}

#[test]
fn emits_guarded_print_continue_inside_the_repeating_block() {
    let mut module = module_with_cpu0();
    push_cpu_node(&mut module, "condition", "cpu.const_bool", vec!["false"]);
    push_cpu_const_i64(&mut module, "shown", "9");
    push_cpu_node(
        &mut module,
        "repeat",
        "cpu.guard_loop_print_continue",
        vec!["condition", "shown"],
    );
    push_cpu_const_i64(&mut module, "later", "0");
    push_deps(&mut module, &[("condition", "repeat"), ("shown", "repeat")]);

    let llvm_ir = emit_module(&module).expect("guarded print-continue should lower");
    let repeat_offset = llvm_ir.find("\nguard_loop_print_continue_repeat.").unwrap();
    let print_offset = repeat_offset
        + llvm_ir[repeat_offset..]
            .find("call void @nuis_debug_print_i64")
            .unwrap();
    let cont_offset = llvm_ir.find("\nguard_loop_print_continue_cont.").unwrap();
    assert!(repeat_offset < print_offset);
    assert!(print_offset < cont_offset);
    assert!(!llvm_ir.contains("deferred lowering"));
}
