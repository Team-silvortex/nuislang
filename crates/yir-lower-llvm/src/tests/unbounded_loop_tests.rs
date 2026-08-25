use super::support::*;

#[test]
fn unbounded_post_flow_loop_enters_the_body_without_a_limit_compare() {
    let mut module = module_with_cpu0();
    for (name, value) in [
        ("initial", "0"),
        ("unused_limit", "0"),
        ("step", "1"),
        ("control_rhs", "6"),
        ("carry_initial", "0"),
    ] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "loop",
        "cpu.loop_while_scalar_post_flow_chain",
        vec![
            "initial",
            "unused_limit",
            "step",
            "always",
            "add",
            "carry0_ge",
            "control_rhs",
            "break",
            "carry_initial",
            "add_current",
        ],
    );
    for input in [
        "initial",
        "unused_limit",
        "step",
        "control_rhs",
        "carry_initial",
    ] {
        push_dep(&mut module, input, "loop");
    }

    let llvm_ir = emit_module(&module).expect("unbounded post-flow loop should lower");
    let cond_offset = llvm_ir
        .find("\nloop_while_scalar_post_flow_chain_cond.")
        .expect("condition block");
    let body_offset = llvm_ir
        .find("\nloop_while_scalar_post_flow_chain_body.")
        .expect("body block");
    let entry_block = &llvm_ir[cond_offset..body_offset];
    assert!(entry_block.contains("br label %loop_while_scalar_post_flow_chain_body."));
    assert!(!entry_block.contains("icmp"));
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn invariant_pattern_gate_controls_post_flow_loop_entry() {
    let mut module = module_with_cpu0();
    for (name, value) in [
        ("initial", "0"),
        ("pattern_condition", "1"),
        ("step", "1"),
        ("control_rhs", "3"),
        ("carry_initial", "0"),
    ] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "loop",
        "cpu.loop_while_scalar_post_flow_chain",
        vec![
            "initial",
            "pattern_condition",
            "step",
            "invariant_true",
            "add",
            "current_ge",
            "control_rhs",
            "break",
            "carry_initial",
            "add_current",
        ],
    );
    for input in [
        "initial",
        "pattern_condition",
        "step",
        "control_rhs",
        "carry_initial",
    ] {
        push_dep(&mut module, input, "loop");
    }

    let llvm_ir = emit_module(&module).expect("pattern-gated post-flow loop should lower");
    let cond_offset = llvm_ir
        .find("\nloop_while_scalar_post_flow_chain_cond.")
        .expect("condition block");
    let body_offset = llvm_ir
        .find("\nloop_while_scalar_post_flow_chain_body.")
        .expect("body block");
    let entry_block = &llvm_ir[cond_offset..body_offset];
    assert!(entry_block.contains(" = icmp ne i64 "));
    assert!(entry_block.contains(", 0"));
    assert!(entry_block.contains("br i1"));
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn invariant_pattern_gate_controls_conditional_post_flow_loop_entry() {
    let mut module = module_with_cpu0();
    for (name, value) in [
        ("initial", "0"),
        ("pattern_condition", "1"),
        ("step", "1"),
        ("control_rhs", "3"),
        ("carry_initial", "0"),
    ] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "loop",
        "cpu.loop_while_scalar_post_flow_cond_chain",
        vec![
            "initial",
            "pattern_condition",
            "step",
            "invariant_true",
            "add",
            "current_ge",
            "control_rhs",
            "break",
            "carry_initial",
            "always",
            "initial",
            "add_current",
            "add_current",
        ],
    );
    for input in [
        "initial",
        "pattern_condition",
        "step",
        "control_rhs",
        "carry_initial",
    ] {
        push_dep(&mut module, input, "loop");
    }

    let llvm_ir = emit_module(&module).expect("conditional pattern-gated loop should lower");
    let cond_offset = llvm_ir
        .find("\nloop_while_scalar_post_flow_cond_chain_cond.")
        .expect("condition block");
    let body_offset = llvm_ir
        .find("\nloop_while_scalar_post_flow_cond_chain_body.")
        .expect("body block");
    let entry_block = &llvm_ir[cond_offset..body_offset];
    assert!(entry_block.contains(" = icmp ne i64 "));
    assert!(entry_block.contains(", 0"));
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn terminal_pattern_transition_has_no_second_simple_backedge() {
    let mut module = module_with_cpu0();
    push_pattern_exit_loop(&mut module, "cpu.loop_while_scalar_post_flow_chain", false);

    let llvm_ir = emit_module(&module).expect("terminal pattern loop should lower");
    assert_eq!(
        count_occurrences(
            &llvm_ir,
            "br label %loop_while_scalar_post_flow_chain_cond."
        ),
        1
    );
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn terminal_pattern_transition_has_no_second_conditional_backedge() {
    let mut module = module_with_cpu0();
    push_pattern_exit_loop(
        &mut module,
        "cpu.loop_while_scalar_post_flow_cond_chain",
        true,
    );

    let llvm_ir = emit_module(&module).expect("conditional terminal pattern loop should lower");
    assert_eq!(
        count_occurrences(
            &llvm_ir,
            "br label %loop_while_scalar_post_flow_cond_chain_cond."
        ),
        1
    );
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn dynamic_pattern_carry_false_still_produces_loop_fields() {
    let mut module = module_with_cpu0();
    for (name, value) in [
        ("initial", "0"),
        ("unused_limit", "0"),
        ("step", "1"),
        ("control_rhs", "100"),
        ("active", "0"),
        ("payload", "3"),
    ] {
        push_cpu_const_i64(&mut module, name, value);
    }
    push_cpu_node(
        &mut module,
        "loop",
        "cpu.loop_while_scalar_post_flow_cond_chain",
        vec![
            "initial",
            "unused_limit",
            "step",
            "pattern_carry0",
            "add",
            "current_gt",
            "control_rhs",
            "break",
            "active",
            "always",
            "initial",
            "keep",
            "keep",
            "payload",
            "always",
            "initial",
            "keep",
            "keep",
        ],
    );
    push_cpu_node(&mut module, "result", "cpu.field", vec!["loop", "carry0"]);
    push_cpu_node(&mut module, "return", "cpu.return_i64", vec!["result"]);
    push_deps(
        &mut module,
        &[
            ("initial", "loop"),
            ("unused_limit", "loop"),
            ("step", "loop"),
            ("control_rhs", "loop"),
            ("active", "loop"),
            ("payload", "loop"),
            ("loop", "result"),
            ("result", "return"),
        ],
    );

    let llvm_ir = emit_module(&module).expect("dynamic false pattern carry should lower");
    assert!(llvm_ir.contains("loop_while_scalar_post_flow_cond_chain_cond."));
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn false_invariant_pattern_gate_skips_unreachable_payload_projection() {
    let mut module = module_with_cpu0();
    push_false_option_gate_inputs(&mut module);
    push_cpu_node(
        &mut module,
        "loop",
        "cpu.loop_while_scalar_post_flow_chain",
        vec![
            "initial",
            "pattern_condition",
            "step",
            "invariant_true",
            "add",
            "current_gt",
            "wrong_payload",
            "break",
            "carry_initial",
            "add_current",
        ],
    );
    push_loop_result(&mut module);
    push_deps(
        &mut module,
        &[
            ("initial", "loop"),
            ("pattern_condition", "loop"),
            ("step", "loop"),
            ("wrong_payload", "loop"),
            ("carry_initial", "loop"),
        ],
    );

    assert_false_gate_keeps_initial_carry(&emit_module(&module).unwrap());
}

#[test]
fn false_invariant_pattern_gate_skips_conditional_flow_inputs() {
    let mut module = module_with_cpu0();
    push_false_option_gate_inputs(&mut module);
    push_cpu_node(
        &mut module,
        "loop",
        "cpu.loop_while_scalar_post_flow_cond_chain",
        vec![
            "initial",
            "pattern_condition",
            "step",
            "invariant_true",
            "add",
            "current_gt",
            "wrong_payload",
            "break",
            "carry_initial",
            "always",
            "initial",
            "add_current",
            "add_current",
        ],
    );
    push_loop_result(&mut module);
    push_deps(
        &mut module,
        &[
            ("initial", "loop"),
            ("pattern_condition", "loop"),
            ("step", "loop"),
            ("wrong_payload", "loop"),
            ("carry_initial", "loop"),
        ],
    );

    assert_false_gate_keeps_initial_carry(&emit_module(&module).unwrap());
}

fn push_false_option_gate_inputs(module: &mut YirModule) {
    for (name, value) in [("initial", "4"), ("step", "99"), ("carry_initial", "7")] {
        push_cpu_const_i64(module, name, value);
    }
    push_cpu_node(module, "none", "cpu.struct", vec!["Option.None"]);
    push_cpu_node(
        module,
        "pattern_condition",
        "cpu.variant_is",
        vec!["none", "Option.Some"],
    );
    push_cpu_node(
        module,
        "wrong_payload",
        "cpu.variant_field",
        vec!["none", "Option.Some", "value"],
    );
    push_deps(
        module,
        &[("none", "pattern_condition"), ("none", "wrong_payload")],
    );
}

fn push_loop_result(module: &mut YirModule) {
    push_cpu_node(module, "result", "cpu.field", vec!["loop", "carry0"]);
    push_cpu_node(module, "return", "cpu.return_i64", vec!["result"]);
    push_deps(module, &[("loop", "result"), ("result", "return")]);
}

fn assert_false_gate_keeps_initial_carry(llvm_ir: &str) {
    let carry_reg = llvm_ir
        .lines()
        .find(|line| line.contains(" = add i64 0, 7"))
        .and_then(|line| line.trim().split_once(" = ").map(|(reg, _)| reg))
        .expect("carry initial register");
    assert!(llvm_ir.contains(&format!("ret i64 {carry_reg}")));
    assert!(!llvm_ir.contains("post_flow_chain_cond."));
    assert!(!llvm_ir.contains("post_flow_cond_chain_cond."));
    assert!(!llvm_ir.contains("deferred lowering"));
}

fn push_pattern_exit_loop(module: &mut YirModule, instruction: &str, conditional: bool) {
    for (name, value) in [
        ("initial", "0"),
        ("pattern_condition", "1"),
        ("step", "1"),
        ("control_rhs", "3"),
        ("carry_initial", "0"),
    ] {
        push_cpu_const_i64(module, name, value);
    }
    let mut args = vec![
        "initial",
        "pattern_condition",
        "step",
        "pattern_exit",
        "add",
        "current_gt",
        "control_rhs",
        "continue",
        "carry_initial",
    ];
    if conditional {
        args.extend(["always", "initial", "add_current", "add_current"]);
    } else {
        args.push("add_current");
    }
    push_cpu_node(module, "loop", instruction, args);
    for input in [
        "initial",
        "pattern_condition",
        "step",
        "control_rhs",
        "carry_initial",
    ] {
        push_dep(module, input, "loop");
    }
}
