use super::lower_nir_to_yir_builtin_cpu;
use crate::frontend::parse_nuis_module;

#[test]
fn lowers_mixed_continue_break_return_tree_into_distinct_terminal_effects() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn classify(value: i64) -> i64 {
            while true {
              if value < 0 {
                continue;
              } else if value == 0 {
                break;
              } else {
                return 7;
              }
            }
            return 3;
          }

          fn main() -> i64 {
            return classify(0) + classify(1);
          }
        }
        "#,
    )
    .unwrap();

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| { node.op.module == "cpu" && node.op.instruction == "guard_loop_continue" }));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.module == "cpu" && node.op.instruction == "guard_return"));
    assert!(yir
        .functions
        .iter()
        .any(|function| function.name == "classify" && function.domain == "cpu"));
    assert!(yir.nodes.iter().any(|node| {
        node.op.module == "cpu"
            && node.op.instruction == "call_i64"
            && node
                .op
                .args
                .first()
                .is_some_and(|callee| callee == "classify")
    }));
}

#[test]
fn lowers_state_carrying_loop_into_unbounded_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            loop {
              let value: i64 = value + 1;
              let acc: i64 = acc + value;
              if acc >= 6 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected unbounded post-flow loop node");
    assert_eq!(loop_node.op.args[3], "always");
    assert_eq!(loop_node.op.args[5], "carry0_ge");
    assert_eq!(loop_node.op.args[7], "break");
}

#[test]
fn lowers_invariant_while_let_payload_into_post_flow_chain() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn main() -> i64 {
            let selected: Option = Option.Some(2);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > payload {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "variant_is"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "variant_field"));
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| {
            node.op.module == "cpu" && node.op.instruction == "loop_while_scalar_post_flow_chain"
        })
        .expect("expected pattern-gated post-flow loop node");
    assert_eq!(loop_node.op.args[3], "invariant_true");
    assert_eq!(loop_node.op.args[5], "current_gt");
    assert_eq!(loop_node.op.args[7], "break");
}

#[test]
fn lowers_terminal_while_let_variant_transition() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active(2);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              let selected: Phase = Phase.Done;
              if cursor > payload {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_post_flow_chain")
        .expect("terminal pattern loop");
    assert_eq!(loop_node.op.args[3], "pattern_exit");
    assert!(yir.nodes.iter().any(|node| {
        node.name.starts_with("loop_variant_state_") && node.op.instruction == "select"
    }));
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn lowers_multi_backedge_while_let_variant_rebuild() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn main() -> i64 {
            let selected: Option = Option.Some(2);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              let selected: Option = Option.Some(payload);
              if cursor > 2 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_post_flow_cond_chain")
        .expect("dynamic pattern loop");
    assert_eq!(loop_node.op.args[3], "pattern_carry0");
    assert!(loop_node.op.args.iter().any(|arg| arg == "add_prev_carry1"));
    let (_, payload_contract) =
        yir_core::split_dynamic_pattern_payload_carry_trailer(&loop_node.op.args).unwrap();
    assert_eq!(
        payload_contract.unwrap().slots,
        [yir_core::DynamicPatternPayloadCarrySlot {
            carry_index: 1,
            codec: yir_core::DynamicPatternPayloadCodec::I64,
        }]
    );
    assert!(yir.nodes.iter().any(|node| {
        node.name.starts_with("loop_variant_state_") && node.op.instruction == "select"
    }));
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(llvm_ir.contains("dynamic-pattern-payload-carry-v2 carry1 i64"));
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn lowers_conditional_while_let_variant_transition_across_backedges() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active(3);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              if payload > 1 {
                let selected: Phase = Phase.Active(payload - 1);
              } else {
                let selected: Phase = Phase.Done;
              }
              if cursor > 100 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_post_flow_cond_chain")
        .expect("dynamic variant transition loop");
    assert_eq!(loop_node.op.args[3], "pattern_carry0");
    assert!(loop_node.op.args.iter().any(|arg| arg == "add_invariant"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.name.starts_with("loop_variant_exit_state_")));
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn lowers_previous_pattern_payload_in_structured_flow_control() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active(4);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              if payload > 1 {
                let selected: Phase = Phase.Active(payload - 1);
              } else {
                let selected: Phase = Phase.Done;
              }
              if payload == 3 {
                continue;
              } else if payload == 2 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_post_flow_cond_chain")
        .expect("dynamic pattern flow-control loop");
    assert_eq!(loop_node.op.args[3], "pattern_carry0");
    assert_eq!(
        loop_node
            .op
            .args
            .iter()
            .filter(|arg| arg.as_str() == "prev_carry1_eq")
            .count(),
        2
    );
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn lowers_ordered_multi_field_pattern_payloads_across_backedges() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active {
              value: i64,
              step: i64,
            },
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active { value: 1, step: 1 };
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active { value: payload, step: stride } = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              if payload < 6 {
                let selected: Phase = Phase.Active {
                  step: stride + 1,
                  value: payload + stride,
                };
              } else {
                let selected: Phase = Phase.Done;
              }
              if cursor > 100 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_post_flow_cond_chain")
        .expect("multi-field dynamic pattern loop");
    assert_eq!(loop_node.op.args[3], "pattern_carry0");
    assert!(loop_node.op.args.iter().any(|arg| arg == "add_prev_carry2"));
    assert_eq!(
        loop_node
            .op
            .args
            .iter()
            .filter(|arg| arg.as_str() == "prev_carry1_lt")
            .count(),
        3
    );
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(!llvm_ir.contains("deferred lowering"));
}

#[test]
fn rejects_non_affine_dynamic_while_let_payload_rebuild_precisely() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active(i64),
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active(3);
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Phase.Active(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + payload;
              let selected: Phase = Phase.Active(payload / 2);
              if cursor > 2 {
                break;
              }
            }
            return acc;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("dynamic tag/payload carry contract"),
        "{error}"
    );
    assert!(error.contains("affine payload rebuilds"), "{error}");
}

