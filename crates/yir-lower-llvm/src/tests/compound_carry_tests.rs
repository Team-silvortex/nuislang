use super::support::*;

#[test]
fn generic_compound_carries_keep_scalar_kinds_and_short_circuit_blocks() {
    for (kind, ty, instruction) in [
        ("i64", "i64", "icmp"),
        ("f32", "float", "fcmp"),
        ("f64", "double", "fcmp"),
    ] {
        let mut module = module_with_cpu0();
        for (name, value) in [
            ("initial", "0"),
            ("limit", "3"),
            ("step", "1"),
            ("seed", "0"),
            ("threshold", "2"),
        ] {
            push_cpu_node(&mut module, name, &format!("cpu.const_{kind}"), vec![value]);
        }
        push_cpu_node(
            &mut module,
            "loop",
            "cpu.loop_while_scalar_cond_chain",
            vec![
                "initial",
                "limit",
                "step",
                "lt",
                "add",
                "seed",
                "and",
                "current_gt",
                "threshold",
                "or",
                "prev_current_ne",
                "threshold",
                "prev_carry0_eq",
                "seed",
                "add_current",
                "keep",
            ],
        );
        for name in ["initial", "limit", "step", "seed", "threshold"] {
            push_dep(&mut module, name, "loop");
        }
        let llvm = emit_module(&module).unwrap();
        assert!(
            !llvm.contains("deferred lowering for cpu.loop_while_scalar_cond_chain"),
            "{kind}: {llvm}"
        );
        assert!(llvm.contains("carry_predicate_and_rhs"));
        assert!(llvm.contains("carry_predicate_or_rhs"));
        assert!(llvm.contains("phi i1"));
        assert!(llvm.contains(&format!("alloca {ty}")));
        assert!(llvm.contains(&format!(" = {instruction} ")));
        assert!(!llvm.contains(" = and i1 "));
        assert!(!llvm.contains(" = or i1 "));
    }
}

#[test]
fn shared_conditional_parser_depth_is_bounded_before_native_admission() {
    for depth in [32, 256, 257, 10000] {
        let mut args = vec!["seed".to_owned()];
        args.extend(std::iter::repeat_n("and".to_owned(), depth - 1));
        for _ in 0..depth {
            args.extend(["current_lt".into(), "threshold".into()]);
        }
        args.extend(["add_current".into(), "keep".into()]);
        let parsed = yir_domain_cpu::parse_conditional_carries(&args, 0, "bounded", true);
        assert_eq!(parsed.is_ok(), depth <= 256);
        if depth > 256 {
            assert!(parsed.unwrap_err().contains("condition depth limit"));
        }
    }
}