#[test]
fn admits_bool_dynamic_while_let_payload_through_the_v2_transport() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active { ready: bool },
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active { ready: true };
            let cursor: i64 = 0;
            while let Phase.Active { ready: flag } = selected {
              let cursor: i64 = cursor + 1;
              let selected: Phase = Phase.Active { ready: flag };
              if cursor > 1 {
                break;
              }
            }
            return cursor;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let loop_node = yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "loop_while_scalar_post_flow_cond_chain")
        .expect("dynamic bool pattern loop");
    assert_eq!(loop_node.op.args[3], "pattern_carry0");
    let (_, payload_contract) =
        yir_core::split_dynamic_pattern_payload_carry_trailer(&loop_node.op.args).unwrap();
    assert_eq!(
        payload_contract.unwrap().slots,
        [yir_core::DynamicPatternPayloadCarrySlot {
            carry_index: 1,
            codec: yir_core::DynamicPatternPayloadCodec::BoolAsI64,
        }]
    );
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "cast_bool_to_i64"));
    assert!(yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "cast_i64_to_bool"));
}

#[test]
fn rejects_i32_dynamic_while_let_payload_before_carry_lowering() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active { value: i32 },
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active { value: i32_from_i64(7) };
            let cursor: i64 = 0;
            while let Phase.Active { value: item } = selected {
              let cursor: i64 = cursor + 1;
              let selected: Phase = Phase.Active { value: item };
              if cursor > 1 {
                break;
              }
            }
            return cursor;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("`i64` and `bool` payload carries"),
        "{error}"
    );
    assert!(error.contains("field `value` bound as `item`"), "{error}");
    assert!(error.contains("type `i32`"), "{error}");
    assert!(
        error.contains("typed-scalar payload carry contract"),
        "{error}"
    );
}

#[test]
fn rejects_owned_text_dynamic_while_let_payload_before_carry_lowering() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Phase {
            Done,
            Active { label: String },
          }

          fn main() -> i64 {
            let selected: Phase = Phase.Active { label: "ready" };
            let cursor: i64 = 0;
            while let Phase.Active { label: text } = selected {
              let cursor: i64 = cursor + 1;
              let selected: Phase = Phase.Active { label: text };
              if cursor > 1 {
                break;
              }
            }
            return cursor;
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let error = lower_nir_to_yir_builtin_cpu(&module).unwrap_err();
    assert!(
        error.contains("`i64` and `bool` payload carries"),
        "{error}"
    );
    assert!(error.contains("field `label` bound as `text`"), "{error}");
    assert!(error.contains("type `String`"), "{error}");
    assert!(
        error.contains("GLM-owned payload carry contract"),
        "{error}"
    );
}

#[test]
fn runtime_none_argument_skips_invariant_pattern_loop_in_llvm() {
    let mut module = parse_nuis_module(
        r#"
        mod cpu Main {
          enum Option {
            None,
            Some(i64),
          }

          fn consume(selected: Option) -> i64 {
            let cursor: i64 = 0;
            let acc: i64 = 0;
            while let Option.Some(payload) = selected {
              let cursor: i64 = cursor + 1;
              let acc: i64 = acc + cursor;
              if cursor > payload {
                break;
              }
            }
            return acc;
          }

          fn main() -> i64 {
            return consume(Option.None);
          }
        }
        "#,
    )
    .unwrap();
    crate::optimize::simplify_nir_module(&mut module);

    let yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
    let llvm_ir = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(llvm_ir.contains("zext i1 false to i64"));
    assert!(!llvm_ir.contains("loop_while_scalar_post_flow_chain_cond."));
    assert!(!llvm_ir.contains("deferred lowering"));
}
